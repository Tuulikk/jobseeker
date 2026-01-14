mod models;
mod api;
mod db;
mod ai;

use iced::widget::{column, row, text, button, scrollable, text_input, container, space, rule, svg, text_editor, stack, tooltip};
use iced::{Element, Task, Theme, Length, Color, Alignment, Padding};
use crate::models::{JobAd, AppSettings, AdStatus};
use crate::api::JobSearchClient;
use crate::db::Db;
use crate::ai::AiRanker;
use std::sync::Arc;
use chrono::{Utc, Datelike};
use tracing::{info, error};
use directories::ProjectDirs;

const SVG_UNREAD: &[u8] = include_bytes!("../assets/icons/circle-fill.svg");
const SVG_BOOKMARK: &[u8] = include_bytes!("../assets/icons/bookmark-star-fill.svg");
const SVG_THUMBS_UP: &[u8] = include_bytes!("../assets/icons/hand-thumbs-up-fill.svg");
const SVG_THUMBS_DOWN: &[u8] = include_bytes!("../assets/icons/hand-thumbs-down-fill.svg");
const SVG_APPLIED: &[u8] = include_bytes!("../assets/icons/check-circle-fill.svg");
const SVG_REJECTED: &[u8] = include_bytes!("../assets/icons/trash3-fill.svg");
const SVG_WEB: &[u8] = include_bytes!("../assets/icons/globe.svg");
const SVG_EMAIL: &[u8] = include_bytes!("../assets/icons/envelope-fill.svg");
const SVG_COPY: &[u8] = include_bytes!("../assets/icons/clipboard-plus-fill.svg");
const SVG_SETTINGS: &[u8] = include_bytes!("../assets/icons/gear-fill.svg");
const SVG_INBOX: &[u8] = include_bytes!("../assets/icons/inbox-fill.svg");
const SVG_PREV: &[u8] = include_bytes!("../assets/icons/chevron-left.svg");
const SVG_NEXT: &[u8] = include_bytes!("../assets/icons/chevron-right.svg");
const SVG_BOLD: &[u8] = include_bytes!("../assets/icons/type-bold.svg");
const SVG_ITALIC: &[u8] = include_bytes!("../assets/icons/type-italic.svg");
const SVG_UNDERLINE: &[u8] = include_bytes!("../assets/icons/type-underline.svg");
const SVG_ALIGN_LEFT: &[u8] = include_bytes!("../assets/icons/text-left.svg");
const SVG_ALIGN_CENTER: &[u8] = include_bytes!("../assets/icons/text-center.svg");
const SVG_ALIGN_RIGHT: &[u8] = include_bytes!("../assets/icons/text-right.svg");
const SVG_ALIGN_JUSTIFY: &[u8] = include_bytes!("../assets/icons/justify.svg");

#[derive(Debug, Clone)]
enum Tab {
    Inbox,
    Drafts,
    Settings,
    ApplicationEditor {
        job_id: String,
        job_headline: String,
        content: text_editor::Content,
        is_editing: bool,
    },
}

impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Tab::Inbox, Tab::Inbox) => true,
            (Tab::Drafts, Tab::Drafts) => true,
            (Tab::Settings, Tab::Settings) => true,
            (Tab::ApplicationEditor { job_id: id1, .. }, Tab::ApplicationEditor { job_id: id2, .. }) => id1 == id2,
            _ => false,
        }
    }
}

impl Eq for Tab {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum InboxFilter {
    #[default]
    All,
    Bookmarked,
    ThumbsUp,
    Applied,
}

#[derive(Debug, Clone)]
enum Message {
    Init,
    InitDb(Arc<Result<Db, String>>),
    LoadSettingsResult(AppSettings),
    SwitchTab(usize),
    CloseTab(usize),
    OpenEditor(String, String),
    LoadDrafts,
    DraftsResult(Result<Vec<(String, String, String)>, String>),
    DraftLoaded(String, String),
    ImportFile(usize),
    ExportPDF(usize),
    ExportWord(usize),
    ToggleEditMode(usize),
    ToggleEditorTools,
    EditorPasteProfile(usize),
    EditorAiImprove(usize),
    EventOccurred(iced::Event),
    EditorContentChanged(usize, text_editor::Action),
    SetFilter(InboxFilter),
    ChangeMonth(i32),
    Search(u32),
    SearchResult(Result<Vec<JobAd>, String>),
    SelectAd(usize),
    SettingsLocP1Changed(String),
    SettingsLocP2Changed(String),
    SettingsLocP3Changed(String),
    SettingsOllamaUrlChanged(String),
    EditorKeywordsChanged(text_editor::Action),
    EditorBlacklistChanged(text_editor::Action),
    EditorProfileChanged(text_editor::Action),
    SaveSettings,
    RateAd(usize),
    RateResult(usize, u8),
    UpdateStatus(usize, AdStatus),
    ClearAds,
    OpenBrowser(usize),
    SendEmail(usize),
    CopyAd(usize),
    CopyText(String),
    NextAd,
    PrevAd,
}

struct Jobseeker {
    active_tab: usize,
    tabs: Vec<Tab>,
    ads: Vec<JobAd>,
    selected_ad: Option<usize>,
    settings: AppSettings,
    db: Arc<Option<Db>>,
    filter: InboxFilter,
    is_searching: bool,
    error_msg: Option<String>,
    current_year: i32,
    current_month: u32,
    drafts_list: Vec<(String, String, String)>,
    show_editor_tools: bool,
    keywords_content: text_editor::Content,
    blacklist_content: text_editor::Content,
    profile_content: text_editor::Content,
}

impl Jobseeker {
    fn new() -> Self {
        let now = Utc::now();
        let settings = AppSettings::default();
        
        let keywords_content = text_editor::Content::with_text(&settings.keywords);
        let blacklist_content = text_editor::Content::with_text(&settings.blacklist_keywords);
        let profile_content = text_editor::Content::with_text(&settings.my_profile);

        Self {
            active_tab: 0,
            tabs: vec![Tab::Inbox, Tab::Drafts, Tab::Settings],
            ads: Vec::new(),
            selected_ad: None,
            settings,
            db: Arc::new(None),
            filter: InboxFilter::All,
            is_searching: false,
            error_msg: None,
            current_year: now.year(),
            current_month: now.month(),
            drafts_list: Vec::new(),
            show_editor_tools: true,
            keywords_content,
            blacklist_content,
            profile_content,
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl Default for Jobseeker {
    fn default() -> Self {
        Self::new()
    }
}

#[no_mangle]
pub extern "Rust" fn start_android(_app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(tracing::log::LevelFilter::Info),
    );
}