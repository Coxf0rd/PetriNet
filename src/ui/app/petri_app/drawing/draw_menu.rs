use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(self.tr("Р¤Р°Р№Р»", "File"), |ui| {
                    if ui.button("РќРѕРІС‹Р№ (Ctrl+N)").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("РћС‚РєСЂС‹С‚СЊ (Ctrl+O)").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.menu_button("РРјРїРѕСЂС‚", |ui| {
                        ui.label("РРјРїРѕСЂС‚ PeSim: TODO");
                    });
                    ui.menu_button("Р­РєСЃРїРѕСЂС‚", |ui| {
                        if ui.button("Р­РєСЃРїРѕСЂС‚ РІ NetStar (gpn)").clicked() {
                            self.export_netstar_file();
                            ui.close_menu();
                        }
                    });
                    if ui.button("РЎРѕС…СЂР°РЅРёС‚СЊ (gpn2) (Ctrl+S)").clicked()
                    {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("РЎРѕС…СЂР°РЅРёС‚СЊ РєР°Рє (gpn2)").clicked()
                    {
                        self.save_file_as();
                        ui.close_menu();
                    }
                });
                ui.menu_button("РћРїС†РёРё", |ui| {
                    ui.menu_button("РЇР·С‹Рє", |ui| {
                        ui.radio_value(&mut self.net.ui.language, Language::Ru, "RU");
                        ui.radio_value(&mut self.net.ui.language, Language::En, "EN");
                    });
                    ui.checkbox(&mut self.net.ui.hide_grid, "РЎРєСЂС‹С‚СЊ СЃРµС‚РєСѓ");
                    ui.checkbox(
                        &mut self.net.ui.snap_to_grid,
                        "РџСЂРёРІСЏР·РєР° Рє СЃРµС‚РєРµ",
                    );
                    ui.checkbox(
                        &mut self.net.ui.colored_petri_nets,
                        "Р¦РІРµС‚РЅС‹Рµ СЃРµС‚Рё РџРµС‚СЂРё",
                    );
                    ui.menu_button(
                        "РЎР±РѕСЂ СЃС‚Р°С‚РёСЃС‚РёРєРё",
                        |ui| {
                            ui.checkbox(
                                &mut self.net.ui.marker_count_stats,
                                "РЎС‚Р°С‚РёСЃС‚РёРєР° РјР°СЂРєРµСЂРѕРІ",
                            );
                        },
                    );
                    ui.menu_button("Help", |ui| {
                        if ui.button("Р Р°Р·СЂР°Р±РѕС‚РєР°").clicked() {
                            self.show_help_development = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("РџРѕРјРѕС‰СЊ РїРѕ СѓРїСЂР°РІР»РµРЅРёСЋ")
                            .clicked()
                        {
                            self.show_help_controls = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("РћРєРЅРѕ", |ui| {
                    let options = [
                        (
                            LayoutMode::TileHorizontal,
                            self.tr(
                                "РџР»РёС‚РєР° РїРѕ РіРѕСЂРёР·РѕРЅС‚Р°Р»Рё",
                                "Tile horizontal",
                            ),
                        ),
                        (
                            LayoutMode::TileVertical,
                            self.tr("РџР»РёС‚РєР° РїРѕ РІРµСЂС‚РёРєР°Р»Рё", "Tile vertical"),
                        ),
                        (
                            LayoutMode::Minimized,
                            self.tr("РЎРІРµСЂРЅСѓС‚СЊ РІСЃРµ", "Minimize all"),
                        ),
                    ];
                    for (mode, label) in options {
                        let selected = self.layout_mode == mode;
                        if ui
                            .add(egui::SelectableLabel::new(selected, label.as_ref()))
                            .clicked()
                        {
                            self.layout_mode = mode;
                        }
                    }
                });

                let markov_available = self.sim_result.is_some();
                ui.add_enabled_ui(markov_available, |ui| {
                    let response = ui
                        .button(self.tr("РњР°СЂРєРѕРІСЃРєР°СЏ РјРѕРґРµР»СЊ", "Markov model"))
                        .on_hover_text(self.tr(
                            "РўСЂРµР±СѓРµС‚СЃСЏ Р°РєС‚РёРІРЅР°СЏ СЃРёРјСѓР»СЏС†РёСЏ",
                            "Requires an active simulation",
                        ));
                    if response.clicked() {
                        self.show_markov_window = true;
                    }
                });

                if ui.button("РЎС‚СЂСѓРєС‚СѓСЂР° СЃРµС‚Рё").clicked() {
                    self.show_table_view = !self.show_table_view;
                    if !self.show_table_view {
                        self.table_fullscreen = false;
                    }
                }
                if ui
                    .button(self.tr(
                        "Р РµР·СѓР»СЊС‚Р°С‚С‹ РёРјРёС‚Р°С†РёРё",
                        "Simulation Results",
                    ))
                    .clicked()
                {
                    if self.sim_result.is_some() {
                        self.show_results = !self.show_results;
                    } else {
                        self.show_results = false;
                    }
                }
                if ui.button("Proof").clicked() && self.sim_result.is_some() {
                    self.show_proof = true;
                }
                if ui
                    .button(self.tr("Р РµР¶РёРј РѕС‚Р»Р°РґРєРё", "Debug Mode"))
                    .clicked()
                    && self.sim_result.is_some()
                {
                    self.show_debug = true;
                }
                if ui.button("ATF").clicked() {
                    self.show_atf = true;
                }
            });
        });
    }
}
