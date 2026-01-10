use anyhow::{Context, Result};
use std::path::PathBuf;
use std::{env, fs};

/// Small helper binary to run the `prepare_user_db()` routine without launching the UI.
/// Usage:
///   cargo run --bin prepare_db
/// Options:
///   -v, --verbose    Print extra diagnostics about backups found in the working dir.
fn main() -> Result<()> {
    let verbose = env::args().any(|a| a == "-v" || a == "--verbose");

    println!(
        "Förbereder per-användar-databas (flyttar/migrerar lokala jobseeker.db om nödvändigt) ..."
    );

    match jobseeker::prepare_user_db() {
        Ok(path) => {
            println!("Databas är nu klar: {}", path.display());

            // Look for backups in the current working directory (saved by migration)
            let mut backups = Vec::new();
            if let Ok(entries) = fs::read_dir(".") {
                for e in entries {
                    if let Ok(entry) = e {
                        if let Ok(name) = entry.file_name().into_string() {
                            if name.starts_with("jobseeker.db.sqlite.bak")
                                || name.starts_with("jobseeker.db.bak.")
                                || name.starts_with("jobseeker.db.backup")
                            {
                                backups.push(name);
                            }
                        }
                    }
                }
            }

            if !backups.is_empty() {
                println!("Hittade backupfiler i arbetskatalogen:");
                for b in backups {
                    println!("  - {}", b);
                }
            } else if verbose {
                println!("Ingen backup hittades i arbetskatalogen.");
            }

            println!(
                "Klart. Starta gärna appen för att verifiera att allt fungerar som förväntat."
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Misslyckades med att förbereda databasen: {}", e);
            eprintln!(
                "Kontrollera att inga Jobseeker-processer körs och att du har skrivbehörighet."
            );
            std::process::exit(1);
        }
    }
}
