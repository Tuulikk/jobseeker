// Include generated Slint code
mod ui {
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}

// Macro for tracing that uses log:: on Android since tracing subscriber is no-op

#[cfg(target_os = "android")]

#[macro_export]

macro_rules! tracing_info { ($($arg:tt)*) => { 

    let msg = format!($($arg)*);

    log::info!("{}", msg);

    $crate::record_log("INFO", &msg);

}; }

#[cfg(target_os = "android")]

#[macro_export]

macro_rules! tracing_error { ($($arg:tt)*) => { 

    let msg = format!($($arg)*);

    log::error!("{}", msg);

    $crate::record_log("ERROR", &msg);

}; }



#[cfg(not(target_os = "android"))]

#[macro_export]

macro_rules! tracing_info { ($($arg:tt)*) => { 

    let msg = format!($($arg)*);

    tracing::info!("{}", msg);

    $crate::record_log("INFO", &msg);

}; }

#[cfg(not(target_os = "android"))]

#[macro_export]

macro_rules! tracing_error { ($($arg:tt)*) => { 

    let msg = format!($($arg)*);

    tracing::error!("{}", msg);

    $crate::record_log("ERROR", &msg);

}; }



pub fn record_log(level: &str, msg: &str) {

    if let Ok(mut logs) = RAW_LOGS.lock() {

        logs.push(LogEntry {

            level: level.into(),

            message: msg.trim().into(),

            timestamp: Utc::now().format("%H:%M:%S").to_string().into(),

        });

        if logs.len() > 500 { logs.remove(0); }

    }

}

use slint::ComponentHandle;
use slint::Model;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;
use regex::Regex;
use chrono::{Datelike, Utc};
use std::path::{Path, PathBuf};

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
pub mod exporter;
pub mod extractor;
#[cfg(target_os = "android")]
pub mod jni_http;

#[cfg(target_os = "android")]
pub mod android_saf;

#[cfg(target_os = "android")]
use jni::objects::{JObject, JValue};

use crate::api::JobSearchClient;
use crate::db::Db;
use crate::ui::*;
use crate::models::{AdStatus, AppSettings, JobAd, UserDocument, DictEntry, Profile};

use std::sync::mpsc;

/// Globalt aktivt profil-ID (tom sträng = legacy/default)
static CURRENT_PROFILE_ID: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();

fn init_profile_id() -> std::sync::Mutex<String> {
    std::sync::Mutex::new(String::new())
}

fn set_current_profile_id(id: &str) {
    let lock = CURRENT_PROFILE_ID.get_or_init(init_profile_id);
    *lock.lock().unwrap() = id.to_string();
}

fn get_current_profile_id() -> String {
    let lock = CURRENT_PROFILE_ID.get_or_init(init_profile_id);
    lock.lock().unwrap().clone()
}
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

// Global sender for clipboard operations to keep the provider alive on Linux
static CLIPBOARD_SENDER: std::sync::OnceLock<mpsc::Sender<String>> = std::sync::OnceLock::new();

// Log buffer to keep track of recent logs for the UI
static LOG_SENDER: std::sync::OnceLock<mpsc::Sender<String>> = std::sync::OnceLock::new();
static RAW_LOGS: std::sync::Mutex<Vec<LogEntry>> = std::sync::Mutex::new(Vec::new());
static LOCAL_AI: std::sync::OnceLock<crate::ai::LocalAi> = std::sync::OnceLock::new();

struct SlintLogWriter {
    sender: mpsc::Sender<String>,
}

impl std::io::Write for SlintLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(msg) = String::from_utf8(buf.to_vec()) {
            let _ = self.sender.send(msg.clone());
            let level = if msg.contains("ERROR") { "ERROR" } else if msg.contains("WARN") { "WARN" } else { "INFO" };
            record_log(level, &msg);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

/// The Clipboard Manager solves a critical issue on Linux where clipboard content
/// is lost if the application that "owns" the data drops its reference too quickly.
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
                tracing_info!("Text copied to clipboard and kept alive.");
            }
            #[cfg(target_os = "android")]
            {
                // JNI-logik för att nå Androids ClipboardManager
                let ctx = ndk_context::android_context();
                let vm_ptr = ctx.vm();
                let activity = ctx.context();
                
                unsafe {
                    let vm = jni::JavaVM::from_raw(vm_ptr as *mut _).unwrap();
                    let mut env = vm.attach_current_thread().unwrap();
                    let activity_obj = JObject::from_raw(activity as jni::sys::jobject);
                    
                    let cls_context = env.find_class("android/content/Context").unwrap();
                    let field_clipboard_service = env.get_static_field(cls_context, "CLIPBOARD_SERVICE", "Ljava/lang/String;").unwrap().l().unwrap();
                    
                    let clipboard_manager = env.call_method(&activity_obj, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;", &[JValue::Object(&field_clipboard_service)]).unwrap().l().unwrap();
                    
                    let cls_clip_data = env.find_class("android/content/ClipData").unwrap();
                    let label = env.new_string("Jobseeker").unwrap();
                    let text_val = env.new_string(&text).unwrap();
                    
                    let clip_data = env.call_static_method(cls_clip_data, "newPlainText", "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;", &[JValue::Object(&label.into()), JValue::Object(&text_val.into())]).unwrap().l().unwrap();
                    
                    env.call_method(clipboard_manager, "setPrimaryClip", "(Landroid/content/ClipData;)V", &[JValue::Object(&clip_data)]).unwrap();
                    tracing_info!("System: Text kopierad till Android urklipp.");
                }
            }
        }
    });
}

fn copy_to_clipboard(text: String) {
    if let Some(sender) = CLIPBOARD_SENDER.get() {
        let _ = sender.send(text);
    }
}

/// Capture crashes and save to a file for later viewing in the UI.
fn setup_crash_handler() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        let path = get_db_path().with_file_name("crash.log");
        let _ = std::fs::write(path, msg);
    }));
}

// --- Synk Logik (Merging) ---

async fn merge_databases(local: &Db, sync_path: &Path) -> anyhow::Result<()> {
    let local_file = get_db_path();
    let sync_file = sync_path.join("jobseeker.redb");
    let json_file = sync_path.join("jobseeker_sync.json");

    if !sync_file.exists() && !json_file.exists() {
        // Första gången - kopiera DB-filen till synkmappen
        tracing_info!("Synk: Skapar ny databasfil i synkmappen...");
        match std::fs::copy(&local_file, &sync_file) {
            Ok(_) => {
                tracing_info!("Synk: Initial fil kopierad till {}", sync_file.display());
                return Ok(());
            }
            Err(e) => {
                tracing_error!("Synk: Kunde inte kopiera DB: {}. Försöker med JSON...", e);
                // Fallback till JSON
            }
        }
    }

    // Om vi har en JSON-fil, använd den som mellanhand
    if json_file.exists() {
        tracing_info!("Synk: Använder JSON som mellanhand...");
        return merge_via_json(local, &json_file).await;
    }

    // Om vi bara har DB-filen, konvertera till JSON för merge
    tracing_info!("Synk: Konverterar DB till JSON för merge...");
    match export_sync_data(local, &json_file).await {
        Ok(_) => {
            tracing_info!("Synk: Data exporterad till JSON. Kommer använda JSON framöver.");
            return Ok(());
        }
        Err(e) => {
            tracing_error!("Synk: Kunde inte exportera till JSON: {}", e);
            return Err(e);
        }
    }
}

