//!/usr/bin/env rust
//! Library helper to migrate an existing SQLite `jobseeker.db` to a Redb store.
//!
//! Exposes a small, well-typed API that can be used from a CLI binary (already present)
//! and from the application startup code to perform automatic migration when needed.
//!
//! Typical usage:
//! ```no_run
//! let res = db_migration::migrate_sqlite_to_redb("jobseeker.db".as_ref(), "jobseeker.db.new".as_ref())?;
//! println!("migrated {} ads, {} apps; december ids: {}", res.ads, res.apps, res.december_ids.len());
//! ```

use anyhow::{Context, Result};
use chrono::Datelike;
use redb::{Database, ReadableTable, TableDefinition};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Table definitions used for Redb insertion. Kept local to avoid coupling.
const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");
const JOB_APPLICATIONS_TABLE: TableDefinition<&str, &str> =
    TableDefinition::new("job_applications");

/// Minimal representation of what is stored in the Redb `job_ads` table (JSON).
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
    #[serde(default)]
    pub qualifications: Option<String>,
    #[serde(default)]
    pub additional_information: Option<String>,
    pub is_read: bool,
    pub rating: Option<u8>,
    pub bookmarked_at: Option<String>,
    pub internal_created_at: String,
    pub search_keyword: Option<String>,
    pub status: i32,
    pub applied_at: Option<String>,
}

/// Minimal representation of what is stored in the Redb `job_applications` table (JSON).
#[derive(Debug, Serialize, Deserialize)]
struct StoredApplication {
    pub job_id: String,
    pub content: String,
    pub updated_at: String,
}

/// Result returned from a successful migration.
pub struct MigrationResult {
    pub ads: usize,
    pub apps: usize,
    /// Set of job IDs that were present in December (based on `internal_created_at`) in the source db.
    pub december_ids: HashSet<String>,
}

/// Detects whether a file appears to be a SQLite DB, a Redb DB, or unknown.
/// Simple header-based detection (works for standard SQLite headers and Redb header).
pub fn detect_db_format(path: &Path) -> Result<DbFormat> {
    let mut f = File::open(path).with_context(|| format!("opening file {}", path.display()))?;
    let mut head = [0u8; 16];
    let n = f
        .read(&mut head)
        .with_context(|| format!("reading header from {}", path.display()))?;
    let head = &head[..n];
    if head.starts_with(b"SQLite format 3") {
        Ok(DbFormat::Sqlite)
    } else if head.starts_with(b"redb") {
        Ok(DbFormat::Redb)
    } else {
        Ok(DbFormat::Unknown)
    }
}

/// Enum describing a DB format guess.
#[derive(Debug, PartialEq, Eq)]
pub enum DbFormat {
    Sqlite,
    Redb,
    Unknown,
}

