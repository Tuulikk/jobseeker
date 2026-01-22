use std::path::PathBuf;
use redb::{Database, TableDefinition};

const SETTINGS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("settings");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Jobseeker Settings Reset Tool");
    println!("==============================");
    println!();

    // Find DB path
    let db_path = if cfg!(target_os = "android") {
        PathBuf::from("/data/data/com.gnawsoftware.jobseeker/files/jobseeker.redb")
    } else {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "GnawSoftware", "Jobseeker") {
            proj_dirs.data_dir().join("jobseeker.redb")
        } else {
            PathBuf::from("jobseeker.redb")
        }
    };

    println!("DB path: {}", db_path.display());

    if !db_path.exists() {
        println!("Error: DB file not found!");
        return Ok(());
    }

    let db = Database::open(&db_path)?;

    // Read current settings
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(SETTINGS_TABLE)?;

    if let Some(settings_json) = table.get("current")? {
        println!("\nCurrent settings in DB:");
        println!("{}", settings_json.value());
    } else {
        println!("\nNo settings found in DB.");
    }
    drop(read_txn);

    // Prompt to reset
    print!("\nReset to default settings? [y/N]: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.eq_ignore_ascii_case("y") {
        // Default settings (from models.rs)
        let default_settings = r#"{
            "keywords": "it",
            "blacklist_keywords": "barnvakt, körkort, barnflicka, nanny, myNanny, undersköterska, parkarbetare",
            "locations_p1": "1283, 1277, 1260, 1292, 1284, 1276, 1231, 1282, 1261",
            "locations_p2": "1280, 1281",
            "locations_p3": "",
            "my_profile": "Jag är en serviceinriktad person med erfarenhet inom IT-support och kundservice.",
            "ollama_url": "http://localhost:11434/v1",
            "app_min_count": 6,
            "app_goal_count": 12,
            "show_motivation": true
        }"#;

        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS_TABLE)?;
            table.insert("current", default_settings)?;
        }
        write_txn.commit()?;

        println!("\nSettings reset to default!");
        println!("\nRestart Jobseeker to apply changes.");
    } else {
        println!("\nAborted. No changes made.");
    }

    Ok(())
}
