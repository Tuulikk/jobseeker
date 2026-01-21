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
use chrono::{Utc, Datelike};

fn swedish_month_name(month: u32) -> &'static str {
    match month {
        1 => "Januari",
        2 => "Februari",
        3 => "Mars",
        4 => "April",
        5 => "Maj",
        6 => "Juni",
        7 => "Juli",
        8 => "Augusti",
        9 => "September",
        10 => "Oktober",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

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
static LOCAL_LOG_GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> = std::sync::OnceLock::new();

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
        // Primary log directory (platform-specific recommended dir)
        let log_dir = if let Some(proj_dirs) = directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker") {
            proj_dirs.data_dir().join("logs")
        } else {
            std::path::PathBuf::from("logs")
        };
        let log_file = log_dir.join("jobseeker.log");
        tracing::info!("Logging to file: {}", log_file.display());
        let _ = std::fs::create_dir_all(&log_dir);

        // Also ensure a local ./logs folder exists so it's easy to find logs in dev environments
        let local_log_dir = std::path::PathBuf::from("logs");
        let _ = std::fs::create_dir_all(&local_log_dir);
        let local_file = local_log_dir.join("jobseeker.log");
        tracing::info!("Also writing a local copy to: {}", local_file.display());

        // Rolling appender in the platform data dir
        let file_appender = tracing_appender::rolling::daily(&log_dir, "jobseeker.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Rolling appender in local ./logs
        let local_appender = tracing_appender::rolling::daily(&local_log_dir, "jobseeker.log");
        let (local_non_blocking, local_guard) = tracing_appender::non_blocking(local_appender);

        // Keep the local guard alive for the lifetime of the program
        let _ = LOCAL_LOG_GUARD.set(local_guard);

        // Add both file writers to the registry so both files receive logs
        let r = registry
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking).with_ansi(false))
            .with(tracing_subscriber::fmt::layer().with_writer(local_non_blocking).with_ansi(false));
        r.init();
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
    #[cfg(target_os = "android")]
    {
        // On Android, use the internal data directory provided by the system
        let path = std::path::PathBuf::from("/data/data/com.gnawsoftware.jobseeker/files");
        let _ = std::fs::create_dir_all(&path);
        return path.join("jobseeker.redb");
    }

    #[cfg(not(target_os = "android"))]
    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::JobSearchClient;

    #[test]
    fn normalize_locations_resolves_codes_and_titlecases_names() {
        // Numeric code should be resolved to a municipality name if available,
        // and plain names should be title-cased.
        let out = normalize_locations("1283, malmö");
        assert_eq!(out, "Helsingborg, Malmö");
    }

    #[test]
    fn normalize_locations_trims_and_ignores_empty_entries() {
        // Leading/trailing commas and whitespace should be ignored.
        let out = normalize_locations(" , 1283, ,  malmö , ");
        assert_eq!(out, "Helsingborg, Malmö");
    }

    #[test]
    fn normalize_locations_empty_input_returns_empty_string() {
        assert_eq!(normalize_locations(""), "");
    }

    #[test]
    fn parse_locations_resolves_malmo_lund() {
        let parsed = JobSearchClient::parse_locations("malmö, lund");
        assert_eq!(parsed, vec!["1280".to_string(), "1281".to_string()]);
    }
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

    // Expose the local log file path (./logs/jobseeker.log) in the UI so it's easy to open/fetch logs.
    let local_path = std::path::PathBuf::from("logs").join("jobseeker.log");
    let local_path_str = local_path.to_string_lossy().to_string();
    let ui_weak_for_log = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak_for_log.upgrade() {
            ui.set_log_file_path(local_path_str.into());
        }
    });

    // Load settings initially, trigger P1 search and load current month from DB
    let db_clone = db.clone();
    let ui_weak_clone = ui_weak.clone();
    let api_client_clone = api_client.clone();

    rt.spawn(async move {
        let settings = db_clone.load_settings().await.unwrap_or(Some(Default::default())).unwrap_or_default();

        let ui_weak_for_callback = ui_weak_clone.clone();
        let settings_for_callback = settings.clone();
        let settings_for_ui = settings_for_callback.clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_for_callback.upgrade() {
                ui.set_settings(AppSettings {
                    keywords: settings_for_ui.keywords.clone().into(),
                    blacklist_keywords: settings_for_ui.blacklist_keywords.clone().into(),
                    locations_p1: normalize_locations(&settings_for_ui.locations_p1).into(),
                    locations_p2: normalize_locations(&settings_for_ui.locations_p2).into(),
                    locations_p3: normalize_locations(&settings_for_ui.locations_p3).into(),
                    my_profile: settings_for_ui.my_profile.clone().into(),
                    ollama_url: settings_for_ui.ollama_url.clone().into(),
                });
                tracing::info!("Loaded settings from DB");
            }
        });

        // Set initial active month to current month and load jobs from DB for that month
        let now = chrono::Utc::now();
        let month_str = format!("{:04}-{:02}", now.year(), now.month());
        let month_display = format!("{} {}", swedish_month_name(now.month()), now.year());
        if let Some(ui) = ui_weak_clone.upgrade() {
            ui.set_active_month(month_str.clone().into());
            ui.set_active_month_display(month_display.clone().into());
            ui.set_status_msg(format!("Laddar {}...", month_display).into());
        }

        match db_clone.get_filtered_jobs(&[], Some(now.year()), Some(now.month())).await {
            Ok(ads) => {
                let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
                let entries: Vec<JobEntry> = ads.into_iter().map(|ad| {
                    let desc_text = ad.description.as_ref()
                        .and_then(|d| d.text.as_ref())
                        .map(|s| s.as_str()).unwrap_or("");
                    let clean_desc = re_html.replace_all(desc_text, "").to_string();
                    let status_int = match ad.status {
                        Some(AdStatus::Rejected) => 1,
                        Some(AdStatus::Bookmarked) => 2,
                        Some(AdStatus::ThumbsUp) => 3,
                        Some(AdStatus::Applied) => 4,
                        Some(AdStatus::New) | None => 0,
                    };
                    JobEntry {
                        id: ad.id.into(),
                        title: ad.headline.into(),
                        employer: ad.employer.and_then(|e| e.name).unwrap_or_else(|| "Okänd".to_string()).into(),
                        location: ad.workplace_address.and_then(|a| a.city).unwrap_or_else(|| "Okänd".to_string()).into(),
                        description: clean_desc.into(),
                        date: ad.publication_date.split('T').next().unwrap_or("").into(),
                        rating: ad.rating.unwrap_or(0) as i32,
                        status: status_int,
                        status_text: "".into(),
                    }
                }).collect();

                let ui_weak_for_invoke = ui_weak_clone.clone();
                let entries_copy = entries.clone();
                let count = entries_copy.len();
                let month_copy = month_str.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_for_invoke.upgrade() {
                        let model = Rc::new(slint::VecModel::from(entries_copy));
                        ui.set_jobs(model.into());
                        ui.set_status_msg(format!("Laddade {} annonser för {}", count, month_copy).into());
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to load jobs for initial month: {}", e);
                let ui_weak_for_err = ui_weak_clone.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_for_err.upgrade() {
                        ui.set_status_msg("Fel vid laddning av annonser för månaden".into());
                    }
                });
            }
        }

        // Initial priority search (as before)
        perform_search(
            api_client_clone.clone(),
            db_clone.clone(),
            ui_weak_clone.clone(),
            Some(1),
            None,
            settings_for_callback.clone()
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

        tracing::info!("search_prio triggered: P{}", prio);

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_searching(true);
            ui.set_status_msg(format!("Laddar Prio {}...", prio).into());
        }

        rt_handle.spawn(async move {
            let settings = db.load_settings().await.unwrap_or(Some(Default::default())).unwrap_or_default();
            tracing::info!("Loaded settings for prio {}: p1='{}' p2='{}' p3='{}'", prio, settings.locations_p1, settings.locations_p2, settings.locations_p3);
            perform_search(api_client, db, ui_weak, Some(prio), None, settings).await;
        });
    });

    // Callback: Job Selected
    let ui_weak_sel = ui.as_weak();
    ui.on_job_selected(move |id, idx| {
        if let Some(_ui) = ui_weak_sel.upgrade() {
            tracing::info!("Valt jobb: {} (idx={})", id, idx);
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

    // Callback: Month Offset (previous/next month requested from UI)
    let db_clone_month = db.clone();
    let rt_clone_month = rt.clone();
    let ui_weak_month = ui.as_weak();
    ui.on_month_offset(move |offset| {
        tracing::info!("Month offset requested: {}", offset);
        let db = db_clone_month.clone();
        let ui_weak_inner = ui_weak_month.clone();
        let rt_handle = rt_clone_month.handle().clone();

        // Read current month on UI thread and compute new month string
        let current_month = if let Some(ui) = ui_weak_inner.upgrade() {
            ui.get_active_month().to_string()
        } else {
            // fallback to current date
            let now = chrono::Utc::now();
            format!("{:04}-{:02}", now.year(), now.month())
        };

        // compute new year/month
        let mut parts = current_month.split('-');
        let year = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or_else(|| chrono::Utc::now().year());
        let month = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
        let mut new_month = month + offset as i32;
        let mut new_year = year;
        while new_month <= 0 {
            new_month += 12;
            new_year -= 1;
        }
        while new_month > 12 {
            new_month -= 12;
            new_year += 1;
        }
        let new_month_str = format!("{:04}-{:02}", new_year, new_month as u32);
        let new_month_display = format!("{} {}", swedish_month_name(new_month as u32), new_year);

        // Clone explicitly so each closure gets its own copy (avoid move-after-use)
        let new_month_str_ui = new_month_str.clone();
        let new_month_display_ui = new_month_display.clone();
        let new_month_str_err = new_month_str.clone();

        // Update UI immediately
        let ui_for_update = ui_weak_inner.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_for_update.upgrade() {
                ui.set_active_month(new_month_str_ui.clone().into());
                ui.set_active_month_display(new_month_display_ui.clone().into());
                ui.set_status_msg(format!("Laddar {}...", new_month_display_ui).into());
            }
        });

        // Spawn async job to fetch data for the month from DB
        rt_handle.spawn(async move {
            match db.get_filtered_jobs(&[], Some(new_year), Some(new_month as u32)).await {
                Ok(ads) => {
                    let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
                    let entries: Vec<JobEntry> = ads.into_iter().map(|ad| {
                        let desc_text = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                        let clean_desc = re_html.replace_all(desc_text, "").to_string();
                        let status_int = match ad.status {
                            Some(AdStatus::Rejected) => 1,
                            Some(AdStatus::Bookmarked) => 2,
                            Some(AdStatus::ThumbsUp) => 3,
                            Some(AdStatus::Applied) => 4,
                            Some(AdStatus::New) | None => 0,
                        };
                        JobEntry {
                            id: ad.id.into(),
                            title: ad.headline.into(),
                            employer: ad.employer.and_then(|e| e.name).unwrap_or_else(|| "Okänd".to_string()).into(),
                            location: ad.workplace_address.and_then(|a| a.city).unwrap_or_else(|| "Okänd".to_string()).into(),
                            description: clean_desc.into(),
                            date: ad.publication_date.split('T').next().unwrap_or("").into(),
                            rating: ad.rating.unwrap_or(0) as i32,
                            status: status_int,
                            status_text: "".into(),
                        }
                    }).collect();

                    let ui_for_invoke = ui_weak_inner.clone();
                    let entries_copy = entries.clone();
                    let count = entries_copy.len();
                    let month_copy = new_month_str.clone();
                    tracing::info!("DB get_filtered_jobs returned {} ads for month {}", count, month_copy);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_for_invoke.upgrade() {
                            let model = Rc::new(slint::VecModel::from(entries_copy));
                            ui.set_jobs(model.into());
                            ui.set_status_msg(format!("Laddade {} annonser för {}", count, month_copy).into());
                        }
                    });
                }
                Err(e) => {
                tracing::error!("Failed to load jobs for month {}: {}", new_month_str_err, e);
                let ui_for_err = ui_weak_inner.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_for_err.upgrade() {
                        ui.set_status_msg("Fel vid laddning av annonser för månaden".into());
                    }
                });
            }
            }
        });
    });
}

