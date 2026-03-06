use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        let mut best_id = None;
        // Keep arc hit-test tighter so node clicks near edges still select the node.
        let mut best_dist = 12.0_f32;

        for arc in &self.net.arcs {
            if !self.arc_visible_by_mode(arc.color, arc.visible) {
                continue;
            }
            let Some((a, b)) = self.arc_screen_endpoints(rect, arc) else {
                continue;
            };
            let dist = Self::segment_distance_to_point(pos, a, b);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(arc.id);
            }
        }

        for inh in &self.net.inhibitor_arcs {
            if !self.arc_visible_by_mode(inh.color, inh.visible) {
                continue;
            }
            let Some((a, b)) = self.inhibitor_screen_endpoints(rect, inh) else {
                continue;
            };
            let dist = Self::segment_distance_to_point(pos, a, b);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(inh.id);
            }
        }

        best_id
    }
}
