#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use petri_net_legacy_editor::ui::app::PetriApp;

fn app_icon() -> egui::IconData {
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let border = x < 2 || y < 2 || x >= size - 2 || y >= size - 2;
            let in_circle = {
                let center_x = size as f32 / 2.0;
                let center_y = size as f32 / 2.0;
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                dx * dx + dy * dy <= (size as f32 * 0.42).powi(2)
            };
            let (r, g, b, a) = if border {
                (18, 18, 18, 255)
            } else if in_circle {
                (52, 120, 220, 255)
            } else {
                (245, 245, 245, 255)
            };
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = a;
        }
    }

    let draw_bar = |image: &mut [u8], x0: u32, y0: u32, w: u32, h: u32| {
        for y in y0..(y0 + h).min(size) {
            for x in x0..(x0 + w).min(size) {
                let idx = ((y * size + x) * 4) as usize;
                image[idx] = 255;
                image[idx + 1] = 255;
                image[idx + 2] = 255;
                image[idx + 3] = 255;
            }
        }
    };

    draw_bar(&mut rgba, 22, 18, 7, 30);
    draw_bar(&mut rgba, 22, 18, 20, 7);
    draw_bar(&mut rgba, 22, 30, 18, 7);

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_icon(Arc::new(app_icon())),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "PetriNet",
        native_options,
        Box::new(|cc| Ok(Box::new(PetriApp::new(cc)))),
    )
}
