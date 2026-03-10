use super::*;
use crate::markov::{BuildStopReason, MarkovComputationMode, StationaryStatus};
use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::property_window::{show_property_window, PropertyWindowConfig};
use crate::ui::scroll_utils;
use egui::{Color32, RichText, WidgetText};

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
                let mut arc_mode_changed = false;
                let markov_checkbox_label = self.tr(
                    "показывать дуги марковской модели в рабочей области",
                    "Show Markov model arcs in workspace",
                );
                let simulation_hint = self.tr(
                    "Сначала запустите симуляцию, чтобы рассчитать марковскую модель",
                    "Run a simulation first to calculate the Markov model",
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

                ui.add_enabled_ui(simulation_ready, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(self.tr("Режим дуг в рабочей области:", "Arc mode in workspace:"));
                        arc_mode_changed |= ui
                            .selectable_value(
                                &mut self.markov_arc_view_mode,
                                MarkovArcViewMode::AggregatedWeighted,
                                "Aggregated",
                            )
                            .changed();
                        arc_mode_changed |= ui
                            .selectable_value(
                                &mut self.markov_arc_view_mode,
                                MarkovArcViewMode::ObservedAll,
                                "All observed",
                            )
                            .changed();
                    });
                });

                if toggle_changed {
                    for place in &mut self.net.places {
                        place.show_markov_model = self.markov_model_enabled;
                    }
                    self.refresh_markov_place_arcs();
                }
                if arc_mode_changed {
                    self.refresh_markov_place_arcs();
                }

                ui.add_space(6.0);

                if let Some(chain) = &self.markov_model {
                    self.draw_markov_chain_summary(ui, chain);
                } else if simulation_ready {
                    ui.label(self.tr(
                        "Марковская модель ещё не рассчитана для текущего результата симуляции",
                        "The Markov model has not been calculated for the current simulation result yet",
                    ));
                } else {
                    ui.label(self.tr("Постройте модель", "Build the model"));
                }
            },
        );

        self.show_markov_window = open;
    }

    fn markov_section_height(preferred: f32, min_height: f32) -> f32 {
        preferred.max(min_height)
    }

    fn draw_markov_chain_summary(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        let stationary = chain.stationary.as_ref().map(|values| values.as_slice());

        ui.horizontal(|ui| {
            let mode_text = match chain.computation_mode {
                MarkovComputationMode::Exact => self.tr("Режим: точный", "Mode: exact"),
                MarkovComputationMode::Approximate => {
                    self.tr("Режим: примерный", "Mode: approximate")
                }
            };
            let states_label = match chain.computation_mode {
                MarkovComputationMode::Exact => self.tr("Состояний", "States"),
                MarkovComputationMode::Approximate => self.tr("Состояний в логе", "States in log"),
            };
            let transitions_label = match chain.computation_mode {
                MarkovComputationMode::Exact => self.tr("Переходов", "Transitions"),
                MarkovComputationMode::Approximate => self.tr(
                    "Наблюдённых переходов в логе",
                    "Observed transitions in log",
                ),
            };
            ui.label(mode_text.as_ref());
            ui.separator();
            ui.label(format!(
                "{}: {}{}",
                states_label,
                chain.state_count(),
                if chain.limit_reached {
                    format!(" ({})", self.tr("лимит", "limit reached"))
                } else {
                    String::new()
                }
            ));

            ui.label(format!(
                "{}: {}",
                transitions_label, chain.transition_count_after_merge
            ));
        });

        ui.horizontal_wrapped(|ui| {
            let stop_reason = match &chain.build_stop_reason {
                BuildStopReason::ExhaustedStateSpace { explored_states } => format!(
                    "{}: {}",
                    self.tr(
                        "Остановка: пространство состояний исчерпано",
                        "Stop: state-space exhausted",
                    ),
                    explored_states
                ),
                BuildStopReason::StateLimitReached {
                    explored_states,
                    limit,
                } => format!(
                    "{}: {} / {}",
                    self.tr(
                        "Остановка: достигнут лимит состояний",
                        "Stop: state limit reached",
                    ),
                    explored_states,
                    limit
                ),
                BuildStopReason::ApproximationFromSimulation {
                    sampled_states,
                    sampled_steps,
                } => format!(
                    "{}: {} / {}",
                    self.tr(
                        "Остановка: использована аппроксимация по журналу симуляции (состояний/шагов)",
                        "Stop: approximation from simulation log (states/steps)",
                    ),
                    sampled_states,
                    sampled_steps
                ),
            };
            ui.label(stop_reason);
            if chain.computation_mode == MarkovComputationMode::Approximate {
                if let Some(sim_result) = self.sim_result.as_deref() {
                    ui.separator();
                    ui.label(format!(
                        "{}: {}",
                        self.tr(
                            "Срабатываний переходов симулятора",
                            "Simulation fired transitions",
                        ),
                        sim_result.fired_count
                    ));
                }
            }
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
                    ui.label(self.markov_stationary_status_text(chain));
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

    fn markov_stationary_status_text(&self, chain: &MarkovChain) -> String {
        match &chain.stationary_status {
            StationaryStatus::Computed => self
                .tr(
                    "Стационарное распределение рассчитано",
                    "Stationary distribution computed",
                )
                .into_owned(),
            StationaryStatus::LimitReached { explored_states, limit } => format!(
                "{}: {} / {}",
                self.tr(
                    "Стационарное распределение не вычислено: достигнут лимит состояний",
                    "Stationary distribution unavailable: state limit reached",
                ),
                explored_states,
                limit
            ),
            StationaryStatus::TimedNetUnsupported {
                delayed_places,
                stochastic_places,
            } => {
                let mut details = Vec::new();
                if *delayed_places > 0 {
                    details.push(format!(
                        "{}: {}",
                        self.tr("позиций с задержкой", "delayed places"),
                        delayed_places
                    ));
                }
                if *stochastic_places > 0 {
                    details.push(format!(
                        "{}: {}",
                        self.tr("позиций со стохастикой", "stochastic places"),
                        stochastic_places
                    ));
                }
                format!(
                    "{}{}{}",
                    self.tr(
                        "Стационарное распределение для сети с задержками/стохастикой сейчас не рассчитывается",
                        "Stationary distribution is currently unavailable for timed or stochastic nets",
                    ),
                    if details.is_empty() { "" } else { ": " },
                    details.join(", ")
                )
            }
            StationaryStatus::SolverDidNotConverge => self
                .tr(
                    "Стационарное распределение не вычислено: численный решатель не сошёлся",
                    "Stationary distribution unavailable: numerical solver did not converge",
                )
                .into_owned(),
            StationaryStatus::NoDynamicTransitions => self
                .tr(
                    "Стационарное распределение не вычислено: в графе состояний нет выходящих интенсивностей",
                    "Stationary distribution unavailable: the state graph has no outgoing rates",
                )
                .into_owned(),
        }
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

        let [state_col, place_col, tokens_col, prob_col] = Self::markov_stationary_column_widths();
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let fallback_offsets;
        let row_offsets = if self.markov_stationary_row_offsets.len() == chain.states.len() + 1 {
            self.markov_stationary_row_offsets.as_slice()
        } else {
            fallback_offsets = Self::markov_build_stationary_row_offsets(chain);
            fallback_offsets.as_slice()
        };
        let total_rows = row_offsets.last().copied().unwrap_or(0);

        egui::Grid::new("markov_stationary_header")
            .striped(true)
            .show(ui, |ui| {
                Self::markov_draw_cell(
                    ui,
                    state_col,
                    RichText::new(self.tr("Состояние", "State")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    place_col,
                    RichText::new(self.tr("Позиция", "Place")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    tokens_col,
                    RichText::new(self.tr("Маркеры", "Tokens")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    prob_col,
                    RichText::new(self.tr("Вероятность", "Probability")).strong(),
                );
                ui.end_row();
            });

        scroll_utils::show_virtualized_rows(
            ui,
            "markov_stationary_distribution",
            180.0,
            row_h,
            total_rows,
            |ui: &mut egui::Ui, idx: usize| {
                let Some((state_idx, local_row_idx)) =
                    Self::markov_row_group_index(row_offsets, idx)
                else {
                    return;
                };
                let Some(marking) = chain.states.get(state_idx) else {
                    return;
                };
                let probability = stationary.get(state_idx).copied();
                let nonzero_count = marking.iter().filter(|&&tokens| tokens > 0).count();

                let (place_text, place_hover, tokens_text) = if nonzero_count == 0 {
                    (
                        self.tr("пустая маркировка", "empty marking").into_owned(),
                        None,
                        String::new(),
                    )
                } else if let Some((place_idx, tokens)) =
                    Self::markov_nonzero_place_entry(marking, local_row_idx)
                {
                    let place_text = self.markov_place_label(place_idx);
                    (place_text.clone(), Some(place_text), tokens.to_string())
                } else {
                    return;
                };

                egui::Grid::new(("markov_stationary_row", idx))
                    .num_columns(4)
                    .show(ui, |ui| {
                        let state_text = if local_row_idx == 0 {
                            format!("S{}", state_idx + 1)
                        } else {
                            String::new()
                        };
                        Self::markov_draw_cell(ui, state_col, state_text);

                        let place_response =
                            Self::markov_draw_cell(ui, place_col, place_text.as_str());
                        if let Some(full) = &place_hover {
                            place_response.on_hover_text(full);
                        }

                        Self::markov_draw_cell(ui, tokens_col, tokens_text.as_str());

                        let prob_text = if local_row_idx == 0 {
                            probability
                                .map(|value| format!("{:.6}", value))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        Self::markov_draw_cell(ui, prob_col, prob_text);
                        ui.end_row();
                    });
            },
        );
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        if chain.transitions.is_empty() {
            ui.label(self.tr("Переходов не найдено", "No transitions detected"));
            return;
        }

        let [state_col, target_col, prob_col] = Self::markov_state_graph_column_widths();
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let fallback_offsets;
        let row_offsets =
            if self.markov_state_graph_row_offsets.len() == chain.transitions.len() + 1 {
                self.markov_state_graph_row_offsets.as_slice()
            } else {
                fallback_offsets = Self::markov_build_state_graph_row_offsets(chain);
                fallback_offsets.as_slice()
            };
        let total_rows = row_offsets.last().copied().unwrap_or(0);

        egui::Grid::new("markov_state_graph_header")
            .striped(true)
            .show(ui, |ui| {
                Self::markov_draw_cell(
                    ui,
                    state_col,
                    RichText::new(self.tr("Состояние", "State")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    target_col,
                    RichText::new(self.tr("Переход", "Transition")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    prob_col,
                    RichText::new(self.tr("Вероятность", "Probability")).strong(),
                );
                ui.end_row();
            });

        scroll_utils::show_virtualized_rows(
            ui,
            "markov_state_graph",
            180.0,
            row_h,
            total_rows,
            |ui: &mut egui::Ui, idx: usize| {
                let Some((state_idx, local_row_idx)) =
                    Self::markov_row_group_index(row_offsets, idx)
                else {
                    return;
                };
                let Some(edges) = chain.transitions.get(state_idx) else {
                    return;
                };

                let (target_text, prob_text) = if edges.is_empty() {
                    (
                        self.tr("переходов нет", "no transitions").into_owned(),
                        String::new(),
                    )
                } else {
                    let Some((dest, rate)) = edges.get(local_row_idx).copied() else {
                        return;
                    };
                    let total_rate: f64 = edges.iter().map(|(_, edge_rate)| *edge_rate).sum();
                    let probability = if total_rate > 0.0 {
                        (rate / total_rate).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    (
                        format!("-> S{}", dest + 1),
                        format!("{:.2}%", probability * 100.0),
                    )
                };

                egui::Grid::new(("markov_state_graph_row", idx))
                    .num_columns(3)
                    .show(ui, |ui| {
                        let state_text = if local_row_idx == 0 {
                            format!("S{}", state_idx + 1)
                        } else {
                            String::new()
                        };
                        Self::markov_draw_cell(ui, state_col, state_text);
                        Self::markov_draw_cell(ui, target_col, target_text);
                        Self::markov_draw_cell(ui, prob_col, prob_text);
                        ui.end_row();
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

        ui.label(self.tr("Отображение марковской метки", "Markov highlight display"));

        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());

        let max_height = Self::markov_section_height(180.0, 140.0);

        scroll_utils::show_list_with_scroll(
            ui,
            "markov_place_distribution",
            max_height,
            |ui: &mut egui::Ui| {
                for (place_idx, place) in &markov_highlight_places {
                    ui.group(|ui: &mut egui::Ui| {
                        let place_label = if place.name.trim().is_empty() {
                            format!("P{}", place.id)
                        } else {
                            format!("P{} {{{}}}", place.id, place.name.trim())
                        };

                        ui.label(format!("{}: {}", self.tr("Позиция", "Place"), place_label));

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
                                    ui.add_sized(
                                        [140.0, 0.0],
                                        egui::Label::new(format!(
                                            "{} {}",
                                            count,
                                            self.tr("маркеров", "tokens")
                                        )),
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

    fn markov_draw_cell(
        ui: &mut egui::Ui,
        width: f32,
        text: impl Into<WidgetText>,
    ) -> egui::Response {
        ui.add_sized([width, 0.0], egui::Label::new(text).truncate())
    }

    fn markov_stationary_column_widths() -> [f32; 4] {
        [88.0, 420.0, 90.0, 110.0]
    }

    fn markov_state_graph_column_widths() -> [f32; 3] {
        [88.0, 420.0, 120.0]
    }

    pub(in crate::ui::app) fn markov_build_stationary_row_offsets(
        chain: &MarkovChain,
    ) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(chain.states.len().saturating_add(1));
        offsets.push(0);
        let mut total = 0usize;
        for marking in &chain.states {
            let row_count = marking.iter().filter(|&&tokens| tokens > 0).count().max(1);
            total = total.saturating_add(row_count);
            offsets.push(total);
        }
        offsets
    }

    pub(in crate::ui::app) fn markov_build_state_graph_row_offsets(
        chain: &MarkovChain,
    ) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(chain.transitions.len().saturating_add(1));
        offsets.push(0);
        let mut total = 0usize;
        for edges in &chain.transitions {
            total = total.saturating_add(edges.len().max(1));
            offsets.push(total);
        }
        offsets
    }

    fn markov_row_group_index(row_offsets: &[usize], row_idx: usize) -> Option<(usize, usize)> {
        if row_offsets.len() < 2 {
            return None;
        }
        let Some(total_rows) = row_offsets.last().copied() else {
            return None;
        };
        if row_idx >= total_rows {
            return None;
        }

        let state_idx = row_offsets.partition_point(|&offset| offset <= row_idx) - 1;
        let local_row_idx = row_idx.saturating_sub(row_offsets[state_idx]);
        Some((state_idx, local_row_idx))
    }

    fn markov_nonzero_place_entry(
        marking: &[u32],
        target_nonzero_idx: usize,
    ) -> Option<(usize, u32)> {
        let mut current_nonzero_idx = 0usize;
        for (place_idx, tokens) in marking.iter().copied().enumerate() {
            if tokens == 0 {
                continue;
            }
            if current_nonzero_idx == target_nonzero_idx {
                return Some((place_idx, tokens));
            }
            current_nonzero_idx = current_nonzero_idx.saturating_add(1);
        }
        None
    }

    fn markov_place_label(&self, place_idx: usize) -> String {
        match self.net.places.get(place_idx) {
            Some(place) => {
                let name = place.name.trim();
                let default_name = format!("P{}", place.id);
                if name.is_empty() || name == default_name {
                    default_name
                } else {
                    format!("{} ({})", default_name, name)
                }
            }
            None => format!("P{}", place_idx + 1),
        }
    }
}
