use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn segment_distance_to_point(pos: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        if ab.length_sq() <= f32::EPSILON {
            return pos.distance(a);
        }
        let t = ((pos - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        let proj = a + ab * t;
        proj.distance(pos)
    }
}
