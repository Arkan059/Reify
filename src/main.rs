#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod code_image;
mod fonts;
mod render_export;
mod render_screen;
mod scene;
mod spotify;
mod state;
mod theme;
mod worker;

use eframe::egui;
use std::sync::Arc;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("reify")
            .with_icon(Arc::new(load_icon())),
        ..Default::default()
    };
    eframe::run_native(
        "reify",
        options,
        Box::new(|creation_context| Ok(Box::new(app::App::new(creation_context)))),
    )
}

fn load_icon() -> egui::IconData {
    let bytes = include_bytes!("../icon.ico");
    match image::load_from_memory(bytes) {
        Ok(decoded) => {
            let rgba = decoded.to_rgba8();
            let width = rgba.width();
            let height = rgba.height();
            egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }
        }
        Err(_) => egui::IconData::default(),
    }
}
