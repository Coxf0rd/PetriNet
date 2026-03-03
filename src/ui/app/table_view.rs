use super::*;

impl PetriApp {
    pub(super) fn draw_table_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("РЎС‚СЂСѓРєС‚СѓСЂР° СЃРµС‚Рё");
        ui.horizontal(|ui| {
            if ui.button("РЎРєСЂС‹С‚СЊ СЃС‚СЂСѓРєС‚СѓСЂСѓ").clicked()
            {
                self.show_table_view = false;
                self.table_fullscreen = false;
            }
            if ui
                .button(if self.table_fullscreen {
                    "РћР±С‹С‡РЅС‹Р№ СЂРµР¶РёРј"
                } else {
                    "РџРѕР»РЅС‹Р№ СЌРєСЂР°РЅ"
                })
                .clicked()
            {
                self.table_fullscreen = !self.table_fullscreen;
            }
        });
        ui.separator();
        if !self.show_table_view {
            return;
        }

        let mut p_count = self.net.places.len() as i32;
        let mut t_count = self.net.transitions.len() as i32;
        ui.horizontal(|ui| {
            ui.label("РњРµСЃС‚Р°:");
            ui.add(egui::DragValue::new(&mut p_count).range(0..=200));
            ui.label("РџРµСЂРµС…РѕРґС‹:");
            ui.add(egui::DragValue::new(&mut t_count).range(0..=200));
            if ui
                .button("РџСЂРёРјРµРЅРёС‚СЊ РєРѕР»РёС‡РµСЃС‚РІРѕ")
                .clicked()
            {
                self.net
                    .set_counts(p_count.max(0) as usize, t_count.max(0) as usize);
            }
        });

