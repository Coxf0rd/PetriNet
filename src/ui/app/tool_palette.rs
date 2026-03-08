use super::*;
use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};

impl PetriApp {
    pub(super) fn draw_tool_palette(&mut self, ctx: &egui::Context) {
        if self.tool == Tool::Run {
            self.tool = Tool::Edit;
        }

        let panel =
            egui::SidePanel::left("tools")
                .resizable(true)
                .show(ctx, |ui: &mut egui::Ui| {
                    ui.heading("Инструменты");
                    ui.separator();

                    for (tool_variant, icon, label) in [
                        (Tool::Place, "O", "Позиция"),
                        (Tool::Transition, "II", "Переход"),
                        (Tool::Arc, "↗", "Дуга"),
                        (Tool::Text, "A", "Текст"),
                        (Tool::Frame, "[]", "Рамка"),
                        (Tool::Edit, "✥", "Редактировать"),
                        (Tool::Delete, "✖", "Удалить"),
                    ] {
                        let selected = self.tool == tool_variant;
                        let text = format!("{} {}", icon, label);
                        if ui.add(egui::SelectableLabel::new(selected, text)).clicked() {
                            self.tool = tool_variant;
                        }
                    }

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
                                ui.selectable_value(
                                    &mut self.arc_display_color,
                                    NodeColor::Red,
                                    c_red,
                                );
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

                    ui.separator();
                    ui.label(self.tr("Редактор", "Editor"));
                    let title = self.tr("Показать сетку", "Show grid");
                    let mut show_grid = !self.net.ui.hide_grid;
                    if ui.checkbox(&mut show_grid, title).changed() {
                        self.net.ui.hide_grid = !show_grid;
                    }
                    let snap_label = self.tr("Привязка к сетке", "Snap to grid");
                    ui.checkbox(&mut self.net.ui.snap_to_grid, snap_label);

                    ui.separator();
                    ui.label(self.tr("Выделение", "Selection"));

                    let places_selected = !self.canvas.selected_places.is_empty();
                    let transitions_selected = !self.canvas.selected_transitions.is_empty();
                    let arcs_selected =
                        self.canvas.selected_arc.is_some() || !self.canvas.selected_arcs.is_empty();

                    if places_selected {
                        let selected_place_ids = if self.canvas.selected_places.is_empty() {
                            self.canvas.selected_place.into_iter().collect::<Vec<_>>()
                        } else {
                            self.canvas.selected_places.clone()
                        };

                        if !selected_place_ids.is_empty() {
                            let mut bulk_place_size = self
                                .net
                                .places
                                .iter()
                                .find(|p| p.id == selected_place_ids[0])
                                .map(|p| p.size)
                                .unwrap_or(VisualSize::Medium);

                            ui.label(self.tr("Размер выбранных позиций", "Selected place size"));
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.radio_value(
                                    &mut bulk_place_size,
                                    VisualSize::Small,
                                    self.tr("Малый", "Small"),
                                );
                                ui.radio_value(
                                    &mut bulk_place_size,
                                    VisualSize::Medium,
                                    self.tr("Средний", "Medium"),
                                );
                                ui.radio_value(
                                    &mut bulk_place_size,
                                    VisualSize::Large,
                                    self.tr("Большой", "Large"),
                                );
                            });

                            if bulk_place_size
                                != self
                                    .net
                                    .places
                                    .iter()
                                    .find(|p| p.id == selected_place_ids[0])
                                    .map(|p| p.size)
                                    .unwrap_or(VisualSize::Medium)
                            {
                                self.push_undo_snapshot();
                                let ids: HashSet<u64> =
                                    selected_place_ids.iter().copied().collect();
                                for place in &mut self.net.places {
                                    if ids.contains(&place.id) {
                                        place.size = bulk_place_size;
                                    }
                                }
                            }

                            let mut bulk_color = self
                                .net
                                .places
                                .iter()
                                .find(|p| p.id == selected_place_ids[0])
                                .map(|p| p.color)
                                .unwrap_or(NodeColor::Default);
                            let previous_color = bulk_color;

                            ui.label(self.tr("Цвет выбранных позиций", "Selected place color"));
                            egui::ComboBox::from_id_source("bulk_place_color_combo")
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
                                let ids: HashSet<u64> =
                                    selected_place_ids.iter().copied().collect();
                                for place in &mut self.net.places {
                                    if ids.contains(&place.id) {
                                        place.color = bulk_color;
                                    }
                                }
                            }
                        }
                    }

