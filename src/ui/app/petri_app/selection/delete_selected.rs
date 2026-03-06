use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn delete_selected(&mut self) {
        let text_ids = self.collect_selected_text_ids();
        if !text_ids.is_empty() {
            self.push_undo_snapshot();
            let text_set: HashSet<u64> = text_ids.into_iter().collect();
            self.text_blocks.retain(|item| !text_set.contains(&item.id));
            self.canvas.selected_texts.clear();
            self.canvas.selected_text = None;
            return;
        }
        let frame_ids = self.collect_selected_frame_ids();
        if !frame_ids.is_empty() {
            self.push_undo_snapshot();
            let frame_set: HashSet<u64> = frame_ids.into_iter().collect();
            self.decorative_frames
                .retain(|item| !frame_set.contains(&item.id));
            self.canvas.selected_frames.clear();
            self.canvas.selected_frame = None;
            return;
        }
        let mut arc_ids = self.canvas.selected_arcs.clone();
        if let Some(arc_id) = self.canvas.selected_arc.take() {
            arc_ids.push(arc_id);
        }
        arc_ids.sort_unstable();
        arc_ids.dedup();
        if !arc_ids.is_empty() {
            self.canvas.selected_arcs.clear();
            self.push_undo_snapshot();
            self.net.arcs.retain(|a| !arc_ids.contains(&a.id));
            self.net.inhibitor_arcs.retain(|a| !arc_ids.contains(&a.id));
            self.net.rebuild_matrices_from_arcs();
            return;
        }

        let mut place_ids = self.canvas.selected_places.clone();
        let mut transition_ids = self.canvas.selected_transitions.clone();
        if let Some(id) = self.canvas.selected_place {
            place_ids.push(id);
        }
        if let Some(id) = self.canvas.selected_transition {
            transition_ids.push(id);
        }
        place_ids.sort_unstable();
        place_ids.dedup();
        transition_ids.sort_unstable();
        transition_ids.dedup();

        if !place_ids.is_empty() || !transition_ids.is_empty() {
            self.push_undo_snapshot();
            let mut place_idxs: Vec<usize> = place_ids
                .iter()
                .filter_map(|id| self.place_idx_by_id(*id))
                .collect();
            place_idxs.sort_unstable();
            place_idxs.dedup();
            for idx in place_idxs.iter().rev() {
                self.net.tables.remove_place_row(*idx);
            }
            let mut transition_idxs: Vec<usize> = transition_ids
                .iter()
                .filter_map(|id| self.transition_idx_by_id(*id))
                .collect();
            transition_idxs.sort_unstable();
            transition_idxs.dedup();
            for idx in transition_idxs.iter().rev() {
                self.net.tables.remove_transition_column(*idx);
            }
            self.net.places.retain(|p| !place_ids.contains(&p.id));
            self.net
                .transitions
                .retain(|t| !transition_ids.contains(&t.id));
            self.net
                .set_counts(self.net.places.len(), self.net.transitions.len());
            self.clear_selection();
        }
    }
}
