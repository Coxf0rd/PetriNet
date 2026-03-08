use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_text_props_window(
        &mut self,
        ctx: &egui::Context,
        text_id: u64,
        title: String,
    ) -> bool {
        let Some(text_idx) = self.text_idx_by_id(text_id) else {
            return false;
        };
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        show_property_window(
            ctx,
            title,
            &mut open,
            PropertyWindowConfig::new("text_props_window")
                .default_size(egui::vec2(460.0, 360.0))
                .min_size(egui::vec2(360.0, 260.0)),
            |ui: &mut egui::Ui| {
                let text = &mut self.text_blocks[text_idx];

                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(t("Шрифт", "Font"));
                    egui::ComboBox::from_id_source("text_font_combo")
                        .selected_text(text.font_name.clone())
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            for name in Self::text_font_candidates() {
                                ui.selectable_value(
                                    &mut text.font_name,
                                    (*name).to_string(),
                                    *name,
                                );
                            }
                        });

                    ui.label(t("Размер", "Size"));
                    ui.add(egui::DragValue::new(&mut text.font_size).range(6.0..=72.0));

                    ui.label(t("Цвет", "Color"));
                    egui::ComboBox::from_id_source("text_color_combo")
                        .selected_text(Self::text_color_text(text.color, is_ru))
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Default,
                                Self::text_color_text(NodeColor::Default, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Blue,
                                Self::text_color_text(NodeColor::Blue, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Red,
                                Self::text_color_text(NodeColor::Red, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Green,
                                Self::text_color_text(NodeColor::Green, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Yellow,
                                Self::text_color_text(NodeColor::Yellow, is_ru),
                            );
                        });
                });

                ui.separator();
                ui.add(
                    egui::TextEdit::multiline(&mut text.text)
                        .desired_rows(6)
                        .desired_width(f32::INFINITY),
                );
            },
        );
        open
    }
}
