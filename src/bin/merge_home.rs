/*
merge_home.rs

CLI tool to merge applied entries from a home SQLite `jobseeker.db` into the
per-user Redb database used by the app (default: `~/.local/share/Jobseeker/jobseeker.db`),
and to export the final set of applied jobs into a timestamped CSV in an
`exports/` directory under the per-user data dir.

Usage:
  cargo run --bin merge_home -- [--home /path/to/jobseeker.db] [--dest /path/to/jobseeker.db]
                              [--export-dir /path/to/exports] [--yes] [--dry-run]

Notes:
 - Makes a backup of the destination Redb DB before applying changes.
 - If export CSV already exists and its content is identical to the new export,
   no new file is written (prevents duplicate daily exports).
 - Designed to be run when the GUI is not running (file locks may block otherwise).
*/

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use redb::{Database, ReadableTable, TableDefinition};
use rusqlite::Connection;
use serde_json::Value;
use serde_json::json;
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");

#[derive(Debug, Clone)]
struct Record {
    id: String,
    headline: Option<String>,
    employer_name: Option<String>,
    city: Option<String>,
    publication_date: Option<String>,
    applied_at: Option<String>,
    status: Option<i64>,
    bookmarked_at: Option<String>,
    internal_created_at: Option<String>,
}

fn usage_and_exit() -> ! {
    eprintln!(
        "Usage: merge_home [--home <path>] [--dest <path>] [--export-dir <path>] [--yes] [--dry-run]"
    );
    std::process::exit(1);
}

fn default_home_db() -> PathBuf {
    if let Ok(h) = env::var("HOME") {
        PathBuf::from(h).join("jobseeker.db")
    } else {
        PathBuf::from("jobseeker.db")
    }
}

fn default_dest_db() -> PathBuf {
    // jobseeker::default_db_path() is re-exported in crate root; prefer that if present
    if let Some(p) = jobseeker::default_db_path() {
        p
    } else {
        // fallback to local file
        PathBuf::from("jobseeker.db")
    }
}

fn default_export_dir(dest_db: &Path) -> PathBuf {
    // Prefer per-user data dir under the same parent as the dest db
    if let Some(parent) = dest_db.parent() {
        parent.join("exports")
    } else {
        PathBuf::from("./exports")
    }
}

fn timestamped_name(prefix: &str) -> String {
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    format!("{}_{}.csv", prefix, ts)
}

fn quote_csv_field(s: &str) -> String {
    // Simple CSV quoting: double-quote, double internal quotes
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn export_rows_to_csv(
    path: &Path,
    rows: &[(String, String, String, String, String, String)],
) -> Result<()> {
    let mut f =
        fs::File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    // Header
    writeln!(
        f,
        "id,headline,employer_name,city,publication_date,applied_at"
    )?;
    for (id, headline, employer, city, pubdate, applied_at) in rows {
        writeln!(
            f,
            "{},{},{},{},{},{}",
            quote_csv_field(id),
            quote_csv_field(headline),
            quote_csv_field(employer),
            quote_csv_field(city),
            quote_csv_field(pubdate),
            quote_csv_field(applied_at)
        )?;
    }
    Ok(())
}

fn canonical_rows_for_export(
    records: &[Record],
) -> Vec<(String, String, String, String, String, String)> {
    // Build rows and sort by applied_at desc then id for determinism
    let mut rows: Vec<_> = records
        .iter()
        .filter(|r| r.applied_at.is_some() || r.status == Some(4))
        .map(|r| {
            let id = r.id.clone();
            let headline = r.headline.as_deref().unwrap_or("").to_string();
            let employer = r.employer_name.as_deref().unwrap_or("").to_string();
            let city = r.city.as_deref().unwrap_or("").to_string();
            let pubdate = r.publication_date.as_deref().unwrap_or("").to_string();
            let applied_at = r.applied_at.as_deref().unwrap_or("").to_string();
            (id, headline, employer, city, pubdate, applied_at)
        })
        .collect();

    rows.sort_by(|a, b| {
        // Compare applied_at descending, empty strings go last
        match (a.5.as_str(), b.5.as_str()) {
            ("", "") => a.0.cmp(&b.0),
            ("", _) => Ordering::Greater,
            (_, "") => Ordering::Less,
            (x, y) => y.cmp(x).then_with(|| a.0.cmp(&b.0)),
        }
    });
    rows
}

fn read_applied_from_sqlite(path: &Path) -> Result<Vec<Record>> {
    let conn =
        Connection::open(path).with_context(|| format!("Opening sqlite {}", path.display()))?;

    // Query relevant fields
    let sql = "SELECT id, headline, employer_name, city, publication_date, applied_at, status, bookmarked_at, internal_created_at FROM job_ads WHERE applied_at IS NOT NULL OR status = 4";
    let mut stmt = conn.prepare(sql).context("Prepare sqlite query")?;
    let mut rows = stmt.query([]).context("Query sqlite")?;

    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let rec = Record {
            id: row.get::<usize, String>(0)?,
            headline: row.get::<_, Option<String>>(1)?,
            employer_name: row.get::<_, Option<String>>(2)?,
            city: row.get::<_, Option<String>>(3)?,
            publication_date: row.get::<_, Option<String>>(4)?,
            applied_at: row.get::<_, Option<String>>(5)?,
            status: row.get::<_, Option<i64>>(6)?,
            bookmarked_at: row.get::<_, Option<String>>(7)?,
            internal_created_at: row.get::<_, Option<String>>(8)?,
        };
        out.push(rec);
    }
    Ok(out)
}

