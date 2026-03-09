use crate::ui::app::MarkovPlaceArc;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn markov_expected_tokens(
        chain: &MarkovChain,
        place_count: usize,
    ) -> Option<Vec<f64>> {
        let weights = Self::chain_state_weights(chain);
        if weights.is_empty() {
            return None;
        }
        let mut expected = vec![0.0; place_count];
        for (state, prob) in chain.states.iter().zip(weights.iter()) {
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
        let weights = Self::chain_state_weights(chain);
        if weights.is_empty() {
            return Vec::new();
        }
        let mut distribution = HashMap::new();
        for (state, prob) in chain.states.iter().zip(weights.iter()) {
            let count = *state.get(place_idx).unwrap_or(&0);
            *distribution.entry(count).or_insert(0.0) += *prob;
        }
        let mut vec = distribution.into_iter().collect::<Vec<_>>();
        vec.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        vec
    }

    pub(in crate::ui::app) fn refresh_markov_place_arcs(&mut self) {
        if let Some(chain) = self.markov_model.as_ref() {
            self.markov_place_arcs = self.build_markov_place_arcs(chain);
        } else {
            self.markov_place_arcs.clear();
        }
        if self
            .selected_markov_arc
            .is_some_and(|idx| idx >= self.markov_place_arcs.len())
        {
            self.selected_markov_arc = None;
        }
    }

    fn build_markov_place_arcs(&self, chain: &MarkovChain) -> Vec<MarkovPlaceArc> {
        let mut arcs = HashMap::new();
        let state_weights = Self::chain_state_weights(chain);
        for (state_idx, edges) in chain.transitions.iter().enumerate() {
            let state_prob = *state_weights.get(state_idx).unwrap_or(&0.0);
            if state_prob <= 0.0 {
                continue;
            }
            let src_marking = &chain.states[state_idx];
            for &(dest_idx, rate) in edges {
                if rate <= 0.0 {
                    continue;
                }
                let dest_marking = &chain.states[dest_idx];
                let weight = state_prob * rate;
                let (consumed, produced) = Self::markov_places_delta(src_marking, dest_marking);
                if consumed.is_empty() {
                    continue;
                }
                let from_places = consumed
                    .into_iter()
                    .filter(|&idx| self.net.places[idx].show_markov_model)
                    .collect::<Vec<_>>();
                if from_places.is_empty() {
                    continue;
                }
                let pair_count = from_places.len() * produced.len().max(1);
                let contribution = weight / pair_count as f64;
                for from_idx in from_places {
                    if produced.is_empty() {
                        let key = (self.net.places[from_idx].id, None);
                        *arcs.entry(key).or_insert(0.0) += contribution;
                    } else {
                        for &to_idx in &produced {
                            let key = (
                                self.net.places[from_idx].id,
                                Some(self.net.places[to_idx].id),
                            );
                            *arcs.entry(key).or_insert(0.0) += contribution;
                        }
                    }
                }
            }
        }
        let mut result = arcs
            .into_iter()
            .map(|((from, to), probability)| MarkovPlaceArc {
                from_place_id: from,
                to_place_id: to,
                probability,
            })
            .collect::<Vec<_>>();
        let total: f64 = result.iter().map(|arc| arc.probability).sum();
        if total > 0.0 {
            for arc in &mut result {
                arc.probability /= total;
            }
        }
        result.sort_unstable_by(|a, b| {
            b.probability
                .partial_cmp(&a.probability)
                .unwrap_or(Ordering::Equal)
        });
        if result.is_empty() {
            result = self.fallback_markov_place_arcs();
        }
        result
    }

    fn fallback_markov_place_arcs(&self) -> Vec<MarkovPlaceArc> {
        let mut arcs = HashMap::new();
        let transition_count = self.net.transitions.len();
        for tr_idx in 0..transition_count {
            let mut total_pre = 0.0;
            for (place_idx, place) in self.net.places.iter().enumerate() {
                if !place.show_markov_model {
                    continue;
                }
                total_pre += self.net.tables.pre[place_idx][tr_idx] as f64;
            }
            if total_pre <= 0.0 {
                continue;
            }
            let mut output_places = Vec::new();
            for (place_idx, _) in self.net.places.iter().enumerate() {
                if self.net.tables.post[place_idx][tr_idx] > 0 {
                    output_places.push(place_idx);
                }
            }
            for (place_idx, place) in self.net.places.iter().enumerate() {
                if !place.show_markov_model {
                    continue;
                }
                let consumed = self.net.tables.pre[place_idx][tr_idx] as f64;
                if consumed <= 0.0 {
                    continue;
                }
                let share = consumed / total_pre;
                if output_places.is_empty() {
                    let key = (place.id, None);
                    *arcs.entry(key).or_insert(0.0) += share;
                } else {
                    let per_output = share / output_places.len() as f64;
                    for &to_idx in &output_places {
                        let key = (place.id, Some(self.net.places[to_idx].id));
                        *arcs.entry(key).or_insert(0.0) += per_output;
                    }
                }
            }
        }
        let mut result = arcs
            .into_iter()
            .map(|((from, to), probability)| MarkovPlaceArc {
                from_place_id: from,
                to_place_id: to,
                probability,
            })
            .collect::<Vec<_>>();
        let total: f64 = result.iter().map(|arc| arc.probability).sum();
        if total > 0.0 {
            for arc in &mut result {
                arc.probability /= total;
            }
        }
        result
    }

    fn chain_state_weights(chain: &MarkovChain) -> Vec<f64> {
        let mut weights = chain
            .stationary
            .as_ref()
            .cloned()
            .unwrap_or_else(|| vec![1.0; chain.states.len()]);
        let total: f64 = weights.iter().sum();
        if total > 0.0 {
            for w in weights.iter_mut() {
                *w /= total;
            }
        }
        weights
    }

    fn markov_places_delta(src: &[u32], dest: &[u32]) -> (Vec<usize>, Vec<usize>) {
        let mut consumed = Vec::new();
        let mut produced = Vec::new();
        for (idx, (&before, &after)) in src.iter().zip(dest.iter()).enumerate() {
            if before > after {
                consumed.push(idx);
            } else if after > before {
                produced.push(idx);
            }
        }
        (consumed, produced)
    }
}
