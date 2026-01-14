// Include generated Slint code
mod ui {
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}

use slint::ComponentHandle;
use slint::Model;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;

mod models;
mod api;
mod db;
mod ai;

use crate::api::JobSearchClient;
use crate::ui::*;

fn setup_ui(ui: &App, rt: Arc<Runtime>) {
    let ui_weak = ui.as_weak();
    let api_client = Arc::new(JobSearchClient::new());

    ui.on_search_pressed(move |query| {
        let ui_weak = ui_weak.clone();
        let api_client = api_client.clone();
        let query = query.to_string();
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_searching(true);
            ui.set_status_msg("Söker...".into());
        }

        let rt_handle = rt.handle().clone();
        rt_handle.spawn(async move {
            // Här kan vi lägga till kommuner från inställningar senare
            let result = api_client.search(&query, &[], 50).await;
            
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_searching(false);
                    match result {
                        Ok(ads) => {
                            let entries: Vec<JobEntry> = ads.into_iter().map(|ad| {
                                JobEntry {
                                    id: ad.id.into(),
                                    title: ad.headline.into(),
                                    employer: ad.employer.and_then(|e| e.name).unwrap_or_else(|| "Okänd".to_string()).into(),
                                    location: ad.workplace_address.and_then(|a| a.city).unwrap_or_else(|| "Okänd".to_string()).into(),
                                    date: ad.publication_date.split('T').next().unwrap_or("").into(),
                                    rating: ad.rating.unwrap_or(0) as i32,
                                    status_text: "".into(),
                                }
                            }).collect();
                            
                            let model = Rc::new(slint::VecModel::from(entries));
                            ui.set_jobs(model.into());
                            ui.set_status_msg(format!("Hittade {} annonser", ui.get_jobs().row_count()).into());
                        }
                        Err(e) => {
                            ui.set_status_msg(format!("Fel: {}", e).into());
                        }
                    }
                }
            });
        });
    });

    let ui_weak = ui.as_weak();
    ui.on_job_selected(move |id| {
        if let Some(_ui) = ui_weak.upgrade() {
            tracing::info!("Valt jobb: {}", id);
        }
    });

    ui.on_save_settings(move |settings| {
        tracing::info!("Sparar inställningar: Sökord='{}', Plats='{}'", settings.keywords, settings.locations_p1);
    });
}

#[no_mangle]
#[cfg(target_os = "android")]
pub extern "Rust" fn android_main(app: slint::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(tracing::log::LevelFilter::Info),
    );
    tracing::info!("Starting Jobseeker on Android");
    slint::android::init(app).expect("Failed to initialize Slint on Android");

    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let ui = App::new().expect("Failed to create Slint UI");
    
    setup_ui(&ui, rt);
    
    ui.run().expect("Failed to run Slint UI");
}

#[cfg(not(target_os = "android"))]
fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Jobseeker on Desktop");
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let ui = App::new().expect("Failed to create Slint UI");
    
    setup_ui(&ui, rt);
    
    ui.run().expect("Failed to run Slint UI");
}
