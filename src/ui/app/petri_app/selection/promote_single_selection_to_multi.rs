use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn promote_single_selection_to_multi(&mut self) {
        if let Some(place_id) = self.canvas.selected_place.take() {
            if !self.canvas.selected_places.contains(&place_id) {
                self.canvas.selected_places.push(place_id);
            }
        }
        if let Some(transition_id) = self.canvas.selected_transition.take() {
            if !self.canvas.selected_transitions.contains(&transition_id) {
                self.canvas.selected_transitions.push(transition_id);
            }
        }
        if let Some(arc_id) = self.canvas.selected_arc.take() {
            if !self.canvas.selected_arcs.contains(&arc_id) {
                self.canvas.selected_arcs.push(arc_id);
            }
        }
        if let Some(text_id) = self.canvas.selected_text.take() {
            if !self.canvas.selected_texts.contains(&text_id) {
                self.canvas.selected_texts.push(text_id);
            }
        }
        if let Some(frame_id) = self.canvas.selected_frame.take() {
            if !self.canvas.selected_frames.contains(&frame_id) {
                self.canvas.selected_frames.push(frame_id);
            }
        }
    }
}
