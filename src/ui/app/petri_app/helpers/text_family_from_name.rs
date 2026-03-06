use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_family_from_name(name: &str) -> egui::FontFamily {
        let lower = name.to_ascii_lowercase();
        if lower.contains("courier") || lower.contains("mono") {
            egui::FontFamily::Monospace
        } else {
            egui::FontFamily::Proportional
        }
    }
}
