use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_markov_window;
        egui::Window::new(self.tr("Марковская модель", "Markov model"))
            .id(egui::Id::new("markov_window"))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button(self.tr("Пересчитать", "Rebuild")).clicked() {
                        self.net.sanitize_values();
                        self.calculate_markov_model();
                    }
                    ui.label(self.tr(
                        "Получение предельного распределения решений колмогоровских уравнений",
                        "Stationary probabilities solve Kolmogorov equations",
                    ));
                });
                if let Some(chain) = &self.markov_model {
                    let stationary = chain.stationary.as_ref();
                    ui.label(format!(
                        "{}: {}{}",
                        self.tr("Состояний,", "States"),
                        chain.state_count(),
                        if chain.limit_reached {
                            format!(" ({})", self.tr("лимит,", "limit reached"))
                        } else {
                            String::new()
                        }
                    ));
                    let total_edges: usize = chain
                        .transitions
                        .iter()
                        .map(|edges| edges.len())
                        .sum::<usize>();
                    ui.label(format!(
                        "{}: {}",
                        self.tr("Переходы", "Transitions"),
                        total_edges
                    ));
                    if let Some(stationary) = stationary {
                        ui.label(self.tr("Стационарное распределение", "Stationary distribution"));
                        egui::ScrollArea::vertical()
                            .max_height(280.0)
                            .show(ui, |ui| {
                                egui::Grid::new("markov_states")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(self.tr("Состояние", "State"));
                                        ui.label(self.tr("π", "π"));
                                        ui.end_row();
                                        let rows = chain.state_count().min(32);
                                        for idx in 0..rows {
                                            ui.label(Self::format_marking(&chain.states[idx]));
                                            ui.label(format!("{:.6}", stationary[idx]));
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
                            });
                    } else {
                        ui.label(self.tr(
                            "Стационарное распределение не вычислено",
                            "Unable to compute stationary",
                        ));
                    }
                    ui.separator();
                    ui.label(self.tr("Граф состояний", "State graph"));
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .show(ui, |ui| {
                            let mut rows = 0;
                            let max_rows = 12;
                            for (idx, edges) in chain.transitions.iter().enumerate() {
                                if rows >= max_rows {
                                    break;
                                }
                                if edges.is_empty() {
                                    continue;
                                }
                                let sum: f64 = edges.iter().map(|(_, rate)| *rate).sum();
                                let transitions = edges
                                    .iter()
                                    .map(|(dest, rate)| {
                                        let prob = if sum > 0.0 {
                                            (rate / sum).clamp(0.0, 1.0)
                                        } else {
                                            0.0
                                        };
                                        format!("S{} ({:.2})", dest + 1, prob)
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                ui.horizontal(|ui| {
                                    ui.label(format!("S{} →", idx + 1));
                                    ui.label(transitions);
                                });
                                rows += 1;
                            }
                            if rows == 0 {
                                ui.label(
                                    self.tr("Переходов не найдено", "No transitions detected"),
                                );
                            } else if chain.state_count() > rows {
                                ui.label(format!(
                                    "... {} {} ...",
                                    chain.state_count() - rows,
                                    self.tr("состояний пропущено", "states skipped"),
                                ));
                            }
                        });
                    let markov_focus_places = self
                        .net
                        .places
                        .iter()
                        .enumerate()
                        .filter(|(_, place)| place.show_markov_model)
                        .collect::<Vec<_>>();
                    if !markov_focus_places.is_empty() {
                        ui.separator();
                        ui.label(
                            self.tr("Марковская модель по позициям", "Markov model per place"),
                        );
                        let expectation =
                            Self::markov_expected_tokens(chain, self.net.places.len());
                        for (place_idx, place) in markov_focus_places {
                            ui.group(|ui| {
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
                                    .and_then(|values| values.get(place_idx))
                                {
                                    ui.label(format!(
                                        "{}: {:.3}",
                                        self.tr("Ожидаемое число маркеров", "Expected tokens"),
                                        expected
                                    ));
                                }
                                let distribution =
                                    Self::markov_tokens_distribution(chain, place_idx);
                                if !distribution.is_empty() {
                                    let max_rows = 6;
                                    for (count, prob) in distribution.iter().take(max_rows) {
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "{} {}",
                                                count,
                                                self.tr("маркеров", "tokens")
                                            ));
                                            ui.label(format!("{:.2}%", prob * 100.0));
                                        });
                                    }
                                    if distribution.len() > max_rows {
                                        ui.label(format!(
                                            "... {} ...",
                                            distribution.len() - max_rows
                                        ));
                                    }
                                } else if stationary.is_some() {
                                    ui.label(self.tr(
                                        "Для позиции не найдено состояний",
                                        "No states found for this place",
                                    ));
                                } else {
                                    ui.label(self.tr(
                                        "Стационарное распределение не вычислено",
                                        "Stationary distribution unavailable",
                                    ));
                                }
                            });
                        }
                    }
                } else {
                    ui.label(self.tr("Постройте модель", "Build the model"));
                }
            });
        self.show_markov_window = open;
    }
}
