use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_arc_props_window(
        &mut self,
        ctx: &egui::Context,
        arc_id: u64,
        title: String,
    ) -> bool {
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        #[derive(Clone, Copy)]
        enum SelectedArc {
            Regular(usize),
            Inhibitor(usize),
        }

        let variant = if let Some(idx) = self.arc_idx_by_id(arc_id) {
            SelectedArc::Regular(idx)
        } else if let Some(idx) = self.inhibitor_arc_idx_by_id(arc_id) {
            SelectedArc::Inhibitor(idx)
        } else {
            return false;
        };

        let mut weight = match variant {
            SelectedArc::Regular(idx) => self.net.arcs[idx].weight,
            SelectedArc::Inhibitor(_) => 1,
        };
        let mut threshold = match variant {
            SelectedArc::Inhibitor(idx) => self.net.inhibitor_arcs[idx].threshold,
            SelectedArc::Regular(_) => 1,
        };
        let mut color = match variant {
            SelectedArc::Regular(idx) => self.net.arcs[idx].color,
            SelectedArc::Inhibitor(idx) => self.net.inhibitor_arcs[idx].color,
        };
        let mut show_weight = match variant {
            SelectedArc::Regular(idx) => self.net.arcs[idx].show_weight,
            SelectedArc::Inhibitor(idx) => self.net.inhibitor_arcs[idx].show_weight,
        };
        let mut is_inhibitor = matches!(variant, SelectedArc::Inhibitor(_));
        let can_be_inhibitor = match variant {
            SelectedArc::Regular(idx) => {
                Self::arc_place_transition_pair(self.net.arcs[idx].from, self.net.arcs[idx].to)
                    .is_some()
            }
            SelectedArc::Inhibitor(_) => true,
        };
        if !can_be_inhibitor && is_inhibitor {
            is_inhibitor = false;
        }

        let color_combo = |ui: &mut egui::Ui, value: &mut NodeColor| {
            egui::ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(Self::node_color_text(*value, is_ru))
                .show_ui(ui, |ui: &mut egui::Ui| {
                    ui.selectable_value(
                        value,
                        NodeColor::Default,
                        Self::node_color_text(NodeColor::Default, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Blue,
                        Self::node_color_text(NodeColor::Blue, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Red,
                        Self::node_color_text(NodeColor::Red, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Green,
                        Self::node_color_text(NodeColor::Green, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Yellow,
                        Self::node_color_text(NodeColor::Yellow, is_ru),
                    );
                });
        };

        let mut open = true;
        show_property_window(
            ctx,
            title,
            &mut open,
            PropertyWindowConfig::new("arc_props_window")
                .default_size(egui::vec2(420.0, 440.0))
                .min_size(egui::vec2(320.0, 320.0)),
            |ui: &mut egui::Ui| {
                let mut corrected_inputs = false;
                ui.label(format!("ID: A{}", arc_id));
                ui.separator();

                ui.add_enabled_ui(can_be_inhibitor, |ui: &mut egui::Ui| {
                    ui.checkbox(&mut is_inhibitor, t("Ингибиторная дуга", "Inhibitor arc"));
                });

                if matches!(variant, SelectedArc::Regular(_)) && !can_be_inhibitor {
                    ui.label(t(
                        "Ингибиторная дуга должна начинаться с позиции и заканчиваться на переходе",
                        "Inhibitor arcs must start at a position and end at a transition",
                    ));
                }

                corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                corrected_inputs |= sanitize_u32(&mut weight, 1, u32::MAX);

                let weight_label = t("Кратность (вес)", "Weight");
                let show_weight_label =
                    t("Показывать кратность (вес)", "Show multiplicity (weight)");

                if is_inhibitor {
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.label(t("Порог", "Threshold"));
                        if ui
                            .add(egui::DragValue::new(&mut threshold).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                        }
                    });
                } else {
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.label(weight_label);
                        if ui
                            .add(egui::DragValue::new(&mut weight).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut weight, 1, u32::MAX);
                        }
                    });
                }

                ui.checkbox(&mut show_weight, show_weight_label);

                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(t("Цвет", "Color"));
                    color_combo(ui, &mut color);
                });

                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
            },
        );

        let new_weight = weight.max(1);
        let new_threshold = threshold.max(1);
        let mut should_rebuild = false;

        match variant {
            SelectedArc::Regular(idx) => {
                if is_inhibitor {
                    if let Some((place_id, transition_id)) = Self::arc_place_transition_pair(
                        self.net.arcs[idx].from,
                        self.net.arcs[idx].to,
                    ) {
                        let arc = self.net.arcs.remove(idx);
                        self.net.inhibitor_arcs.push(crate::model::InhibitorArc {
                            id: arc.id,
                            place_id,
                            transition_id,
                            threshold: new_threshold,
                            color,
                            visible: arc.visible,
                            show_weight,
                        });
                        self.canvas.selected_arc = Some(arc.id);
                        if !self.canvas.selected_arcs.contains(&arc.id) {
                            self.canvas.selected_arcs.push(arc.id);
                        }
                        should_rebuild = true;
                    }
                } else {
                    let arc = &mut self.net.arcs[idx];
                    if arc.weight != new_weight {
                        should_rebuild = true;
                    }
                    arc.weight = new_weight;
                    arc.color = color;
                    arc.show_weight = show_weight;
                }
            }
            SelectedArc::Inhibitor(idx) => {
                if !is_inhibitor {
                    let inh = self.net.inhibitor_arcs.remove(idx);
                    self.net.arcs.push(crate::model::Arc {
                        id: inh.id,
                        from: NodeRef::Place(inh.place_id),
                        to: NodeRef::Transition(inh.transition_id),
                        weight: new_weight,
                        color,
                        visible: inh.visible,
                        show_weight,
                    });
                    self.canvas.selected_arc = Some(inh.id);
                    if !self.canvas.selected_arcs.contains(&inh.id) {
                        self.canvas.selected_arcs.push(inh.id);
                    }
                    should_rebuild = true;
                } else {
                    let inh = &mut self.net.inhibitor_arcs[idx];
                    if inh.threshold != new_threshold {
                        should_rebuild = true;
                    }
                    inh.threshold = new_threshold;
                    inh.color = color;
                    inh.show_weight = show_weight;
                }
            }
        }

        if should_rebuild {
            self.net.rebuild_matrices_from_arcs();
        }

        open
    }
}