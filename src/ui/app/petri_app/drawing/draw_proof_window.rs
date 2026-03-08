// This patched version of the proof window replaces the ad‑hoc `egui::Window`
// implementation with the standard `show_property_window` helper. The new
// property window respects the global project guidelines: its minimum size
// defines the base dimensions, and the default size is automatically set to
// 20% larger in both width and height. Additionally, we fix a long‑standing
// encoding bug in the Russian message for an empty trace.

use super::*;

impl PetriApp {
    /// Render the Proof window, which displays the simulation trace used to
    /// generate a proof. When the simulation has not been run, a helpful
    /// message is shown. This implementation uses the common property
    /// window template to ensure consistent look and feel across the UI.
    pub(in crate::ui::app) fn draw_proof_window(&mut self, ctx: &egui::Context) {
        if !self.show_proof {
            return;
        }
        let mut open = self.show_proof;
        // Define the base and default sizes: the default size is 20% larger
        // than the minimum size to provide a comfortable initial window.
        let min_size = egui::vec2(440.0, 360.0);
        let default_size = min_size * 1.2;
        show_property_window(
            ctx,
            // The title is localised via `self.tr` rather than a hardcoded
            // string to support both Russian and English UIs.
            self.tr("Доказательство", "Proof"),
            &mut open,
            PropertyWindowConfig::new("proof_window")
                .default_size(default_size)
                .min_size(min_size),
            |ui: &mut egui::Ui| {
                // If there is no simulation result, ask the user to run the
                // simulation first.
                let Some(result) = self.sim_result.as_ref() else {
                    ui.label(self.tr("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };

                // Show a descriptive heading explaining what the proof is.
                ui.label(self.tr(
                    "Доказательство построено по журналу состояний (trace).",
                    "Proof is generated from simulation trace.",
                ));
                ui.separator();
                let visible_steps = Self::debug_visible_log_indices(result);
                if visible_steps.is_empty() {
                    // Fix the previously garbled Russian message for an empty trace.
                    ui.label(self.tr(
                        "Трасса пуста.",
                        "Trace is empty.",
                    ));
                    return;
                }
                // Height of each row in the grid depends on the body font.
                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                // Wrap the grid and its rows in a horizontal scroll area.  This allows
                // the Marking column to extend beyond the window width while
                // keeping the header aligned with the rows.  The horizontal
                // scrollbar appears only when needed (on hover) so it doesn’t
                // clutter the UI.
                egui::ScrollArea::horizontal()
                    .id_source("proof_grid_horizontal")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                    .show(ui, |ui| {
                        // Draw the header of the grid.  We place it inside the horizontal
                        // scroll area so that it scrolls together with the row contents.
                        egui::Grid::new("proof_grid_header")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(self.tr("Шаг", "Step"));
                                ui.label(self.tr("Время", "Time"));
                                ui.label(self.tr("Сработал переход", "Fired transition"));
                                ui.label(self.tr("Маркировка", "Marking"));
                                ui.end_row();
                            });
                        // Use a vertical scroll area to show each step.  The height of
                        // the scroll area adapts to the window, but we clamp it to a
                        // reasonable size so that the header remains visible.
                        egui::ScrollArea::vertical()
                            .id_source("proof_grid_scroll")
                            .max_height(360.0)
                            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                            .show_rows(ui, row_h, visible_steps.len(), |ui, range| {
                                egui::Grid::new("proof_grid_rows")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for row_idx in range {
                                            let entry = &result.logs[visible_steps[row_idx]];
                                            ui.label(row_idx.to_string());
                                            ui.label(format!("{:.3}", entry.time));
                                            ui.label(
                                                entry
                                                    .fired_transition
                                                    .map(|i| format!("T{}", i + 1))
                                                    .unwrap_or_else(|| "-".to_string()),
                                            );
                                            // The marking can be long; wrapping it in its own
                                            // `Label` ensures it can shrink and expand as needed.  We
                                            // rely on the horizontal scroll area to allow the row to
                                            // exceed the window width.
                                            ui.label(format!("{:?}", entry.marking));
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
            },
        );
        self.show_proof = open;
    }
}