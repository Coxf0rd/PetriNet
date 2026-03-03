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
            ui.separator();
            ui.label("Вектор начальной маркировки (M0)");
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
            ui.label("Вектор максимальных емкостей (Mo)");
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
            ui.label("Вектор временных задержек в позициях (Mz)");
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
            ui.label("Вектор приоритетов переходов (Mpr)");
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
                ui.label("Матрица инциденций Pre");
                if ui
                    .small_button(self.tr("Импорт CSV", "Import CSV"))
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
                ui.label("Матрица инциденций Post");
                if ui
                    .small_button(self.tr("Импорт CSV", "Import CSV"))
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
                ui.label("Матрица ингибиторных дуг");
                if ui
                    .small_button(self.tr("Импорт CSV", "Import CSV"))
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
        egui::Window::new("Параметры симуляции")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.sim_params.use_time_limit, "Лимит времени (сек)");
                ui.add_enabled(
                    self.sim_params.use_time_limit,
                    egui::DragValue::new(&mut self.sim_params.time_limit_sec)
                        .speed(0.1)
                        .range(0.0..=1_000_000.0),
                );

                ui.checkbox(&mut self.sim_params.use_pass_limit, "Лимит срабатываний");
                ui.add_enabled(
                    self.sim_params.use_pass_limit,
                    egui::DragValue::new(&mut self.sim_params.pass_limit).range(0..=u64::MAX),
                );

                ui.horizontal(|ui| {
                    ui.label("Диапазон мест для вывода маркировки");
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
                ui.label("Условия остановки");
                let mut stop_place_enabled = self.sim_params.stop.through_place.is_some();
                ui.checkbox(&mut stop_place_enabled, "Через место Pk прошло N маркеров");
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
                ui.checkbox(&mut stop_time_enabled, "Время симуляции достигло T секунд");
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

                if ui.button("СТАРТ").clicked() {
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
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label(match result.cycle_time {
                        Some(t) => format!(
                            "{}: {:.6} {}",
                            self.tr("Время цикла", "Cycle time"),
                            t,
                            self.tr("сек", "sec")
                        ),
                        None => format!("{}: N/A", self.tr("Время цикла", "Cycle time")),
                    });
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
                        ui.label(format!(
                            "{} {}",
                            self.tr("Текущий шаг:", "Current stride:"),
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
                                "Детальная статистика по позициям доступна",
                                "Detailed per-place statistics available",
                            ));
                            if ui.button(self.tr("Статистика", "Statistics")).clicked() {
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
                    ui.label(self.tr("Журнал (таблица)", "Log (table)"));
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                        egui::Grid::new("sim_log_grid_header")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(self.tr("Время", "Time"));
                                for (p, _) in self.net.places.iter().enumerate() {
                                    ui.label(format!("P{}", p + 1));
                                }
                                ui.end_row();
                            });

                        egui::ScrollArea::vertical().max_height(320.0).show_rows(
                            ui,
                            row_h,
                            result.logs.len(),
                            |ui, range| {
                                egui::Grid::new("sim_log_grid_rows")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for idx in range {
                                            let entry = &result.logs[idx];
                                            ui.label(format!("{:.3}", entry.time));
                                            for token in &entry.marking {
                                                ui.label(token.to_string());
                                            }
                                            ui.end_row();
                                        }
                                    });
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
                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                        egui::Grid::new("stats_grid_header")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(self.tr("Позиция", "Place"));
                                ui.label("Min");
                                ui.label("Max");
                                ui.label("Avg");
                                ui.end_row();
                            });
                        egui::ScrollArea::vertical().max_height(180.0).show_rows(
                            ui,
                            row_h,
                            rows.len(),
                            |ui, range| {
                                egui::Grid::new("stats_grid_rows")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for row_idx in range {
                                            let p = rows[row_idx];
                                            let st = &stats[p];
                                            ui.label(format!("P{}", p + 1));
                                            ui.label(st.min.to_string());
                                            ui.label(st.max.to_string());
                                            ui.label(format!("{:.3}", st.avg));
                                            ui.end_row();
                                        }
                                    });
                            },
                        );
                    }

                    if let Some(flow) = &result.place_flow {
                        let want_flow = show_all_places_in_stats
                            || self
                                .net
                                .places
                                .iter()
                                .any(|p| p.stats.markers_input || p.stats.markers_output);
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
                                        .map(|pl| pl.stats.markers_input || pl.stats.markers_output)
                                        .unwrap_or(false);
                                    if show_all_places_in_stats || selected {
                                        Some(p)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                            egui::Grid::new("flow_grid_header")
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(self.tr("Позиция", "Place"));
                                    ui.label(self.tr("Вход", "In"));
                                    ui.label(self.tr("Выход", "Out"));
                                    ui.end_row();
                                });
                            egui::ScrollArea::vertical().max_height(180.0).show_rows(
                                ui,
                                row_h,
                                rows.len(),
                                |ui, range| {
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
                                },
                            );
                        }
                    }

                    if let Some(load) = &result.place_load {
                        let want_load = show_all_places_in_stats
                            || self.net.places.iter().any(|p| {
                                p.stats.load_total || p.stats.load_input || p.stats.load_output
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
                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                            egui::Grid::new("load_grid_header")
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(self.tr("Позиция", "Place"));
                                    ui.label(self.tr("Общая", "Total"));
                                    ui.label(self.tr("Вход", "Input"));
                                    ui.label(self.tr("Выход", "Output"));
                                    ui.end_row();
                                });
                            egui::ScrollArea::vertical().max_height(180.0).show_rows(
                                ui,
                                row_h,
                                rows.len(),
                                |ui, range| {
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
                                },
                            );
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
            .filter_map(|(idx, place)| {
                if place.stats.any_enabled() {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        if available_places.is_empty() {
            self.show_place_stats_window = false;
            return;
        }
        if !available_places.contains(&self.place_stats_view_place) {
            self.place_stats_view_place = available_places[0];
        }
        let place_idx = self.place_stats_view_place;

        let mut open = self.show_place_stats_window;
        egui::Window::new(self.tr("Статистика", "Statistics"))
            .id(egui::Id::new("results_place_stats_window"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                let place_name = self
                    .net
                    .places
                    .get(place_idx)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("P{}", place_idx + 1));

                ui.horizontal(|ui| {
                    ui.label(self.tr("Позиция", "Place"));
                    let mut selected_ordinal = available_places
                        .iter()
                        .position(|&idx| idx == place_idx)
                        .unwrap_or(0);
                    ui.add(
                        egui::DragValue::new(&mut selected_ordinal)
                            .range(0..=available_places.len().saturating_sub(1)),
                    );
                    self.place_stats_view_place = available_places[selected_ordinal];
                    ui.label(format!("P{}", self.place_stats_view_place + 1));
                    ui.separator();
                    ui.label(place_name);
                });

                let sampled = Self::sampled_indices(result.logs.len(), Self::MAX_PLOT_POINTS);
                let mut values = Vec::<f64>::with_capacity(sampled.len());
                let mut times = Vec::<f64>::with_capacity(sampled.len());
                for idx in sampled {
                    let entry = &result.logs[idx];
                    if let Some(value) = entry.marking.get(place_idx) {
                        values.push(*value as f64);
                        let t = if entry.time.is_finite() {
                            entry.time
                        } else {
                            idx as f64
                        };
                        times.push(t);
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
                let utilization = result
                    .place_load
                    .as_ref()
                    .and_then(|load| load.get(place_idx))
                    .and_then(|l| l.avg_over_capacity)
                    .map(|v| v * 100.0)
                    .unwrap_or(0.0);

                ui.horizontal(|ui| {
                    ui.label(format!("{} {:.3}", self.tr("Максимум", "Maximum"), max_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), max_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Минимум", "Minimum"), min_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), min_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Среднее", "Average"), avg));
                    ui.label(format!(
                        "{} {:.3}%",
                        self.tr("Утилизация", "Utilization"),
                        utilization
                    ));
                });

                if let Some(place) = self.net.places.get(place_idx) {
                    ui.horizontal(|ui| {
                        let mut markers_total = place.stats.markers_total;
                        let mut markers_input = place.stats.markers_input;
                        let mut markers_output = place.stats.markers_output;
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(&mut markers_total, self.tr("Общая", "Total")),
                        );
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(
                                &mut markers_input,
                                self.tr("На входе", "On input"),
                            ),
                        );
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(
                                &mut markers_output,
                                self.tr("На выходе", "On output"),
                            ),
                        );
                    });
                }

                let desired_size = egui::Vec2::new(ui.available_width(), 320.0);
                let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));

                let x_min = times.first().copied().unwrap_or(0.0);
                let mut x_max = times.last().copied().unwrap_or(1.0);
                if x_max <= x_min {
                    x_max = x_min + (values.len().max(1) as f64);
                }
                let y_min = 0.0;
                let mut y_max = max_v.max(1.0);
                if y_max <= y_min {
                    y_max = y_min + 1.0;
                }

                for i in 1..10 {
                    let x = rect.left() + rect.width() * (i as f32 / 10.0);
                    painter.line_segment(
                        [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                        Stroke::new(0.5, Color32::LIGHT_GRAY),
                    );
                }

                let to_screen = |x: f64, y: f64| -> Pos2 {
                    let xr = ((x - x_min) / (x_max - x_min)).clamp(0.0, 1.0) as f32;
                    let yr = ((y - y_min) / (y_max - y_min)).clamp(0.0, 1.0) as f32;
                    Pos2::new(
                        rect.left() + xr * rect.width(),
                        rect.bottom() - yr * rect.height(),
                    )
                };

                let mut points = Vec::with_capacity(values.len());
                for (x, y) in times.iter().zip(values.iter()) {
                    points.push(to_screen(*x, *y));
                }
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(points, Stroke::new(1.5, Color32::BLUE)));
                }

                painter.text(
                    Pos2::new(rect.left() + 4.0, rect.top() + 4.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.0}", y_max),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
                painter.text(
                    Pos2::new(rect.left() + 4.0, rect.bottom() - 4.0),
                    egui::Align2::LEFT_BOTTOM,
                    "0",
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
            });

        self.show_place_stats_window = open;
    }
}
