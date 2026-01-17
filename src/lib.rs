// Include generated Slint code
mod ui {
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}

use slint::ComponentHandle;
use slint::Model;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;
use regex::Regex;

pub mod models;
pub mod api;
pub mod db;
pub mod ai;

use crate::api::JobSearchClient;
use crate::db::Db;
use crate::ui::*;
use crate::models::AdStatus;

use std::sync::mpsc;
use tracing_subscriber::prelude::*;

// Log buffer to keep track of recent logs for the UI
static LOG_SENDER: std::sync::OnceLock<mpsc::Sender<String>> = std::sync::OnceLock::new();

struct SlintLogWriter {
    sender: mpsc::Sender<String>,
}

impl std::io::Write for SlintLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(msg) = String::from_utf8(buf.to_vec()) {
            let _ = self.sender.send(msg);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn setup_logging() -> (Option<tracing_appender::non_blocking::WorkerGuard>, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel();
    let _ = LOG_SENDER.set(tx.clone());

    let slint_writer = SlintLogWriter { sender: tx };
    let mut registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout).with_ansi(true))
        .with(tracing_subscriber::fmt::layer().with_writer(move || slint_writer.sender.clone().into_writer()).with_ansi(false));

    // Only enable file logging on non-Android platforms to avoid permission crashes
    #[cfg(not(target_os = "android"))]
    {
        let log_dir = if let Some(proj_dirs) = directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker") {
            proj_dirs.data_dir().join("logs")
        } else {
            std::path::PathBuf::from("logs")
        };
        let _ = std::fs::create_dir_all(&log_dir);

        let file_appender = tracing_appender::rolling::daily(&log_dir, "jobseeker.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        
        registry.with(tracing_subscriber::fmt::layer().with_writer(non_blocking).with_ansi(false)).init();
        (Some(guard), rx)
    }

    #[cfg(target_os = "android")]
    {
        registry.init();
        (None, rx)
    }
}

// Simple trait to convert Sender to Writer
trait ToWriter {
    fn into_writer(self) -> mpsc_writer::MpscWriter;
}

impl ToWriter for mpsc::Sender<String> {
    fn into_writer(self) -> mpsc_writer::MpscWriter {
        mpsc_writer::MpscWriter { sender: self }
    }
}

mod mpsc_writer {
    use std::sync::mpsc;
    pub struct MpscWriter { pub sender: mpsc::Sender<String> }
    impl std::io::Write for MpscWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if let Ok(msg) = String::from_utf8(buf.to_vec()) {
                let _ = self.sender.send(msg);
            }
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
}

fn get_db_path() -> std::path::PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker") {
        let data_dir = proj_dirs.data_dir();
        if let Err(e) = std::fs::create_dir_all(data_dir) {
            tracing::error!("Failed to create data directory: {}", e);
            return std::path::PathBuf::from("jobseeker.redb");
        }
        data_dir.join("jobseeker.redb")
    } else {
        std::path::PathBuf::from("jobseeker.redb")
    }
}

