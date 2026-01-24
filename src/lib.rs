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
use chrono::Datelike;

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
use tracing_subscriber::EnvFilter;

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

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,winit=warn,calloop=warn,slint=warn,i_slint_backend_winit=warn"));

    let registry = tracing_subscriber::registry()
        .with(filter)
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
        // Ensure the local log file exists immediately so developers can read it in the project
        let _ = std::fs::File::create(&local_file);
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
    spawn_log_task(ui_weak.clone(), log_rx);

    // Hjälpfunktion för att uppdatera statistik (trådsäker)
    let db_for_stats = db.clone();
    let ui_for_stats = ui.as_weak();
    let rt_for_stats = rt.clone();
    let refresh_stats = move || {
        let db = db_for_stats.clone();
        let ui_weak = ui_for_stats.clone();
        let rt = rt_for_stats.clone();
        
        // 1. Hämta data från UI på huvudtråden
        let month_info = if let Some(ui) = ui_weak.upgrade() {
            let month_str = ui.get_active_month().to_string();
            let parts: Vec<&str> = month_str.split('-').collect();
            if parts.len() == 2 {
                Some((parts[0].parse().unwrap_or(2026), parts[1].parse().unwrap_or(1)))
            } else { None }
        } else { None };

        // 2. Om vi har data, kör databasjobbet i bakgrunden via rts spawn
        if let Some((year, month)) = month_info {
            rt.spawn(async move {
                if let Ok(ads) = db.get_filtered_jobs(&[], Some(year), Some(month)).await {
                    let total_count = ads.len() as i32;
                    let mut applied = 0;
                    let mut bookmarked = 0;
                    let mut thumbsup = 0;
                    let mut rejected = 0;
                    
                    let mut counts = std::collections::HashMap::new();
                    for ad in ads {
                        // Räkna statusar
                        match ad.status {
                            Some(AdStatus::Applied) => applied += 1,
                            Some(AdStatus::Bookmarked) => bookmarked += 1,
                            Some(AdStatus::ThumbsUp) => thumbsup += 1,
                            Some(AdStatus::Rejected) => rejected += 1,
                            _ => {}
                        }

                        // Räkna sökord
                        if let Some(kw) = ad.search_keyword {
                            *counts.entry(kw).or_insert(0) += 1;
                        }
                    }
                    
                    let mut stats_vec: Vec<KeywordStat> = counts.into_iter()
                        .map(|(name, count)| KeywordStat { name: name.into(), count })
                        .collect();
                    stats_vec.sort_by(|a, b| b.count.cmp(&a.count));
                    stats_vec.truncate(10);
                    
                    // 3. Skicka tillbaka resultatet till UI-tråden
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_total_ads_count(total_count);
                            ui.set_applied_count(applied);
                            ui.set_bookmarked_count(bookmarked);
                            ui.set_thumbsup_count(thumbsup);
                            ui.set_rejected_count(rejected);
                            ui.set_top_keywords(Rc::new(slint::VecModel::from(stats_vec)).into());
                        }
                    });
                }
            });
        }
    };

    let refresh_stats_cmd = refresh_stats.clone();
    ui.on_stats_requested(move || {
        refresh_stats_cmd();
    });

    // Förbered variabler för alla callbacks
    let db_c = db.clone();
    let rt_c = rt.clone();
    let ui_c = ui.as_weak();
    let refresh_stats_c = refresh_stats.clone();

    // Callback: Month Offset
    ui.on_month_offset(move |offset| {
        let db = db_c.clone();
        let rt = rt_c.clone();
        let ui_weak = ui_c.clone();
        let refresh_stats = refresh_stats_c.clone();

        tracing::info!("Month offset requested: {}", offset);
        refresh_stats();

        let current_month = if let Some(ui) = ui_weak.upgrade() {
            ui.get_active_month().to_string()
        } else {
            format!("{:04}-{:02}", chrono::Utc::now().year(), chrono::Utc::now().month())
        };

        let mut parts = current_month.split('-');
        let year = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(2026);
        let month = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
        let mut new_month = month + offset as i32;
        let mut new_year = year;
        while new_month <= 0 { new_month += 12; new_year -= 1; }
        while new_month > 12 { new_month -= 12; new_year += 1; }
        
        let new_month_str = format!("{:04}-{:02}", new_year, new_month as u32);
        let new_month_display = format!("{} {}", swedish_month_name(new_month as u32), new_year);

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_active_month(new_month_str.clone().into());
            ui.set_active_month_display(new_month_display.clone().into());
            ui.set_status_msg(format!("Laddar {}...", new_month_display).into());
        }

        let ui_final = ui_weak.clone();
        rt.spawn(async move {
            if let Ok(ads) = db.get_filtered_jobs(&[], Some(new_year), Some(new_month as u32)).await {
                let applied_count = ads.iter().filter(|ad| ad.status == Some(AdStatus::Applied)).count() as i32;
                let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
                let entries: Vec<JobEntry> = ads.into_iter().map(|ad| {
                    let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                    let formatted_desc = raw_desc.replace("<li>", "\n • ").replace("</li>", "").replace("<ul>", "\n").replace("</ul>", "\n")
                        .replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("<p>", "\n\n").replace("</p>", "")
                        .replace("<strong>", "").replace("</strong>", "").replace("<b>", "").replace("</b>", "");
                    let mut clean_desc = re_html.replace_all(&formatted_desc, "").to_string();
                    let mut extra_info = String::new();
                    if ad.driving_license_required { extra_info.push_str("\n\nKÖRKORT:\n • Krav på körkort\n"); }
                    if let Some(req) = &ad.must_have {
                        if !req.skills.is_empty() || !req.languages.is_empty() || !req.work_experiences.is_empty() {
                            extra_info.push_str("\n\nKRAV:\n");
                            for s in &req.skills { extra_info.push_str(&format!(" • {}\n", s.label)); }
                            for l in &req.languages { extra_info.push_str(&format!(" • {} (Språk)\n", l.label)); }
                            for w in &req.work_experiences { extra_info.push_str(&format!(" • {} (Erfarenhet)\n", w.label)); }
                        }
                    }
                    if let Some(nice) = &ad.nice_to_have {
                        if !nice.skills.is_empty() || !nice.languages.is_empty() || !nice.work_experiences.is_empty() {
                            extra_info.push_str("\n\nMERITERANDE:\n");
                            for s in &nice.skills { extra_info.push_str(&format!(" • {}\n", s.label)); }
                            for l in &nice.languages { extra_info.push_str(&format!(" • {} (Språk)\n", l.label)); }
                            for w in &nice.work_experiences { extra_info.push_str(&format!(" • {} (Erfarenhet)\n", w.label)); }
                        }
                    }
                    clean_desc.push_str(&extra_info);
                    JobEntry {
                        id: ad.id.into(), title: ad.headline.into(), employer: ad.employer.and_then(|e| e.name).unwrap_or_default().into(),
                        location: ad.workplace_address.and_then(|a| a.city).unwrap_or_default().into(),
                        description: clean_desc.into(), date: ad.publication_date.split('T').next().unwrap_or("").into(),
                        apply_url: ad.application_details.and_then(|d| d.url).unwrap_or_default().into(),
                        rating: ad.rating.unwrap_or(0) as i32,
                        status: match ad.status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 },
                        status_text: "".into(),
                    }
                }).collect();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_final.upgrade() {
                        ui.set_jobs(Rc::new(slint::VecModel::from(entries)).into());
                        ui.set_applied_count(applied_count);
                        ui.set_status_msg(format!("Laddade annonser för {}", new_month_display).into());
                    }
                });
            }
        });
    });

    // Callback: Free Search
    let api_search = Arc::new(JobSearchClient::new());
    let db_search = db.clone();
    let ui_search = ui.as_weak();
    let rt_search = rt.clone();
    ui.on_search_pressed(move |query| {
        let api = api_search.clone();
        let db = db_search.clone();
        let ui_weak = ui_search.clone();
        let q = query.to_string();
        rt_search.spawn(async move {
            let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default();
            perform_search(api, db, ui_weak, None, Some(q), settings).await;
        });
    });

    // Callback: Prio Search
    let api_prio = Arc::new(JobSearchClient::new());
    let db_prio = db.clone();
    let ui_prio = ui.as_weak();
    let rt_prio = rt.clone();
    ui.on_search_prio(move |prio| {
        let api = api_prio.clone();
        let db = db_prio.clone();
        let ui_weak = ui_prio.clone();
        rt_prio.spawn(async move {
            let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default();
            perform_search(api, db, ui_weak, Some(prio), None, settings).await;
        });
    });

    // Callback: Job Selected
    ui.on_job_selected(|_id, _idx| {});

    // Callback: Job Action
    let db_action = db.clone();
    let ui_action = ui.as_weak();
    let rt_action = rt.clone();
    ui.on_job_action(move |id, action| {
        let db = db_action.clone();
        let ui_weak = ui_action.clone();
        let id_str = id.to_string();
        let act = action.to_string();
        rt_action.spawn(async move {
            if act == "open" || act == "apply_direct" {
                if let Ok(Some(ad)) = db.get_job_ad(&id_str).await {
                    let url = if act == "open" { ad.webpage_url } else { ad.application_details.and_then(|d| d.url) };
                    if let Some(u) = url { let _ = webbrowser::open(&u); }
                }
                return;
            }
            let target = match act.as_str() { "reject" => AdStatus::Rejected, "save" => AdStatus::Bookmarked, "thumbsup" => AdStatus::ThumbsUp, "apply" => AdStatus::Applied, _ => return };
            let current = db.get_job_ad(&id_str).await.ok().flatten().and_then(|ad| ad.status);
            let new_status = if current == Some(target) { None } else { Some(target) };
            if db.update_ad_status(&id_str, new_status).await.is_ok() {
                let status_int = match new_status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 };
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let jobs = ui.get_jobs();
                        let mut vec: Vec<JobEntry> = jobs.iter().collect();
                        if let Some(pos) = vec.iter().position(|j| j.id == id_str) {
                            if status_int == 1 { vec.remove(pos); }
                            else { vec[pos].status = status_int; }
                            ui.set_jobs(Rc::new(slint::VecModel::from(vec)).into());
                        }
                    }
                });
            }
        });
    });

    // Callback: Copy Text
    ui.on_copy_text(|text| {
        #[cfg(not(target_os = "android"))]
        if let Ok(mut cb) = arboard::Clipboard::new() { let _ = cb.set_text(text.to_string()); }
    });

    // Callback: Save Settings
    let db_save = db.clone();
    let ui_save = ui.as_weak();
    let rt_save = rt.clone();
    ui.on_save_settings(move |s| {
        let db = db_save.clone();
        let ui_weak = ui_save.clone();
        let settings = crate::models::AppSettings {
            keywords: s.keywords.to_string(), blacklist_keywords: s.blacklist_keywords.to_string(),
            locations_p1: s.locations_p1.to_string(), locations_p2: s.locations_p2.to_string(),
            locations_p3: s.locations_p3.to_string(), my_profile: s.my_profile.to_string(),
            ollama_url: s.ollama_url.to_string(), app_min_count: s.app_min_count,
            app_goal_count: s.app_goal_count, show_motivation: s.show_motivation,
        };
        let s_ui = settings.clone();
        rt_save.spawn(async move {
            if db.save_settings(&settings).await.is_ok() {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_settings(AppSettings {
                            keywords: s_ui.keywords.into(), blacklist_keywords: s_ui.blacklist_keywords.into(),
                            locations_p1: normalize_locations(&s_ui.locations_p1).into(), locations_p2: normalize_locations(&s_ui.locations_p2).into(),
                            locations_p3: normalize_locations(&s_ui.locations_p3).into(), my_profile: s_ui.my_profile.into(),
                            ollama_url: s_ui.ollama_url.into(), app_min_count: s_ui.app_min_count,
                            app_goal_count: s_ui.app_goal_count, show_motivation: s_ui.show_motivation,
                        });
                        ui.set_status_msg("Inställningar sparade".into());
                    }
                });
            }
        });
    });

    // Initial laddning
    let db_init = db.clone();
    let ui_init = ui.as_weak();
    rt.spawn(async move {
        let settings = db_init.load_settings().await.unwrap_or_default().unwrap_or_default();
        let s = settings.clone();
        
        let ui_for_settings = ui_init.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_for_settings.upgrade() {
                ui.set_settings(AppSettings {
                    keywords: s.keywords.into(), blacklist_keywords: s.blacklist_keywords.into(),
                    locations_p1: normalize_locations(&s.locations_p1).into(), locations_p2: normalize_locations(&s.locations_p2).into(),
                    locations_p3: normalize_locations(&s.locations_p3).into(), my_profile: s.my_profile.into(),
                    ollama_url: s.ollama_url.into(), app_min_count: s.app_min_count,
                    app_goal_count: s.app_goal_count, show_motivation: s.show_motivation,
                });
            }
        });
        
        let now = chrono::Utc::now();
        let month_str = format!("{:04}-{:02}", now.year(), now.month());
        let month_display = format!("{} {}", swedish_month_name(now.month()), now.year());
        
        let ui_for_month = ui_init.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_for_month.upgrade() {
                ui.set_active_month(month_str.into());
                ui.set_active_month_display(month_display.into());
            }
        });

        perform_search(Arc::new(JobSearchClient::new()), db_init, ui_init, Some(1), None, settings).await;
    });
}

