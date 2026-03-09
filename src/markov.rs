use std::collections::{HashMap, VecDeque};

use crate::model::{PetriNet, StochasticDistribution};

const DEFAULT_MIN_MAX_STATES: usize = 2_000;
const TARGET_MARKOV_GRAPH_BYTES: usize = 256 * 1024 * 1024;
const HARD_AUTO_MAX_STATES: usize = 1_000_000;
const APPROX_STATE_OVERHEAD_BYTES: usize = 96;
const APPROX_EDGE_OVERHEAD_BYTES: usize = 24;
const APPROX_SEEN_ENTRY_BYTES: usize = 48;
const APPROX_QUEUE_ENTRY_BYTES: usize = 8;
const STATIONARY_TOLERANCE: f64 = 1e-11;
const STATIONARY_MAX_ITERS: usize = 20_000;

/// Результат распределения цепи Маркова и её графа состояний.
pub struct MarkovChain {
    pub states: Vec<Vec<u32>>,
    pub transitions: Vec<Vec<(usize, f64)>>,
    pub stationary: Option<Vec<f64>>,
    pub limit_reached: bool,
    pub state_limit: usize,
    pub stationary_status: StationaryStatus,
}

#[derive(Clone, Debug)]
pub enum StationaryStatus {
    Computed,
    LimitReached { explored_states: usize, limit: usize },
    TimedNetUnsupported,
    SolverDidNotConverge,
    NoDynamicTransitions,
}

impl MarkovChain {
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
}

#[derive(Clone, Debug, Default)]
struct TransitionSpec {
    pre: Vec<(usize, u32)>,
    post: Vec<(usize, u32)>,
    inhibitor: Vec<(usize, u32)>,
    capacity_effects: Vec<(usize, u32, u32)>,
    priority: i32,
    pre_weight: u32,
    active: bool,
}

/// Построить граф состояний и решить уравнение Кольмогорова для стационарного распределения.
/// Для timed/stochastic сетей стационарное распределение по одной только маркировке не считается,
/// потому что такое состояние системы неполное.
pub fn build_markov_chain(net: &PetriNet, max_states: Option<usize>) -> MarkovChain {
    let fixed_limit = max_states;
    let mut adaptive_limit = fixed_limit.unwrap_or(DEFAULT_MIN_MAX_STATES);
    let specs = build_transition_specs(net);
    let can_compute_stationary = !net_has_timed_behavior(net);

    let initial_marking = net.tables.m0.clone();
    let mut states = Vec::new();
    let mut transitions = Vec::new();
    let mut seen = HashMap::new();
    let mut queue = VecDeque::new();
    states.push(initial_marking.clone());
    transitions.push(Vec::new());
    seen.insert(initial_marking.clone(), 0);
    queue.push_back(0);

    let mut limit_reached = false;
    let mut total_edge_count = 0_usize;
    while let Some(idx) = queue.pop_front() {
        if states.len() >= adaptive_limit {
            if fixed_limit.is_none() {
                adaptive_limit = auto_state_limit(net, states.len(), total_edge_count, queue.len());
            }
            if states.len() >= adaptive_limit {
                limit_reached = true;
                break;
            }
        }

        let marking = states[idx].clone();
        let enabled = enabled_transition_candidates(net, &specs, &marking);
        let mut edges_by_state: HashMap<usize, f64> = HashMap::new();

        for &t in &enabled {
            let next_marking = apply_transition(&marking, &specs[t]);
            let state_id = if let Some(&id) = seen.get(&next_marking) {
                id
            } else {
                let id = states.len();
                if id >= adaptive_limit {
                    if fixed_limit.is_none() {
                        adaptive_limit = auto_state_limit(net, id + 1, total_edge_count, queue.len() + 1);
                    }
                    if id >= adaptive_limit {
                        limit_reached = true;
                        break;
                    }
                }
                states.push(next_marking.clone());
                transitions.push(Vec::new());
                seen.insert(next_marking, id);
                queue.push_back(id);
                id
            };
            *edges_by_state.entry(state_id).or_insert(0.0) += 1.0;
        }

        let mut edges: Vec<(usize, f64)> = edges_by_state.into_iter().collect();
        edges.sort_unstable_by_key(|(state_id, _)| *state_id);
        total_edge_count = total_edge_count.saturating_add(edges.len());
        transitions[idx] = edges;

        if fixed_limit.is_none() {
            adaptive_limit = auto_state_limit(net, states.len(), total_edge_count, queue.len());
        }

        if limit_reached {
            break;
        }
    }

    let (stationary, stationary_status) = if !can_compute_stationary {
        (None, StationaryStatus::TimedNetUnsupported)
    } else if limit_reached {
        (
            None,
            StationaryStatus::LimitReached {
                explored_states: states.len(),
                limit: adaptive_limit,
            },
        )
    } else {
        match compute_stationary_sparse(&transitions) {
            Some(stationary) => (Some(stationary), StationaryStatus::Computed),
            None if transitions.iter().all(|edges| edges.is_empty()) => {
                (None, StationaryStatus::NoDynamicTransitions)
            }
            None => (None, StationaryStatus::SolverDidNotConverge),
        }
    };

    MarkovChain {
        states,
        transitions,
        stationary,
        limit_reached,
        state_limit: adaptive_limit,
        stationary_status,
    }
}

