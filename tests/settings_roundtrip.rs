use std::fs;
use chrono::Utc;

/// Integration test: Save settings (including locations_p1/2/3) and load them back.
///
/// This verifies that `Db::save_settings` and `Db::load_settings` preserve the
/// three priority location fields through a roundtrip.
#[tokio::test]
async fn settings_roundtrip() {
    // Build a unique temp path to avoid collisions between test runs
    let mut tmp = std::env::temp_dir();
    tmp.push(format!(
        "jobseeker_test_{}_{}.redb",
        std::process::id(),
        Utc::now().timestamp()
    ));
    let path_str = tmp
        .to_str()
        .expect("Temp path should be valid UTF-8")
        .to_string();

    // Ensure no leftover DB file present
    let _ = fs::remove_file(&tmp);

    // Initialize DB
    let db = Jobseeker::db::Db::new(&path_str)
        .await
        .expect("Failed to create/open test DB");

    // Prepare settings with all three priority locations populated
    let settings = Jobseeker::models::AppSettings {
        keywords: "it, support".to_string(),
        blacklist_keywords: "körkort, barnvakt".to_string(),
        locations_p1: "1283, 1277".to_string(),
        locations_p2: "1280, 1281".to_string(),
        locations_p3: "".to_string(),
        my_profile: "Testprofil".to_string(),
        ollama_url: "http://localhost:11434/v1".to_string(),
    };

    // Save and load back
    db.save_settings(&settings)
        .await
        .expect("Failed to save settings");

    let loaded = db
        .load_settings()
        .await
        .expect("Failed to load settings")
        .expect("No settings found after save");

    // Verify the three priority fields roundtrip correctly
    assert_eq!(loaded.locations_p1, settings.locations_p1);
    assert_eq!(loaded.locations_p2, settings.locations_p2);
    assert_eq!(loaded.locations_p3, settings.locations_p3);

    // Cleanup: drop DB and remove file
    drop(db);
    let _ = fs::remove_file(&tmp);
}
