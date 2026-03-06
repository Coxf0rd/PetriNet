use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_markov_window;
        egui::Window::new(self.tr("РњР°СЂРєРѕРІСЃРєР°СЏ РјРѕРґРµР»СЊ", "Markov model"))
            .id(egui::Id::new("markov_window"))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .small_button(self.tr("РџРµСЂРµС‡РёСЃС‚РёС‚СЊ", "Rebuild"))
                        .clicked()
                    {
                        self.net.sanitize_values();
                        self.calculate_markov_model();
                    }
                    ui.label(self.tr(
                        "РџРѕР»СѓС‡РµРЅРёРµ РїСЂРµРґРµР»РѕРІР°РЅРёСЏ РїРѕР»СЏРјРѕС‡РёРЅРµР№ РєРѕР»РјРѕРіРѕСЂРѕРІС‹С… СѓСЂР°РІРЅРµРЅРёР№",
                        "Stationary probabilities solve Kolmogorov equations",
                    ));
                });
                if let Some(chain) = &self.markov_model {
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
                    let total_edges: usize =
                        chain.transitions.iter().map(|edges| edges.len()).sum::<usize>();
                    ui.label(format!(
                        "{}: {}",
                        self.tr("РџРµСЂРµС…РѕРґС‹", "Transitions"),
                        total_edges
                    ));
                    if let Some(stationary) = &chain.stationary {
                        ui.label(self.tr(
                            "РЎС‚Р°С†РёРѕРЅР°СЂРЅРѕРµ РїРѕРґРµР»РµРЅРёРµ", 
                            "Stationary distribution",
                        ));
                        egui::ScrollArea::vertical()
                            .max_height(280.0)
                            .show(ui, |ui| {
                                egui::Grid::new("markov_states").striped(true).show(ui, |ui| {
                                    ui.label(self.tr("РЎРѕСЃС‚РѕСЏРЅРёРµ", "State"));
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
                            "РўРµСЂРѕС‚РѕРІР°РЅРёРµ РЅРµ РІС‹РїРѕР»РЅРµРЅРѕ",
                            "Unable to compute stationary" ,
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
                                ui.label(self.tr(
                                    "Переходов не найдено",
                                    "No transitions detected",
                                ));
                            } else if chain.state_count() > rows {
                                ui.label(format!(
                                    "... {} {} ...",
                                    chain.state_count() - rows,
                                    self.tr("состояний пропущено", "states skipped"),
                                ));
                            }
                        });
                } else {
                    ui.label(self.tr(
                        "РџРѕСЃС‚СЂРѕР№С‚Рµ РјРѕРґРµР»СЊ",
                        "Build the model",
                    ));
                }
            });
        self.show_markov_window = open;
    }
}