fn load_all_from_redb(path: &Path) -> Result<Vec<Record>> {
    let db = Database::create(path).with_context(|| format!("open redb {}", path.display()))?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(JOB_ADS_TABLE)?;

    let mut res = Vec::new();
    for item in table.iter()? {
        let (_k, v) = item?;
        let json_str = v.value();
        if let Ok(val) = serde_json::from_str::<Value>(json_str) {
            let rec = Record {
                id: val
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                headline: val
                    .get("headline")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                employer_name: val
                    .get("employer_name")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                city: val
                    .get("city")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                publication_date: val
                    .get("publication_date")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                applied_at: val
                    .get("applied_at")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                status: val.get("status").and_then(|x| x.as_i64()),
                bookmarked_at: val
                    .get("bookmarked_at")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                internal_created_at: val
                    .get("internal_created_at")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
            };
            res.push(rec);
        }
    }
    Ok(res)
}

fn merge_into_redb(dest: &Path, src_records: &[Record]) -> Result<(usize, usize)> {
    // Returns (inserted_count, updated_count)
    let db = Database::create(dest).with_context(|| format!("open redb {}", dest.display()))?;
    let write_txn = db.begin_write().context("begin write txn")?;

    // 1) First pass: sample existing JSON for all relevant IDs into a local map
    let mut presence: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();
    {
        let read_table = write_txn
            .open_table(JOB_ADS_TABLE)
            .context("open job_ads table for sampling")?;
        for r in src_records {
            match read_table.get(r.id.as_str())? {
                Some(val) => {
                    presence.insert(r.id.clone(), Some(val.value().to_string()));
                }
                None => {
                    presence.insert(r.id.clone(), None);
                }
            }
        }
    } // read_table dropped here

    // 2) Second pass: perform inserts/updates using the sampled data (no conflicting borrows)
    let mut inserted = 0usize;
    let mut updated = 0usize;
    {
        let mut table = write_txn
            .open_table(JOB_ADS_TABLE)
            .context("open job_ads table for writing")?;
        for r in src_records {
            match presence.get(&r.id) {
                Some(Some(existing_json)) => {
                    // Parse existing JSON and decide if update needed
                    let mut val: Value =
                        serde_json::from_str(existing_json).context("parse existing json")?;

                    let need_update = match (
                        val.get("applied_at").and_then(|v| v.as_str()),
                        r.applied_at.as_deref(),
                    ) {
                        (Some(a), Some(b)) => a != b,
                        (None, Some(_)) => true,
                        (Some(_), None) => true,
                        (None, None) => false,
                    } || val.get("status").and_then(|v| v.as_i64()).unwrap_or(0)
                        != r.status.unwrap_or(0);

                    if need_update {
                        if let Some(ref a) = r.applied_at {
                            val["applied_at"] = json!(a);
                            val["status"] = json!(4);
                        } else {
                            val.as_object_mut().and_then(|m| m.remove("applied_at"));
                            val["status"] = json!(r.status.unwrap_or(0));
                        }
                        let new_json = serde_json::to_string(&val)?;
                        table.insert(r.id.as_str(), new_json.as_str())?;
                        updated += 1;
                    }
                }
                Some(None) => {
                    // Insert new minimal record
                    let mut val = serde_json::Map::new();
                    val.insert("id".to_string(), Value::String(r.id.clone()));
                    val.insert(
                        "headline".to_string(),
                        Value::String(r.headline.clone().unwrap_or_default()),
                    );
                    if let Some(em) = &r.employer_name {
                        val.insert("employer_name".to_string(), Value::String(em.clone()));
                    }
                    if let Some(c) = &r.city {
                        val.insert("city".to_string(), Value::String(c.clone()));
                    }
                    val.insert(
                        "publication_date".to_string(),
                        Value::String(r.publication_date.clone().unwrap_or_default()),
                    );
                    if let Some(a) = &r.applied_at {
                        val.insert("applied_at".to_string(), Value::String(a.clone()));
                        val.insert(
                            "status".to_string(),
                            Value::Number(serde_json::Number::from(4)),
                        );
                    } else {
                        val.insert(
                            "status".to_string(),
                            Value::Number(serde_json::Number::from(r.status.unwrap_or(0))),
                        );
                    }
                    if let Some(i) = &r.internal_created_at {
                        val.insert("internal_created_at".to_string(), Value::String(i.clone()));
                    }

                    let json = Value::Object(val);
                    table.insert(
                        json.get("id").and_then(|x| x.as_str()).unwrap(),
                        serde_json::to_string(&json)?.as_str(),
                    )?;
                    inserted += 1;
                }
                None => {
                    // Defensive: treat as new
                    let mut val = serde_json::Map::new();
                    val.insert("id".to_string(), Value::String(r.id.clone()));
                    val.insert(
                        "headline".to_string(),
                        Value::String(r.headline.clone().unwrap_or_default()),
                    );
                    if let Some(a) = &r.applied_at {
                        val.insert("applied_at".to_string(), Value::String(a.clone()));
                        val.insert(
                            "status".to_string(),
                            Value::Number(serde_json::Number::from(4)),
                        );
                    } else {
                        val.insert(
                            "status".to_string(),
                            Value::Number(serde_json::Number::from(r.status.unwrap_or(0))),
                        );
                    }
                    if let Some(i) = &r.internal_created_at {
                        val.insert("internal_created_at".to_string(), Value::String(i.clone()));
                    }
                    let json = Value::Object(val);
                    table.insert(
                        json.get("id").and_then(|x| x.as_str()).unwrap(),
                        serde_json::to_string(&json)?.as_str(),
                    )?;
                    inserted += 1;
                }
            }
        }
    }

    write_txn.commit()?;
    Ok((inserted, updated))
}

