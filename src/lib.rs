// Include generated Slint code
mod ui {
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}

use slint::ComponentHandle;
use slint::Model; // Ensure Model trait is imported for .iter()
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

fn setup_ui(ui: &App, rt: Arc<Runtime>, db: Arc<Db>) {
    let ui_weak = ui.as_weak();
    let api_client = Arc::new(JobSearchClient::new());
    
    // Load settings initially and trigger P1 search
    let db_clone = db.clone();
    let ui_weak_clone = ui_weak.clone();
    let api_client_clone = api_client.clone();
    let _rt_handle = rt.handle().clone(); 
    
    rt.spawn(async move {
        let settings = db_clone.load_settings().await.unwrap_or(Some(Default::default())).unwrap_or_default();
        
        let ui_weak_for_callback = ui_weak_clone.clone();
        let settings_for_callback = settings.clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_for_callback.upgrade() {
                ui.set_settings(AppSettings {
                    keywords: settings_for_callback.keywords.clone().into(),
                    blacklist_keywords: settings_for_callback.blacklist_keywords.clone().into(),
                    locations_p1: settings_for_callback.locations_p1.clone().into(),
                    locations_p2: settings_for_callback.locations_p2.clone().into(),
                    locations_p3: settings_for_callback.locations_p3.clone().into(),
                });
                tracing::info!("Loaded settings from DB");
            }
        });

        // Auto-search Prio 1
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
    let ui_weak = ui.as_weak();
    ui.on_job_selected(move |id| {
        if let Some(_ui) = ui_weak.upgrade() {
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

        let _rt_handle = rt_clone.handle().clone();
        rt_clone.spawn(async move {
            let new_status = match action_str.as_str() {
                "reject" => AdStatus::Rejected,
                "save" => AdStatus::Bookmarked,
                "thumbsup" => AdStatus::ThumbsUp,
                "apply" => AdStatus::Applied,
                _ => return,
            };

            if let Err(e) = db.update_ad_status(&id_str, new_status).await {
                tracing::error!("Failed to update status: {}", e);
            } else {
                tracing::info!("Updated job {} to {:?}", id_str, new_status);
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let jobs = ui.get_jobs();
                        // Create a new vector to modify
                        let mut vec: Vec<JobEntry> = jobs.iter().collect();
                        
                        if let Some(pos) = vec.iter().position(|j| j.id == id_str) {
                            let mut entry = vec[pos].clone();
                            entry.status = new_status as i32;
                            
                            if new_status == AdStatus::Rejected {
                                vec.remove(pos);
                            } else {
                                vec[pos] = entry;
                            }
                            
                            // Create a new ModelRc from the vector
                            let new_model = Rc::new(slint::VecModel::from(vec));
                            ui.set_jobs(new_model.into());
                        }
                    }
                });
            }
        });
    });

    // Callback: Save Settings
    let db_clone = db.clone();
    let rt_clone = rt.clone();
    ui.on_save_settings(move |ui_settings| {
        let db = db_clone.clone();
        let settings = crate::models::AppSettings {
            keywords: ui_settings.keywords.to_string(),
            blacklist_keywords: ui_settings.blacklist_keywords.to_string(),
            locations_p1: ui_settings.locations_p1.to_string(),
            locations_p2: ui_settings.locations_p2.to_string(),
            locations_p3: ui_settings.locations_p3.to_string(),
            ..Default::default()
        };
        
        let _rt_handle = rt_clone.handle().clone();
        rt_clone.spawn(async move {
            if let Err(e) = db.save_settings(&settings).await {
                tracing::error!("Failed to save settings: {}", e);
            } else {
                tracing::info!("Settings saved successfully");
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
    let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
    
    // Determine query and locations
    let (query, locations_str) = if let Some(q) = free_query {
        (q, String::new()) 
    } else if let Some(p) = prio {
        let locs = match p {
            1 => &settings.locations_p1,
            2 => &settings.locations_p2,
            3 => &settings.locations_p3,
            _ => &settings.locations_p1,
        };
        (settings.keywords.clone(), locs.clone())
    } else {
        (String::new(), String::new())
    };

    if query.is_empty() {
        let ui_weak_local = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak_local.upgrade() {
                ui.set_searching(false);
                ui.set_status_msg("Inga sökord angivna.".into());
            }
        });
        return;
    }

    let municipalities = JobSearchClient::parse_locations(&locations_str);
    tracing::info!("Searching: query='{}', locs={:?}", query, municipalities);

    let result = api_client.search(&query, &municipalities, 100).await;
    
    let blacklist: Vec<String> = settings.blacklist_keywords.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    // Only for hiding the spinner immediately if error occurs later logic handles update
    // Actually better to just do it in the result block
    
    match result {
        Ok(api_ads) => {
            let mut final_entries = Vec::new();
            
            for ad in api_ads {
                // 1. Check blacklist
                let title = ad.headline.to_lowercase();
                let desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("").to_lowercase();
                
                if !blacklist.is_empty() && blacklist.iter().any(|bad| title.contains(bad) || desc.contains(bad)) {
                    continue; 
                }

                // 2. Check/Save DB
                let db_ad_opt = db.get_job_ad(&ad.id).await.ok().flatten();
                
                let display_ad = if let Some(local_ad) = db_ad_opt {
                    local_ad
                } else {
                    let _ = db.save_job_ad(&ad).await;
                    ad
                };

                // 3. Filter Rejected
                if display_ad.status == Some(AdStatus::Rejected) {
                    continue;
                }

                // 4. Convert to UI struct
                let desc_text = display_ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                let clean_desc = re_html.replace_all(desc_text, "").to_string();
                
                let status_int = match display_ad.status {
                    Some(AdStatus::Rejected) => 1,
                    Some(AdStatus::Bookmarked) => 2,
                    Some(AdStatus::ThumbsUp) => 3,
                    Some(AdStatus::Applied) => 4,
                    _ => 0,
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

    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    
    // Initialize DB
    let db_path = get_db_path();
    tracing::info!("Using database path: {:?}", db_path);
    let db = rt.block_on(async {
        Db::new(db_path.to_str().unwrap()).await
    }).expect("Failed to initialize database");
    let db = Arc::new(db);

    let ui = App::new().expect("Failed to create Slint UI");
    
    setup_ui(&ui, rt, db);
    
    ui.run().expect("Failed to run Slint UI");
}

pub fn desktop_main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Jobseeker on Desktop");
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    
    // Initialize DB
    let db_path = get_db_path();
    tracing::info!("Using database path: {:?}", db_path);
    let db = rt.block_on(async {
        Db::new(db_path.to_str().unwrap()).await
    }).expect("Failed to initialize database");
    let db = Arc::new(db);

    let ui = App::new().expect("Failed to create Slint UI");
    
    setup_ui(&ui, rt, db);
    
    ui.run().expect("Failed to run Slint UI");
}
