use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_transition_properties(&mut self, ctx: &egui::Context) {
        if !self.show_transition_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_transition
            .or_else(|| self.canvas.selected_transitions.last().copied())
        {
            self.transition_props_id = Some(id);
        }
        if let Some(transition_id) = self.transition_props_id {
            let title = self
                .tr("Свойства перехода", "Transition Properties")
                .to_string();
            self.show_transition_props =
                self.draw_transition_props_window(ctx, transition_id, title);
        } else {
            self.show_transition_props = false;
        }
    }
}