// Log receiver task
fn spawn_log_task(ui_weak: slint::Weak<App>, log_rx: mpsc::Receiver<String>) {
    std::thread::spawn(move || {
        let mut log_lines: Vec<String> = Vec::new();
        while let Ok(msg) = log_rx.recv() {
            log_lines.push(msg.trim().to_string());
            if log_lines.len() > 100 { log_lines.remove(0); }
            let lines = log_lines.join("\n");
            let ui = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui.upgrade() { ui.set_system_logs(lines.into()); }
            });
        }
    });
}

// Helper function to handle search logic
// 🛑 STOP! CRITICAL ARCHITECTURAL DECISION - DO NOT "OPTIMIZE" THIS!
// Why are we searching keywords individually? 
// 1. The JobTech API's "concept extraction" is unstable with complex OR-queries.
// 2. Combining terms like "it" and "butik" in one OR-chain often results in 0 hits.
// 3. Batched searching (groups of 5) also proved unreliable in production tests.
// 4. Individual searching is the ONLY way to guarantee 100% hit rate across all terms.
// Performance cost is negligible compared to the value of not missing job opportunities.
async fn perform_search(
    api_client: Arc<JobSearchClient>,
    db: Arc<Db>,
    ui_weak: slint::Weak<App>,
    prio: Option<i32>,
    free_query: Option<String>,
    settings: crate::models::AppSettings
) {
    // 1. Förbered parametrar
    let now = chrono::Utc::now();
    let current_year = now.year();
    let current_month = now.month();
    
    let (y, m) = if let Some(ui) = ui_weak.upgrade() {
        let month_str = ui.get_active_month().to_string();
        let parts: Vec<&str> = month_str.split('-').collect();
        if parts.len() == 2 {
            (parts[0].parse().unwrap_or(current_year), parts[1].parse().unwrap_or(current_month))
        } else {
            (current_year, current_month)
        }
    } else {
        (current_year, current_month)
    };

    let (raw_query, locations_str) = match (free_query.clone(), prio) {
        (Some(q), _) => (q, String::new()),
        (None, Some(p)) => {
            let locs = match p {
                1 => &settings.locations_p1,
                2 => &settings.locations_p2,
                3 => &settings.locations_p3,
                _ => &settings.locations_p1,
            };
            (settings.keywords.clone(), locs.clone())
        },
        _ => (String::new(), String::new()),
    };

    let municipalities = JobSearchClient::parse_locations(&locations_str);
    let query_parts: Vec<_> = raw_query.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.replace("\"", "")) // Send raw terms to avoid API parsing errors
        .collect();

    // 2. LADDA FRÅN DB DIREKT (Visa cache för användaren direkt)
    let ui_early = ui_weak.clone();
    let prio_early = prio;
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_early.upgrade() {
            ui.set_searching(true);
            ui.set_status_msg(format!("Söker efter nytt... (Visar sparade jobb för P{})", prio_early.unwrap_or(0)).into());
        }
    });

    // Hjälpfunktion för att ladda och visa från DB
    let refresh_ui_from_db = |ui: &App, ads: Vec<crate::models::JobAd>, p: Option<i32>, muns: Vec<String>, msg: String| {
        let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
        let prio_municipality_names: Vec<String> = if p.is_some() {
            muns.iter().filter_map(|code| JobSearchClient::get_municipality_name(code)).map(|s| s.to_lowercase()).collect()
        } else {
            Vec::new()
        };

        let mut entries: Vec<JobEntry> = ads.into_iter()
            .filter(|ad| {
                if !prio_municipality_names.is_empty() {
                    if let Some(ref addr) = ad.workplace_address {
                        if let Some(ref mun) = addr.municipality {
                            return prio_municipality_names.contains(&mun.to_lowercase());
                        }
                    }
                    return false;
                }
                true
            })
            .map(|ad| {
                let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                let formatted_desc = raw_desc.replace("<li>", "\n • ").replace("</li>", "").replace("<ul>", "\n").replace("</ul>", "\n")
                    .replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("<p>", "\n\n").replace("</p>", "")
                    .replace("<strong>", "").replace("</strong>", "").replace("<b>", "").replace("</b>", "");
                let mut clean_desc = re_html.replace_all(&formatted_desc, "").to_string();
                let mut extra_info = String::new();
                if ad.driving_license_required { extra_info.push_str("\n\nKÖRKORT:\n • Krav på körkort\n"); }
                if let Some(req) = &ad.must_have {
                    if !req.skills.is_empty() || !req.languages.is_empty() || !req.work_experiences.is_empty() {
                        extra_info.push_str("\n\nKRAV:\n");
                        for s in &req.skills { extra_info.push_str(&format!(" • {}\n", s.label)); }
                        for l in &req.languages { extra_info.push_str(&format!(" • {} (Språk)\n", l.label)); }
                        for w in &req.work_experiences { extra_info.push_str(&format!(" • {} (Erfarenhet)\n", w.label)); }
                    }
                }
                if let Some(nice) = &ad.nice_to_have {
                    if !nice.skills.is_empty() || !nice.languages.is_empty() || !nice.work_experiences.is_empty() {
                        extra_info.push_str("\n\nMERITERANDE:\n");
                        for s in &nice.skills { extra_info.push_str(&format!(" • {}\n", s.label)); }
                        for l in &nice.languages { extra_info.push_str(&format!(" • {} (Språk)\n", l.label)); }
                        for w in &nice.work_experiences { extra_info.push_str(&format!(" • {} (Erfarenhet)\n", w.label)); }
                    }
                }
                clean_desc.push_str(&extra_info);
                let status_int = match ad.status {
                    Some(crate::models::AdStatus::Rejected) => 1,
                    Some(crate::models::AdStatus::Bookmarked) => 2,
                    Some(crate::models::AdStatus::ThumbsUp) => 3,
                    Some(crate::models::AdStatus::Applied) => 4,
                    _ => 0,
                };
                JobEntry {
                    id: ad.id.into(),
                    title: ad.headline.into(),
                    employer: ad.employer.and_then(|e| e.name).unwrap_or_else(|| "Okänd".to_string()).into(),
                    location: ad.workplace_address.and_then(|a| a.city).unwrap_or_else(|| "Okänd".to_string()).into(),
                    description: clean_desc.into(),
                    date: ad.publication_date.split('T').next().unwrap_or("").into(),
                    apply_url: ad.application_details.and_then(|d| d.url).unwrap_or_default().into(),
                    rating: ad.rating.unwrap_or(0) as i32,
                    status: status_int,
                    status_text: "".into(),
                }
            }).collect();

        // Sortering (nyast först)
        entries.sort_by(|a, b| b.date.cmp(&a.date));

        let model = std::rc::Rc::new(slint::VecModel::from(entries));
        ui.set_jobs(model.into());
        ui.set_status_msg(msg.into());
    };

    // Kör första laddningen från DB
    if let Ok(existing_ads) = db.get_filtered_jobs(&[], Some(y), Some(m)).await {
        let ui_early_2 = ui_weak.clone();
        let muns_early_2 = municipalities.clone();
        let loc_display = locations_str.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_early_2.upgrade() {
                let msg = format!("Visar sparade jobb för {}. Söker efter nytt...", loc_display);
                refresh_ui_from_db(&ui, existing_ads, prio, muns_early_2, msg);
            }
        });
    }

    // 3. API-ANROP (Ett per sökord för maximal pålitlighet)
    let mut new_count = 0;
    let blacklist: Vec<String> = settings.blacklist_keywords.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();

    for keyword in &query_parts {
        match api_client.search(keyword, &municipalities, 100).await {
            Ok(ads) => {
                for mut ad in ads {
                    // Spara vilket sökord som hittade annonsen för statistik
                    ad.search_keyword = Some(keyword.clone());

                    let is_blacklisted = blacklist.iter().any(|word| {
                        ad.headline.to_lowercase().contains(word) || 
                        ad.description.as_ref().and_then(|d| d.text.as_deref()).map(|t| t.to_lowercase().contains(word)).unwrap_or(false)
                    });

                    if !is_blacklisted {
                        if let Ok(None) = db.get_job_ad(&ad.id).await {
                            if let Err(e) = db.save_job_ad(&ad).await {
                                tracing::error!("Failed to auto-save ad {}: {}", ad.id, e);
                            } else {
                                new_count += 1;
                            }
                        }
                    }
                }
            },
            Err(e) => {
                tracing::error!("Sökning på '{}' misslyckades: {:?}", keyword, e);
            }
        }
    }

    // 4. Slutlig uppdatering av UI med allt från DB
    if let Ok(final_ads) = db.get_filtered_jobs(&[], Some(y), Some(m)).await {
        let ui_final = ui_weak.clone();
        let muns_final = municipalities.clone();
        let msg = if new_count > 0 {
            format!("Klar! Hittade {} nya annonser.", new_count)
        } else {
            "Inga nya annonser hittades just nu.".to_string()
        };
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_final.upgrade() {
                refresh_ui_from_db(&ui, final_ads, prio, muns_final, msg);
                ui.set_searching(false);
            }
        });
    } else {
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() { ui.set_searching(false); }
        });
    }
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