// Helper function to handle search logic
// Tool Definition Header: API Mapping Fix for perform_search
async fn perform_search(
    api_client: Arc<JobSearchClient>,
    db: Arc<Db>,
    ui_weak: slint::Weak<App>,
    prio: Option<i32>,
    free_query: Option<String>,
    settings: crate::models::AppSettings
) {
    // 1. Uppdatera UI initialt (trådsäkert utan att hålla 'ui' över await)
    let ui_start = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_start.upgrade() {
            ui.set_searching(true);
            ui.set_status_msg("Söker...".into());
        }
    });

    let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");

    // 2. Bestäm sökparametrar baserat på dina PRIO-zoner
    let (raw_query, locations_str) = match (free_query, prio) {
        (Some(q), _) => (q, String::new()),
        (None, Some(p)) => {
            let locs = match p {
                1 => &settings.locations_p1,
                2 => &settings.locations_p2,
                3 => &settings.locations_p3,
                _ => &settings.locations_p1,
            };
            // LOGG: Här ser vi om agenten har tömt dina inställningar
            println!("INFO: Söker med Prio P{}. Platser: '{}'", p, locs);
            (settings.keywords.clone(), locs.clone())
        },
        _ => (String::new(), String::new()),
    };

    let query_for_api = raw_query.replace(',', " ");
    let municipalities = JobSearchClient::parse_locations(&locations_str);

    // LOGG: Här ser vi om parse_locations faktiskt lyckas skapa ID-koder
    tracing::info!("Tolkade {} st kommun-ID:n: {:?}", municipalities.len(), municipalities);

    // Visa enkel sammanfattning av vad som skickas i UI (senaste API‑request)
    let ui_for_req = ui_weak.clone();
    let request_info = format!("query='{}' municipalities={:?}", query_for_api, municipalities);
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_for_req.upgrade() {
            ui.set_last_api_request(request_info.into());
        }
    });

    // 3. API-anropet (här sker .await, så ingen 'ui' får finnas i scope)
    let result = api_client.search(&query_for_api, &municipalities, 100).await;

    match result {
        Ok(ads) => {
            tracing::info!("API hittade {} råa annonser (query='{}', municipalities={:?})", ads.len(), query_for_api, municipalities);

            let blacklist: Vec<String> = settings.blacklist_keywords
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            let mut final_jobs = Vec::new();
            for ad in ads {
                let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_deref()).unwrap_or("");
                let clean_description = re_html.replace_all(raw_desc, "").into_owned();

                let employer_name = ad.employer.as_ref().and_then(|e| e.name.as_deref()).unwrap_or("Ospecificerad");
                let city = ad.workplace_address.as_ref().and_then(|a| a.city.as_deref()).unwrap_or("Ospecificerad");
                let pub_date = ad.publication_date.clone();

                let desc_lc = clean_description.to_lowercase();
                let title_lc = ad.headline.to_lowercase();

                let is_blacklisted = blacklist.iter().any(|word| {
                    title_lc.contains(word) || desc_lc.contains(word)
                });

                if !is_blacklisted {
                    // Kontrollera status i databasen (också asynkront)
                    let db_status = if let Ok(Some(existing)) = db.get_job_ad(&ad.id).await {
                        // Mappa AdStatus enum till i32 för Slint
                        match existing.status {
                            Some(crate::models::AdStatus::New) => 0,
                            Some(crate::models::AdStatus::Rejected) => 1,
                            Some(crate::models::AdStatus::Bookmarked) => 2,
                            Some(crate::models::AdStatus::ThumbsUp) => 3,
                            Some(crate::models::AdStatus::Applied) => 4,
                            None => 0,
                        }
                    } else {
                        0
                    };

                    final_jobs.push(JobEntry {
                        id: ad.id.into(),
                        title: ad.headline.into(),
                        employer: employer_name.to_string().into(),
                        location: city.to_string().into(),
                        description: clean_description.into(),
                        date: pub_date.into(),
                        status: db_status, // Nu matchar typerna (i32)
                        status_text: "Ny".into(),
                        rating: 0,
                    });
                }
            }

            // 4. Skicka tillbaka resultatet till UI-tråden
            let ui_final = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_final.upgrade() {
                    let count = final_jobs.len();
                    let job_model = std::rc::Rc::new(slint::VecModel::from(final_jobs));
                    ui.set_jobs(job_model.into());
                    ui.set_status_msg(format!("Hittade {} annonser", count).into());
                    ui.set_searching(false);
                }
            });
        },
        Err(e) => {
            println!("ERROR: API-anrop misslyckades: {:?}", e);
            let ui_err = ui_weak.clone();
            let err_msg = e.to_string();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_err.upgrade() {
                    ui.set_status_msg(format!("Fel: {}", err_msg).into());
                    ui.set_searching(false);
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