/// Exportera all data från DB till JSON-fil
async fn export_sync_data(db: &Db, json_path: &Path) -> anyhow::Result<()> {
    use serde_json::json;

    let jobs = db.get_filtered_jobs(&[], None, None).await?;
    let docs = db.get_documents().await?;
    let dict = db.get_dictionary().await?;
    let settings = db.load_settings().await?.unwrap_or_default();

    let sync_data = json!({
        "version": "1.0",
        "exported_at": Utc::now().to_rfc3339(),
        "settings": settings,
        "jobs": jobs,
        "documents": docs,
        "dictionary": dict,
    });

    let json_str = serde_json::to_string_pretty(&sync_data)?;
    std::fs::write(json_path, json_str)?;

    tracing_info!("Synk: Exporterade {} jobb, {} dokument till JSON", jobs.len(), docs.len());
    Ok(())
}

/// Importera data från DB-fil eller JSON-fil till tom databas
async fn import_from_sync_folder(db: &Db, sync_path: &Path) -> anyhow::Result<bool> {
    let db_file = sync_path.join("jobseeker.redb");
    let json_file = sync_path.join("jobseeker_sync.json");

    // Om JSON har data, importera den
    if json_file.exists() {
        let json_str = std::fs::read_to_string(&json_file)?;
        let data: serde_json::Value = serde_json::from_str(&json_str)?;

        let has_jobs = data.get("jobs")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        if has_jobs {
            return import_json_data(db, &data).await.map(|_| true);
        }
    }

    // Om DB-fil finns, öppna den med RedB och exportera till JSON, sen importera
    if db_file.exists() {
        tracing_info!("Import: Hittade DB-fil ({} bytes). Öppnar med RedB...", db_file.metadata()?.len());

        match Db::new(db_file.to_str().unwrap()) {
            Ok(remote_db) => {
                // Exportera till JSON
                tracing_info!("Import: Exporterar från DB till JSON...");
                match export_sync_data(&remote_db, &json_file).await {
                    Ok(_) => {
                        tracing_info!("Import: Exporterat! Importerar nu JSON-data...");
                        drop(remote_db);

                        // Importera från den nyligen skapade JSON-filen
                        let json_str = std::fs::read_to_string(&json_file)?;
                        let data: serde_json::Value = serde_json::from_str(&json_str)?;
                        return import_json_data(db, &data).await.map(|_| true);
                    }
                    Err(e) => {
                        tracing_error!("Import: Kunde inte exportera DB till JSON: {}", e);
                        return Ok(false);
                    }
                }
            }
            Err(e) => {
                tracing_error!("Import: Kunde inte öppna DB-filen med RedB: {}", e);
                tracing_error!("Import: Detta beror troligen på SAF-begränsningar.");
                return Ok(false);
            }
        }
    }

    Ok(false)
}

async fn import_json_data(db: &Db, data: &serde_json::Value) -> anyhow::Result<()> {
    let mut stats = (0, 0, 0);

    // Import settings
    if let Some(settings) = data.get("settings") {
        let settings: AppSettings = serde_json::from_value(settings.clone())?;
        db.save_settings(&settings).await?;
        stats.0 += 1;
    }

    // Import jobs
    if let Some(jobs) = data.get("jobs").and_then(|v| v.as_array()) {
        for job_json in jobs {
            let job: JobAd = serde_json::from_value(job_json.clone())?;
            if db.save_job_ad(&job).await.is_ok() {
                stats.1 += 1;
            }
        }
    }

    // Import documents
    if let Some(docs) = data.get("documents").and_then(|v| v.as_array()) {
        for doc_json in docs {
            let doc: UserDocument = serde_json::from_value(doc_json.clone())?;
            if db.save_document(&doc).await.is_ok() {
                stats.2 += 1;
            }
        }
    }

    tracing_info!("Import: Laddade {} jobb, {} dokument från JSON", stats.1, stats.2);
    Ok(())
}

/// Merge data via JSON-fil
async fn merge_via_json(local: &Db, json_path: &Path) -> anyhow::Result<()> {
    use serde_json::Value;

    let json_str = std::fs::read_to_string(json_path)?;
    let data: Value = serde_json::from_str(&json_str)?;

    let mut stats = (0, 0, 0); // (jobs, docs, dict)

    // Merge settings
    if let Some(remote_set) = data.get("settings") {
        let remote_settings: AppSettings = serde_json::from_value(remote_set.clone())?;
        if let Ok(Some(local_set)) = local.load_settings().await {
            if remote_settings.updated_at > local_set.updated_at {
                local.save_settings(&remote_settings).await?;
                tracing_info!("Synk: Uppdaterade inställningar");
            }
        }
    }

    // Merge jobs
    if let Some(remote_jobs) = data.get("jobs").and_then(|v| v.as_array()) {
        for job_json in remote_jobs {
            let remote_job: JobAd = serde_json::from_value(job_json.clone())?;
            if let Ok(Some(local_job)) = local.get_job_ad(&remote_job.id).await {
                if remote_job.updated_at > local_job.updated_at {
                    local.save_job_ad(&remote_job).await?;
                    stats.0 += 1;
                }
            } else {
                local.save_job_ad(&remote_job).await?;
                stats.0 += 1;
            }
        }
    }

    // Merge documents
    if let Some(remote_docs) = data.get("documents").and_then(|v| v.as_array()) {
        let local_docs = local.get_documents().await?;
        for doc_json in remote_docs {
            let remote_doc: UserDocument = serde_json::from_value(doc_json.clone())?;
            if let Some(local_doc) = local_docs.iter().find(|d| d.id == remote_doc.id) {
                if remote_doc.updated_at > local_doc.updated_at {
                    local.save_document(&remote_doc).await?;
                    stats.1 += 1;
                }
            } else {
                local.save_document(&remote_doc).await?;
                stats.1 += 1;
            }
        }
    }

    // Uppdatera JSON-filen med ny data från local
    export_sync_data(local, json_path).await?;

    if stats.0 > 0 || stats.1 > 0 {
        tracing_info!("Synk: Klar. Uppdaterade {} jobb, {} dokument.", stats.0, stats.1);
    } else {
        tracing_info!("Synk: Inga ändringar behövdes.");
    }

    Ok(())
}

