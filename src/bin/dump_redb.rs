use anyhow::{Context, Result};
use chrono::Datelike;
use redb::{Database, ReadableTable, TableDefinition};
use serde_json::Value;
use std::env;
use std::path::PathBuf;

/// Helper binary to inspect and dump `job_ads` entries from the Redb database.
///
/// Usage:
///   cargo run --bin dump_redb -- [--db /path/to/jobseeker.db] [--limit N] [--dec-only] [--id ID] [--json]
///
/// - `--db <path>` : use explicit database path (otherwise uses per-user default if available, else ./jobseeker.db)
/// - `--limit N`   : limit number of rows printed
/// - `--dec-only`  : only print entries whose `internal_created_at` month == 12
/// - `--id ID`     : show only the entry with the given ID
/// - `--json`      : print raw JSON value for each entry
///
/// Note: run this when the GUI is not running (concurrent writers lock the DB).
const JOB_ADS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("job_ads");

fn usage_and_exit() -> ! {
    eprintln!(
        "Usage: dump_redb [--db <path>] [--limit N] [--dec-only] [--id ID] [--json]\n\
         Example: cargo run --bin dump_redb -- --limit 20 --dec-only"
    );
    std::process::exit(1);
}

fn parse_args() -> (Option<PathBuf>, Option<usize>, bool, Option<String>, bool) {
    let mut args = env::args().skip(1);
    let mut db_path: Option<PathBuf> = None;
    let mut limit: Option<usize> = None;
    let mut dec_only = false;
    let mut id_filter: Option<String> = None;
    let mut json_out = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db" => {
                if let Some(p) = args.next() {
                    db_path = Some(PathBuf::from(p));
                } else {
                    usage_and_exit();
                }
            }
            "--limit" => {
                if let Some(n) = args.next() {
                    limit = n.parse::<usize>().ok();
                } else {
                    usage_and_exit();
                }
            }
            "--dec-only" => dec_only = true,
            "--id" => {
                if let Some(id) = args.next() {
                    id_filter = Some(id);
                } else {
                    usage_and_exit();
                }
            }
            "--json" => json_out = true,
            "-h" | "--help" => usage_and_exit(),
            other => {
                eprintln!("Unknown argument: {}", other);
                usage_and_exit();
            }
        }
    }

    (db_path, limit, dec_only, id_filter, json_out)
}

fn pick_db_path(cli: Option<PathBuf>) -> PathBuf {
    if let Some(p) = cli {
        return p;
    }
    // Use library helper if available
    if let Some(p) = jobseeker::default_db_path() {
        return p;
    }
    // fallback to local file
    PathBuf::from("jobseeker.db")
}

fn pretty_line(id: &str, json: &Value) -> String {
    let headline = json
        .get("headline")
        .and_then(Value::as_str)
        .unwrap_or("<no headline>");
    let status = json
        .get("status")
        .and_then(Value::as_i64)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<no status>".to_string());
    let bookmarked_at = json
        .get("bookmarked_at")
        .and_then(Value::as_str)
        .unwrap_or("");
    let applied_at = json.get("applied_at").and_then(Value::as_str).unwrap_or("");
    let internal_created_at = json
        .get("internal_created_at")
        .and_then(Value::as_str)
        .unwrap_or("");

    format!(
        "ID={} | status={} | headline=\"{}\" | bookmarked_at={} | applied_at={} | internal_created_at={}",
        id,
        status,
        headline.replace('\n', " "),
        bookmarked_at,
        applied_at,
        internal_created_at
    )
}

fn internal_month_is_dec(json: &Value) -> bool {
    if let Some(s) = json.get("internal_created_at").and_then(Value::as_str) {
        if s.len() >= 7 {
            // Try parse via substring YYYY-MM
            if let Ok(month) = s[5..7].parse::<u32>() {
                return month == 12;
            }
        }
        // Fallback: try rfc3339 parsing
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return dt.month() == 12;
        }
    }
    false
}

fn main() -> Result<()> {
    let (cli_db, limit, dec_only, id_filter, json_out) = parse_args();

    let db_path = pick_db_path(cli_db);
    println!("Opening DB at: {}", db_path.display());

    // Open DB (create opens existing too). If DB is open by another process, this will error.
    let db = Database::create(&db_path).with_context(|| {
        format!(
            "Couldn't open database at {}. Is the app running? Close it and retry.",
            db_path.display()
        )
    })?;

    let read_txn = db
        .begin_read()
        .context("Failed to start read transaction")?;
    let table = read_txn
        .open_table(JOB_ADS_TABLE)
        .context("Failed to open job_ads table")?;

    let mut seen = 0usize;
    let mut printed = 0usize;
    let mut dec_count = 0usize;

    for item_res in table.iter()? {
        let (key, value) = item_res?;
        let id = key.value();
        let raw = value.value();
        let json: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON for id {}: {}", id, e);
                continue;
            }
        };

        seen += 1;

        // Apply id filter
        if let Some(ref wanted) = id_filter
            && id != wanted.as_str()
        {
            continue;
        }

        // December filter
        if dec_only && !internal_month_is_dec(&json) {
            continue;
        }
        if internal_month_is_dec(&json) {
            dec_count += 1;
        }

        // Output
        if json_out {
            println!(
                "ID={} JSON={}",
                id,
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| raw.to_string())
            );
        } else {
            println!("{}", pretty_line(id, &json));
        }

        printed += 1;
        if let Some(lim) = limit
            && printed >= lim
        {
            break;
        }
    }

    println!(
        "\nScanned: {} ads; printed: {}; December matches: {}",
        seen, printed, dec_count
    );

    Ok(())
}
