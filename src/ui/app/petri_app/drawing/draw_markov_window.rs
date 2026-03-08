use super::*;
use egui::{Color32, RichText, Vec2};
// Import the property section helpers to unify collapsible sections across the UI.
use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::scroll_utils;

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_markov_window;
        let viewport = ctx.available_rect();
        let max_height = (viewport.height() - 120.0).max(360.0);
        let max_width = (viewport.width() - 120.0).max(360.0);

        egui::Window::new(self.tr("Марковская модель", "Markov model"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("markov_window"))
            .default_size(Vec2::new(520.0, 520.0))
            .min_size(Vec2::new(360.0, 360.0))
            .max_size(Vec2::new(max_width, max_height))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let simulation_ready = self.sim_result.is_some();
                    let mut toggle_changed = false;
                    let markov_checkbox_label =
                        self.tr("включить марковскую модель", "Enable Markov model");
                    let simulation_hint = self.tr(
                        "Сначала запустите симуляцию, чтобы включить марковскую модель",
                        "Run a simulation first to enable the model",
                    );

                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(simulation_ready, |ui| {
                            if ui
                                .checkbox(&mut self.markov_model_enabled, markov_checkbox_label.as_ref())
                                .changed()
                            {
                                toggle_changed = true;
                            }
                        });

                        if !simulation_ready {
                            ui.colored_label(
                                Color32::from_rgb(190, 40, 40),
                                simulation_hint.as_ref(),
                            );
                        }
                    });

                    if toggle_changed {
                        for place in &mut self.net.places {
                            place.show_markov_model = self.markov_model_enabled;
                        }

                        if self.markov_model_enabled {
                            self.calculate_markov_model();
                        } else {
                            self.markov_place_arcs.clear();
                        }
                    }

                    ui.separator();
                    ui.add_space(6.0);

                    if let Some(chain) = &self.markov_model {
                        self.draw_markov_chain_summary(ui, chain);
                    } else {
                        ui.label(self.tr("Постройте модель", "Build the model"));
                    }

                    if !self.markov_model_enabled {
                        ui.separator();
                        ui.label(self.tr(
                            "Включите флажок выше, чтобы увидеть марковскую модель",
                            "Toggle the checkbox above to display the Markov model",
                        ));
                    }
                });
            });

        self.show_markov_window = open;
    }

    fn draw_markov_chain_summary(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        let stationary = chain.stationary.as_ref().map(|values| values.as_slice());

        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}{}",
                self.tr("Состояний", "States"),
                chain.state_count(),
                if chain.limit_reached {
                    format!(" ({})", self.tr("лимит", "limit reached"))
                } else {
                    String::new()
                }
            ));

            ui.label(format!(
                "{}: {}",
                self.tr("Переходов", "Transitions"),
                chain
                    .transitions
                    .iter()
                    .map(|edges| edges.len())
                    .sum::<usize>()
            ));
        });

        ui.separator();

        // Replace the ad-hoc CollapsingHeader calls with the unified property
        // section helper.  Each section is identified by a unique ID so that
        // collapsed/expanded state persists across redraws.  We ignore the
        // optional return value because the contents are rendered for side
        // effects only.
        let _ = show_collapsible_property_section(
            ui,
            self.tr("Стационарное распределение", "Stationary distribution"),
            PropertySectionConfig::new("markov_stationary_section").default_open(false),
            |ui| {
                if let Some(stationary) = stationary {
                    self.draw_markov_stationary_grid(ui, chain, stationary);
                } else {
                    ui.label(self.tr(
                        "Стационарное распределение не вычислено",
                        "Unable to compute stationary",
                    ));
                }
            },
        );

        let _ = show_collapsible_property_section(
            ui,
            self.tr("Граф состояний", "State graph"),
            PropertySectionConfig::new("markov_state_graph_section").default_open(false),
            |ui| {
                self.draw_markov_state_graph(ui, chain);
            },
        );

        let _ = show_collapsible_property_section(
            ui,
            self.tr("Отображение марковской метки", "Markov highlight display"),
            PropertySectionConfig::new("markov_highlight_section").default_open(false),
            |ui| {
                self.draw_markov_highlight(ui, chain, stationary);
            },
        );
    }

    fn draw_markov_stationary_grid(
        &self,
        ui: &mut egui::Ui,
        chain: &MarkovChain,
        stationary: &[f64],
    ) {
        if chain.state_count() == 0 {
            ui.label(self.tr("Состояний не найдено", "No states found"));
            return;
        }
        // Calculate the available width for the header before rendering any rows.
        // This ensures that both the header and the rows share the same width.
        let header_available = ui.available_width();
        let header_marking_width = Self::markov_marking_column_width(header_available);

        // Draw the header row of the stationary distribution table.
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("Состояние", "State")).strong());
            ui.allocate_ui(Vec2::new(header_marking_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Маркировка", "Marking")).strong());
            });
            ui.label(RichText::new("π").strong());
        });

        // Determine a dynamic maximum height for the scroll area.  Use the
        // remaining available height, but enforce a sensible minimum of 360.0
        // points to avoid extremely small scroll areas.
        let mut max_height = ui.available_height();
        if max_height < 360.0 {
            max_height = 360.0;
        }

        // Row height is based on the body text style plus some padding.
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let row_count = stationary.len();

        // Virtualize the stationary distribution rows.  For each row, compute
        // the marking column width based on the row's available width.  This
        // ensures that the row contents align with the header even when the
        // scroll area's width changes due to scroll bars or margins.
        scroll_utils::show_virtualized_rows(
            ui,
            "markov_stationary_distribution",
            max_height,
            row_h,
            row_count,
            |ui: &mut egui::Ui, idx: usize| {
                let value = stationary[idx];
                // Determine current row width and compute marking column width.
                let row_available = ui.available_width();
                let marking_width = Self::markov_marking_column_width(row_available);
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(format!("S{}", idx + 1));
                    ui.allocate_ui(Vec2::new(marking_width, 0.0), |ui: &mut egui::Ui| {
                        self.draw_state_marking_table(ui, &chain.states[idx][..], idx);
                    });
                    ui.label(format!("{:.6}", value));
                });
                ui.add_space(6.0);
            },
        );
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        ui.label(self.tr("Граф состояний", "State graph"));

        // Compute the available width and derive the transitions column width for the
        // header.  This width will be recomputed for each row to adapt to the
        // scroll area's width.
        let header_available = ui.available_width();
        let header_transitions_width = Self::markov_transitions_column_width(header_available);

        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("Состояние", "State")).strong());
            ui.allocate_ui(Vec2::new(header_transitions_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Переходы", "Transitions")).strong());
            });
        });

        // Determine a dynamic maximum height for the state graph scroll area.
        let mut max_height = ui.available_height();
        if max_height < 320.0 {
            max_height = 320.0;
        }

        // Use a scroll area with a visible scroll bar when needed.  Within the
        // scroll area, recompute the transitions column width based on the
        // current available width to ensure alignment with the header.
        scroll_utils::show_list_with_scroll(ui, "markov_state_graph", max_height, |ui: &mut egui::Ui| {
            if chain.transitions.is_empty() {
                ui.label(self.tr("Переходов не найдено", "No transitions detected"));
                return;
            }

            for (idx, edges) in chain.transitions.iter().enumerate() {
                // Determine the width for the transitions column based on the row's width.
                let row_available = ui.available_width();
                let transitions_width = Self::markov_transitions_column_width(row_available);
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(format!("S{}", idx + 1));
                    ui.allocate_ui(Vec2::new(transitions_width, 0.0), |ui: &mut egui::Ui| {
                        if edges.is_empty() {
                            ui.label(self.tr("Переходов нет", "No transitions"));
                        } else {
                            let total_rate: f64 = edges.iter().map(|(_, rate)| *rate).sum();
                            ui.vertical(|ui: &mut egui::Ui| {
                                for (dest, rate) in edges {
                                    let prob = if total_rate > 0.0 {
                                        (rate / total_rate).clamp(0.0, 1.0)
                                    } else {
                                        0.0
                                    };
                                    ui.add_sized(
                                        [transitions_width, 0.0],
                                        egui::Label::new(format!(
                                            "→ S{} ({:.2})",
                                            dest + 1,
                                            prob
                                        ))
                                        .wrap(),
                                    );
                                }
                            });
                        }
                    });
                });
                ui.add_space(6.0);
            }
        });
    }

    fn draw_markov_highlight(
        &self,
        ui: &mut egui::Ui,
        chain: &MarkovChain,
        stationary: Option<&[f64]>,
    ) {
        let markov_highlight_places = self
            .net
            .places
            .iter()
            .enumerate()
            .filter(|(_, place)| place.markov_highlight)
            .collect::<Vec<_>>();

        if markov_highlight_places.is_empty() {
            ui.label(self.tr(
                "Отметьте марковскую метку в свойствах позиции, чтобы увидеть её отображение",
                "Enable the Markov highlight on a place to view its display",
            ));
            return;
        }

        ui.label(self.tr("Отображение марковской метки", "Markov highlight display"));

        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());

        // Compute a dynamic maximum height for the place distribution scroll area.
        let mut max_height = ui.available_height();
        if max_height < 320.0 {
            max_height = 320.0;
        }

        // Use a scroll area with a visible scroll bar when needed for the place
        // highlight distribution.  We compute widths inside each row to adapt
        // to the current available width of the scroll area.
        scroll_utils::show_list_with_scroll(ui, "markov_place_distribution", max_height, |ui: &mut egui::Ui| {
            for (place_idx, place) in &markov_highlight_places {
                ui.group(|ui: &mut egui::Ui| {
                    let place_label = if place.name.is_empty() {
                        format!("P{}", place.id)
                    } else {
                        place.name.clone()
                    };

                    ui.label(format!(
                        "{}: {} (P{})",
                        self.tr("Позиция", "Place"),
                        place_label,
                        place.id
                    ));

                    if let Some(expected) = expectation
                        .as_ref()
                        .and_then(|values| values.get(*place_idx))
                    {
                        ui.label(format!(
                            "{}: {:.3}",
                            self.tr("Ожидаемое число маркеров", "Expected tokens"),
                            expected
                        ));
                    }

                    let distribution = Self::markov_tokens_distribution(chain, *place_idx);

                    if !distribution.is_empty() {
                        for (count, prob) in distribution.iter() {
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.label(format!(
                                    "{} {}",
                                    count,
                                    self.tr("маркеров", "tokens")
                                ));
                                ui.label(format!("{:.2}%", prob * 100.0));
                            });
                        }
                    } else if stationary.is_some() {
                        ui.label(self.tr(
                            "Для этой позиции состояния не найдены",
                            "No states found for this place",
                        ));
                    } else {
                        ui.label(self.tr(
                            "Стационарное распределение недоступно",
                            "Stationary distribution unavailable",
                        ));
                    }
                });

                ui.add_space(4.0);
            }
        });
    }

    fn draw_state_marking_table(&self, ui: &mut egui::Ui, marking: &[u32], state_idx: usize) {
        const COLUMNS: usize = 2;

        if marking.is_empty() {
            ui.label("—");
            return;
        }

        let rows = (marking.len() + COLUMNS - 1) / COLUMNS;

        egui::Grid::new(format!(
            "state_marking_summary_{}_{}",
            state_idx,
            marking.len()
        ))
        .striped(true)
        .spacing([6.0, 2.0])
        .show(ui, |ui| {
            for row in 0..rows {
                for col in 0..COLUMNS {
                    let idx = row + col * rows;
                    if idx < marking.len() {
                        ui.label(format!("P{}", idx + 1));
                        ui.label(marking[idx].to_string());
                    } else {
                        ui.label(" ");
                        ui.label(" ");
                    }
                }
                ui.end_row();
            }
        });
    }

    fn markov_marking_column_width(available: f32) -> f32 {
        const MIN_WIDTH: f32 = 120.0;
        let max_width = (available * 0.7).max(MIN_WIDTH);
        let width = (available * 0.55).clamp(MIN_WIDTH, max_width);
        width.min(available)
    }

    fn markov_transitions_column_width(available: f32) -> f32 {
        const MIN_WIDTH: f32 = 180.0;
        let max_width = (available * 0.65).max(MIN_WIDTH);
        let width = (available * 0.6).clamp(MIN_WIDTH, max_width);
        width.min(available)
    }
}