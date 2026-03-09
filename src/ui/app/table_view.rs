use super::*;

use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::scroll_utils;

impl PetriApp {
    fn scroll_area_rows(
        ui: &mut egui::Ui,
        id: egui::Id,
        row_count: usize,
        row_height: f32,
        max_height: f32,
        mut add_rows: impl FnMut(&mut egui::Ui, std::ops::Range<usize>),
    ) {
        egui::ScrollArea::vertical()
            .id_source(id)
            .max_height(max_height)
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show_rows(ui, row_height, row_count, |ui, rows| add_rows(ui, rows));
    }

    fn draw_collapsible_section<R>(
        ui: &mut egui::Ui,
        id: impl std::hash::Hash,
        title: impl Into<egui::WidgetText>,
        default_open: bool,
        top_spacing: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        show_collapsible_property_section(
            ui,
            PropertySectionConfig::new(id)
                .label(title)
                .default_open(default_open)
                .top_spacing(top_spacing),
            add_contents,
        )
    }

    fn draw_fixed_cell(ui: &mut egui::Ui, width: f32, text: impl Into<egui::WidgetText>) {
        ui.add_sized([width, 0.0], egui::Label::new(text));
    }

    /// Draw the network structure editor (vectors and matrices).
    ///
    /// This view was previously built with a series of separators and labels followed by
    /// scrollable tables.  To unify the look and feel with other collapsible sections in the
    /// application, each logical section (vector or matrix) is now wrapped in a
    /// `show_collapsible_property_section` call.  These sections share the same framing and
    /// hidden scroll bar behaviour as property windows throughout the UI.  The bodies of
    /// sections still use `scroll_area_rows` for row virtualization when rendering large
    /// tables, so performance remains the same.  Import buttons for matrices are moved
    /// inside the collapsible body.
    pub(super) fn draw_table_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Структура сети");
        ui.horizontal(|ui| {
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
        let row_label_w = 64.0;
        let cell_w = 54.0;
        let import_csv_label = self.tr("Импорт CSV", "Import CSV");

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

        egui::ScrollArea::both().show(ui, |ui| {
            let _ = Self::draw_collapsible_section(
                ui,
                "m0_section",
                self.tr("Вектор начальной маркировки (M0)", "Initial marking vector (M0)"),
                true,
                0.0,
                |ui: &mut egui::Ui| {
                    Self::scroll_area_rows(
                        ui,
                        egui::Id::new("m0_grid_scroll"),
                        self.net.places.len(),
                        row_h,
                        vector_scroll_height,
                        |ui, rows| {
                            egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                                for i in rows {
                                    Self::draw_fixed_cell(ui, row_label_w, format!("P{}", i + 1));
                                    ui.add_sized(
                                        [cell_w, 0.0],
                                        egui::DragValue::new(&mut self.net.tables.m0[i]).range(0..=u32::MAX),
                                    );
                                    ui.end_row();
                                }
                            });
                        },
                    );
                },
            );

            let _ = Self::draw_collapsible_section(
                ui,
                "mo_section",
                self.tr("Вектор максимальных емкостей (Mo)", "Max capacities vector (Mo)"),
                false,
                6.0,
                |ui: &mut egui::Ui| {
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
                                    Self::draw_fixed_cell(ui, row_label_w, format!("P{}", i + 1));
                                    if ui
                                        .add_sized(
                                            [cell_w, 0.0],
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
                },
            );

            let _ = Self::draw_collapsible_section(
                ui,
                "mz_section",
                self.tr("Вектор временных задержек в позициях (Mz)", "Delay vector (Mz)"),
                false,
                6.0,
                |ui: &mut egui::Ui| {
                    Self::scroll_area_rows(
                        ui,
                        egui::Id::new("mz_grid_scroll"),
                        self.net.places.len(),
                        row_h,
                        vector_scroll_height,
                        |ui, rows| {
                            egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                                for i in rows {
                                    Self::draw_fixed_cell(ui, row_label_w, format!("P{}", i + 1));
                                    ui.add_sized(
                                        [cell_w * 1.5, 0.0],
                                        egui::DragValue::new(&mut self.net.tables.mz[i])
                                            .speed(0.1)
                                            .range(0.0..=10_000.0),
                                    );
                                    ui.end_row();
                                }
                            });
                        },
                    );
                },
            );

            let _ = Self::draw_collapsible_section(
                ui,
                "mpr_section",
                self.tr("Вектор приоритетов переходов (Mpr)", "Transition priority vector (Mpr)"),
                false,
                6.0,
                |ui: &mut egui::Ui| {
                    Self::scroll_area_rows(
                        ui,
                        egui::Id::new("mpr_grid_scroll"),
                        self.net.transitions.len(),
                        row_h,
                        vector_scroll_height,
                        |ui, rows| {
                            egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                                for t in rows {
                                    Self::draw_fixed_cell(ui, row_label_w, format!("T{}", t + 1));
                                    ui.add_sized(
                                        [cell_w * 1.5, 0.0],
                                        egui::DragValue::new(&mut self.net.tables.mpr[t]).speed(1),
                                    );
                                    ui.end_row();
                                }
                            });
                        },
                    );
                },
            );

            let mut matrices_changed = false;

            let mut pre_changed = false;
            let _ = Self::draw_collapsible_section(
                ui,
                "pre_section",
                self.tr("Матрица инцидентности Pre", "Incidence matrix Pre"),
                false,
                6.0,
                |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button(import_csv_label.clone()).clicked() {
                            self.import_matrix_csv(MatrixCsvTarget::Pre);
                        }
                    });
                    Self::scroll_area_rows(
                        ui,
                        egui::Id::new("pre_grid_scroll"),
                        self.net.places.len() + 1,
                        row_h,
                        matrix_scroll_height,
                        |ui, rows| {
                            egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                                for row in rows {
                                    if row == 0 {
                                        Self::draw_fixed_cell(ui, row_label_w, "");
                                        for t in 0..self.net.transitions.len() {
                                            Self::draw_fixed_cell(ui, cell_w, format!("T{}", t + 1));
                                        }
                                    } else {
                                        let p = row - 1;
                                        Self::draw_fixed_cell(ui, row_label_w, format!("P{}", p + 1));
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
                                    }
                                    ui.end_row();
                                }
                            });
                        },
                    );
                },
            );
            matrices_changed |= pre_changed;

            let mut post_changed = false;
            let _ = Self::draw_collapsible_section(
                ui,
                "post_section",
                self.tr("Матрица инцидентности Post", "Incidence matrix Post"),
                false,
                6.0,
                |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button(import_csv_label.clone()).clicked() {
                            self.import_matrix_csv(MatrixCsvTarget::Post);
                        }
                    });
                    Self::scroll_area_rows(
                        ui,
                        egui::Id::new("post_grid_scroll"),
                        self.net.places.len() + 1,
                        row_h,
                        matrix_scroll_height,
                        |ui, rows| {
                            egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                                for row in rows {
                                    if row == 0 {
                                        Self::draw_fixed_cell(ui, row_label_w, "");
                                        for t in 0..self.net.transitions.len() {
                                            Self::draw_fixed_cell(ui, cell_w, format!("T{}", t + 1));
                                        }
                                    } else {
                                        let p = row - 1;
                                        Self::draw_fixed_cell(ui, row_label_w, format!("P{}", p + 1));
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
                                    }
                                    ui.end_row();
                                }
                            });
                        },
                    );
                },
            );
            matrices_changed |= post_changed;

            let mut inhibitor_changed = false;
            let _ = Self::draw_collapsible_section(
                ui,
                "inhibitor_section",
                self.tr("Матрица ингибиторных дуг", "Inhibitor matrix"),
                false,
                6.0,
                |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button(import_csv_label.clone()).clicked() {
                            self.import_matrix_csv(MatrixCsvTarget::Inhibitor);
                        }
                    });
                    Self::scroll_area_rows(
                        ui,
                        egui::Id::new("inh_grid_scroll"),
                        self.net.places.len() + 1,
                        row_h,
                        matrix_scroll_height,
                        |ui, rows| {
                            egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                                for row in rows {
                                    if row == 0 {
                                        Self::draw_fixed_cell(ui, row_label_w, "");
                                        for t in 0..self.net.transitions.len() {
                                            Self::draw_fixed_cell(ui, cell_w, format!("T{}", t + 1));
                                        }
                                    } else {
                                        let p = row - 1;
                                        Self::draw_fixed_cell(ui, row_label_w, format!("P{}", p + 1));
                                        for t in 0..self.net.transitions.len() {
                                            inhibitor_changed |= ui
                                                .add_sized(
                                                    [cell_w, 0.0],
                                                    egui::DragValue::new(&mut self.net.tables.inhibitor[p][t])
                                                        .range(0..=u32::MAX)
                                                        .speed(1),
                                                )
                                                .changed();
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                        },
                    );
                },
            );
            matrices_changed |= inhibitor_changed;

            if matrices_changed {
                self.net.rebuild_arcs_from_matrices();
            }
        });
    }

    /// Draw the simulation parameters dialog.
    pub(super) fn draw_sim_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_sim_params;
        let mut close_now = false;
        egui::Window::new(self.tr("Параметры симуляции", "Simulation Parameters"))
            .open(&mut open)
            .resizable(true)
            .default_size(egui::vec2(420.0, 520.0))
            .min_size(egui::vec2(360.0, 320.0))
            .show(ctx, |ui| {
                let mut corrected_inputs = false;
                let pass_limit_label = self.tr("Лимит срабатываний", "Fire count limit");
                ui.checkbox(&mut self.sim_params.use_pass_limit, pass_limit_label);
                ui.add_enabled(
                    self.sim_params.use_pass_limit,
                    egui::DragValue::new(&mut self.sim_params.pass_limit).range(0..=u64::MAX),
                );
                corrected_inputs |= sanitize_u64(&mut self.sim_params.pass_limit, 0, 1_000_000);
                let time_limit_label = self.tr("Лимит времени (сек)", "Time limit (sec)");
                ui.checkbox(&mut self.sim_params.use_time_limit, time_limit_label);
                ui.add_enabled(
                    self.sim_params.use_time_limit,
                    egui::DragValue::new(&mut self.sim_params.time_limit)
                        .range(0.0..=1_000_000.0)
                        .speed(1.0),
                );
                corrected_inputs |= sanitize_f64(&mut self.sim_params.time_limit, 0.0, 1_000_000.0);
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
                    self.calculate_markov_model();
                    self.refresh_debug_animation_state();
                    self.debug_step = 0;
                    self.sync_debug_animation_for_step();
                    self.debug_playing = false;
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

    /// Draw the simulation results window with collapsible sections.
    ///
    /// The original implementation displayed all results in a single vertically scrolling area
    /// with separators and labels.  To make the interface consistent with other parts of
    /// the application, each major section (log table, marker statistics, flows and load)
    /// is now wrapped in a collapsible property section.  Within each section the
    /// original virtualization logic is preserved to ensure good performance on large
    /// datasets.  Heights of the scrollable areas are computed dynamically based on
    /// the available space so that sections can grow on larger windows while still
    /// respecting a predefined maximum.
    pub(super) fn draw_results(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.sim_result.clone() {
            let mut open = self.show_results;
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .min_size(egui::vec2(960.0, 540.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("results_window_scroll")
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
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
                                .filter_map(|(idx, place)| place.stats.any_enabled().then_some(idx))
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

                            let time_col = 72.0;
                            let place_col = 48.0;

                            let _ = Self::draw_collapsible_section(
                                ui,
                                "results_log_section",
                                self.tr("Журнал (таблица)", "Log (table)"),
                                true,
                                0.0,
                                |ui: &mut egui::Ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button(self.tr("Экспорт CSV", "Export CSV")).clicked() {
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
                                                            self.tr("Ошибка экспорта CSV", "CSV export error"),
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    });

                                    egui::ScrollArea::horizontal().show(ui, |ui| {
                                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                        egui::Grid::new("sim_log_grid_header").striped(true).show(ui, |ui| {
                                            Self::draw_fixed_cell(ui, time_col, self.tr("Время", "Time"));
                                            for (p, _) in self.net.places.iter().enumerate() {
                                                Self::draw_fixed_cell(ui, place_col, format!("P{}", p + 1));
                                            }
                                            ui.end_row();
                                        });

                                        let visible_log_indices = Self::debug_visible_log_indices(&result);
                                        scroll_utils::show_virtualized_rows(
                                            ui,
                                            "sim_log_grid_scroll",
                                            320.0,
                                            row_h,
                                            visible_log_indices.len(),
                                            |ui: &mut egui::Ui, row_idx: usize| {
                                                let entry = &result.logs[visible_log_indices[row_idx]];
                                                ui.horizontal(|ui| {
                                                    Self::draw_fixed_cell(ui, time_col, format!("{:.3}", entry.time));
                                                    for token in &entry.marking {
                                                        Self::draw_fixed_cell(ui, place_col, token.to_string());
                                                    }
                                                });
                                            },
                                        );
                                    });
                                },
                            );

                            if let Some(stats) = &result.place_stats {
                                let any_place_stats_selected =
                                    self.net.places.iter().any(|p| p.stats.any_enabled());
                                let show_all_places_in_stats = !any_place_stats_selected;
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
                                        if show_all_places_in_stats || selected { Some(p) } else { None }
                                    })
                                    .collect();

                                if !rows.is_empty() {
                                    let _ = Self::draw_collapsible_section(
                                        ui,
                                        "results_marker_stats_section",
                                        self.tr("Статистика маркеров (min/max/avg)", "Token statistics (min/max/avg)"),
                                        true,
                                        6.0,
                                        |ui: &mut egui::Ui| {
                                            let c1=84.0; let c2=72.0; let c3=72.0; let c4=72.0;
                                            egui::Grid::new("stats_grid_header").striped(true).show(ui, |ui| {
                                                Self::draw_fixed_cell(ui, c1, self.tr("Позиция", "Place"));
                                                Self::draw_fixed_cell(ui, c2, "Min");
                                                Self::draw_fixed_cell(ui, c3, "Max");
                                                Self::draw_fixed_cell(ui, c4, "Avg");
                                                ui.end_row();
                                            });
                                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "stats_grid_scroll",
                                                180.0,
                                                row_h,
                                                rows.len(),
                                                |ui: &mut egui::Ui, row_idx: usize| {
                                                    let p = rows[row_idx];
                                                    let st = &stats[p];
                                                    ui.horizontal(|ui| {
                                                        Self::draw_fixed_cell(ui, c1, format!("P{}", p + 1));
                                                        Self::draw_fixed_cell(ui, c2, st.min.to_string());
                                                        Self::draw_fixed_cell(ui, c3, st.max.to_string());
                                                        Self::draw_fixed_cell(ui, c4, format!("{:.3}", st.avg));
                                                    });
                                                },
                                            );
                                        },
                                    );
                                }
                            }

                            if let Some(flow) = &result.place_flow {
                                let any_place_stats_selected = self
                                    .net
                                    .places
                                    .iter()
                                    .any(|p| p.stats.markers_input || p.stats.markers_output);
                                let show_all_places_in_stats = !any_place_stats_selected;
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
                                        if show_all_places_in_stats || selected { Some(p) } else { None }
                                    })
                                    .collect();

                                if !rows.is_empty() {
                                    let _ = Self::draw_collapsible_section(
                                        ui,
                                        "results_flow_section",
                                        self.tr("Потоки (вход/выход)", "Flows (in/out)"),
                                        true,
                                        6.0,
                                        |ui: &mut egui::Ui| {
                                            let c1=84.0; let c2=72.0; let c3=72.0;
                                            egui::Grid::new("flow_grid_header").striped(true).show(ui, |ui| {
                                                Self::draw_fixed_cell(ui, c1, self.tr("Позиция", "Place"));
                                                Self::draw_fixed_cell(ui, c2, self.tr("Вход", "In"));
                                                Self::draw_fixed_cell(ui, c3, self.tr("Выход", "Out"));
                                                ui.end_row();
                                            });
                                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "flow_grid_scroll",
                                                180.0,
                                                row_h,
                                                rows.len(),
                                                |ui: &mut egui::Ui, row_idx: usize| {
                                                    let p = rows[row_idx];
                                                    let st = &flow[p];
                                                    ui.horizontal(|ui| {
                                                        Self::draw_fixed_cell(ui, c1, format!("P{}", p + 1));
                                                        Self::draw_fixed_cell(ui, c2, st.in_tokens.to_string());
                                                        Self::draw_fixed_cell(ui, c3, st.out_tokens.to_string());
                                                    });
                                                },
                                            );
                                        },
                                    );
                                }
                            }

                            if let Some(load) = &result.place_load {
                                let any_place_stats_selected = self
                                    .net
                                    .places
                                    .iter()
                                    .any(|p| p.stats.load_total || p.stats.load_input || p.stats.load_output);
                                let show_all_places_in_stats = !any_place_stats_selected;
                                let rows: Vec<usize> = load
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(p, _)| {
                                        let selected = self
                                            .net
                                            .places
                                            .get(p)
                                            .map(|pl| pl.stats.load_total || pl.stats.load_input || pl.stats.load_output)
                                            .unwrap_or(false);
                                        if show_all_places_in_stats || selected { Some(p) } else { None }
                                    })
                                    .collect();

                                if !rows.is_empty() {
                                    let _ = Self::draw_collapsible_section(
                                        ui,
                                        "results_load_section",
                                        self.tr("Загруженность", "Load"),
                                        true,
                                        6.0,
                                        |ui: &mut egui::Ui| {
                                            let c1=84.0; let c2=84.0; let c3=84.0; let c4=84.0;
                                            egui::Grid::new("load_grid_header").striped(true).show(ui, |ui| {
                                                Self::draw_fixed_cell(ui, c1, self.tr("Позиция", "Place"));
                                                Self::draw_fixed_cell(ui, c2, self.tr("Общая", "Total"));
                                                Self::draw_fixed_cell(ui, c3, self.tr("Вход", "Input"));
                                                Self::draw_fixed_cell(ui, c4, self.tr("Выход", "Output"));
                                                ui.end_row();
                                            });
                                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "load_grid_scroll",
                                                180.0,
                                                row_h,
                                                rows.len(),
                                                |ui: &mut egui::Ui, row_idx: usize| {
                                                    let p = rows[row_idx];
                                                    let st = &load[p];
                                                    ui.horizontal(|ui| {
                                                        Self::draw_fixed_cell(ui, c1, format!("P{}", p + 1));
                                                        Self::draw_fixed_cell(ui, c2, match st.avg_over_capacity { Some(v) => format!("{:.3}", v), None => "N/A".to_string() });
                                                        Self::draw_fixed_cell(ui, c3, match st.in_rate { Some(v) => format!("{:.3}", v), None => "N/A".to_string() });
                                                        Self::draw_fixed_cell(ui, c4, match st.out_rate { Some(v) => format!("{:.3}", v), None => "N/A".to_string() });
                                                    });
                                                },
                                            );
                                        },
                                    );
                                }
                            }
                        });
                });
            self.show_results = open;
        }
    }

    pub(in crate::ui::app) fn draw_place_statistics_window(&mut self, ctx: &egui::Context) {
        if !self.show_place_stats_window || self.net.places.is_empty() {
            self.show_place_stats_window = false;
            return;
        }
        let title = self
            .tr("Статистика по месту", "Place statistics")
            .to_string();
        let close_label = self.tr("Закрыть", "Close").to_string();
        let place_index = self
            .place_stats_view_place
            .min(self.net.places.len().saturating_sub(1));
        let place_name = format!("P{}", place_index + 1);
        let current_tokens = self.net.tables.m0.get(place_index).copied().unwrap_or(0);
        let sim_result = self.sim_result.clone();
        let position_label = self.tr("Позиция", "Place").to_string();
        let tokens_label = self.tr("Маркеры", "Tokens").to_string();
        let latest_time_label = self.tr("Последнее время", "Latest time").to_string();
        let final_marking_label = self.tr("Итоговые маркеры", "Final marking").to_string();
        let no_data_label = self
            .tr(
                "Данные симуляции отсутствуют",
                "Simulation data unavailable",
            )
            .to_string();
        let mut open = self.show_place_stats_window;
        let mut close_requested = false;
        egui::Window::new(title)
            .open(&mut open)
            .resizable(true)
            .default_size(egui::vec2(320.0, 200.0))
            .show(ctx, |ui| {
                ui.label(format!("{}: {}", position_label, place_name));
                ui.label(format!("{}: {}", tokens_label, current_tokens));
                ui.separator();
                if let Some(result) = sim_result.as_ref() {
                    let latest_marking =
                        result.final_marking.get(place_index).copied().unwrap_or(0);
                    ui.label(format!("{}: {:.3}", latest_time_label, result.sim_time));
                    ui.label(format!("{}: {}", final_marking_label, latest_marking));
                } else {
                    ui.label(no_data_label);
                }
                if ui.button(&close_label).clicked() {
                    close_requested = true;
                }
            });
        if close_requested {
            open = false;
        }
        self.show_place_stats_window = open;
    }
}