fn normalize_locations(input: &str) -> String {
    input.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.chars().all(char::is_numeric) {
                JobSearchClient::get_municipality_name(s).unwrap_or_else(|| s.to_string())
            } else {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
                }
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn setup_ui(ui: &App, rt: Arc<Runtime>, db: Arc<Db>, log_rx: mpsc::Receiver<String>) {
    let ui_weak = ui.as_weak();
    
    // Log receiver task
    let ui_weak_log = ui.as_weak();
    std::thread::spawn(move || {
        let mut log_lines: Vec<String> = Vec::new();
        while let Ok(msg) = log_rx.recv() {
            log_lines.push(msg.trim().to_string());
            if log_lines.len() > 100 { log_lines.remove(0); }
            
            let lines_to_show = log_lines.join("\n");
            let ui_weak = ui_weak_log.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_system_logs(lines_to_show.into());
                }
            });
        }
    });

    let api_client = Arc::new(JobSearchClient::new());
    
    // Load settings initially and trigger P1 search
    let db_clone = db.clone();
    let ui_weak_clone = ui_weak.clone();
    let api_client_clone = api_client.clone();
    
    rt.spawn(async move {
        let settings = db_clone.load_settings().await.unwrap_or(Some(Default::default())).unwrap_or_default();
        
        let ui_weak_for_callback = ui_weak_clone.clone();
        let settings_for_callback = settings.clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_for_callback.upgrade() {
                ui.set_settings(AppSettings {
                    keywords: settings_for_callback.keywords.clone().into(),
                    blacklist_keywords: settings_for_callback.blacklist_keywords.clone().into(),
                    locations_p1: normalize_locations(&settings_for_callback.locations_p1).into(),
                    locations_p2: normalize_locations(&settings_for_callback.locations_p2).into(),
                    locations_p3: normalize_locations(&settings_for_callback.locations_p3).into(),
                    my_profile: settings_for_callback.my_profile.clone().into(),
                    ollama_url: settings_for_callback.ollama_url.clone().into(),
                });
                tracing::info!("Loaded settings from DB");
            }
        });

        perform_search(
            api_client_clone, 
            db_clone,
            ui_weak_clone, 
            Some(1), 
            None, 
            settings
        ).await;
    });

    // Callback: Free Search
    let api_client_c = api_client.clone();
    let db_c = db.clone();
    let ui_weak_c = ui_weak.clone();
    let rt_handle = rt.handle().clone();
    
    ui.on_search_pressed(move |query| {
        let api_client = api_client_c.clone();
        let db = db_c.clone();
        let ui_weak = ui_weak_c.clone();
        let query = query.to_string();
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_searching(true);
            ui.set_status_msg("Söker fritt...".into());
        }

        rt_handle.spawn(async move {
            let settings = db.load_settings().await.unwrap_or(Some(Default::default())).unwrap_or_default();
            perform_search(api_client, db, ui_weak, None, Some(query), settings).await;
        });
    });

    // Callback: Prio Search
    let api_client_c = api_client.clone();
    let db_c = db.clone();
    let ui_weak_c = ui_weak.clone();
    let rt_handle = rt.handle().clone();

    ui.on_search_prio(move |prio| {
        let api_client = api_client_c.clone();
        let db = db_c.clone();
        let ui_weak = ui_weak_c.clone();
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_searching(true);
            ui.set_status_msg(format!("Laddar Prio {}...", prio).into());
        }

        rt_handle.spawn(async move {
            let settings = db.load_settings().await.unwrap_or(Some(Default::default())).unwrap_or_default();
            perform_search(api_client, db, ui_weak, Some(prio), None, settings).await;
        });
    });

    // Callback: Job Selected
    let ui_weak_sel = ui.as_weak();
    ui.on_job_selected(move |id| {
        if let Some(_ui) = ui_weak_sel.upgrade() {
            tracing::info!("Valt jobb: {}", id);
        }
    });

    // Callback: Job Action
    let db_clone = db.clone();
    let rt_clone = rt.clone();
    let ui_weak_action = ui.as_weak();
    
    ui.on_job_action(move |id, action| {
        let db = db_clone.clone();
        let id_str = id.to_string();
        let action_str = action.to_string();
        let ui_weak = ui_weak_action.clone();

        let rt_handle = rt_clone.handle().clone();
        rt_handle.spawn(async move {
            if action_str == "open" {
                if let Ok(Some(ad)) = db.get_job_ad(&id_str).await {
                    if let Some(url) = ad.webpage_url {
                        tracing::info!("Opening browser for job {}: {}", id_str, url);
                        let _ = webbrowser::open(&url);
                    }
                }
                return;
            }

            let target_status = match action_str.as_str() {
                "reject" => AdStatus::Rejected,
                "save" => AdStatus::Bookmarked,
                "thumbsup" => AdStatus::ThumbsUp,
                "apply" => AdStatus::Applied,
                _ => return,
            };

            // Toggle logic
            let current_ad = db.get_job_ad(&id_str).await.ok().flatten();
            let current_status = current_ad.and_then(|ad| ad.status);
            
            let new_status = if current_status == Some(target_status) {
                tracing::info!("Toggling status OFF for job {} (was {:?})", id_str, target_status);
                None
            } else {
                tracing::info!("Setting status for job {} to {:?}", id_str, target_status);
                Some(target_status)
            };

            if let Err(e) = db.update_ad_status(&id_str, new_status).await {
                tracing::error!("Failed to update status for {}: {}", id_str, e);
            } else {
                let status_int = match new_status {
                    Some(AdStatus::Rejected) => 1,
                    Some(AdStatus::Bookmarked) => 2,
                    Some(AdStatus::ThumbsUp) => 3,
                    Some(AdStatus::Applied) => 4,
                    Some(AdStatus::New) | None => 0,
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let jobs = ui.get_jobs();
                        let mut vec: Vec<JobEntry> = jobs.iter().collect();
                        
                        if let Some(pos) = vec.iter().position(|j| j.id == id_str) {
                            let mut entry = vec[pos].clone();
                            entry.status = status_int;
                            
                            // If rejected, remove from list if not in a special view
                            if status_int == 1 {
                                vec.remove(pos);
                                tracing::info!("Removed rejected job {} from view", id_str);
                            } else {
                                vec[pos] = entry;
                            }
                            ui.set_jobs(Rc::new(slint::VecModel::from(vec)).into());
                        }
                    }
                });
            }
        });
    });

    // Callback: Copy Text to Clipboard
    ui.on_copy_text(move |text| {
        let text_str = text.to_string();
        #[cfg(not(target_os = "android"))]
        {
            use arboard::Clipboard;
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(text_str) {
                        tracing::error!("Failed to copy to clipboard: {}", e);
                    } else {
                        tracing::info!("Copied to clipboard");
                    }
                }
                Err(e) => tracing::error!("Failed to initialize clipboard: {}", e),
            }
        }
        #[cfg(target_os = "android")]
        {
            tracing::info!("Copy requested on Android (Not yet implemented via JNI): {}", text_str);
        }
    });

    // Callback: Save Settings
    let db_clone = db.clone();
    let rt_clone = rt.clone();
    let ui_weak_save = ui.as_weak();
    ui.on_save_settings(move |ui_settings| {
        let db = db_clone.clone();
        let ui_weak = ui_weak_save.clone();
        
        let settings = crate::models::AppSettings {
            keywords: ui_settings.keywords.to_string(),
            blacklist_keywords: ui_settings.blacklist_keywords.to_string(),
            locations_p1: ui_settings.locations_p1.to_string(),
            locations_p2: ui_settings.locations_p2.to_string(),
            locations_p3: ui_settings.locations_p3.to_string(),
            my_profile: ui_settings.my_profile.to_string(),
            ollama_url: ui_settings.ollama_url.to_string(),
        };
        
        let rt_handle = rt_clone.handle().clone();
        rt_handle.spawn(async move {
            if let Err(e) = db.save_settings(&settings).await {
                tracing::error!("Failed to save settings: {}", e);
            } else {
                tracing::info!("Settings saved successfully");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut s = ui.get_settings();
                        s.locations_p1 = normalize_locations(&s.locations_p1).into();
                        s.locations_p2 = normalize_locations(&s.locations_p2).into();
                        s.locations_p3 = normalize_locations(&s.locations_p3).into();
                        ui.set_settings(s);
                    }
                });
            }
        });
    });
}

