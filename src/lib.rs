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
        1 => "Januari", 2 => "Februari", 3 => "Mars", 4 => "April",
        5 => "Maj", 6 => "Juni", 7 => "Juli", 8 => "Augusti",
        9 => "September", 10 => "Oktober", 11 => "November", 12 => "December",
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

// Global sender for clipboard operations to keep the provider alive on Linux
static CLIPBOARD_SENDER: std::sync::OnceLock<mpsc::Sender<String>> = std::sync::OnceLock::new();

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

/// The Clipboard Manager solves a critical issue on Linux where clipboard content
/// is lost if the application that "owns" the data drops its reference too quickly.
/// By spawning a long-lived thread that manages a single Clipboard instance,
/// we ensure that the OS and other applications have enough time to fetch the data.
fn setup_clipboard_manager() {
    let (tx, rx) = mpsc::channel::<String>();
    let _ = CLIPBOARD_SENDER.set(tx);
    
    std::thread::spawn(move || {
        #[cfg(not(target_os = "android"))]
        let mut clipboard = arboard::Clipboard::new().ok();
        
        while let Ok(text) = rx.recv() {
            #[cfg(not(target_os = "android"))]
            if let Some(ref mut cb) = clipboard {
                let _ = cb.set_text(text);
                tracing::info!("Text copied to clipboard and kept alive.");
            }
            #[cfg(target_os = "android")]
            let _ = text; // Silence unused warning on Android
        }
    });
}

fn copy_to_clipboard(text: String) {
    if let Some(sender) = CLIPBOARD_SENDER.get() {
        let _ = sender.send(text);
    }
}

/// Triggers an automatic backup of the database to a user-defined sync folder.
/// This is designed to work seamlessly with Syncthing, Dropbox, or other 
/// folder-monitoring sync tools.
async fn trigger_sync(db: &Db) {
    if let Ok(Some(settings)) = db.load_settings().await {
        if !settings.sync_path.is_empty() {
            let sync_dir = std::path::PathBuf::from(&settings.sync_path);
            if sync_dir.exists() && sync_dir.is_dir() {
                let db_path = get_db_path();
                let target_path = sync_dir.join("jobseeker.redb");
                if let Err(e) = std::fs::copy(&db_path, &target_path) {
                    tracing::error!("Automatisk synk misslyckades: {}", e);
                } else {
                    tracing::info!("Automatisk synk klar: {:?}", target_path);
                }
            }
        }
    }
}

fn setup_logging() -> (Option<tracing_appender::non_blocking::WorkerGuard>, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel();
    let _ = LOG_SENDER.set(tx.clone());
    let slint_writer = SlintLogWriter { sender: tx };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,winit=warn,calloop=warn,slint=warn,i_slint_backend_winit=warn"));
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout).with_ansi(true))
        .with(tracing_subscriber::fmt::layer().with_writer(move || slint_writer.sender.clone().into_writer()).with_ansi(false));

    #[cfg(not(target_os = "android"))]
    {
        let log_dir = directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker").map(|p| p.data_dir().join("logs")).unwrap_or_else(|| std::path::PathBuf::from("logs"));
        let _ = std::fs::create_dir_all(&log_dir);
        let file_appender = tracing_appender::rolling::daily(&log_dir, "jobseeker.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        let r = registry.with(tracing_subscriber::fmt::layer().with_writer(non_blocking).with_ansi(false));
        r.init();
        (Some(guard), rx)
    }
    #[cfg(target_os = "android")]
    { registry.init(); (None, rx) }
}

trait ToWriter { fn into_writer(self) -> mpsc_writer::MpscWriter; }
impl ToWriter for mpsc::Sender<String> { fn into_writer(self) -> mpsc_writer::MpscWriter { mpsc_writer::MpscWriter { sender: self } } }
mod mpsc_writer {
    use std::sync::mpsc;
    pub struct MpscWriter { pub sender: mpsc::Sender<String> }
    impl std::io::Write for MpscWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { if let Ok(msg) = String::from_utf8(buf.to_vec()) { let _ = self.sender.send(msg); } Ok(buf.len()) }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
}

