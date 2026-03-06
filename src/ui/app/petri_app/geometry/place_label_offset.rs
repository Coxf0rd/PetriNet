use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn place_label_offset(
        pos: LabelPosition,
        radius: f32,
        scale: f32,
    ) -> Vec2 {
        if pos == LabelPosition::Center {
            return Vec2::ZERO;
        }
        let distance = radius + 10.0 * scale;
        match pos {
            LabelPosition::Top => Vec2::new(0.0, -distance),
            LabelPosition::Bottom => Vec2::new(0.0, distance),
            LabelPosition::Left => Vec2::new(-distance, 0.0),
            LabelPosition::Right => Vec2::new(distance, 0.0),
            LabelPosition::Center => Vec2::ZERO,
        }
    }
}
