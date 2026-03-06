use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_idx_by_id(&self, id: u64) -> Option<usize> {
        self.decorative_frames.iter().position(|item| item.id == id)
    }
}
