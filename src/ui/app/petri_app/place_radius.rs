use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn place_radius(size: VisualSize) -> f32 {
        match size {
            VisualSize::Small => 14.0,
            VisualSize::Medium => 20.0,
            VisualSize::Large => 28.0,
        }
    }
}
