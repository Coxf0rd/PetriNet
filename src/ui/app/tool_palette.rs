use super::*;

impl PetriApp {
    pub(super) fn draw_tool_palette(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("tools").resizable(false).show(ctx, |ui| {
            ui.heading("РРЅСЃС‚СЂСѓРјРµРЅС‚С‹");
            ui.separator();
            ui.radio_value(&mut self.tool, Tool::Place, "РњРµСЃС‚Рѕ");
            ui.radio_value(&mut self.tool, Tool::Transition, "РџРµСЂРµС…РѕРґ");
            ui.radio_value(&mut self.tool, Tool::Arc, "Р”СѓРіР°");
            ui.radio_value(&mut self.tool, Tool::Text, "РўРµРєСЃС‚");
            ui.radio_value(&mut self.tool, Tool::Frame, "Р Р°РјРєР°");
            ui.radio_value(&mut self.tool, Tool::Edit, "Р РµРґР°РєС‚РёСЂРѕРІР°С‚СЊ");
            ui.radio_value(&mut self.tool, Tool::Delete, "РЈРґР°Р»РёС‚СЊ");
            ui.radio_value(&mut self.tool, Tool::Run, "Р—Р°РїСѓСЃРє");

            if ui.button("РЎРўРђР Рў").clicked() {
                self.reset_sim_stop_controls();
                self.show_sim_params = true;
            }

            ui.separator();
            ui.label(self.tr("РћС‚РѕР±СЂР°Р¶РµРЅРёРµ СЃРІСЏР·РµР№", "Link visibility"));
            let is_ru = matches!(self.net.ui.language, Language::Ru);
            egui::ComboBox::from_label(self.tr("Р РµР¶РёРј", "Mode"))
                .selected_text(Self::arc_display_mode_text(self.arc_display_mode, is_ru))
                .show_ui(ui, |ui| {
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
                let color_label = if is_ru { "Р¦РІРµС‚" } else { "Color" };
                let c_default = if is_ru { "РџРѕ СѓРјРѕР»С‡Р°РЅРёСЋ" } else { "Default" };
                let c_blue = if is_ru { "РЎРёРЅРёР№" } else { "Blue" };
                let c_red = if is_ru { "РљСЂР°СЃРЅС‹Р№" } else { "Red" };
                let c_green = if is_ru { "Р—РµР»РµРЅС‹Р№" } else { "Green" };
                let c_yellow = if is_ru { "Р–РµР»С‚С‹Р№" } else { "Yellow" };

                egui::ComboBox::from_label(color_label)
                    .selected_text(Self::node_color_text(self.arc_display_color, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Default, c_default);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Blue, c_blue);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Red, c_red);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Green, c_green);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Yellow, c_yellow);
                    });
            }

            let selected_arc_ids = self.collect_selected_arc_ids();
            if !selected_arc_ids.is_empty() {
                ui.separator();
                let color_label = self.tr("Р¦РІРµС‚", "Color");

                if selected_arc_ids.len() == 1 {
                    let arc_id = selected_arc_ids[0];
                    ui.label(self.tr("Р’С‹Р±СЂР°РЅРЅР°СЏ СЃРІСЏР·СЊ", "Selected link"));

                    if let Some(arc) = self.net.arcs.iter_mut().find(|a| a.id == arc_id) {
                        egui::ComboBox::from_label(color_label)
                            .selected_text(Self::node_color_text(arc.color, is_ru))
                            .show_ui(ui, |ui| {
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
                    } else if let Some(inh) = self.net.inhibitor_arcs.iter_mut().find(|a| a.id == arc_id) {
                        egui::ComboBox::from_label(color_label)
                            .selected_text(Self::node_color_text(inh.color, is_ru))
                            .show_ui(ui, |ui| {
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
                        format!("Р’С‹Р±СЂР°РЅРѕ СЃРІСЏР·РµР№: {}", selected_arc_ids.len())
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
                        .show_ui(ui, |ui| {
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
    }
}
