use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_idx_by_id(&self, id: u64) -> Option<usize> {
        self.text_blocks.iter().position(|item| item.id == id)
    }
}
