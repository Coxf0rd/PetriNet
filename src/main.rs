#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use image::ImageFormat;
use petri_net_legacy_editor::ui::app::PetriApp;

fn app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/petrinet.ico");
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Ico).ok()?;
    let rgba = image.to_rgba8();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width: image.width(),
        height: image.height(),
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]);
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "PetriNet",
        native_options,
        Box::new(|cc| Ok(Box::new(PetriApp::new(cc)))),
    )
}
