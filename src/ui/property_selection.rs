use std::hash::Hash;

use eframe::egui;
// Import scroll utilities for unified scroll behaviour.
use crate::ui::scroll_utils;

/// Configuration for property sections within the UI.
#[derive(Debug, Clone)]
pub(crate) struct PropertySectionConfig {
    pub id: egui::Id,
    pub default_open: bool,
    pub top_spacing: f32,
}

impl PropertySectionConfig {
    /// Create a new configuration with the given identifier.
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            default_open: true,
            top_spacing: 0.0,
        }
    }

    /// Set whether the section is open by default.
    pub fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
        self
    }

    /// Set additional spacing above the section.
    pub fn top_spacing(mut self, value: f32) -> Self {
        self.top_spacing = value;
        self
    }
}

/// Show a non-collapsible property section.
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

/// Show a collapsible property section with a unified scroll behaviour.
///
/// The body of the section is wrapped in a vertical scroll area with
/// hidden scrollbars.  This allows long property sections to scroll
/// without displaying a visible scroll bar, matching the desired UI
/// behaviour for collapsible blocks.  The return value is `Some(R)`
/// when the section is open, or `None` when collapsed.
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
                    // Wrap the contents in a ScrollArea with hidden scrollbars using
                    // our scroll utilities.  The scroll area auto-shrinks both axes and
                    // scroll bars are hidden so that long property sections can scroll
                    // without displaying a visible bar.
                    scroll_utils::show_hidden_vertical_scroll(ui, config.id.with("collapsible_section"), ui.available_height(), |ui| {
                        // Ensure the contents can grow horizontally to fill the available width.
                        ui.set_min_width(0.0);
                        ui.set_max_width(ui.available_width());
                        add_contents(ui)
                    })
                })
                .body_returned
        })
        .inner
}