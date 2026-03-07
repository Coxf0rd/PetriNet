use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_transition_props_window(
        &mut self,
        ctx: &egui::Context,
        transition_id: u64,
        title: String,
    ) -> bool {
        let Some(transition_idx) = self.transition_idx_by_id(transition_id) else {
            return false;
        };

        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(title)
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("transition_props_window"))
            .resizable(true)
            .default_size(egui::vec2(420.0, 520.0))
            .min_size(egui::vec2(320.0, 360.0))
            .open(&mut open)
            .show(ctx, |ui| {
                let mut corrected_inputs = false;
                ui.label(format!("ID: T{}", transition_id));
                ui.separator();
                let mut priority = self.net.tables.mpr[transition_idx];
                corrected_inputs |= sanitize_i32(&mut priority, -1_000_000, 1_000_000);
                ui.horizontal(|ui| {
                    ui.label(t("Приоритет", "Priority"));
                    if ui.add(egui::DragValue::new(&mut priority)).changed() {
                        corrected_inputs |= sanitize_i32(&mut priority, -1_000_000, 1_000_000);
                    }
                });
                self.net.tables.mpr[transition_idx] = priority;
                ui.label(t("Размер перехода", "Transition size"));
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.net.transitions[transition_idx].size,
                        VisualSize::Small,
                        t("Малый", "Small"),
                    );
                    ui.radio_value(
                        &mut self.net.transitions[transition_idx].size,
                        VisualSize::Medium,
                        t("Средний", "Medium"),
                    );
                    ui.radio_value(
                        &mut self.net.transitions[transition_idx].size,
                        VisualSize::Large,
                        t("Большой", "Large"),
                    );
                });

                egui::ComboBox::from_label(t("Положение метки", "Label position"))
                    .selected_text(Self::label_pos_text(
                        self.net.transitions[transition_idx].label_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Положение текста", "Text position"))
                    .selected_text(Self::label_pos_text(
                        self.net.transitions[transition_idx].text_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Цвет", "Color"))
                    .selected_text(Self::node_color_text(
                        self.net.transitions[transition_idx].color,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Default,
                            t("По умолчанию", "Default"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Blue,
                            t("Синий", "Blue"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Red,
                            t("Красный", "Red"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Green,
                            t("Зеленый", "Green"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Yellow,
                            t("Желтый", "Yellow"),
                        );
                    });

                ui.separator();
                ui.label(t("Название", "Name"));
                ui.text_edit_singleline(&mut self.net.transitions[transition_idx].name);
                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
            });
        open
    }
}
