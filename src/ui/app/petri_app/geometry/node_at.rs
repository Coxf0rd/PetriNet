use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn node_at(&self, rect: Rect, pos: Pos2) -> Option<NodeRef> {
        for place in &self.net.places {
            let center = self.world_to_screen(rect, place.pos);
            if center.distance(pos) <= Self::place_radius(place.size) * self.canvas.zoom {
                return Some(NodeRef::Place(place.id));
            }
        }
        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let r = Rect::from_min_size(p, Self::transition_dimensions(tr.size) * self.canvas.zoom);
            if r.contains(pos) {
                return Some(NodeRef::Transition(tr.id));
            }
        }
        for place in &self.net.places {
            let center = self.world_to_screen(rect, place.pos);
            let radius = Self::place_radius(place.size) * self.canvas.zoom;
            let name_offset = Self::keep_label_inside(
                rect,
                center,
                Self::place_label_offset(place.text_position, radius, self.canvas.zoom),
            );
            let label_center = center + name_offset;
            let label_rect = Self::approx_text_rect(label_center, &place.name, self.canvas.zoom);
            if label_rect.contains(pos) {
                return Some(NodeRef::Place(place.id));
            }
        }
        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let dims = Self::transition_dimensions(tr.size) * self.canvas.zoom;
            let r = Rect::from_min_size(p, dims);
            let label_center = r.center() + Self::label_offset(tr.label_position, self.canvas.zoom);
            let label_rect = Self::approx_text_rect(label_center, &tr.name, self.canvas.zoom);
            if label_rect.contains(pos) {
                return Some(NodeRef::Transition(tr.id));
            }
        }
        None
    }
}