        let row_label_w = 46.0;
        let cell_w = 42.0;
        egui::ScrollArea::both().show(ui, |ui| {
            ui.separator();
            ui.label("Р’РµРєС‚РѕСЂ РЅР°С‡Р°Р»СЊРЅРѕР№ РјР°СЂРєРёСЂРѕРІРєРё (M0)");
            egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                for i in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", i + 1)));
                    ui.add_sized(
                        [cell_w * 1.4, 0.0],
                        egui::DragValue::new(&mut self.net.tables.m0[i]).range(0..=u32::MAX),
                    );
                    ui.end_row();
                }
            });

            ui.separator();
            ui.label("Р’РµРєС‚РѕСЂ РјР°РєСЃРёРјР°Р»СЊРЅС‹С… РµРјРєРѕСЃС‚РµР№ (Mo)");
            egui::Grid::new("mo_grid").striped(true).show(ui, |ui| {
                for i in 0..self.net.places.len() {
                    let mut cap = self.net.tables.mo[i].unwrap_or(0);
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", i + 1)));
                    if ui
                        .add_sized(
                            [cell_w * 1.4, 0.0],
                            egui::DragValue::new(&mut cap).range(0..=u32::MAX),
                        )
                        .changed()
                    {
                        self.net.tables.mo[i] = if cap == 0 { None } else { Some(cap) };
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.label("Р’РµРєС‚РѕСЂ РІСЂРµРјРµРЅРЅС‹С… Р·Р°РґРµСЂР¶РµРє РІ РїРѕР·РёС†РёСЏС… (Mz)");
            egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                for i in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", i + 1)));
                    ui.add_sized(
                        [cell_w * 1.8, 0.0],
                        egui::DragValue::new(&mut self.net.tables.mz[i])
                            .speed(0.1)
                            .range(0.0..=10_000.0),
                    );
                    ui.end_row();
                }
            });

            ui.separator();
            ui.label("Р’РµРєС‚РѕСЂ РїСЂРёРѕСЂРёС‚РµС‚РѕРІ РїРµСЂРµС…РѕРґРѕРІ (Mpr)");
            egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                    ui.add_sized(
                        [cell_w * 1.8, 0.0],
                        egui::DragValue::new(&mut self.net.tables.mpr[t]).speed(1),
                    );
                    ui.end_row();
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("РњР°С‚СЂРёС†Р° РёРЅС†РёРґРµРЅС†РёР№ Pre");
                if ui
                    .small_button(self.tr("РРјРїРѕСЂС‚ CSV", "Import CSV"))
                    .clicked()
                {
                    self.import_matrix_csv(MatrixCsvTarget::Pre);
                }
            });
            let mut changed = false;
            egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([cell_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                }
                ui.end_row();
                for p in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", p + 1)));
                    for t in 0..self.net.transitions.len() {
                        changed |= ui
                            .add_sized(
                                [cell_w, 0.0],
                                egui::DragValue::new(&mut self.net.tables.pre[p][t])
                                    .range(0..=u32::MAX)
                                    .speed(1),
                            )
                            .changed();
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("РњР°С‚СЂРёС†Р° РёРЅС†РёРґРµРЅС†РёР№ Post");
                if ui
                    .small_button(self.tr("РРјРїРѕСЂС‚ CSV", "Import CSV"))
                    .clicked()
                {
                    self.import_matrix_csv(MatrixCsvTarget::Post);
                }
            });
            egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([cell_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                }
                ui.end_row();
                for p in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", p + 1)));
                    for t in 0..self.net.transitions.len() {
                        changed |= ui
                            .add_sized(
                                [cell_w, 0.0],
                                egui::DragValue::new(&mut self.net.tables.post[p][t])
                                    .range(0..=u32::MAX)
                                    .speed(1),
                            )
                            .changed();
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("РњР°С‚СЂРёС†Р° РёРЅРіРёР±РёС‚РѕСЂРЅС‹С… РґСѓРі");
                if ui
                    .small_button(self.tr("РРјРїРѕСЂС‚ CSV", "Import CSV"))
                    .clicked()
                {
                    self.import_matrix_csv(MatrixCsvTarget::Inhibitor);
                }
            });
            egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([cell_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                }
                ui.end_row();
                for p in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", p + 1)));
                    for t in 0..self.net.transitions.len() {
                        changed |= ui
                            .add_sized(
                                [cell_w, 0.0],
                                egui::DragValue::new(&mut self.net.tables.inhibitor[p][t])
                                    .range(0..=u32::MAX)
                                    .speed(1),
                            )
                            .changed();
                    }
                    ui.end_row();
                }
            });

            if changed {
                self.net.rebuild_arcs_from_matrices();
            }
        });
    }

    pub(super) fn draw_sim_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_sim_params;
        let mut close_now = false;
        egui::Window::new("РџР°СЂР°РјРµС‚СЂС‹ СЃРёРјСѓР»СЏС†РёРё")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.checkbox(
                    &mut self.sim_params.use_time_limit,
                    "Р›РёРјРёС‚ РІСЂРµРјРµРЅРё (СЃРµРє)",
                );
                ui.add_enabled(
                    self.sim_params.use_time_limit,
                    egui::DragValue::new(&mut self.sim_params.time_limit_sec)
                        .speed(0.1)
                        .range(0.0..=1_000_000.0),
                );

                ui.checkbox(
                    &mut self.sim_params.use_pass_limit,
                    "Р›РёРјРёС‚ СЃСЂР°Р±Р°С‚С‹РІР°РЅРёР№",
                );
                ui.add_enabled(
                    self.sim_params.use_pass_limit,
                    egui::DragValue::new(&mut self.sim_params.pass_limit).range(0..=u64::MAX),
                );

                ui.horizontal(|ui| {
                    ui.label("Р”РёР°РїР°Р·РѕРЅ РјРµСЃС‚ РґР»СЏ РІС‹РІРѕРґР° РјР°СЂРєРёСЂРѕРІРєРё");
                    ui.add(
                        egui::DragValue::new(&mut self.sim_params.display_range_start)
                            .range(0..=10000),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.sim_params.display_range_end)
                            .range(0..=10000),
                    );
                });

                ui.separator();
                ui.label("РЈСЃР»РѕРІРёСЏ РѕСЃС‚Р°РЅРѕРІРєРё");
                let mut stop_place_enabled = self.sim_params.stop.through_place.is_some();
                ui.checkbox(
                    &mut stop_place_enabled,
                    "Р§РµСЂРµР· РјРµСЃС‚Рѕ Pk РїСЂРѕС€Р»Рѕ N РјР°СЂРєРµСЂРѕРІ",
                );
                if stop_place_enabled {
                    let (mut p, mut n) = self.sim_params.stop.through_place.unwrap_or((0, 1));
                    ui.horizontal(|ui| {
                        ui.label("Pk");
                        ui.add(egui::DragValue::new(&mut p).range(0..=10000));
                        ui.label("N");
                        ui.add(egui::DragValue::new(&mut n).range(1..=u64::MAX));
                    });
                    self.sim_params.stop.through_place = Some((p, n));
                } else {
                    self.sim_params.stop.through_place = None;
                }

                let mut stop_time_enabled = self.sim_params.stop.sim_time.is_some();
                ui.checkbox(
                    &mut stop_time_enabled,
                    "Р’СЂРµРјСЏ СЃРёРјСѓР»СЏС†РёРё РґРѕСЃС‚РёРіР»Рѕ T СЃРµРєСѓРЅРґ",
                );
                if stop_time_enabled {
                    let mut t = self.sim_params.stop.sim_time.unwrap_or(1.0);
                    ui.add(
                        egui::DragValue::new(&mut t)
                            .speed(0.1)
                            .range(0.0..=1_000_000.0),
                    );
                    self.sim_params.stop.sim_time = Some(t);
                } else {
                    self.sim_params.stop.sim_time = None;
                }

                if ui.button("РЎРўРђР Рў").clicked() {
                    self.net.rebuild_matrices_from_arcs();
                    self.sim_result = Some(std::sync::Arc::new(run_simulation(
                        &self.net,
                        &self.sim_params,
                        false,
                        self.net.ui.marker_count_stats,
                    )));
                    self.debug_step = 0;
                    self.debug_playing = false;
                    self.last_debug_tick = None;
                    self.show_results = true;
                    self.show_place_stats_window = false;
                    self.show_sim_params = false;
                    close_now = true;
                }
            });
        if close_now {
            open = false;
        }
        self.show_sim_params = open;
    }

    pub(super) fn draw_results(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.sim_result.clone() {
            let mut open = self.show_results;
            egui::Window::new(self.tr("Р РµР·СѓР»СЊС‚Р°С‚С‹/РЎС‚Р°С‚РёСЃС‚РёРєР°", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .show(ctx, |ui| {
                            ui.label(match result.cycle_time {
                                Some(t) => format!(
                                    "{}: {:.6} {}",
                                    self.tr("Р’СЂРµРјСЏ С†РёРєР»Р°", "Cycle time"),
                                    t,
                                    self.tr("СЃРµРє", "sec")
                                ),
                                None => format!("{}: N/A", self.tr("Р’СЂРµРјСЏ С†РёРєР»Р°", "Cycle time")),
                            });
                            ui.label(format!(
                                "{}: {}",
                                self.tr("РЎСЂР°Р±РѕС‚Р°Р»Рѕ РїРµСЂРµС…РѕРґРѕРІ", "Fired transitions"),
                                result.fired_count
                            ));
                            if result.log_entries_total > result.logs.len() {
                                ui.label(format!(
                                    "{}: {} / {} ({})",
                                    self.tr("Р–СѓСЂРЅР°Р» СЃСЌРјРїР»РёСЂРѕРІР°РЅ", "Log sampled"),
                                    result.logs.len(),
                                    result.log_entries_total,
                                    self.tr("С€Р°Рі СЃСЌРјРїР»РёСЂРѕРІР°РЅРёСЏ", "sampling stride"),
                                ));
                                ui.label(format!(
                                    "{} {}",
                                    self.tr("РўРµРєСѓС‰РёР№ С€Р°Рі:", "Current stride:"),
                                    result.log_sampling_stride,
                                ));
                            }

                            let stats_places: Vec<usize> = self
                                .net
                                .places
                                .iter()
                                .enumerate()
                                .filter_map(|(idx, place)| {
                                    if place.stats.any_enabled() {
                                        Some(idx)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !stats_places.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(self.tr(
                                        "Р”РµС‚Р°Р»СЊРЅР°СЏ СЃС‚Р°С‚РёСЃС‚РёРєР° РїРѕ РїРѕР·РёС†РёСЏРј РґРѕСЃС‚СѓРїРЅР°",
                                        "Detailed per-place statistics available",
                                    ));
                                    if ui.button(self.tr("РЎС‚Р°С‚РёСЃС‚РёРєР°", "Statistics")).clicked()
                                    {
                                        let selected = stats_places
                                            .iter()
                                            .position(|&p| p == self.place_stats_view_place)
                                            .unwrap_or(0);
                                        self.place_stats_view_place = stats_places[selected];
                                        self.show_place_stats_window = true;
                                    }
                                });
                            }

                            ui.separator();
                            ui.label(self.tr("Р–СѓСЂРЅР°Р» (С‚Р°Р±Р»РёС†Р°)", "Log (table)"));
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                egui::Grid::new("sim_log_grid_header").striped(true).show(
                                    ui,
                                    |ui| {
                                        ui.label(self.tr("Р’СЂРµРјСЏ", "Time"));
                                        for (p, _) in self.net.places.iter().enumerate() {
                                            ui.label(format!("P{}", p + 1));
                                        }
                                        ui.end_row();
                                    },
                                );

                                egui::ScrollArea::vertical().max_height(320.0).show_rows(
                                    ui,
                                    row_h,
                                    result.logs.len(),
                                    |ui, range| {
                                        egui::Grid::new("sim_log_grid_rows").striped(true).show(
                                            ui,
                                            |ui| {
                                                for idx in range {
                                                    let entry = &result.logs[idx];
                                                    ui.label(format!("{:.3}", entry.time));
                                                    for token in &entry.marking {
                                                        ui.label(token.to_string());
                                                    }
                                                    ui.end_row();
                                                }
                                            },
                                        );
                                    },
                                );
                            });

                            let any_place_stats_selected =
                                self.net.places.iter().any(|p| p.stats.any_enabled());
                            let show_all_places_in_stats = !any_place_stats_selected;

                            if let Some(stats) = &result.place_stats {
                                ui.separator();
                                ui.label(self.tr(
                                    "РЎС‚Р°С‚РёСЃС‚РёРєР° РјР°СЂРєРµСЂРѕРІ (min/max/avg)",
                                    "Token statistics (min/max/avg)",
                                ));
                                let rows: Vec<usize> = stats
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(p, _)| {
                                        let selected = self
                                            .net
                                            .places
                                            .get(p)
                                            .map(|pl| pl.stats.markers_total)
                                            .unwrap_or(false);
                                        if show_all_places_in_stats || selected {
                                            Some(p)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                egui::Grid::new("stats_grid_header")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(self.tr("РџРѕР·РёС†РёСЏ", "Place"));
                                        ui.label("Min");
                                        ui.label("Max");
                                        ui.label("Avg");
                                        ui.end_row();
                                    });
                                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                egui::ScrollArea::vertical()
                                    .id_source("stats_grid_scroll")
                                    .max_height(180.0)
                                    .show_rows(ui, row_h, rows.len(), |ui, range| {
                                        egui::Grid::new("stats_grid_rows").striped(true).show(
                                            ui,
                                            |ui| {
                                                for row_idx in range {
                                                    let p = rows[row_idx];
                                                    let st = &stats[p];
                                                    ui.label(format!("P{}", p + 1));
                                                    ui.label(st.min.to_string());
                                                    ui.label(st.max.to_string());
                                                    ui.label(format!("{:.3}", st.avg));
                                                    ui.end_row();
                                                }
                                            },
                                        );
                                    });
                            }

                            if let Some(flow) = &result.place_flow {
                                let want_flow =
                                    show_all_places_in_stats
                                        || self.net.places.iter().any(|p| {
                                            p.stats.markers_input || p.stats.markers_output
                                        });
                                if want_flow {
                                    ui.separator();
                                    ui.label(self.tr("РџРѕС‚РѕРєРё (РІС…РѕРґ/РІС‹С…РѕРґ)", "Flows (in/out)"));
                                    let rows: Vec<usize> = flow
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(p, _)| {
                                            let selected = self
                                                .net
                                                .places
                                                .get(p)
                                                .map(|pl| {
                                                    pl.stats.markers_input
                                                        || pl.stats.markers_output
                                                })
                                                .unwrap_or(false);
                                            if show_all_places_in_stats || selected {
                                                Some(p)
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    egui::Grid::new("flow_grid_header").striped(true).show(
                                        ui,
                                        |ui| {
                                            ui.label(self.tr("РџРѕР·РёС†РёСЏ", "Place"));
                                            ui.label(self.tr("Р’С…РѕРґ", "In"));
                                            ui.label(self.tr("Р’С‹С…РѕРґ", "Out"));
                                            ui.end_row();
                                        },
                                    );
                                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                    egui::ScrollArea::vertical()
                                        .id_source("flow_grid_scroll")
                                        .max_height(180.0)
                                        .show_rows(ui, row_h, rows.len(), |ui, range| {
                                            egui::Grid::new("flow_grid_rows").striped(true).show(
                                                ui,
                                                |ui| {
                                                    for row_idx in range {
                                                        let p = rows[row_idx];
                                                        let st = &flow[p];
                                                        ui.label(format!("P{}", p + 1));
                                                        ui.label(st.in_tokens.to_string());
                                                        ui.label(st.out_tokens.to_string());
                                                        ui.end_row();
                                                    }
                                                },
                                            );
                                        });
                                }
                            }

                            if let Some(load) = &result.place_load {
                                let want_load = show_all_places_in_stats
                                    || self.net.places.iter().any(|p| {
                                        p.stats.load_total
                                            || p.stats.load_input
                                            || p.stats.load_output
                                    });
                                if want_load {
                                    ui.separator();
                                    ui.label(self.tr("Р—Р°РіСЂСѓР¶РµРЅРЅРѕСЃС‚СЊ", "Load"));
                                    let rows: Vec<usize> = load
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(p, _)| {
                                            let selected = self
                                                .net
                                                .places
                                                .get(p)
                                                .map(|pl| {
                                                    pl.stats.load_total
                                                        || pl.stats.load_input
                                                        || pl.stats.load_output
                                                })
                                                .unwrap_or(false);
                                            if show_all_places_in_stats || selected {
                                                Some(p)
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    egui::Grid::new("load_grid_header").striped(true).show(
                                        ui,
                                        |ui| {
                                            ui.label(self.tr("РџРѕР·РёС†РёСЏ", "Place"));
                                            ui.label(self.tr("РћР±С‰Р°СЏ", "Total"));
                                            ui.label(self.tr("Р’С…РѕРґ", "Input"));
                                            ui.label(self.tr("Р’С‹С…РѕРґ", "Output"));
                                            ui.end_row();
                                        },
                                    );
                                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                    egui::ScrollArea::vertical()
                                        .id_source("load_grid_scroll")
                                        .max_height(180.0)
                                        .show_rows(ui, row_h, rows.len(), |ui, range| {
                                            egui::Grid::new("load_grid_rows").striped(true).show(
                                                ui,
                                                |ui| {
                                                    for row_idx in range {
                                                        let p = rows[row_idx];
                                                        let st = &load[p];
                                                        ui.label(format!("P{}", p + 1));
                                                        ui.label(match st.avg_over_capacity {
                                                            Some(v) => format!("{:.3}", v),
                                                            None => "N/A".to_string(),
                                                        });
                                                        ui.label(match st.in_rate {
                                                            Some(v) => format!("{:.3}", v),
                                                            None => "N/A".to_string(),
                                                        });
                                                        ui.label(match st.out_rate {
                                                            Some(v) => format!("{:.3}", v),
                                                            None => "N/A".to_string(),
                                                        });
                                                        ui.end_row();
                                                    }
                                                },
                                            );
                                        });
                                }
                            }
                });
            self.show_results = open;
        }
    }

    pub(super) fn draw_place_statistics_window(&mut self, ctx: &egui::Context) {
        if !self.show_place_stats_window {
            return;
        }
        let Some(result) = self.sim_result.clone() else {
            self.show_place_stats_window = false;
            return;
        };

        let available_places: Vec<usize> = self
            .net
            .places
            .iter()
            .enumerate()
            .filter_map(|(idx, place)| place.stats.any_enabled().then_some(idx))
            .collect();
        if available_places.is_empty() {
            self.show_place_stats_window = false;
            return;
        }
        if !available_places.contains(&self.place_stats_view_place) {
            self.place_stats_view_place = available_places[0];
        }

        let mut open = self.show_place_stats_window;
        egui::Window::new(self.tr("РЎС‚Р°С‚РёСЃС‚РёРєР°", "Statistics"))
            .id(egui::Id::new("results_place_stats_window"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(self.tr("РџРѕР·РёС†РёСЏ", "Place"));
                    let selected_place_text = self
                        .net
                        .places
                        .get(self.place_stats_view_place)
                        .map(|p| {
                            format!(
                                "P{} | {}",
                                self.place_stats_view_place + 1,
                                if p.name.is_empty() {
                                    format!("P{}", self.place_stats_view_place + 1)
                                } else {
                                    p.name.clone()
                                }
                            )
                        })
                        .unwrap_or_else(|| format!("P{}", self.place_stats_view_place + 1));
                    egui::ComboBox::from_id_source("results_stats_place_combo")
                        .selected_text(selected_place_text)
                        .width(420.0)
                        .show_ui(ui, |ui| {
                            for idx in &available_places {
                                let label = self
                                    .net
                                    .places
                                    .get(*idx)
                                    .map(|p| {
                                        format!(
                                            "P{} | {}",
                                            *idx + 1,
                                            if p.name.is_empty() {
                                                format!("P{}", *idx + 1)
                                            } else {
                                                p.name.clone()
                                            }
                                        )
                                    })
                                    .unwrap_or_else(|| format!("P{}", *idx + 1));
                                ui.selectable_value(&mut self.place_stats_view_place, *idx, label);
                            }
                        });
                    ui.label(format!("P{}", self.place_stats_view_place + 1));
                    ui.separator();
                    let selected_name = self
                        .net
                        .places
                        .get(self.place_stats_view_place)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| format!("P{}", self.place_stats_view_place + 1));
                    ui.label(selected_name);
                });

                let place_idx = self.place_stats_view_place;
                let place_stats = self
                    .net
                    .places
                    .get(place_idx)
                    .map(|p| p.stats)
                    .unwrap_or_default();
                let mut available_series = Vec::new();
                if place_stats.markers_total {
                    available_series.push(PlaceStatsSeries::MarkersTotal);
                }
                if place_stats.markers_input {
                    available_series.push(PlaceStatsSeries::MarkersInput);
                }
                if place_stats.markers_output {
                    available_series.push(PlaceStatsSeries::MarkersOutput);
                }
                if available_series.is_empty() {
                    available_series.push(PlaceStatsSeries::MarkersTotal);
                }
                if !available_series.contains(&self.place_stats_series) {
                    self.place_stats_series = available_series[0];
                }
                ui.horizontal(|ui| {
                    ui.label(self.tr("РџРѕРєР°Р·Р°С‚РµР»СЊ", "Metric"));
                    for series in available_series {
                        let label = match series {
                            PlaceStatsSeries::MarkersTotal => self.tr("РћР±С‰Р°СЏ", "Total"),
                            PlaceStatsSeries::MarkersInput => {
                                self.tr("РќР° РІС…РѕРґРµ", "On input")
                            }
                            PlaceStatsSeries::MarkersOutput => {
                                self.tr("РќР° РІС‹С…РѕРґРµ", "On output")
                            }
                        };
                        ui.selectable_value(&mut self.place_stats_series, series, label);
                    }
                });

                let sampled = Self::sampled_indices(result.logs.len(), Self::MAX_PLOT_POINTS);
                let mut values = Vec::<f64>::with_capacity(sampled.len());
                let mut times = Vec::<f64>::with_capacity(sampled.len());

                let mut cumulative_in = vec![0_u64; result.logs.len()];
                let mut cumulative_out = vec![0_u64; result.logs.len()];
                let mut in_sum = 0_u64;
                let mut out_sum = 0_u64;
                for (idx, entry) in result.logs.iter().enumerate() {
                    if let Some(t_idx) = entry.fired_transition {
                        in_sum = in_sum.saturating_add(
                            *self
                                .net
                                .tables
                                .post
                                .get(place_idx)
                                .and_then(|row| row.get(t_idx))
                                .unwrap_or(&0) as u64,
                        );
                        out_sum = out_sum.saturating_add(
                            *self
                                .net
                                .tables
                                .pre
                                .get(place_idx)
                                .and_then(|row| row.get(t_idx))
                                .unwrap_or(&0) as u64,
                        );
                    }
                    cumulative_in[idx] = in_sum;
                    cumulative_out[idx] = out_sum;
                }

                for idx in sampled {
                    let entry = &result.logs[idx];
                    let value = match self.place_stats_series {
                        PlaceStatsSeries::MarkersTotal => {
                            entry.marking.get(place_idx).copied().unwrap_or_default() as f64
                        }
                        PlaceStatsSeries::MarkersInput => {
                            cumulative_in.get(idx).copied().unwrap_or_default() as f64
                        }
                        PlaceStatsSeries::MarkersOutput => {
                            cumulative_out.get(idx).copied().unwrap_or_default() as f64
                        }
                    };
                    values.push(value);
                    times.push(if entry.time.is_finite() {
                        entry.time
                    } else {
                        idx as f64
                    });
                }

                if values.len() >= 2 {
                    let mut has_increasing_x = false;
                    for i in 1..times.len() {
                        if times[i] > times[i - 1] {
                            has_increasing_x = true;
                            break;
                        }
                    }
                    if !has_increasing_x {
                        for (i, t) in times.iter_mut().enumerate() {
                            *t = i as f64;
                        }
                    }
                }
                if values.is_empty() {
                    ui.label(self.tr(
                        "РќРµС‚ РґР°РЅРЅС‹С… РґР»СЏ РѕС‚РѕР±СЂР°Р¶РµРЅРёСЏ",
                        "No data to display",
                    ));
                    return;
                }
                if result.logs.len() > values.len() {
                    ui.label(format!(
                        "{}: {} / {}",
                        self.tr("Р“СЂР°С„РёРє СЃСЌРјРїР»РёСЂРѕРІР°РЅ", "Plot sampled"),
                        values.len(),
                        result.logs.len()
                    ));
                }

                let mut max_v = values[0];
                let mut min_v = values[0];
                let mut max_t = times[0];
                let mut min_t = times[0];
                let mut sum = 0.0;
                for (v, t) in values.iter().zip(times.iter()) {
                    sum += *v;
                    if *v > max_v {
                        max_v = *v;
                        max_t = *t;
                    }
                    if *v < min_v {
                        min_v = *v;
                        min_t = *t;
                    }
                }
                let avg = sum / values.len() as f64;
                let place_load = result
                    .place_load
                    .as_ref()
                    .and_then(|load| load.get(place_idx));
                let summary_tail = match self.place_stats_series {
                    PlaceStatsSeries::MarkersTotal => format!(
                        "{} {:.3}%",
                        self.tr("РЈС‚РёР»РёР·Р°С†РёСЏ", "Utilization"),
                        place_load
                            .and_then(|l| l.avg_over_capacity)
                            .map(|v| v * 100.0)
                            .unwrap_or(0.0)
                    ),
                    PlaceStatsSeries::MarkersInput => format!(
                        "{} {:.3}",
                        self.tr("РЎСЂ. РІС…РѕРґ/СЃРµРє", "Avg in/sec"),
                        place_load.and_then(|l| l.in_rate).unwrap_or(0.0)
                    ),
                    PlaceStatsSeries::MarkersOutput => format!(
                        "{} {:.3}",
                        self.tr("РЎСЂ. РІС‹С…РѕРґ/СЃРµРє", "Avg out/sec"),
                        place_load.and_then(|l| l.out_rate).unwrap_or(0.0)
                    ),
                };

                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} {:.3}",
                        self.tr("РњР°РєСЃРёРјСѓРј", "Maximum"),
                        max_v
                    ));
                    ui.label(format!("{} {:.3}", self.tr("Р’СЂРµРјСЏ", "Time"), max_t));
                    ui.separator();
                    ui.label(format!(
                        "{} {:.3}",
                        self.tr("РњРёРЅРёРјСѓРј", "Minimum"),
                        min_v
                    ));
                    ui.label(format!("{} {:.3}", self.tr("Р’СЂРµРјСЏ", "Time"), min_t));
                    ui.separator();
                    ui.label(format!(
                        "{} {:.3}",
                        self.tr("РЎСЂРµРґРЅРµРµ", "Average"),
                        avg
                    ));
                    ui.label(summary_tail);
                });
                ui.horizontal(|ui| {
                    ui.label(self.tr("РњР°СЃС€С‚Р°Р± X", "X zoom"));
                    ui.add(
                        egui::Slider::new(&mut self.place_stats_zoom_x, 1.0..=20.0)
                            .logarithmic(true),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.place_stats_zoom_x)
                            .range(1.0..=20.0)
                            .speed(0.01)
                            .fixed_decimals(3),
                    );
                    ui.label(self.tr("РЎРґРІРёРі X", "X pan"));
                    ui.add(egui::Slider::new(&mut self.place_stats_pan_x, 0.0..=1.0));
                    ui.add(
                        egui::DragValue::new(&mut self.place_stats_pan_x)
                            .range(0.0..=1.0)
                            .speed(0.001)
                            .fixed_decimals(3),
                    );
                    ui.separator();
                    let grid_label = self.tr("Показать сетку", "Show grid");
                    ui.checkbox(&mut self.place_stats_show_grid, grid_label);
                });

                let total = values.len();
                let visible = (((total as f32) / self.place_stats_zoom_x).round() as usize)
                    .clamp(2, total.max(2));
                let max_start = total.saturating_sub(visible);
                let start = ((max_start as f32) * self.place_stats_pan_x)
                    .round()
                    .clamp(0.0, max_start as f32) as usize;
                let end = (start + visible).min(total);
                let values_window = &values[start..end];
                let times_window = &times[start..end];

                let desired_size = egui::Vec2::new(ui.available_width(), 360.0);
                let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));
                let left_pad = 50.0;
                let right_pad = 14.0;
                let top_pad = 14.0;
                let bottom_pad = 28.0;
                let plot_rect = Rect::from_min_max(
                    Pos2::new(rect.left() + left_pad, rect.top() + top_pad),
                    Pos2::new(rect.right() - right_pad, rect.bottom() - bottom_pad),
                );
                painter.rect_stroke(plot_rect, 0.0, Stroke::new(1.0, Color32::GRAY));

                let x_min = times_window.first().copied().unwrap_or(0.0);
                let mut x_max = times_window.last().copied().unwrap_or(1.0);
                if x_max <= x_min {
                    x_max = x_min + (times_window.len().max(1) as f64);
                }
                let y_min = 0.0;
                let mut y_max = values_window
                    .iter()
                    .copied()
                    .fold(0.0_f64, |acc, v| if v > acc { v } else { acc })
                    .max(1.0);
                if y_max <= y_min {
                    y_max = y_min + 1.0;
                }

                ui.label(format!(
                    "{}: [{:.3} .. {:.3}] | {}: {} / {}",
                    self.tr("Диапазон X", "X range"),
                    x_min,
                    x_max,
                    self.tr("Точки", "Points"),
                    values_window.len(),
                    values.len()
                ));

                if self.place_stats_show_grid {
                    for i in 1..10 {
                        let x = plot_rect.left() + plot_rect.width() * (i as f32 / 10.0);
                        painter.line_segment(
                            [
                                Pos2::new(x, plot_rect.top()),
                                Pos2::new(x, plot_rect.bottom()),
                            ],
                            Stroke::new(0.5, Color32::LIGHT_GRAY),
                        );
                    }
                    for i in 1..4 {
                        let y = plot_rect.bottom() - plot_rect.height() * (i as f32 / 4.0);
                        painter.line_segment(
                            [
                                Pos2::new(plot_rect.left(), y),
                                Pos2::new(plot_rect.right(), y),
                            ],
                            Stroke::new(0.5, Color32::LIGHT_GRAY),
                        );
                    }
                }

                let to_screen = |x: f64, y: f64| -> Pos2 {
                    let xr = ((x - x_min) / (x_max - x_min)).clamp(0.0, 1.0) as f32;
                    let yr = ((y - y_min) / (y_max - y_min)).clamp(0.0, 1.0) as f32;
                    Pos2::new(
                        plot_rect.left() + xr * plot_rect.width(),
                        plot_rect.bottom() - yr * plot_rect.height(),
                    )
                };

                let mut points = Vec::with_capacity(values_window.len());
                for (x, y) in times_window.iter().zip(values_window.iter()) {
                    points.push(to_screen(*x, *y));
                }
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(points, Stroke::new(1.6, Color32::BLUE)));
                }

                painter.text(
                    Pos2::new(rect.left() + 4.0, plot_rect.top()),
                    egui::Align2::LEFT_TOP,
                    format!("{:.3}", y_max),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
                painter.text(
                    Pos2::new(rect.left() + 4.0, plot_rect.bottom()),
                    egui::Align2::LEFT_BOTTOM,
                    "0",
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
                painter.text(
                    Pos2::new(plot_rect.left(), plot_rect.bottom() + 6.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.3}", x_min),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
                painter.text(
                    Pos2::new(plot_rect.right(), plot_rect.bottom() + 6.0),
                    egui::Align2::RIGHT_TOP,
                    format!("{:.3}", x_max),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
                painter.text(
                    Pos2::new(plot_rect.center().x, rect.bottom() - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    self.tr("РћСЃСЊ X: РІСЂРµРјСЏ/С€Р°РіРё", "X axis: time/steps"),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
            });

        self.show_place_stats_window = open;
    }
}
