/*!
Library root for Jobseeker helper modules.

This crate exposes a small, well-scoped public API that is used by
the binaries in this workspace:

- `data`:
    Helpers to determine a sensible per-user database path (XDG / AppData
    style), create the directory if needed, and perform automatic migration
    / relocation of any `jobseeker.db` found in the working directory.

The top-level re-exports make it convenient for the GUI's startup code to
call into these helpers without caring about internal module layout.
*/

pub mod data;

// Re-export the most important functions and types so `main.rs` (and tests)
// can use `jobseeker::{...}` directly.
pub use data::{default_db_path, prepare_user_db};