/// Migrate an SQLite-based source DB into a new Redb DB at `dst`.
///
/// - `src` must point to a readable SQLite DB containing `job_ads` and `job_applications`.
/// - `dst` must not already exist (this function will error if it does).
/// - On success the caller can (and should) replace the original `src` with the new `dst`,
///   or keep the original as a backup.
///
/// Returns counts and a set of December job IDs (for verification).
pub fn migrate_sqlite_to_redb(src: &Path, dst: &Path) -> Result<MigrationResult> {
    // Safety checks
    if !src.exists() {
        anyhow::bail!("Source SQLite DB '{}' does not exist", src.display());
    }
    if dst.exists() {
        anyhow::bail!(
            "Destination '{}' already exists; choose another path or remove the file first",
            dst.display()
        );
    }

    // Open SQLite connection
    let conn =
        Connection::open(src).with_context(|| format!("opening sqlite DB '{}'", src.display()))?;

    // Ensure expected tables exist
    if !table_exists(&conn, "job_ads")? || !table_exists(&conn, "job_applications")? {
        anyhow::bail!("Source DB is missing required tables (job_ads, job_applications)");
    }

    // Discover columns for robust reading
    let ad_cols = table_columns(&conn, "job_ads")?;
    let app_cols = table_columns(&conn, "job_applications")?;

    // Build index maps for column access
    let ad_index: HashMap<String, usize> = ad_cols
        .iter()
        .enumerate()
        .map(|(i, c)| (c.clone(), i))
        .collect();
    let app_index: HashMap<String, usize> = app_cols
        .iter()
        .enumerate()
        .map(|(i, c)| (c.clone(), i))
        .collect();

    // Collect ads from sqlite
    let select_ads = format!("SELECT {} FROM job_ads", ad_cols.join(", "));
    let mut stmt = conn
        .prepare(&select_ads)
        .context("preparing job_ads select")?;
    let rows = stmt
        .query_map([], |row| {
            // helpers to get by index with type conversion
            let get_str_opt = |name: &str| -> rusqlite::Result<Option<String>> {
                if let Some(&idx) = ad_index.get(name) {
                    row.get::<usize, Option<String>>(idx)
                } else {
                    Ok(None)
                }
            };
            let get_i64_opt = |name: &str| -> rusqlite::Result<Option<i64>> {
                if let Some(&idx) = ad_index.get(name) {
                    row.get::<usize, Option<i64>>(idx)
                } else {
                    Ok(None)
                }
            };

            // required
            let id = if let Some(&idx) = ad_index.get("id") {
                row.get::<usize, String>(idx)?
            } else {
                return Err(rusqlite::Error::InvalidQuery);
            };
            let headline = if let Some(&idx) = ad_index.get("headline") {
                row.get::<usize, String>(idx)?
            } else {
                String::new()
            };

            let publication_date = get_str_opt("publication_date")?
                .or_else(|| get_str_opt("internal_created_at").ok().flatten())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            let s = StoredJobAd {
                id,
                headline,
                description: get_str_opt("description")?,
                employer_name: get_str_opt("employer_name")?,
                employer_workplace: get_str_opt("employer_workplace")?,
                application_url: get_str_opt("application_url")?,
                webpage_url: get_str_opt("webpage_url")?,
                publication_date,
                last_application_date: get_str_opt("last_application_date")?,
                occupation_label: get_str_opt("occupation_label")?,
                city: get_str_opt("city")?,
                municipality: get_str_opt("municipality")?,
                working_hours_label: get_str_opt("working_hours_label")?,
                qualifications: get_str_opt("qualifications")?,
                additional_information: get_str_opt("additional_information")?,
                is_read: get_i64_opt("is_read")?.map(|v| v != 0).unwrap_or(false),
                rating: get_i64_opt("rating")?
                    .and_then(|v| (0..=u8::MAX as i64).contains(&v).then(|| v as u8)),
                bookmarked_at: get_str_opt("bookmarked_at")?,
                internal_created_at: get_str_opt("internal_created_at")?
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                search_keyword: get_str_opt("search_keyword")?,
                status: get_i64_opt("status")?.unwrap_or(0) as i32,
                applied_at: get_str_opt("applied_at")?,
            };
            Ok(s)
        })
        .context("querying job_ads rows")?;

    let mut ads: Vec<StoredJobAd> = Vec::new();
    let mut december_ids: HashSet<String> = HashSet::new();

    for r in rows {
        let ad = r?;
        if let Some(dt) = chrono::DateTime::parse_from_rfc3339(&ad.internal_created_at).ok() {
            if dt.month() == 12 {
                december_ids.insert(ad.id.clone());
            }
        }
        ads.push(ad);
    }

    // Collect applications
    let select_apps = format!("SELECT {} FROM job_applications", app_cols.join(", "));
    let mut stmt = conn
        .prepare(&select_apps)
        .context("preparing job_applications select")?;
    let rows = stmt
        .query_map([], |row| {
            let get_str_opt = |name: &str| -> rusqlite::Result<Option<String>> {
                if let Some(&idx) = app_index.get(name) {
                    row.get::<usize, Option<String>>(idx)
                } else {
                    Ok(None)
                }
            };

            let job_id = if let Some(&idx) = app_index.get("job_id") {
                row.get::<usize, String>(idx)?
            } else {
                return Err(rusqlite::Error::InvalidQuery);
            };

            let content = get_str_opt("content")?.unwrap_or_default();
            let updated_at =
                get_str_opt("updated_at")?.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            Ok(StoredApplication {
                job_id,
                content,
                updated_at,
            })
        })
        .context("querying job_applications rows")?;

    let mut apps: Vec<StoredApplication> = Vec::new();
    for r in rows {
        apps.push(r?);
    }

    // Create redb and write data
    let db =
        Database::create(dst).with_context(|| format!("creating redb at '{}'", dst.display()))?;
    let write_txn = db.begin_write().context("begin redb write txn")?;
    {
        let mut ads_table = write_txn
            .open_table(JOB_ADS_TABLE)
            .context("open job_ads table")?;
        let mut apps_table = write_txn
            .open_table(JOB_APPLICATIONS_TABLE)
            .context("open job_applications table")?;

        for ad in &ads {
            let json = serde_json::to_string(ad).context("serialize StoredJobAd")?;
            ads_table
                .insert(ad.id.as_str(), json.as_str())
                .with_context(|| format!("insert ad id={}", ad.id))?;
        }

        for app in &apps {
            let json = serde_json::to_string(app).context("serialize StoredApplication")?;
            apps_table
                .insert(app.job_id.as_str(), json.as_str())
                .with_context(|| format!("insert app job_id={}", app.job_id))?;
        }
    }
    write_txn.commit().context("commit redb write txn")?;

    // Verify by reading destination db (using same handle)
    let read_txn = db.begin_read().context("begin read txn for verification")?;
    let ads_table = read_txn.open_table(JOB_ADS_TABLE)?;
    let apps_table = read_txn.open_table(JOB_APPLICATIONS_TABLE)?;

    let mut dest_december_ids: HashSet<String> = HashSet::new();
    let mut dest_ad_count: usize = 0;
    for item in ads_table.iter()? {
        let (_k, v) = item?;
        let stored: StoredJobAd =
            serde_json::from_str(v.value()).context("deserialize JSON from redb")?;
        if let Some(dt) = chrono::DateTime::parse_from_rfc3339(&stored.internal_created_at).ok() {
            if dt.month() == 12 {
                dest_december_ids.insert(stored.id.clone());
            }
        }
        dest_ad_count += 1;
    }

    let mut dest_app_count: usize = 0;
    for _ in apps_table.iter()? {
        dest_app_count += 1;
    }

    // Sanity check: ensure december id sets match (caller can decide what to do with mismatch)
    if december_ids != dest_december_ids {
        // Not fatal, but surface as context for caller
        // We'll still return with counts and the december_ids from the source.
        // Logging a context may be useful for calling code to surface to user.
        tracing::warn!("December ID set mismatch between source and dest during migration");
    }

    Ok(MigrationResult {
        ads: dest_ad_count,
        apps: dest_app_count,
        december_ids,
    })
}

/// Return true if the named table exists in sqlite DB.
fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
    let mut rows = stmt.query([table])?;
    Ok(rows.next()?.is_some())
}

/// Helper to return the ordered column names for a table.
fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info('{}')", table))?;
    let cols = stmt
        .query_map([], |row| row.get::<usize, String>(1))
        .context("querying pragma table_info")?;
    let mut v = Vec::new();
    for c in cols {
        v.push(c?);
    }
    Ok(v)
}
