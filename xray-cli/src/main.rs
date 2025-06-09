mod app;
mod config;
mod xray;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Xray GUI",
        options,
        Box::new(|cc| Ok(Box::new(app::XrayGui::new(cc)))),
    )
    .expect("Failed to start app");
}
