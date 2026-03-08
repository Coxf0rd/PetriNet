use std::hash::Hash;

use eframe::egui;

#[derive(Debug, Clone)]
pub(crate) struct PropertySectionConfig {
    pub id: egui::Id,
    pub default_open: bool,
    pub top_spacing: f32,
}

impl PropertySectionConfig {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            default_open: true,
            top_spacing: 0.0,
        }
    }

    pub fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
        self
    }

    pub fn top_spacing(mut self, value: f32) -> Self {
        self.top_spacing = value;
        self
    }
}

pub(crate) fn show_property_section<R>(
    ui: &mut egui::Ui,
    title: impl Into<egui::WidgetText>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let title = title.into();

    let frame = egui::Frame::group(ui.style());

    frame
        .show(ui, |ui: &mut egui::Ui| {
            ui.set_width(ui.available_width());
            ui.set_max_width(ui.available_width());

            ui.vertical(|ui: &mut egui::Ui| {
                ui.set_width(ui.available_width());
                ui.set_max_width(ui.available_width());

                ui.label(title.strong());
                ui.add_space(4.0);

                add_contents(ui)
            })
            .inner
        })
        .inner
}

pub(crate) fn show_collapsible_property_section<R>(
    ui: &mut egui::Ui,
    title: impl Into<egui::WidgetText>,
    config: PropertySectionConfig,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    if config.top_spacing > 0.0 {
        ui.add_space(config.top_spacing);
    }

    let title = title.into();
    let frame = egui::Frame::group(ui.style());

    frame
        .show(ui, |ui: &mut egui::Ui| {
            ui.set_width(ui.available_width());
            ui.set_max_width(ui.available_width());

            egui::CollapsingHeader::new(title)
                .id_source(config.id)
                .default_open(config.default_open)
                .show(ui, |ui: &mut egui::Ui| {
                    ui.set_width(ui.available_width());
                    ui.set_max_width(ui.available_width());

                    add_contents(ui)
                })
                .body_returned
        })
        .inner
}