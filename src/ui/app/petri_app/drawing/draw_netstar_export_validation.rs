use super::*;
use crate::ui::scroll_utils;

impl PetriApp {
    /// Draw the Netstar export validation window.
    ///
    /// This implementation replaces the previous ad-hoc scroll area with a
    /// virtualized list using the `scroll_utils::show_virtualized_rows`
    /// helper.  Combining the errors and warnings into a single list and
    /// virtualizing it avoids performance issues when there are many
    /// messages.  The scroll bar appears only on hover, matching the
    /// behaviour of other list-based UI components.
    pub(in crate::ui::app) fn draw_netstar_export_validation(&mut self, ctx: &egui::Context) {
        // If the window is not supposed to be shown, exit early.
        if !self.show_netstar_export_validation {
            return;
        }
        // Clone the current report.  If it is absent, clear the state and exit.
        let Some(report) = self.netstar_export_validation.clone() else {
            self.clear_netstar_export_validation();
            return;
        };
        let mut open = self.show_netstar_export_validation;
        // Remember the target path so we can show it in the UI.
        let target_path = self.pending_netstar_export_path.clone();
        // Count errors and warnings.
        let errors = report.error_count();
        let warnings = report.warning_count();
        let mut do_export = false;
        let mut do_cancel = false;
        // Create the export validation window.
        egui::Window::new(self.tr("Проверка экспорта", "Export validation"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("netstar_export_validation_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(620.0)
            .show(ctx, |ui| {
                // Show the path of the pending export, if available.
                if let Some(path) = &target_path {
                    ui.label(format!("{} {}", self.tr("Файл:", "File:"), path.display()));
                }
                ui.separator();
                // Show error and warning counts.
                ui.label(format!(
                    "{}: {}    {}: {}",
                    self.tr("Ошибки", "Errors"),
                    errors,
                    self.tr("Предупреждения", "Warnings"),
                    warnings
                ));
                if report.is_clean() {
                    // If there are no issues, display a green message.
                    ui.colored_label(
                        Color32::from_rgb(0, 128, 0),
                        self.tr("Проблем не найдено.", "No issues found."),
                    );
                } else {
                    // Show an instruction about clicking rows.
                    ui.label(self.tr(
                        "Нажмите на строку ошибки/предупреждения, чтобы выделить объект в графе.",
                        "Click an issue row to select the related object on the graph.",
                    ));
                    // Determine the height of each row.  We base this on the
                    // default body text style plus a small padding.
                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                    // Calculate the total number of rows (errors + warnings).
                    let total_rows = report.errors.len() + report.warnings.len();
                    // Use a virtualized scroll area to display the issues.  The
                    // `scroll_utils` helper hides the scroll bar when not hovered and
                    // automatically takes care of the scroll area sizing.
                    scroll_utils::show_virtualized_rows(
                        ui,
                        "netstar_export_issues",
                        260.0,
                        row_h,
                        total_rows,
                        |ui: &mut egui::Ui, idx: usize| {
                            // Render either an error or a warning depending on the index.
                            if idx < report.errors.len() {
                                let issue = &report.errors[idx];
                                let line = format!("[{}] {}", self.tr("Ошибка", "Error"), issue);
                                let response = ui.add(
                                    egui::Label::new(egui::RichText::new(line).color(Color32::RED))
                                        .sense(egui::Sense::click()),
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
                            } else {
                                let warn_idx = idx - report.errors.len();
                                let issue = &report.warnings[warn_idx];
                                let line =
                                    format!("[{}] {}", self.tr("Предупреждение", "Warning"), issue);
                                let response = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(line)
                                            .color(Color32::from_rgb(160, 110, 0)),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if response.clicked() {
                                    let _ = self.select_export_issue_target(issue);
                                }
                            }
                            ui.add_space(2.0);
                        },
                    );
                }
                // If there are errors, inform the user that export is blocked.
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
                // Show Cancel and Export buttons.
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
        // Handle window closure and user actions.
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
