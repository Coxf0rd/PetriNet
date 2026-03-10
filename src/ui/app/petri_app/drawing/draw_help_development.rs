use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_help_development(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_development;
        egui::Window::new("Help: Разработка")
            .constrained_to_viewport(ctx)
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Информация о приложении");
                ui.separator();
                ui.label(egui::RichText::new(format!("Версия: {}", env!("CARGO_PKG_VERSION"))).size(20.0));
                ui.label(egui::RichText::new("Разработчик: НФФЛ").size(18.0));
                ui.separator();
                ui.label(" Любые проблемы программы игнорируются создателем. Репозиторий открытый: https://github.com/Coxf0rd/PetriNet");
            });
        self.show_help_development = open;
    }
}
