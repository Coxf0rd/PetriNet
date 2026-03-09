use super::*;

// Import the property window helpers so the debug window can use the same
// sizing and margin conventions as other property windows.
use crate::ui::property_window::{show_property_window, PropertyWindowConfig};
use crate::ui::scroll_utils;

impl PetriApp {
    /// Draw the debug (trace) window.
    ///
    /// The original implementation created an `egui::Window` directly which could grow
    /// to fill the viewport and prevented resizing when the content was large.
    /// This implementation uses our `show_property_window` helper to apply
    /// consistent margins, default/min sizes, and hidden scrollbars.  The
    /// debugging controls and log display are otherwise identical to the
    /// original implementation.
    pub(in crate::ui::app) fn draw_debug_window(&mut self, ctx: &egui::Context) {
        if !self.show_debug {
            return;
        }
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = self.show_debug;
        // Use the property window helper so the window respects margins and has
        // sensible default/min sizes.  The `debug_window` ID is unique to
        // preserve open/closed state across frames.
        show_property_window(
            ctx,
            t("Режим отладки", "Debug Mode"),
            &mut open,
            PropertyWindowConfig::new("debug_window")
                .default_size(egui::vec2(600.0, 480.0))
                .min_size(egui::vec2(400.0, 320.0)),
            |ui| {
                // If there is no simulation result, display a hint and exit.
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
                // Controls for stepping through the log and controlling playback
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
                // Slider for jumping to a specific step
                let slider_response = ui.add(
                    egui::Slider::new(&mut self.debug_step, 0..=steps - 1).text(t("Шаг", "Step")),
                );
                if slider_response.changed() {
                    self.debug_playing = false;
                    self.debug_animation_last_update = None;
                    self.sync_debug_animation_for_step();
                }
                // Handle playback timing
                if self.debug_playing && steps > 1 {
                    let interval = std::time::Duration::from_millis(self.debug_interval_ms.max(1));
                    let now = std::time::Instant::now();
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
                    ctx.request_repaint_after(std::time::Duration::from_millis(16));
                }
                // Animation toggle checkbox
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
                // Show the current log entry details
                if let Some(&log_idx) = visible_steps.get(self.debug_step) {
                    if let Some(entry) = result.logs.get(log_idx) {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(t("Текущее время", "Current time"));
                            ui.label("t");
                            ui.label(format!("= {:.3}", entry.time));
                        });
                        if let Some(tr_idx) = entry.fired_transition {
                            let transition_text = egui::RichText::new(format!(
                                "{}: T{}",
                                t("Переход", "Transition"),
                                tr_idx + 1
                            ))
                            .color(egui::Color32::from_rgb(80, 120, 255));

                            let response = ui
                                .add(egui::Label::new(transition_text).sense(egui::Sense::click()));

                            if let Some(tr) = self.net.transitions.get(tr_idx) {
                                self.canvas.selected_transition = Some(tr.id);

                                if response.clicked() {
                                    let screen_rect = ctx.available_rect();
                                    let target_x = screen_rect.left() + screen_rect.width() * 0.3;
                                    let target_y = screen_rect.center().y;
                                    self.canvas.pan = egui::vec2(
                                        target_x - tr.pos[0] * self.canvas.zoom,
                                        target_y - tr.pos[1] * self.canvas.zoom,
                                    );
                                }
                            }
                        } else {
                            ui.label(format!("{}: -", t("Переход", "Transition")));
                        }

                        // Header
                        egui::Grid::new("debug_marking_header")
                            .num_columns(3)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.add_sized([120.0, 0.0], egui::Label::new("Позиция"));
                                ui.add_sized([80.0, 0.0], egui::Label::new("Маркеры"));
                                ui.add_sized([100.0, 0.0], egui::Label::new("Изменение"));
                                ui.end_row();
                            });

                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
                        let row_count = entry.marking.len();

                        scroll_utils::show_virtualized_rows(
                            ui,
                            "debug_marking_grid",
                            ui.available_height(),
                            row_h,
                            row_count,
                            |ui: &mut egui::Ui, idx: usize| {
                                let current = entry.marking[idx];
                                let prev = if log_idx > 0 {
                                    result
                                        .logs
                                        .get(log_idx - 1)
                                        .and_then(|e| e.marking.get(idx))
                                        .copied()
                                        .unwrap_or(current)
                                } else {
                                    current
                                };
                                let delta: i32 = current as i32 - prev as i32;

                                let place_label = format!("P{}", idx + 1);
                                let delta_text = if delta > 0 {
                                    format!("+{}", delta)
                                } else {
                                    delta.to_string()
                                };

                                let color = if delta > 0 {
                                    egui::Color32::GREEN
                                } else if delta < 0 {
                                    egui::Color32::RED
                                } else {
                                    ui.visuals().text_color()
                                };

                                let row_response = ui
                                    .horizontal(|ui| {
                                        let place_resp = ui.add_sized(
                                            [120.0, 0.0],
                                            egui::SelectableLabel::new(false, place_label),
                                        );

                                        ui.add_sized(
                                            [80.0, 0.0],
                                            egui::Label::new(current.to_string()),
                                        );

                                        ui.add_sized(
                                            [100.0, 0.0],
                                            egui::Label::new(
                                                egui::RichText::new(delta_text).color(color),
                                            ),
                                        );

                                        place_resp
                                    })
                                    .inner;

                                if row_response.clicked() {
                                    if let Some(place) = self.net.places.get(idx) {
                                        self.canvas.selected_place = Some(place.id);
                                        let screen_rect = ctx.available_rect();
                                        let target_x = screen_rect.left() + screen_rect.width() * 0.3;
                                        let target_y = screen_rect.center().y;
                                        self.canvas.pan = egui::vec2(
                                            target_x - place.pos[0] * self.canvas.zoom,
                                            target_y - place.pos[1] * self.canvas.zoom,
                                        );
                                    }
                                }
                            },
                        );
                    }
                }
            },
        );
        self.show_debug = open;
    }
}
