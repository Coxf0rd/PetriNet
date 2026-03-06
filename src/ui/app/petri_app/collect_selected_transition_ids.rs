use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_transition_ids(&self) -> Vec<u64> {
        let mut transition_ids = self.canvas.selected_transitions.clone();
        if let Some(id) = self.canvas.selected_transition {
            transition_ids.push(id);
        }
        transition_ids.sort_unstable();
        transition_ids.dedup();
        transition_ids
    }
}
