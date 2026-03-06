use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn inhibitor_screen_endpoints(
        &self,
        rect: Rect,
        inh: &crate::model::InhibitorArc,
    ) -> Option<(Pos2, Pos2)> {
        let (Some(pi), Some(ti)) = (
            self.place_idx_by_id(inh.place_id),
            self.transition_idx_by_id(inh.transition_id),
        ) else {
            return None;
        };

        let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
        let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
        let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
        let t_rect = Rect::from_min_size(
            t_min,
            Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom,
        );
        let t_center = t_rect.center();
        let delta = t_center - p_center;
        let dir = if delta.length_sq() > 0.0 {
            delta.normalized()
        } else {
            Vec2::X
        };
        let from = p_center + dir * p_radius;
        let to = Self::rect_border_point(t_rect, -dir);

        Some((from, to))
    }
}
