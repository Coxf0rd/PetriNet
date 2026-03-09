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
        // Predefined maximum heights for vector and matrix sections.  Actual height
        // is computed dynamically based on the available vertical space.  Each
        // section will use up to these values but can be smaller if the window is
        // short.  This provides a responsive layout while avoiding sections
        // consuming the entire viewport.
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
                self.net
                    .set_counts(p_count.max(0) as usize, t_count.max(0) as usize);
            }
        });

        // Pre-compute translation strings for vector and matrix section labels.  We call
        // `self.tr` here to avoid capturing `self` inside the UI closures below, which
        // would lead to borrow checker conflicts.  These strings will be moved into
        // the collapsible section definitions.
        let label_m0 = self.tr(
            "Вектор начальной маркировки (M0)",
            "Initial marking vector (M0)",
        );
        let label_mo = self.tr(
            "Вектор максимальных емкостей (Mo)",
            "Max capacities vector (Mo)",
        );
        let label_mz = self.tr(
            "Вектор временных задержек в позициях (Mz)",
            "Delay vector (Mz)",
        );
        let label_mpr = self.tr(
            "Вектор приоритетов переходов (Mpr)",
            "Transition priority vector (Mpr)",
        );
        let label_pre = self.tr("Матрица инцидентности Pre", "Incidence matrix Pre");
        let label_post = self.tr("Матрица инцидентности Post", "Incidence matrix Post");
        let label_inh = self.tr("Матрица ингибиторных дуг", "Inhibitor matrix");
        let label_import_csv = self.tr("Импорт CSV", "Import CSV");

        // Render vector sections if enabled.
        if self.show_struct_vectors {
            // Initial marking vector (M0)
            {
                let count = self.net.places.len();
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("m0")
                        .default_open(true)
                        .label(label_m0.clone()),
                    |ui: &mut egui::Ui| {
                        // Compute dynamic height relative to available space (max 60% up to predefined vector height).
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(vector_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("m0_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                                    for i in rows {
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("P{}", i + 1)),
                                        );
                                        ui.add_sized(
                                            [42.0 * 1.4, 0.0],
                                            egui::DragValue::new(&mut self.net.tables.m0[i])
                                                .range(0..=u32::MAX),
                                        );
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    },
                );
            }
            // Maximum capacity vector (Mo)
            {
                let count = self.net.places.len();
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("mo")
                        .default_open(true)
                        .label(label_mo.clone()),
                    |ui: &mut egui::Ui| {
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(vector_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("mo_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("mo_grid").striped(true).show(ui, |ui| {
                                    for i in rows {
                                        let mut cap = self.net.tables.mo[i].unwrap_or(0);
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("P{}", i + 1)),
                                        );
                                        if ui
                                            .add_sized(
                                                [42.0 * 1.4, 0.0],
                                                egui::DragValue::new(&mut cap).range(0..=u32::MAX),
                                            )
                                            .changed()
                                        {
                                            self.net.tables.mo[i] =
                                                if cap == 0 { None } else { Some(cap) };
                                        }
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    },
                );
            }
            // Delay vector in places (Mz)
            {
                let count = self.net.places.len();
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("mz")
                        .default_open(true)
                        .label(label_mz.clone()),
                    |ui: &mut egui::Ui| {
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(vector_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("mz_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                                    for i in rows {
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("P{}", i + 1)),
                                        );
                                        ui.add_sized(
                                            [42.0 * 1.8, 0.0],
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
            }
            // Priority vector for transitions (Mpr)
            {
                let count = self.net.transitions.len();
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("mpr")
                        .default_open(true)
                        .label(label_mpr.clone()),
                    |ui: &mut egui::Ui| {
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(vector_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("mpr_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                                    for t in rows {
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("T{}", t + 1)),
                                        );
                                        ui.add_sized(
                                            [42.0 * 1.8, 0.0],
                                            egui::DragValue::new(&mut self.net.tables.mpr[t])
                                                .speed(1),
                                        );
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    },
                );
            }
        }
        // Track whether any of the matrices were modified during drawing.
        let mut matrices_changed = false;
        // Pre matrix section
        if self.show_struct_pre {
            {
                let count = self.net.places.len();
                let mut pre_changed_local = false;
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("pre_matrix")
                        .default_open(false)
                        .label(label_pre.clone()),
                    |ui: &mut egui::Ui| {
                        // Import button
                        ui.horizontal(|ui| {
                            if ui.small_button(label_import_csv.clone()).clicked() {
                                self.import_matrix_csv(MatrixCsvTarget::Pre);
                            }
                        });
                        // Dynamic height (60% of avail height, capped by matrix_scroll_height)
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(matrix_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("pre_matrix_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                                    ui.add_sized([46.0, 0.0], egui::Label::new(""));
                                    for t in 0..self.net.transitions.len() {
                                        ui.add_sized(
                                            [42.0, 0.0],
                                            egui::Label::new(format!("T{}", t + 1)),
                                        );
                                    }
                                    ui.end_row();
                                    for p in rows.clone() {
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("P{}", p + 1)),
                                        );
                                        for t in 0..self.net.transitions.len() {
                                            let cell_changed = ui
                                                .add_sized(
                                                    [42.0, 0.0],
                                                    egui::DragValue::new(
                                                        &mut self.net.tables.pre[p][t],
                                                    )
                                                    .range(0..=u32::MAX)
                                                    .speed(1),
                                                )
                                                .changed();
                                            if cell_changed {
                                                pre_changed_local = true;
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    },
                );
                matrices_changed |= pre_changed_local;
            }
        }
        // Post matrix section
        if self.show_struct_post {
            {
                let count = self.net.places.len();
                let mut post_changed_local = false;
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("post_matrix")
                        .default_open(false)
                        .label(label_post.clone()),
                    |ui: &mut egui::Ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button(label_import_csv.clone()).clicked() {
                                self.import_matrix_csv(MatrixCsvTarget::Post);
                            }
                        });
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(matrix_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("post_matrix_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                                    ui.add_sized([46.0, 0.0], egui::Label::new(""));
                                    for t in 0..self.net.transitions.len() {
                                        ui.add_sized(
                                            [42.0, 0.0],
                                            egui::Label::new(format!("T{}", t + 1)),
                                        );
                                    }
                                    ui.end_row();
                                    for p in rows.clone() {
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("P{}", p + 1)),
                                        );
                                        for t in 0..self.net.transitions.len() {
                                            let cell_changed = ui
                                                .add_sized(
                                                    [42.0, 0.0],
                                                    egui::DragValue::new(
                                                        &mut self.net.tables.post[p][t],
                                                    )
                                                    .range(0..=u32::MAX)
                                                    .speed(1),
                                                )
                                                .changed();
                                            if cell_changed {
                                                post_changed_local = true;
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    },
                );
                matrices_changed |= post_changed_local;
            }
        }
        // Inhibitor matrix section
        if self.show_struct_inhibitor {
            {
                let count = self.net.places.len();
                let mut inh_changed_local = false;
                show_collapsible_property_section(
                    ui,
                    PropertySectionConfig::new("inh_matrix")
                        .default_open(false)
                        .label(label_inh.clone()),
                    |ui: &mut egui::Ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button(label_import_csv.clone()).clicked() {
                                self.import_matrix_csv(MatrixCsvTarget::Inhibitor);
                            }
                        });
                        let avail_height = ui.available_height();
                        let dynamic_height = (avail_height * 0.6).min(matrix_scroll_height);
                        Self::scroll_area_rows(
                            ui,
                            egui::Id::new("inh_matrix_grid_scroll"),
                            count,
                            row_h,
                            dynamic_height,
                            |ui: &mut egui::Ui, rows: std::ops::Range<usize>| {
                                ui.set_min_width(0.0);
                                egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                                    ui.add_sized([46.0, 0.0], egui::Label::new(""));
                                    for t in 0..self.net.transitions.len() {
                                        ui.add_sized(
                                            [42.0, 0.0],
                                            egui::Label::new(format!("T{}", t + 1)),
                                        );
                                    }
                                    ui.end_row();
                                    for p in rows.clone() {
                                        ui.add_sized(
                                            [46.0, 0.0],
                                            egui::Label::new(format!("P{}", p + 1)),
                                        );
                                        for t in 0..self.net.transitions.len() {
                                            let cell_changed = ui
                                                .add_sized(
                                                    [42.0, 0.0],
                                                    egui::DragValue::new(
                                                        &mut self.net.tables.inhibitor[p][t],
                                                    )
                                                    .range(0..=u32::MAX)
                                                    .speed(1),
                                                )
                                                .changed();
                                            if cell_changed {
                                                inh_changed_local = true;
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    },
                );
                matrices_changed |= inh_changed_local;
            }
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
                .show(ctx, |ui| {
                    // Top-level vertical scroll area for the window.  This hides the scroll bar
                    // by default; scroll bars on inner lists remain visible on hover via
                    // scroll_utils.
                    egui::ScrollArea::vertical()
                        .id_source("results_window_scroll")
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            // Summary information about the simulation.
                            let cycle_time = if result.fired_count > 0 {
                                Some(result.sim_time / result.fired_count as f64)
                            } else {
                                None
                            };
                            ui.label(match cycle_time {
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
                            // Log (table) section
                            show_collapsible_property_section(
                                ui,
                                PropertySectionConfig::new("results_log_section")
                                    .default_open(true)
                                    .label(self.tr("Журнал (таблица)", "Log (table)")),
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
                                                            self.tr(
                                                                "Журнал экспортирован",
                                                                "Log exported"
                                                            ),
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
                                        let row_h =
                                            ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                        // Render the header row once outside of virtualization.
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
                                        // Visible log indices for virtualization (precomputed)
                                        let visible_log_indices =
                                            Self::debug_visible_log_indices(&result);
                                        // Dynamically compute the maximum height for the log table.  Allow the
                                        // table to occupy up to 60% of the available height but never exceed
                                        // 320 px.  This provides a responsive layout while preventing a
                                        // single section from consuming the entire window.
                                        let avail_height = ui.available_height();
                                        let dynamic_height = (avail_height * 0.6).min(320.0);
                                        scroll_utils::show_virtualized_rows(
                                            ui,
                                            "sim_log_grid_scroll",
                                            dynamic_height,
                                            row_h,
                                            visible_log_indices.len(),
                                            |ui, row_idx| {
                                                let entry =
                                                    &result.logs[visible_log_indices[row_idx]];
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
                                        PropertySectionConfig::new("results_marker_stats_section")
                                            .default_open(true)
                                            .label(self.tr(
                                                "Статистика маркеров (min/max/avg)",
                                                "Token statistics (min/max/avg)",
                                            )),
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
                                            let row_h =
                                                ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            // Dynamically compute height: up to 40% of available height but at most 180 px.
                                            let avail_height = ui.available_height();
                                            let dynamic_height = (avail_height * 0.4).min(180.0);
                                            // Virtualize the rows using our scroll utility.  The scroll bar
                                            // appears on hover and the height adapts to the available space.
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "stats_grid_scroll",
                                                dynamic_height,
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
                                            .map(|pl| {
                                                pl.stats.markers_input || pl.stats.markers_output
                                            })
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
                                        PropertySectionConfig::new("results_flow_section")
                                            .default_open(true)
                                            .label(
                                                self.tr("Потоки (вход/выход)", "Flows (in/out)"),
                                            ),
                                        |ui| {
                                            egui::Grid::new("flow_grid_header").striped(true).show(
                                                ui,
                                                |ui| {
                                                    ui.label(self.tr("Позиция", "Place"));
                                                    ui.label(self.tr("Вход", "In"));
                                                    ui.label(self.tr("Выход", "Out"));
                                                    ui.end_row();
                                                },
                                            );
                                            let row_h =
                                                ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            // Dynamically compute height: up to 40% of available height but at most 180 px.
                                            let avail_height = ui.available_height();
                                            let dynamic_height = (avail_height * 0.4).min(180.0);
                                            // Virtualize the rows using our scroll utility.  Height adapts to available space.
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "flow_grid_scroll",
                                                dynamic_height,
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
                                let any_place_stats_selected = self.net.places.iter().any(|p| {
                                    p.stats.load_total || p.stats.load_input || p.stats.load_output
                                });
                                let show_all_places_in_stats = !any_place_stats_selected;
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
                                if !rows.is_empty() {
                                    show_collapsible_property_section(
                                        ui,
                                        PropertySectionConfig::new("results_load_section")
                                            .default_open(true)
                                            .label(self.tr("Загруженность", "Load")),
                                        |ui| {
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
                                            let row_h =
                                                ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                            // Dynamically compute height: up to 40% of available height but at most 180 px.
                                            let avail_height = ui.available_height();
                                            let dynamic_height = (avail_height * 0.4).min(180.0);
                                            // Virtualize the rows using our scroll utility.  Height adapts to available space.
                                            scroll_utils::show_virtualized_rows(
                                                ui,
                                                "load_grid_scroll",
                                                dynamic_height,
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
