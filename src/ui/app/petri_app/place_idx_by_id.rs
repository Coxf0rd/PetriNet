use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn place_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.places.iter().position(|p| p.id == id)
    }
}
