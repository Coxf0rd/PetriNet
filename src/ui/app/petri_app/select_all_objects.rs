use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn select_all_objects(&mut self) {
        self.canvas.selected_place = None;
        self.canvas.selected_transition = None;
        self.canvas.selected_places = self.net.places.iter().map(|place| place.id).collect();
        self.canvas.selected_transitions = self.net.transitions.iter().map(|tr| tr.id).collect();
        self.canvas.selected_arcs = self.net.arcs.iter().map(|arc| arc.id).collect();
        self.canvas
            .selected_arcs
            .extend(self.net.inhibitor_arcs.iter().map(|arc| arc.id));
        self.canvas.selected_arc = self.canvas.selected_arcs.first().copied();
        self.canvas.selected_texts = self.text_blocks.iter().map(|text| text.id).collect();
        self.canvas.selected_text = self.canvas.selected_texts.first().copied();
        self.canvas.selected_frames = self
            .decorative_frames
            .iter()
            .map(|frame| frame.id)
            .collect();
        self.canvas.selected_frame = self.canvas.selected_frames.first().copied();
    }
}
