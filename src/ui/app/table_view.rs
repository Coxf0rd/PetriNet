use super::*;

use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::property_window::{show_property_window, PropertyWindowConfig};
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


    fn section_open(ctx: &egui::Context, id: impl std::hash::Hash, default_open: bool) -> bool {
        egui::collapsing_header::CollapsingState::load_with_default_open(
            ctx,
            egui::Id::new(id),
            default_open,
        )
        .is_open()
    }

    fn place_stats_points(
        result: &SimulationResult,
        place_index: usize,
        series: PlaceStatsSeries,
    ) -> Vec<(f64, f64)> {
        let mut points = Vec::with_capacity(result.logs.len());
        let mut cumulative_in = 0.0_f64;
        let mut cumulative_out = 0.0_f64;
        let mut prev = result.logs.first().and_then(|e| e.marking.get(place_index)).copied().unwrap_or(0);
        for entry in &result.logs {
            let current = entry.marking.get(place_index).copied().unwrap_or(0);
            let y = match series {
                PlaceStatsSeries::Total => current as f64,
                PlaceStatsSeries::Input => {
                    if current > prev { cumulative_in += (current - prev) as f64; }
                    cumulative_in
                }
                PlaceStatsSeries::Output => {
                    if prev > current { cumulative_out += (prev - current) as f64; }
                    cumulative_out
                }
            };
            points.push((entry.time, y));
            prev = current;
        }
        points
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
            let any_results_section_open = Self::section_open(ctx, "results_log_section", true)
                || Self::section_open(ctx, "results_marker_stats_section", true)
                || Self::section_open(ctx, "results_flow_section", true)
                || Self::section_open(ctx, "results_load_section", true);
            let results_min_size = if any_results_section_open {
                egui::vec2(960.0, 420.0)
            } else {
                egui::vec2(420.0, 180.0)
            };
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .min_size(results_min_size)
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
        let Some(result) = self.sim_result.clone() else {
            self.show_place_stats_window = false;
            return;
        };

        let place_index = self.place_stats_view_place.min(self.net.places.len().saturating_sub(1));
        let mut open = self.show_place_stats_window;
        let title = self.tr("Статистика", "Statistics");
        let position_label = self.tr("Позиция", "Place").to_string();
        let indicator_label = self.tr("Показатель", "Indicator").to_string();
        let sampled_label = self.tr("График сэмплирован", "Chart sampled").to_string();
        let total_tab = self.tr("Общая", "Total").to_string();
        let input_tab = self.tr("На входе", "Input").to_string();
        let output_tab = self.tr("На выходе", "Output").to_string();
        let show_grid_label = self.tr("Показать сетку", "Show grid").to_string();
        let scale_x_label = self.tr("Масштаб X", "Scale X").to_string();
        let pan_x_label = self.tr("Сдвиг X", "Pan X").to_string();

        let min_size = egui::vec2(720.0, 420.0);
        let default_size = min_size * 1.2;

        show_property_window(
            ctx,
            title,
            &mut open,
            PropertyWindowConfig::new("place_statistics_window")
                .default_size(default_size)
                .min_size(min_size),
            |ui: &mut egui::Ui| {
                let points = Self::place_stats_points(&result, place_index, self.place_stats_series);
                let x_min = points.first().map(|p| p.0).unwrap_or(0.0);
                let x_max = points.last().map(|p| p.0).unwrap_or(1.0).max(x_min + 1.0);
                let y_max = points.iter().map(|(_, y)| *y).fold(0.0_f64, f64::max).max(1.0);
                let avg = if points.is_empty() { 0.0 } else { points.iter().map(|(_, y)| *y).sum::<f64>() / points.len() as f64 };
                let y_min_val = points.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
                let y_min_val = if y_min_val.is_finite() { y_min_val } else { 0.0 };
                let utilization = if y_max > 0.0 { avg / y_max * 100.0 } else { 0.0 };

                let _ = Self::draw_collapsible_section(
                    ui,
                    "place_stats_controls_section",
                    self.tr("Параметры", "Parameters"),
                    true,
                    0.0,
                    |ui: &mut egui::Ui| {
                        ui.horizontal(|ui| {
                            ui.label(&position_label);
                            egui::ComboBox::from_id_source("place_stats_place_combo")
                                .selected_text(format!("P{}", place_index + 1))
                                .show_ui(ui, |ui| {
                                    for idx in 0..self.net.places.len() {
                                        ui.selectable_value(&mut self.place_stats_view_place, idx, format!("P{}", idx + 1));
                                    }
                                });
                            ui.label(format!("P{}", place_index + 1));
                            ui.label(match self.place_stats_series {
                                PlaceStatsSeries::Total => total_tab.clone(),
                                PlaceStatsSeries::Input => input_tab.clone(),
                                PlaceStatsSeries::Output => output_tab.clone(),
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label(&indicator_label);
                            ui.selectable_value(&mut self.place_stats_series, PlaceStatsSeries::Total, &total_tab);
                            ui.selectable_value(&mut self.place_stats_series, PlaceStatsSeries::Input, &input_tab);
                            ui.selectable_value(&mut self.place_stats_series, PlaceStatsSeries::Output, &output_tab);
                        });
                        ui.label(format!("{}: {} / {}", sampled_label, result.logs.len(), result.log_entries_total));
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {:.3}", self.tr("Максимум", "Maximum"), y_max));
                            ui.separator();
                            ui.label(format!("{} {:.3}", self.tr("Минимум", "Minimum"), y_min_val));
                            ui.separator();
                            ui.label(format!("{} {:.3}", self.tr("Среднее", "Average"), avg));
                            ui.separator();
                            ui.label(format!("{} {:.3}%", self.tr("Утилизация", "Utilization"), utilization));
                        });
                        ui.horizontal(|ui| {
                            ui.label(&scale_x_label);
                            ui.add(egui::Slider::new(&mut self.place_stats_zoom_x, 1.0..=20.0).show_value(false));
                            ui.add(egui::DragValue::new(&mut self.place_stats_zoom_x).range(1.0..=20.0).speed(0.1));
                            ui.label(&pan_x_label);
                            ui.add(egui::Slider::new(&mut self.place_stats_pan_x, 0.0..=1.0).show_value(false));
                            ui.add(egui::DragValue::new(&mut self.place_stats_pan_x).range(0.0..=1.0).speed(0.01));
                            ui.checkbox(&mut self.place_stats_show_grid, &show_grid_label);
                        });
                    },
                );

                let _ = Self::draw_collapsible_section(
                    ui,
                    "place_stats_graph_section",
                    self.tr("График", "Chart"),
                    true,
                    6.0,
                    |ui: &mut egui::Ui| {
                        let desired = egui::vec2(ui.available_width().max(200.0), ui.available_height().max(300.0));
                        let (response, painter) = ui.allocate_painter(desired, egui::Sense::hover());
                        let rect = response.rect.shrink2(egui::vec2(14.0, 10.0));
                        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

                        let full_span = (x_max - x_min).max(1.0);
                        let zoom = self.place_stats_zoom_x.max(1.0) as f64;
                        let visible_span = (full_span / zoom).max(full_span / 1000.0).min(full_span);
                        let max_pan = (full_span - visible_span).max(0.0);
                        let pan = (self.place_stats_pan_x.clamp(0.0, 1.0) as f64) * max_pan;
                        let visible_min_x = x_min + pan;
                        let visible_max_x = visible_min_x + visible_span;
                        let visible_y_max = y_max.max(1.0);

                        let to_screen = |x: f64, y: f64| -> egui::Pos2 {
                            let tx = if visible_span > 0.0 { ((x - visible_min_x) / visible_span).clamp(0.0, 1.0) } else { 0.0 };
                            let ty = (y / visible_y_max).clamp(0.0, 1.0);
                            egui::pos2(
                                rect.left() + rect.width() * tx as f32,
                                rect.bottom() - rect.height() * ty as f32,
                            )
                        };

                        if self.place_stats_show_grid {
                            for i in 1..10 {
                                let t = i as f32 / 10.0;
                                let x = egui::lerp(rect.left()..=rect.right(), t);
                                let y = egui::lerp(rect.top()..=rect.bottom(), t);
                                painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(1.0, egui::Color32::from_gray(220)));
                                painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(1.0, egui::Color32::from_gray(220)));
                            }
                        }

                        let visible_points: Vec<(f64, f64)> = points
                            .iter()
                            .copied()
                            .filter(|(x, _)| *x >= visible_min_x && *x <= visible_max_x)
                            .collect();
                        if visible_points.len() >= 2 {
                            let poly: Vec<egui::Pos2> = visible_points.iter().map(|(x, y)| to_screen(*x, *y)).collect();
                            painter.add(egui::Shape::line(poly, egui::Stroke::new(2.0, egui::Color32::BLUE)));

                            if let Some(mouse) = response.hover_pos() {
                                let mut best: Option<(f32, egui::Pos2, f64, f64)> = None;
                                for window in visible_points.windows(2) {
                                    let (x1,y1) = window[0];
                                    let (x2,y2) = window[1];
                                    let p1 = to_screen(x1,y1);
                                    let p2 = to_screen(x2,y2);
                                    let v = p2 - p1;
                                    let len2 = v.length_sq();
                                    if len2 <= 0.0 { continue; }
                                    let t = ((mouse - p1).dot(v) / len2).clamp(0.0, 1.0);
                                    let proj = p1 + v * t;
                                    let dist = proj.distance(mouse);
                                    let hx = x1 + (x2 - x1) * t as f64;
                                    let hy = y1 + (y2 - y1) * t as f64;
                                    if best.map(|b| dist < b.0).unwrap_or(true) {
                                        best = Some((dist, proj, hx, hy));
                                    }
                                }
                                if let Some((dist, pos, hx, hy)) = best {
                                    if dist <= 8.0 {
                                        painter.circle_filled(pos, 4.0, egui::Color32::WHITE);
                                        painter.circle_stroke(pos, 4.0, egui::Stroke::new(1.5, egui::Color32::BLUE));
                                        painter.text(
                                            pos + egui::vec2(8.0, 8.0),
                                            egui::Align2::LEFT_TOP,
                                            format!("X: {:.3}, Y: {:.3}", hx, hy),
                                            egui::TextStyle::Body.resolve(ui.style()),
                                            egui::Color32::BLACK,
                                        );
                                    }
                                }
                            }
                        }

                        painter.text(
                            egui::pos2(rect.center().x, rect.bottom() + 18.0),
                            egui::Align2::CENTER_CENTER,
                            self.tr("Ось X: Время/шаги", "X axis: Time/steps"),
                            egui::TextStyle::Body.resolve(ui.style()),
                            egui::Color32::GRAY,
                        );
                        ui.add_space(6.0);
                        ui.label(format!(
                            "{} X: [{:.3} .. {:.3}] | {} Y: {:.3}",
                            self.tr("Диапазон", "Range"),
                            visible_min_x,
                            visible_max_x,
                            self.tr("Максимум", "Maximum"),
                            visible_y_max
                        ));
                    },
                );
            },
        );
        self.show_place_stats_window = open;
    }
}