async fn trigger_sync(db: &Db, ui_weak: slint::Weak<App>) {
    let pid = get_current_profile_id();
    if let Ok(Some(settings)) = db.load_settings_for(&pid).await {
        let path_raw = settings.sync_path.trim();
        if !path_raw.is_empty() {
            let sync_dir = PathBuf::from(path_raw);
            tracing_info!("Synk: Använder sökpath: {:?}", sync_dir);

            if !sync_dir.exists() {
                tracing_info!("Synk: Skapar synkmapp...");
                if let Err(e) = std::fs::create_dir_all(&sync_dir) {
                    tracing_error!("Synk: Kunde inte skapa mappen: {}", e);
                    return;
                }
            }

            if let Err(e) = merge_databases(db, &sync_dir).await {
                tracing_error!("Synk: Misslyckades ({}). Välj en annan mapp eller använd app-mappen.", e);
            } else {
                tracing_info!("Synk: Genomförd mot {:?}", sync_dir);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_msg("Synk genomförd".into());
                    }
                });
            }
        }
    }
}

// --- Hjälpfunktioner ---

fn get_folder_entries(path: &Path) -> Vec<FolderEntry> {
    let mut entries = Vec::new();
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path_str = entry.path().to_string_lossy().to_string();
                    entries.push(FolderEntry { name: name.into(), path: path_str.into() });
                }
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

fn setup_logging() -> (Option<tracing_appender::non_blocking::WorkerGuard>, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel();
    let _ = LOG_SENDER.set(tx.clone());

    #[cfg(not(target_os = "android"))]
    {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,slint=warn"));
        let registry = tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout).with_ansi(true));
        registry.init();
        (None, rx)
    }
    #[cfg(target_os = "android")]
    {
        use tracing::Subscriber;
        struct NoOpSubscriber;
        impl Subscriber for NoOpSubscriber {
            fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool { false }
            fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id { tracing::span::Id::from_u64(0) }
            fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}
            fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}
            fn event(&self, _event: &tracing::Event<'_>) {}
            fn enter(&self, _span: &tracing::span::Id) {}
            fn exit(&self, _span: &tracing::span::Id) {}
        }
        let _ = tracing::subscriber::set_global_default(NoOpSubscriber);
        (None, rx)
    }
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

fn get_db_path() -> PathBuf {
    #[cfg(target_os = "android")]
    { let path = PathBuf::from("/data/data/com.gnawsoftware.jobseeker/files"); let _ = std::fs::create_dir_all(&path); return path.join("jobseeker.redb"); }
    #[cfg(not(target_os = "android"))]
    { directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker").map(|p| { let d = p.data_dir(); let _ = std::fs::create_dir_all(d); d.join("jobseeker.redb") }).unwrap_or_else(|| PathBuf::from("jobseeker.redb")) }
}

fn normalize_locations(input: &str) -> String {
    input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| {
        if s.chars().all(char::is_numeric) { JobSearchClient::get_municipality_name(s).unwrap_or_else(|| s.to_string()) }
        else { let mut chars = s.chars(); match chars.next() { None => String::new(), Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str() } }
    }).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ")
}

// --- UI Setup ---

