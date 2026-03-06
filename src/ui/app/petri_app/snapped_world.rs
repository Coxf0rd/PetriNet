use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn snapped_world(&self, world: [f32; 2]) -> [f32; 2] {
        if self.net.ui.snap_to_grid {
            self.snap_point_to_grid(world)
        } else {
            world
        }
    }
}
