use crate::models::{
    AdStatus, ApplicationDetails, Description, Employer, JobAd, Occupation, WorkingHours,
    WorkplaceAddress,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

// Table definitions
const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");
const JOB_APPLICATIONS_TABLE: TableDefinition<&str, &str> =
    TableDefinition::new("job_applications");

// Serializable version of JobAd for storage
#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredJobAd {
    pub id: String,
    pub headline: String,
    pub description: Option<String>,
    pub employer_name: Option<String>,
    pub employer_workplace: Option<String>,
    pub application_url: Option<String>,
    pub webpage_url: Option<String>,
    pub publication_date: String,
    pub last_application_date: Option<String>,
    pub occupation_label: Option<String>,
    pub city: Option<String>,
    pub municipality: Option<String>,
    pub working_hours_label: Option<String>,
    pub is_read: bool,
    pub rating: Option<u8>,
    pub bookmarked_at: Option<String>,
    pub internal_created_at: String,
    pub search_keyword: Option<String>,
    pub status: i32,
    pub applied_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredApplication {
    pub job_id: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct Db {
    db: std::sync::Arc<Database>,
}

impl Db {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Database::create(db_path).context("Failed to create/open database")?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _table = write_txn.open_table(JOB_ADS_TABLE)?;
            let _table = write_txn.open_table(JOB_APPLICATIONS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self {
            db: std::sync::Arc::new(db),
        })
    }

    pub async fn save_application_draft(&self, job_id: &str, content: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let draft = StoredApplication {
            job_id: job_id.to_string(),
            content: content.to_string(),
            updated_at: now,
        };

        let json = serde_json::to_string(&draft)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_APPLICATIONS_TABLE)?;
            table.insert(job_id, json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_application_draft(&self, job_id: &str) -> Result<Option<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_APPLICATIONS_TABLE)?;

        if let Some(value) = table.get(job_id)? {
            let draft: StoredApplication = serde_json::from_str(value.value())?;
            Ok(Some(draft.content))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_drafts(&self) -> Result<Vec<(String, String, String)>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_APPLICATIONS_TABLE)?;

        let mut drafts = Vec::new();
        for item in table.iter()? {
            let (key, value) = item?;
            let draft: StoredApplication = serde_json::from_str(value.value())?;

            // Get headline from job_ads
            let ads_table = read_txn.open_table(JOB_ADS_TABLE)?;
            let headline = if let Some(ad_json) = ads_table.get(key.value())? {
                let stored: StoredJobAd = serde_json::from_str(ad_json.value())?;
                stored.headline
            } else {
                "Unknown".to_string()
            };

            drafts.push((draft.job_id, headline, draft.updated_at));
        }

        // Sort by updated_at descending
        drafts.sort_by(|a, b| b.2.cmp(&a.2));
        Ok(drafts)
    }

    pub async fn save_job_ad(&self, ad: &JobAd) -> Result<()> {
        let stored = StoredJobAd {
            id: ad.id.clone(),
            headline: ad.headline.clone(),
            description: ad.description.as_ref().and_then(|d| d.text.clone()),
            employer_name: ad.employer.as_ref().and_then(|e| e.name.clone()),
            employer_workplace: ad.employer.as_ref().and_then(|e| e.workplace.clone()),
            application_url: ad.application_details.as_ref().and_then(|a| a.url.clone()),
            webpage_url: ad.webpage_url.clone(),
            publication_date: ad.publication_date.clone(),
            last_application_date: ad.last_application_date.clone(),
            occupation_label: ad.occupation.as_ref().and_then(|o| o.label.clone()),
            city: ad.workplace_address.as_ref().and_then(|w| w.city.clone()),
            municipality: ad
                .workplace_address
                .as_ref()
                .and_then(|w| w.municipality.clone()),
            working_hours_label: ad.working_hours_type.as_ref().and_then(|w| w.label.clone()),
            is_read: ad.is_read,
            rating: ad.rating,
            bookmarked_at: ad.bookmarked_at.map(|d| d.to_rfc3339()),
            internal_created_at: ad.internal_created_at.to_rfc3339(),
            search_keyword: ad.search_keyword.clone(),
            status: ad.status.unwrap_or(AdStatus::New) as i32,
            applied_at: ad.applied_at.map(|d| d.to_rfc3339()),
        };

        let json = serde_json::to_string(&stored)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
            table.insert(stored.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_filtered_jobs(
        &self,
        status_filter: &[AdStatus],
        year: Option<i32>,
        month: Option<u32>,
    ) -> Result<Vec<JobAd>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        let mut ads = Vec::new();

        for item in table.iter()? {
            let (_key, value) = item?;
            let stored: StoredJobAd = serde_json::from_str(value.value())?;

            // Filter by status
            let ad_status = match stored.status {
                0 => AdStatus::New,
                1 => AdStatus::Rejected,
                2 => AdStatus::Bookmarked,
                3 => AdStatus::ThumbsUp,
                4 => AdStatus::Applied,
                _ => AdStatus::New,
            };

            // Apply status filter
            if !status_filter.is_empty() {
                // For historical views with month filter
                if year.is_some() && month.is_some() && status_filter.len() == 1 {
                    match status_filter[0] {
                        AdStatus::Applied => {
                            if stored.applied_at.is_none() {
                                continue;
                            }
                        }
                        AdStatus::Bookmarked | AdStatus::ThumbsUp => {
                            if stored.bookmarked_at.is_none() {
                                continue;
                            }
                        }
                        _ => {
                            if !status_filter.contains(&ad_status) {
                                continue;
                            }
                        }
                    }
                } else {
                    if !status_filter.contains(&ad_status) {
                        continue;
                    }
                }
            } else {
                // "All" filter - exclude rejected
                if ad_status == AdStatus::Rejected {
                    continue;
                }
            }

            // Apply time filter
            if let (Some(y), Some(m)) = (year, month) {
                let mut matches = false;

                if status_filter.len() == 1 {
                    // Specific status filter - check relevant date
                    let date_str = match status_filter[0] {
                        AdStatus::Applied => stored.applied_at.as_ref(),
                        AdStatus::Bookmarked | AdStatus::ThumbsUp => stored.bookmarked_at.as_ref(),
                        _ => Some(&stored.internal_created_at),
                    };

                    if let Some(ds) = date_str {
                        if let Ok(dt) = DateTime::parse_from_rfc3339(ds) {
                            let dt_utc = dt.with_timezone(&Utc);
                            if dt_utc.year() == y && dt_utc.month() == m {
                                matches = true;
                            }
                        }
                    }
                } else {
                    // "All" filter - check any activity date
                    for date_str in [
                        stored.applied_at.as_ref(),
                        stored.bookmarked_at.as_ref(),
                        Some(&stored.internal_created_at),
                    ]
                    .iter()
                    .filter_map(|&d| d)
                    {
                        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
                            let dt_utc = dt.with_timezone(&Utc);
                            if dt_utc.year() == y && dt_utc.month() == m {
                                matches = true;
                                break;
                            }
                        }
                    }
                }

                if !matches {
                    continue;
                }
            }

            // Convert to JobAd
            ads.push(self.stored_to_job_ad(&stored)?);
        }

        // Sort by publication date descending
        ads.sort_by(|a, b| b.publication_date.cmp(&a.publication_date));
        Ok(ads)
    }

    pub async fn update_ad_status(&self, id: &str, status: AdStatus) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        if let Some(value) = table.get(id)? {
            let mut stored: StoredJobAd = serde_json::from_str(value.value())?;
            stored.status = status as i32;

            match status {
                AdStatus::Applied => {
                    stored.applied_at = Some(now);
                }
                AdStatus::Bookmarked | AdStatus::ThumbsUp => {
                    stored.bookmarked_at = Some(now);
                }
                _ => {}
            }

            drop(table);
            drop(read_txn);

            let json = serde_json::to_string(&stored)?;
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
                table.insert(id, json.as_str())?;
            }
            write_txn.commit()?;
        }

        Ok(())
    }

    pub async fn mark_as_read(&self, id: &str) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        if let Some(value) = table.get(id)? {
            let mut stored: StoredJobAd = serde_json::from_str(value.value())?;
            stored.is_read = true;

            drop(table);
            drop(read_txn);

            let json = serde_json::to_string(&stored)?;
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
                table.insert(id, json.as_str())?;
            }
            write_txn.commit()?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_job_by_id(&self, job_id: &str) -> Result<Option<JobAd>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        if let Some(value) = table.get(job_id)? {
            let stored: StoredJobAd = serde_json::from_str(value.value())?;
            Ok(Some(self.stored_to_job_ad(&stored)?))
        } else {
            Ok(None)
        }
    }

    pub async fn update_rating(&self, id: &str, rating: u8) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        if let Some(value) = table.get(id)? {
            let mut stored: StoredJobAd = serde_json::from_str(value.value())?;
            stored.rating = Some(rating);

            drop(table);
            drop(read_txn);

            let json = serde_json::to_string(&stored)?;
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
                table.insert(id, json.as_str())?;
            }
            write_txn.commit()?;
        }

        Ok(())
    }

    pub async fn update_draft_headline(&self, job_id: &str, new_headline: &str) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        if let Some(value) = table.get(job_id)? {
            let mut stored: StoredJobAd = serde_json::from_str(value.value())?;
            stored.headline = new_headline.to_string();

            drop(table);
            drop(read_txn);

            let json = serde_json::to_string(&stored)?;
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
                table.insert(job_id, json.as_str())?;
            }
            write_txn.commit()?;
        }

        Ok(())
    }

    pub async fn clear_non_bookmarked(&self) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        let mut to_delete = Vec::new();
        for item in table.iter()? {
            let (key, value) = item?;
            let stored: StoredJobAd = serde_json::from_str(value.value())?;

            // Keep if bookmarked, thumbs up, or applied
            if stored.status != 2 && stored.status != 3 && stored.status != 4 {
                to_delete.push(key.value().to_string());
            }
        }

        drop(table);
        drop(read_txn);

        if !to_delete.is_empty() {
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
                for id in to_delete {
                    table.remove(id.as_str())?;
                }
            }
            write_txn.commit()?;
        }

        Ok(())
    }

    fn stored_to_job_ad(&self, stored: &StoredJobAd) -> Result<JobAd> {
        Ok(JobAd {
            id: stored.id.clone(),
            headline: stored.headline.clone(),
            description: stored.description.as_ref().map(|text| Description {
                text: Some(text.clone()),
            }),
            employer: if stored.employer_name.is_some() || stored.employer_workplace.is_some() {
                Some(Employer {
                    name: stored.employer_name.clone(),
                    workplace: stored.employer_workplace.clone(),
                })
            } else {
                None
            },
            application_details: stored
                .application_url
                .as_ref()
                .map(|url| ApplicationDetails {
                    url: Some(url.clone()),
                }),
            webpage_url: stored.webpage_url.clone(),
            publication_date: stored.publication_date.clone(),
            last_application_date: stored.last_application_date.clone(),
            occupation: stored.occupation_label.as_ref().map(|label| Occupation {
                label: Some(label.clone()),
            }),
            workplace_address: if stored.city.is_some() || stored.municipality.is_some() {
                Some(WorkplaceAddress {
                    city: stored.city.clone(),
                    municipality: stored.municipality.clone(),
                })
            } else {
                None
            },
            working_hours_type: stored
                .working_hours_label
                .as_ref()
                .map(|label| WorkingHours {
                    label: Some(label.clone()),
                }),
            is_read: stored.is_read,
            rating: stored.rating,
            bookmarked_at: stored
                .bookmarked_at
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            internal_created_at: DateTime::parse_from_rfc3339(&stored.internal_created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            search_keyword: stored.search_keyword.clone(),
            status: Some(match stored.status {
                0 => AdStatus::New,
                1 => AdStatus::Rejected,
                2 => AdStatus::Bookmarked,
                3 => AdStatus::ThumbsUp,
                4 => AdStatus::Applied,
                _ => AdStatus::New,
            }),
            applied_at: stored
                .applied_at
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_get_application_draft() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Db::new(db_path.to_str().unwrap()).unwrap();

        // Save a draft
        db.save_application_draft("job123", "My application content")
            .await
            .unwrap();

        // Retrieve it
        let content = db.get_application_draft("job123").await.unwrap();
        assert_eq!(content, Some("My application content".to_string()));

        // Non-existent draft
        let missing = db.get_application_draft("job999").await.unwrap();
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_job_ad_save_and_filter() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Db::new(db_path.to_str().unwrap()).unwrap();

        let ad = JobAd {
            id: "test123".to_string(),
            headline: "Test Job".to_string(),
            description: Some(Description {
                text: Some("Description".to_string()),
            }),
            employer: None,
            application_details: None,
            webpage_url: None,
            publication_date: "2025-01-07".to_string(),
            last_application_date: None,
            occupation: None,
            workplace_address: None,
            working_hours_type: None,
            is_read: false,
            rating: None,
            bookmarked_at: None,
            internal_created_at: Utc::now(),
            search_keyword: Some("rust".to_string()),
            status: Some(AdStatus::New),
            applied_at: None,
        };

        db.save_job_ad(&ad).await.unwrap();

        let ads = db.get_filtered_jobs(&[], None, None).await.unwrap();
        assert_eq!(ads.len(), 1);
        assert_eq!(ads[0].id, "test123");
    }

    #[tokio::test]
    async fn test_status_filter() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Db::new(db_path.to_str().unwrap()).unwrap();

        let mut ad = JobAd {
            id: "test456".to_string(),
            headline: "Bookmarked Job".to_string(),
            description: None,
            employer: None,
            application_details: None,
            webpage_url: None,
            publication_date: "2025-01-07".to_string(),
            last_application_date: None,
            occupation: None,
            workplace_address: None,
            working_hours_type: None,
            is_read: false,
            rating: None,
            bookmarked_at: None,
            internal_created_at: Utc::now(),
            search_keyword: None,
            status: Some(AdStatus::New),
            applied_at: None,
        };

        db.save_job_ad(&ad).await.unwrap();
        db.update_ad_status("test456", AdStatus::Bookmarked)
            .await
            .unwrap();

        let bookmarked = db
            .get_filtered_jobs(&[AdStatus::Bookmarked], None, None)
            .await
            .unwrap();
        assert_eq!(bookmarked.len(), 1);
        assert_eq!(bookmarked[0].status, Some(AdStatus::Bookmarked));
    }
}
