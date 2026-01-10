/*
Daily export binary for Jobseeker

- Exports the current set of "applied" jobs (status == 4 or applied_at present)
  to CSV files in the per-user data directory under `exports/`.
- Only writes a new timestamped file when the CSV content differs from the most
  recent export (to avoid duplicate exports with only timestamp differences).
- Intended to be scheduled (systemd user timer or cron). Can be run manually.

Usage:
  cargo run --bin daily_export -- [--db /path/to/jobseeker.db] [--dry-run] [--limit N]

Notes:
- The script determines the DB path via `jobseeker::default_db_path()` unless
  overridden with `--db`.
- The CSV format is: id,headline,employer_name,city,publication_date,applied_at
*/

use anyhow::Context;
use chrono::Utc;
use redb::{Database, ReadableTable, TableDefinition};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");

fn usage_and_exit() -> ! {
    eprintln!(
        "Usage: daily_export [--db <path>] [--dry-run] [--limit N]\n\n\
         Options:\n  --db <path>   Use explicit DB path (overrides default)\n  --dry-run     Do everything except write files\n  --limit N     Limit number of rows exported (for testing)\n"
    );
    std::process::exit(1);
}

fn quote_csv_field(s: &str) -> String {
    if s.contains('"') || s.contains(',') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn pick_db_path(override_path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p;
    }
    jobseeker::default_db_path().unwrap_or_else(|| PathBuf::from("jobseeker.db"))
}

fn default_export_dir(db_path: &Path) -> PathBuf {
    if let Some(parent) = db_path.parent() {
        parent.join("exports")
    } else {
        PathBuf::from("./exports")
    }
}

fn latest_export_in_dir(dir: &Path) -> Option<PathBuf> {
    let mut entries = match fs::read_dir(dir) {
        Ok(e) => e.filter_map(|x| x.ok()).collect::<Vec<_>>(),
        Err(_) => return None,
    };
    // We use lexicographic order on filenames since timestamp is YYYYMMDD-HHMMSS
    entries.sort_by_key(|e| e.file_name());
    for entry in entries.into_iter().rev() {
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.starts_with("applied-") && s.ends_with(".csv") {
            return Some(entry.path());
        }
    }
    None
}

fn build_csv_rows_from_db(db_path: &Path, limit: Option<usize>) -> Result<String, anyhow::Error> {
    let db = Database::create(db_path)
        .with_context(|| format!("Failed to open redb at {}", db_path.display()))?;
    let read_txn = db
        .begin_read()
        .context("Failed to begin read transaction")?;
    let table = read_txn
        .open_table(JOB_ADS_TABLE)
        .context("Failed to open job_ads table")?;

    let mut rows = Vec::new();

    for item in table.iter()? {
        let (_k, v) = item?;
        let raw = v.value();
        // parse JSON
        if let Ok(json) = serde_json::from_str::<Value>(raw) {
            let status = json.get("status").and_then(|x| x.as_i64()).unwrap_or(0);
            let applied_at = json
                .get("applied_at")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if status == 4 || !applied_at.is_empty() {
                let id = json
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let headline = json
                    .get("headline")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let employer_name = json
                    .get("employer_name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let city = json
                    .get("city")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let publication_date = json
                    .get("publication_date")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();

                rows.push((
                    id,
                    headline,
                    employer_name,
                    city,
                    publication_date,
                    applied_at,
                ));
            }
        }
        if let Some(n) = limit
            && rows.len() >= n
        {
            break;
        }
    }

    // header + rows
    let mut out = String::new();
    out.push_str("id,headline,employer_name,city,publication_date,applied_at\n");
    for (id, headline, employer, city, pubd, applied) in rows {
        let line = format!(
            "{},{},{},{},{},{}\n",
            quote_csv_field(&id),
            quote_csv_field(&headline),
            quote_csv_field(&employer),
            quote_csv_field(&city),
            quote_csv_field(&pubd),
            quote_csv_field(&applied)
        );
        out.push_str(&line);
    }
    Ok(out)
}

fn write_if_changed(
    export_dir: &Path,
    csv_content: &str,
) -> Result<Option<PathBuf>, anyhow::Error> {
    fs::create_dir_all(export_dir)
        .with_context(|| format!("Failed to create export dir {}", export_dir.display()))?;

    if let Some(latest) = latest_export_in_dir(export_dir)
        && let Ok(existing) = fs::read_to_string(&latest)
        && existing == csv_content
    {
        // No change
        return Ok(None);
    }

    let ts = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("applied-{}.csv", ts);
    let path = export_dir.join(&filename);
    let mut f = fs::File::create(&path)
        .with_context(|| format!("Failed to create export file {}", path.display()))?;
    f.write_all(csv_content.as_bytes())
        .with_context(|| format!("Failed to write CSV to {}", path.display()))?;

    // Also update 'latest.csv' (atomic replace)
    let latest = export_dir.join("latest.csv");
    let tmp = export_dir.join(format!(".latest.tmp.{}", ts));
    fs::write(&tmp, csv_content)?;
    fs::rename(&tmp, &latest)?;

    Ok(Some(path))
}

fn main() -> Result<(), anyhow::Error> {
    let mut args = env::args().skip(1);
    let mut db_override: Option<PathBuf> = None;
    let mut dry_run = false;
    let mut limit: Option<usize> = None;

    while let Some(a) = args.next() {
        match a.as_str() {
            "--db" => {
                if let Some(p) = args.next() {
                    db_override = Some(PathBuf::from(p));
                } else {
                    usage_and_exit();
                }
            }
            "--dry-run" => dry_run = true,
            "--limit" => {
                if let Some(n) = args.next() {
                    limit = n.parse::<usize>().ok();
                } else {
                    usage_and_exit();
                }
            }
            "-h" | "--help" => usage_and_exit(),
            _ => usage_and_exit(),
        }
    }

    let db_path = pick_db_path(db_override);
    let export_dir = default_export_dir(&db_path);

    println!("DB path: {}", db_path.display());
    println!("Export dir: {}", export_dir.display());

    // Build new CSV content
    let csv_content = build_csv_rows_from_db(&db_path, limit).context("Failed to build CSV")?;

    if csv_content.trim().is_empty() {
        println!("No applied rows to export (CSV would be empty).");
        // We consider this a successful run (nothing to do).
        return Ok(());
    }

    if dry_run {
        println!(
            "Dry-run: would write CSV with size {} bytes.",
            csv_content.len()
        );
        return Ok(());
    }

    // Write only when content changed
    match write_if_changed(&export_dir, &csv_content)? {
        Some(path) => {
            println!("Wrote new export: {}", path.display());
        }
        None => {
            println!("No change detected; export skipped.");
        }
    }

    Ok(())
}
