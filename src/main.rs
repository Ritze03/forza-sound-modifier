#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod i18n;
mod data;
mod engine;
mod tab_car;
mod tab_class;
mod tab_misc;
mod tab_profiles;
mod tab_reverb;
mod tab_setup;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Forza Horizon 6 Sound Modifier")
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Forza Horizon 6 Sound Modifier",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
