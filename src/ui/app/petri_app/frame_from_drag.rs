use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_from_drag(
        start: [f32; 2],
        current: [f32; 2],
    ) -> ([f32; 2], f32, f32) {
        let min_x = start[0].min(current[0]);
        let min_y = start[1].min(current[1]);
        let width = (current[0] - start[0]).abs();
        let height = (current[1] - start[1]).abs();
        ([min_x, min_y], width, height)
    }
}
