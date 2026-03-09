use super::*;
use crate::markov::{BuildStopReason, StationaryStatus};
use crate::ui::property_selection::{show_collapsible_property_section, PropertySectionConfig};
use crate::ui::property_window::{show_property_window, PropertyWindowConfig};
use crate::ui::scroll_utils;
use egui::{Color32, Frame, RichText, WidgetText};

struct MarkovStationaryRow {
    state_index: Option<usize>,
    place_text: String,
    place_hover: Option<String>,
    tokens_text: String,
    probability: Option<f64>,
}

struct MarkovTransitionRow {
    group_state_index: usize,
    source_index: Option<usize>,
    target_text: String,
    probability: Option<f64>,
}

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        if !self.show_markov_window {
            return;
        }

        let mut open = self.show_markov_window;

        show_property_window(
            ctx,
            self.tr("РњР°СЂРєРѕРІСЃРєР°СЏ РјРѕРґРµР»СЊ", "Markov model"),
            &mut open,
            PropertyWindowConfig::new("markov_window")
                .default_size(egui::vec2(520.0, 520.0))
                .min_size(egui::vec2(360.0, 280.0))
                .resizable(true),
            |ui| {
                let simulation_ready = self.sim_result.is_some();
                let mut toggle_changed = false;
                let markov_checkbox_label = self.tr(
                    "РїРѕРєР°Р·С‹РІР°С‚СЊ РґСѓРіРё РјР°СЂРєРѕРІСЃРєРѕР№ РјРѕРґРµР»Рё РІ СЂР°Р±РѕС‡РµР№ РѕР±Р»Р°СЃС‚Рё",
                    "Show Markov model arcs in workspace",
                );
                let simulation_hint = self.tr(
                    "РЎРЅР°С‡Р°Р»Р° Р·Р°РїСѓСЃС‚РёС‚Рµ СЃРёРјСѓР»СЏС†РёСЋ, С‡С‚РѕР±С‹ СЂР°СЃСЃС‡РёС‚Р°С‚СЊ РјР°СЂРєРѕРІСЃРєСѓСЋ РјРѕРґРµР»СЊ",
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

                if toggle_changed {
                    for place in &mut self.net.places {
                        place.show_markov_model = self.markov_model_enabled;
                    }
                    self.refresh_markov_place_arcs();
                }

                ui.add_space(6.0);

                if let Some(chain) = &self.markov_model {
                    self.draw_markov_chain_summary(ui, chain);
                } else if simulation_ready {
                    ui.label(self.tr(
                        "РњР°СЂРєРѕРІСЃРєР°СЏ РјРѕРґРµР»СЊ РµС‰С‘ РЅРµ СЂР°СЃСЃС‡РёС‚Р°РЅР° РґР»СЏ С‚РµРєСѓС‰РµРіРѕ СЂРµР·СѓР»СЊС‚Р°С‚Р° СЃРёРјСѓР»СЏС†РёРё",
                        "The Markov model has not been calculated for the current simulation result yet",
                    ));
                } else {
                    ui.label(self.tr("РџРѕСЃС‚СЂРѕР№С‚Рµ РјРѕРґРµР»СЊ", "Build the model"));
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
            ui.label(format!(
                "{}: {}{}",
                self.tr("РЎРѕСЃС‚РѕСЏРЅРёР№", "States"),
                chain.state_count(),
                if chain.limit_reached {
                    format!(" ({})", self.tr("Р»РёРјРёС‚", "limit reached"))
                } else {
                    String::new()
                }
            ));

            ui.label(format!(
                "{}: {}",
                self.tr("РџРµСЂРµС…РѕРґРѕРІ", "Transitions"),
                chain.transition_count_after_merge
            ));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{}: {}",
                self.tr("Переходов до схлопывания", "Transitions before merge"),
                chain.transition_count_before_merge
            ));
            ui.separator();
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
            };
            ui.label(stop_reason);
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
                .label(self.tr(
                    "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ",
                    "Stationary distribution",
                ))
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
                .label(self.tr("Р“СЂР°С„ СЃРѕСЃС‚РѕСЏРЅРёР№", "State graph"))
                .default_open(false),
            |ui| {
                self.draw_markov_state_graph(ui, chain);
            },
        );

        let _ = show_collapsible_property_section(
            ui,
            PropertySectionConfig::new("markov_highlight_section")
                .label(self.tr(
                    "РћС‚РѕР±СЂР°Р¶РµРЅРёРµ РјР°СЂРєРѕРІСЃРєРѕР№ РјРµС‚РєРё",
                    "Markov highlight display",
                ))
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
                    "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ СЂР°СЃСЃС‡РёС‚Р°РЅРѕ",
                    "Stationary distribution computed",
                )
                .into_owned(),
            StationaryStatus::LimitReached { explored_states, limit } => format!(
                "{}: {} / {}",
                self.tr(
                    "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ РЅРµ РІС‹С‡РёСЃР»РµРЅРѕ: РґРѕСЃС‚РёРіРЅСѓС‚ Р»РёРјРёС‚ СЃРѕСЃС‚РѕСЏРЅРёР№",
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
                        self.tr("РїРѕР·РёС†РёР№ СЃ Р·Р°РґРµСЂР¶РєРѕР№", "delayed places"),
                        delayed_places
                    ));
                }
                if *stochastic_places > 0 {
                    details.push(format!(
                        "{}: {}",
                        self.tr("РїРѕР·РёС†РёР№ СЃРѕ СЃС‚РѕС…Р°СЃС‚РёРєРѕР№", "stochastic places"),
                        stochastic_places
                    ));
                }
                format!(
                    "{}{}{}",
                    self.tr(
                        "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ РґР»СЏ СЃРµС‚Рё СЃ Р·Р°РґРµСЂР¶РєР°РјРё/СЃС‚РѕС…Р°СЃС‚РёРєРѕР№ СЃРµР№С‡Р°СЃ РЅРµ СЂР°СЃСЃС‡РёС‚С‹РІР°РµС‚СЃСЏ",
                        "Stationary distribution is currently unavailable for timed or stochastic nets",
                    ),
                    if details.is_empty() { "" } else { ": " },
                    details.join(", ")
                )
            }
            StationaryStatus::SolverDidNotConverge => self
                .tr(
                    "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ РЅРµ РІС‹С‡РёСЃР»РµРЅРѕ: С‡РёСЃР»РµРЅРЅС‹Р№ СЂРµС€Р°С‚РµР»СЊ РЅРµ СЃРѕС€С‘Р»СЃСЏ",
                    "Stationary distribution unavailable: numerical solver did not converge",
                )
                .into_owned(),
            StationaryStatus::NoDynamicTransitions => self
                .tr(
                    "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ РЅРµ РІС‹С‡РёСЃР»РµРЅРѕ: РІ РіСЂР°С„Рµ СЃРѕСЃС‚РѕСЏРЅРёР№ РЅРµС‚ РІС‹С…РѕРґСЏС‰РёС… РёРЅС‚РµРЅСЃРёРІРЅРѕСЃС‚РµР№",
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
            ui.label(self.tr("РЎРѕСЃС‚РѕСЏРЅРёР№ РЅРµ РЅР°Р№РґРµРЅРѕ", "No states found"));
            return;
        }

        let rows = self.markov_stationary_rows(chain, stationary);
        let [state_col, place_col, tokens_col, prob_col] = Self::markov_stationary_column_widths();
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;

        egui::Grid::new("markov_stationary_header")
            .striped(true)
            .show(ui, |ui| {
                Self::markov_draw_cell(
                    ui,
                    state_col,
                    RichText::new(self.tr("РЎРѕСЃС‚РѕСЏРЅРёРµ", "State")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    place_col,
                    RichText::new(self.tr("РџРѕР·РёС†РёСЏ", "Place")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    tokens_col,
                    RichText::new(self.tr("РњР°СЂРєРµСЂС‹", "Tokens")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    prob_col,
                    RichText::new(self.tr("Р’РµСЂРѕСЏС‚РЅРѕСЃС‚СЊ", "Probability")).strong(),
                );
                ui.end_row();
            });

        scroll_utils::show_virtualized_rows(
            ui,
            "markov_stationary_distribution",
            180.0,
            row_h,
            rows.len(),
            |ui: &mut egui::Ui, idx: usize| {
                let row = &rows[idx];
                egui::Grid::new(("markov_stationary_row", idx))
                    .num_columns(4)
                    .show(ui, |ui| {
                        let state_text = row
                            .state_index
                            .map(|state_idx| format!("S{}", state_idx + 1))
                            .unwrap_or_default();
                        Self::markov_draw_cell(ui, state_col, state_text);

                        let place_response =
                            Self::markov_draw_cell(ui, place_col, row.place_text.as_str());
                        if let Some(full) = &row.place_hover {
                            place_response.on_hover_text(full);
                        }

                        Self::markov_draw_cell(ui, tokens_col, row.tokens_text.as_str());

                        let prob_text = row
                            .probability
                            .map(|value| format!("{:.6}", value))
                            .unwrap_or_default();
                        Self::markov_draw_cell(ui, prob_col, prob_text);
                        ui.end_row();
                    });
            },
        );
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        if chain.transitions.is_empty() {
            ui.label(self.tr(
                "РџРµСЂРµС…РѕРґРѕРІ РЅРµ РЅР°Р№РґРµРЅРѕ",
                "No transitions detected",
            ));
            return;
        }

        let rows = self.markov_state_graph_rows(chain);
        let [state_col, target_col, prob_col] = Self::markov_state_graph_column_widths();
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;

        egui::Grid::new("markov_state_graph_header")
            .striped(true)
            .show(ui, |ui| {
                Self::markov_draw_cell(
                    ui,
                    state_col,
                    RichText::new(self.tr("РЎРѕСЃС‚РѕСЏРЅРёРµ", "State")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    target_col,
                    RichText::new(self.tr("РџРµСЂРµС…РѕРґ", "Transition")).strong(),
                );
                Self::markov_draw_cell(
                    ui,
                    prob_col,
                    RichText::new(self.tr("Р’РµСЂРѕСЏС‚РЅРѕСЃС‚СЊ", "Probability")).strong(),
                );
                ui.end_row();
            });

        scroll_utils::show_virtualized_rows(
            ui,
            "markov_state_graph",
            180.0,
            row_h,
            rows.len(),
            |ui: &mut egui::Ui, idx: usize| {
                let row = &rows[idx];
                let fill = if row.group_state_index % 2 == 1 {
                    Color32::from_rgb(235, 245, 255)
                } else {
                    Color32::TRANSPARENT
                };
                Frame::none().fill(fill).show(ui, |ui| {
                    egui::Grid::new(("markov_state_graph_row", idx))
                        .num_columns(3)
                        .show(ui, |ui| {
                            let state_text = row
                                .source_index
                                .map(|state_idx| format!("S{}", state_idx + 1))
                                .unwrap_or_default();
                            Self::markov_draw_cell(ui, state_col, state_text);
                            Self::markov_draw_cell(ui, target_col, row.target_text.as_str());
                            let prob_text = row
                                .probability
                                .map(|value| format!("{:.2}%", value * 100.0))
                                .unwrap_or_default();
                            Self::markov_draw_cell(ui, prob_col, prob_text);
                            ui.end_row();
                        });
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
                "РћС‚РјРµС‚СЊС‚Рµ РјР°СЂРєРѕРІСЃРєСѓСЋ РјРµС‚РєСѓ РІ СЃРІРѕР№СЃС‚РІР°С… РїРѕР·РёС†РёРё, С‡С‚РѕР±С‹ СѓРІРёРґРµС‚СЊ РµС‘ РѕС‚РѕР±СЂР°Р¶РµРЅРёРµ",
                "Enable the Markov highlight on a place to view its display",
            ));
            return;
        }

        ui.label(self.tr(
            "РћС‚РѕР±СЂР°Р¶РµРЅРёРµ РјР°СЂРєРѕРІСЃРєРѕР№ РјРµС‚РєРё",
            "Markov highlight display",
        ));

        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());

        let max_height = Self::markov_section_height(180.0, 140.0);

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
                        let place_label = if place.name.trim().is_empty() {
                            format!("P{}", place.id)
                        } else {
                            format!("P{} {{{}}}", place.id, place.name.trim())
                        };

                        ui.label(format!("{}: {}", self.tr("РџРѕР·РёС†РёСЏ", "Place"), place_label));

                        if let Some(expected) = expectation
                            .as_ref()
                            .and_then(|values| values.get(*place_idx))
                        {
                            ui.label(format!(
                                "{}: {:.3}",
                                self.tr("РћР¶РёРґР°РµРјРѕРµ С‡РёСЃР»Рѕ РјР°СЂРєРµСЂРѕРІ", "Expected tokens"),
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
                                            self.tr("РјР°СЂРєРµСЂРѕРІ", "tokens")
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
                                "Р”Р»СЏ СЌС‚РѕР№ РїРѕР·РёС†РёРё СЃРѕСЃС‚РѕСЏРЅРёСЏ РЅРµ РЅР°Р№РґРµРЅС‹",
                                "No states found for this place",
                            ));
                        } else {
                            ui.label(self.tr(
                                "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ СЂР°СЃРїСЂРµРґРµР»РµРЅРёРµ РЅРµРґРѕСЃС‚СѓРїРЅРѕ",
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

    fn markov_stationary_rows(
        &self,
        chain: &MarkovChain,
        stationary: &[f64],
    ) -> Vec<MarkovStationaryRow> {
        let mut rows = Vec::new();

        for (state_idx, marking) in chain.states.iter().enumerate() {
            let mut nonzero_entries = Vec::new();
            for (place_idx, tokens) in marking.iter().enumerate() {
                if *tokens == 0 {
                    continue;
                }
                nonzero_entries.push((place_idx, *tokens));
            }

            if nonzero_entries.is_empty() {
                rows.push(MarkovStationaryRow {
                    state_index: Some(state_idx),
                    place_text: self
                        .tr("РїСѓСЃС‚Р°СЏ РјР°СЂРєРёСЂРѕРІРєР°", "empty marking")
                        .into_owned(),
                    place_hover: None,
                    tokens_text: String::new(),
                    probability: stationary.get(state_idx).copied(),
                });
                continue;
            }

            for (entry_idx, (place_idx, tokens)) in nonzero_entries.into_iter().enumerate() {
                let place_text = self.markov_place_label(place_idx);
                rows.push(MarkovStationaryRow {
                    state_index: (entry_idx == 0).then_some(state_idx),
                    place_hover: Some(place_text.clone()),
                    place_text,
                    tokens_text: tokens.to_string(),
                    probability: (entry_idx == 0)
                        .then(|| stationary.get(state_idx).copied())
                        .flatten(),
                });
            }
        }

        rows
    }

    fn markov_state_graph_rows(&self, chain: &MarkovChain) -> Vec<MarkovTransitionRow> {
        let mut rows = Vec::new();

        for (state_idx, edges) in chain.transitions.iter().enumerate() {
            if edges.is_empty() {
                rows.push(MarkovTransitionRow {
                    group_state_index: state_idx,
                    source_index: Some(state_idx),
                    target_text: self
                        .tr("РїРµСЂРµС…РѕРґРѕРІ РЅРµС‚", "no transitions")
                        .into_owned(),
                    probability: None,
                });
                continue;
            }

            let total_rate: f64 = edges.iter().map(|(_, rate)| *rate).sum();
            for (edge_idx, (dest, rate)) in edges.iter().enumerate() {
                let probability = if total_rate > 0.0 {
                    (rate / total_rate).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                rows.push(MarkovTransitionRow {
                    group_state_index: state_idx,
                    source_index: (edge_idx == 0).then_some(state_idx),
                    target_text: format!("в†’ S{}", dest + 1),
                    probability: Some(probability),
                });
            }
        }

        rows
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