fn find_latest_export(export_dir: &Path) -> Option<PathBuf> {
    if !export_dir.exists() {
        return None;
    }
    let mut entries: Vec<_> = fs::read_dir(export_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            if let Some(n) = e.file_name().to_str() {
                n.starts_with("applied_") && n.ends_with(".csv")
            } else {
                false
            }
        })
        .collect();
    // Sort by file name (timestamp encoded)
    entries.sort_by_key(|e| e.file_name());
    entries.pop().map(|e| e.path())
}

fn read_file_to_string(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?)
}

fn main() -> Result<()> {
    // Parse args
    let mut args = env::args().skip(1);
    let mut home_db: Option<PathBuf> = None;
    let mut dest_db: Option<PathBuf> = None;
    let mut export_dir: Option<PathBuf> = None;
    let mut yes = false;
    let mut dry_run = false;

    while let Some(a) = args.next() {
        match a.as_str() {
            "--home" => {
                if let Some(p) = args.next() {
                    home_db = Some(PathBuf::from(p));
                } else {
                    usage_and_exit();
                }
            }
            "--dest" => {
                if let Some(p) = args.next() {
                    dest_db = Some(PathBuf::from(p));
                } else {
                    usage_and_exit();
                }
            }
            "--export-dir" => {
                if let Some(p) = args.next() {
                    export_dir = Some(PathBuf::from(p));
                } else {
                    usage_and_exit();
                }
            }
            "-y" | "--yes" => yes = true,
            "--dry-run" => dry_run = true,
            "-h" | "--help" => usage_and_exit(),
            _ => {
                eprintln!("Unknown argument");
                usage_and_exit();
            }
        }
    }

    let home_db = home_db.unwrap_or_else(default_home_db);
    let dest_db = dest_db.unwrap_or_else(default_dest_db);
    let export_dir = export_dir.unwrap_or_else(|| default_export_dir(&dest_db));

    println!("Home DB: {}", home_db.display());
    println!("Destination DB: {}", dest_db.display());
    println!("Export dir: {}", export_dir.display());

    if !home_db.exists() {
        return Err(anyhow!(
            "Home DB not found at {}. Aborting.",
            home_db.display()
        ));
    }

    // Read applied records from home DB
    let applied = read_applied_from_sqlite(&home_db)?;
    if applied.is_empty() {
        println!("No applied entries found in home DB.");
    } else {
        println!("Found {} applied entries in home DB.", applied.len());
    }

    if dry_run {
        println!("Dry-run enabled: no changes will be made. Exiting.");
        return Ok(());
    }

    if !yes {
        println!("Proceed to merge these entries into destination DB? (y/N)");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let ok = matches!(line.trim().to_lowercase().as_str(), "y" | "yes");
        if !ok {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    // Backup dest DB
    if dest_db.exists() {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let backup = dest_db.with_file_name(format!(
            "{}.mergepre.bak.{}",
            dest_db.file_name().unwrap().to_string_lossy(),
            ts
        ));
        fs::copy(&dest_db, &backup)
            .with_context(|| format!("Failed to backup {}", dest_db.display()))?;
        println!("Created backup of destination DB at {}", backup.display());
    }

    // Merge
    let (inserted, updated) = merge_into_redb(&dest_db, &applied)?;
    println!("Merge completed: inserted={} updated={}", inserted, updated);

    // Export final applied set
    fs::create_dir_all(&export_dir)
        .with_context(|| format!("Failed to create export dir {}", export_dir.display()))?;
    let final_records = load_all_from_redb(&dest_db)?;
    let rows = canonical_rows_for_export(&final_records);
    let csv_content_rows: Vec<_> = rows
        .iter()
        .map(|t| {
            format!(
                "{},{},{},{},{},{}",
                quote_csv_field(&t.0),
                quote_csv_field(&t.1),
                quote_csv_field(&t.2),
                quote_csv_field(&t.3),
                quote_csv_field(&t.4),
                quote_csv_field(&t.5)
            )
        })
        .collect();
    let csv_data = {
        let mut s = String::new();
        s.push_str("id,headline,employer_name,city,publication_date,applied_at\n");
        for r in &csv_content_rows {
            s.push_str(r);
            s.push('\n');
        }
        s
    };

    // Check latest existing export to avoid duplicates
    let latest = find_latest_export(&export_dir);
    if let Some(latest_path) = latest {
        let existing = read_file_to_string(&latest_path).unwrap_or_default();
        if existing == csv_data {
            println!(
                "No change in applied list compared to {} -- not writing new export.",
                latest_path.display()
            );
            println!("Final export unchanged: {}", latest_path.display());
            return Ok(());
        }
    }

    // Write new export
    let fname = timestamped_name("applied");
    let out = export_dir.join(fname);
    fs::write(&out, &csv_data)?;
    println!("Wrote export CSV: {}", out.display());

    Ok(())
}
