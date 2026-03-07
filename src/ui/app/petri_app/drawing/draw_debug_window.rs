use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_debug_window(&mut self, ctx: &egui::Context) {
        if !self.show_debug {
            return;
        }
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = self.show_debug;
        egui::Window::new(t("Режим отладки", "Debug Mode"))
            .open(&mut open)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.clone() else {
                    ui.label(t("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };
                let visible_steps = Self::debug_visible_log_indices(&result);
                let steps = visible_steps.len();
                if steps == 0 {
                    ui.label(t("Пустой журнал.", "Empty log."));
                    return;
                }
                if self.debug_step >= steps {
                    self.debug_step = steps - 1;
                }

                ui.horizontal(|ui| {
                    if ui.button("<<").clicked() {
                        self.debug_playing = false;
                        self.debug_animation_last_update = None;
                        self.debug_step = self.debug_step.saturating_sub(1);
                        self.sync_debug_animation_for_step();
                    }
                    if ui
                        .button(if self.debug_playing {
                            t("Пауза", "Pause")
                        } else {
                            t("Пуск", "Play")
                        })
                        .clicked()
                    {
                        if self.debug_playing {
                            self.debug_playing = false;
                        } else {
                            self.debug_playing = true;
                        }
                        self.debug_animation_last_update = None;
                    }
                    if ui.button(">>").clicked() {
                        self.debug_playing = false;
                        self.debug_animation_last_update = None;
                        self.debug_step = (self.debug_step + 1).min(steps - 1);
                        self.sync_debug_animation_for_step();
                    }
                    ui.label(t("Скорость (мс сим.сек):", "Speed (ms per sim sec):"));
                    ui.add(egui::DragValue::new(&mut self.debug_interval_ms).range(50..=5_000));
                });

                let slider_response = ui.add(
                    egui::Slider::new(&mut self.debug_step, 0..=steps - 1).text(t("Шаг", "Step")),
                );
                if slider_response.changed() {
                    self.debug_playing = false;
                    self.debug_animation_last_update = None;
                    self.sync_debug_animation_for_step();
                }
                if self.debug_playing && steps > 1 {
                    let interval = Duration::from_millis(self.debug_interval_ms.max(1));
                    let now = Instant::now();
                    match self.debug_animation_last_update {
                        Some(last) => {
                            if now.duration_since(last) >= interval {
                                if self.debug_step < steps - 1 {
                                    self.debug_step += 1;
                                    self.sync_debug_animation_for_step();
                                } else {
                                    self.debug_playing = false;
                                }
                                self.debug_animation_last_update = Some(now);
                            }
                        }
                        None => {
                            self.debug_animation_last_update = Some(now);
                        }
                    }
                }
                if self.debug_playing {
                    ctx.request_repaint_after(Duration::from_millis(16));
                }
                let animation_response = ui.checkbox(
                    &mut self.debug_animation_enabled,
                    t("Включить анимацию", "Enable animation"),
                );
                if animation_response.changed() {
                    self.debug_arc_animation = self.debug_animation_enabled;
                    self.debug_animation_last_update = None;
                    if self.debug_animation_enabled {
                        self.refresh_debug_animation_state();
                    } else {
                        self.debug_playing = false;
                        self.clear_debug_animation_state();
                    }
                }
                if self.debug_animation_enabled {
                    if self.debug_animation_events.is_empty() {
                        ui.label(t(
                            "Сначала запустите симуляцию, чтобы увидеть анимацию.",
                            "Run a simulation first to see the animation.",
                        ));
                    }
                }
                if let Some(&log_idx) = visible_steps.get(self.debug_step) {
                    if let Some(entry) = result.logs.get(log_idx) {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(t("Текущее время", "Current time"));
                            ui.label("t");
                            ui.label(format!("= {:.3}", entry.time));
                        });
                        ui.label(format!(
                            "{}: {}",
                            t("Переход", "Transition"),
                            entry
                                .fired_transition
                                .map(|i| format!("T{}", i + 1))
                                .unwrap_or_else(|| "-".to_string())
                        ));
                        egui::Grid::new("debug_marking_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                for (idx, marking) in entry.marking.iter().enumerate() {
                                    ui.label(format!("P{}", idx + 1));
                                    ui.label(marking.to_string());
                                    ui.end_row();
                                }
                            });
                    }
                }
            });
        self.show_debug = open;
    }
}
