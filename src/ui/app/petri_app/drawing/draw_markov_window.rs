use super::*;
use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::property_window::{show_property_window, PropertyWindowConfig};
use crate::ui::scroll_utils;
use egui::{Color32, WidgetText};

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
                        ui.colored_label(Color32::from_rgb(190, 40, 40), simulation_hint.as_ref());
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

        let (state_col, marking_col, prob_col) = Self::markov_stationary_column_widths();
        egui::Grid::new("markov_stationary_header")
            .striped(true)
            .show(ui, |ui| {
                Self::markov_draw_cell(ui, state_col, self.tr("Состояние", "State"));
                Self::markov_draw_cell(ui, marking_col, self.tr("Маркировка", "Marking"));
                Self::markov_draw_cell(ui, prob_col, self.tr("Вероятность", "Probability"));
                ui.end_row();
            });

        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let max_height = Self::markov_section_height(ui, 320.0, 140.0);

        scroll_utils::show_virtualized_rows(
            ui,
            "markov_stationary_distribution",
            max_height,
            row_h,
            stationary.len(),
            |ui: &mut egui::Ui, row_idx: usize| {
                ui.horizontal(|ui| {
                    Self::markov_draw_cell(ui, state_col, format!("S{}", row_idx + 1));
                    Self::markov_draw_cell(
                        ui,
                        marking_col,
                        self.markov_marking_summary(&chain.states[row_idx]),
                    );
                    Self::markov_draw_cell(ui, prob_col, format!("{:.6}", stationary[row_idx]));
                });
            },
        );
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        if chain.transitions.is_empty() {
            ui.label(self.tr("Переходов не найдено", "No transitions detected"));
            return;
        }

        let (state_col, transitions_col) = Self::markov_state_graph_column_widths();
        egui::Grid::new("markov_state_graph_header")
            .striped(true)
            .show(ui, |ui| {
                Self::markov_draw_cell(ui, state_col, self.tr("Состояние", "State"));
                Self::markov_draw_cell(ui, transitions_col, self.tr("Переходы", "Transitions"));
                ui.end_row();
            });

        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let max_height = Self::markov_section_height(ui, 280.0, 140.0);

        scroll_utils::show_virtualized_rows(
            ui,
            "markov_state_graph",
            max_height,
            row_h,
            chain.transitions.len(),
            |ui: &mut egui::Ui, row_idx: usize| {
                ui.horizontal(|ui| {
                    Self::markov_draw_cell(ui, state_col, format!("S{}", row_idx + 1));
                    Self::markov_draw_cell(
                        ui,
                        transitions_col,
                        self.markov_transition_summary(chain, row_idx),
                    );
                });
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

                        ui.label(format!("{}: {}", self.tr("Позиция", "Place"), place_label,));

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
                                    egui::Label::new(self.tr("Число маркеров", "Token count")),
                                );
                                ui.add_sized(
                                    [84.0, 0.0],
                                    egui::Label::new(self.tr("Вероятность", "Probability")),
                                );
                            });
                            for (count, prob) in distribution.iter() {
                                ui.horizontal(|ui: &mut egui::Ui| {
                                    ui.add_sized([140.0, 0.0], egui::Label::new(count.to_string()));
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

    fn markov_marking_summary(&self, marking: &[u32]) -> String {
        let mut parts = Vec::new();
        for (idx, &tokens) in marking.iter().enumerate() {
            if tokens > 0 {
                parts.push(format!(
                    "{} = {}",
                    self.markov_place_display_name(idx),
                    tokens
                ));
            }
        }

        if parts.is_empty() {
            self.tr("все 0", "all 0").into_owned()
        } else {
            parts.join(", ")
        }
    }

    fn markov_transition_summary(&self, chain: &MarkovChain, row_idx: usize) -> String {
        let edges = &chain.transitions[row_idx];
        if edges.is_empty() {
            return self.tr("Переходов нет", "No transitions").into_owned();
        }

        let total_rate: f64 = edges.iter().map(|(_, rate)| *rate).sum();
        edges
            .iter()
            .map(|(dest, rate)| {
                let prob = if total_rate > 0.0 {
                    (rate / total_rate).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                format!("S{} ({:.2})", dest + 1, prob)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn markov_stationary_column_widths() -> (f32, f32, f32) {
        (76.0, 520.0, 96.0)
    }

    fn markov_state_graph_column_widths() -> (f32, f32) {
        (76.0, 540.0)
    }

    fn markov_draw_cell(ui: &mut egui::Ui, width: f32, text: impl Into<WidgetText>) {
        ui.add_sized([width, 0.0], egui::Label::new(text));
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
}
