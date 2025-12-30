use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum AdStatus {
    New = 0,
    Rejected = 1,
    Bookmarked = 2,
    ThumbsUp = 3,
    Applied = 4,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobAd {
    pub id: String,
    pub headline: String,
    pub description: Option<Description>,
    pub employer: Option<Employer>,
    pub application_details: Option<ApplicationDetails>,
    pub webpage_url: Option<String>,
    pub publication_date: String,
    pub last_application_date: Option<String>,
    pub occupation: Option<Occupation>,
    pub workplace_address: Option<WorkplaceAddress>,
    
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub rating: Option<u8>,
    #[serde(default)]
    pub bookmarked_at: Option<DateTime<Utc>>,
    #[serde(default = "Utc::now")]
    pub internal_created_at: DateTime<Utc>,
    #[serde(default)]
    pub search_keyword: Option<String>,
    #[serde(default)]
    pub status: Option<AdStatus>,
    #[serde(default)]
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Description {
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Employer {
    pub name: Option<String>,
    pub workplace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationDetails {
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Occupation {
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkplaceAddress {
    pub city: Option<String>,
    pub municipality: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub keywords: String,
    pub blacklist_keywords: String,
    pub locations_p1: String,
    pub locations_p2: String,
    pub locations_p3: String,
    pub my_profile: String,
    pub ollama_url: String,
}

impl AppSettings {
    pub fn load() -> Self {
        std::fs::read_to_string("settings.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("settings.json", s);
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            keywords: "it, Supporttekniker, helpdesk, kundtjänst, kundsupport".to_string(),
            blacklist_keywords: "databas, barnvakt, körkort, barnflicka, nanny, myNanny, undersköterska, parkarbetare".to_string(),
            locations_p1: "1283, 1277, 1260, 1292, 1284, 1276, 1231, 1282, 1261".to_string(),
            locations_p2: "1280, 1281".to_string(),
            locations_p3: "".to_string(),
            my_profile: "Jag är en serviceinriktad person med erfarenhet inom IT-support och kundservice.".to_string(),
            ollama_url: "http://localhost:11434/v1".to_string(),
        }
    }
}