use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_place_properties(&mut self, ctx: &egui::Context) {
        if !self.show_place_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_place
            .or_else(|| self.canvas.selected_places.last().copied())
        {
            self.place_props_id = Some(id);
        }
        if let Some(place_id) = self.place_props_id {
            let title = self
                .tr("Свойства позиции", "Position Properties")
                .to_string();
            self.show_place_props = self.draw_place_props_window(ctx, place_id, title);
        } else {
            self.show_place_props = false;
        }
    }
}
