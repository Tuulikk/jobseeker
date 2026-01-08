mod ai;
mod api;
mod db;
mod models;
mod rich_editor;

use crate::rich_editor::{RichEditor, RichEditorMessage};

use crate::ai::AiRanker;
use crate::api::JobSearchClient;
use crate::db::Db;
use crate::models::{AdStatus, AppSettings, JobAd};
use chrono::{Datelike, Utc};
use iced::widget::{
    button, column, container, row, rule, scrollable, space, svg, text, text_editor, text_input,
};
use iced::{Alignment, Color, Element, Length, Padding, Task, Theme};
use std::sync::Arc;
use tracing::{error, info};

// SVG Icon bytes embedded in the binary
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
#[allow(dead_code)]
const SVG_BOLD: &[u8] = include_bytes!("../assets/icons/type-bold.svg");
#[allow(dead_code)]
const SVG_ITALIC: &[u8] = include_bytes!("../assets/icons/type-italic.svg");
#[allow(dead_code)]
const SVG_UNDERLINE: &[u8] = include_bytes!("../assets/icons/type-underline.svg");
#[allow(dead_code)]
const SVG_ALIGN_LEFT: &[u8] = include_bytes!("../assets/icons/text-left.svg");
#[allow(dead_code)]
const SVG_ALIGN_CENTER: &[u8] = include_bytes!("../assets/icons/text-center.svg");
#[allow(dead_code)]
const SVG_ALIGN_RIGHT: &[u8] = include_bytes!("../assets/icons/text-right.svg");
#[allow(dead_code)]
const SVG_ALIGN_JUSTIFY: &[u8] = include_bytes!("../assets/icons/justify.svg");
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    info!("Starting Jobseeker Gnag v0.2...");

    iced::application(
        || (Jobseeker::new(), Task::done(Message::Init)),
        Jobseeker::update,
        Jobseeker::view,
    )
    .title(get_title)
    .theme(Jobseeker::theme)
    .subscription(Jobseeker::subscription)
    .window(iced::window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        ..Default::default()
    })
    .run()
}

fn get_title(_: &Jobseeker) -> String {
    "Jobseeker Gnag v0.2 - NY".to_string()
}

#[derive(Debug, Clone)]
enum Tab {
    Inbox,
    Drafts,
    Settings,
    ApplicationEditor {
        job_id: String,
        job_headline: String,
        content: RichEditor,
        is_editing: bool,
    },
}
impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Tab::Inbox, Tab::Inbox) => true,
            (Tab::Settings, Tab::Settings) => true,
            (
                Tab::ApplicationEditor { job_id: id1, .. },
                Tab::ApplicationEditor { job_id: id2, .. },
            ) => id1 == id2,
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
struct UpdateInfo {
    version: String,
    html_url: String,
}

impl UpdateInfo {
    fn is_newer_than(&self, current: &str) -> bool {
        // Simple numeric comparison for dot-separated versions (e.g., 1.2.3)
        let parse = |s: &str| {
            s.split('.')
                .map(|p| p.parse::<u64>().unwrap_or(0))
                .collect::<Vec<_>>()
        };
        let cur = parse(current);
        let remote = parse(&self.version);
        let n = cur.len().max(remote.len());
        for i in 0..n {
            let c = *cur.get(i).unwrap_or(&0);
            let r = *remote.get(i).unwrap_or(&0);
            if r > c {
                return true;
            }
            if r < c {
                return false;
            }
        }
        false
    }
}

async fn check_github_updates() -> Result<UpdateInfo, String> {
    // Query GitHub Releases API for latest release
    let url = "https://api.github.com/repos/Gnaw-Software/Jobseeker/releases/latest";
    let client = reqwest::Client::builder()
        .user_agent("Jobseeker-update-check")
        .build()
        .map_err(|e| format!("Kunde inte skapa HTTP-klient: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Kunde inte kontakta GitHub API: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let j: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Kunde inte tolka JSON: {}", e))?;
    let tag = j["tag_name"].as_str().ok_or("Ingen tag_name i response")?;
    let version = tag.trim_start_matches('v').to_string();
    let html_url = j["html_url"]
        .as_str()
        .ok_or("Ingen html_url i response")?
        .to_string();

    Ok(UpdateInfo { version, html_url })
}

#[derive(Debug, Clone, PartialEq)]
enum UpdateCheckStatus {
    None,
    Checking,
    UpToDate,
    NewVersion { version: String, html_url: String },
    Error(String),
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
    drafts_list: Vec<(String, String, String)>, // id, headline, updated_at
    show_editor_tools: bool,
    show_markdown_preview: bool,

    // Editor states for settings
    keywords_content: text_editor::Content,
    blacklist_content: text_editor::Content,
    profile_content: text_editor::Content,

    // Draft renaming state
    renaming_draft_id: Option<String>,
    renaming_draft_new_name: String,

    // Update check state
    update_check_status: UpdateCheckStatus,
    is_checking_update: bool,

    // Misc
    #[allow(dead_code)]
    is_init: bool,
}

