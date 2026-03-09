use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn invalidate_markov_model(&mut self) {
        self.markov_model = None;
        self.markov_model_pending_compute = false;
        self.markov_limit_reached = false;
        self.markov_annotations.clear();
        self.markov_place_arcs.clear();
        self.selected_markov_arc = None;
    }

    pub(in crate::ui::app) fn calculate_markov_model(&mut self) {
        self.net.sanitize_values();
        let chain = build_markov_chain(&self.net, None);
        self.markov_limit_reached = chain.limit_reached;
        self.markov_model = Some(chain);
        self.markov_model_pending_compute = false;
        self.update_markov_annotations();
        self.refresh_markov_place_arcs();
    }
}
