use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn label_offset(pos: LabelPosition, scale: f32) -> Vec2 {
        match pos {
            LabelPosition::Top => Vec2::new(0.0, -24.0 * scale),
            LabelPosition::Bottom => Vec2::new(0.0, 24.0 * scale),
            LabelPosition::Left => Vec2::new(-28.0 * scale, 0.0),
            LabelPosition::Right => Vec2::new(28.0 * scale, 0.0),
            LabelPosition::Center => Vec2::ZERO,
        }
    }
}
