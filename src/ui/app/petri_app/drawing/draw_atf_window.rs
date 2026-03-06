use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_atf_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_atf;
        egui::Window::new("ATF").open(&mut open).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Левая область");
                    ui.horizontal(|ui| {
                        ui.label("P:");
                        ui.add(egui::DragValue::new(&mut self.atf_selected_place).range(0..=10000));
                        if ui.button("OK").clicked() {
                            self.atf_text = generate_atf(
                                &self.net,
                                self.atf_selected_place
                                    .min(self.net.places.len().saturating_sub(1)),
                            );
                        }
                    });
                    if ui.button("Сгенерировать ATF").clicked() {
                        self.atf_text = generate_atf(
                            &self.net,
                            self.atf_selected_place
                                .min(self.net.places.len().saturating_sub(1)),
                        );
                    }
                    if ui.button("Открыть ATF файл").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("ATF", &["atf", "txt"])
                            .pick_file()
                        {
                            match fs::read_to_string(&path) {
                                Ok(text) => self.atf_text = text,
                                Err(e) => self.last_error = Some(e.to_string()),
                            }
                        }
                    }
                });
                ui.separator();
                ui.add(
                    egui::TextEdit::multiline(&mut self.atf_text)
                        .desired_rows(30)
                        .desired_width(700.0),
                );
            });
        });
        self.show_atf = open;
    }
}
