//! Helpers for determining a per-user database location and for migrating any
//! legacy `jobseeker.db` that might live next to the executable/repo into the
//! per-user storage location.
//!
//! Exported helpers:
//! - `default_db_path()` -> Option<PathBuf>
//! - `prepare_user_db()` -> Result<PathBuf>
//!
//! `prepare_user_db()` will:
//! 1. Decide the per-user path (respecting JOBSEEKER_DB_PATH if set)
//! 2. If a database already exists in that destination, return it
//! 3. If a `./jobseeker.db` exists next to the executable, try to:
//!    - detect its format (SQLite or Redb)
//!    - convert SQLite -> Redb (using the migration library) or move Redb directly
//!    - backup the original `jobseeker.db` (to `jobseeker.db.sqlite.bak.<ts>`)
//!    - return the new per-user DB path
//!
//! This keeps personal data out of the repository and gives a single canonical
//! per-user store.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use redb::Database;
use std::env;
use std::fs;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Return the *preferred* per-user database path, if we can determine one.
///
/// - If `JOBSEEKER_DB_PATH` environment variable is set, that's returned (as PathBuf).
/// - Otherwise we use platform-standard locations via `directories::ProjectDirs`:
///   on Linux e.g. `~/.local/share/Jobseeker/jobseeker.db`.
pub fn default_db_path() -> Option<PathBuf> {
    if let Ok(p) = env::var("JOBSEEKER_DB_PATH") {
        return Some(PathBuf::from(p));
    }

    if let Some(pd) = ProjectDirs::from("se", "gnaw-software", "Jobseeker") {
        // `data_local_dir` is a good place for user-specific application data.
        let path = pd.data_local_dir().join("jobseeker.db");
        return Some(path);
    }

    None
}

/// Ensure the application database lives in the per-user location and return its path.
///
/// Behaviour:
/// - If `JOBSEEKER_DB_PATH` is set, it is used and its parent directory is created.
/// - If a database already exists at the destination, that path is returned.
/// - If a local `./jobseeker.db` exists:
///     * If it's SQLite -> perform a migration to Redb at destination (backup original).
///     * If it's already a Redb DB -> move it into destination (backup if desired).
/// - If there is no local DB and no destination DB yet, parent directories are created
///   and the returned path will be used by the app to create a fresh database.
pub fn prepare_user_db() -> Result<PathBuf> {
    // 1) Resolve destination path
    let dest = if let Ok(p) = env::var("JOBSEEKER_DB_PATH") {
        PathBuf::from(p)
    } else if let Some(dp) = default_db_path() {
        dp
    } else {
        // Fallback: use local file (worst-case)
        PathBuf::from("jobseeker.db")
    };

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create DB directory {}", parent.display()))?;
    }

    // If destination already exists, we're done
    if dest.exists() {
        info!("Using existing database at {}", dest.display());
        return Ok(dest);
    }

    // 2) Check for local database file `./jobseeker.db`
    let local = Path::new("jobseeker.db");
    if !local.exists() {
        // Nothing to migrate; return the destination (which does not exist yet)
        info!(
            "No local database found and no destination DB yet; will use {} on first run",
            dest.display()
        );
        return Ok(dest);
    }

    // 3) Local DB exists - do NOT perform automatic SQLite -> Redb migration.
    // If the local file is a Redb DB, move it into place. If it is a SQLite DB,
    // back the file up and continue without migrating (user-requested policy).
    // Detect Redb by attempting to open it with `Database::create`.
    if Database::create(local).is_ok() {
        info!(
            "Found Redb DB at {} — moving to {}",
            local.display(),
            dest.display()
        );

        // Move (rename) local redb file into per-user destination. If rename
        // fails (cross-device), fall back to copy+remove.
        match fs::rename(local, &dest) {
            Ok(()) => {
                info!("Moved DB into place: {}", dest.display());
                Ok(dest)
            }
            Err(e) => {
                warn!("Rename failed ({}); attempting copy + remove fallback", e);
                fs::copy(local, &dest)
                    .with_context(|| format!("failed to copy local DB to {}", dest.display()))?;
                fs::remove_file(local).with_context(|| {
                    format!("failed to remove original local DB {}", local.display())
                })?;
                info!("Copied DB into place: {}", dest.display());
                Ok(dest)
            }
        }
    } else {
        // Likely an old SQLite DB — do not attempt automatic migration.
        // Move it to a timestamped backup to avoid accidental data loss.
        let backup = backup_path(local)?;
        fs::rename(local, &backup).with_context(|| {
            format!(
                "Found SQLite DB at {}; moved to backup {}. Automatic migration is disabled. Run migration manually if needed.",
                local.display(),
                backup.display()
            )
        })?;
        info!("Backed up local SQLite DB to {}", backup.display());
        Ok(dest)
    }
}

/// Helper: create a timestamped backup path beside `path`, using a Unix timestamp.
fn backup_path(path: &Path) -> Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup = format!("{}.sqlite.bak.{}", path.display(), ts);
    Ok(PathBuf::from(backup))
}

// Migration helpers removed.
// Automatic SQLite -> Redb migration has been intentionally removed from the codebase.
// If you need to migrate an old SQLite file to the Redb format, run a migration tool
// from a previous commit or use an external script. The application will not perform
// automatic migration anymore to avoid accidental data loss.
