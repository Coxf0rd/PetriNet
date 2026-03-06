use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn screen_to_world(&self, rect: Rect, p: Pos2) -> [f32; 2] {
        [
            (p.x - rect.left() - self.canvas.pan.x) / self.canvas.zoom,
            (p.y - rect.top() - self.canvas.pan.y) / self.canvas.zoom,
        ]
    }
}
