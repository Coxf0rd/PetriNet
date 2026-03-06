use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sync_primary_selection_from_multi(&mut self) {
        self.canvas.selected_place = self.canvas.selected_places.last().copied();
        self.canvas.selected_transition = self.canvas.selected_transitions.last().copied();
        self.canvas.selected_arc = self.canvas.selected_arcs.last().copied();
        self.canvas.selected_text = self.canvas.selected_texts.last().copied();
        self.canvas.selected_frame = self.canvas.selected_frames.last().copied();
    }
}