fn get_db_path() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        let path = std::path::PathBuf::from("/data/data/com.gnawsoftware.jobseeker/files"); 
        let _ = std::fs::create_dir_all(&path); 
        return path.join("jobseeker.redb"); 
    }
    #[cfg(not(target_os = "android"))]
    {
        directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker").map(|p| {
            let d = p.data_dir();
            let _ = std::fs::create_dir_all(d);
            d.join("jobseeker.redb")
        }).unwrap_or_else(|| std::path::PathBuf::from("jobseeker.redb"))
    }
}

fn normalize_locations(input: &str) -> String {
    input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| {
        if s.chars().all(char::is_numeric) { JobSearchClient::get_municipality_name(s).unwrap_or_else(|| s.to_string()) }
        else { let mut chars = s.chars(); match chars.next() { None => String::new(), Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str() } }
    }).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ")
}

fn setup_ui(ui: &App, rt: Arc<Runtime>, db: Arc<Db>, log_rx: mpsc::Receiver<String>) {
    let ui_weak = ui.as_weak();
    spawn_log_task(ui_weak.clone(), log_rx);

    let db_for_stats = db.clone();
    let ui_for_stats = ui.as_weak();
    let rt_for_stats = rt.clone();
    let refresh_stats = move || {
        let db = db_for_stats.clone();
        let ui_weak = ui_for_stats.clone();
        let rt = rt_for_stats.clone();
        let month_info = if let Some(ui) = ui_weak.upgrade() {
            let month_str = ui.get_active_month().to_string();
            let parts: Vec<&str> = month_str.split('-').collect();
            if parts.len() == 2 { Some((parts[0].parse().unwrap_or(2026), parts[1].parse().unwrap_or(1))) } else { None }
        } else { None };

        if let Some((year, month)) = month_info {
            rt.spawn(async move {
                if let Ok(ads) = db.get_filtered_jobs(&[], Some(year), Some(month)).await {
                    let total_count = ads.len() as i32;
                    let (mut applied, mut bookmarked, mut thumbsup, mut rejected) = (0, 0, 0, 0);
                    let mut counts = std::collections::HashMap::new();
                    for ad in ads {
                        match ad.status { Some(AdStatus::Applied) => applied += 1, Some(AdStatus::Bookmarked) => bookmarked += 1, Some(AdStatus::ThumbsUp) => thumbsup += 1, Some(AdStatus::Rejected) => rejected += 1, _ => {} }
                        if let Some(kw) = ad.search_keyword { *counts.entry(kw).or_insert(0) += 1; }
                    }
                    let mut stats_vec: Vec<KeywordStat> = counts.into_iter().map(|(name, count)| KeywordStat { name: name.into(), count }).collect();
                    stats_vec.sort_by(|a, b| b.count.cmp(&a.count)); stats_vec.truncate(10);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_total_ads_count(total_count); ui.set_applied_count(applied); ui.set_bookmarked_count(bookmarked);
                            ui.set_thumbsup_count(thumbsup); ui.set_rejected_count(rejected); ui.set_top_keywords(Rc::new(slint::VecModel::from(stats_vec)).into());
                        }
                    });
                }
            });
        }
    };

    let rs_cmd = refresh_stats.clone();
    ui.on_stats_requested(move || rs_cmd());

    // Callback: Export Report
    let db_export = db.clone();
    let ui_export = ui.as_weak();
    let rt_export = rt.clone();
    ui.on_export_requested(move |method, _format, include_jobs, include_params, include_analysis| {
        let db = db_export.clone();
        let ui_weak = ui_export.clone();
        let method = method.to_string();
        let data = if let Some(ui) = ui_weak.upgrade() { Some((ui.get_active_month().to_string(), ui.get_active_month_display().to_string())) } else { None };

        if let Some((month_str, month_display)) = data {
            rt_export.spawn(async move {
                let parts: Vec<&str> = month_str.split('-').collect();
                let year = parts[0].parse().unwrap_or(2026);
                let month = parts[1].parse().unwrap_or(1);
                let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default();
                
                let mut report = format!("AKTIVITETSRAPPORT - {}
==========================================\n\n", month_display.to_uppercase());
                if include_params {
                    report.push_str(&format!("SÖKPARAMETRAR:\n• Sökord: {}
• Prio 1: {}
• Prio 2: {}
\n", settings.keywords, normalize_locations(&settings.locations_p1), normalize_locations(&settings.locations_p2)));
                }
                if include_jobs {
                    if let Ok(ads) = db.get_filtered_jobs(&[AdStatus::Applied], Some(year), Some(month)).await {
                        report.push_str(&format!("SÖKTA JOBB ({} st):\n", ads.len()));
                        for ad in ads {
                            let date = ad.applied_at.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "Okänt datum".to_string());
                            report.push_str(&format!("• {}: {}, {} ({})\n", date, ad.employer.and_then(|e| e.name).unwrap_or_default(), ad.headline, ad.workplace_address.and_then(|a| a.city).unwrap_or_default()));
                            if let Some(url) = ad.webpage_url { report.push_str(&format!("  Länk: {}\n", url)); }
                        }
                        report.push_str("\n");
                    }
                }
                if include_analysis {
                    if let Ok(ads) = db.get_filtered_jobs(&[], Some(year), Some(month)).await {
                        let app = ads.iter().filter(|a| a.status == Some(AdStatus::Applied)).count();
                        let rej = ads.iter().filter(|a| a.status == Some(AdStatus::Rejected)).count();
                        report.push_str(&format!("AKTIVITETSANALYS:\n• Totalt granskade: {}
• Konvertering: {} sökta, {} avvisade\n", ads.len(), app, rej));
                    }
                }
                report.push_str("\nGenererad via Jobseeker 2026\n");

                if method == "clipboard" || method == "email" {
                    copy_to_clipboard(report.clone());
                    if method == "clipboard" {
                        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Rapport kopierad till urklipp!".into()); } });
                    } else {
                        let subject_text = format!("Aktivitetsrapport - {}", month_display);
                        let subject = urlencoding::encode(&subject_text);
                        let body_text = if report.len() > 1500 { 
                            format!("Rapporten är kopierad till ditt urklipp - klistra in den här!\n\n(Texten var för lång för direktlänk: {} tecken)", report.len()) 
                        } else { 
                            report 
                        };
                        let body_encoded = urlencoding::encode(&body_text);
                        let mailto = format!("mailto:?subject={}&body={}", subject, body_encoded);
                        let _ = webbrowser::open(&mailto);
                        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Öppnar e-post (rapport kopierad till urklipp)".into()); } });
                    }
                } else if method == "file" {
                    let file_name = format!("jobb-rapport-{}.txt", month_str);
                    let file_path = directories::UserDirs::new().and_then(|u| u.download_dir().map(|d| d.join(&file_name))).unwrap_or_else(|| std::path::PathBuf::from(&file_name));
                    if std::fs::write(&file_path, report).is_ok() {
                        tracing::info!("Rapport sparad till: {:?}", file_path);
                        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg(format!("Rapport sparad: {}", file_name).into()); } });
                    }
                }
            });
        }
    });

    // Callback: Month Offset
    let (db_month, rt_month, ui_month, rs_month) = (db.clone(), rt.clone(), ui.as_weak(), refresh_stats.clone());
    ui.on_month_offset(move |offset| {
        rs_month();
        let (db, rt, ui_weak) = (db_month.clone(), rt_month.clone(), ui_month.clone());
        let data = if let Some(ui) = ui_weak.upgrade() { Some(ui.get_active_month().to_string()) } else { None };
        if let Some(cm) = data {
            let mut parts = cm.split('-');
            let year = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(2026);
            let month = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
            let mut nm = month + offset as i32; let mut ny = year;
            while nm <= 0 { nm += 12; ny -= 1; } while nm > 12 { nm -= 12; ny += 1; }
            let nms = format!("{:04}-{:02}", ny, nm as u32);
            let nmd = format!("{} {}", swedish_month_name(nm as u32), ny);
            if let Some(ui) = ui_weak.upgrade() { ui.set_active_month(nms.clone().into()); ui.set_active_month_display(nmd.clone().into()); }
            let ui_f = ui_weak.clone();
            rt.spawn(async move {
                if let Ok(ads) = db.get_filtered_jobs(&[], Some(ny), Some(nm as u32)).await {
                    let app_count = ads.iter().filter(|ad| ad.status == Some(AdStatus::Applied)).count() as i32;
                    let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
                    let entries: Vec<JobEntry> = ads.into_iter().map(|ad| {
                        let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                        let formatted_desc = raw_desc.replace("<li>", "\n • ").replace("</li>", "").replace("<ul>", "\n").replace("</ul>", "\n").replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("<p>", "\n\n").replace("</p>", "").replace("<strong>", "").replace("</strong>", "").replace("<b>", "").replace("</b>", "");
                        let mut clean_desc = re_html.replace_all(&formatted_desc, "").to_string();
                        if ad.driving_license_required { clean_desc.push_str("\n\nKÖRKORT:\n • Krav på körkort\n"); }
                        JobEntry { id: ad.id.into(), title: ad.headline.into(), employer: ad.employer.and_then(|e| e.name).unwrap_or_default().into(), location: ad.workplace_address.and_then(|a| a.city).unwrap_or_default().into(), description: clean_desc.into(), date: ad.publication_date.split('T').next().unwrap_or("").into(), apply_url: ad.application_details.and_then(|d| d.url).unwrap_or_default().into(), rating: ad.rating.unwrap_or(0) as i32, status: match ad.status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 }, status_text: "".into() }
                    }).collect();
                    let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_f.upgrade() { ui.set_jobs(Rc::new(slint::VecModel::from(entries)).into()); ui.set_applied_count(app_count); } });
                }
            });
        }
    });

    // Callback: Free Search
    let (api_s, db_s, ui_s, rt_s) = (Arc::new(JobSearchClient::new()), db.clone(), ui.as_weak(), rt.clone());
    ui.on_search_pressed(move |q| { let (api, db, ui_weak, q_str) = (api_s.clone(), db_s.clone(), ui_s.clone(), q.to_string()); rt_s.spawn(async move { let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default(); perform_search(api, db, ui_weak, None, Some(q_str), settings).await; }); });

    // Callback: Prio Search
    let (api_p, db_p, ui_p, rt_p) = (Arc::new(JobSearchClient::new()), db.clone(), ui.as_weak(), rt.clone());
    ui.on_search_prio(move |p| { let (api, db, ui_weak) = (api_p.clone(), db_p.clone(), ui_p.clone()); rt_p.spawn(async move { let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default(); perform_search(api, db, ui_weak, Some(p), None, settings).await; }); });

    // Callback: Job Action
    let (db_a, ui_a, rt_a) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_job_action(move |id, act| {
        let (db, ui_weak, id_str, action) = (db_a.clone(), ui_a.clone(), id.to_string(), act.to_string());
        rt_a.spawn(async move {
            if action == "open" || action == "apply_direct" { if let Ok(Some(ad)) = db.get_job_ad(&id_str).await { let url = if action == "open" { ad.webpage_url } else { ad.application_details.and_then(|d| d.url) }; if let Some(u) = url { let _ = webbrowser::open(&u); } } return; }
            let target = match action.as_str() { "reject" => AdStatus::Rejected, "save" => AdStatus::Bookmarked, "thumbsup" => AdStatus::ThumbsUp, "apply" => AdStatus::Applied, _ => return };
            let current = db.get_job_ad(&id_str).await.ok().flatten().and_then(|ad| ad.status);
            let new_status = if current == Some(target) { None } else { Some(target) };
            if db.update_ad_status(&id_str, new_status).await.is_ok() {
                trigger_sync(&db).await;
                let status_int = match new_status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 };
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { let jobs = ui.get_jobs(); let mut vec: Vec<JobEntry> = jobs.iter().collect(); if let Some(pos) = vec.iter().position(|j| j.id == id_str) { if status_int == 1 { vec.remove(pos); } else { vec[pos].status = status_int; } ui.set_jobs(Rc::new(slint::VecModel::from(vec)).into()); } } });
            }
        });
    });

    ui.on_copy_text(|text| copy_to_clipboard(text.to_string()));

    // Callback: Save Settings
    let (db_set, ui_set, rt_set) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_save_settings(move |s| {
        let (db, ui_weak) = (db_set.clone(), ui_set.clone());
        let settings = crate::models::AppSettings { 
            keywords: s.keywords.to_string(), 
            blacklist_keywords: s.blacklist_keywords.to_string(), 
            locations_p1: s.locations_p1.to_string(), 
            locations_p2: s.locations_p2.to_string(), 
            locations_p3: s.locations_p3.to_string(), 
            my_profile: s.my_profile.to_string(), 
            ollama_url: s.ollama_url.to_string(), 
            sync_path: s.sync_path.to_string(),
            app_min_count: s.app_min_count, 
            app_goal_count: s.app_goal_count, 
            show_motivation: s.show_motivation 
        };
        let s_ui = settings.clone();
        rt_set.spawn(async move {
            if db.save_settings(&settings).await.is_ok() {
                trigger_sync(&db).await;
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_settings(AppSettings { keywords: s_ui.keywords.into(), blacklist_keywords: s_ui.blacklist_keywords.into(), locations_p1: normalize_locations(&s_ui.locations_p1).into(), locations_p2: normalize_locations(&s_ui.locations_p2).into(), locations_p3: normalize_locations(&s_ui.locations_p3).into(), my_profile: s_ui.my_profile.into(), ollama_url: s_ui.ollama_url.into(), sync_path: s_ui.sync_path.into(), app_min_count: s_ui.app_min_count, app_goal_count: s_ui.app_goal_count, show_motivation: s_ui.show_motivation }); ui.set_status_msg("Inställningar sparade".into()); } });
            }
        });
    });

    // Callback: Database Action (Synk/Backup)
    let ui_db = ui.as_weak();
    ui.on_db_action(move |act| {
        let ui_weak = ui_db.clone();
        if act == "backup" {
            let db_path = get_db_path();
            let backup_name = format!("jobseeker_backup_{}.redb", chrono::Local::now().format("%Y%m%d_%H%M"));
            let backup_path = directories::UserDirs::new()
                .and_then(|u| u.download_dir().map(|d| d.join(&backup_name)))
                .unwrap_or_else(|| std::path::PathBuf::from(&backup_name));
            
            if std::fs::copy(&db_path, &backup_path).is_ok() {
                tracing::info!("Backup skapad: {:?}", backup_path);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_msg(format!("Backup sparad: {}", backup_name).into());
                    }
                });
            } else {
                tracing::error!("Misslyckades att skapa backup!");
            }
        }
    });

    // Initial laddning
    let (db_i, ui_i, rt_i) = (db.clone(), ui.as_weak(), rt.clone());
    let db_path_str = get_db_path().to_string_lossy().to_string();
    rt_i.spawn(async move {
        let settings = db_i.load_settings().await.unwrap_or_default().unwrap_or_default();
        let (s, u_s) = (settings.clone(), ui_i.clone());
        let d_path = db_path_str.clone();
        let _ = slint::invoke_from_event_loop(move || { 
            if let Some(ui) = u_s.upgrade() { 
                ui.set_database_path(d_path.into());
                ui.set_settings(AppSettings { 
                    keywords: s.keywords.into(), 
                    blacklist_keywords: s.blacklist_keywords.into(), 
                    locations_p1: normalize_locations(&s.locations_p1).into(), 
                    locations_p2: normalize_locations(&s.locations_p2).into(), 
                    locations_p3: normalize_locations(&s.locations_p3).into(), 
                    my_profile: s.my_profile.into(), 
                    ollama_url: s.ollama_url.into(), 
                    sync_path: s.sync_path.into(),
                    app_min_count: s.app_min_count, 
                    app_goal_count: s.app_goal_count, 
                    show_motivation: s.show_motivation 
                }); 
            } 
        });
        let now = chrono::Utc::now();
        let (ms, md, u_m) = (format!("{:04}-{:02}", now.year(), now.month()), format!("{} {}", swedish_month_name(now.month()), now.year()), ui_i.clone());
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = u_m.upgrade() { ui.set_active_month(ms.into()); ui.set_active_month_display(md.into()); } });
        perform_search(Arc::new(JobSearchClient::new()), db_i, ui_i, Some(1), None, settings).await;
    });
}

