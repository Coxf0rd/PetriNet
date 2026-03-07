use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_netstar_export_validation(&mut self, ctx: &egui::Context) {
        if !self.show_netstar_export_validation {
            return;
        }

        let Some(report) = self.netstar_export_validation.clone() else {
            self.clear_netstar_export_validation();
            return;
        };

        let mut open = self.show_netstar_export_validation;
        let target_path = self.pending_netstar_export_path.clone();
        let errors = report.error_count();
        let warnings = report.warning_count();
        let mut do_export = false;
        let mut do_cancel = false;

        egui::Window::new(self.tr("Проверка экспорта", "Export validation"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("netstar_export_validation_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(620.0)
            .show(ctx, |ui| {
                if let Some(path) = &target_path {
                    ui.label(format!("{} {}", self.tr("Файл:", "File:"), path.display()));
                }
                ui.separator();
                ui.label(format!(
                    "{}: {}    {}: {}",
                    self.tr("Ошибки", "Errors"),
                    errors,
                    self.tr("Предупреждения", "Warnings"),
                    warnings
                ));

                if report.is_clean() {
                    ui.colored_label(
                        Color32::from_rgb(0, 128, 0),
                        self.tr("Проблем не найдено.", "No issues found."),
                    );
                } else {
                    ui.label(self.tr(
                        "Нажмите на строку ошибки/предупреждения, чтобы выделить объект в графе.",
                        "Click an issue row to select the related object on the graph.",
                    ));
                    egui::ScrollArea::vertical()
                        .max_height(260.0)
                        .show(ui, |ui| {
                            for issue in &report.errors {
                                let line = format!("[{}] {}", self.tr("Ошибка", "Error"), issue);
                                let response = ui.add(
                                    egui::Label::new(egui::RichText::new(line).color(Color32::RED))
                                        .sense(Sense::click()),
                                );
                                if response.clicked() && !self.select_export_issue_target(issue) {
                                    self.status_hint = Some(
                                        self.tr(
                                            "Не удалось определить объект по строке отчёта.",
                                            "Could not resolve target object from issue row.",
                                        )
                                        .to_string(),
                                    );
                                }
                            }
                            for issue in &report.warnings {
                                let line =
                                    format!("[{}] {}", self.tr("Предупреждение", "Warning"), issue);
                                let response = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(line)
                                            .color(Color32::from_rgb(160, 110, 0)),
                                    )
                                    .sense(Sense::click()),
                                );
                                if response.clicked() {
                                    let _ = self.select_export_issue_target(issue);
                                }
                            }
                        });
                }

                if errors > 0 {
                    ui.separator();
                    ui.colored_label(
                        Color32::RED,
                        self.tr(
                            "Экспорт заблокирован: исправьте ошибки в модели.",
                            "Export blocked: fix model errors first.",
                        ),
                    );
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(self.tr("Отмена", "Cancel")).clicked() {
                        do_cancel = true;
                    }
                    let export_label = if warnings > 0 {
                        self.tr(
                            "Экспортировать с предупреждениями",
                            "Export despite warnings",
                        )
                    } else {
                        self.tr("Экспортировать", "Export")
                    };
                    if ui
                        .add_enabled(errors == 0, egui::Button::new(export_label))
                        .clicked()
                    {
                        do_export = true;
                    }
                });
            });

        if !open {
            do_cancel = true;
        }
        if do_cancel {
            self.clear_netstar_export_validation();
        }
        if do_export {
            self.confirm_netstar_export_from_validation();
        }
    }
}
