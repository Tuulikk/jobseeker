/*
Utility to fix missing fields in job ads

This utility reads all job ads from the database and ensures they have the
required fields. If a field is missing, it will be added with a default value.

Usage:
  cargo run --bin fix_missing_fields [--db <path>] [--dry-run] [--verbose]

Safety:
  - Creates a backup of the database before making changes
  - Shows what will be changed before applying changes (with --dry-run)
*/

use anyhow::{Context, Result};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
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
    #[serde(default = "default_false")]
    pub is_read: bool,
    #[serde(default)]
    pub rating: Option<u8>,
    #[serde(default)]
    pub bookmarked_at: Option<String>,
    pub internal_created_at: String,
    #[serde(default)]
    pub search_keyword: Option<String>,
    #[serde(default = "default_status")]
    pub status: i32,
    #[serde(default)]
    pub applied_at: Option<String>,
}

fn default_false() -> bool {
    false
}

fn default_status() -> i32 {
    0 // New
}

fn usage() {
    eprintln!(
        r#"Usage:
  fix_missing_fields [--db <path>] [--dry-run] [--verbose]

Defaults:
  --db ~/.local/share/jobseeker/jobseeker.db

Options:
  --dry-run     Show what would be changed without actually writing
  --verbose      Print extra debug info
"#
    );
}

fn fix_missing_fields(db_path: &Path, dry_run: bool, verbose: bool) -> Result<(usize, usize)> {
    let db = Database::create(db_path).context("Failed to open database")?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(JOB_ADS_TABLE)?;

    let mut ads_to_update: Vec<(String, StoredJobAd)> = Vec::new();
    let mut missing_fields: Vec<String> = Vec::new();

    for item_res in table.iter()? {
        let (key, value) = item_res?;
        let id = key.value().to_string();
        let raw_json = value.value();

        // Try to parse - if it fails, we know there's a missing field
        match serde_json::from_str::<StoredJobAd>(raw_json) {
            Ok(_) => {
                // All fields present, no need to update
            }
            Err(e) => {
                // Check if it's a missing field error
                let error_msg = e.to_string();
                if error_msg.contains("missing field") {
                    if verbose {
                        eprintln!("Job {} has missing field(s): {}", id, error_msg);
                    }

                    // Parse with a more lenient structure to get the data
                    if let Ok(mut partial) = serde_json::from_str::<serde_json::Value>(raw_json) {
                        // Add missing fields with defaults
                        if partial.get("is_read").is_none() {
                            partial["is_read"] = serde_json::Value::Bool(false);
                            missing_fields.push(format!("{}: is_read", id));
                        }
                        if partial.get("status").is_none() {
                            partial["status"] = serde_json::Value::Number(0.into());
                            missing_fields.push(format!("{}: status", id));
                        }
                        if partial.get("rating").is_none() {
                            partial["rating"] = serde_json::Value::Null;
                            missing_fields.push(format!("{}: rating", id));
                        }
                        if partial.get("search_keyword").is_none() {
                            partial["search_keyword"] = serde_json::Value::Null;
                            missing_fields.push(format!("{}: search_keyword", id));
                        }
                        if partial.get("applied_at").is_none() {
                            partial["applied_at"] = serde_json::Value::Null;
                            missing_fields.push(format!("{}: applied_at", id));
                        }
                        if partial.get("bookmarked_at").is_none() {
                            partial["bookmarked_at"] = serde_json::Value::Null;
                            missing_fields.push(format!("{}: bookmarked_at", id));
                        }
                        if partial.get("qualifications").is_none() {
                            partial["qualifications"] = serde_json::Value::Null;
                            missing_fields.push(format!("{}: qualifications", id));
                        }
                        if partial.get("additional_information").is_none() {
                            partial["additional_information"] = serde_json::Value::Null;
                            missing_fields.push(format!("{}: additional_information", id));
                        }

                        // Now try to parse the fixed JSON
                        let fixed_json = serde_json::to_string(&partial)?;
                        if let Ok(stored) = serde_json::from_str::<StoredJobAd>(&fixed_json) {
                            ads_to_update.push((id.clone(), stored));
                        } else {
                            eprintln!("Failed to parse fixed JSON for job {}", id);
                        }
                    } else {
                        eprintln!("Failed to parse job {} as JSON", id);
                    }
                } else {
                    eprintln!("Unexpected error parsing job {}: {}", id, error_msg);
                }
            }
        }
    }

    if verbose {
        println!("\n=== MISSING FIELDS FOUND ===");
        for field in &missing_fields {
            println!("{}", field);
        }
    }

    if dry_run {
        println!("\n=== DRY RUN SUMMARY ===");
        println!("Ads to update: {}", ads_to_update.len());
        println!("Dry run - no changes made");
        return Ok((ads_to_update.len(), 0));
    }

    // Perform the updates
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(JOB_ADS_TABLE)?;

        for (id, ad) in &ads_to_update {
            let json = serde_json::to_string(ad).context("Failed to serialize job")?;
            table
                .insert(id.as_str(), json.as_str())
                .context("Failed to update job")?;
        }
    }
    write_txn.commit()?;

    Ok((ads_to_update.len(), missing_fields.len()))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
        return Ok(());
    }

    let mut db_path = format!(
        "{}/.local/share/jobseeker/jobseeker.db",
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    );
    let mut dry_run = false;
    let mut verbose = false;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--db" | "-d" => {
                i += 1;
                if i < args.len() {
                    db_path = args[i].clone();
                } else {
                    eprintln!("Missing value for --db");
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

    let path = Path::new(&db_path);

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Database file {} does not exist",
            path.display()
        ));
    }

    if verbose {
        println!("Database: {}", path.display());
        println!("Dry-run: {}", dry_run);
    }

    // Always create backup before modifying
    if !dry_run {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let backup_name = format!("{}.fix_fields.bak.{}", path.display(), now);
        fs::copy(path, &backup_name).context("Failed to create backup")?;
        println!("Created backup: {}", backup_name);
    }

    let (updated, missing_count) = fix_missing_fields(path, dry_run, verbose)?;

    println!("\n=== RESULT ===");
    println!("Updated {} ads with missing fields", updated);
    println!("Total missing fields found: {}", missing_count);

    if dry_run {
        println!("Dry-run finished. No changes made.");
    } else {
        println!("Fix completed successfully.");
    }

    Ok(())
}
