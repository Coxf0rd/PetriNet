use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn inhibitor_arc_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.inhibitor_arcs.iter().position(|arc| arc.id == id)
    }
}
