use super::*;

impl PetriApp {
    pub(super) fn draw_table_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Структура сети");
        ui.horizontal(|ui| {
            if ui.button("Скрыть структуру").clicked() {
                self.show_table_view = false;
                self.table_fullscreen = false;
            }
            if ui
                .button(if self.table_fullscreen {
                    "Обычный режим"
                } else {
                    "Полный экран"
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
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let vector_scroll_height = 220.0;
        let matrix_scroll_height = 320.0;

        ui.horizontal(|ui| {
            ui.label("Показывать:");
            let vectors_label = self.tr("Векторы", "Vectors");
            let pre_label = self.tr("Матрица Pre", "Pre matrix");
            let post_label = self.tr("Матрица Post", "Post matrix");
            let inhibitor_label = self.tr("Ингибиторные дуги", "Inhibitor matrix");
            ui.checkbox(&mut self.show_struct_vectors, vectors_label);
            ui.checkbox(&mut self.show_struct_pre, pre_label);
            ui.checkbox(&mut self.show_struct_post, post_label);
            ui.checkbox(&mut self.show_struct_inhibitor, inhibitor_label);
        });

        let mut p_count = self.net.places.len() as i32;
        let mut t_count = self.net.transitions.len() as i32;
        ui.horizontal(|ui| {
            ui.label("Места:");
            ui.add(egui::DragValue::new(&mut p_count).range(0..=200));
            ui.label("Переходы:");
            ui.add(egui::DragValue::new(&mut t_count).range(0..=200));
            if ui.button("Применить количество").clicked() {
                self.net
                    .set_counts(p_count.max(0) as usize, t_count.max(0) as usize);
            }
        });

        let row_label_w = 46.0;
        let cell_w = 42.0;
        egui::ScrollArea::both().show(ui, |ui| {
            if self.show_struct_vectors {
                ui.separator();
                ui.label("Вектор начальной маркировки (M0)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("m0_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                            for i in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", i + 1)),
                                );
                                ui.add_sized(
                                    [cell_w * 1.4, 0.0],
                                    egui::DragValue::new(&mut self.net.tables.m0[i])
                                        .range(0..=u32::MAX),
                                );
                                ui.end_row();
                            }
                        });
                    },
                );

                ui.separator();
                ui.label("Вектор максимальных емкостей (Mo)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("mo_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("mo_grid").striped(true).show(ui, |ui| {
                            for i in rows {
                                let mut cap = self.net.tables.mo[i].unwrap_or(0);
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", i + 1)),
                                );
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
                    },
                );

                ui.separator();
                ui.label("Вектор временных задержек в позициях (Mz)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("mz_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                            for i in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", i + 1)),
                                );
                                ui.add_sized(
                                    [cell_w * 1.8, 0.0],
                                    egui::DragValue::new(&mut self.net.tables.mz[i])
                                        .speed(0.1)
                                        .range(0.0..=10_000.0),
                                );
                                ui.end_row();
                            }
                        });
                    },
                );

                ui.separator();
                ui.label("Вектор приоритетов переходов (Mpr)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("mpr_grid_scroll"),
                    self.net.transitions.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                            for t in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                                ui.add_sized(
                                    [cell_w * 1.8, 0.0],
                                    egui::DragValue::new(&mut self.net.tables.mpr[t]).speed(1),
                                );
                                ui.end_row();
                            }
                        });
                    },
                );
            }
            let mut matrices_changed = false;
            if self.show_struct_pre {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Матрица инцидентций Pre");
                    if ui
                        .small_button(self.tr("Импорт CSV", "Import CSV"))
                        .clicked()
                    {
                        self.import_matrix_csv(MatrixCsvTarget::Pre);
                    }
                });
                let mut pre_changed = false;
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("pre_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                            ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                            for t in 0..self.net.transitions.len() {
                                ui.add_sized(
                                    [cell_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                            }
                            ui.end_row();
                            for p in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", p + 1)),
                                );
                                for t in 0..self.net.transitions.len() {
                                    pre_changed |= ui
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
                    },
                );
                matrices_changed |= pre_changed;
            }
            if self.show_struct_post {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Матрица инцидентций Post");
                    if ui
                        .small_button(self.tr("Импорт CSV", "Import CSV"))
                        .clicked()
                    {
                        self.import_matrix_csv(MatrixCsvTarget::Post);
                    }
                });
                let mut post_changed = false;
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("post_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                            ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                            for t in 0..self.net.transitions.len() {
                                ui.add_sized(
                                    [cell_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                            }
                            ui.end_row();
                            for p in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", p + 1)),
                                );
                                for t in 0..self.net.transitions.len() {
                                    post_changed |= ui
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
                    },
                );
                matrices_changed |= post_changed;
            }
            if self.show_struct_inhibitor {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Матрица ингибиторных дуг");
                    if ui
                        .small_button(self.tr("Импорт CSV", "Import CSV"))
                        .clicked()
                    {
                        self.import_matrix_csv(MatrixCsvTarget::Inhibitor);
                    }
                });
                let mut inhibitor_changed = false;
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("inh_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                            ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                            for t in 0..self.net.transitions.len() {
                                ui.add_sized(
                                    [cell_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                            }
                            ui.end_row();
                            for p in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", p + 1)),
                                );
                                for t in 0..self.net.transitions.len() {
                                    inhibitor_changed |= ui
                                        .add_sized(
                                            [cell_w, 0.0],
                                            egui::DragValue::new(
                                                &mut self.net.tables.inhibitor[p][t],
                                            )
                                            .range(0..=u32::MAX)
                                            .speed(1),
                                        )
                                        .changed();
                                }
                                ui.end_row();
                            }
                        });
                    },
                );
                matrices_changed |= inhibitor_changed;
            }
            if matrices_changed {
                self.net.rebuild_arcs_from_matrices();
            }
        });
    }

    pub(super) fn draw_sim_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_sim_params;
        let mut close_now = false;
        egui::Window::new(self.tr("Параметры симуляции", "Simulation Parameters"))
            .open(&mut open)
            .show(ctx, |ui| {
                let mut corrected_inputs = false;
                let max_places = self.net.places.len();
                let time_limit_label = self.tr("Лимит времени (сек)", "Time limit (sec)");
                ui.checkbox(&mut self.sim_params.use_time_limit, time_limit_label);
                ui.add_enabled(
                    self.sim_params.use_time_limit,
                    egui::DragValue::new(&mut self.sim_params.time_limit_sec)
                        .speed(0.1)
                        .range(0.0..=1_000_000.0),
                );
                corrected_inputs |=
                    sanitize_f64(&mut self.sim_params.time_limit_sec, 0.0, 1_000_000.0);

                let pass_limit_label = self.tr("Лимит срабатываний", "Fire count limit");
                ui.checkbox(&mut self.sim_params.use_pass_limit, pass_limit_label);
                ui.add_enabled(
                    self.sim_params.use_pass_limit,
                    egui::DragValue::new(&mut self.sim_params.pass_limit).range(0..=u64::MAX),
                );
                corrected_inputs |= sanitize_u64(&mut self.sim_params.pass_limit, 0, 1_000_000);

                ui.horizontal(|ui| {
                    ui.label(self.tr(
                        "Диапазон мест для вывода маркировки",
                        "Place range for marking output",
                    ));
                    ui.add(
                        egui::DragValue::new(&mut self.sim_params.display_range_start)
                            .range(0..=10000),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.sim_params.display_range_end)
                            .range(0..=10000),
                    );
                    corrected_inputs |=
                        sanitize_usize(&mut self.sim_params.display_range_start, 0, max_places);
                    corrected_inputs |=
                        sanitize_usize(&mut self.sim_params.display_range_end, 0, max_places);
                    if self.sim_params.display_range_end < self.sim_params.display_range_start {
                        self.sim_params.display_range_end = self.sim_params.display_range_start;
                        corrected_inputs = true;
                    }
                });

                ui.separator();
                ui.label(self.tr("Условия остановки", "Stop conditions"));
                let mut stop_place_enabled = self.sim_params.stop.through_place.is_some();
                let stop_place_label = self.tr(
                    "Через место Pk прошло N маркеров",
                    "N tokens passed through place Pk",
                );
                ui.checkbox(&mut stop_place_enabled, stop_place_label);
                if stop_place_enabled {
                    let (mut p, mut n) = self.sim_params.stop.through_place.unwrap_or((0, 1));
                    let max_place_idx = self.net.places.len().saturating_sub(1);
                    ui.horizontal(|ui| {
                        ui.label(self.tr("Pk (k-1)", "Pk (k-1)"));
                        ui.add(egui::DragValue::new(&mut p).range(0..=max_place_idx));
                        ui.label("N");
                        ui.add(egui::DragValue::new(&mut n).range(1..=u64::MAX));
                    });
                    corrected_inputs |= sanitize_usize(&mut p, 0, max_place_idx);
                    corrected_inputs |= sanitize_u64(&mut n, 1, 1_000_000);
                    p = p.min(max_place_idx);
                    self.sim_params.stop.through_place = Some((p, n));
                } else {
                    self.sim_params.stop.through_place = None;
                }

                let mut stop_time_enabled = self.sim_params.stop.sim_time.is_some();
                let stop_time_label = self.tr(
                    "Время симуляции достигло T секунд",
                    "Simulation time reached T seconds",
                );
                ui.checkbox(&mut stop_time_enabled, stop_time_label);
                if stop_time_enabled {
                    let mut t = self.sim_params.stop.sim_time.unwrap_or(1.0);
                    ui.add(
                        egui::DragValue::new(&mut t)
                            .speed(0.1)
                            .range(0.0..=1_000_000.0),
                    );
                    corrected_inputs |= sanitize_f64(&mut t, 0.0, 1_000_000.0);
                    self.sim_params.stop.sim_time = Some(t);
                } else {
                    self.sim_params.stop.sim_time = None;
                }

                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
                if ui.button(self.tr("СТАРТ", "START")).clicked() {
                    self.net.sanitize_values();
                    self.net.rebuild_matrices_from_arcs();
                    self.sim_result = Some(std::sync::Arc::new(run_simulation(
                        &self.net,
                        &self.sim_params,
                        false,
                        self.net.ui.marker_count_stats,
                    )));
                    self.refresh_debug_animation_state();
                    self.debug_step = 0;
                    self.sync_debug_animation_for_step();
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
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("results_window_scroll")
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.label(match result.cycle_time {
                                Some(t) => format!(
                                    "{}: {:.6} {}",
                                    self.tr("Время цикла", "Cycle time"),
                                    t,
                                    self.tr("сек", "sec")
                                ),
                                None => format!("{}: N/A", self.tr("Время цикла", "Cycle time")),
                            });
                            let total_minutes = result.sim_time / 60.0;
                            ui.label(format!(
                                "{}: {:.4} {} / {:.4} {}",
                                self.tr("Итоговое время эмуляции", "Total simulation time"),
                                result.sim_time,
                                self.tr("сек", "sec"),
                                total_minutes,
                                self.tr("мин", "min")
                            ));
                            ui.label(format!(
                                "{}: {}",
                                self.tr("Сработало переходов", "Fired transitions"),
                                result.fired_count
                            ));
                            if result.log_entries_total > result.logs.len() {
                                ui.label(format!(
                                    "{}: {} / {} ({})",
                                    self.tr("Журнал сэмплирован", "Log sampled"),
                                    result.logs.len(),
                                    result.log_entries_total,
                                    self.tr("шаг сэмплирования", "sampling stride"),
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
                                        "Детальная статистика по позициям доступна",
                                        "Detailed per-place statistics available",
                                    ));
                                    if ui.button(self.tr("Статистика", "Statistics")).clicked()
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
                            ui.horizontal(|ui| {
                                ui.label(self.tr("Журнал (таблица)", "Log (table)"));
                                if ui.button(self.tr("Экспорт CSV", "Export CSV")).clicked()
                                {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("CSV", &["csv"])
                                        .set_file_name("simulation_log.csv")
                                        .save_file()
                                    {
                                        let mut csv = String::new();
                                        csv.push_str("time");
                                        for (p, _) in self.net.places.iter().enumerate() {
                                            csv.push(',');
                                            csv.push_str(&format!("P{}", p + 1));
                                        }
                                        csv.push('\n');
                                        for entry in &result.logs {
                                            csv.push_str(&format!("{:.6}", entry.time));
                                            for token in &entry.marking {
                                                csv.push(',');
                                                csv.push_str(&token.to_string());
                                            }
                                            csv.push('\n');
                                        }
                                        match std::fs::write(&path, csv) {
                                            Ok(_) => {
                                                self.status_hint = Some(format!(
                                                    "{}: {}",
                                                    self.tr("Журнал экспортирован", "Log exported"),
                                                    path.display()
                                                ));
                                                self.last_error = None;
                                            }
                                            Err(e) => {
                                                self.last_error = Some(format!(
                                                    "{}: {}",
                                                    self.tr(
                                                        "Ошибка экспорта CSV",
                                                        "CSV export error"
                                                    ),
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                            });
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                egui::Grid::new("sim_log_grid_header").striped(true).show(
                                    ui,
                                    |ui| {
                                        ui.label(self.tr("Время", "Time"));
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
                                    "Статистика маркеров (min/max/avg)",
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
                                        ui.label(self.tr("Позиция", "Place"));
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
                                    ui.label(self.tr("Потоки (вход/выход)", "Flows (in/out)"));
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
                                            ui.label(self.tr("Позиция", "Place"));
                                            ui.label(self.tr("Вход", "In"));
                                            ui.label(self.tr("Выход", "Out"));
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
                                    ui.label(self.tr("Загруженность", "Load"));
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
                                            ui.label(self.tr("Позиция", "Place"));
                                            ui.label(self.tr("Общая", "Total"));
                                            ui.label(self.tr("Вход", "Input"));
                                            ui.label(self.tr("Выход", "Output"));
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
        egui::Window::new(self.tr("Статистика", "Statistics"))
            .id(egui::Id::new("results_place_stats_window"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(self.tr("Позиция", "Place"));
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
                    available_series.push(PlaceStatsSeries::Total);
                }
                if place_stats.markers_input {
                    available_series.push(PlaceStatsSeries::Input);
                }
                if place_stats.markers_output {
                    available_series.push(PlaceStatsSeries::Output);
                }
                if available_series.is_empty() {
                    available_series.push(PlaceStatsSeries::Total);
                }
                if !available_series.contains(&self.place_stats_series) {
                    self.place_stats_series = available_series[0];
                }
                ui.horizontal(|ui| {
                    ui.label(self.tr("Показатель", "Metric"));
                    for series in available_series {
                        let label = match series {
                            PlaceStatsSeries::Total => self.tr("Общая", "Total"),
                            PlaceStatsSeries::Input => self.tr("На входе", "On input"),
                            PlaceStatsSeries::Output => self.tr("На выходе", "On output"),
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
                        PlaceStatsSeries::Total => {
                            entry.marking.get(place_idx).copied().unwrap_or_default() as f64
                        }
                        PlaceStatsSeries::Input => {
                            cumulative_in.get(idx).copied().unwrap_or_default() as f64
                        }
                        PlaceStatsSeries::Output => {
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
                    ui.label(self.tr("Нет данных для отображения", "No data to display"));
                    return;
                }
                if result.logs.len() > values.len() {
                    ui.label(format!(
                        "{}: {} / {}",
                        self.tr("График сэмплирован", "Plot sampled"),
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
                    PlaceStatsSeries::Total => format!(
                        "{} {:.3}%",
                        self.tr("Утилизация", "Utilization"),
                        place_load
                            .and_then(|l| l.avg_over_capacity)
                            .map(|v| v * 100.0)
                            .unwrap_or(0.0)
                    ),
                    PlaceStatsSeries::Input => format!(
                        "{} {:.3}",
                        self.tr("Ср. вход/сек", "Avg in/sec"),
                        place_load.and_then(|l| l.in_rate).unwrap_or(0.0)
                    ),
                    PlaceStatsSeries::Output => format!(
                        "{} {:.3}",
                        self.tr("Ср. выход/сек", "Avg out/sec"),
                        place_load.and_then(|l| l.out_rate).unwrap_or(0.0)
                    ),
                };

                ui.horizontal(|ui| {
                    ui.label(format!("{} {:.3}", self.tr("Максимум", "Maximum"), max_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), max_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Минимум", "Minimum"), min_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), min_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Среднее", "Average"), avg));
                    ui.label(summary_tail);
                });
                ui.horizontal(|ui| {
                    ui.label(self.tr("Масштаб X", "X zoom"));
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
                    ui.label(self.tr("Сдвиг X", "X pan"));
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
                let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));
                let left_pad = 50.0;
                let right_pad = 14.0;
                let top_pad = 14.0;
                let bottom_pad = 36.0;
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

                let x_step = ((x_max - x_min) / 10.0).max(0.000_001);
                let y_step = ((y_max - y_min) / 10.0).max(0.000_001);

                if self.place_stats_show_grid {
                    ui.label(format!(
                        "{}: {:.3} | {}: {:.3}",
                        self.tr("Шаг сетки X", "Grid step X"),
                        x_step,
                        self.tr("Шаг сетки Y", "Grid step Y"),
                        y_step
                    ));
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
                    for i in 1..10 {
                        let y = plot_rect.bottom() - plot_rect.height() * (i as f32 / 10.0);
                        painter.line_segment(
                            [
                                Pos2::new(plot_rect.left(), y),
                                Pos2::new(plot_rect.right(), y),
                            ],
                            Stroke::new(0.5, Color32::LIGHT_GRAY),
                        );
                    }

                    for i in 0..=10 {
                        let t = i as f32 / 10.0;
                        let x = plot_rect.left() + plot_rect.width() * t;
                        let xv = x_min + x_step * i as f64;
                        painter.text(
                            Pos2::new(x, plot_rect.bottom() + 6.0),
                            egui::Align2::CENTER_TOP,
                            format!("{:.1}", xv),
                            egui::FontId::default(),
                            Color32::DARK_GRAY,
                        );
                    }

                    for i in 0..=10 {
                        let t = i as f32 / 10.0;
                        let y = plot_rect.bottom() - plot_rect.height() * t;
                        let yv = y_min + y_step * i as f64;
                        painter.text(
                            Pos2::new(rect.left() + 4.0, y),
                            egui::Align2::LEFT_CENTER,
                            format!("{:.1}", yv),
                            egui::FontId::default(),
                            Color32::DARK_GRAY,
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

                let mut points_data = Vec::with_capacity(values_window.len());
                let mut line_points = Vec::with_capacity(values_window.len());
                for (x, y) in times_window.iter().zip(values_window.iter()) {
                    let pt = to_screen(*x, *y);
                    points_data.push((pt, *x, *y));
                    line_points.push(pt);
                }
                if line_points.len() >= 2 {
                    painter.add(egui::Shape::line(
                        line_points,
                        Stroke::new(1.6, Color32::BLUE),
                    ));
                }

                if let Some(mouse_pos) = response.hover_pos() {
                    if plot_rect.contains(mouse_pos) {
                        let x_tolerance = 32.0;
                        let y_tolerance = 80.0;
                        if let Some((pos, x, y)) = points_data
                            .iter()
                            .filter_map(|(pos, x, y)| {
                                let dx = (mouse_pos.x - pos.x).abs();
                                let dy = (mouse_pos.y - pos.y).abs();
                                let normalized =
                                    (dx / x_tolerance).powi(2) + (dy / y_tolerance).powi(2);
                                if normalized <= 1.0 {
                                    Some((normalized, *pos, *x, *y))
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| {
                                a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .map(|(_, pos, x, y)| (pos, x, y))
                        {
                            painter.circle_filled(pos, 4.0, Color32::WHITE);
                            painter.circle_stroke(pos, 4.0, Stroke::new(2.0, Color32::BLUE));
                            let point_label = format!(
                                "{}: {:.3}, {}: {:.3}",
                                self.tr("X", "X"),
                                x,
                                self.tr("Y", "Y"),
                                y
                            );
                            painter.text(
                                pos + Vec2::new(6.0, 12.0),
                                egui::Align2::LEFT_TOP,
                                point_label,
                                egui::FontId::default(),
                                Color32::BLACK,
                            );
                        }
                    }
                }

                if !self.place_stats_show_grid {
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
                }
                painter.text(
                    Pos2::new(plot_rect.center().x, rect.bottom() - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    self.tr("Ось X: время/шаги", "X axis: time/steps"),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
            });

        self.show_place_stats_window = open;
    }

    fn scroll_area_rows<F>(
        ui: &mut egui::Ui,
        id: egui::Id,
        row_len: usize,
        row_h: f32,
        max_height: f32,
        body: F,
    ) where
        F: FnOnce(&mut egui::Ui, std::ops::Range<usize>),
    {
        if row_len == 0 {
            return;
        }
        let available = ui.available_height();
        let height = if available.is_finite() {
            max_height.min(available.max(row_h))
        } else {
            max_height.max(row_h)
        };
        egui::ScrollArea::vertical()
            .id_source(id)
            .max_height(height)
            .show_rows(ui, row_h, row_len, body);
    }
}
