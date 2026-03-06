use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_screen_endpoints(
        &self,
        rect: Rect,
        arc: &crate::model::Arc,
    ) -> Option<(Pos2, Pos2)> {
        let (from_center, from_radius, from_rect, to_center, to_radius, to_rect) =
            match (arc.from, arc.to) {
                (NodeRef::Place(p), NodeRef::Transition(t)) => {
                    let (Some(pi), Some(ti)) =
                        (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                    else {
                        return None;
                    };
                    let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                    let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                    let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                    let t_rect = Rect::from_min_size(
                        t_min,
                        Self::transition_dimensions(self.net.transitions[ti].size)
                            * self.canvas.zoom,
                    );
                    (
                        p_center,
                        Some(p_radius),
                        None,
                        t_rect.center(),
                        None,
                        Some(t_rect),
                    )
                }
                (NodeRef::Transition(t), NodeRef::Place(p)) => {
                    let (Some(pi), Some(ti)) =
                        (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                    else {
                        return None;
                    };
                    let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                    let t_rect = Rect::from_min_size(
                        t_min,
                        Self::transition_dimensions(self.net.transitions[ti].size)
                            * self.canvas.zoom,
                    );
                    let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                    let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                    (
                        t_rect.center(),
                        None,
                        Some(t_rect),
                        p_center,
                        Some(p_radius),
                        None,
                    )
                }
                _ => return None,
            };

        let mut from = from_center;
        let mut to = to_center;
        let delta = to_center - from_center;
        let dir = if delta.length_sq() > 0.0 {
            delta.normalized()
        } else {
            Vec2::X
        };

        if let Some(radius) = from_radius {
            from += dir * radius;
        } else if let Some(r) = from_rect {
            from = Self::rect_border_point(r, dir);
        }

        if let Some(radius) = to_radius {
            to -= dir * radius;
        } else if let Some(r) = to_rect {
            to = Self::rect_border_point(r, -dir);
        }

        Some((from, to))
    }
}
