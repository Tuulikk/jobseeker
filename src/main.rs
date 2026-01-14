mod models;
mod api;
mod db;
mod ai;

use slint::ComponentHandle;

#[no_mangle]
#[cfg(target_os = "android")]
pub extern "Rust" fn android_main(app: slint::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(tracing::log::LevelFilter::Info),
    );
    slint::android::init(app).expect("Failed to initialize Slint on Android");
    
    let ui = ui::App::new().expect("Failed to create Slint UI");
    ui.run().expect("Failed to run Slint UI");
}

#[cfg(not(target_os = "android"))]
fn main() -> slint::Result<()> {
    let ui = ui::App::new()?;
    ui.run()
}
