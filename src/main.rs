use slint::ComponentHandle;

fn main() -> slint::Result<()> {
    let ui = ui::App::new()?;
    ui.run()
}
