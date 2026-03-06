use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn update_markov_annotations(&mut self) {
        self.markov_annotations.clear();
        let Some(chain) = &self.markov_model else {
            return;
        };
        let expectation = chain.stationary.as_ref().map(|stationary| {
            let mut expected = vec![0.0; self.net.places.len()];
            for (state, prob) in chain.states.iter().zip(stationary.iter()) {
                for (idx, &tokens) in state.iter().enumerate().take(expected.len()) {
                    expected[idx] += *prob * tokens as f64;
                }
            }
            expected
        });
        for (idx, place) in self.net.places.iter().enumerate() {
            if !place.markov_highlight {
                continue;
            }
            let label = if let Some(expected) = expectation.as_ref() {
                format!("{} ≈ {:.3}", self.tr("π", "π"), expected[idx])
            } else {
                self.tr("Нет распределения", "No distribution").to_string()
            };
            self.markov_annotations.insert(place.id, label);
        }
    }
}
