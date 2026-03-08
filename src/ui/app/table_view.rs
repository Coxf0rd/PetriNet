use super::*;

use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::scroll_utils;

impl PetriApp {
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
        // Top controls for hiding the structure view or toggling full screen.
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
        // Row height used for virtualized grids.
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        // Default heights for vector and matrix sections.  These constants limit the
        // maximum height of each scroll area, preventing a single section from consuming
        // the entire viewport.  The user can scroll within each table to view more data.
        let vector_scroll_height = 220.0;
        let matrix_scroll_height = 320.0;

        // Toggles for which structures to display and controls for changing counts.
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
                self
                    .net
                    .set_counts(p_count.max(0) as usize, t_count.max(0) as usize);
            }
        });

        // Helper closure to build a vector section.  Each vector is wrapped in a collapsible
        // property section.  The body uses row virtualization to render only the visible
        // portion of the vector.  A unique id is passed to the section and grid to ensure
        // consistent state across frames.
        let mut make_vector_section = |ui: &mut egui::Ui,
                                       id_str: &str,
                                       label_ru: &str,
                                       label_en: &str,
                                       row_count: usize,
                                       f: &mut dyn FnMut(&mut egui::Ui, std::ops::Range<usize>)| {
            let config = PropertySectionConfig {
                id: egui::Id::new(id_str),
                label: self.tr(label_ru, label_en),
                // Default all vector sections to open; users can collapse them if needed.
                default_open: true,
            };
            // Use the collapse helper to render the section.
            show_collapsible_property_section(ui, config, |ui| {
                // Within the body, render a virtualized grid of rows.  We limit the
                // maximum height to the vector_scroll_height constant; additional rows
                // can be viewed via scrolling.
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new(format!("{}_grid_scroll", id_str)),
                    row_count,
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        // Reset minimum width so the grid spans the available width.
                        ui.set_min_width(0.0);
                        f(ui, rows);
                    },
                );
            });
        };

        // Helper closure to build a matrix section.  Each matrix is wrapped in a collapsible
        // property section.  The body shows an optional import CSV button and a virtualized
        // grid.  The closure returns a boolean indicating whether any cell was changed.
        let mut make_matrix_section = |ui: &mut egui::Ui,
                                       id_str: &str,
                                       label_ru: &str,
                                       label_en: &str,
                                       row_count: usize,
                                       import_target: Option<MatrixCsvTarget>,
                                       mut grid_fn: Box<dyn FnMut(&mut egui::Ui, std::ops::Range<usize>) -> bool>| {
            let config = PropertySectionConfig {
                id: egui::Id::new(id_str),
                label: self.tr(label_ru, label_en),
                default_open: false,
            };
            show_collapsible_property_section(ui, config, |ui| {
                let mut changed = false;
                // Optional import button.
                if let Some(target) = import_target {
                    ui.horizontal(|ui| {
                        if ui
                            .small_button(self.tr("Импорт CSV", "Import CSV"))
                            .clicked()
                        {
                            self.import_matrix_csv(target);
                        }
                    });
                }
                // Render the matrix grid with virtualization.  Limit height to
                // matrix_scroll_height to prevent the section from consuming the entire
                // viewport.  A hidden scroll bar within the section allows scrolling.
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new(format!("{}_grid_scroll", id_str)),
                    row_count,
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        ui.set_min_width(0.0);
                        // Delegate rendering to the provided grid function which returns
                        // whether any cell was modified.
                        changed = grid_fn(ui, rows) || changed;
                    },
                );
                changed
            })
        };

        // Render vector sections if enabled.
        if self.show_struct_vectors {
            // Initial marking vector (M0)
            make_vector_section(ui, "m0", "Вектор начальной маркировки (M0)", "Initial marking vector (M0)", self.net.places.len(), &mut |ui, rows| {
                egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                    for i in rows {
                        ui.add_sized([46.0, 0.0], egui::Label::new(format!("P{}", i + 1)));
                        ui.add_sized([
                            42.0 * 1.4,
                            0.0,
                        ], egui::DragValue::new(&mut self.net.tables.m0[i]).range(0..=u32::MAX));
                        ui.end_row();
                    }
                });
            });
            // Maximum capacity vector (Mo)
            make_vector_section(ui, "mo", "Вектор максимальных емкостей (Mo)", "Max capacities vector (Mo)", self.net.places.len(), &mut |ui, rows| {
                egui::Grid::new("mo_grid").striped(true).show(ui, |ui| {
                    for i in rows {
                        let mut cap = self.net.tables.mo[i].unwrap_or(0);
                        ui.add_sized([46.0, 0.0], egui::Label::new(format!("P{}", i + 1)));
                        if ui
                            .add_sized([
                                42.0 * 1.4,
                                0.0,
                            ], egui::DragValue::new(&mut cap).range(0..=u32::MAX))
                            .changed()
                        {
                            self.net.tables.mo[i] = if cap == 0 { None } else { Some(cap) };
                        }
                        ui.end_row();
                    }
                });
            });
            // Delay vector in places (Mz)
            make_vector_section(ui, "mz", "Вектор временных задержек в позициях (Mz)", "Delay vector (Mz)", self.net.places.len(), &mut |ui, rows| {
                egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                    for i in rows {
                        ui.add_sized([46.0, 0.0], egui::Label::new(format!("P{}", i + 1)));
                        ui.add_sized([
                            42.0 * 1.8,
                            0.0,
                        ], egui::DragValue::new(&mut self.net.tables.mz[i]).speed(0.1).range(0.0..=10_000.0));
                        ui.end_row();
                    }
                });
            });
            // Priority vector for transitions (Mpr)
            make_vector_section(ui, "mpr", "Вектор приоритетов переходов (Mpr)", "Transition priority vector (Mpr)", self.net.transitions.len(), &mut |ui, rows| {
                egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                    for t in rows {
                        ui.add_sized([46.0, 0.0], egui::Label::new(format!("T{}", t + 1)));
                        ui.add_sized([
                            42.0 * 1.8,
                            0.0,
                        ], egui::DragValue::new(&mut self.net.tables.mpr[t]).speed(1));
                        ui.end_row();
                    }
                });
            });
        }
        // Track whether any of the matrices were modified during drawing.
        let mut matrices_changed = false;
        // Pre matrix section
        if self.show_struct_pre {
            let pre_changed = make_matrix_section(
                ui,
                "pre_matrix",
                "Матрица инцидентности Pre",
                "Incidence matrix Pre",
                self.net.places.len(),
                Some(MatrixCsvTarget::Pre),
                Box::new(|ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                    let mut changed = false;
                    egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                        ui.add_sized([46.0, 0.0], egui::Label::new(""));
                        for t in 0..self.net.transitions.len() {
                            ui.add_sized([42.0, 0.0], egui::Label::new(format!("T{}", t + 1)));
                        }
                        ui.end_row();
                        for p in rows {
                            ui.add_sized([46.0, 0.0], egui::Label::new(format!("P{}", p + 1)));
                            for t in 0..self.net.transitions.len() {
                                changed |= ui
                                    .add_sized([
                                        42.0,
                                        0.0,
                                    ], egui::DragValue::new(&mut self.net.tables.pre[p][t]).range(0..=u32::MAX).speed(1))
                                    .changed();
                            }
                            ui.end_row();
                        }
                    });
                    changed
                }),
            );
            matrices_changed |= pre_changed;
        }
        // Post matrix section
        if self.show_struct_post {
            let post_changed = make_matrix_section(
                ui,
                "post_matrix",
                "Матрица инцидентности Post",
                "Incidence matrix Post",
                self.net.places.len(),
                Some(MatrixCsvTarget::Post),
                Box::new(|ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                    let mut changed = false;
                    egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                        ui.add_sized([46.0, 0.0], egui::Label::new(""));
                        for t in 0..self.net.transitions.len() {
                            ui.add_sized([42.0, 0.0], egui::Label::new(format!("T{}", t + 1)));
                        }
                        ui.end_row();
                        for p in rows {
                            ui.add_sized([46.0, 0.0], egui::Label::new(format!("P{}", p + 1)));
                            for t in 0..self.net.transitions.len() {
                                changed |= ui
                                    .add_sized([
                                        42.0,
                                        0.0,
                                    ], egui::DragValue::new(&mut self.net.tables.post[p][t]).range(0..=u32::MAX).speed(1))
                                    .changed();
                            }
                            ui.end_row();
                        }
                    });
                    changed
                }),
            );
            matrices_changed |= post_changed;
        }
        // Inhibitor matrix section
        if self.show_struct_inhibitor {
            let inh_changed = make_matrix_section(
                ui,
                "inh_matrix",
                "Матрица ингибиторных дуг",
                "Inhibitor matrix",
                self.net.places.len(),
                Some(MatrixCsvTarget::Inhibitor),
                Box::new(|ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                    let mut changed = false;
                    egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                        ui.add_sized([46.0, 0.0], egui::Label::new(""));
                        for t in 0..self.net.transitions.len() {
                            ui.add_sized([42.0, 0.0], egui::Label::new(format!("T{}", t + 1)));
                        }
                        ui.end_row();
                        for p in rows {
                            ui.add_sized([46.0, 0.0], egui::Label::new(format!("P{}", p + 1)));
                            for t in 0..self.net.transitions.len() {
                                changed |= ui
                                    .add_sized([
                                        42.0,
                                        0.0,
                                    ], egui::DragValue::new(&mut self.net.tables.inhibitor[p][t]).range(0..=u32::MAX).speed(1))
                                    .changed();
                            }
                            ui.end_row();
                        }
                    });
                    changed
                }),
            );
            matrices_changed |= inh_changed;
        }
        // Rebuild arcs if any matrix changed.
        if matrices_changed {
            self.net.rebuild_arcs_from_matrices();
        }
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
    /// datasets.
    pub(super) fn draw_results(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.sim_result.clone() {
            let mut open = self.show_results;
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .show(ctx, |ui| {
                    // Top-level vertical scroll area for the window.  This hides the scroll bar
                    // by default; scroll bars on inner lists remain visible on hover via
                    // scroll_utils.
                    egui::ScrollArea::vertical()
                        .id_source("results_window_scroll")
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            // Summary information about the simulation.
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
                            // Link to detailed per-place statistics if available.
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
                            // Log (table) section
                            show_collapsible_property_section(
                                ui,
                                PropertySectionConfig {
                                    id: egui::Id::new("results_log_section"),
                                    label: self.tr("Журнал (таблица)", "Log (table)"),
                                    default_open: true,
                                },
                                |ui| {
                                    // Export CSV button
                                    ui.horizontal(|ui| {
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
                                                                "CSV export error",
                                                            ),
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    });
                                    // Horizontal scroll for the log header and rows.
                                    egui::ScrollArea::horizontal().show(ui, |ui| {
                                        // Determine the height of each row based on the current text style.
                                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                        // Render the header row once outside of virtualization.
                                        egui::Grid::new("sim_log_grid_header")
                                            .striped(true)
                                            .show(ui, |ui| {
                                                ui.label(self.tr("Время", "Time"));
                                                for (p, _) in self.net.places.iter().enumerate() {
                                                    ui.label(format!("P{}", p + 1));
                                                }
                                                ui.end_row();
                                            });
                                        // Visible log indices for virtualization (precomputed)
                                        let visible_log_indices = Self::debug_visible_log_indices(&result);
                                        // Use our scroll utility to virtualize the log rows.  The scroll bar
                                        // becomes visible on hover, matching other lists.  The height is
                                        // limited to 320 px to prevent the log from consuming the entire
                                        // window.
                                        scroll_utils::show_virtualized_rows(
                                            ui,
                                            "sim_log_grid_scroll",
                                            320.0,
                                            row_h,
                                            visible_log_indices.len(),
                                            |ui, row_idx| {
                                                let entry = &result.logs[visible_log_indices[row_idx]];
                                                // Render a single row.  We use a horizontal layout instead
                                                // of a Grid here because each row is drawn independently by
                                                // the virtualization helper.  This maintains alignment
                                                // between the time and token columns.
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("{:.3}", entry.time));
                                                    for token in &entry.marking {
                                                        ui.label(token.to_string());
                                                    }
                                                });
                                            },
                                        );
                                    });
                                },
                            );
                            // Marker statistics section
                            if let Some(stats) = &result.place_stats {
                                // Determine rows to display based on whether per-place stats are enabled.
                                let any_place_stats_selected = self
                                    .net
                                    .places
                                    .iter()
                                    .any(|p| p.stats.any_enabled());
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
                                        if show_all_places_in_stats || selected {
                                            Some(p)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if !rows.is_empty() {
                                    show_collapsible_property_section(
                                        ui,
                                        PropertySectionConfig {
                                            id: egui::Id::new("results_marker_stats_section"),
                                            label: self.tr(
                                                "Статистика маркеров (min/max/avg)",
                                                "Token statistics (min/max/avg)",
                                            ),
                                            default_open: true,
                                        },
                                        |ui| {
                                            // Header
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
                                            // Virtualize the rows using our scroll utility.  The scroll bar
                                            // appears on hover and the height is limited to 180 px, matching
                                            // previous behaviour.  Each row is rendered with a horizontal
                                            // layout to align the four columns.
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "stats_grid_scroll",
                                                180.0,
                                                row_h,
                                                rows.len(),
                                                |ui, row_idx| {
                                                    let p = rows[row_idx];
                                                    let st = &stats[p];
                                                    ui.horizontal(|ui| {
                                                        ui.label(format!("P{}", p + 1));
                                                        ui.label(st.min.to_string());
                                                        ui.label(st.max.to_string());
                                                        ui.label(format!("{:.3}", st.avg));
                                                    });
                                                },
                                            );
                                        },
                                    );
                                }
                            }
                            // Flow section
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
                                        if show_all_places_in_stats || selected {
                                            Some(p)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if !rows.is_empty() {
                                    show_collapsible_property_section(
                                        ui,
                                        PropertySectionConfig {
                                            id: egui::Id::new("results_flow_section"),
                                            label: self.tr("Потоки (вход/выход)", "Flows (in/out)"),
                                            default_open: true,
                                        },
                                        |ui| {
                                            egui::Grid::new("flow_grid_header")
                                                .striped(true)
                                                .show(ui, |ui| {
                                                    ui.label(self.tr("Позиция", "Place"));
                                                    ui.label(self.tr("Вход", "In"));
                                                    ui.label(self.tr("Выход", "Out"));
                                                    ui.end_row();
                                                });
                                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            // Virtualize the rows using our scroll utility.  Height
                                            // limited to 180 px and scroll bar appears on hover.  Each
                                            // row uses a horizontal layout for three columns: place,
                                            // input and output.
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "flow_grid_scroll",
                                                180.0,
                                                row_h,
                                                rows.len(),
                                                |ui, row_idx| {
                                                    let p = rows[row_idx];
                                                    let st = &flow[p];
                                                    ui.horizontal(|ui| {
                                                        ui.label(format!("P{}", p + 1));
                                                        ui.label(st.in_tokens.to_string());
                                                        ui.label(st.out_tokens.to_string());
                                                    });
                                                },
                                            );
                                        },
                                    );
                                }
                            }
                            // Load section
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
                                        if show_all_places_in_stats || selected {
                                            Some(p)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                if !rows.is_empty() {
                                    show_collapsible_property_section(
                                        ui,
                                        PropertySectionConfig {
                                            id: egui::Id::new("results_load_section"),
                                            label: self.tr("Загруженность", "Load"),
                                            default_open: true,
                                        },
                                        |ui| {
                                            egui::Grid::new("load_grid_header")
                                                .striped(true)
                                                .show(ui, |ui| {
                                                    ui.label(self.tr("Позиция", "Place"));
                                                    ui.label(self.tr("Общая", "Total"));
                                                    ui.label(self.tr("Вход", "Input"));
                                                    ui.label(self.tr("Выход", "Output"));
                                                    ui.end_row();
                                                });
                                            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            // Virtualize the rows using our scroll utility.  Height is
                                            // limited to 180 px and scroll bar appears on hover.  Each row
                                            // is rendered horizontally with four columns.
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "load_grid_scroll",
                                                180.0,
                                                row_h,
                                                rows.len(),
                                                |ui, row_idx| {
                                                    let p = rows[row_idx];
                                                    let st = &load[p];
                                                    ui.horizontal(|ui| {
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
}