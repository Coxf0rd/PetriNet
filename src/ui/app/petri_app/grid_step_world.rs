use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn grid_step_world(&self) -> f32 {
        if self.net.ui.snap_to_grid {
            Self::GRID_STEP_SNAP
        } else {
            Self::GRID_STEP_FREE
        }
    }
}
