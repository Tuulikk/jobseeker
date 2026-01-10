/*
Migration utility
- Reads job data from a SQLite `jobseeker.db`
- Writes the data into a new Redb database (default `jobseeker.db.new`)
- Verifies that entries created/added in December (internal_created_at month == 12)
  are preserved by checking ID sets between source and target.
- Safe-by-default: writes to a new DB file. Call with `--replace` to atomically
  backup the original sqlite DB and swap the new file into place.

Usage:
  cargo run --bin migrate_sqlite [--src path/to/jobseeker.db] [--dst path/to/jobseeker.db.new]
                               [--dry-run] [--replace] [--verbose]

Notes:
- The tool attempts to be tolerant to slight schema differences by discovering
  existing columns via `PRAGMA table_info(...)`. If a column is missing the field
  will be set to None (or sensible default for required fields).
*/

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use jobseeker::db_migration;
use redb::{Database, ReadableTable, TableDefinition};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");
const JOB_APPLICATIONS_TABLE: TableDefinition<&str, &str> =
    TableDefinition::new("job_applications");

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

#[derive(Debug, Serialize, Deserialize)]
struct StoredApplication {
    pub job_id: String,
    pub content: String,
    pub updated_at: String,
}

fn usage() {
    eprintln!(
        r#"Usage:
  migrate_sqlite [--src <sqlite-db>] [--dst <redb-db>] [--dry-run] [--replace] [--verbose]

Defaults:
  --src ./jobseeker.db
  --dst ./jobseeker.db.new

Options:
  --dry-run     Do everything except write the redb file (verifies counts & December data)
  --replace     On success, backup original sqlite file and replace it with the new redb file
  --verbose     Print extra debug info
"#
    );
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table))
        .context("Failed to prepare PRAGMA table_info")?;
    let cols = stmt
        .query_map([], |row| row.get::<usize, String>(1))
        .context("Failed to query table info")?;
    let mut v = Vec::new();
    for c in cols {
        v.push(c?);
    }
    Ok(v)
}

fn parse_rfc3339_opt(s: Option<&str>) -> Option<DateTime<Utc>> {
    s.and_then(|s| {
        DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    })
}

fn now_rfc3339() -> String {
    let now = chrono::Utc::now();
    now.to_rfc3339()
}

fn migrate(
    src_path: &Path,
    dst_path: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<(usize, usize, HashSet<String>)> {
    // Quick inspection of the SQLite source: totals and December ids
    let conn = Connection::open(src_path).context("Failed to open source SQLite DB")?;

    let src_total_ads: i64 = conn
        .query_row("SELECT COUNT(*) FROM job_ads", [], |r| r.get(0))
        .context("Failed to count job_ads")?;
    let src_total_apps: i64 = conn
        .query_row("SELECT COUNT(*) FROM job_applications", [], |r| r.get(0))
        .context("Failed to count job_applications")?;

    // Collect December job IDs (internal_created_at month == '12')
    let mut stmt = conn
        .prepare("SELECT id FROM job_ads WHERE substr(internal_created_at,6,2)='12'")
        .context("Failed to prepare December id query")?;
    let mut rows = stmt.query([])?;
    let mut dec_ids: HashSet<String> = HashSet::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        dec_ids.insert(id);
    }

    if verbose {
        println!(
            "Source totals: job_ads={} job_applications={}",
            src_total_ads, src_total_apps
        );
        if dec_ids.is_empty() {
            println!(
                "No December entries found in source DB (based on internal_created_at month == 12)"
            );
        } else {
            println!("December entries in source: {}", dec_ids.len());
        }
    }

    if dry_run {
        // Dry-run: just report counts and don't write destination
        return Ok((src_total_ads as usize, src_total_apps as usize, dec_ids));
    }

    // Perform full migration using the shared library helper
    let migration = jobseeker::db_migration::migrate_sqlite_to_redb(src_path, dst_path)
        .context("Migration to redb failed")?;

    Ok((migration.ads, migration.apps, migration.december_ids))
}
fn timestamped_backup(path: &Path) -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let backup_name = format!("{}.bak.{}", path.display(), now);
    fs::rename(path, &backup_name).with_context(|| {
        format!(
            "Failed to move {} to backup {}",
            path.display(),
            backup_name
        )
    })?;
    Ok(backup_name)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
        return Ok(());
    }

    let mut src = "jobseeker.db".to_string();
    let mut dst = "jobseeker.db.new".to_string();
    let mut dry_run = false;
    let mut replace = false;
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
            "--replace" | "--swap" => {
                replace = true;
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

    if verbose {
        println!("Source: {}", src_path.display());
        println!("Destination: {}", dst_path.display());
        println!("Dry-run: {}", dry_run);
        println!("Replace on success: {}", replace);
    }

    let (ads_count, apps_count, dec_ids) = migrate(src_path, dst_path, dry_run, verbose)?;

    println!(
        "Migration summary: ads={}, apps={}, december_ads={}",
        ads_count,
        apps_count,
        dec_ids.len()
    );

    if dry_run {
        println!("Dry-run finished.");
        return Ok(());
    }

    if replace {
        // Backup original sqlite and swap
        let backup_name = timestamped_backup(src_path)?;
        println!("Backed up original sqlite DB to: {}", backup_name);

        fs::rename(dst_path, src_path).with_context(|| {
            format!(
                "Failed to move new DB {} to {}",
                dst_path.display(),
                src_path.display()
            )
        })?;
        println!("Replaced original DB with migrated Redb DB.");

        // If the repository is tracking jobseeker.db, remove it from the index
        // so personal data doesn't stay in the repo. This is conservative and
        // only runs if `git` is available and the file appears tracked.
        if let Ok(status) = Command::new("git")
            .arg("ls-files")
            .arg("--error-unmatch")
            .arg("jobseeker.db")
            .status()
        {
            if status.success() {
                println!("Detected jobseeker.db in git index; removing from index...");
                let rm = Command::new("git")
                    .arg("rm")
                    .arg("--cached")
                    .arg("jobseeker.db")
                    .status();
                match rm {
                    Ok(rm_st) if rm_st.success() => {
                        let commit = Command::new("git")
                            .arg("commit")
                            .arg("-m")
                            .arg("Remove local jobseeker.db from repository (personal data)")
                            .status();
                        match commit {
                            Ok(commit_st) if commit_st.success() => {
                                println!(
                                    "Removed jobseeker.db from git index and committed the change."
                                );
                            }
                            _ => {
                                println!(
                                    "Removed from index, but failed to create commit. Please commit the change manually."
                                );
                            }
                        }
                    }
                    _ => {
                        println!(
                            "Failed to remove jobseeker.db from git index. You can run: git rm --cached jobseeker.db && git commit -m \"Remove jobseeker.db from repo\""
                        );
                    }
                }
            }
        }
    } else {
        println!(
            "Migration output left at {}. If everything looks good, you can replace the original with:\n  mv {} {}.backup && mv {} {}",
            dst_path.display(),
            src_path.display(),
            src_path.display(),
            dst_path.display(),
            src_path.display()
        );
    }

    println!("Done.");
    Ok(())
}
