use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn snap_point_to_grid(&self, p: [f32; 2]) -> [f32; 2] {
        [
            self.snap_scalar_to_grid(p[0]),
            self.snap_scalar_to_grid(p[1]),
        ]
    }
}