fn auto_state_limit(
    net: &PetriNet,
    state_count: usize,
    edge_count: usize,
    queue_len: usize,
) -> usize {
    let states = state_count.max(1);
    let estimated_bytes = estimate_graph_bytes(net, states, edge_count, queue_len);
    let approx_bytes_per_state = estimated_bytes.checked_div(states).unwrap_or(estimated_bytes).max(1);

    let memory_budget_limit = TARGET_MARKOV_GRAPH_BYTES
        .checked_div(approx_bytes_per_state)
        .unwrap_or(DEFAULT_MIN_MAX_STATES);

    let places = net.places.len();
    let transitions = net.transitions.len();
    let arc_count = net
        .tables
        .pre
        .iter()
        .flatten()
        .chain(net.tables.post.iter().flatten())
        .chain(net.tables.inhibitor.iter().flatten())
        .filter(|&&w| w > 0)
        .count();
    let total_weight: usize = net
        .tables
        .pre
        .iter()
        .flatten()
        .chain(net.tables.post.iter().flatten())
        .chain(net.tables.inhibitor.iter().flatten())
        .map(|&w| w as usize)
        .sum();
    let initial_tokens: usize = net.tables.m0.iter().map(|&v| v as usize).sum();

    let structure_floor = DEFAULT_MIN_MAX_STATES
        .saturating_add(places.saturating_mul(24))
        .saturating_add(transitions.saturating_mul(40))
        .saturating_add(arc_count.saturating_mul(10))
        .saturating_add(initial_tokens.min(5_000).saturating_mul(2))
        .saturating_add(total_weight.min(10_000));

    memory_budget_limit
        .max(structure_floor)
        .max(states)
        .clamp(DEFAULT_MIN_MAX_STATES, HARD_AUTO_MAX_STATES)
}

fn estimate_graph_bytes(
    net: &PetriNet,
    state_count: usize,
    edge_count: usize,
    queue_len: usize,
) -> usize {
    let places = net.places.len();
    let marking_bytes = places.saturating_mul(std::mem::size_of::<u32>());
    let state_bytes = state_count.saturating_mul(marking_bytes.saturating_add(APPROX_STATE_OVERHEAD_BYTES));
    let seen_bytes = state_count.saturating_mul(marking_bytes.saturating_add(APPROX_SEEN_ENTRY_BYTES));
    let edge_bytes = edge_count.saturating_mul(APPROX_EDGE_OVERHEAD_BYTES);
    let queue_bytes = queue_len.saturating_mul(APPROX_QUEUE_ENTRY_BYTES);

    state_bytes
        .saturating_add(seen_bytes)
        .saturating_add(edge_bytes)
        .saturating_add(queue_bytes)
}

