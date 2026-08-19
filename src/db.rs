use crate::models::{JobAd, AppSettings, AdStatus, UserDocument, DictEntry, Profile};
use anyhow::{Result, Context};
use redb::{Database, TableDefinition, ReadableTable};
use std::sync::Arc;
use chrono::Utc;

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");
const APPLICATIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("applications");
const SETTINGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("settings");
const DOCUMENTS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("documents");
const DICTIONARY_TABLE: TableDefinition<&str, &str> = TableDefinition::new("dictionary");
const PROFILES_TABLE: TableDefinition<&str, &str> = TableDefinition::new("profiles");

#[derive(Clone)]
pub struct Db {
    pub database: Arc<Database>,
}

impl Db {
    /// Opens or creates the RedB database at the given path.
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Database::create(db_path)
            .context("Failed to create/open RedB database")?;

        // Initiera tabeller
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(JOB_ADS_TABLE)?;
            let _ = write_txn.open_table(APPLICATIONS_TABLE)?;
            let _ = write_txn.open_table(SETTINGS_TABLE)?;
            let _ = write_txn.open_table(DOCUMENTS_TABLE)?;
            let _ = write_txn.open_table(DICTIONARY_TABLE)?;
            let _ = write_txn.open_table(PROFILES_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { database: Arc::new(db) })
    }

    // --- Inställningar (per profil) ---
    /// Hämta settings-nyckel för en profil. Utan profil_id används "current:" (legacy).
    fn settings_key(profile_id: &str) -> String {
        if profile_id.is_empty() {
            "current:".to_string()
        } else {
            format!("current:{}", profile_id)
        }
    }

    pub async fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let mut settings = settings.clone();
        settings.updated_at = Utc::now();
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS_TABLE)?;
            let json = serde_json::to_string(&settings)?;
            let key = Self::settings_key(&settings.profile_id);
            table.insert(key.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_settings_for(&self, profile_id: &str) -> Result<Option<AppSettings>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(SETTINGS_TABLE)?;
        let key = Self::settings_key(profile_id);
        let settings = table.get(key.as_str())?;
        if let Some(json) = settings {
            let settings: AppSettings = serde_json::from_str(json.value())?;
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }

    /// Ladda inställningar för den aktiva profilen (för bakåtkompatibilitet)
    pub async fn load_settings(&self) -> Result<Option<AppSettings>> {
        // Försök först med "current:" (legacy), fallback till vanliga "current"
        match self.load_settings_for("").await? {
            Some(s) => Ok(Some(s)),
            None => {
                let read_txn = self.database.begin_read()?;
                let table = read_txn.open_table(SETTINGS_TABLE)?;
                if let Some(json) = table.get("current")? {
                    let settings: AppSettings = serde_json::from_str(json.value())?;
                    Ok(Some(settings))
                } else {
                    Ok(None)
                }
            }
        }
    }

    // --- Jobbannonser ---
    pub async fn save_job_ad(&self, ad: &JobAd) -> Result<()> {
        let mut ad = ad.clone();
        ad.updated_at = Utc::now();
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
            let json = serde_json::to_string(&ad)?;
            table.insert(ad.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_filtered_jobs(&self, status_filter: &[AdStatus], year: Option<i32>, month: Option<u32>) -> Result<Vec<JobAd>> {
        self.get_filtered_jobs_for("", status_filter, year, month).await
    }

    pub async fn get_filtered_jobs_for(&self, profile_id: &str, status_filter: &[AdStatus], year: Option<i32>, month: Option<u32>) -> Result<Vec<JobAd>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;
        let mut results = Vec::new();
        
        for result in table.iter()? {
            let (_key, value) = result?;
            let ad: JobAd = serde_json::from_str(value.value())?;
            
            // Filtrera på profil-ID
            if !profile_id.is_empty() && ad.profile_id != profile_id { continue; }
            
            if !status_filter.is_empty() {
                if let Some(status) = ad.status {
                    if !status_filter.contains(&status) { continue; }
                } else if !status_filter.contains(&AdStatus::New) {
                    continue;
                }
            }
            
            if let Some(y) = year {
                let parts: Vec<&str> = ad.publication_date.split('-').collect();
                if parts.len() >= 1 && parts[0].parse::<i32>().unwrap_or(0) != y { continue; }
            }
            
            if let Some(m) = month {
                let parts: Vec<&str> = ad.publication_date.split('-').collect();
                if parts.len() >= 2 && parts[1].parse::<u32>().unwrap_or(0) != m { continue; }
            }
            
            results.push(ad);
        }
        
        results.sort_by(|a, b| b.publication_date.cmp(&a.publication_date));
        Ok(results)
    }

    pub async fn update_ad_status(&self, id: &str, status: Option<AdStatus>) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
            let existing_json = table.get(id)?.map(|j| j.value().to_string());
            if let Some(json) = existing_json {
                let mut ad: JobAd = serde_json::from_str(&json)?;
                ad.status = status;
                ad.updated_at = Utc::now();
                if status == Some(AdStatus::Applied) { ad.applied_at = Some(Utc::now()); }
                let new_json = serde_json::to_string(&ad)?;
                table.insert(id, new_json.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_job_ad(&self, id: &str) -> Result<Option<JobAd>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(JOB_ADS_TABLE)?;
        if let Some(json) = table.get(id)? {
            let ad: JobAd = serde_json::from_str(json.value())?;
            Ok(Some(ad))
        } else {
            Ok(None)
        }
    }

    pub async fn update_rating(&self, id: &str, rating: u8) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(JOB_ADS_TABLE)?;
            let existing_json = table.get(id)?.map(|j| j.value().to_string());
            if let Some(json) = existing_json {
                let mut ad: JobAd = serde_json::from_str(&json)?;
                ad.rating = Some(rating);
                ad.updated_at = Utc::now();
                let new_json = serde_json::to_string(&ad)?;
                table.insert(id, new_json.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    // --- Dokumenthantering ---
    pub async fn save_document(&self, doc: &UserDocument) -> Result<()> {
        let mut doc = doc.clone();
        doc.updated_at = Utc::now();
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(DOCUMENTS_TABLE)?;
            let json = serde_json::to_string(&doc)?;
            table.insert(doc.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_documents(&self) -> Result<Vec<UserDocument>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(DOCUMENTS_TABLE)?;
        let mut results = Vec::new();
        for result in table.iter()? {
            let (_key, value) = result?;
            let doc: UserDocument = serde_json::from_str(value.value())?;
            results.push(doc);
        }
        Ok(results)
    }

    pub async fn delete_document(&self, id: &str) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(DOCUMENTS_TABLE)?;
            table.remove(id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn set_main_cv(&self, id: &str) -> Result<()> {
        let mut docs = self.get_documents().await?;
        for doc in &mut docs {
            doc.is_main = doc.id == id;
            doc.updated_at = Utc::now();
            self.save_document(doc).await?;
        }
        Ok(())
    }

    // --- Ordbok / Kunskapsbas ---
    pub async fn save_dict_entry(&self, entry: &DictEntry) -> Result<()> {
        let mut entry = entry.clone();
        entry.updated_at = Utc::now();
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(DICTIONARY_TABLE)?;
            let json = serde_json::to_string(&entry)?;
            table.insert(entry.key.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_dictionary(&self) -> Result<Vec<DictEntry>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(DICTIONARY_TABLE)?;
        let mut results = Vec::new();
        for result in table.iter()? {
            let (_key, value) = result?;
            let entry: DictEntry = serde_json::from_str(value.value())?;
            results.push(entry);
        }
        Ok(results)
    }

    pub async fn get_dict_entries(&self) -> Result<Vec<DictEntry>> {
        self.get_dictionary().await
    }

    pub async fn delete_dict_entry(&self, key: &str) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(DICTIONARY_TABLE)?;
            table.remove(key)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // --- Profilhantering ---
    pub async fn save_profile(&self, profile: &Profile) -> Result<()> {
        let mut profile = profile.clone();
        profile.updated_at = Utc::now();
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(PROFILES_TABLE)?;
            let json = serde_json::to_string(&profile)?;
            table.insert(profile.id.as_str(), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_profiles(&self) -> Result<Vec<Profile>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(PROFILES_TABLE)?;
        let mut results = Vec::new();
        for result in table.iter()? {
            let (_key, value) = result?;
            let profile: Profile = serde_json::from_str(value.value())?;
            results.push(profile);
        }
        results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(results)
    }

    pub async fn get_profile(&self, id: &str) -> Result<Option<Profile>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(PROFILES_TABLE)?;
        if let Some(json) = table.get(id)? {
            let profile: Profile = serde_json::from_str(json.value())?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_profile(&self, id: &str) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(PROFILES_TABLE)?;
            table.remove(id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Hämta dokument filtrerade på profil
    pub async fn get_documents_for(&self, profile_id: &str) -> Result<Vec<UserDocument>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(DOCUMENTS_TABLE)?;
        let mut results = Vec::new();
        for result in table.iter()? {
            let (_key, value) = result?;
            let doc: UserDocument = serde_json::from_str(value.value())?;
            if profile_id.is_empty() || doc.profile_id == profile_id {
                results.push(doc);
            }
        }
        Ok(results)
    }

    /// Hämta dictionary filtrerat på profil
    pub async fn get_dict_entries_for(&self, profile_id: &str) -> Result<Vec<DictEntry>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(DICTIONARY_TABLE)?;
        let mut results = Vec::new();
        for result in table.iter()? {
            let (_key, value) = result?;
            let entry: DictEntry = serde_json::from_str(value.value())?;
            if profile_id.is_empty() || entry.profile_id == profile_id {
                results.push(entry);
            }
        }
        Ok(results)
    }

    /// Spara aktivt profil-ID i settings-tabellen
    pub async fn set_active_profile_id(&self, profile_id: &str) -> Result<()> {
        let write_txn = self.database.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS_TABLE)?;
            table.insert("_active_profile", profile_id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_active_profile_id(&self) -> Result<Option<String>> {
        let read_txn = self.database.begin_read()?;
        let table = read_txn.open_table(SETTINGS_TABLE)?;
        if let Some(val) = table.get("_active_profile")? {
            let id: String = val.value().to_string();
            if id.is_empty() { Ok(None) } else { Ok(Some(id)) }
        } else {
            Ok(None)
        }
    }
}
