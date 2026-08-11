#![windows_subsystem = "windows"]

mod app;
mod sys;
mod ui;

use app::SystemMonitorApp;
use eframe::NativeOptions;
use egui::Vec2;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BTOP GUI - 系統監視器")
            .with_inner_size(Vec2::new(1280.0, 850.0))
            .with_min_inner_size(Vec2::new(900.0, 600.0))
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "BTOP GUI - 系統監視器",
        native_options,
        Box::new(|cc| Ok(Box::new(SystemMonitorApp::new(cc)))),
    )
}
