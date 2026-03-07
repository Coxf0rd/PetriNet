use std::collections::HashMap;

use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn markov_expected_tokens(
        chain: &MarkovChain,
        place_count: usize,
    ) -> Option<Vec<f64>> {
        let stationary = chain.stationary.as_ref()?;
        let mut expected = vec![0.0; place_count];
        for (state, prob) in chain.states.iter().zip(stationary.iter()) {
            for (idx, &tokens) in state.iter().enumerate().take(place_count) {
                expected[idx] += *prob * tokens as f64;
            }
        }
        Some(expected)
    }

    pub(in crate::ui::app) fn markov_tokens_distribution(
        chain: &MarkovChain,
        place_idx: usize,
    ) -> Vec<(u32, f64)> {
        let stationary = match chain.stationary.as_ref() {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut distribution = HashMap::new();
        for (state, prob) in chain.states.iter().zip(stationary.iter()) {
            let count = *state.get(place_idx).unwrap_or(&0);
            *distribution.entry(count).or_insert(0.0) += *prob;
        }
        let mut vec = distribution.into_iter().collect::<Vec<_>>();
        vec.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        vec
    }
}
