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
    // 0. Single-instance IPC check: If another instance of Eclipse is already running,
    // tell it to show itself and bring its window to the front, then exit this process immediately.
    if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:49281") {
        use std::io::Write;
        let _ = stream.write_all(b"SHOW\n");
        let _ = stream.flush();
        #[cfg(windows)]
        ui::tray::show_window_native();
        return Ok(());
    }

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

