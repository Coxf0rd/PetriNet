use super::*;
use egui::Color32;

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_markov_window;
        egui::Window::new(self.tr("Марковская модель", "Markov model"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("markov_window"))
            .open(&mut open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
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
                            if self.markov_model_enabled {
                                if let Some(chain) = &self.markov_model {
                                    let stationary = chain.stationary.as_ref();
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
                                    ui.label(
                                        self.tr(
                                            "Стационарное распределение",
                                            "Stationary distribution",
                                        ),
                                    );
                                    egui::ScrollArea::vertical()
                                        .id_source("markov_stationary_distribution")
                                        .max_height(260.0)
                                        .show(ui, |ui| {
                                            if let Some(stationary) = stationary {
                                                egui::Grid::new("markov_states")
                                                    .striped(true)
                                                    .show(ui, |ui| {
                                                        ui.label(self.tr("Состояние", "State"));
                                                        ui.label("π");
                                                        ui.end_row();
                                                        let rows = chain.state_count().min(32);
                                                        for idx in 0..rows {
                                                            ui.label(Self::format_marking(
                                                                &chain.states[idx],
                                                            ));
                                                            ui.label(format!(
                                                                "{:.6}",
                                                                stationary[idx]
                                                            ));
                                                            ui.end_row();
                                                        }
                                                        if chain.state_count() > rows {
                                                            ui.label(format!(
                                                                "... {} ...",
                                                                chain.state_count() - rows
                                                            ));
                                                            ui.label("");
                                                            ui.end_row();
                                                        }
                                                    });
                                            } else {
                                                ui.label(self.tr(
                                                    "Стационарное распределение не вычислено",
                                                    "Unable to compute stationary",
                                                ));
                                            }
                                        });
                                    ui.separator();
                                    ui.label(self.tr("Граф состояний", "State graph"));
                                    egui::ScrollArea::vertical()
                                        .id_source("markov_state_graph")
                                        .max_height(320.0)
                                        .show(ui, |ui| {
                                            let graph_width = ui.available_width();
                                            let has_transitions =
                                                chain.transitions.iter().any(|edges| !edges.is_empty());
                                            if has_transitions {
                                                egui::Grid::new("markov_state_graph_grid")
                                                    .striped(true)
                                                    .min_col_width(graph_width)
                                                    .show(ui, |ui| {
                                                        ui.label(self.tr("Состояние", "State"));
                                                        ui.label(self.tr("Переходы", "Transitions"));
                                                        ui.end_row();
                                                        for (idx, edges) in
                                                            chain.transitions.iter().enumerate()
                                                        {
                                                            ui.label(format!("S{}", idx + 1));
                                                            if edges.is_empty() {
                                                                ui.label(
                                                                    self.tr(
                                                                        "Переходов нет",
                                                                        "No transitions",
                                                                    ),
                                                                );
                                                            } else {
                                                                let total_rate: f64 = edges
                                                                    .iter()
                                                                    .map(|(_, rate)| *rate)
                                                                    .sum();
                                                                ui.vertical(|ui| {
                                                                    for (dest, rate) in edges {
                                                                        let prob = if total_rate > 0.0
                                                                        {
                                                                            (rate / total_rate)
                                                                                .clamp(0.0, 1.0)
                                                                        } else {
                                                                            0.0
                                                                        };
                                                                        ui.label(format!(
                                                                            "→ S{} ({:.2})",
                                                                            dest + 1,
                                                                            prob
                                                                        ));
                                                                    }
                                                                });
                                                            }
                                                            ui.end_row();
                                                        }
                                                    });
                                            } else {
                                                ui.label(self.tr(
                                                    "Переходов не найдено",
                                                    "No transitions detected",
                                                ));
                                            }
                                        });
                                    let markov_highlight_places = self
                                        .net
                                        .places
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, place)| place.markov_highlight)
                                        .collect::<Vec<_>>();
                                    if markov_highlight_places.is_empty() {
                                        ui.separator();
                                        ui.label(self.tr(
                                            "Отметьте марковскую метку в свойствах позиции, чтобы увидеть её отображение",
                                            "Enable the Markov highlight on a place to view its display",
                                        ));
                                    } else {
                                        ui.separator();
                                        ui.label(
                                            self.tr(
                                                "Отображение марковской метки",
                                                "Markov highlight display",
                                            ),
                                        );
                                        let expectation =
                                            Self::markov_expected_tokens(chain, self.net.places.len());
                                        egui::ScrollArea::vertical()
                                            .id_source("markov_place_distribution")
                                            .max_height(320.0)
                                            .show(ui, |ui| {
                                                for (place_idx, place) in
                                                    &markov_highlight_places
                                                {
                                                    ui.group(|ui| {
                                                        let place_label = if place.name.is_empty() {
                                                            format!("P{}", place.id)
                                                        } else {
                                                            place.name.clone()
                                                        };
                                                        ui.label(format!(
                                                            "{}: {} (P{})",
                                                            self.tr("РџРѕР·РёС†РёСЏ", "Place"),
                                                            place_label,
                                                            place.id
                                                        ));
                                                        if let Some(expected) = expectation
                                                            .as_ref()
                                                            .and_then(|values| {
                                                                values.get(*place_idx)
                                                            })
                                                        {
                                                            ui.label(format!(
                                                                "{}: {:.3}",
                                                                self.tr(
                                                                    "Ожидаемое число маркеров",
                                                                    "Expected tokens"
                                                                ),
                                                                expected
                                                            ));
                                                        }
                                                        let distribution = Self::markov_tokens_distribution(
                                                            chain, *place_idx,
                                                        );
                                                        if !distribution.is_empty() {
                                                            for (count, prob) in distribution.iter() {
                                                                ui.horizontal(|ui| {
                                                                    ui.label(format!(
                                                                        "{} {}",
                                                                        count,
                                                                        self.tr("маркеров", "tokens")
                                                                    ));
                                                                    ui.label(format!(
                                                                        "{:.2}%",
                                                                        prob * 100.0
                                                                    ));
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
                                } else {
                                    ui.label(self.tr("Постройте модель", "Build the model"));
                                }
                            } else {
                                ui.label(self.tr(
                                    "Включите флажок выше, чтобы увидеть марковскую модель",
                                    "Toggle the checkbox above to display the Markov model",
                                ));
                            }
                        });
                    });
            });
        self.show_markov_window = open;
    }
}