                    if transitions_selected {
                        let selected_transition_ids = if self.canvas.selected_transitions.is_empty()
                        {
                            self.canvas
                                .selected_transition
                                .into_iter()
                                .collect::<Vec<_>>()
                        } else {
                            self.canvas.selected_transitions.clone()
                        };

                        if !selected_transition_ids.is_empty() {
                            let mut bulk_transition_size = self
                                .net
                                .transitions
                                .iter()
                                .find(|t| t.id == selected_transition_ids[0])
                                .map(|t| t.size)
                                .unwrap_or(VisualSize::Medium);

                            ui.label(
                                self.tr("Размер выбранных переходов", "Selected transition size"),
                            );
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.radio_value(
                                    &mut bulk_transition_size,
                                    VisualSize::Small,
                                    self.tr("Малый", "Small"),
                                );
                                ui.radio_value(
                                    &mut bulk_transition_size,
                                    VisualSize::Medium,
                                    self.tr("Средний", "Medium"),
                                );
                                ui.radio_value(
                                    &mut bulk_transition_size,
                                    VisualSize::Large,
                                    self.tr("Большой", "Large"),
                                );
                            });

                            if bulk_transition_size
                                != self
                                    .net
                                    .transitions
                                    .iter()
                                    .find(|t| t.id == selected_transition_ids[0])
                                    .map(|t| t.size)
                                    .unwrap_or(VisualSize::Medium)
                            {
                                self.push_undo_snapshot();
                                let ids: HashSet<u64> =
                                    selected_transition_ids.iter().copied().collect();
                                for transition in &mut self.net.transitions {
                                    if ids.contains(&transition.id) {
                                        transition.size = bulk_transition_size;
                                    }
                                }
                            }

                            let mut bulk_color = self
                                .net
                                .transitions
                                .iter()
                                .find(|t| t.id == selected_transition_ids[0])
                                .map(|t| t.color)
                                .unwrap_or(NodeColor::Default);
                            let previous_color = bulk_color;