fn net_has_timed_behavior(net: &PetriNet) -> bool {
    net.tables.mz.iter().any(|&delay| delay > 0.0)
        || net
            .places
            .iter()
            .any(|place| place.stochastic != StochasticDistribution::None)
}

fn build_transition_specs(net: &PetriNet) -> Vec<TransitionSpec> {
    let places = net.places.len();
    let transitions = net.transitions.len();
    let mut specs = Vec::with_capacity(transitions);

    for t in 0..transitions {
        let mut pre = Vec::new();
        let mut post = Vec::new();
        let mut inhibitor = Vec::new();
        let mut capacity_effects = Vec::new();
        let mut pre_weight = 0_u32;

        for p in 0..places {
            let pre_w = net.tables.pre[p][t];
            let post_w = net.tables.post[p][t];
            let inh_w = net.tables.inhibitor[p][t];

            if pre_w > 0 {
                pre.push((p, pre_w));
                pre_weight = pre_weight.saturating_add(pre_w);
            }
            if post_w > 0 {
                post.push((p, post_w));
            }
            if inh_w > 0 {
                inhibitor.push((p, inh_w));
            }
            if pre_w > 0 || post_w > 0 {
                capacity_effects.push((p, pre_w, post_w));
            }
        }

        let active = !(pre.is_empty() && post.is_empty() && inhibitor.is_empty());
        specs.push(TransitionSpec {
            pre,
            post,
            inhibitor,
            capacity_effects,
            priority: *net.tables.mpr.get(t).unwrap_or(&0),
            pre_weight,
            active,
        });
    }

    specs
}

fn enabled_transition_candidates(
    net: &PetriNet,
    specs: &[TransitionSpec],
    marking: &[u32],
) -> Vec<usize> {
    let mut enabled = Vec::new();
    for (t, spec) in specs.iter().enumerate() {
        if !spec.active {
            continue;
        }
        if !transition_enabled(net, spec, marking) {
            continue;
        }
        enabled.push(t);
    }

    if enabled.is_empty() {
        return enabled;
    }

    let mut best_priority = i32::MIN;
    let mut best_pre_weight = 0_u32;
    for &t in &enabled {
        let spec = &specs[t];
        if spec.priority > best_priority {
            best_priority = spec.priority;
            best_pre_weight = spec.pre_weight;
        } else if spec.priority == best_priority {
            best_pre_weight = best_pre_weight.max(spec.pre_weight);
        }
    }

    enabled
        .into_iter()
        .filter(|&t| {
            let spec = &specs[t];
            spec.priority == best_priority && spec.pre_weight == best_pre_weight
        })
        .collect()
}

fn transition_enabled(net: &PetriNet, spec: &TransitionSpec, marking: &[u32]) -> bool {
    for &(p, need) in &spec.pre {
        if marking[p] < need {
            return false;
        }
    }

    for &(p, threshold) in &spec.inhibitor {
        if marking[p] >= threshold {
            return false;
        }
    }

    for &(p, pre_w, post_w) in &spec.capacity_effects {
        if let Some(cap) = net.tables.mo[p] {
            let after = marking[p]
                .saturating_sub(pre_w)
                .saturating_add(post_w);
            if after > cap {
                return false;
            }
        }
    }

    true
}

fn apply_transition(marking: &[u32], spec: &TransitionSpec) -> Vec<u32> {
    let mut next = marking.to_vec();
    for &(p, pre_w) in &spec.pre {
        next[p] = next[p].saturating_sub(pre_w);
    }
    for &(p, post_w) in &spec.post {
        next[p] = next[p].saturating_add(post_w);
    }
    next
}

