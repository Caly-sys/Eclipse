#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod audio;
pub mod config;
pub mod library;
pub mod lyrics;
pub mod streams;
pub mod ui;

use eframe::egui::vec2;
use std::sync::Arc;
use ui::EclipseApp;

fn main() -> eframe::Result<()> {
    // Load window icon from assets/eclipse.png
    let icon = load_app_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Eclipse — music, in the dark")
        .with_inner_size(vec2(1150.0, 720.0))
        .with_min_inner_size(vec2(950.0, 600.0))
        .with_decorations(false) // Frameless dark glass window
        .with_transparent(true);

    if let Some(i) = icon {
        viewport = viewport.with_icon(Arc::new(i));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "Eclipse",
        native_options,
        Box::new(|cc| Ok(Box::new(EclipseApp::new(cc)))),
    )
}

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/eclipse.png");
    if let Ok(image) = image::load_from_memory(icon_bytes) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        return Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        });
    }
    None
}

