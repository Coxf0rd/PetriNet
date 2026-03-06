use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_fully_inside_rect(sel: Rect, from: Pos2, to: Pos2) -> bool {
        if !sel.contains(from) || !sel.contains(to) {
            return false;
        }

        let arrow = to - from;
        if arrow.length_sq() <= f32::EPSILON {
            return true;
        }

        let dir = arrow.normalized();
        let left = to - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
        let right = to - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
        sel.contains(left) && sel.contains(right)
    }
}
