use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn calculate_markov_model(&mut self) {
        self.net.sanitize_values();
        let chain = build_markov_chain(&self.net, Some(500));
        self.markov_limit_reached = chain.limit_reached;
        self.markov_model = Some(chain);
        self.update_markov_annotations();
        self.refresh_markov_place_arcs();
    }
}
