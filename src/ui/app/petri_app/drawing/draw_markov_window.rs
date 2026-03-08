use super::*;
use egui::{scroll_area, Color32, RichText, Vec2};

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
                chain.transitions.iter().map(|edges| edges.len()).sum::<usize>()
            ));
        });

        ui.separator();

        egui::CollapsingHeader::new(self.tr(
            "Стационарное распределение",
            "Stationary distribution",
        ))
        .id_source("markov_stationary_section")
        .default_open(false)
        .show(ui, |ui| {
            if let Some(stationary) = stationary {
                self.draw_markov_stationary_grid(ui, chain, stationary);
            } else {
                ui.label(self.tr(
                    "Стационарное распределение не вычислено",
                    "Unable to compute stationary",
                ));
            }
        });

        egui::CollapsingHeader::new(self.tr("Граф состояний", "State graph"))
            .id_source("markov_state_graph_section")
            .default_open(false)
            .show(ui, |ui| {
                self.draw_markov_state_graph(ui, chain);
            });

        egui::CollapsingHeader::new(self.tr(
            "Отображение марковской метки",
            "Markov highlight display",
        ))
        .id_source("markov_highlight_section")
        .default_open(false)
        .show(ui, |ui| {
            self.draw_markov_highlight(ui, chain, stationary);
        });
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

        let available = ui.available_width();
        let marking_width = Self::markov_marking_column_width(available);

        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("Состояние", "State")).strong());
            ui.allocate_ui(Vec2::new(marking_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Маркировка", "Marking")).strong());
            });
            ui.label(RichText::new("π").strong());
        });

        egui::ScrollArea::vertical()
            .id_source("markov_stationary_distribution")
            .max_height(360.0)
            .auto_shrink([false, false])
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                for (idx, value) in stationary.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("S{}", idx + 1));
                        ui.allocate_ui(Vec2::new(marking_width, 0.0), |ui| {
                            self.draw_state_marking_table(ui, &chain.states[idx], idx);
                        });
                        ui.label(format!("{:.6}", value));
                    });
                    ui.add_space(6.0);
                }
            });
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        ui.label(self.tr("Граф состояний", "State graph"));

        let available = ui.available_width();
        let transitions_width = Self::markov_transitions_column_width(available);

        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("Состояние", "State")).strong());
            ui.allocate_ui(Vec2::new(transitions_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Переходы", "Transitions")).strong());
            });
        });

        egui::ScrollArea::vertical()
            .id_source("markov_state_graph")
            .max_height(320.0)
            .auto_shrink([false, false])
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                if chain.transitions.is_empty() {
                    ui.label(self.tr("Переходов не найдено", "No transitions detected"));
                    return;
                }

                for (idx, edges) in chain.transitions.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("S{}", idx + 1));
                        ui.allocate_ui(Vec2::new(transitions_width, 0.0), |ui| {
                            if edges.is_empty() {
                                ui.label(self.tr("Переходов нет", "No transitions"));
                            } else {
                                let total_rate: f64 = edges.iter().map(|(_, rate)| *rate).sum();

                                ui.vertical(|ui| {
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

        egui::ScrollArea::vertical()
            .id_source("markov_place_distribution")
            .max_height(320.0)
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                for (place_idx, place) in &markov_highlight_places {
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
                                ui.horizontal(|ui| {
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