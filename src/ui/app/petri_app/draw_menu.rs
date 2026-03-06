use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(self.tr("\u{424}\u{430}\u{439}\u{43B}", "File"), |ui| {
                    if ui.button("Новый (Ctrl+N)").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Открыть (Ctrl+O)").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.menu_button("Импорт", |ui| {
                        ui.label("Импорт PeSim: TODO");
                    });
                    ui.menu_button("Экспорт", |ui| {
                        if ui.button("Экспорт в NetStar (gpn)").clicked() {
                            self.export_netstar_file();
                            ui.close_menu();
                        }
                    });
                    if ui.button("Сохранить (gpn2) (Ctrl+S)").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Сохранить как (gpn2)").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    if ui.button("Выход").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Опции", |ui| {
                    ui.menu_button("Язык", |ui| {
                        ui.radio_value(&mut self.net.ui.language, Language::Ru, "RU");
                        ui.radio_value(&mut self.net.ui.language, Language::En, "EN");
                    });
                    ui.checkbox(&mut self.net.ui.hide_grid, "Скрыть сетку");
                    ui.checkbox(&mut self.net.ui.snap_to_grid, "Привязка к сетке");
                    ui.checkbox(&mut self.net.ui.colored_petri_nets, "Цветные сети Петри");
                    ui.menu_button("Сбор статистики", |ui| {
                        ui.checkbox(&mut self.net.ui.marker_count_stats, "Статистика маркеров");
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Разработка").clicked() {
                            self.show_help_development = true;
                            ui.close_menu();
                        }
                        if ui.button("Помощь по управлению").clicked() {
                            self.show_help_controls = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Окно", |ui| {
                    if ui.button("Каскад").clicked() {
                        self.layout_mode = LayoutMode::Cascade;
                    }
                    if ui.button("Плитка по горизонтали").clicked() {
                        self.layout_mode = LayoutMode::TileHorizontal;
                    }
                    if ui.button("Плитка по вертикали").clicked() {
                        self.layout_mode = LayoutMode::TileVertical;
                    }
                    if ui.button("Свернуть все").clicked() {
                        self.layout_mode = LayoutMode::Minimized;
                    }
                    if ui.button("Упорядочить все").clicked() {
                        self.layout_mode = LayoutMode::TileVertical;
                        self.show_graph_view = true;
                    }
                });

                if ui.button("Параметры симуляции").clicked() {
                    self.reset_sim_stop_controls();
                    self.show_sim_params = true;
                }
                if ui.button("Структура сети").clicked() {
                    self.show_table_view = !self.show_table_view;
                    if !self.show_table_view {
                        self.table_fullscreen = false;
                    }
                }
                if ui
                    .button(self.tr("Результаты имитации", "Simulation Results"))
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
                if ui.button(self.tr("Режим отладки", "Debug Mode")).clicked()
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
