use super::*;

impl PetriApp {
    pub(super) fn draw_tool_palette(&mut self, ctx: &egui::Context) {
        if self.tool == Tool::Run {
            self.tool = Tool::Edit;
        }

        let panel = egui::SidePanel::left("tools")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Инструменты");
                ui.separator();

                ui.radio_value(&mut self.tool, Tool::Place, "◯ Место");
                ui.radio_value(&mut self.tool, Tool::Transition, "▮ Переход");
                ui.radio_value(&mut self.tool, Tool::Arc, "↗ Дуга");
                ui.radio_value(&mut self.tool, Tool::Text, "A Текст");
                ui.radio_value(&mut self.tool, Tool::Frame, "▭ Рамка");
                ui.radio_value(&mut self.tool, Tool::Edit, "✥ Редактировать");
                ui.radio_value(&mut self.tool, Tool::Delete, "✖ Удалить");

                if ui.button("СТАРТ").clicked() {
                    self.reset_sim_stop_controls();
                    self.show_sim_params = true;
                }

                ui.separator();
                ui.label(self.tr("Отображение связей", "Link visibility"));
                let is_ru = matches!(self.net.ui.language, Language::Ru);
                egui::ComboBox::from_label(self.tr("Режим", "Mode"))
                    .selected_text(Self::arc_display_mode_text(self.arc_display_mode, is_ru))
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(
                            &mut self.arc_display_mode,
                            ArcDisplayMode::All,
                            Self::arc_display_mode_text(ArcDisplayMode::All, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.arc_display_mode,
                            ArcDisplayMode::OnlyColor,
                            Self::arc_display_mode_text(ArcDisplayMode::OnlyColor, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.arc_display_mode,
                            ArcDisplayMode::Hidden,
                            Self::arc_display_mode_text(ArcDisplayMode::Hidden, is_ru),
                        );
                    });

                if self.arc_display_mode == ArcDisplayMode::OnlyColor {
                    let color_label = if is_ru { "Цвет" } else { "Color" };
                    let c_default = if is_ru {
                        "По умолчанию"
                    } else {
                        "Default"
                    };
                    let c_blue = if is_ru { "Синий" } else { "Blue" };
                    let c_red = if is_ru { "Красный" } else { "Red" };
                    let c_green = if is_ru { "Зеленый" } else { "Green" };
                    let c_yellow = if is_ru { "Желтый" } else { "Yellow" };

                    egui::ComboBox::from_label(color_label)
                        .selected_text(Self::node_color_text(self.arc_display_color, is_ru))
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Default,
                                c_default,
                            );
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Blue,
                                c_blue,
                            );
                            ui.selectable_value(&mut self.arc_display_color, NodeColor::Red, c_red);
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Green,
                                c_green,
                            );
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Yellow,
                                c_yellow,
                            );
                        });
                }

                let selected_arc_ids = self.collect_selected_arc_ids();
                if !selected_arc_ids.is_empty() {
                    ui.separator();
                    let color_label = self.tr("Цвет", "Color");

                    if selected_arc_ids.len() == 1 {
                        let arc_id = selected_arc_ids[0];
                        ui.label(self.tr("Выбранная связь", "Selected link"));

                        if let Some(arc) = self.net.arcs.iter_mut().find(|a| a.id == arc_id) {
                            egui::ComboBox::from_label(color_label)
                                .selected_text(Self::node_color_text(arc.color, is_ru))
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Default,
                                        Self::node_color_text(NodeColor::Default, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Blue,
                                        Self::node_color_text(NodeColor::Blue, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Red,
                                        Self::node_color_text(NodeColor::Red, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Green,
                                        Self::node_color_text(NodeColor::Green, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Yellow,
                                        Self::node_color_text(NodeColor::Yellow, is_ru),
                                    );
                                });
                        } else if let Some(inh) =
                            self.net.inhibitor_arcs.iter_mut().find(|a| a.id == arc_id)
                        {
                            egui::ComboBox::from_label(color_label)
                                .selected_text(Self::node_color_text(inh.color, is_ru))
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Default,
                                        Self::node_color_text(NodeColor::Default, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Blue,
                                        Self::node_color_text(NodeColor::Blue, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Red,
                                        Self::node_color_text(NodeColor::Red, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Green,
                                        Self::node_color_text(NodeColor::Green, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Yellow,
                                        Self::node_color_text(NodeColor::Yellow, is_ru),
                                    );
                                });
                        }
                    } else {
                        let selected_label = if is_ru {
                            format!("Выбрано связей: {}", selected_arc_ids.len())
                        } else {
                            format!("Selected links: {}", selected_arc_ids.len())
                        };
                        ui.label(selected_label);

                        let mut bulk_color = selected_arc_ids
                            .iter()
                            .find_map(|id| {
                                self.net
                                    .arcs
                                    .iter()
                                    .find(|a| a.id == *id)
                                    .map(|a| a.color)
                                    .or_else(|| {
                                        self.net
                                            .inhibitor_arcs
                                            .iter()
                                            .find(|a| a.id == *id)
                                            .map(|a| a.color)
                                    })
                            })
                            .unwrap_or(NodeColor::Default);
                        let previous_color = bulk_color;

                        egui::ComboBox::from_label(color_label)
                            .selected_text(Self::node_color_text(bulk_color, is_ru))
                            .show_ui(ui, |ui: &mut egui::Ui| {
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Default,
                                    Self::node_color_text(NodeColor::Default, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Blue,
                                    Self::node_color_text(NodeColor::Blue, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Red,
                                    Self::node_color_text(NodeColor::Red, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Green,
                                    Self::node_color_text(NodeColor::Green, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Yellow,
                                    Self::node_color_text(NodeColor::Yellow, is_ru),
                                );
                            });

                        if bulk_color != previous_color {
                            self.push_undo_snapshot();
                            let ids: HashSet<u64> = selected_arc_ids.iter().copied().collect();
                            for arc in &mut self.net.arcs {
                                if ids.contains(&arc.id) {
                                    arc.color = bulk_color;
                                }
                            }
                            for inh in &mut self.net.inhibitor_arcs {
                                if ids.contains(&inh.id) {
                                    inh.color = bulk_color;
                                }
                            }
                        }
                    }
                }
            });

        let open_props_by_rclick = ctx.input(|i| {
            if !i.pointer.button_clicked(egui::PointerButton::Secondary) {
                return false;
            }
            let Some(pos) = i.pointer.interact_pos() else {
                return false;
            };
            panel.response.rect.contains(pos)
        });
        if open_props_by_rclick {
            self.show_new_element_props = true;
        }
        if self.show_new_element_props {
            let is_ru = matches!(self.net.ui.language, Language::Ru);
            let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };
            let mut open = self.show_new_element_props;
            egui::Window::new(t(
                "Свойства создаваемых элементов",
                "New Element Properties",
            ))
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                let size_text = |size: VisualSize| -> &'static str {
                    if is_ru {
                        match size {
                            VisualSize::Small => "Малый",
                            VisualSize::Medium => "Средний",
                            VisualSize::Large => "Большой",
                        }
                    } else {
                        match size {
                            VisualSize::Small => "Small",
                            VisualSize::Medium => "Medium",
                            VisualSize::Large => "Large",
                        }
                    }
                };

                let color_combo = |ui: &mut egui::Ui, value: &mut NodeColor, is_ru: bool| {
                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(Self::node_color_text(*value, is_ru))
                        .show_ui(ui, |ui| {
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

                ui.group(|ui| {
                    ui.label(t("Новые места", "New places"));
                    egui::ComboBox::from_label(t("Размер", "Size"))
                        .selected_text(size_text(self.new_place_size))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_place_size,
                                VisualSize::Small,
                                size_text(VisualSize::Small),
                            );
                            ui.selectable_value(
                                &mut self.new_place_size,
                                VisualSize::Medium,
                                size_text(VisualSize::Medium),
                            );
                            ui.selectable_value(
                                &mut self.new_place_size,
                                VisualSize::Large,
                                size_text(VisualSize::Large),
                            );
                        });
                    ui.horizontal(|ui| {
                        ui.label(t("Цвет", "Color"));
                        color_combo(ui, &mut self.new_place_color, is_ru);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t("Маркеры", "Tokens"));
                        ui.add(
                            egui::DragValue::new(&mut self.new_place_marking).range(0..=u32::MAX),
                        );
                    });
                    ui.horizontal(|ui| {
                        let mut unlimited = self.new_place_capacity.is_none();
                        ui.checkbox(&mut unlimited, t("Без ограничений", "Unlimited"));
                        if unlimited {
                            self.new_place_capacity = None;
                        } else {
                            let mut cap = self.new_place_capacity.unwrap_or(1).max(1);
                            ui.label(t("Емкость", "Capacity"));
                            ui.add(egui::DragValue::new(&mut cap).range(1..=u32::MAX));
                            self.new_place_capacity = Some(cap);
                        }
                    });
                });

                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.label(t("Новые переходы", "New transitions"));
                    egui::ComboBox::from_label(t("Размер", "Size"))
                        .selected_text(size_text(self.new_transition_size))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_transition_size,
                                VisualSize::Small,
                                size_text(VisualSize::Small),
                            );
                            ui.selectable_value(
                                &mut self.new_transition_size,
                                VisualSize::Medium,
                                size_text(VisualSize::Medium),
                            );
                            ui.selectable_value(
                                &mut self.new_transition_size,
                                VisualSize::Large,
                                size_text(VisualSize::Large),
                            );
                        });
                    ui.horizontal(|ui| {
                        ui.label(t("Цвет", "Color"));
                        color_combo(ui, &mut self.new_transition_color, is_ru);
                    });
                    ui.horizontal(|ui| {
                        ui.label(t("Приоритет", "Priority"));
                        ui.add(
                            egui::DragValue::new(&mut self.new_transition_priority)
                                .range(-1_000_000..=1_000_000),
                        );
                    });
                });

                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.label(t("Новые дуги", "New arcs"));
                    ui.horizontal(|ui| {
                        ui.label(t("Кратность", "Weight"));
                        ui.add(egui::DragValue::new(&mut self.new_arc_weight).range(1..=u32::MAX));
                    });
                    ui.horizontal(|ui| {
                        ui.label(t("Цвет", "Color"));
                        color_combo(ui, &mut self.new_arc_color, is_ru);
                    });
                    let inhibitor_label = t("Ингибиторная дуга", "Inhibitor arc");
                    ui.checkbox(&mut self.new_arc_inhibitor, inhibitor_label);
                    if self.new_arc_inhibitor {
                        ui.horizontal(|ui| {
                            ui.label(t("Порог", "Threshold"));
                            ui.add(
                                egui::DragValue::new(&mut self.new_arc_inhibitor_threshold)
                                    .range(1..=u32::MAX),
                            );
                        });
                    }
                });
            });
            self.show_new_element_props = open;
        }
    }
}
