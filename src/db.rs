use redb::{Database, TableDefinition, ReadableTable};
use crate::models::{JobAd, AdStatus, AppSettings};
use anyhow::{Result, Context};
use chrono::Utc;
use std::sync::Arc;

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");
const APPLICATIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_applications");
const SETTINGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("settings");

/// RedB database wrapper. Uses JSON serialization for values to support
/// complex job advertisement and settings objects while keeping the key-value structure.
#[derive(Clone, Debug)]
pub struct Db {
    database: Arc<Database>,
}

impl Db {
    /// Opens or creates the RedB database at the given path.
    /// Tables are automatically initialized if they don't exist.
    pub async fn new(db_path: &str) -> Result<Self> {
        let db = Database::create(db_path)
            .context("Failed to create/open RedB database")?;

        // Initiera tabeller
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(JOB_ADS_TABLE)?;
            let _ = write_txn.open_table(APPLICATIONS_TABLE)?;
            let _ = write_txn.open_table(SETTINGS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { database: Arc::new(db) })
    }

    // --- Inställningar ---
    /// Saves the application settings as a JSON blob.
    pub async fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS_TABLE)?;
            let json = serde_json::to_string(settings)?;
            table.insert("current", json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_settings(&self) -> Result<Option<AppSettings>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(SETTINGS_TABLE)?;
        if let Some(json_handle) = table.get("current")? {
            let settings: AppSettings = serde_json::from_str(json_handle.value())?;
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }

    // --- Jobbapplikationer ---
    /// Drafts are stored indexed by job_id.
    pub async fn save_application_draft(&self, job_id: &str, content: &str) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(APPLICATIONS_TABLE)?;
            table.insert(job_id, content)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_application_draft(&self, job_id: &str) -> Result<Option<String>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(APPLICATIONS_TABLE)?;
        let value = table.get(job_id)?;
        Ok(value.map(|v| v.value().to_string()))
    }

    // --- Jobbannonser ---
    /// Primary storage for fetched job ads. Deduplication is handled by job ID.
    pub async fn save_job_ad(&self, ad: &JobAd) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
            let json = serde_json::to_string(ad)?;
            table.insert(ad.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Fetches jobs based on status and time (year/month).
    /// Rejected jobs are excluded by default unless explicitly requested.
    pub async fn get_filtered_jobs(&self, status_filter: &[AdStatus], year: Option<i32>, month: Option<u32>) -> Result<Vec<JobAd>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;

        let mut ads = Vec::new();
        for item in table.iter()? {
            let (_, json_handle) = item?;
            let ad_json = json_handle.value();
            let ad: JobAd = match serde_json::from_str(ad_json) {
                Ok(ad) => ad,
                Err(_) => continue,
            };

            if !status_filter.is_empty() {
                if let Some(status) = ad.status {
                    if !status_filter.contains(&status) { continue; }
                } else { continue; }
            } else if ad.status == Some(AdStatus::Rejected) {
                // By default, don't show rejected ads in the main inbox
                continue;
            }

            if let (Some(y), Some(m)) = (year, month) {
                let mut matched = false;
                let year_str = y.to_string();
                let month_str = format!("{:02}", m);

                // Try to match publication date against the requested month
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ad.publication_date) {
                    use chrono::Datelike;
                    if dt.year() == y && dt.month() == m { matched = true; }
                }
                else if let Ok(dt) = chrono::NaiveDate::parse_from_str(&ad.publication_date, "%Y-%m-%d") {
                    use chrono::Datelike;
                    if dt.year() == y && dt.month() == m { matched = true; }
                }
                else {
                    let prefix = format!("{}-{}", year_str, month_str);
                    if ad.publication_date.starts_with(&prefix) {
                        matched = true;
                    }
                }

                if matched {
                    ads.push(ad);
                }
            } else {
                ads.push(ad);
            }
        }

        // Sorting: Priority order is Applied > Bookmarked > Created
        ads.sort_by(|a, b| {
            let date_a = a.applied_at.or(a.bookmarked_at).unwrap_or(a.internal_created_at);
            let date_b = b.applied_at.or(b.bookmarked_at).unwrap_or(b.internal_created_at);
            date_b.cmp(&date_a)
        });
        Ok(ads)
    }

    /// Updates status and automatically sets the corresponding timestamp (applied_at/bookmarked_at).
    pub async fn update_ad_status(&self, id: &str, status: Option<AdStatus>) -> Result<()> {
        let mut ad = self.get_job_ad(id).await?.context("Ad not found")?;
        ad.status = status;

        let now = Utc::now();
        if let Some(s) = status {
            match s {
                AdStatus::Applied => ad.applied_at = Some(now),
                AdStatus::Bookmarked | AdStatus::ThumbsUp => ad.bookmarked_at = Some(now),
                _ => {}
            }
        }

        self.save_job_ad(&ad).await?;
        Ok(())
    }

    pub async fn get_job_ad(&self, id: &str) -> Result<Option<JobAd>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;
        if let Some(json_handle) = table.get(id)? {
            let ad: JobAd = serde_json::from_str(json_handle.value())?;
            Ok(Some(ad))
        } else {
            Ok(None)
        }
    }

    pub async fn mark_as_read(&self, id: &str) -> Result<()> {
        if let Some(mut ad) = self.get_job_ad(id).await? {
            ad.is_read = true;
            self.save_job_ad(&ad).await?;
        }
        Ok(())
    }

    pub async fn update_rating(&self, id: &str, rating: u8) -> Result<()> {
        if let Some(mut ad) = self.get_job_ad(id).await? {
            ad.rating = Some(rating);
            self.save_job_ad(&ad).await?;
        }
        Ok(())
    }

    pub async fn clear_non_bookmarked(&self) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
            let mut keys_to_remove = Vec::new();

            for item in table.iter()? {
                let (id_handle, json_handle) = item?;
                let ad: JobAd = serde_json::from_str(json_handle.value())?;

                let status = ad.status.unwrap_or(AdStatus::New);
                if status == AdStatus::New || status == AdStatus::Rejected {
                    keys_to_remove.push(id_handle.value().to_string());
                }
            }

            for key in keys_to_remove {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
