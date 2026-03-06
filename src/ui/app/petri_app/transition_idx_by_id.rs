use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn transition_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.transitions.iter().position(|t| t.id == id)
    }
}
