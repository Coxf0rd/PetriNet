use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.arcs.iter().position(|arc| arc.id == id)
    }
}
