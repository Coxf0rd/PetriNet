use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_help_controls(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_controls;
        egui::Window::new("Help: Помощь по управлению")
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading("Основные кнопки и комбинации");
                ui.separator();
                ui.label("ЛКМ: создать/выбрать элемент (в зависимости от активного инструмента)");
                ui.label("СКМ + перетаскивание: двигать рабочую область");
                ui.label("Delete: удалить выделенное");
                ui.separator();
                ui.label("Ctrl+N: новый файл");
                ui.label("Ctrl+O: открыть файл");
                ui.label("Ctrl+S: сохранить файл");
                ui.label("Ctrl+C: копировать выделенное");
                ui.label("Ctrl+V: вставить");
                ui.label("Ctrl+Z: отменить последнее действие");
                ui.label("Ctrl+Q: выход");
                ui.label("Ctrl + колесо: изменить масштаб графа");
            });
        self.show_help_controls = open;
    }
}
