use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn new_file(&mut self) {
        self.net = PetriNet::new();
        self.net.set_counts(0, 0);
        self.file_path = None;
        self.text_blocks.clear();
        self.next_text_id = 1;
        self.decorative_frames.clear();
        self.next_frame_id = 1;
        self.undo_stack.clear();
        self.legacy_export_hints = None;
        self.status_hint = None;
        self.show_netstar_export_validation = false;
        self.pending_netstar_export_path = None;
        self.netstar_export_validation = None;
        self.markov_model = None;
        self.markov_limit_reached = false;
        self.markov_annotations.clear();
        self.show_markov_window = false;
        self.canvas.cursor_valid = false;
    }
}
