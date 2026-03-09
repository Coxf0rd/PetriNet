use super::*;
use egui::{Color32, RichText};
use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::property_window::{show_property_window, PropertyWindowConfig};
use crate::ui::scroll_utils;

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        if !self.show_markov_window {
            return;
        }

        let mut open = self.show_markov_window;

        show_property_window(
            ctx,
            self.tr("Марковская модель", "Markov model"),
            &mut open,
            PropertyWindowConfig::new("markov_window")
                .default_size(egui::vec2(520.0, 520.0))
                .min_size(egui::vec2(360.0, 280.0))
                .resizable(true),
            |ui| {
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
                            .checkbox(
                                &mut self.markov_model_enabled,
                                markov_checkbox_label.as_ref(),
                            )
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

            },
        );

        self.show_markov_window = open;
    }

    fn markov_section_height(ui: &egui::Ui, preferred: f32, min_height: f32) -> f32 {
        let available = ui.available_height().max(min_height);
        let adaptive = available * 0.45;
        adaptive.clamp(min_height, preferred)
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
            PropertySectionConfig::new("markov_stationary_section")
                .label(self.tr("Стационарное распределение", "Stationary distribution"))
                .default_open(false),
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
            PropertySectionConfig::new("markov_state_graph_section")
                .label(self.tr("Граф состояний", "State graph"))
                .default_open(false),
            |ui| {
                self.draw_markov_state_graph(ui, chain);
            },
        );

        let _ = show_collapsible_property_section(
            ui,
            PropertySectionConfig::new("markov_highlight_section")
                .label(self.tr("Отображение марковской метки", "Markov highlight display"))
                .default_open(false),
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
        let max_height = Self::markov_section_height(ui, 320.0, 140.0);
        let state_col = 76.0;
        let prob_col = 84.0;

        scroll_utils::show_list_with_scroll(
            ui,
            "markov_stationary_distribution",
            max_height,
            |ui: &mut egui::Ui| {
                for (idx, value) in stationary.iter().copied().enumerate() {
                    let row_available = ui.available_width();
                    let marking_width = Self::markov_marking_column_width(row_available);
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.add_sized([state_col, 0.0], egui::Label::new(format!("S{}", idx + 1)));
                        ui.allocate_ui(Vec2::new(marking_width, 0.0), |ui: &mut egui::Ui| {
                            self.draw_state_marking_table(ui, &chain.states[idx][..], idx);
                        });
                        ui.add_sized([prob_col, 0.0], egui::Label::new(format!("{:.6}", value)));
                    });
                    ui.add_space(6.0);
                }
            },
        );
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        // Compute the available width and derive the transitions column width for the
        // header.  This width will be recomputed for each row to adapt to the
        // scroll area's width.
        let header_available = ui.available_width();
        let header_transitions_width = Self::markov_transitions_column_width(header_available);

        let state_col = 76.0;
        ui.horizontal(|ui| {
            ui.add_sized(
                [state_col, 0.0],
                egui::Label::new(RichText::new(self.tr("Состояние", "State")).strong()),
            );
            ui.allocate_ui(Vec2::new(header_transitions_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Переходы", "Transitions")).strong());
            });
        });

        let max_height = Self::markov_section_height(ui, 280.0, 140.0);

        // Use a scroll area with a visible scroll bar when needed.  Within the
        // scroll area, recompute the transitions column width based on the
        // current available width to ensure alignment with the header.
        scroll_utils::show_list_with_scroll(
            ui,
            "markov_state_graph",
            max_height,
            |ui: &mut egui::Ui| {
                if chain.transitions.is_empty() {
                    ui.label(self.tr("Переходов не найдено", "No transitions detected"));
                    return;
                }

                for (idx, edges) in chain.transitions.iter().enumerate() {
                    // Determine the width for the transitions column based on the row's width.
                    let row_available = ui.available_width();
                    let transitions_width = Self::markov_transitions_column_width(row_available);
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.add_sized([state_col, 0.0], egui::Label::new(format!("S{}", idx + 1)));
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
            },
        );
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

        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());

        let max_height = Self::markov_section_height(ui, 280.0, 140.0);

        // Use a scroll area with a visible scroll bar when needed for the place
        // highlight distribution.  We compute widths inside each row to adapt
        // to the current available width of the scroll area.
        scroll_utils::show_list_with_scroll(
            ui,
            "markov_place_distribution",
            max_height,
            |ui: &mut egui::Ui| {
                for (place_idx, place) in &markov_highlight_places {
                    ui.group(|ui: &mut egui::Ui| {
                        let place_label = if place.name.is_empty() {
                            format!("P{}", place.id)
                        } else {
                            format!("P{} ({})", place.id, place.name)
                        };

                        ui.label(format!(
                            "{}: {}",
                            self.tr("Позиция", "Place"),
                            place_label,
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
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.add_sized(
                                    [140.0, 0.0],
                                    egui::Label::new(
                                        RichText::new(self.tr("Число маркеров", "Token count")).strong(),
                                    ),
                                );
                                ui.add_sized(
                                    [84.0, 0.0],
                                    egui::Label::new(
                                        RichText::new(self.tr("Вероятность", "Probability")).strong(),
                                    ),
                                );
                            });
                            for (count, prob) in distribution.iter() {
                                ui.horizontal(|ui: &mut egui::Ui| {
                                    ui.add_sized(
                                        [140.0, 0.0],
                                        egui::Label::new(count.to_string()),
                                    );
                                    ui.add_sized(
                                        [84.0, 0.0],
                                        egui::Label::new(format!("{:.2}%", prob * 100.0)),
                                    );
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
            },
        );
    }

    fn draw_state_marking_table(&self, ui: &mut egui::Ui, marking: &[u32], state_idx: usize) {
        const COLUMNS: usize = 2;

        if marking.is_empty() {
            ui.label("—");
            return;
        }

        let rows = (marking.len() + COLUMNS - 1) / COLUMNS;
        let place_header = self.tr("Позиция", "Place");
        let tokens_header = self.tr("Маркеры", "Tokens");

        egui::Grid::new(format!(
            "state_marking_summary_{}_{}",
            state_idx,
            marking.len()
        ))
        .striped(true)
        .spacing([6.0, 2.0])
        .min_col_width(48.0)
        .show(ui, |ui| {
            for _ in 0..COLUMNS {
                ui.label(RichText::new(place_header.as_ref()).strong());
                ui.label(RichText::new(tokens_header.as_ref()).strong());
            }
            ui.end_row();

            for row in 0..rows {
                for col in 0..COLUMNS {
                    let idx = row + col * rows;
                    if idx < marking.len() {
                        ui.label(self.markov_place_display_name(idx));
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

    fn markov_place_display_name(&self, place_idx: usize) -> String {
        self.net
            .places
            .get(place_idx)
            .map(|place| {
                if place.name.is_empty() {
                    format!("P{}", place.id)
                } else {
                    format!("P{} ({})", place.id, place.name)
                }
            })
            .unwrap_or_else(|| format!("P{}", place_idx + 1))
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
