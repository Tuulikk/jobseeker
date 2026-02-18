use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AdStatus {
    New = 0,
    Rejected = 1,
    Bookmarked = 2,
    ThumbsUp = 3,
    Applied = 4,
}

impl<'de> Deserialize<'de> for AdStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        struct AdStatusVisitor;
        impl<'de> serde::de::Visitor<'de> for AdStatusVisitor {
            type Value = AdStatus;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer or string representing AdStatus")
            }
            fn visit_u64<E>(self, v: u64) -> Result<AdStatus, E> where E: serde::de::Error {
                Ok(match v {
                    1 => AdStatus::Rejected,
                    2 => AdStatus::Bookmarked,
                    3 => AdStatus::ThumbsUp,
                    4 => AdStatus::Applied,
                    _ => AdStatus::New,
                })
            }
            fn visit_str<E>(self, v: &str) -> Result<AdStatus, E> where E: serde::de::Error {
                Ok(match v {
                    "Rejected" => AdStatus::Rejected,
                    "Bookmarked" => AdStatus::Bookmarked,
                    "ThumbsUp" => AdStatus::ThumbsUp,
                    "Applied" => AdStatus::Applied,
                    _ => AdStatus::New,
                })
            }
        }
        deserializer.deserialize_any(AdStatusVisitor)
    }
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
    pub working_hours_type: Option<WorkingHours>,
    #[serde(default)]
    pub must_have: Option<Requirements>,
    #[serde(default)]
    pub nice_to_have: Option<Requirements>,
    #[serde(default)]
    pub driving_license_required: bool,
    
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub rating: Option<u8>,
    #[serde(default)]
    pub bookmarked_at: Option<DateTime<Utc>>,
    #[serde(default = "Utc::now")]
    pub internal_created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub search_keyword: Option<String>,
    #[serde(default)]
    pub status: Option<AdStatus>,
    #[serde(default)]
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkingHours {
    pub label: Option<String>,
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
pub struct Requirements {
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub work_experiences: Vec<WorkExperience>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Language {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkExperience {
    pub label: String,
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
    pub sync_path: String,
    pub app_min_count: i32,
    pub app_goal_count: i32,
    pub show_motivation: bool,
    #[serde(default)]
    pub main_cv_id: String,
    #[serde(default)]
    pub show_dev_logs: bool,
    #[serde(default = "default_true")]
    pub auto_extract: bool,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

fn default_true() -> bool { true }

/// User documents (CVs, cover letters, etc.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDocument {
    pub id: String,
    pub name: String,
    pub doc_type: String, // "CV", "Brev", "Annat"
    pub content: String,
    #[serde(default)]
    pub is_main: bool,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Dictionary entries for knowledge base
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictEntry {
    pub key: String,
    pub value: String,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            keywords: "".to_string(),
            blacklist_keywords: "".to_string(),
            locations_p1: "".to_string(),
            locations_p2: "".to_string(),
            locations_p3: "".to_string(),
            my_profile: "".to_string(),
            ollama_url: "http://localhost:11434/v1".to_string(),
            sync_path: "".to_string(),
            app_min_count: 6,
            app_goal_count: 12,
            show_motivation: true,
            main_cv_id: "".to_string(),
            show_dev_logs: false,
            auto_extract: true,
            updated_at: Utc::now(),
        }
    }
}
