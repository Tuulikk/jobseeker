/*
CLI: fix_applied
Purpose: Clear 'applied' status for given job IDs (with a backup).

Usage:
  cargo run --bin fix_applied -- [--db /path/to/jobseeker.db] [--yes] [--dry-run] id1 id2 ...

Flags:
  --db <path>    : Use explicit DB path (otherwise uses per-user default if available).
  -y, --yes      : Skip confirmation prompt and apply changes immediately.
  --dry-run      : Show planned changes but don't modify the DB.
  -h, --help     : Show usage.

Behavior:
- Creates a timestamped backup of the DB file before making any changes.
- For each provided ID:
    - Clears the 'applied_at' field (removes it)
    - If the ad had been 'Applied' (status == 4), sets:
        - status = 2 (Bookmarked) if bookmarked_at exists,
        - otherwise status = 0 (New)
- All changes are performed in a single write transaction so they are atomic.
- If the DB is locked (app running), the tool will report the error; close the app and try again.
*/

use anyhow::{Context, Result};
use redb::{Database, TableDefinition};
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");

fn usage_and_exit() -> ! {
    eprintln!(
        "Usage: fix_applied [--db <path>] [--yes] [--dry-run] id1 id2 ...
Options:
  --db <path>    Use explicit DB path (default: per-user DB or ./jobseeker.db)
  -y, --yes      Apply without confirmation
  --dry-run      Show changes but don't apply
  -h, --help     Show this help"
    );
    std::process::exit(1);
}

fn default_db_path_or_local() -> PathBuf {
    jobseeker::default_db_path().unwrap_or_else(|| PathBuf::from("jobseeker.db"))
}

fn timestamped_backup_name(path: &Path) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = format!("{}.fix_applied.bak.{}", path.display(), ts);
    PathBuf::from(s)
}

fn prompt_confirm() -> bool {
    print!("Proceed with the changes? (y/N): ");
    io::stdout().flush().ok();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_ok() {
        let t = line.trim().to_lowercase();
        t == "y" || t == "yes"
    } else {
        false
    }
}

fn main() -> Result<()> {
    // Parse args
    let mut args = env::args().skip(1);
    let mut db_path_arg: Option<PathBuf> = None;
    let mut yes = false;
    let mut dry_run = false;
    let mut ids: Vec<String> = Vec::new();

    while let Some(a) = args.next() {
        match a.as_str() {
            "--db" => {
                if let Some(p) = args.next() {
                    db_path_arg = Some(PathBuf::from(p));
                } else {
                    eprintln!("Missing value for --db");
                    usage_and_exit();
                }
            }
            "-y" | "--yes" => yes = true,
            "--dry-run" => dry_run = true,
            "-h" | "--help" => usage_and_exit(),
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                usage_and_exit();
            }
            other => ids.push(other.to_string()),
        }
    }

    if ids.is_empty() {
        eprintln!("No IDs supplied.");
        usage_and_exit();
    }

    // Determine DB path
    let db_path = db_path_arg.unwrap_or_else(default_db_path_or_local);

    if !db_path.exists() {
        eprintln!("Database file not found at {}.", db_path.display());
        eprintln!("If your DB is in a different location, pass --db <path>");
        std::process::exit(1);
    }

    println!("Database path: {}", db_path.display());

    // Create a backup copy
    let backup = timestamped_backup_name(&db_path);
    fs::copy(&db_path, &backup).with_context(|| {
        format!(
            "Failed to create backup of DB {} -> {}",
            db_path.display(),
            backup.display()
        )
    })?;
    println!("Backup created at: {}", backup.display());

    // Open DB for read to inspect current values
    let db = Database::create(&db_path).with_context(|| {
        format!(
            "Failed to open DB at {} (is the app running and holding a lock?)",
            db_path.display()
        )
    })?;

    let read_txn = db
        .begin_read()
        .context("Failed to begin read transaction.")?;
    let read_table = read_txn
        .open_table(JOB_ADS_TABLE)
        .context("Failed to open job_ads table")?;

    // Collect updates
    struct Plan {
        id: String,
        old_status: i64,
        old_applied: Option<String>,
        old_bookmarked: Option<String>,
        new_status: i64,
    }
    let mut plans = Vec::<(Plan, Value)>::new();

    for id in &ids {
        match read_table.get(id.as_str()) {
            Ok(Some(v)) => {
                let raw = v.value();
                let mut val: Value = serde_json::from_str(raw)
                    .with_context(|| format!("Failed to parse JSON for id {}", id))?;

                let old_status = val.get("status").and_then(|x| x.as_i64()).unwrap_or(0i64);
                let old_applied = val
                    .get("applied_at")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string());
                let old_bookmarked = val
                    .get("bookmarked_at")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string());

                // Determine new status after clearing 'applied'
                let new_status = if old_bookmarked.is_some() {
                    2 // Bookmarked
                } else {
                    0 // New
                };

                // Prepare a modified copy
                if let Some(map) = val.as_object_mut() {
                    map.remove("applied_at");
                    map.insert("status".to_string(), json!(new_status));
                }

                plans.push((
                    Plan {
                        id: id.clone(),
                        old_status,
                        old_applied,
                        old_bookmarked,
                        new_status,
                    },
                    val,
                ));
            }
            Ok(None) => {
                println!("ID {} not found in database; skipping.", id);
            }
            Err(e) => {
                eprintln!("Error reading id {}: {}", id, e);
            }
        }
    }

    if plans.is_empty() {
        println!("No applicable IDs to update. Exiting.");
        return Ok(());
    }

    // Summarize planned changes
    println!("Planned changes:");
    for (p, _) in &plans {
        println!(
            "- ID {}: status {} -> {}, applied_at: {} -> cleared, bookmarked_at: {:?}",
            p.id,
            p.old_status,
            p.new_status,
            p.old_applied.as_deref().unwrap_or("<none>"),
            p.old_bookmarked
        );
    }

    if dry_run {
        println!("Dry-run requested; no changes will be performed.");
        return Ok(());
    }

    if !yes && !prompt_confirm() {
        println!("Aborted by user. No changes made.");
        return Ok(());
    }

    // Apply all changes in a single write transaction
    let write_txn = db
        .begin_write()
        .context("Failed to begin write transaction.")?;
    {
        let mut write_table = write_txn
            .open_table(JOB_ADS_TABLE)
            .context("Failed to open job_ads table for writing")?;

        for (p, val) in plans.iter() {
            let new_json = serde_json::to_string(val)
                .with_context(|| format!("Failed to serialize updated JSON for id {}", p.id))?;
            write_table
                .insert(p.id.as_str(), new_json.as_str())
                .with_context(|| format!("Failed to write id {} into DB", p.id))?;
            println!("Applied change to ID {}.", p.id);
        }
    }
    write_txn
        .commit()
        .context("Failed to commit write transaction.")?;

    println!(
        "All changes applied. Backup retained at: {}",
        backup.display()
    );
    println!("You can verify with the app now (open app or run the dump tool).");

    Ok(())
}
