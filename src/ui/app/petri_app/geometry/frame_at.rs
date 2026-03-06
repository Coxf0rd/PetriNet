use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        self.decorative_frames
            .iter()
            .rev()
            .find(|frame| {
                let min = self.world_to_screen(rect, frame.pos);
                let size = Vec2::new(
                    frame.width.max(Self::FRAME_MIN_SIDE),
                    frame.height.max(Self::FRAME_MIN_SIDE),
                ) * self.canvas.zoom;
                let r = Rect::from_min_size(min, size);
                let tolerance = (6.0 * self.canvas.zoom).max(4.0);
                r.expand(tolerance).contains(pos) && !r.shrink(tolerance).contains(pos)
            })
            .map(|frame| frame.id)
    }
}