fn setup_ui(ui: &App, rt: Arc<Runtime>, db: Arc<Db>, log_rx: mpsc::Receiver<String>) {
    let ui_weak = ui.as_weak();
    spawn_log_task(ui_weak.clone(), log_rx);

    // Check for previous crash
    let crash_file = get_db_path().with_file_name("crash.log");
    if crash_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&crash_file) {
            ui.set_system_logs(format!("PREVIOUS CRASH DETECTED:\n\n{}\n\n---\n\n", content).into());
            ui.set_status_msg("Appen kraschade senast. Se loggar i Inställningar.".into());
        }
        let _ = std::fs::remove_file(crash_file);
    }

    // --- Profilinitiering ---
    let db_profiles = db.clone();
    let ui_profiles = ui.as_weak();
    rt.spawn(async move {
        let profiles = db_profiles.get_profiles().await.unwrap_or_default();
        if profiles.is_empty() {
            // Skapa standardprofil
            let default_profile = Profile {
                id: "default".to_string(),
                name: "Default".to_string(),
                icon: "👤".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let _ = db_profiles.save_profile(&default_profile).await;
            set_current_profile_id("default");
            let _ = db_profiles.set_active_profile_id("default").await;
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_profiles.upgrade() {
                    ui.set_current_profile_id("default".into());
                    ui.set_current_profile_name("Default".into());
                    ui.set_current_profile_icon("👤".into());
                }
            });
        } else if profiles.len() == 1 {
            set_current_profile_id(&profiles[0].id);
            let _ = db_profiles.set_active_profile_id(&profiles[0].id).await;
            let p = profiles[0].clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_profiles.upgrade() {
                    ui.set_current_profile_id(p.id.into());
                    ui.set_current_profile_name(p.name.into());
                    ui.set_current_profile_icon(p.icon.into());
                }
            });
        } else {
            // Flera profiler — visa profilväljare
            let ui_picker = ui_profiles.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_picker.upgrade() {
                    let profile_infos: Vec<ProfileInfo> = profiles.iter().map(|p| ProfileInfo {
                        id: p.id.clone().into(),
                        name: p.name.clone().into(),
                        icon: p.icon.clone().into(),
                    }).collect();
                    ui.set_profiles(std::rc::Rc::new(slint::VecModel::from(profile_infos)).into());
                    ui.set_show_profile_picker(true);
                }
            });
        }
    });

    // --- Profil-callbacks ---
    let db_sel = db.clone();
    let rt_sel = rt.clone();
    let ui_sel = ui.as_weak();
    ui.on_select_profile(move |id| {
        let db = db_sel.clone();
        let ui_weak = ui_sel.clone();
        let rt = rt_sel.clone();
        let profile_id = id.to_string();
        rt.spawn(async move {
            set_current_profile_id(&profile_id);
            let _ = db.set_active_profile_id(&profile_id).await;
            if let Ok(Some(profile)) = db.get_profile(&profile_id).await {
                let p = profile.clone();
                if let Ok(Some(settings)) = db.load_settings_for(&profile_id).await {
                    let s = settings.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_current_profile_id(p.id.clone().into());
                            ui.set_current_profile_name(p.name.clone().into());
                            ui.set_current_profile_icon(p.icon.clone().into());
                            ui.set_settings(crate::ui::AppSettings {
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
                                show_motivation: s.show_motivation,
                                main_cv_id: s.main_cv_id.into(),
                                show_dev_logs: s.show_dev_logs,
                                auto_extract: s.auto_extract,
                            });
                        }
                    });
                }
            }
        });
    });

    let db_create = db.clone();
    let rt_create = rt.clone();
    let ui_create = ui.as_weak();
    ui.on_create_profile(move |name| {
        let db = db_create.clone();
        let ui_weak = ui_create.clone();
        let rt = rt_create.clone();
        let name_str = name.to_string();
        rt.spawn(async move {
            let id = format!("profile_{}", Utc::now().timestamp());
            let new_profile = Profile {
                id: id.clone(),
                name: name_str,
                icon: "👤".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let _ = db.save_profile(&new_profile).await;
            // Ladda om profillistan
            if let Ok(profiles) = db.get_profiles().await {
                let infos: Vec<ProfileInfo> = profiles.iter().map(|p| ProfileInfo {
                    id: p.id.clone().into(),
                    name: p.name.clone().into(),
                    icon: p.icon.clone().into(),
                }).collect();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_profiles(std::rc::Rc::new(slint::VecModel::from(infos)).into());
                    }
                });
            }
        });
    });

    let db_del = db.clone();
    let ui_del = ui.as_weak();
    ui.on_delete_profile(move |id| {
        let db = db_del.clone();
        let ui_weak = ui_del.clone();
        let id_str = id.to_string();
        tokio::spawn(async move {
            let _ = db.delete_profile(&id_str).await;
            if let Ok(profiles) = db.get_profiles().await {
                let infos: Vec<ProfileInfo> = profiles.iter().map(|p| ProfileInfo {
                    id: p.id.clone().into(),
                    name: p.name.clone().into(),
                    icon: p.icon.clone().into(),
                }).collect();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_profiles(std::rc::Rc::new(slint::VecModel::from(infos)).into());
                    }
                });
            }
        });
    });

    let ui_switch = ui.as_weak();
    let db_switch = db.clone();
    let rt_switch = rt.clone();
    ui.on_switch_profile(move || {
        let ui_weak = ui_switch.clone();
        let db = db_switch.clone();
        let rt = rt_switch.clone();
        rt.spawn(async move {
            if let Ok(profiles) = db.get_profiles().await {
                let infos: Vec<ProfileInfo> = profiles.iter().map(|p| ProfileInfo {
                    id: p.id.clone().into(),
                    name: p.name.clone().into(),
                    icon: p.icon.clone().into(),
                }).collect();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_profiles(std::rc::Rc::new(slint::VecModel::from(infos)).into());
                        ui.set_show_profile_picker(true);
                    }
                });
            }
        });
    });

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
                let pid = get_current_profile_id();
                if let Ok(ads) = db.get_filtered_jobs_for(&pid, &[], Some(year), Some(month)).await {
                    let total_count = ads.len() as i32;
                    let (mut applied, mut bookmarked, mut thumbsup, mut rejected) = (0, 0, 0, 0);
                    let mut counts = std::collections::HashMap::new();
                    for ad in ads {
                        match ad.status {
                            Some(AdStatus::Applied) => applied += 1,
                            Some(AdStatus::Bookmarked) => bookmarked += 1,
                            Some(AdStatus::ThumbsUp) => thumbsup += 1,
                            Some(AdStatus::Rejected) => rejected += 1,
                            _ => {} 
                        }
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
                let pid = get_current_profile_id();
                let settings = db.load_settings_for(&pid).await.unwrap_or_default().unwrap_or_default();
                
                let mut report = format!("AKTIVITETSRAPPORT - {}\n==========================================\n\n", month_display.to_uppercase());
                if include_params {
                    report.push_str(&format!("SÖKPARAMETRAR:\n• Sökord: {}\n• Prio 1: {}\n• Prio 2: {}\n\n", settings.keywords, normalize_locations(&settings.locations_p1), normalize_locations(&settings.locations_p2)));
                }
                if include_jobs {
                    if let Ok(ads) = db.get_filtered_jobs_for(&pid, &[AdStatus::Applied], Some(year), Some(month)).await {
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
                    if let Ok(ads) = db.get_filtered_jobs_for(&pid, &[], Some(year), Some(month)).await {
                        let app = ads.iter().filter(|a| a.status == Some(AdStatus::Applied)).count();
                        let rej = ads.iter().filter(|a| a.status == Some(AdStatus::Rejected)).count();
                        report.push_str(&format!("AKTIVITETSANALYS:\n• Totalt granskade: {}\n• Konvertering: {} sökta, {} avvisade\n", ads.len(), app, rej));
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
                        let mailto = format!("mailto:?subject={}&body={}", subject, urlencoding::encode(&body_text));
                        let _ = webbrowser::open(&mailto);
                        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Öppnar e-post (rapport kopierad till urklipp)".into()); } });
                    }
                } else if method == "file" {
                    let file_name = format!("jobb-rapport-{}.txt", month_str);
                    let file_path = directories::UserDirs::new().and_then(|u| u.download_dir().map(|d| d.join(&file_name))).unwrap_or_else(|| PathBuf::from(&file_name));
                    if std::fs::write(&file_path, report).is_ok() {
                        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg(format!("Rapport sparad: {}", file_name).into()); } });
                    }
                }
            });
        }
    });

    // Callback: Month Offset
    let (db_month, rt_month, ui_month, rs_month) = (db.clone(), rt.clone(), ui.as_weak(), refresh_stats.clone());
    ui.on_month_offset(move |offset| {
        tracing_info!("UI: Byter månad med offset {}", offset);
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
                let pid = get_current_profile_id();
                if let Ok(ads) = db.get_filtered_jobs_for(&pid, &[], Some(ny), Some(nm as u32)).await {
                    let app_count = ads.iter().filter(|ad| ad.status == Some(AdStatus::Applied)).count() as i32;
                    let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
                    let entries: Vec<JobEntry> = ads.into_iter().map(|ad| {
                        let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                        let formatted_desc = raw_desc.replace("<li>", "\n • ").replace("</li>", "").replace("<ul>", "\n").replace("</ul>", "\n").replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("<p>", "\n\n").replace("</p>", "").replace("<strong>", "").replace("</strong>", "").replace("<b>", "").replace("</b>", "");
                        let mut clean_desc = re_html.replace_all(&formatted_desc, "").to_string();
                        if ad.driving_license_required { clean_desc.push_str("\n\nKÖRKORT:\n • Krav på körkort\n"); }
                        JobEntry { id: ad.id.into(), title: ad.headline.into(), employer: ad.employer.and_then(|e| e.name).unwrap_or_default().into(), location: ad.workplace_address.and_then(|a| a.city).unwrap_or_default().into(), description: clean_desc.into(), date: ad.publication_date.split('T').next().unwrap_or("").into(), apply_url: ad.application_details.and_then(|d| d.url).unwrap_or_default().into(), rating: ad.rating.unwrap_or(0) as i32, status: match ad.status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 }, status_text: "".into(), ai_summary: "".into() }
                    }).collect();
                    let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_f.upgrade() { ui.set_jobs(Rc::new(slint::VecModel::from(entries)).into()); ui.set_applied_count(app_count); } });
                }
            });
        }
    });

    // Callback: Free Search
    let (api_s, db_s, ui_s, rt_s) = (Arc::new(JobSearchClient::new()), db.clone(), ui.as_weak(), rt.clone());
    ui.on_search_pressed(move |q| { 
        let (api, db, ui_weak, q_str) = (api_s.clone(), db_s.clone(), ui_s.clone(), q.to_string()); 
        tracing_info!("UI: Fri sökning på '{}'", q_str);
        rt_s.spawn(async move { 
            let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default(); 
            perform_search(api, db, ui_weak, None, Some(q_str), settings).await; 
        }); 
    });

    // Callback: Prio Search
    let (api_p, db_p, ui_p, rt_p) = (Arc::new(JobSearchClient::new()), db.clone(), ui.as_weak(), rt.clone());
    ui.on_search_prio(move |p| { 
        let (api, db, ui_weak) = (api_p.clone(), db_p.clone(), ui_p.clone()); 
        tracing_info!("UI: Prio-sökning på P{}", p);
        rt_p.spawn(async move { 
            let settings = db.load_settings().await.unwrap_or_default().unwrap_or_default(); 
            perform_search(api, db, ui_weak, Some(p), None, settings).await; 
        }); 
    });

    let ui_job_sel = ui.as_weak();
    ui.on_job_selected(move |id, idx| {
        tracing_info!("UI: Jobb valt: {} (index {})", id, idx);
        let ui_weak = ui_job_sel.clone();
        let id_str = id.to_string();
        
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if ui.get_settings().auto_extract {
                    let jobs = ui.get_jobs();
                    if let Some(job) = jobs.iter().nth(idx as usize) {
                        if job.ai_summary.is_empty() {
                            // Trigger analysis if not already done
                            ui.invoke_job_action(id_str.into(), "analyze".into());
                        }
                    }
                }
            }
        }).unwrap();
    });

    // Callback: Job Action
    let (db_a, ui_a, rt_a) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_job_action(move |id, act| {
        let (db, ui_weak, id_str, action) = (db_a.clone(), ui_a.clone(), id.to_string(), act.to_string());
        tracing_info!("UI: Utför åtgärd '{}' på jobb {}", action, id_str);
        rt_a.spawn(async move {
            if action == "open" || action == "apply_direct" { if let Ok(Some(ad)) = db.get_job_ad(&id_str).await { let url = if action == "open" { ad.webpage_url } else { ad.application_details.and_then(|d| d.url) }; if let Some(u) = url { let _ = webbrowser::open(&u); } } return; }
            
            if action == "analyze" {
                if let Ok(Some(ad)) = db.get_job_ad(&id_str).await {
                    let desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).cloned().unwrap_or_default();
                    tracing_info!("System: Startar automatisk extraktion av kärnan...");
                    
                    let ai = LOCAL_AI.get_or_init(|| {
                        tracing_info!("System: Initierar FastExtractor-algoritm...");
                        crate::ai::LocalAi::new().unwrap()
                    });

                    if let Ok(summary) = ai.extractive_summarize(&desc) {
                        let summary_shared: slint::SharedString = summary.into();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                let jobs = ui.get_jobs();
                                let mut vec: Vec<JobEntry> = jobs.iter().collect();
                                if let Some(pos) = vec.iter().position(|j| j.id == id_str) {
                                    vec[pos].ai_summary = summary_shared;
                                    ui.set_jobs(Rc::new(slint::VecModel::from(vec)).into());
                                    ui.set_status_msg("Analys klar".into());
                                }
                            }
                        });
                    }
                }
                return;
            }

            let target = match action.as_str() { "reject" => AdStatus::Rejected, "save" => AdStatus::Bookmarked, "thumbsup" => AdStatus::ThumbsUp, "apply" => AdStatus::Applied, _ => return };
            let current = db.get_job_ad(&id_str).await.ok().flatten().and_then(|ad| ad.status);
            let new_status = if current == Some(target) { None } else { Some(target) };
            if db.update_ad_status(&id_str, new_status).await.is_ok() {
                trigger_sync(&db, ui_weak.clone()).await;
                let status_int = match new_status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 };
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { let jobs = ui.get_jobs(); let mut vec: Vec<JobEntry> = jobs.iter().collect(); if let Some(pos) = vec.iter().position(|j| j.id == id_str) { if status_int == 1 { vec.remove(pos); } else { vec[pos].status = status_int; } ui.set_jobs(Rc::new(slint::VecModel::from(vec)).into()); } } });
            }
        });
    });

    ui.on_copy_text(|t| copy_to_clipboard(t.to_string()));

    // Callback: Save Settings
    let (db_set, ui_set, rt_set) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_save_settings(move |s| {
        let (db, ui_weak) = (db_set.clone(), ui_set.clone());
        let settings = AppSettings {
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
            show_motivation: s.show_motivation,
            main_cv_id: s.main_cv_id.to_string(),
            show_dev_logs: s.show_dev_logs,
            auto_extract: s.auto_extract,
            updated_at: Utc::now(),
            profile_id: get_current_profile_id(),
        };
        let s_ui = settings.clone();
        rt_set.spawn(async move {
            if db.save_settings(&settings).await.is_ok() {
                trigger_sync(&db, ui_weak.clone()).await;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_settings(crate::ui::AppSettings {
                            keywords: s_ui.keywords.into(),
                            blacklist_keywords: s_ui.blacklist_keywords.into(),
                            locations_p1: normalize_locations(&s_ui.locations_p1).into(),
                            locations_p2: normalize_locations(&s_ui.locations_p2).into(),
                            locations_p3: normalize_locations(&s_ui.locations_p3).into(),
                            my_profile: s_ui.my_profile.into(),
                            ollama_url: s_ui.ollama_url.into(),
                            sync_path: s_ui.sync_path.into(),
                            app_min_count: s_ui.app_min_count,
                            app_goal_count: s_ui.app_goal_count,
                            show_motivation: s_ui.show_motivation,
                            main_cv_id: s_ui.main_cv_id.into(),
                            show_dev_logs: s_ui.show_dev_logs,
                            auto_extract: s_ui.auto_extract,
                        });
                        ui.set_status_msg("Sparat & Synkat".into());
                    }
                });
            }
        });
    });

    // Callback: Database Action (Synk/Backup)
    let ui_db = ui.as_weak();
    let db_db = db.clone();
    let rt_db = rt.clone();
    ui.on_db_action(move |act| {
        let (db, ui_weak, rt) = (db_db.clone(), ui_db.clone(), rt_db.clone());
        if act == "backup" {
            let db_path = get_db_path();
            let backup_name = format!("backup_{}.redb", Utc::now().format("%Y%m%d_%H%M"));
            let backup_path = PathBuf::from(&backup_name);
            if std::fs::copy(&db_path, &backup_path).is_ok() {
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg(format!("Backup: {}", backup_name).into()); } });
            }
        } else if act == "sync" {
            rt.spawn(async move {
                trigger_sync(&db, ui_weak.clone()).await;
            });
        }
    });

    // Callback: Pick Sync Path
    let ui_pick = ui.as_weak();
    ui.on_pick_sync_path(move || {
        let ui_weak = ui_pick.clone();
        #[cfg(not(target_os = "android"))]
        {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                let path_str = path.to_string_lossy().to_string();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut s = ui.get_settings();
                        s.sync_path = path_str.into();
                        ui.set_settings(s);
                    }
                });
            }
        }
        #[cfg(target_os = "android")]
        {
            // Testa om vi redan har write access
            let test_path = PathBuf::from("/sdcard/Documents/.jobseeker_test.tmp");
            let has_permission = std::fs::write(&test_path, b"test").is_ok();
            let _ = std::fs::remove_file(&test_path);

            if !has_permission {
                // Öppna inställningarna för att be om tillstånd
                if let Err(e) = crate::android_saf::request_all_files_access() {
                    tracing_error!("Kunde inte öppna inställningar: {}", e);
                }
            }

            // Visa filbläddrare om vi har tillstånd, annars instruktion
            if has_permission {
                // Visa filbläddraren - användaren väljer mapp
                let start_path = "/sdcard";
                let entries = get_folder_entries(Path::new(start_path));
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_current_folder_path(start_path.into());
                        ui.set_folder_entries(Rc::new(slint::VecModel::from(entries)).into());
                        ui.set_show_folder_picker(true);
                    }
                });
            } else {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_msg("Öppnar inställningar. Tryck på Tillåt åtkomst till alla filer där, sedan välj mapp igen.".into());
                    }
                });
            }
        }
    });

    // Callback: Custom Folder Picker Navigation
    let ui_fold = ui.as_weak();
    ui.on_select_folder(move |path| {
        let (ui_weak, p_str) = (ui_fold.clone(), path.to_string());
        let entries = get_folder_entries(Path::new(&p_str));
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_current_folder_path(p_str.into()); ui.set_folder_entries(Rc::new(slint::VecModel::from(entries)).into()); } });
    });

    let ui_create = ui.as_weak();
    ui.on_create_folder(move |parent, name| {
        let (ui_weak, p_str, n_str) = (ui_create.clone(), parent.to_string(), name.to_string());
        let target = PathBuf::from(&p_str).join(&n_str);
        if std::fs::create_dir_all(&target).is_ok() {
            let entries = get_folder_entries(Path::new(&p_str));
            let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_folder_entries(Rc::new(slint::VecModel::from(entries)).into()); } });
        }
    });

    let ui_back = ui.as_weak();
    ui.on_folder_go_back(move || {
        let ui_weak = ui_back.clone();
        if let Some(ui) = ui_weak.upgrade() {
            let current = PathBuf::from(ui.get_current_folder_path().to_string());
            if let Some(parent) = current.parent() {
                let p_str = parent.to_string_lossy().to_string();
                let entries = get_folder_entries(parent);
                ui.set_current_folder_path(p_str.into());
                ui.set_folder_entries(Rc::new(slint::VecModel::from(entries)).into());
            }
        }
    });

    // Callback: Logging system
    let ui_logs = ui.as_weak();
    ui.on_request_logs(move |filter| {
        let filter = filter.to_string();
        let logs = RAW_LOGS.lock().unwrap().iter().filter(|l| filter == "ALL" || l.level == filter).cloned().collect::<Vec<_>>();
        let ui_clone = ui_logs.clone();
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_clone.upgrade() { ui.set_static_logs(Rc::new(slint::VecModel::from(logs)).into()); } });
    });

    let db_log_save = db.clone();
    ui.on_save_logs_to_file(move || {
        let db = db_log_save.clone();
        tokio::spawn(async move {
            if let Ok(Some(settings)) = db.load_settings().await {
                if !settings.sync_path.is_empty() {
                    let path = PathBuf::from(settings.sync_path).join(format!("jobseeker_log_{}.txt", Utc::now().format("%Y%m%d_%H%M")));
                    let logs = RAW_LOGS.lock().unwrap().iter().map(|l| format!("[{}] {}: {}", l.timestamp, l.level, l.message)).collect::<Vec<_>>().join("\n");
                    let _ = std::fs::write(path, logs);
                }
            }
        });
    });

    // Callback: Select document
    let (db_sel, ui_sel, rt_sel) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_select_doc(move |id| {
        let db = db_sel.clone();
        let ui_weak = ui_sel.clone();
        let id_str = id.to_string();
        rt_sel.spawn(async move {
            if let Ok(docs) = db.get_documents().await {
                if let Some(doc) = docs.iter().find(|d| d.id == id_str).cloned() {
                    let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_selected_doc_content(doc.content.clone().into()); } });
                }
            }
        });
    });

    // Callback: Save document
    let (db_save, ui_save, rt_save) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_save_doc(move |id, content| {
        let db = db_save.clone();
        let ui_weak = ui_save.clone();
        let id_str = id.to_string();
        let content_str = content.to_string();
        rt_save.spawn(async move {
            if let Ok(mut docs) = db.get_documents().await {
                if let Some(doc) = docs.iter_mut().find(|d| d.id == id_str) {
                    doc.content = content_str.clone();
                    let _ = db.save_document(doc).await;
                    trigger_sync(&db, ui_weak.clone()).await;
                    let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Dokument sparat & synkat".into()); } });
                }
            }
        });
    });

    // Callback: Delete document
    let (db_del, ui_del, rt_del) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_delete_doc(move |id| {
        let db = db_del.clone();
        let ui_weak = ui_del.clone();
        let id_str = id.to_string();
        rt_del.spawn(async move {
            let _ = db.delete_document(&id_str).await;
            trigger_sync(&db, ui_weak.clone()).await;
            if let Ok(docs) = db.get_documents().await {
                let entries: Vec<DocEntry> = docs.into_iter().map(|d| DocEntry { id: d.id.into(), name: d.name.into(), doc_type: d.doc_type.into(), is_main: d.is_main }).collect();
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_documents(Rc::new(slint::VecModel::from(entries)).into()); ui.set_status_msg("Dokument raderat".into()); } });
            }
        });
    });

    // Callback: Export document
    let (db_exp, ui_exp, rt_exp) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_export_doc(move |id, format| {
        let db = db_exp.clone();
        let ui_weak = ui_exp.clone();
        let id_str = id.to_string();
        let format_str = format.to_string();
        rt_exp.spawn(async move {
            if let Ok(docs) = db.get_documents().await {
                if let Some(doc) = docs.iter().find(|d| d.id == id_str).cloned() {
                    let export_dir = directories::UserDirs::new().and_then(|u| u.download_dir().map(|p| p.to_path_buf())).unwrap_or(PathBuf::from("."));
                    let file_name = format!("{}.{}", doc.name.replace('/', "_"), if format_str == "pdf" { "pdf" } else { "md" });
                    let file_path = export_dir.join(&file_name);
                    let result = if format_str == "pdf" { crate::exporter::export_doc_to_pdf(&doc.name, &doc.content, &file_path) } else { crate::exporter::export_doc_to_md(&doc.content, &file_path) };
                    match result {
                        Ok(_) => { let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg(format!("Exporterat: {}", file_name).into()); } }); }
                        Err(e) => { let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg(format!("Export misslyckades: {}", e).into()); } }); }
                    }
                }
            }
        });
    });

    // Callback: Set as main CV
    let (db_main, ui_main, rt_main) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_set_as_main(move |id| {
        let db = db_main.clone();
        let ui_weak = ui_main.clone();
        let id_str = id.to_string();
        rt_main.spawn(async move {
            let _ = db.set_main_cv(&id_str).await;
            trigger_sync(&db, ui_weak.clone()).await;
            let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Huvud-CV uppdaterat".into()); } });
        });
    });

    // Callback: Add dictionary entry
    let (db_add, ui_add, rt_add) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_add_entry(move |key, value| {
        let db = db_add.clone();
        let ui_weak = ui_add.clone();
        let key_str = key.to_string();
        let value_str = value.to_string();
        rt_add.spawn(async move {
            let pid = get_current_profile_id();
            let entry = DictEntry { key: key_str, value: value_str, updated_at: Utc::now(), profile_id: pid };
            let _ = db.save_dict_entry(&entry).await;
            trigger_sync(&db, ui_weak.clone()).await;
            let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Ord tillagt i ordboken".into()); } });
        });
    });

    // Callback: Delete dictionary entry
    let (db_del_dict, ui_del_dict, rt_del_dict) = (db.clone(), ui.as_weak(), rt.clone());
    ui.on_delete_entry(move |key| {
        let db = db_del_dict.clone();
        let ui_weak = ui_del_dict.clone();
        let key_str = key.to_string();
        rt_del_dict.spawn(async move {
            let _ = db.delete_dict_entry(&key_str).await;
            trigger_sync(&db, ui_weak.clone()).await;
            let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_status_msg("Ord raderat".into()); } });
        });
    });

    // Initial laddning (profil-medveten)
    let (db_i, ui_i, rt_i) = (db.clone(), ui.as_weak(), rt.clone());
    let db_path_str = get_db_path().to_string_lossy().to_string();
    let rt_i_initial = rt_i.clone();
    rt_i_initial.spawn(async move {
        trigger_sync(&db_i, ui_i.clone()).await;
        let pid = get_current_profile_id();
        let settings = db_i.load_settings_for(&pid).await.unwrap_or_default().unwrap_or_default();
        let (s, u_s) = (settings.clone(), ui_i.clone());
        let d_path = db_path_str.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = u_s.upgrade() {
                ui.set_database_path(d_path.into());
                ui.set_settings(crate::ui::AppSettings {
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
                    show_motivation: s.show_motivation,
                    main_cv_id: s.main_cv_id.into(),
                    show_dev_logs: s.show_dev_logs,
                    auto_extract: s.auto_extract,
                });
            }
        });

        let db_docs = db_i.clone();
        let ui_docs = ui_i.clone();
        let rt_docs = rt_i.clone();
        let pid_docs = pid.clone();
        rt_docs.spawn(async move {
            if let Ok(docs) = db_docs.get_documents_for(&pid_docs).await {
                let entries: Vec<DocEntry> = docs.into_iter().map(|d| DocEntry { id: d.id.into(), name: d.name.into(), doc_type: d.doc_type.into(), is_main: d.is_main }).collect();
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_docs.upgrade() { ui.set_documents(Rc::new(slint::VecModel::from(entries)).into()); } });
            }
        });

        let db_dict = db_i.clone();
        let ui_dict = ui_i.clone();
        let rt_dict = rt_i.clone();
        let pid_dict = pid.clone();
        rt_dict.spawn(async move {
            if let Ok(entries) = db_dict.get_dict_entries_for(&pid_dict).await {
                let dict_entries: Vec<crate::ui::DictEntry> = entries.into_iter().map(|e| crate::ui::DictEntry { key: e.key.into(), value: e.value.into() }).collect();
                let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_dict.upgrade() { ui.set_dictionary(Rc::new(slint::VecModel::from(dict_entries)).into()); } });
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

async fn perform_search(api_client: Arc<JobSearchClient>, db: Arc<Db>, ui_weak: slint::Weak<App>, prio: Option<i32>, free_query: Option<String>, settings: AppSettings) {
    let now = chrono::Utc::now();
    let (y, m) = if let Some(ui) = ui_weak.upgrade() { let month_str = ui.get_active_month().to_string(); let parts: Vec<&str> = month_str.split('-').collect(); if parts.len() == 2 { (parts[0].parse().unwrap_or(now.year()), parts[1].parse().unwrap_or(now.month())) } else { (now.year(), now.month()) } } else { (now.year(), now.month()) };
    let (raw_query, locations_str) = match (free_query.clone(), prio) { (Some(q), _) => (q, String::new()), (None, Some(p)) => { let locs = match p { 1 => &settings.locations_p1, 2 => &settings.locations_p2, 3 => &settings.locations_p3, _ => &settings.locations_p1 }; (settings.keywords.clone(), locs.clone()) }, _ => (String::new(), String::new()) };
    let municipalities = JobSearchClient::parse_locations(&locations_str);
    let query_parts: Vec<_> = raw_query.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.replace("\"", "")).collect();
    let ui_early = ui_weak.clone(); let p_early = prio;
    let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_early.upgrade() { ui.set_searching(true); ui.set_status_msg(format!("Söker efter nytt... (Visar sparade jobb för P{}", p_early.unwrap_or(0)).into()); } });

    let refresh_ui_from_db = |ui: &App, ads: Vec<crate::models::JobAd>, p: Option<i32>, muns: Vec<String>, msg: String| {
        let re_html = Regex::new(r"<[^>]*>").expect("Invalid regex");
        let pmn: Vec<String> = if p.is_some() { muns.iter().filter_map(|code| JobSearchClient::get_municipality_name(code)).map(|s| s.to_lowercase()).collect() } else { Vec::new() };
        let applied_count = ads.iter().filter(|ad| ad.status == Some(AdStatus::Applied)).count() as i32;
        let mut entries: Vec<JobEntry> = ads.into_iter().filter(|ad| { if !pmn.is_empty() { if let Some(ref addr) = ad.workplace_address { if let Some(ref mun) = addr.municipality { return pmn.contains(&mun.to_lowercase()); } } return false; } true }).map(|ad| {
            let raw_desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
            let formatted_desc = raw_desc.replace("<li>", "\n • ").replace("</li>", "").replace("<ul>", "\n").replace("</ul>", "\n").replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n").replace("<p>", "\n\n").replace("</p>", "").replace("<strong>", "").replace("</strong>", "").replace("<b>", "").replace("</b>", "");
            let mut clean_desc = re_html.replace_all(&formatted_desc, "").to_string();
            if ad.driving_license_required { clean_desc.push_str("\n\nKÖRKORT:\n • Krav på körkort\n"); }
            JobEntry { id: ad.id.into(), title: ad.headline.into(), employer: ad.employer.and_then(|e| e.name).unwrap_or_default().into(), location: ad.workplace_address.and_then(|a| a.city).unwrap_or_default().into(), description: clean_desc.into(), date: ad.publication_date.split('T').next().unwrap_or("").into(), apply_url: ad.application_details.and_then(|d| d.url).unwrap_or_default().into(), rating: ad.rating.unwrap_or(0) as i32, status: match ad.status { Some(AdStatus::Rejected) => 1, Some(AdStatus::Bookmarked) => 2, Some(AdStatus::ThumbsUp) => 3, Some(AdStatus::Applied) => 4, _ => 0 }, status_text: "".into(), ai_summary: "".into() }
        }).collect();
        entries.sort_by(|a, b| b.date.cmp(&a.date));
        ui.set_jobs(std::rc::Rc::new(slint::VecModel::from(entries)).into()); ui.set_applied_count(applied_count); ui.set_status_msg(msg.into());
    };

    let profile_id_for_search = get_current_profile_id();
    if let Ok(existing_ads) = db.get_filtered_jobs_for(&profile_id_for_search, &[], Some(y), Some(m)).await {
        let ui_e2 = ui_weak.clone(); let muns_e2 = municipalities.clone(); let loc_d = locations_str.clone();
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_e2.upgrade() { let msg = format!("Visar sparade jobb för {}. Söker efter nytt...", loc_d); refresh_ui_from_db(&ui, existing_ads, prio, muns_e2, msg); } });
    }

    let mut new_count = 0; let blacklist: Vec<String> = settings.blacklist_keywords.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    for keyword in &query_parts {
        match api_client.search(keyword, &municipalities, 100).await {
            Ok(ads) => { for mut ad in ads { 
                ad.search_keyword = Some(keyword.clone());
                ad.profile_id = profile_id_for_search.clone(); // Sätt profil på nya annonser
                let is_blacklisted = blacklist.iter().any(|word| ad.headline.to_lowercase().contains(word) || ad.description.as_ref().and_then(|d| d.text.as_deref()).map(|t| t.to_lowercase().contains(word)).unwrap_or(false)); 
                if !is_blacklisted { if let Ok(None) = db.get_job_ad(&ad.id).await { if db.save_job_ad(&ad).await.is_ok() { new_count += 1; } } } 
            } },
            Err(e) => { tracing_error!("Sökning på '{}' misslyckades: {}", keyword, e); }
        }
    }

    if let Ok(final_ads) = db.get_filtered_jobs_for(&profile_id_for_search, &[], Some(y), Some(m)).await {
        trigger_sync(&db, ui_weak.clone()).await;
        tracing_info!("Search: Klart! Hittade {} nya unika annonser för denna månad", new_count);
        let ui_f = ui_weak.clone(); let muns_f = municipalities.clone();
        let msg = if new_count > 0 { format!("Klar! Hittade {} nya annonser.", new_count) } else { "Inga nya annonser hittades just nu.".to_string() };
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_f.upgrade() { refresh_ui_from_db(&ui, final_ads, prio, muns_f, msg); ui.set_searching(false); } });
    } else {
        let _ = slint::invoke_from_event_loop(move || { if let Some(ui) = ui_weak.upgrade() { ui.set_searching(false); } });
    }
}

