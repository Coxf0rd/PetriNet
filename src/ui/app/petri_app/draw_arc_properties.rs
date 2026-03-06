use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_arc_properties(&mut self, ctx: &egui::Context) {
        if !self.show_arc_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_arc
            .or_else(|| self.canvas.selected_arcs.last().copied())
        {
            self.arc_props_id = Some(id);
        }
        if let Some(arc_id) = self.arc_props_id {
            let title = self
                .tr("РЎРІРѕР№СЃС‚РІР° РґСѓРіРё", "Arc Properties")
                .to_string();
            self.show_arc_props = self.draw_arc_props_window(ctx, arc_id, title);
        } else {
            self.show_arc_props = false;
        }
    }
}
