// Include generated Slint code
mod ui {
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}

use slint::ComponentHandle;
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

        let rt_handle = rt_clone.handle().clone();
        rt_handle.spawn(async move {
            let new_status = match action_str.as_str() {
                "reject" => AdStatus::Rejected,
                "save" => {
                    // Toggle logic could be here, but simpler to just set for now or check DB
                    AdStatus::Bookmarked
                },
                "thumbsup" => AdStatus::ThumbsUp,
                "apply" => AdStatus::Applied,
                _ => return,
            };

            // TODO: Implement toggle logic by checking current status if needed.
            // For now, enforce the new status.
            if let Err(e) = db.update_ad_status(&id_str, new_status).await {
                tracing::error!("Failed to update status: {}", e);
            } else {
                tracing::info!("Updated job {} to {:?}", id_str, new_status);
                
                // Update UI Model directly to reflect change immediately
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let jobs = ui.get_jobs();
                        // Find the row and update it
                        // This is a bit inefficient in Slint VecModel, usually we reload or map.
                        // But we can iterate rows.
                        // Since we can't easily modify a ModelRc in place efficiently without full reload in this simple setup,
                        // we might just re-trigger a local filter or just update visually if we had a better model.
                        // For now: simplest is to live with it updating on next search OR 
                        // construct a new vector with the updated item.
                        
                        let mut vec: Vec<JobEntry> = jobs.iter().collect();
                        if let Some(pos) = vec.iter().position(|j| j.id == id_str) {
                            let mut entry = vec[pos].clone();
                            entry.status = new_status as i32;
                            
                            // If rejected, maybe remove from list?
                            if new_status == AdStatus::Rejected {
                                vec.remove(pos);
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
        
        let rt_handle = rt_clone.handle().clone();
        rt_handle.spawn(async move {
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
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_searching(false);
                ui.set_status_msg("Inga sökord angivna.".into());
            }
        });
        return;
    }

    let municipalities = JobSearchClient::parse_locations(&locations_str);
    tracing::info!("Searching: query='{}', locs={:?}", query, municipalities);

    let result = api_client.search(&query, &municipalities, 100).await;
    
    // Prepare blacklist
    let blacklist: Vec<String> = settings.blacklist_keywords.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_searching(false);
            
            // Handle result inside spawn or handle here? 
            // We need async DB access to check status. 
            // Spawning a new task inside the UI thread callback is not ideal, 
            // better to continue async here (we are in async perform_search).
        }
    });
    
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
                // We must use the ad from DB if it exists (to get status), otherwise save the new one.
                // Since db.get_job_ad is async, we do it here.
                let db_ad_opt = db.get_job_ad(&ad.id).await.unwrap_or(None);
                
                let display_ad = if let Some(local_ad) = db_ad_opt {
                    local_ad
                } else {
                    // Save new ad to DB
                    let _ = db.save_job_ad(&ad).await;
                    ad
                };

                // 3. Filter Rejected (unless we are in a "Trash" view, but this is Inbox)
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
            
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let model = Rc::new(slint::VecModel::from(final_entries));
                    ui.set_jobs(model.into());
                    ui.set_status_msg(format!("Hittade {} annonser", count).into());
                }
            });
        }
        Err(e) => {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_status_msg(format!("Fel: {}", e).into());
                }
            });
        }
    }
}