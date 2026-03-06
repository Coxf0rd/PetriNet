use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn push_undo_snapshot(&mut self) {
        self.undo_stack.push(UndoSnapshot {
            net: self.net.clone(),
            text_blocks: self.text_blocks.clone(),
            next_text_id: self.next_text_id,
            decorative_frames: self.decorative_frames.clone(),
            next_frame_id: self.next_frame_id,
        });
        // Keep memory bounded.
        if self.undo_stack.len() > 64 {
            self.undo_stack.remove(0);
        }
    }
}