// Helper function to handle search logic
async fn perform_search(
    api_client: Arc<JobSearchClient>,
    db: Arc<Db>,
    ui_weak: slint::Weak<App>,
    prio: Option<i32>,
    free_query: Option<String>,
    settings: crate::models::AppSettings
) {
    tracing::info!("--- SEARCH START ---");
    tracing::debug!("Loaded Settings: keywords='{}', p1='{}', p2='{}', p3='{}'", 
        settings.keywords, settings.locations_p1, settings.locations_p2, settings.locations_p3);

    let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
    
    let (raw_query, locations_str) = if let Some(q) = free_query {
        tracing::info!("Type: Free Search, Query: '{}'", q);
        (q, String::new()) 
    } else if let Some(p) = prio {
        let locs = match p {
            1 => &settings.locations_p1,
            2 => &settings.locations_p2,
            3 => &settings.locations_p3,
            _ => &settings.locations_p1,
        };
        tracing::info!("Type: Prio Search, Level: {}, Keywords: '{}', Locations: '{}'", p, settings.keywords, locs);
        (settings.keywords.clone(), locs.clone())
    } else {
        tracing::warn!("Type: Unknown Search");
        (String::new(), String::new())
    };

    let query = raw_query.replace(',', " ");

    if query.trim().is_empty() {
        tracing::warn!("Aborting search: Empty query");
        let ui_weak_local = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_local.upgrade() {
                ui.set_searching(false);
                ui.set_status_msg("Inga sökord angivna.".into());
            }
        });
        return;
    }

    if prio.is_some() && locations_str.trim().is_empty() {
        tracing::warn!("Aborting Prio search: No locations configured for this level");
        let ui_weak_local = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_local.upgrade() {
                ui.set_searching(false);
                ui.set_status_msg(format!("Inga platser konfigurerade för Prio {}.", prio.unwrap()).into());
            }
        });
        return;
    }

    let municipalities = JobSearchClient::parse_locations(&locations_str);
    tracing::info!("Parsed Municipalities: {:?} from input '{}'", municipalities, locations_str);
    
    if prio.is_some() && !locations_str.is_empty() && municipalities.is_empty() {
        tracing::error!("Aborting search: Prio selected with locations, but none could be resolved to IDs");
        let ui_weak_local = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_local.upgrade() {
                ui.set_searching(false);
                ui.set_status_msg("Kunde inte hitta kommunernas ID:n.".into());
            }
        });
        return;
    }

    let result = api_client.search(&query, &municipalities, 100).await;
    
    let blacklist: Vec<String> = settings.blacklist_keywords.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    match result {
        Ok(api_ads) => {
            let mut final_entries = Vec::new();
            
            for ad in api_ads {
                let title = ad.headline.to_lowercase();
                let desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("").to_lowercase();
                
                if !blacklist.is_empty() && blacklist.iter().any(|bad| title.contains(bad) || desc.contains(bad)) {
                    continue; 
                }

                let db_ad_opt = db.get_job_ad(&ad.id).await.ok().flatten();
                
                let display_ad = if let Some(local_ad) = db_ad_opt {
                    local_ad
                } else {
                    let _ = db.save_job_ad(&ad).await;
                    ad
                };

                if display_ad.status == Some(AdStatus::Rejected) {
                    continue;
                }

                let desc_text = display_ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                let clean_desc = re_html.replace_all(desc_text, "").to_string();
                
                let status_int = match display_ad.status {
                    Some(AdStatus::Rejected) => 1,
                    Some(AdStatus::Bookmarked) => 2,
                    Some(AdStatus::ThumbsUp) => 3,
                    Some(AdStatus::Applied) => 4,
                    Some(AdStatus::New) | None => 0,
                };

                final_entries.push(JobEntry {
                    id: display_ad.id.into(),
                    title: display_ad.headline.into(),
                    employer: display_ad.employer.and_then(|e| e.name).unwrap_or_else(|| "Okänd".to_string()).into(),
                    location: display_ad.workplace_address.and_then(|a| a.city).unwrap_or_else(|| "Okänd".to_string()).into(),
                    description: clean_desc.into(),
                    date: display_ad.publication_date.split('T').next().unwrap_or("").into(),
                    rating: display_ad.rating.unwrap_or(0) as i32,
                    status: status_int,
                    status_text: "".into(),
                });
            }

            let count = final_entries.len();
            let ui_weak_success = ui_weak.clone();
            
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak_success.upgrade() {
                    ui.set_searching(false);
                    let model = Rc::new(slint::VecModel::from(final_entries));
                    ui.set_jobs(model.into());
                    ui.set_status_msg(format!("Hittade {} annonser", count).into());
                }
            });
        }
        Err(e) => {
            let ui_weak_error = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak_error.upgrade() {
                    ui.set_searching(false);
                    ui.set_status_msg(format!("Fel: {}", e).into());
                }
            });
        }
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "Rust" fn android_main(app: slint::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    tracing::info!("Starting Jobseeker on Android");
    slint::android::init(app).expect("Failed to initialize Slint on Android");

    let (guard, log_rx) = setup_logging();
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    
    let db_path = get_db_path();
    tracing::info!("Using database path: {:?}", db_path);
    let db = rt.block_on(async {
        Db::new(db_path.to_str().unwrap()).await
    }).expect("Failed to initialize database");
    let db = Arc::new(db);

    let ui = App::new().expect("Failed to create Slint UI");
    
    setup_ui(&ui, rt, db, log_rx);
    
    let _log_guard = guard;
    ui.run().expect("Failed to run Slint UI");
}

pub fn desktop_main() {
    let (guard, log_rx) = setup_logging();
    tracing::info!("Starting Jobseeker on Desktop");
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    
    let db_path = get_db_path();
    tracing::info!("Using database path: {:?}", db_path);
    let db = rt.block_on(async {
        Db::new(db_path.to_str().unwrap()).await
    }).expect("Failed to initialize database");
    let db = Arc::new(db);

    let ui = App::new().expect("Failed to create Slint UI");
    
    setup_ui(&ui, rt, db, log_rx);
    
    let _log_guard = guard;
    ui.run().expect("Failed to run Slint UI");
}