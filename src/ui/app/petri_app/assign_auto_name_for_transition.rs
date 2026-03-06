use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn assign_auto_name_for_transition(&mut self, transition_id: u64) {
        let mut ids: Vec<u64> = self.net.transitions.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let rank = ids
            .iter()
            .position(|&id| id == transition_id)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| self.net.transitions.len().max(1));
        let new_name = format!("T{rank}");
        if let Some(index) = self.transition_idx_by_id(transition_id) {
            self.net.transitions[index].name = new_name;
        }
    }
}
