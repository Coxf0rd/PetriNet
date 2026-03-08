use eframe::egui;
use std::hash::Hash;

#[derive(Clone)]
pub(crate) struct PropertySectionConfig {
    pub id: egui::Id,
    pub label: egui::WidgetText,
    pub default_open: bool,
    pub top_spacing: f32,
}

impl PropertySectionConfig {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            label: egui::WidgetText::from(""),
            default_open: true,
            top_spacing: 0.0,
        }
    }

    pub fn label(mut self, label: impl Into<egui::WidgetText>) -> Self {
        self.label = label.into();
        self
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

impl std::fmt::Debug for PropertySectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertySectionConfig")
            .field("id", &self.id)
            .field("default_open", &self.default_open)
            .field("top_spacing", &self.top_spacing)
            .finish()
    }
}

pub(crate) fn show_collapsible_property_section<R>(
    ui: &mut egui::Ui,
    config: PropertySectionConfig,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    if config.top_spacing > 0.0 {
        ui.add_space(config.top_spacing);
    }

    let frame = egui::Frame::group(ui.style());

    frame
        .show(ui, |ui: &mut egui::Ui| {
            ui.set_width(ui.available_width());
            ui.set_max_width(ui.available_width());

            egui::CollapsingHeader::new(config.label)
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