fn spawn_log_task(ui_weak: slint::Weak<App>, log_rx: mpsc::Receiver<String>) {
    std::thread::spawn(move || {
        let mut log_lines: Vec<String> = Vec::new();
        while let Ok(msg) = log_rx.recv() {
            log_lines.push(msg.trim().to_string()); if log_lines.len() > 100 { log_lines.remove(0); }
            let lines = log_lines.join("\n"); let ui = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui.upgrade() { ui.set_system_logs(lines.into()); } });
        }
    });
}

async fn perform_search(api_client: Arc<JobSearchClient>, db: Arc<Db>, ui_weak: slint::Weak<App>, prio: Option<i32>, free_query: Option<String>, settings: crate::models::AppSettings) {
    let now = chrono::Utc::now();
    let (y, m) = if let Some(ui) = ui_weak.upgrade() { let month_str = ui.get_active_month().to_string(); let parts: Vec<&str> = month_str.split('-').collect(); if parts.len() == 2 { (parts[0].parse().unwrap_or(now.year()), parts[1].parse().unwrap_or(now.month())) } else { (now.year(), now.month()) } } else { (now.year(), now.month()) };
    let (raw_query, locations_str) = match (free_query.clone(), prio) { (Some(q), _) => (q, String::new()), (None, Some(p)) => { let locs = match p { 1 => &settings.locations_p1, 2 => &settings.locations_p2, 3 => &settings.locations_p3, _ => &settings.locations_p1 }; (settings.keywords.clone(), locs.clone()) }, _ => (String::new(), String::new()) };
    let municipalities = JobSearchClient::parse_locations(&locations_str);
    let query_parts: Vec<_> = raw_query.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.replace("\"", "")).collect();
    let ui_early = ui_weak.clone(); let p_early = prio;
    let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_early.upgrade() { ui.set_searching(true); ui.set_status_msg(format!("Söker efter nytt... (Visar sparade jobb för P{})", p_early.unwrap_or(0)).into()); } });

    let refresh_ui_from_db = |ui: &App, ads: Vec<crate::models::JobAd>, p: Option<i32>, muns: Vec<String>, msg: String| {
        let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
        let pmn: Vec<String> = if p.is_some() { muns.iter().filter_map(|code| JobSearchClient::get_municipality_name(code)).map(|s| s.to_lowercase()).collect() } else { Vec::new() };
        
        let applied_count = ads.iter().filter(|ad| ad.status == Some(AdStatus::Applied)).count() as i32;

        let mut entries: Vec<JobEntry> = ads.into_iter().filter(|ad| { 
            if !pmn.is_empty() { 
                if let Some(ref addr) = ad.workplace_address { 
                    if let Some(ref mun) = addr.municipality { return pmn.contains(&mun.to_lowercase()); } 
                } 
                return false; 
            } 
            true 
        }).map(|ad| {
            let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
            let formatted_desc = raw_desc.replace("<li>", "\n • ").replace("</li>", "").replace("<ul>", "\n").replace("</ul>", "\n").replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("<p>", "\n\n").replace("</p>", "").replace("<strong>", "").replace("</strong>", "").replace("<b>", "").replace("</b>", "");
            let mut clean_desc = re_html.replace_all(&formatted_desc, "").to_string();
            if ad.driving_license_required { clean_desc.push_str("\n\nKÖRKORT:\n • Krav på körkort\n"); }
            JobEntry { id: ad.id.into(), title: ad.headline.into(), employer: ad.employer.and_then(|e| e.name).unwrap_or_default().into(), location: ad.workplace_address.and_then(|a| a.city).unwrap_or_default().into(), description: clean_desc.into(), date: ad.publication_date.split('T').next().unwrap_or("").into(), apply_url: ad.application_details.and_then(|d| d.url).unwrap_or_default().into(), rating: ad.rating.unwrap_or(0) as i32, status: match ad.status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 }, status_text: "".into() }
        }).collect();
        
        entries.sort_by(|a, b| b.date.cmp(&a.date));
        
        ui.set_jobs(std::rc::Rc::new(slint::VecModel::from(entries)).into()); 
        ui.set_applied_count(applied_count);
        ui.set_status_msg(msg.into());
    };

    if let Ok(existing_ads) = db.get_filtered_jobs(&[], Some(y), Some(m)).await {
        let ui_e2 = ui_weak.clone(); let muns_e2 = municipalities.clone(); let loc_d = locations_str.clone();
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_e2.upgrade() { let msg = format!("Visar sparade jobb för {}. Söker efter nytt...", loc_d); refresh_ui_from_db(&ui, existing_ads, prio, muns_e2, msg); } });
    }

    let mut new_count = 0; let blacklist: Vec<String> = settings.blacklist_keywords.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    for keyword in &query_parts {
        match api_client.search(keyword, &municipalities, 100).await {
            Ok(ads) => { for mut ad in ads { ad.search_keyword = Some(keyword.clone()); let is_blacklisted = blacklist.iter().any(|word| ad.headline.to_lowercase().contains(word) || ad.description.as_ref().and_then(|d| d.text.as_deref()).map(|t| t.to_lowercase().contains(word)).unwrap_or(false)); if !is_blacklisted { if let Ok(None) = db.get_job_ad(&ad.id).await { if db.save_job_ad(&ad).await.is_ok() { new_count += 1; } } } } },
            Err(e) => { tracing::error!("Sökning på '{}' misslyckades: {:?}", keyword, e); }
        }
    }

    if let Ok(final_ads) = db.get_filtered_jobs(&[], Some(y), Some(m)).await {
        trigger_sync(&db).await;
        let ui_f = ui_weak.clone(); let muns_f = municipalities.clone();
        let msg = if new_count > 0 { format!("Klar! Hittade {} nya annonser.", new_count) } else { "Inga nya annonser hittades just nu.".to_string() };
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_f.upgrade() { refresh_ui_from_db(&ui, final_ads, prio, muns_f, msg); ui.set_searching(false); } });
    } else {
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_searching(false); } });
    }
}

pub fn desktop_main() {
    let (guard, log_rx) = setup_logging();
    setup_clipboard_manager();
    tracing::info!("Starting Jobseeker on Desktop");
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let db_path = get_db_path();
    let db = rt.block_on(async { Db::new(db_path.to_str().unwrap()).await }).expect("Failed to initialize database");
    let db = Arc::new(db);
    let ui = App::new().expect("Failed to create Slint UI");
    setup_ui(&ui, rt, db, log_rx);
    let _log_guard = guard;
    ui.run().expect("Failed to run Slint UI");
}
