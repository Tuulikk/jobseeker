/*
Merge SQLite data into existing Redb database

This utility reads job data from a SQLite database and merges it into an existing
Redb database. It preserves existing data in the Redb database and only adds
jobs that don't already exist (based on ID).

Usage:
  cargo run --bin merge_sqlite_to_redb [--src <sqlite-db>] [--dst <redb-db>]
                                     [--dry-run] [--verbose]

Safety:
  - Always creates a backup of the destination database before modification
  - Checks for existing IDs to avoid duplicates
  - Reports what will be changed before applying changes
*/

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use redb::{Database, ReadableTable, TableDefinition};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");

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

fn usage() {
    eprintln!(
        r#"Usage:
  merge_sqlite_to_redb [--src <sqlite-db>] [--dst <redb-db>] [--dry-run] [--verbose]

Defaults:
  --src ./jobseeker.db
  --dst ~/.local/share/jobseeker/jobseeker.db

Options:
  --dry-run     Show what would be changed without actually writing
  --verbose     Print extra debug info
"#
    );
}

fn get_existing_redb_ids(db: &Database) -> Result<HashSet<String>> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(JOB_ADS_TABLE)?;

    let mut ids = HashSet::new();
    let iter = table.iter()?;
    for entry in iter {
        let (key, _) = entry?;
        ids.insert(key.value().to_string());
    }

    Ok(ids)
}

fn read_sqlite_jobs(conn: &Connection) -> Result<Vec<StoredJobAd>> {
    let mut stmt = conn
        .prepare("SELECT * FROM job_ads")
        .context("Failed to prepare SELECT statement")?;

    let mut rows = stmt.query([])?;
    let mut jobs = Vec::new();

    while let Some(row) = rows.next()? {
        let job = StoredJobAd {
            id: row.get("id")?,
            headline: row.get("headline")?,
            description: row.get("description").ok(),
            employer_name: row.get("employer_name").ok(),
            employer_workplace: row.get("employer_workplace").ok(),
            application_url: row.get("application_url").ok(),
            webpage_url: row.get("webpage_url").ok(),
            publication_date: row.get("publication_date")?,
            last_application_date: row.get("last_application_date").ok(),
            occupation_label: row.get("occupation_label").ok(),
            city: row.get("city").ok(),
            municipality: row.get("municipality").ok(),
            working_hours_label: row.get("working_hours_label").ok(),
            qualifications: row.get("qualifications").ok(),
            additional_information: row.get("additional_information").ok(),
            is_read: row.get("is_read").unwrap_or(false),
            rating: row.get("rating").ok(),
            bookmarked_at: row.get("bookmarked_at").ok(),
            internal_created_at: row.get("internal_created_at")?,
            search_keyword: row.get("search_keyword").ok(),
            status: row.get("status").unwrap_or(0),
            applied_at: row.get("applied_at").ok(),
        };

        jobs.push(job);
    }

    Ok(jobs)
}

fn merge_sqlite_to_redb(
    src_path: &Path,
    dst_path: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<(usize, usize)> {
    // Open source SQLite
    let conn = Connection::open(src_path).context("Failed to open source SQLite DB")?;

    // Read jobs from SQLite
    let sqlite_jobs = read_sqlite_jobs(&conn)?;

    if verbose {
        println!("Read {} jobs from SQLite", sqlite_jobs.len());
    }

    // Open destination Redb
    let db = Database::open(dst_path).context("Failed to open destination Redb DB")?;

    // Get existing IDs
    let existing_ids = get_existing_redb_ids(&db)?;

    if verbose {
        println!("Found {} existing jobs in Redb", existing_ids.len());
    }

    // Find new jobs to add
    let mut new_jobs: Vec<StoredJobAd> = Vec::new();
    let mut duplicate_ids: Vec<String> = Vec::new();

    for job in &sqlite_jobs {
        if existing_ids.contains(&job.id) {
            duplicate_ids.push(job.id.clone());
        } else {
            new_jobs.push(job.clone());
        }
    }

    println!("\n=== MERGE SUMMARY ===");
    println!("Jobs in SQLite: {}", sqlite_jobs.len());
    println!("Jobs in Redb: {}", existing_ids.len());
    println!("Jobs to add: {}", new_jobs.len());
    println!("Duplicate IDs (skipped): {}", duplicate_ids.len());

    if dry_run {
        if verbose && !new_jobs.is_empty() {
            println!("\n=== JOBS TO BE ADDED ===");
            for job in &new_jobs {
                println!(
                    "ID: {}, Status: {}, Headline: {}",
                    job.id, job.status, job.headline
                );
            }
        }
        return Ok((new_jobs.len(), duplicate_ids.len()));
    }

    // Perform the merge
    let write_txn = db.begin_write()?;

    {
        let mut table = write_txn.open_table(JOB_ADS_TABLE)?;

        for job in &new_jobs {
            let json = serde_json::to_string(&job).context("Failed to serialize job")?;
            table
                .insert(job.id.as_str(), json.as_str())
                .context("Failed to insert job")?;
        }
    }

    write_txn.commit().context("Failed to commit transaction")?;

    Ok((new_jobs.len(), duplicate_ids.len()))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
        return Ok(());
    }

    let mut src = "jobseeker.db".to_string();
    let mut dst = format!(
        "{}/.local/share/jobseeker/jobseeker.db",
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    );
    let mut dry_run = false;
    let mut verbose = false;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--src" | "-s" => {
                i += 1;
                if i < args.len() {
                    src = args[i].clone();
                } else {
                    eprintln!("Missing value for --src");
                    usage();
                    return Ok(());
                }
            }
            "--dst" | "-d" => {
                i += 1;
                if i < args.len() {
                    dst = args[i].clone();
                } else {
                    eprintln!("Missing value for --dst");
                    usage();
                    return Ok(());
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--verbose" | "-v" => {
                verbose = true;
            }
            other => {
                eprintln!("Unknown arg: {}", other);
                usage();
                return Ok(());
            }
        }
        i += 1;
    }

    let src_path = Path::new(&src);
    let dst_path = Path::new(&dst);

    if !src_path.exists() {
        return Err(anyhow::anyhow!(
            "Source file {} does not exist",
            src_path.display()
        ));
    }

    if !dst_path.exists() {
        return Err(anyhow::anyhow!(
            "Destination file {} does not exist",
            dst_path.display()
        ));
    }

    if verbose {
        println!("Source: {}", src_path.display());
        println!("Destination: {}", dst_path.display());
        println!("Dry-run: {}", dry_run);
    }

    // Always create backup before modifying
    if !dry_run {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let backup_name = format!("{}.merge_pre.{}", dst_path.display(), now);
        fs::copy(dst_path, &backup_name).context("Failed to create backup")?;
        println!("Created backup: {}", backup_name);
    }

    let (added, skipped) = merge_sqlite_to_redb(src_path, dst_path, dry_run, verbose)?;

    println!("\n=== RESULT ===");
    println!("Added {} new jobs", added);
    println!("Skipped {} duplicate jobs", skipped);

    if dry_run {
        println!("Dry-run finished. No changes made.");
    } else {
        println!("Merge completed successfully.");
    }

    Ok(())
}
