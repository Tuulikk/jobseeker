mod models;
mod api;
mod db;
mod ai;

use iced::{Element, Task, Theme, Length, Color, Alignment};
use iced::widget::{column, row, text, button, scrollable, text_input, container, space, rule};
use crate::models::{JobAd, AppSettings, AdStatus};
use crate::api::JobSearchClient;
use crate::db::Db;
use crate::ai::AiRanker;
use std::sync::Arc;
use chrono::{Utc, Datelike};
use tracing::{info, error};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    info!("Starting Jobseeker Gnag v0.2...");

    iced::application(|| (Jobseeker::new(), Task::done(Message::Init)), Jobseeker::update, Jobseeker::view)
        .title(get_title)
        .theme(Jobseeker::theme)
        .run()
}

fn get_title(_: &Jobseeker) -> String {
    "Jobseeker Gnag v0.2 - NY".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    Inbox,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum InboxFilter {
    #[default]
    All,
    Bookmarked,
    ThumbsUp,
    Applied,
}

struct Jobseeker {
    page: Page,
    ads: Vec<JobAd>,
    selected_ad: Option<usize>,
    settings: AppSettings,
    db: Arc<Option<Db>>,
    filter: InboxFilter,
    is_searching: bool,
    error_msg: Option<String>,
    current_year: i32,
    current_month: u32,
}

impl Jobseeker {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            page: Page::Inbox,
            ads: Vec::new(),
            selected_ad: None,
            settings: AppSettings::load(),
            db: Arc::new(None),
            filter: InboxFilter::All,
            is_searching: false,
            error_msg: None,
            current_year: now.year(),
            current_month: now.month(),
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

#[derive(Debug, Clone)]
enum Message {
    Init,
    InitDb(Arc<Result<Db, String>>),
    GoToPage(Page),
    SetFilter(InboxFilter),
    ChangeMonth(i8),
    Search(u8),
    SearchResult(Result<Vec<JobAd>, String>),
    SelectAd(usize),
    SettingsKeywordsChanged(String),
    SettingsBlacklistChanged(String),
    SettingsLocP1Changed(String),
    SettingsLocP2Changed(String),
    SettingsLocP3Changed(String),
    SettingsProfileChanged(String),
    SettingsOllamaUrlChanged(String),
    SaveSettings,
    RateAd(usize),
    RateResult(usize, u8),
    UpdateStatus(usize, AdStatus),
    ClearAds,
    OpenBrowser(usize),
    SendEmail(usize),
    CopyAd(usize),
}

impl Jobseeker {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Init => {
                info!("Initializing DB...");
                Task::perform(async {
                    Db::new("jobseeker.db").await
                }, |res| Message::InitDb(Arc::new(res.map_err(|e| e.to_string()))))
            }
            Message::InitDb(res) => {
                match &*res {
                    Ok(db) => {
                        info!("DB initialized successfully.");
                        self.db = Arc::new(Some(db.clone()));
                        return self.refresh_list();
                    }
                    Err(err_str) => {
                        error!("DB Init Failed: {}", err_str);
                        self.error_msg = Some(format!("Database Error: {}", err_str));
                        Task::none()
                    }
                }
            }
            Message::GoToPage(page) => {
                self.page = page;
                Task::none()
            }
            Message::SetFilter(filter) => {
                self.filter = filter;
                self.refresh_list()
            }
            Message::ChangeMonth(delta) => {
                let mut m = self.current_month as i32 + delta as i32;
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
                
                info!("Starting multi-search P{} for keywords: '{}'", priority, keywords_raw);
                
                Task::perform(async move {
                    let client = JobSearchClient::new();
                    let loc_vec: Vec<String> = locations.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    let keyword_vec: Vec<String> = keywords_raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    let blacklist_vec: Vec<String> = blacklist_raw.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
                    
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
                    
                    let filtered_ads: Vec<JobAd> = all_fetched_ads.into_iter().filter(|ad| {
                        let headline = ad.headline.to_lowercase();
                        let desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.to_lowercase()).unwrap_or_default();
                        !blacklist_vec.iter().any(|bad_word| headline.contains(bad_word) || desc.contains(bad_word))
                    }).collect();
                    
                    if let Some(db) = &*db_clone {
                        for ad in &filtered_ads {
                            let _ = db.save_job_ad(ad).await;
                        }
                        db.get_filtered_jobs(&[], Utc::now().year(), Utc::now().month()).await
                    } else {
                        Ok(filtered_ads)
                    }
                }, |res| Message::SearchResult(res.map_err(|e| e.to_string())))
            }
            Message::SearchResult(Ok(ads)) => {
                self.is_searching = false;
                self.ads = ads;
                self.selected_ad = None;
                Task::none()
            }
            Message::SearchResult(Err(e)) => {
                self.is_searching = false;
                self.error_msg = Some(format!("Search failed: {}", e));
                Task::none()
            }
            Message::SelectAd(index) => {
                self.selected_ad = Some(index);
                if let Some(ad) = self.ads.get_mut(index) {
                    if !ad.is_read {
                        ad.is_read = true;
                        let id = ad.id.clone();
                        let db_clone = Arc::clone(&self.db);
                        return Task::perform(async move {
                            if let Some(db) = &*db_clone {
                                let _ = db.mark_as_read(&id).await;
                            }
                        }, |_| Message::SaveSettings);
                    }
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
                    let current_filter = self.filter;
                    return Task::perform(async move {
                        if let Some(db) = &*db_clone {
                            let _ = db.update_ad_status(&id, status).await;
                        }
                    }, move |_| Message::SetFilter(current_filter));
                }
                Task::none()
            }
            Message::SettingsKeywordsChanged(val) => {
                self.settings.keywords = val;
                Task::done(Message::SaveSettings)
            }
            Message::SettingsBlacklistChanged(val) => {
                self.settings.blacklist_keywords = val;
                Task::done(Message::SaveSettings)
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
            Message::SettingsProfileChanged(val) => {
                self.settings.my_profile = val;
                Task::done(Message::SaveSettings)
            }
            Message::SettingsOllamaUrlChanged(val) => {
                self.settings.ollama_url = val;
                Task::done(Message::SaveSettings)
            }
            Message::SaveSettings => {
                self.settings.save();
                Task::none()
            }
            Message::RateAd(index) => {
                if let Some(ad) = self.ads.get(index) {
                    let ad_clone = ad.clone();
                    let profile = self.settings.my_profile.clone();
                    let url = self.settings.ollama_url.clone();
                    Task::perform(async move {
                        let ranker = AiRanker::new(&url, "not-needed").expect("Invalid AI URL");
                        ranker.rate_job(&ad_clone, &profile).await.unwrap_or(0)
                    }, move |res| Message::RateResult(index, res))
                } else {
                    Task::none()
                }
            }
            Message::RateResult(index, rating) => {
                if let Some(ad) = self.ads.get_mut(index) {
                    ad.rating = Some(rating);
                    let id = ad.id.clone();
                    let db_clone = Arc::clone(&self.db);
                    return Task::perform(async move {
                        if let Some(db) = &*db_clone {
                            let _ = db.update_rating(&id, rating).await;
                        }
                    }, |_| Message::SaveSettings);
                }
                Task::none()
            }
            Message::ClearAds => {
                let db_clone = Arc::clone(&self.db);
                Task::perform(async move {
                    if let Some(db) = &*db_clone {
                        let _ = db.clear_non_bookmarked().await;
                        db.get_filtered_jobs(&[], Utc::now().year(), Utc::now().month()).await
                    } else {
                        Ok(vec![])
                    }
                }, |res| Message::SearchResult(res.map_err(|e| e.to_string())))
            }
            Message::OpenBrowser(index) => {
                if let Some(ad) = self.ads.get(index) {
                    if let Some(url) = &ad.webpage_url {
                        let _ = webbrowser::open(url);
                    }
                }
                Task::none()
            }
            Message::SendEmail(index) => {
                if let Some(ad) = self.ads.get(index) {
                    let subject = format!("Jobbtips: {}", ad.headline);
                    let employer = ad.employer.as_ref().and_then(|e| e.name.as_ref()).map(|s| s.as_str()).unwrap_or("Okänd");
                    let body = format!("Kolla in detta jobb!\n\nRubrik: {}\nArbetsgivare: {}\nLänk: {}", 
                        ad.headline, 
                        employer,
                        ad.webpage_url.as_deref().unwrap_or("Ingen länk")
                    );
                    let mailto = format!("mailto:?subject={}&body={}", 
                        urlencoding::encode(&subject), 
                        urlencoding::encode(&body)
                    );
                    let _ = webbrowser::open(&mailto);
                }
                Task::none()
            }
            Message::CopyAd(index) => {
                if let Some(ad) = self.ads.get(index) {
                    let employer = ad.employer.as_ref().and_then(|e| e.name.as_ref()).map(|s| s.as_str()).unwrap_or("Okänd");
                    let desc = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
                    let content = format!("{}\nArbetsgivare: {}\n\n{}\n\nLänk: {}", 
                        ad.headline, employer, desc, ad.webpage_url.as_deref().unwrap_or("N/A")
                    );
                    return iced::clipboard::write(content);
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
        
        Task::perform(async move {
            if let Some(db) = &*db_clone {
                match filter {
                    InboxFilter::All => db.get_filtered_jobs(&[], year, month).await,
                    InboxFilter::Bookmarked => db.get_filtered_jobs(&[AdStatus::Bookmarked, AdStatus::ThumbsUp], year, month).await,
                    InboxFilter::ThumbsUp => db.get_filtered_jobs(&[AdStatus::ThumbsUp], year, month).await,
                    InboxFilter::Applied => db.get_filtered_jobs(&[AdStatus::Applied], year, month).await,
                }
            } else {
                Ok(vec![])
            }
        }, |res| Message::SearchResult(res.map_err(|e| e.to_string())))
    }

    fn view(&self) -> Element<'_, Message> {
        let nav_bar = row![
            button("Inbox").on_press(Message::GoToPage(Page::Inbox)),
            button("Inställningar").on_press(Message::GoToPage(Page::Settings)),
            space::horizontal(),
            text("Jobseeker Gnag").size(20).color(Color::from_rgb(0.4, 0.4, 0.4)),
        ].spacing(10).padding(10).align_y(Alignment::Center);

        let search_controls = if self.is_searching {
            row![text("Söker...").color(Color::from_rgb(0.0, 0.5, 1.0))]
        } else {
            row![
                text("Område:").size(16).color(Color::from_rgb(0.9, 0.9, 0.9)),
                button("1").on_press(Message::Search(1)),
                button("2").on_press(Message::Search(2)),
                button("3").on_press(Message::Search(3)),
                space::horizontal(),
                button("Töm").on_press(Message::ClearAds),
            ].spacing(10).align_y(Alignment::Center)
        };

        let toolbar = container(search_controls)
            .width(Length::Fill)
            .padding(10)
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.1, 0.1, 0.15).into()),
                ..Default::default()
            });

        let content: Element<Message> = match self.page {
            Page::Inbox => self.view_inbox(),
            Page::Settings => self.view_settings(),
        };

        column![
            nav_bar,
            toolbar,
            rule::horizontal(1),
            container(content).width(Length::Fill).height(Length::Fill)
        ].into()
    }

    fn view_inbox(&self) -> Element<'_, Message> {
        let filter_bar = row![
            button("Alla").on_press(Message::SetFilter(InboxFilter::All)),
            button("🔖 Bokm.").on_press(Message::SetFilter(InboxFilter::Bookmarked)),
            button("👍 Toppen").on_press(Message::SetFilter(InboxFilter::ThumbsUp)),
            button("✅ Sökta").on_press(Message::SetFilter(InboxFilter::Applied)),
        ].spacing(5).align_y(Alignment::Center);

        let month_navigator = row![
            button("<").on_press(Message::ChangeMonth(-1)),
            text(format!("{:04}-{:02}", self.current_year, self.current_month)).size(16),
            button(">").on_press(Message::ChangeMonth(1)),
        ].spacing(10).align_y(Alignment::Center);

        let mut sidebar_content = column![filter_bar, month_navigator].spacing(10).width(Length::Fill);

        if let Some(err) = &self.error_msg {
            sidebar_content = sidebar_content.push(
                container(text(err).color(Color::from_rgb(1.0, 0.3, 0.3))).padding(10)
            );
        }

        if self.ads.is_empty() && !self.is_searching {
            sidebar_content = sidebar_content.push(
                container(text("Här var det tomt.")).padding(20)
            );
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
                    button("Nej 👎").on_press(Message::UpdateStatus(index, AdStatus::Rejected)),
                    button("Spara 🔖").on_press(Message::UpdateStatus(index, AdStatus::Bookmarked)),
                    button("Toppen 👍").on_press(Message::UpdateStatus(index, AdStatus::ThumbsUp)),
                    button("HAR SÖKT ✅").on_press(Message::UpdateStatus(index, AdStatus::Applied)),
                    space::horizontal(),
                    button("Webb 🌐").on_press(Message::OpenBrowser(index)),
                    button("E-post ✉").on_press(Message::SendEmail(index)),
                    button("Kopiera 📋").on_press(Message::CopyAd(index)),
                ].spacing(10);

                container(
                    scrollable(
                        column![
                            action_toolbar,
                            text(&ad.headline).size(30).width(Length::Fill).color(Color::WHITE),
                            row![
                                text(ad.employer.as_ref().and_then(|e| e.name.clone()).unwrap_or_else(|| "Okänd arbetsgivare".into())).size(20),
                                text(format!("Publicerad: {}", ad.publication_date.split('T').next().unwrap_or(&ad.publication_date))).color(Color::from_rgb(0.5, 0.5, 0.5)),
                            ].spacing(20),
                            button("Betygsätt med AI").on_press(Message::RateAd(index)),
                            text(ad.description.as_ref().and_then(|d| d.text.clone()).unwrap_or_else(|| "Ingen beskrivning tillgänglig".into()))
                        ].spacing(15).padding(10)
                    )
                ).width(Length::Fill).height(Length::Fill).padding(10)
            } else {
                container(text("Annonsen hittades inte")).width(Length::Fill).height(Length::Fill).padding(10)
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

    fn ad_row<'a>(&self, i: usize, ad: &'a JobAd) -> Element<'a, Message> {
        let (status_text, _icon_color) = match ad.status {
            Some(AdStatus::Rejected) => ("[X] ", Color::from_rgb(0.8, 0.3, 0.3)),
            Some(AdStatus::Bookmarked) => ("[*] ", Color::from_rgb(0.3, 0.6, 0.8)),
            Some(AdStatus::ThumbsUp) => ("[+] ", Color::from_rgb(0.3, 0.8, 0.3)),
            Some(AdStatus::Applied) => ("[OK] ", Color::from_rgb(0.5, 0.5, 0.5)),
            _ => if !ad.is_read { ("( ) ", Color::WHITE) } else { ("    ", Color::WHITE) },
        };

        let rating_text = match ad.rating {
            Some(r) => format!("[{}★]", r),
            None => "[---]".to_string(),
        };

        let date_str = ad.publication_date.split('T').next().unwrap_or(&ad.publication_date);
        let short_date = if date_str.len() > 5 { &date_str[5..] } else { date_str };
        let keyword_text = ad.search_keyword.as_deref().unwrap_or("---");

        button(
            row![
                text(status_text).color(Color::WHITE),
                column![
                    text(&ad.headline).size(18).width(Length::Fill).color(Color::WHITE),
                    row![
                        text(rating_text).size(14).color(Color::from_rgb(1.0, 1.0, 0.0)),
                        text(ad.employer.as_ref().and_then(|e| e.name.clone()).unwrap_or_default())
                            .size(14)
                            .color(Color::from_rgb(0.8, 0.8, 0.8))
                            .width(Length::Fill),
                        text(short_date).size(14).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ].spacing(5),
                    text(format!("Sökord: {}", keyword_text)).size(14).color(Color::from_rgb(0.0, 0.8, 0.8))
                ].spacing(2)
            ].spacing(10).align_y(Alignment::Center)
        )
        .on_press(Message::SelectAd(i))
        .width(Length::Fill)
        .padding(8)
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        container(
            scrollable(
                column![
                    text("Inställningar").size(30),
                    column![
                        text("Sökord"),
                        text_input("t.ex. rust, go", &self.settings.keywords)
                            .on_input(Message::SettingsKeywordsChanged),
                    ].spacing(5),
                    column![
                        text("Svartlista"),
                        text_input("Ord att dölja", &self.settings.blacklist_keywords)
                            .on_input(Message::SettingsBlacklistChanged),
                    ].spacing(5),
                    column![
                        text("Område 1: Nordvästra Skåne"),
                        text_input("Koder", &self.settings.locations_p1)
                            .on_input(Message::SettingsLocP1Changed),
                    ].spacing(5),
                    column![
                        text("Område 2: Malmö / Lund"),
                        text_input("Koder", &self.settings.locations_p2)
                            .on_input(Message::SettingsLocP2Changed),
                    ].spacing(5),
                    column![
                        text("Område 3: Resten"),
                        text_input("Koder", &self.settings.locations_p3)
                            .on_input(Message::SettingsLocP3Changed),
                    ].spacing(5),
                    column![
                        text("Min Profil"),
                        text_input("Beskrivning", &self.settings.my_profile)
                            .on_input(Message::SettingsProfileChanged),
                    ].spacing(5),
                    column![
                        text("AI Endpoint"),
                        text_input("URL", &self.settings.ollama_url)
                            .on_input(Message::SettingsOllamaUrlChanged),
                    ].spacing(5),
                ].spacing(20).padding(20)
            )
        ).width(Length::Fill).height(Length::Fill).into()
    }
}
