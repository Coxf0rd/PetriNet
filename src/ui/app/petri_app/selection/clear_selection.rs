use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn clear_selection(&mut self) {
        self.canvas.selected_place = None;
        self.canvas.selected_transition = None;
        self.canvas.selected_places.clear();
        self.canvas.selected_transitions.clear();
        self.canvas.selected_arc = None;
        self.canvas.selected_arcs.clear();
        self.canvas.selected_text = None;
        self.canvas.selected_texts.clear();
        self.canvas.selected_frame = None;
        self.canvas.selected_frames.clear();
        self.canvas.frame_draw_start_world = None;
        self.canvas.frame_draw_current_world = None;
        self.canvas.frame_resize_id = None;
        self.canvas.selection_toggle_mode = false;
    }
}
