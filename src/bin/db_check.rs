// Small diagnostic tool to try opening the Jobseeker redb database and print helpful
// error information. Intended for local debugging only.
// Usage:
//   cargo run --bin db_check [path/to/jobseeker.db] [-v|--verbose]
//
// Examples:
//   cargo run --bin db_check                      # checks ./jobseeker.db
//   cargo run --bin db_check -v                   # verbose output (prints first bytes)
//   cargo run --bin db_check /tmp/mydb.db --verbose
//
// This program does not attempt to modify the DB (only opens it and starts a read txn).
use redb::Database;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process;

fn print_error_chain(mut e: &(dyn Error)) {
    eprintln!("Error: {}", e);
    while let Some(source) = e.source() {
        eprintln!("Caused by: {}", source);
        e = source;
    }
}

fn main() {
    // Simple arg parsing: first non-flag argument is the db path, flags: -v/--verbose
    let args: Vec<String> = env::args().skip(1).collect();
    let mut db_path = String::from("jobseeker.db");
    let mut verbose = false;

    for a in &args {
        match a.as_str() {
            "-v" | "--verbose" => verbose = true,
            other => db_path = other.to_string(),
        }
    }

    eprintln!("DB check: path = {}", db_path);

    // Basic file info (if exists)
    match fs::metadata(&db_path) {
        Ok(md) => {
            eprintln!(
                "Path exists. Size: {} bytes. Is file: {}. Permissions: {:?}",
                md.len(),
                Path::new(&db_path).is_file(),
                md.permissions()
            );

            if verbose {
                match File::open(&db_path) {
                    Ok(mut f) => {
                        let mut buf = [0u8; 64];
                        match f.read(&mut buf) {
                            Ok(n) if n > 0 => {
                                eprintln!("File head ({} bytes): {:02X?}", n, &buf[..n]);
                                // Also print as utf8 (best-effort) for readability
                                if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                                    eprintln!("File head as utf8 (best-effort): {}", s);
                                }
                            }
                            Ok(_) => eprintln!("File is empty"),
                            Err(err) => eprintln!("Failed to read file head: {}", err),
                        }
                    }
                    Err(err) => {
                        eprintln!("Cannot open file for inspection: {}", err);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Could not stat '{}': {}", db_path, err);
            eprintln!("If the file doesn't exist, Database::create will try to create it.");
        }
    }

    eprintln!("Attempting to create/open redb database...");
    match Database::create(&db_path) {
        Ok(db) => {
            eprintln!("Database opened/created successfully: {}", db_path);
            // Try starting a read transaction to verify basic DB functionality (non-destructive)
            match db.begin_read() {
                Ok(_) => {
                    eprintln!("Successfully started a read transaction.");
                    println!("OK - database is readable and seems fine");
                    process::exit(0);
                }
                Err(err) => {
                    eprintln!("Opened DB but failed to start read transaction: {}", err);
                    if let Some(src) = err.source() {
                        print_error_chain(src);
                    }
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to create/open database: {}", err);
            // Print full cause chain
            if let Some(src) = err.source() {
                print_error_chain(src);
            }

            eprintln!();
            eprintln!("Common causes and checks:");
            eprintln!(
                "- Another process has an exclusive lock on the file (check: `lsof {}` or `fuser -v {}`)",
                db_path, db_path
            );
            eprintln!(
                "- Permission denied on the DB file or on the parent directory (check ownership & mode)"
            );
            eprintln!("- Filesystem is read-only or disk is full");
            eprintln!(
                "- The file might be corrupted or is not a redb database (running with -v prints the file head)"
            );
            eprintln!();
            eprintln!("Suggested steps:");
            eprintln!("- Ensure no other Jobseeker instances are running and try again");
            eprintln!(
                "- Backup current DB: `mv {0} {0}.bak` and start the app so it creates a fresh DB",
                db_path
            );
            eprintln!("- If unsure, run this tool with -v to inspect the file head");
            process::exit(1);
        }
    }
}