pub fn desktop_main() {
    setup_crash_handler();
    let (guard, log_rx) = setup_logging();
    setup_clipboard_manager();
    tracing_info!("Starting Jobseeker on Desktop");
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let db_path = get_db_path();
    let db = rt.block_on(async { Db::new(db_path.to_str().unwrap()) }).expect("Failed to initialize database");
    let db = Arc::new(db);
    let ui = App::new().expect("Failed to create Slint UI");
    #[cfg(target_os = "android")]
    {
        ui.set_has_mouse_wheel(false);
        ui.set_is_android(true);
    }
    #[cfg(not(target_os = "android"))]
    {
        ui.set_has_mouse_wheel(true);
        ui.set_is_android(false);
    }

    setup_ui(&ui, rt, db, log_rx);
    let _log_guard = guard;
    ui.run().expect("Failed to run Slint UI");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
unsafe fn android_main(app: slint::android::AndroidApp) {
    let files_dir = PathBuf::from("/data/data/com.gnawsoftware.jobseeker/files");
    let _ = std::fs::create_dir_all(&files_dir);
    let (guard, log_rx) = setup_logging();
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("Jobseeker"));
    tracing_info!("Starting Jobseeker on Android (Pure Rust)");
    setup_crash_handler();

    // Initiera JNI VM för SAF
    let vm_ptr = app.vm_as_ptr();
    if let Ok(vm) = unsafe { jni::JavaVM::from_raw(vm_ptr as *mut _) } {
        crate::android_saf::init_vm(vm);
    }

    slint::android::init(app).expect("Failed to initialize Slint on Android");
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let db_path = get_db_path();
    let db = rt.block_on(async { Db::new(db_path.to_str().unwrap()) }).expect("Failed to initialize database");
    let db = Arc::new(db);

    // Importera från synkmapp om databasen är tom
    let db_clone = db.clone();
    rt.block_on(async {
        // Kolla om DB är tom
        if let Ok(jobs) = db_clone.get_filtered_jobs(&[], None, None).await {
            if jobs.is_empty() {
                tracing_info!("Start: Databasen är tom, försöker importera från synkmapp...");
                if let Ok(Some(settings)) = db_clone.load_settings().await {
                    let sync_path = settings.sync_path;
                    if !sync_path.is_empty() {
                        let sync_path = PathBuf::from(sync_path);
                        match import_from_sync_folder(&db_clone, &sync_path).await {
                            Ok(true) => {
                                tracing_info!("Start: Import slutförd!");
                            }
                            Ok(false) => {
                                tracing_info!("Start: Ingen data hittades att importera");
                            }
                            Err(e) => {
                                tracing_error!("Start: Import misslyckades: {}", e);
                            }
                        }
                    }
                }
            }
        }
    });

    let ui = App::new().expect("Failed to create Slint UI");
    #[cfg(target_os = "android")]
    {
        ui.set_has_mouse_wheel(false);
        ui.set_is_android(true);
    }
    #[cfg(not(target_os = "android"))]
    {
        ui.set_has_mouse_wheel(true);
        ui.set_is_android(false);
    }

    setup_ui(&ui, rt, db, log_rx);
    let _log_guard = guard;
    ui.run().expect("Failed to run Slint UI");
}
