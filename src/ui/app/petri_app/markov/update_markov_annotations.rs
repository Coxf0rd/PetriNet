use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn update_markov_annotations(&mut self) {
        self.markov_annotations.clear();
        let Some(chain) = &self.markov_model else {
            return;
        };
        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());
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
