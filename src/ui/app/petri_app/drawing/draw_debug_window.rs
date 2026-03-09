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
                        ui.label(format!(
                            "{}: {}",
                            t("Переход", "Transition"),
                            entry
                                .fired_transition
                                .map(|i| format!("T{}", i + 1))
                                .unwrap_or_else(|| "-".to_string())
                        ));
                        // Render the marking grid using virtualized rows.  Use the scroll
                        // utilities to hide the scroll bar while constraining the height.
                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                        let row_count = entry.marking.len();
                        // Virtualized rows require explicit type annotations on the closure
                        // parameters so that the Rust compiler can infer the types.
                        scroll_utils::show_virtualized_rows(
                            ui,
                            "debug_marking_grid",
                            200.0,
                            row_h,
                            row_count,
                            |ui: &mut egui::Ui, idx: usize| {
                                egui::Grid::new("debug_marking_grid_rows")
                                    .striped(true)
                                    .show(ui, |ui: &mut egui::Ui| {
                                        ui.add_sized([72.0, 0.0], egui::Label::new(format!("P{}", idx + 1)));
                                        ui.add_sized([84.0, 0.0], egui::Label::new(entry.marking[idx].to_string()));
                                        ui.end_row();
                                    });
                            },
                        );
                    }
                }
            },
        );
        self.show_debug = open;
    }
}
