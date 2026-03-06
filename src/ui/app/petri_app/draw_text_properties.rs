use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_text_properties(&mut self, ctx: &egui::Context) {
        if !self.show_text_props {
            return;
        }
        if let Some(id) = self.canvas.selected_text {
            self.text_props_id = Some(id);
        }
        if let Some(text_id) = self.text_props_id {
            let title = self.tr("Редактирование текста", "Text Editing").to_string();
            self.show_text_props = self.draw_text_props_window(ctx, text_id, title);
        } else {
            self.show_text_props = false;
        }
    }
}
