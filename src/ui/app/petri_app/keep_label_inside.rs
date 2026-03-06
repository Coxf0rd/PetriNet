use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn keep_label_inside(
        rect: Rect,
        center: Pos2,
        mut offset: Vec2,
    ) -> Vec2 {
        let candidate = center + offset;
        let margin = 8.0;
        if candidate.y > rect.bottom() - margin {
            offset.y = -offset.y.abs();
        } else if candidate.y < rect.top() + margin {
            offset.y = offset.y.abs();
        }
        if candidate.x > rect.right() - margin {
            offset.x = -offset.x.abs();
        } else if candidate.x < rect.left() + margin {
            offset.x = offset.x.abs();
        }
        offset
    }
}
