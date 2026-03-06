use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn snap_scalar_to_grid(&self, v: f32) -> f32 {
        let step = self.grid_step_world();
        (v / step).round() * step
    }
}
