use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let desired = ui.available_size_before_wrap();
        let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
        let painter = ui.painter_at(rect);

        let step = self.grid_step_world();
        let mut x = rect.left();
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(225)),
            );
            x += step;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(225)),
            );
            y += step;
        }

        ui.allocate_ui_at_rect(rect.shrink(6.0), |ui| {
            if self.show_table_view {
                self.draw_table_view(ui);
            }
        });
    }
}
