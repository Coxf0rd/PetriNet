use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn select_export_issue_target(&mut self, issue: &str) -> bool {
        let mut arc_candidate: Option<u64> = None;
        let mut place_candidate: Option<u64> = None;
        let mut transition_candidate: Option<u64> = None;

        for token in issue.split(|c: char| !c.is_ascii_alphanumeric()) {
            if token.len() < 2 {
                continue;
            }
            let (prefix, rest) = token.split_at(1);
            let Ok(id) = rest.parse::<u64>() else {
                continue;
            };
            match prefix {
                "A" | "a" => arc_candidate = Some(id),
                "P" | "p" => place_candidate = Some(id),
                "T" | "t" => transition_candidate = Some(id),
                _ => {}
            }
        }

        if let Some(arc_id) = arc_candidate {
            let arc_exists = self.net.arcs.iter().any(|a| a.id == arc_id)
                || self.net.inhibitor_arcs.iter().any(|a| a.id == arc_id);
            if arc_exists {
                self.clear_selection();
                self.canvas.selected_arc = Some(arc_id);
                self.canvas.selected_arcs.push(arc_id);
                return true;
            }
        }

        if let Some(place_ref) = place_candidate {
            let by_id = self.place_idx_by_id(place_ref);
            let by_ordinal = place_ref
                .checked_sub(1)
                .and_then(|idx| usize::try_from(idx).ok())
                .filter(|&idx| idx < self.net.places.len());
            if let Some(idx) = by_id.or(by_ordinal) {
                let place_id = self.net.places[idx].id;
                self.clear_selection();
                self.canvas.selected_place = Some(place_id);
                self.canvas.selected_places.push(place_id);
                self.place_props_id = Some(place_id);
                self.show_place_props = true;
                return true;
            }
        }

        if let Some(transition_ref) = transition_candidate {
            let by_id = self.transition_idx_by_id(transition_ref);
            let by_ordinal = transition_ref
                .checked_sub(1)
                .and_then(|idx| usize::try_from(idx).ok())
                .filter(|&idx| idx < self.net.transitions.len());
            if let Some(idx) = by_id.or(by_ordinal) {
                let transition_id = self.net.transitions[idx].id;
                self.clear_selection();
                self.canvas.selected_transition = Some(transition_id);
                self.canvas.selected_transitions.push(transition_id);
                self.transition_props_id = Some(transition_id);
                self.show_transition_props = true;
                return true;
            }
        }

        false
    }
}