impl Jobseeker {
    fn new() -> Self {
        let now = Utc::now();
        let mut settings = AppSettings::load();

        // Beautify existing locations (convert codes to names)
        settings.locations_p1 = Self::beautify_locations(&settings.locations_p1);
        settings.locations_p2 = Self::beautify_locations(&settings.locations_p2);
        settings.locations_p3 = Self::beautify_locations(&settings.locations_p3);

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
            show_markdown_preview: false,
            keywords_content,
            blacklist_content,
            profile_content,
            renaming_draft_id: None,
            renaming_draft_new_name: String::new(),
            update_check_status: UpdateCheckStatus::None,
            is_checking_update: false,
            is_init: false,
        }
    }

    fn beautify_locations(raw: &str) -> String {
        raw.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.chars().all(|c| c.is_numeric()) {
                    JobSearchClient::get_municipality_name(s).unwrap_or_else(|| s.to_string())
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
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

#[derive(Debug, Clone)]
enum Message {
    Init,
    InitDb(Arc<Result<Db, String>>),
    SwitchTab(usize),
    CloseTab(usize),
    OpenEditor(String, String), // job_id, job_headline
    NewApplication,
    LoadDrafts,
    DraftsResult(Result<Vec<(String, String, String)>, String>), // id, headline, updated_at
    DraftLoaded(String, String),                                 // job_id, content
    ImportFile(usize),
    ExportPDF(usize),
    ExportWord(usize),
    ToggleEditMode(usize),
    ToggleEditorTools,
    ToggleMarkdownPreview(usize),
    EditorPasteProfile(usize),
    #[allow(dead_code)]
    EditorAiImprove(usize),
    EventOccurred(iced::Event),
    EditorContentChanged(usize),
    // Markdown formatting
    EditorBold(usize),
    EditorItalic(usize),
    EditorHeading1(usize),
    EditorHeading2(usize),
    EditorHeading3(usize),
    EditorBulletList(usize),
    EditorNumberedList(usize),
    EditorInsertCompany(usize),
    EditorInsertLink(usize),
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
    StartRenameDraft(String),      // draft_id
    UpdateRenameDraftName(String), // new_name
    SaveRenameDraft,
    CancelRenameDraft,
    CheckUpdates,
    CheckUpdatesResult(Result<UpdateInfo, String>),
    OpenReleaseUrl(String),
}

impl Jobseeker {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Init => {
                info!("Initializing DB...");
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(|| Db::new("jobseeker.db"))
                            .await
                            .unwrap()
                    },
                    |res| Message::InitDb(Arc::new(res.map_err(|e| e.to_string()))),
                )
            }
            Message::InitDb(res) => match &*res {
                Ok(db) => {
                    info!("DB initialized successfully.");
                    self.db = Arc::new(Some(db.clone()));
                    // Auto-search on startup if we have keywords configured
                    if !self.settings.keywords.trim().is_empty()
                        && !self.settings.locations_p1.trim().is_empty()
                    {
                        info!("Auto-searching priority 1 area on startup...");
                        Task::done(Message::Search(1))
                    } else {
                        self.refresh_list()
                    }
                }
                Err(err_str) => {
                    error!("DB Init Failed: {}", err_str);
                    self.error_msg = Some(format!("Database Error: {}", err_str));
                    Task::none()
                }
            },
            Message::SwitchTab(index) => {
                // Save settings when leaving Settings tab
                if matches!(self.tabs.get(self.active_tab), Some(Tab::Settings)) {
                    self.settings.save();
                }

                if index < self.tabs.len() {
                    self.active_tab = index;
                    // Ladda utkast om vi går till Drafts-fliken
                    if matches!(self.tabs[index], Tab::Drafts) {
                        return Task::done(Message::LoadDrafts);
                    }
                }
                Task::none()
            }
            Message::LoadDrafts => {
                let db_clone = Arc::clone(&self.db);
                Task::perform(
                    async move {
                        if let Some(db) = &*db_clone {
                            db.get_all_drafts().await
                        } else {
                            Ok(vec![])
                        }
                    },
                    |res| Message::DraftsResult(res.map_err(|e| e.to_string())),
                )
            }
            Message::DraftsResult(Ok(drafts)) => {
                self.drafts_list = drafts;
                Task::none()
            }
            Message::DraftsResult(Err(e)) => {
                self.error_msg = Some(format!("Kunde inte ladda utkast: {}", e));
                Task::none()
            }
            Message::CloseTab(index) => {
                if index < self.tabs.len() && self.tabs.len() > 1 {
                    // Stäng inte Inbox eller Settings om de är de sista
                    if !matches!(self.tabs[index], Tab::Inbox | Tab::Settings) {
                        self.tabs.remove(index);
                        if self.active_tab >= self.tabs.len() {
                            self.active_tab = self.tabs.len() - 1;
                        }
                    }
                }
                Task::none()
            }
            Message::OpenEditor(id, headline) => {
                // Kolla om den redan är öppen
                let existing = self.tabs.iter().position(|t| {
                    if let Tab::ApplicationEditor { job_id, .. } = t {
                        job_id == &id
                    } else {
                        false
                    }
                });

                if let Some(idx) = existing {
                    self.active_tab = idx;
                    Task::none()
                } else {
                    let db_clone = Arc::clone(&self.db);
                    let id_clone = id.clone();

                    // Skapa fliken direkt med tomt innehåll först
                    let new_tab = Tab::ApplicationEditor {
                        job_id: id.clone(),
                        job_headline: headline,
                        content: RichEditor::with_text(""),
                        is_editing: true,
                    };
                    self.tabs.push(new_tab);
                    self.active_tab = self.tabs.len() - 1;

                    // Ladda utkast från DB asynkront
                    let id_for_task = id_clone.clone();
                    Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                db.get_application_draft(&id_for_task).await.unwrap_or(None)
                            } else {
                                None
                            }
                        },
                        move |content| Message::DraftLoaded(id_clone, content.unwrap_or_default()),
                    )
                }
            }
            Message::DraftLoaded(id, content_str) => {
                for tab in self.tabs.iter_mut() {
                    if let Tab::ApplicationEditor {
                        job_id, content, ..
                    } = tab
                        && job_id == &id
                    {
                        content.set_text(&content_str);
                    }
                }
                Task::none()
            }
            Message::NewApplication => {
                // Skapa nytt utkast, öppna editor och spara i DB
                let id = format!("draft-{}", Utc::now().format("%Y%m%d%H%M%S"));
                let headline = format!("Nytt utkast {}", Utc::now().format("%Y-%m-%d"));
                let initial_content = crate::rich_editor::markdown::create_template(
                    "",
                    "",
                    &self.settings.my_profile,
                );

                let ad = JobAd {
                    id: id.clone(),
                    headline: headline.clone(),
                    description: None,
                    employer: None,
                    application_details: None,
                    webpage_url: None,
                    publication_date: Utc::now().to_rfc3339(),
                    last_application_date: None,
                    occupation: None,
                    workplace_address: None,
                    working_hours_type: None,
                    qualifications: None,
                    additional_information: None,
                    is_read: false,
                    rating: None,
                    bookmarked_at: None,
                    internal_created_at: Utc::now(),
                    search_keyword: None,
                    status: None,
                    applied_at: None,
                };

                // Lägg till flik i UI direkt
                let new_tab = Tab::ApplicationEditor {
                    job_id: id.clone(),
                    job_headline: headline,
                    content: RichEditor::with_text(&initial_content),
                    is_editing: true,
                };
                self.tabs.push(new_tab);
                self.active_tab = self.tabs.len() - 1;

                // Spara i DB asynkront och uppdatera utkast-listan när färdigt
                let db_clone = Arc::clone(&self.db);
                let id_clone = id.clone();
                let initial_clone = initial_content.clone();
                let ad_clone = ad.clone();
                if (*db_clone).is_some() {
                    Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.save_job_ad(&ad_clone).await;
                                let _ = db.save_application_draft(&id_clone, &initial_clone).await;
                            }
                        },
                        |_| Message::LoadDrafts,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ImportFile(index) => {
                if let Some(Tab::ApplicationEditor { job_id, .. }) = self.tabs.get(index) {
                    let id_clone = job_id.clone();
                    Task::perform(
                        async move {
                            if let Some(path) = rfd::AsyncFileDialog::new()
                                .add_filter("Text", &["txt", "md"])
                                .pick_file()
                                .await
                            {
                                tokio::fs::read_to_string(path.path()).await.ok()
                            } else {
                                None
                            }
                        },
                        move |res| Message::DraftLoaded(id_clone, res.unwrap_or_default()),
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExportPDF(index) => {
                if let Some(Tab::ApplicationEditor {
                    job_headline,
                    content,
                    ..
                }) = self.tabs.get(index)
                {
                    let markdown_text = content.text();
                    let headline = job_headline.clone();
                    Task::perform(
                        async move {
                            if let Some(path) = rfd::AsyncFileDialog::new()
                                .set_file_name(format!("Ansokan_{}.html", headline))
                                .save_file()
                                .await
                            {
                                // Konvertera Markdown till HTML
                                let html = crate::rich_editor::markdown::to_html(&markdown_text);
                                let _ = tokio::fs::write(path.path(), html).await;

                                // Info till användaren
                                println!("✓ HTML-fil skapad: {:?}", path.path());
                                println!(
                                    "  Öppna filen i webbläsare och tryck Ctrl+P för att spara som PDF"
                                );
                            }
                        },
                        |_| Message::SaveSettings,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ExportWord(index) => {
                if let Some(Tab::ApplicationEditor {
                    job_headline,
                    content,
                    ..
                }) = self.tabs.get(index)
                {
                    let markdown_text = content.text();
                    let headline = job_headline.clone();
                    Task::perform(
                        async move {
                            if let Some(path) = rfd::AsyncFileDialog::new()
                                .set_file_name(format!("Ansokan_{}.docx", headline))
                                .save_file()
                                .await
                            {
                                // Använd Markdown-till-DOCX konvertering
                                match crate::rich_editor::export::markdown_to_docx(
                                    &markdown_text,
                                    path.path(),
                                )
                                .await
                                {
                                    Ok(_) => println!("✓ Word-dokument skapat: {:?}", path.path()),
                                    Err(e) => eprintln!("✗ Fel vid Word-export: {}", e),
                                }
                            }
                        },
                        |_| Message::SaveSettings,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ToggleEditMode(index) => {
                if let Some(Tab::ApplicationEditor { is_editing, .. }) = self.tabs.get_mut(index) {
                    *is_editing = !*is_editing;
                }
                Task::none()
            }
            Message::ToggleEditorTools => {
                self.show_editor_tools = !self.show_editor_tools;
                Task::none()
            }
            Message::ToggleMarkdownPreview(index) => {
                if let Some(Tab::ApplicationEditor { .. }) = self.tabs.get(index) {
                    self.show_markdown_preview = !self.show_markdown_preview;
                }
                Task::none()
            }
            Message::EditorPasteProfile(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    let profile = self.settings.my_profile.clone();
                    content.update(RichEditorMessage::InsertText(profile));
                }
                Task::none()
            }

            Message::EditorInsertLink(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::Link);
                }
                Task::none()
            }
            Message::EditorAiImprove(index) => {
                if let Some(Tab::ApplicationEditor {
                    job_id, content, ..
                }) = self.tabs.get(index)
                {
                    let current_text = content.text();
                    let ad_ref = self.ads.iter().find(|a| &a.id == job_id);
                    let ad_desc = ad_ref
                        .and_then(|a| a.description.as_ref())
                        .and_then(|d| d.text.clone())
                        .unwrap_or_default();
                    let url = self.settings.ollama_url.clone();
                    let _job_id_clone = job_id.clone();

                    Task::perform(
                        async move {
                            let _ranker = AiRanker::new(&url, "not-needed").ok()?;
                            let prompt = format!(
                                "Här är en jobbannons: {}\n\nHär är mitt nuvarande utkast på ansökan: {}\n\nFörbättra texten så den blir mer professionell och matchar annonsen bättre. Svara bara med den förbättrade texten.",
                                ad_desc, current_text
                            );
                            // Vi återanvänder AiRanker för enkelhetens skull eller lägger till en metod för chat
                            Some(prompt) // Placeholder för faktiskt AI-anrop
                        },
                        move |_res| Message::SaveSettings,
                    ) // Placeholder
                } else {
                    Task::none()
                }
            }
            Message::EventOccurred(event) => {
                if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Middle,
                )) = event
                {
                    self.show_editor_tools = !self.show_editor_tools;
                }
                Task::none()
            }
            Message::EditorContentChanged(index) => {
                if let Some(Tab::ApplicationEditor {
                    job_id, content, ..
                }) = self.tabs.get(index)
                {
                    let db_clone = Arc::clone(&self.db);
                    let id_clone = job_id.clone();
                    let text_clone = content.text();
                    Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.save_application_draft(&id_clone, &text_clone).await;
                            }
                        },
                        |_| Message::SaveSettings,
                    )
                } else {
                    Task::none()
                }
            }
            Message::EditorBold(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::Bold);
                }
                Task::none()
            }
            Message::EditorItalic(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::Italic);
                }
                Task::none()
            }
            Message::EditorHeading1(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::Heading1);
                }
                Task::none()
            }
            Message::EditorHeading2(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::Heading2);
                }
                Task::none()
            }
            Message::EditorHeading3(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::Heading3);
                }
                Task::none()
            }
            Message::EditorBulletList(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::BulletList);
                }
                Task::none()
            }
            Message::EditorNumberedList(index) => {
                if let Some(Tab::ApplicationEditor { content, .. }) = self.tabs.get_mut(index) {
                    content.update(RichEditorMessage::NumberedList);
                }
                Task::none()
            }
            Message::EditorInsertCompany(index) => {
                if let Some(Tab::ApplicationEditor {
                    job_id, content, ..
                }) = self.tabs.get_mut(index)
                {
                    let company = self
                        .ads
                        .iter()
                        .find(|a| &a.id == job_id)
                        .and_then(|a| a.employer.as_ref().and_then(|e| e.name.clone()))
                        .unwrap_or_else(|| "[Företagsnamn]".to_string());
                    content.update(RichEditorMessage::InsertText(company));
                }
                Task::none()
            }
            Message::SetFilter(filter) => {
                self.filter = filter;
                self.refresh_list()
            }
            Message::ChangeMonth(delta) => {
                let mut m = self.current_month as i32 + delta;
                let mut y = self.current_year;
                if m < 1 {
                    m = 12;
                    y -= 1;
                } else if m > 12 {
                    m = 1;
                    y += 1;
                }
                self.current_month = m as u32;
                self.current_year = y;
                self.refresh_list()
            }
            Message::Search(priority) => {
                if self.is_searching {
                    return Task::none();
                }
                self.is_searching = true;
                self.error_msg = None;

                let keywords_raw = self.settings.keywords.clone();
                let blacklist_raw = self.settings.blacklist_keywords.clone();
                let locations = match priority {
                    1 => self.settings.locations_p1.clone(),
                    2 => self.settings.locations_p2.clone(),
                    _ => self.settings.locations_p3.clone(),
                };
                let db_clone = Arc::clone(&self.db);

                info!(
                    "Starting multi-search P{} for keywords: '{}'",
                    priority, keywords_raw
                );

                Task::perform(
                    async move {
                        let client = JobSearchClient::new();
                        let loc_vec: Vec<String> = locations
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .map(|s| {
                                // Om det är sifferkod, behåll den. Annars försök mappa namn.
                                if s.chars().all(|c| c.is_numeric()) {
                                    s
                                } else if let Some(code) =
                                    JobSearchClient::get_municipality_code(&s)
                                {
                                    code.to_string()
                                } else {
                                    s // Behåll originalet som fallback
                                }
                            })
                            .collect();
                        let keyword_vec: Vec<String> = keywords_raw
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        let blacklist_vec: Vec<String> = blacklist_raw
                            .split(',')
                            .map(|s| s.trim().to_lowercase())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let mut all_fetched_ads = Vec::new();

                        for kw in keyword_vec {
                            match client.search(&kw, &loc_vec, 50).await {
                                Ok(mut ads) => {
                                    for ad in &mut ads {
                                        ad.search_keyword = Some(kw.clone());
                                    }
                                    all_fetched_ads.extend(ads);
                                }
                                Err(e) => error!("Search failed for keyword '{}': {}", kw, e),
                            }
                        }

                        let filtered_ads: Vec<JobAd> = all_fetched_ads
                            .into_iter()
                            .filter(|ad| {
                                let headline = ad.headline.to_lowercase();
                                let desc = ad
                                    .description
                                    .as_ref()
                                    .and_then(|d| d.text.as_ref())
                                    .map(|s| s.to_lowercase())
                                    .unwrap_or_default();
                                !blacklist_vec.iter().any(|bad_word| {
                                    headline.contains(bad_word) || desc.contains(bad_word)
                                })
                            })
                            .collect();

                        if let Some(db) = &*db_clone {
                            for ad in &filtered_ads {
                                let _ = db.save_job_ad(ad).await;
                            }
                            db.get_filtered_jobs(&[], None, None).await
                        } else {
                            Ok(filtered_ads)
                        }
                    },
                    |res| Message::SearchResult(res.map_err(|e| e.to_string())),
                )
            }
            Message::SearchResult(Ok(ads)) => {
                self.is_searching = false;
                self.ads = ads;
                // Auto-select first ad to make the UI more inviting
                self.selected_ad = if !self.ads.is_empty() { Some(0) } else { None };

                // Mark first ad as read if it exists
                if let Some(ad) = self.ads.get_mut(0)
                    && !ad.is_read
                {
                    ad.is_read = true;
                    let id = ad.id.clone();
                    let db_clone = Arc::clone(&self.db);
                    return Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.mark_as_read(&id).await;
                            }
                        },
                        |_| Message::SaveSettings,
                    );
                }
                Task::none()
            }
            Message::SearchResult(Err(e)) => {
                self.is_searching = false;
                self.error_msg = Some(format!("Search failed: {}", e));
                Task::none()
            }
            Message::SelectAd(index) => {
                self.selected_ad = Some(index);
                if let Some(ad) = self.ads.get_mut(index)
                    && !ad.is_read
                {
                    ad.is_read = true;
                    let id = ad.id.clone();
                    let db_clone = Arc::clone(&self.db);
                    return Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.mark_as_read(&id).await;
                            }
                        },
                        |_| Message::SaveSettings,
                    );
                }
                Task::none()
            }
            Message::UpdateStatus(index, status) => {
                if let Some(ad) = self.ads.get_mut(index) {
                    ad.status = Some(status);
                    if status == AdStatus::Applied {
                        ad.applied_at = Some(Utc::now());
                    } else if status == AdStatus::Bookmarked || status == AdStatus::ThumbsUp {
                        ad.bookmarked_at = Some(Utc::now());
                    }
                    let id = ad.id.clone();
                    let db_clone = Arc::clone(&self.db);
                    // Update DB in background without refreshing the list
                    // (local changes already applied above)
                    Task::perform(
                        async move {
                            if let Some(db) = &*db_clone
                                && let Err(e) = db.update_ad_status(&id, status).await
                            {
                                eprintln!("Failed to update ad status in DB: {}", e);
                            }
                        },
                        |_| Message::SaveSettings, // No-op message
                    )
                } else {
                    Task::none()
                }
            }
            Message::SettingsLocP1Changed(val) => {
                self.settings.locations_p1 = val;
                Task::done(Message::SaveSettings)
            }
            Message::SettingsLocP2Changed(val) => {
                self.settings.locations_p2 = val;
                Task::done(Message::SaveSettings)
            }
            Message::SettingsLocP3Changed(val) => {
                self.settings.locations_p3 = val;
                Task::done(Message::SaveSettings)
            }
            Message::SettingsOllamaUrlChanged(val) => {
                self.settings.ollama_url = val;
                Task::done(Message::SaveSettings)
            }
            Message::EditorKeywordsChanged(action) => {
                self.keywords_content.perform(action);
                let text = self.keywords_content.text();
                info!("Keywords changed - new length: {}", text.len());
                self.settings.keywords = text;
                Task::none() // Don't auto-save on every keystroke
            }
            Message::EditorBlacklistChanged(action) => {
                self.blacklist_content.perform(action);
                let text = self.blacklist_content.text();
                info!("Blacklist changed - new length: {}", text.len());
                self.settings.blacklist_keywords = text;
                Task::none() // Don't auto-save on every keystroke
            }
            Message::EditorProfileChanged(action) => {
                self.profile_content.perform(action);
                self.settings.my_profile = self.profile_content.text();
                Task::none() // Don't auto-save on every keystroke
            }
            Message::SaveSettings => {
                info!(
                    "Saving settings - keywords length: {}, blacklist length: {}",
                    self.settings.keywords.len(),
                    self.settings.blacklist_keywords.len()
                );
                self.settings.save();
                Task::none()
            }
            Message::RateAd(index) => {
                if let Some(ad) = self.ads.get(index) {
                    let ad_clone = ad.clone();
                    let profile = self.settings.my_profile.clone();
                    let url = self.settings.ollama_url.clone();
                    Task::perform(
                        async move {
                            let ranker = AiRanker::new(&url, "not-needed").expect("Invalid AI URL");
                            ranker.rate_job(&ad_clone, &profile).await.unwrap_or(0)
                        },
                        move |res| Message::RateResult(index, res),
                    )
                } else {
                    Task::none()
                }
            }
            Message::RateResult(index, rating) => {
                if let Some(ad) = self.ads.get_mut(index) {
                    ad.rating = Some(rating);
                    let id = ad.id.clone();
                    let db_clone = Arc::clone(&self.db);
                    Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.update_rating(&id, rating).await;
                            }
                        },
                        |_| Message::SaveSettings,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ClearAds => {
                let db_clone = Arc::clone(&self.db);
                Task::perform(
                    async move {
                        if let Some(db) = &*db_clone {
                            let _ = db.clear_non_bookmarked().await;
                            db.get_filtered_jobs(
                                &[],
                                Some(Utc::now().year()),
                                Some(Utc::now().month()),
                            )
                            .await
                        } else {
                            Ok(vec![])
                        }
                    },
                    |res| Message::SearchResult(res.map_err(|e| e.to_string())),
                )
            }
            Message::OpenBrowser(index) => {
                if let Some(ad) = self.ads.get(index)
                    && let Some(url) = &ad.webpage_url
                {
                    let _ = webbrowser::open(url);
                }
                Task::none()
            }
            Message::SendEmail(index) => {
                if let Some(ad) = self.ads.get(index) {
                    let subject = format!("Jobbtips: {}", ad.headline);
                    let employer = ad
                        .employer
                        .as_ref()
                        .and_then(|e| e.name.as_ref())
                        .map(|s| s.as_str())
                        .unwrap_or("Okänd");
                    let body = format!(
                        "Kolla in detta jobb!\n\nRubrik: {}\nArbetsgivare: {}\nLänk: {}",
                        ad.headline,
                        employer,
                        ad.webpage_url.as_deref().unwrap_or("Ingen länk")
                    );
                    let mailto = format!(
                        "mailto:?subject={}&body={}",
                        urlencoding::encode(&subject),
                        urlencoding::encode(&body)
                    );
                    let _ = webbrowser::open(&mailto);
                }
                Task::none()
            }
            Message::CopyAd(index) => {
                if let Some(ad) = self.ads.get(index) {
                    let employer = ad
                        .employer
                        .as_ref()
                        .and_then(|e| e.name.as_ref())
                        .map(|s| s.as_str())
                        .unwrap_or("Okänd");
                    let desc = ad
                        .description
                        .as_ref()
                        .and_then(|d| d.text.as_ref())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    let content = format!(
                        "{}\nArbetsgivare: {}\n\n{}\n\nLänk: {}",
                        ad.headline,
                        employer,
                        desc,
                        ad.webpage_url.as_deref().unwrap_or("N/A")
                    );
                    return iced::clipboard::write(content);
                }
                Task::none()
            }
            Message::CopyText(val) => iced::clipboard::write(val),
            Message::NextAd => {
                if let Some(current) = self.selected_ad {
                    if current + 1 < self.ads.len() {
                        return Task::done(Message::SelectAd(current + 1));
                    }
                } else if !self.ads.is_empty() {
                    return Task::done(Message::SelectAd(0));
                }
                Task::none()
            }
            Message::PrevAd => {
                if let Some(current) = self.selected_ad
                    && current > 0
                {
                    return Task::done(Message::SelectAd(current - 1));
                }
                Task::none()
            }
            Message::StartRenameDraft(id) => {
                // Find the current headline for this draft
                let headline = self
                    .drafts_list
                    .iter()
                    .find(|(d_id, _, _)| d_id == &id)
                    .map(|(_, h, _)| h.clone())
                    .unwrap_or_default();

                self.renaming_draft_id = Some(id);
                self.renaming_draft_new_name = headline;
                Task::none()
            }
            Message::UpdateRenameDraftName(new_name) => {
                self.renaming_draft_new_name = new_name;
                Task::none()
            }
            Message::SaveRenameDraft => {
                if let Some(draft_id) = &self.renaming_draft_id
                    && !self.renaming_draft_new_name.trim().is_empty()
                {
                    // Save to DB
                    let db_clone = Arc::clone(&self.db);
                    let id_clone = draft_id.clone();
                    let new_headline = self.renaming_draft_new_name.clone();

                    Task::perform(
                        async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.update_draft_headline(&id_clone, &new_headline).await;
                            }
                        },
                        |_| {
                            // Clear rename state and reload drafts
                            let mut state = Self::new();
                            state.renaming_draft_id = None;
                            state.renaming_draft_new_name = String::new();
                            Message::LoadDrafts
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::CancelRenameDraft => {
                self.renaming_draft_id = None;
                self.renaming_draft_new_name = String::new();
                Task::none()
            }
            Message::CheckUpdates => {
                self.is_checking_update = true;
                self.update_check_status = UpdateCheckStatus::Checking;
                Task::perform(async move { check_github_updates().await }, |res| {
                    Message::CheckUpdatesResult(res.map_err(|e| e.to_string()))
                })
            }
            Message::CheckUpdatesResult(Ok(update_info)) => {
                self.is_checking_update = false;
                if update_info.is_newer_than(CURRENT_VERSION) {
                    self.update_check_status = UpdateCheckStatus::NewVersion {
                        version: update_info.version.clone(),
                        html_url: update_info.html_url.clone(),
                    };
                } else {
                    self.update_check_status = UpdateCheckStatus::UpToDate;
                }
                Task::none()
            }
            Message::CheckUpdatesResult(Err(e)) => {
                self.is_checking_update = false;
                self.update_check_status = UpdateCheckStatus::Error(e);
                Task::none()
            }
            Message::OpenReleaseUrl(url) => {
                if url.starts_with("http://") || url.starts_with("https://") {
                    let _ = webbrowser::open(&url);
                }
                Task::none()
            }
        }
    }

    fn refresh_list(&self) -> Task<Message> {
        let db_clone = Arc::clone(&self.db);
        let filter = self.filter;
        let year = self.current_year;
        let month = self.current_month;

        Task::perform(
            async move {
                if let Some(db) = &*db_clone {
                    match filter {
                        InboxFilter::All => {
                            db.get_filtered_jobs(&[], Some(year), Some(month)).await
                        }
                        InboxFilter::Bookmarked => {
                            db.get_filtered_jobs(
                                &[AdStatus::Bookmarked, AdStatus::ThumbsUp],
                                Some(year),
                                Some(month),
                            )
                            .await
                        }
                        InboxFilter::ThumbsUp => {
                            db.get_filtered_jobs(&[AdStatus::ThumbsUp], Some(year), Some(month))
                                .await
                        }
                        InboxFilter::Applied => {
                            db.get_filtered_jobs(&[AdStatus::Applied], Some(year), Some(month))
                                .await
                        }
                    }
                } else {
                    Ok(vec![])
                }
            },
            |res| Message::SearchResult(res.map_err(|e| e.to_string())),
        )
    }

    fn view(&self) -> Element<'_, Message> {
        let active_tab_content = &self.tabs[self.active_tab];

        let toolbar_content: Element<Message> = match active_tab_content {
            Tab::Inbox => {
                if self.is_searching {
                    row![text("Söker...").color(Color::from_rgb(0.0, 0.5, 1.0))].into()
                } else {
                    row![
                        text("Område:")
                            .size(16)
                            .color(Color::from_rgb(0.9, 0.9, 0.9)),
                        button("1").on_press(Message::Search(1)),
                        button("2").on_press(Message::Search(2)),
                        button("3").on_press(Message::Search(3)),
                        space::horizontal(),
                        button(svg(svg::Handle::from_memory(SVG_PREV)).width(20).height(20))
                            .on_press(Message::PrevAd),
                        button(svg(svg::Handle::from_memory(SVG_NEXT)).width(20).height(20))
                            .on_press(Message::NextAd),
                        space::horizontal(),
                        button(
                            row![
                                svg(svg::Handle::from_memory(SVG_REJECTED))
                                    .width(20)
                                    .height(20),
                                text(" Töm")
                            ]
                            .align_y(Alignment::Center)
                        )
                        .on_press(Message::ClearAds),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
                }
            }
            Tab::Drafts => row![
                text("Mina sparade utkast")
                    .size(16)
                    .color(Color::from_rgb(0.9, 0.9, 0.9)),
                space::horizontal(),
                button("Uppdatera lista").on_press(Message::LoadDrafts),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into(),
            Tab::ApplicationEditor { is_editing, .. } => row![
                button(if *is_editing {
                    "Klar (Läs-läge)"
                } else {
                    "Redigera"
                })
                .on_press(Message::ToggleEditMode(self.active_tab)),
                button(if self.show_editor_tools {
                    "Dölj verktyg"
                } else {
                    "Visa verktyg"
                })
                .on_press(Message::ToggleEditorTools),
                button(if self.show_markdown_preview {
                    "Dölj preview"
                } else {
                    "Visa preview"
                })
                .on_press(Message::ToggleMarkdownPreview(self.active_tab)),
                button("Öppna fil").on_press(Message::ImportFile(self.active_tab)),
                space::horizontal(),
                button("Exportera PDF")
                    .on_press(Message::ExportPDF(self.active_tab))
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(Color::from_rgb(0.1, 0.3, 0.1).into()),
                        ..Default::default()
                    }),
                button("Exportera Word")
                    .on_press(Message::ExportWord(self.active_tab))
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(Color::from_rgb(0.1, 0.1, 0.3).into()),
                        ..Default::default()
                    }),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into(),
            Tab::Settings => row![
                text("Applikationsinställningar")
                    .size(16)
                    .color(Color::from_rgb(0.9, 0.9, 0.9)),
            ]
            .into(),
        };

        let toolbar = container(toolbar_content)
            .width(Length::Fill)
            .padding(10)
            .style(|_theme: &Theme| container::Style {
                background: Some(Color::from_rgb(0.1, 0.1, 0.15).into()),
                ..Default::default()
            });

        let mut tab_row = row![].spacing(5).padding(Padding {
            top: 5.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        });

        for (i, tab) in self.tabs.iter().enumerate() {
            let (label, svg_icon) = match tab {
                Tab::Inbox => (" Inbox".to_string(), Some(SVG_INBOX)),
                Tab::Drafts => (" Utkast".to_string(), Some(SVG_COPY)), // Använd CLIPBOARD_PLUS för utkast
                Tab::Settings => (" Inställningar".to_string(), Some(SVG_SETTINGS)),
                Tab::ApplicationEditor { job_headline, .. } => {
                    let mut short = job_headline.clone();
                    if short.len() > 15 {
                        short.truncate(12);
                        short.push_str("...");
                    }
                    (short, None)
                }
            };

            let is_active = self.active_tab == i;

            let mut content = row![].align_y(Alignment::Center).spacing(5);
            if let Some(svg_data) = svg_icon {
                let icon = svg(svg::Handle::from_memory(svg_data))
                    .width(16)
                    .height(16)
                    .style(|_theme: &Theme, _status| svg::Style {
                        color: Some(Color::BLACK),
                    });
                content = content.push(icon);
            }
            content = content.push(text(label).size(14));

            let content = if !matches!(tab, Tab::Inbox | Tab::Drafts | Tab::Settings) {
                content.push(
                    button(text(" x").size(12))
                        .on_press(Message::CloseTab(i))
                        .style(|_theme: &Theme, _status| button::Style {
                            background: None,
                            text_color: Color::from_rgb(0.8, 0.2, 0.2),
                            ..Default::default()
                        }),
                )
            } else {
                content
            };

            let mut tab_btn = button(content)
                .on_press(Message::SwitchTab(i))
                .padding([5, 10]);

            if is_active {
                tab_btn = tab_btn.style(|_theme: &Theme, _status| button::Style {
                    background: Some(Color::from_rgb(0.4, 0.5, 0.6).into()),
                    text_color: Color::WHITE,
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.6, 0.8),
                        width: 2.5,
                        radius: iced::border::Radius {
                            top_left: 5.0,
                            top_right: 5.0,
                            bottom_right: 0.0,
                            bottom_left: 0.0,
                        },
                    },
                    ..Default::default()
                });
            } else {
                tab_btn = tab_btn.style(|_theme: &Theme, _status| button::Style {
                    background: Some(Color::from_rgb(0.3, 0.4, 0.5).into()),
                    text_color: Color::from_rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                });
            }

            tab_row = tab_row.push(tab_btn);
        }

        let tab_bar = container(tab_row)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Color::from_rgb(0.05, 0.05, 0.05).into()),
                ..Default::default()
            });

        let content: Element<Message> = match &self.tabs[self.active_tab] {
            Tab::Inbox => self.view_inbox(),
            Tab::Drafts => self.view_drafts(),
            Tab::Settings => self.view_settings(),
            Tab::ApplicationEditor {
                job_id: _,
                job_headline,
                content,
                is_editing,
            } => self.view_application_editor(self.active_tab, job_headline, content, *is_editing),
        };

        column![
            tab_bar,
            toolbar, // Vi kan ha toolbar kvar eller integrera den i flikarna
            rule::horizontal(1),
            container(content).width(Length::Fill).height(Length::Fill)
        ]
        .into()
    }

    fn view_inbox(&self) -> Element<'_, Message> {
        let filter_bar = row![
            button("Alla").on_press(Message::SetFilter(InboxFilter::All)),
            button(
                row![
                    svg(svg::Handle::from_memory(SVG_BOOKMARK))
                        .width(20)
                        .height(20),
                    text(" Bokm.")
                ]
                .align_y(Alignment::Center)
            )
            .on_press(Message::SetFilter(InboxFilter::Bookmarked)),
            button(
                row![
                    svg(svg::Handle::from_memory(SVG_THUMBS_UP))
                        .width(20)
                        .height(20),
                    text(" Toppen")
                ]
                .align_y(Alignment::Center)
            )
            .on_press(Message::SetFilter(InboxFilter::ThumbsUp)),
            button(
                row![
                    svg(svg::Handle::from_memory(SVG_APPLIED))
                        .width(20)
                        .height(20),
                    text(" Sökta")
                ]
                .align_y(Alignment::Center)
            )
            .on_press(Message::SetFilter(InboxFilter::Applied)),
        ]
        .spacing(5)
        .align_y(Alignment::Center);

        let month_navigator = row![
            button(svg(svg::Handle::from_memory(SVG_PREV)).width(24).height(24))
                .on_press(Message::ChangeMonth(-1)),
            text(format!(
                "{:04}-{:02}",
                self.current_year, self.current_month
            ))
            .size(16),
            button(svg(svg::Handle::from_memory(SVG_NEXT)).width(24).height(24))
                .on_press(Message::ChangeMonth(1)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mut sidebar_content = column![filter_bar, month_navigator]
            .spacing(10)
            .padding(Padding {
                top: 0.0,
                right: 30.0,
                bottom: 0.0,
                left: 15.0,
            })
            .width(Length::Fill);

        if let Some(err) = &self.error_msg {
            sidebar_content = sidebar_content
                .push(container(text(err).color(Color::from_rgb(1.0, 0.3, 0.3))).padding(10));
        }

        if self.ads.is_empty() && !self.is_searching {
            sidebar_content =
                sidebar_content.push(container(text("Här var det tomt.")).padding(20));
        } else {
            for (i, ad) in self.ads.iter().enumerate() {
                sidebar_content = sidebar_content.push(self.ad_row(i, ad));
            }
        }

        let sidebar = container(scrollable(sidebar_content))
            .width(Length::Fixed(400.0))
            .height(Length::Fill)
            .padding(5);

        let details = if let Some(index) = self.selected_ad {
            if let Some(ad) = self.ads.get(index) {
                let action_toolbar = row![
                    button(
                        row![
                            svg(svg::Handle::from_memory(SVG_THUMBS_DOWN))
                                .width(20)
                                .height(20),
                            text(" Nej")
                        ]
                        .align_y(Alignment::Center)
                    )
                    .on_press(Message::UpdateStatus(index, AdStatus::Rejected)),
                    button(
                        row![
                            svg(svg::Handle::from_memory(SVG_BOOKMARK))
                                .width(20)
                                .height(20),
                            text(" Spara")
                        ]
                        .align_y(Alignment::Center)
                    )
                    .on_press(Message::UpdateStatus(index, AdStatus::Bookmarked)),
                    button(
                        row![
                            svg(svg::Handle::from_memory(SVG_THUMBS_UP))
                                .width(20)
                                .height(20),
                            text(" Toppen")
                        ]
                        .align_y(Alignment::Center)
                    )
                    .on_press(Message::UpdateStatus(index, AdStatus::ThumbsUp)),
                    button(
                        row![
                            svg(svg::Handle::from_memory(SVG_APPLIED))
                                .width(20)
                                .height(20),
                            text(" HAR SÖKT")
                        ]
                        .align_y(Alignment::Center)
                    )
                    .on_press(Message::UpdateStatus(index, AdStatus::Applied)),
                    space::horizontal(),
                    button(svg(svg::Handle::from_memory(SVG_WEB)).width(20).height(20))
                        .on_press(Message::OpenBrowser(index)),
                    button(
                        svg(svg::Handle::from_memory(SVG_EMAIL))
                            .width(20)
                            .height(20)
                    )
                    .on_press(Message::SendEmail(index)),
                    button(svg(svg::Handle::from_memory(SVG_COPY)).width(20).height(20))
                        .on_press(Message::CopyAd(index)),
                ]
                .spacing(10);

                let timestamp_info: Element<Message> = if let Some(applied_at) = ad.applied_at {
                    container(
                        text(format!(
                            "SÖKT: {}",
                            applied_at
                                .with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M:%S")
                        ))
                        .color(Color::from_rgb(0.0, 1.0, 0.0))
                        .size(16),
                    )
                    .padding(5)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(Color::from_rgb(0.0, 0.2, 0.0).into()),
                        ..Default::default()
                    })
                    .into()
                } else if let Some(bookmarked_at) = ad.bookmarked_at {
                    text(format!(
                        "Sparad: {}",
                        bookmarked_at
                            .with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M:%S")
                    ))
                    .color(Color::from_rgb(0.3, 0.6, 0.8))
                    .size(14)
                    .into()
                } else {
                    space::vertical().into()
                };

                let report_buttons: Element<Message> = if ad.status == Some(AdStatus::Applied) {
                    let job_type = ad
                        .occupation
                        .as_ref()
                        .and_then(|o| o.label.clone())
                        .unwrap_or_else(|| ad.headline.clone());
                    let company = ad
                        .employer
                        .as_ref()
                        .and_then(|e| e.name.clone())
                        .unwrap_or_default();
                    let date = ad
                        .applied_at
                        .map(|dt| {
                            dt.with_timezone(&chrono::Local)
                                .format("%Y-%m-%d")
                                .to_string()
                        })
                        .unwrap_or_default();
                    let hours = ad
                        .working_hours_type
                        .as_ref()
                        .and_then(|w| w.label.clone())
                        .unwrap_or_else(|| "Heltid".into());
                    let muni = ad
                        .workplace_address
                        .as_ref()
                        .and_then(|w| w.municipality.clone())
                        .unwrap_or_default();

                    container(
                        row![
                            text("Rapport urklipp:")
                                .size(14)
                                .color(Color::from_rgb(0.7, 0.7, 0.7)),
                            button(text("Typ")).on_press(Message::CopyText(job_type)),
                            button(text("Företag")).on_press(Message::CopyText(company)),
                            button(text("Datum")).on_press(Message::CopyText(date)),
                            button(text("Omf.")).on_press(Message::CopyText(hours)),
                            button(text("Kommun")).on_press(Message::CopyText(muni)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    )
                    .padding(10)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(Color::from_rgb(0.1, 0.15, 0.2).into()),
                        ..Default::default()
                    })
                    .into()
                } else {
                    space::vertical().into()
                };

                container(scrollable(
                    column![
                        action_toolbar,
                        report_buttons,
                        text(&ad.headline)
                            .size(30)
                            .width(Length::Fill)
                            .color(Color::WHITE),
                        row![
                            text(
                                ad.employer
                                    .as_ref()
                                    .and_then(|e| e.name.clone())
                                    .unwrap_or_else(|| "Okänd arbetsgivare".into())
                            )
                            .size(20),
                            text(format!(
                                "Publicerad: {}",
                                ad.publication_date
                                    .split('T')
                                    .next()
                                    .unwrap_or(&ad.publication_date)
                            ))
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                        ]
                        .spacing(20),
                        timestamp_info,
                        row![
                            button("Betygsätt med AI").on_press(Message::RateAd(index)),
                            button("Skriv ansökan")
                                .on_press(Message::OpenEditor(ad.id.clone(), ad.headline.clone())),
                        ]
                        .spacing(10),
                        text(
                            ad.description
                                .as_ref()
                                .and_then(|d| d.text.clone())
                                .unwrap_or_else(|| "Ingen beskrivning tillgänglig".into())
                        )
                    ]
                    .spacing(15)
                    .padding(Padding {
                        top: 10.0,
                        right: 30.0,
                        bottom: 10.0,
                        left: 10.0,
                    }),
                ))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
            } else {
                container(text("Annonsen hittades inte"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(10)
            }
        } else {
            container(text("Välj en annons i listan för att se detaljer."))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        };

        row![sidebar, details].into()
    }

    fn view_drafts(&self) -> Element<'_, Message> {
        let mut list_content = column![
            row![
                text("Mina ansökningsutkast").size(30).color(Color::WHITE),
                space::horizontal(),
                button("Ny ansökan").on_press(Message::NewApplication),
            ],
            space::vertical(),
        ]
        .spacing(10);

        if self.drafts_list.is_empty() {
            list_content = list_content
                .push(text("Inga utkast sparade ännu.").color(Color::from_rgb(0.5, 0.5, 0.5)));
        } else {
            for (id, headline, updated_at) in &self.drafts_list {
                let date_part = updated_at.split('T').next().unwrap_or(updated_at);
                let is_renaming = self.renaming_draft_id.as_ref() == Some(id);

                if is_renaming {
                    // Show rename input
                    list_content = list_content.push(
                        container(
                            row![
                                column![
                                    text_input("Nytt namn...", &self.renaming_draft_new_name)
                                        .on_input(Message::UpdateRenameDraftName)
                                        .padding(10)
                                        .size(18),
                                    text("Klicka på ✓ för att spara, ✗ för att avbryta")
                                        .size(12)
                                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                                ]
                                .spacing(5)
                                .width(Length::Fill),
                                row![
                                    button(text("✓").size(20))
                                        .on_press(Message::SaveRenameDraft)
                                        .padding(8),
                                    button(text("✗").size(20))
                                        .on_press(Message::CancelRenameDraft)
                                        .padding(8),
                                ]
                                .spacing(5),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .padding(10),
                        )
                        .width(Length::Fill)
                        .style(|_theme: &Theme| container::Style {
                            background: Some(Color::from_rgb(0.25, 0.25, 0.35).into()),
                            border: iced::Border {
                                color: Color::from_rgb(0.3, 0.6, 0.8),
                                width: 2.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                    );
                } else {
                    // Show normal draft row
                    list_content = list_content.push(
                        button(
                            row![
                                column![
                                    text(headline).size(18).color(Color::WHITE),
                                    text(format!("Senast sparad: {}", date_part))
                                        .size(14)
                                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                                ]
                                .spacing(5)
                                .width(Length::Fill),
                                row![
                                    button(text("✎").size(14))
                                        .on_press(Message::StartRenameDraft(id.clone()))
                                        .padding(5)
                                        .style(|_theme: &Theme, status| {
                                            if status == button::Status::Hovered {
                                                button::Style {
                                                    background: Some(
                                                        Color::from_rgb(0.4, 0.6, 0.8).into(),
                                                    ),
                                                    ..Default::default()
                                                }
                                            } else {
                                                button::Style {
                                                    background: None,
                                                    ..Default::default()
                                                }
                                            }
                                        }),
                                    text("Öppna →")
                                        .size(14)
                                        .color(Color::from_rgb(0.3, 0.6, 0.8)),
                                ]
                                .spacing(5)
                                .align_y(Alignment::Center),
                            ]
                            .align_y(Alignment::Center)
                            .padding(10),
                        )
                        .on_press(Message::OpenEditor(id.clone(), headline.clone()))
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            if status == button::Status::Hovered {
                                button::Style {
                                    background: Some(Color::from_rgb(0.15, 0.15, 0.2).into()),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Color::from_rgb(0.1, 0.1, 0.12).into()),
                                    ..Default::default()
                                }
                            }
                        }),
                    );
                }
            }
        }

        container(scrollable(list_content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(40)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(Message::EventOccurred)
    }

    fn ad_row<'a>(&self, i: usize, ad: &'a JobAd) -> Element<'a, Message> {
        let is_selected = self.selected_ad == Some(i);

        let (status_svg, icon_color) = match ad.status {
            Some(AdStatus::Rejected) => (SVG_REJECTED, Color::from_rgb(0.8, 0.3, 0.3)),
            Some(AdStatus::Bookmarked) => (SVG_BOOKMARK, Color::from_rgb(0.3, 0.6, 0.8)),
            Some(AdStatus::ThumbsUp) => (SVG_THUMBS_UP, Color::from_rgb(0.3, 0.8, 0.3)),
            Some(AdStatus::Applied) => (SVG_APPLIED, Color::from_rgb(0.5, 0.5, 0.5)),
            _ => {
                if !ad.is_read {
                    (SVG_UNREAD, Color::from_rgb(0.0, 0.5, 1.0))
                } else {
                    (SVG_UNREAD, Color::from_rgb(0.2, 0.2, 0.3))
                }
            }
        };

        let rating_text = match ad.rating {
            Some(r) => format!("[{}★]", r),
            None => "[---]".to_string(),
        };

        let date_str = ad
            .publication_date
            .split('T')
            .next()
            .unwrap_or(&ad.publication_date);
        let short_date = if date_str.len() > 5 {
            &date_str[5..]
        } else {
            date_str
        };
        let keyword_text = ad.search_keyword.as_deref().unwrap_or("---");
        let municipality_text = ad
            .workplace_address
            .as_ref()
            .and_then(|a| a.municipality.as_deref())
            .unwrap_or("Okänd");

        let title_color = if !ad.is_read {
            Color::WHITE
        } else {
            Color::from_rgb(0.6, 0.6, 0.7)
        };
        let bg_color = if is_selected {
            Color::from_rgb(0.2, 0.2, 0.3)
        } else if ad.is_read {
            Color::from_rgb(0.08, 0.08, 0.12)
        } else {
            Color::TRANSPARENT
        };

        container(
            button(
                row![
                    svg(svg::Handle::from_memory(status_svg))
                        .width(20)
                        .height(20)
                        .style(move |_, _| svg::Style {
                            color: Some(icon_color)
                        }),
                    column![
                        text(&ad.headline)
                            .size(18)
                            .width(Length::Fill)
                            .color(title_color),
                        row![
                            text(rating_text)
                                .size(14)
                                .color(Color::from_rgb(1.0, 1.0, 0.0)),
                            text(
                                ad.employer
                                    .as_ref()
                                    .and_then(|e| e.name.clone())
                                    .unwrap_or_default()
                            )
                            .size(14)
                            .color(Color::from_rgb(0.8, 0.8, 0.8))
                            .width(Length::Fill),
                            text(short_date)
                                .size(14)
                                .color(Color::from_rgb(0.7, 0.7, 0.7)),
                        ]
                        .spacing(5),
                        text(format!("{} • {}", keyword_text, municipality_text))
                            .size(14)
                            .color(Color::from_rgb(0.0, 0.8, 0.8))
                    ]
                    .spacing(2)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .on_press(Message::SelectAd(i))
            .width(Length::Fill)
            .padding(8)
            .style(move |_theme, status| {
                if status == button::Status::Hovered {
                    button::Style {
                        background: Some(Color::from_rgb(0.15, 0.15, 0.2).into()),
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: None,
                        ..Default::default()
                    }
                }
            }),
        )
        .style(move |_theme| container::Style {
            background: Some(bg_color.into()),
            ..Default::default()
        })
        .padding(Padding {
            top: 0.0,
            right: 5.0,
            bottom: 0.0,
            left: 0.0,
        })
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let update_status_el: Element<'_, Message> = match &self.update_check_status {
            UpdateCheckStatus::None => container(text("")).into(),
            UpdateCheckStatus::Checking => container(
                text("Hämtar versionsinformation...")
                    .size(14)
                    .color(Color::from_rgb(0.8, 0.8, 0.8)),
            )
            .into(),
            UpdateCheckStatus::UpToDate => container(
                text("Du har senaste version!")
                    .size(14)
                    .color(Color::from_rgb(0.0, 0.8, 0.0)),
            )
            .into(),
            UpdateCheckStatus::NewVersion { version, html_url } => {
                let url = html_url.clone();
                container(
                    column![
                        text(format!("Ny version tillgänglig: v{}", version))
                            .size(14)
                            .color(Color::from_rgb(0.0, 0.8, 1.0)),
                        button("Öppna release-sida").on_press(Message::OpenReleaseUrl(url))
                    ]
                    .spacing(5),
                )
                .into()
            }
            UpdateCheckStatus::Error(err) => container(
                text(format!("Kunde inte kolla uppdateringar: {}", err))
                    .size(14)
                    .color(Color::from_rgb(1.0, 0.3, 0.3)),
            )
            .into(),
        };
        container(scrollable(
            column![
                text("Inställningar").size(30).color(Color::WHITE),
                column![
                    text("Sökord")
                        .size(18)
                        .color(Color::from_rgb(0.0, 0.8, 0.8)),
                    text("Ange sökord separerade med kommatecken (t.ex. rust, python, support)")
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    container(
                        text_editor(&self.keywords_content)
                            .on_action(Message::EditorKeywordsChanged)
                    )
                    .height(150)
                    .padding(5)
                    .style(|_theme: &Theme| container::Style {
                        border: iced::Border {
                            color: Color::from_rgb(0.3, 0.3, 0.3),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(10),
                column![
                    text("Svartlista")
                        .size(18)
                        .color(Color::from_rgb(0.8, 0.3, 0.3)),
                    text("Annonser med dessa ord i rubrik eller beskrivning döljs")
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    container(
                        text_editor(&self.blacklist_content)
                            .on_action(Message::EditorBlacklistChanged)
                    )
                    .height(150)
                    .padding(5)
                    .style(|_theme: &Theme| container::Style {
                        border: iced::Border {
                            color: Color::from_rgb(0.3, 0.3, 0.3),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(10),
                column![
                    text("Geografiska områden")
                        .size(18)
                        .color(Color::from_rgb(0.3, 0.6, 0.8)),
                    text("Du kan nu skriva kommunnamn (t.ex. Helsingborg, Malmö) eller koder")
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    column![
                        text("Område 1 (Högsta prioritet)").size(14),
                        text_input("Kommuner eller koder", &self.settings.locations_p1)
                            .on_input(Message::SettingsLocP1Changed)
                            .padding(10),
                    ]
                    .spacing(5),
                    column![
                        text("Område 2").size(14),
                        text_input("Kommuner eller koder", &self.settings.locations_p2)
                            .on_input(Message::SettingsLocP2Changed)
                            .padding(10),
                    ]
                    .spacing(5),
                    column![
                        text("Område 3").size(14),
                        text_input("Kommuner eller koder", &self.settings.locations_p3)
                            .on_input(Message::SettingsLocP3Changed)
                            .padding(10),
                    ]
                    .spacing(5),
                ]
                .spacing(15),
                column![
                    text("AI & Profil")
                        .size(18)
                        .color(Color::from_rgb(1.0, 1.0, 0.0)),
                    column![
                        text("Min bakgrund (används för AI-matchning)").size(14),
                        container(
                            text_editor(&self.profile_content)
                                .on_action(Message::EditorProfileChanged)
                        )
                        .height(120)
                        .padding(5)
                        .style(|_theme: &Theme| container::Style {
                            border: iced::Border {
                                color: Color::from_rgb(0.3, 0.3, 0.3),
                                width: 1.0,
                                radius: 5.0.into(),
                            },
                            ..Default::default()
                        }),
                    ]
                    .spacing(5),
                    column![
                        text("Ollama API URL").size(14),
                        text_input("http://localhost:11434/v1", &self.settings.ollama_url)
                            .on_input(Message::SettingsOllamaUrlChanged)
                            .padding(10),
                    ]
                    .spacing(5),
                ]
                .spacing(15),
                // Update check & Save
                column![
                    row![
                        text(format!("Nuvarande version: v{}", CURRENT_VERSION))
                            .size(14)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                        space::horizontal(),
                        if self.is_checking_update {
                            button(text("Kontrollerar..."))
                        } else {
                            button(text("Kontrollera uppdateringar"))
                                .on_press(Message::CheckUpdates)
                        }
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    // Status (computed earlier)
                    container(update_status_el).padding(5),
                    // Save action (kept simple and consistent)
                    row![
                        button("💾 Spara inställningar")
                            .on_press(Message::SaveSettings)
                            .style(|_theme: &Theme, status| button::Style {
                                background: Some(
                                    if status == iced::widget::button::Status::Hovered {
                                        Color::from_rgb(0.3, 0.7, 0.3).into()
                                    } else {
                                        Color::from_rgb(0.2, 0.6, 0.2).into()
                                    }
                                ),
                                text_color: Color::WHITE,
                                border: iced::Border {
                                    radius: 5.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                    ]
                    .spacing(10)
                ]
                .spacing(30)
                .padding(Padding {
                    top: 20.0,
                    right: 40.0,
                    bottom: 20.0,
                    left: 20.0,
                }),
            ]
            .spacing(30)
            .padding(Padding {
                top: 20.0,
                right: 40.0,
                bottom: 20.0,
                left: 20.0,
            }),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_application_editor<'a>(
        &'a self,
        tab_index: usize,
        _headline: &str,
        content: &'a RichEditor,
        is_editing: bool,
    ) -> Element<'a, Message> {
        let editor_side: Element<'a, Message> = if is_editing {
            let editor_widget =
                content
                    .view(self.show_editor_tools)
                    .map(move |re_msg| match re_msg {
                        RichEditorMessage::ActionPerformed(_) => {
                            Message::EditorContentChanged(tab_index)
                        }
                        RichEditorMessage::Bold => Message::EditorBold(tab_index),
                        RichEditorMessage::Italic => Message::EditorItalic(tab_index),
                        RichEditorMessage::Heading1 => Message::EditorHeading1(tab_index),
                        RichEditorMessage::Heading2 => Message::EditorHeading2(tab_index),
                        RichEditorMessage::Heading3 => Message::EditorHeading3(tab_index),
                        RichEditorMessage::BulletList => Message::EditorBulletList(tab_index),
                        RichEditorMessage::NumberedList => Message::EditorNumberedList(tab_index),
                        RichEditorMessage::Link => Message::EditorInsertLink(tab_index),
                        RichEditorMessage::InsertText(_) => Message::SaveSettings,
                    });

            let edit_field = container(editor_widget)
                .padding(Padding {
                    top: 40.0,
                    right: 60.0,
                    bottom: 40.0,
                    left: 60.0,
                })
                .width(Length::Fixed(800.0))
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::WHITE.into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.7, 0.7, 0.7),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                        offset: iced::Vector::new(2.0, 2.0),
                        blur_radius: 10.0,
                    },
                    ..Default::default()
                });

            // Snabbverktyg för att klistra in profil / infoga företag
            let extra_tools = container(
                row![
                    button(text("Klistra in profil").size(12))
                        .on_press(Message::EditorPasteProfile(tab_index))
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(Color::from_rgb(0.2, 0.3, 0.5).into()),
                            ..Default::default()
                        }),
                    button(text("Infoga företag").size(11).color(Color::WHITE))
                        .on_press(Message::EditorInsertCompany(tab_index))
                        .padding(5)
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(Color::from_rgb(0.2, 0.3, 0.5).into()),
                            ..Default::default()
                        }),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding(10);

            column![edit_field, extra_tools].spacing(10).into()
        } else {
            let body = content.text();
            let display_text = if body.is_empty() {
                "Inget skrivet ännu. Tryck på Redigera för att börja.".to_string()
            } else {
                body
            };

            container(scrollable(text(display_text).color(Color::BLACK).size(16)))
                .padding(Padding {
                    top: 60.0,
                    right: 80.0,
                    bottom: 60.0,
                    left: 80.0,
                })
                .width(Length::Fixed(800.0))
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::WHITE.into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.8, 0.8, 0.8),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        };

        let job_id = if let Tab::ApplicationEditor { job_id, .. } = &self.tabs[tab_index] {
            job_id
        } else {
            ""
        };

        let ad_ref = self.ads.iter().find(|a| a.id == job_id);

        // Preview side (if enabled)
        let preview_side: Option<Element<'a, Message>> = if self.show_markdown_preview {
            let markdown_text = content.text();

            // Use styled Markdown rendering for better preview
            let markdown_preview = if markdown_text.is_empty() {
                text("Skriv något i editorn för att se förhandsvisning här...")
                    .size(14)
                    .into()
            } else {
                crate::rich_editor::markdown::to_iced(&markdown_text)
            };

            Some(
                container(
                    column![
                        container(text("📄 Förhandsvisning").size(18).color(Color::WHITE))
                            .padding(10)
                            .width(Length::Fill)
                            .style(|_theme: &Theme| container::Style {
                                background: Some(Color::from_rgb(0.3, 0.6, 0.8).into()),
                                ..Default::default()
                            }),
                        scrollable(container(markdown_preview).padding(20).width(Length::Fill))
                    ]
                    .spacing(0),
                )
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::WHITE.into()),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.6, 0.8),
                        width: 2.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
            )
        } else {
            None
        };

        let ad_side = if let Some(ad) = ad_ref {
            container(scrollable(
                column![
                    text(&ad.headline).size(24).color(Color::WHITE),
                    text(
                        ad.employer
                            .as_ref()
                            .and_then(|e| e.name.clone())
                            .unwrap_or_default()
                    )
                    .size(18),
                    rule::horizontal(1),
                    text(
                        ad.description
                            .as_ref()
                            .and_then(|d| d.text.clone())
                            .unwrap_or_default()
                    )
                ]
                .spacing(15),
            ))
            .padding(20)
            .width(Length::FillPortion(1))
        } else {
            container(text("Annonstext finns tillgänglig i Inbox-fliken."))
                .padding(20)
                .width(Length::FillPortion(1))
        };

        let mut main_row = row![
            ad_side,
            container(editor_side)
                .width(Length::FillPortion(2))
                .height(Length::Fill)
                .center_x(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::from_rgb(0.85, 0.85, 0.85).into()),
                    ..Default::default()
                })
        ];

        if let Some(preview) = preview_side {
            main_row = main_row.push(preview);
        }

        main_row.into()
    }
}