                            ui.label(
                                self.tr("Цвет выбранных переходов", "Selected transition color"),
                            );
                            egui::ComboBox::from_id_source("bulk_transition_color_combo")
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
                                let ids: HashSet<u64> =
                                    selected_transition_ids.iter().copied().collect();
                                for transition in &mut self.net.transitions {
                                    if ids.contains(&transition.id) {
                                        transition.color = bulk_color;
                                    }
                                }
                            }
                        }
                    }

                    if arcs_selected {
                        let selected_arc_ids = if self.canvas.selected_arcs.is_empty() {
                            self.canvas.selected_arc.into_iter().collect::<Vec<_>>()
                        } else {
                            self.canvas.selected_arcs.clone()
                        };

                        if !selected_arc_ids.is_empty() {
                            let mut bulk_color = self
                                .net
                                .arcs
                                .iter()
                                .find(|a| a.id == selected_arc_ids[0])
                                .map(|a| a.color)
                                .or_else(|| {
                                    self.net
                                        .inhibitor_arcs
                                        .iter()
                                        .find(|a| a.id == selected_arc_ids[0])
                                        .map(|a| a.color)
                                })
                                .unwrap_or(NodeColor::Default);
                            let previous_color = bulk_color;

                            ui.label(self.tr("Цвет выбранных дуг", "Selected arc color"));
                            egui::ComboBox::from_id_source("bulk_arc_color_combo")
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
            let was_open = self.new_element_props_window_was_open;
            let apply_default_size = !was_open && open;
            let invalid_inputs_adjusted = self.tr(
                "Некорректные значения были скорректированы",
                "Invalid inputs were adjusted",
            );
            let mut new_element_props_window_size = self.new_element_props_window_size;
            show_property_window(
                ctx,
                t("Свойства создаваемых элементов", "New Element Properties"),
                &mut open,
                PropertyWindowConfig::new("new_element_props_window")
                    .remember_size(&mut new_element_props_window_size)
                    .apply_default_size(apply_default_size),
                |ui: &mut egui::Ui| {
                    let mut corrected_inputs = false;

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

                    show_collapsible_property_section(
                        ui,
                        t("Новые позиции", "New positions"),
                        PropertySectionConfig::new("new_element_props_place_section")
                            .default_open(true),
                        |ui: &mut egui::Ui| {
                            egui::ComboBox::from_label(t("Размер позиции", "Position size"))
                                .selected_text(size_text(self.new_place_size))
                                .show_ui(ui, |ui: &mut egui::Ui| {
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

                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t("Цвет", "Color"));
                                color_combo(ui, &mut self.new_place_color, is_ru);
                            });

                            let mut marking = self.new_place_marking;
                            corrected_inputs |= sanitize_u32(&mut marking, 0, u32::MAX);
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t("Маркеры", "Tokens"));
                                if ui
                                    .add(egui::DragValue::new(&mut marking).range(0..=u32::MAX))
                                    .changed()
                                {
                                    corrected_inputs |= sanitize_u32(&mut marking, 0, u32::MAX);
                                }
                            });
                            self.new_place_marking = marking;

                            let mut cap = self.new_place_capacity.unwrap_or(0);
                            corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t(
                                    "Макс. емкость (0 = без ограничений)",
                                    "Capacity (0 = unlimited)",
                                ));
                                if ui
                                    .add(egui::DragValue::new(&mut cap).range(0..=u32::MAX))
                                    .changed()
                                {
                                    corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                                }
                            });
                            self.new_place_capacity = if cap == 0 { None } else { Some(cap) };
                        },
                    );

                    show_collapsible_property_section(
                        ui,
                        t("Новые переходы", "New transitions"),
                        PropertySectionConfig::new("new_element_props_transition_section")
                            .default_open(true)
                            .top_spacing(6.0),
                        |ui: &mut egui::Ui| {
                            egui::ComboBox::from_label(t("Размер перехода", "Transition size"))
                                .selected_text(size_text(self.new_transition_size))
                                .show_ui(ui, |ui: &mut egui::Ui| {
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

                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t("Цвет", "Color"));
                                color_combo(ui, &mut self.new_transition_color, is_ru);
                            });

                            let mut transition_priority = self.new_transition_priority;
                            corrected_inputs |=
                                sanitize_i32(&mut transition_priority, -1_000_000, 1_000_000);
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t("Приоритет", "Priority"));
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut transition_priority)
                                            .range(-1_000_000..=1_000_000),
                                    )
                                    .changed()
                                {
                                    corrected_inputs |= sanitize_i32(
                                        &mut transition_priority,
                                        -1_000_000,
                                        1_000_000,
                                    );
                                }
                            });
                            self.new_transition_priority = transition_priority;
                        },
                    );

                    show_collapsible_property_section(
                        ui,
                        t("Новые дуги", "New arcs"),
                        PropertySectionConfig::new("new_element_props_arc_section")
                            .default_open(true)
                            .top_spacing(6.0),
                        |ui: &mut egui::Ui| {
                            let mut arc_weight = self.new_arc_weight;
                            corrected_inputs |= sanitize_u32(&mut arc_weight, 1, u32::MAX);
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t("Кратность (вес)", "Weight"));
                                if ui
                                    .add(egui::DragValue::new(&mut arc_weight).range(1..=u32::MAX))
                                    .changed()
                                {
                                    corrected_inputs |= sanitize_u32(&mut arc_weight, 1, u32::MAX);
                                }
                            });
                            self.new_arc_weight = arc_weight;

                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(t("Цвет", "Color"));
                                color_combo(ui, &mut self.new_arc_color, is_ru);
                            });

                            let inhibitor_label = t("Ингибиторная дуга", "Inhibitor arc");
                            ui.checkbox(&mut self.new_arc_inhibitor, inhibitor_label);
                            if self.new_arc_inhibitor {
                                ui.horizontal(|ui: &mut egui::Ui| {
                                    ui.label(t("Порог", "Threshold"));
                                    let mut threshold = self.new_arc_inhibitor_threshold;
                                    corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut threshold)
                                                .range(1..=u32::MAX),
                                        )
                                        .changed()
                                    {
                                        corrected_inputs |=
                                            sanitize_u32(&mut threshold, 1, u32::MAX);
                                    }
                                    self.new_arc_inhibitor_threshold = threshold;
                                });
                            }
                        },
                    );

                    validation_hint(ui, corrected_inputs, &invalid_inputs_adjusted);
                },
            );
            self.show_new_element_props = open;
            self.new_element_props_window_was_open = open;
            self.new_element_props_window_size = new_element_props_window_size;
        }
    }
}
