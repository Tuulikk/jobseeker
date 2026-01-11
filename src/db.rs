use redb::{Database, TableDefinition, ReadableTable};
use crate::models::{JobAd, AdStatus, AppSettings};
use anyhow::{Result, Context};
use chrono::Utc;
use std::sync::Arc;

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");
const APPLICATIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_applications");
const SETTINGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("settings");

#[derive(Clone, Debug)]
pub struct Db {
    database: Arc<Database>,
}

impl Db {
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

    pub async fn get_all_drafts(&self) -> Result<Vec<(String, String, String)>> {
        let read_txn = self.database.begin_read()?;
        let apps_table = read_txn.open_table(APPLICATIONS_TABLE)?;
        let ads_table = read_txn.open_table(JOB_ADS_TABLE)?;
        
        let mut drafts = Vec::new();
        for item in apps_table.iter()? {
            let (id_handle, _content_handle) = item?;
            let id = id_handle.value();
            
            let headline = if let Some(ad_json) = ads_table.get(id)? {
                match serde_json::from_str::<JobAd>(ad_json.value()) {
                    Ok(ad) => ad.headline,
                    Err(_) => "Okänd annons".to_string(),
                }
            } else {
                "Okänd annons".to_string()
            };

            drafts.push((id.to_string(), headline, "Senast sparad".to_string()));
        }
        Ok(drafts)
    }

    // --- Jobbannonser ---
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
                continue;
            }

            if let (Some(y), Some(m)) = (year, month) {
                let date_to_check = if ad.status == Some(AdStatus::Applied) {
                    ad.applied_at
                } else if ad.status == Some(AdStatus::Bookmarked) || ad.status == Some(AdStatus::ThumbsUp) {
                    ad.bookmarked_at
                } else {
                    Some(ad.internal_created_at)
                };

                if let Some(dt) = date_to_check {
                    use chrono::Datelike;
                    if dt.year() == y && dt.month() == m {
                        ads.push(ad);
                    }
                }
            } else {
                ads.push(ad);
            }
        }

        ads.sort_by(|a, b| b.publication_date.cmp(&a.publication_date));
        Ok(ads)
    }

    pub async fn update_ad_status(&self, id: &str, status: AdStatus) -> Result<()> {
        let mut ad = self.get_job_ad(id).await?.context("Ad not found")?;
        ad.status = Some(status);
        
        let now = Utc::now();
        match status {
            AdStatus::Applied => ad.applied_at = Some(now),
            AdStatus::Bookmarked | AdStatus::ThumbsUp => ad.bookmarked_at = Some(now),
            _ => {}
        }
        
        self.save_job_ad(&ad).await?;
        Ok(())
    }

    async fn get_job_ad(&self, id: &str) -> Result<Option<JobAd>> {
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