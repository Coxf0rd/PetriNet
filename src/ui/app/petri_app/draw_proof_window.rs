use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_proof_window(&mut self, ctx: &egui::Context) {
        if !self.show_proof {
            return;
        }
        let mut open = self.show_proof;
        egui::Window::new("Proof")
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.as_ref() else {
                    ui.label(self.tr("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };
                ui.label(self.tr(
                    "Доказательство построено по журналу состояний (trace).",
                    "Proof is generated from simulation trace.",
                ));
                ui.separator();
                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                egui::Grid::new("proof_grid_header")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(self.tr("Шаг", "Step"));
                        ui.label(self.tr("Время", "Time"));
                        ui.label(self.tr("Сработал переход", "Fired transition"));
                        ui.label(self.tr("Маркировка", "Marking"));
                        ui.end_row();
                    });
                egui::ScrollArea::vertical().max_height(420.0).show_rows(
                    ui,
                    row_h,
                    result.logs.len(),
                    |ui, range| {
                        egui::Grid::new("proof_grid_rows")
                            .striped(true)
                            .show(ui, |ui| {
                                for step in range {
                                    let entry = &result.logs[step];
                                    ui.label(step.to_string());
                                    ui.label(format!("{:.3}", entry.time));
                                    ui.label(
                                        entry
                                            .fired_transition
                                            .map(|i| format!("T{}", i + 1))
                                            .unwrap_or_else(|| "-".to_string()),
                                    );
                                    ui.label(format!("{:?}", entry.marking));
                                    ui.end_row();
                                }
                            });
                    },
                );
            });
        self.show_proof = open;
    }
}
