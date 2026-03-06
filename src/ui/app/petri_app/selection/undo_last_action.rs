use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn undo_last_action(&mut self) {
        let Some(state) = self.undo_stack.pop() else {
            return;
        };
        self.net = state.net;
        self.text_blocks = state.text_blocks;
        self.next_text_id = state.next_text_id;
        self.decorative_frames = state.decorative_frames;
        self.next_frame_id = state.next_frame_id;
        self.clear_selection();
    }
}
