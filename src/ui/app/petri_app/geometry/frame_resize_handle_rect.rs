use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_resize_handle_rect(
        &self,
        rect: Rect,
        frame: &CanvasFrame,
    ) -> Rect {
        let min = self.world_to_screen(rect, frame.pos);
        let width = frame.width.max(Self::FRAME_MIN_SIDE) * self.canvas.zoom;
        let height = frame.height.max(Self::FRAME_MIN_SIDE) * self.canvas.zoom;
        let handle = Self::FRAME_RESIZE_HANDLE_PX;
        let center = Pos2::new(min.x + width, min.y + height);
        Rect::from_center_size(center, Vec2::splat(handle))
    }
}
