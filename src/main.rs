#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod pdf;
mod stitch;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("StichUp - PDF Stitcher & Editor"),
        ..Default::default()
    };
    eframe::run_native(
        "StichUp",
        native_options,
        Box::new(|cc| Box::new(app::StichUpApp::new(cc))),
    )
}
mod export;
