use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_place_stats_dialog(&mut self, ctx: &egui::Context) {
        let Some(place_id) = self.place_stats_dialog_place_id else {
            self.place_stats_dialog_backup = None;
            return;
        };
        if !self.net.ui.marker_count_stats {
            self.place_stats_dialog_place_id = None;
            self.place_stats_dialog_backup = None;
            return;
        }
        let Some(place_idx) = self.place_idx_by_id(place_id) else {
            self.place_stats_dialog_place_id = None;
            self.place_stats_dialog_backup = None;
            return;
        };

        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(t("Статистика", "Statistics"))
            .id(egui::Id::new(("place_stats_dialog", place_id)))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("ID: P{}", place_id));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            if let Some((backup_id, backup)) = self.place_stats_dialog_backup.take()
                            {
                                if backup_id == place_id {
                                    self.net.places[place_idx].stats = backup;
                                }
                            }
                            self.place_stats_dialog_place_id = None;
                        }
                        if ui.button("Ok").clicked() {
                            self.place_stats_dialog_backup = None;
                            self.place_stats_dialog_place_id = None;
                        }
                    });
                });
                ui.separator();

                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
                        ui.label(t("Число маркеров", "Tokens"));
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.markers_total,
                            t("Общая", "Total"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.markers_input,
                            t("На входе", "On input"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.markers_output,
                            t("На выходе", "On output"),
                        );
                    });
                    cols[1].group(|ui| {
                        ui.label(t("Загруженность", "Load"));
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.load_total,
                            t("Общая", "Total"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.load_input,
                            t("Вход", "Input"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.load_output,
                            t("Выход", "Output"),
                        );
                    });
                });
            });

        if !open {
            // Treat closing via X as cancel.
            if let Some((backup_id, backup)) = self.place_stats_dialog_backup.take() {
                if backup_id == place_id {
                    self.net.places[place_idx].stats = backup;
                }
            }
            self.place_stats_dialog_place_id = None;
        }
    }
}