fn compute_stationary_sparse(transitions: &[Vec<(usize, f64)>]) -> Option<Vec<f64>> {
    let n = transitions.len();
    if n == 0 {
        return Some(Vec::new());
    }
    if n == 1 {
        return Some(vec![1.0]);
    }

    let exit_rates: Vec<f64> = transitions
        .iter()
        .map(|edges| edges.iter().map(|(_, rate)| *rate).sum::<f64>())
        .collect();
    let max_exit_rate = exit_rates.iter().copied().fold(0.0_f64, f64::max);
    if max_exit_rate <= f64::EPSILON {
        return None;
    }

    let stay_probabilities: Vec<f64> = exit_rates
        .iter()
        .map(|rate| (1.0 - rate / max_exit_rate).max(0.0))
        .collect();

    let mut incoming: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for (src, edges) in transitions.iter().enumerate() {
        for &(dest, rate) in edges {
            if rate > 0.0 {
                incoming[dest].push((src, rate / max_exit_rate));
            }
        }
    }

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(n.max(1));
    let chunk_size = n.div_ceil(threads).max(1);

    let mut current = vec![1.0 / (n as f64); n];
    let mut next = vec![0.0; n];

    for _ in 0..STATIONARY_MAX_ITERS {
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for start in (0..n).step_by(chunk_size) {
                let end = (start + chunk_size).min(n);
                let current_ref = &current;
                let incoming_ref = &incoming;
                let stay_ref = &stay_probabilities;
                handles.push(scope.spawn(move || {
                    let mut chunk = Vec::with_capacity(end - start);
                    for idx in start..end {
                        let mut value = current_ref[idx] * stay_ref[idx];
                        for &(src, probability) in &incoming_ref[idx] {
                            value += current_ref[src] * probability;
                        }
                        chunk.push((idx, value));
                    }
                    chunk
                }));
            }

            for handle in handles {
                for (idx, value) in handle.join().ok()? {
                    next[idx] = value;
                }
            }
            Some(())
        })?;

        let sum: f64 = next.iter().sum();
        if sum <= f64::EPSILON {
            return None;
        }
        for value in &mut next {
            *value = (*value).max(0.0) / sum;
        }

        let diff: f64 = current
            .iter()
            .zip(next.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        std::mem::swap(&mut current, &mut next);
        next.fill(0.0);

        if diff < STATIONARY_TOLERANCE {
            return Some(current);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PetriNet;

    #[test]
    fn chain_enumerates_states() {
        let mut net = PetriNet::new();
        net.set_counts(2, 1);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[1][0] = 1;
        net.tables.mo[1] = Some(2);
        let chain = build_markov_chain(&net, Some(20));

        assert!(chain.state_count() >= 2);
        assert!(chain.transitions.iter().any(|edges| !edges.is_empty()));
        assert!(chain
            .stationary
            .as_ref()
            .map_or(false, |v| (v.iter().sum::<f64>() - 1.0).abs() < 1e-6));
    }

    #[test]
    fn priority_and_pre_weight_filter_matches_simulation_policy() {
        let mut net = PetriNet::new();
        net.set_counts(2, 3);
        net.tables.m0[0] = 2;
        net.tables.m0[1] = 1;

        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.tables.mpr[0] = 1;

        net.tables.pre[0][1] = 1;
        net.tables.pre[1][1] = 1;
        net.tables.post[0][1] = 1;
        net.tables.mpr[1] = 5;

        net.tables.pre[0][2] = 1;
        net.tables.post[0][2] = 1;
        net.tables.mpr[2] = 5;

        let specs = build_transition_specs(&net);
        let enabled = enabled_transition_candidates(&net, &specs, &net.tables.m0);
        assert_eq!(enabled, vec![1]);
    }

    #[test]
    fn stationary_solver_handles_simple_cycle() {
        let transitions = vec![vec![(1, 1.0)], vec![(0, 1.0)]];
        let stationary = compute_stationary_sparse(&transitions).expect("stationary computed");
        assert!((stationary.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!(stationary.iter().all(|v| *v >= 0.0));
    }
}
