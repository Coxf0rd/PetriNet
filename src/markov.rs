use std::collections::{HashMap, VecDeque};

use crate::model::{PetriNet, StochasticDistribution};

const DEFAULT_MAX_STATES: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationaryStatus {
    Available,
    StateLimitReached,
    TimedSemanticsUnsupported,
    SolverFailed,
}

/// Результат распределения цепи Маркова и её графа состояний.
pub struct MarkovChain {
    pub states: Vec<Vec<u32>>,
    pub transitions: Vec<Vec<(usize, f64)>>,
    pub stationary: Option<Vec<f64>>,
    pub limit_reached: bool,
    pub stationary_status: StationaryStatus,
}

impl MarkovChain {
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
}

/// Построить граф состояний и решить уравнение Кольмогорова для стационарного распределения.
///
/// Важно: в текущем проекте симулятор поддерживает задержки, стохастические распределения и
/// ожидающие release-токены. Для таких сетей конечная стационарная марковская модель по одной
/// лишь маркировке мест не соответствует фактической динамике симулятора, поэтому стационарное
/// распределение для timed/stochastic сетей намеренно не вычисляется.
pub fn build_markov_chain(net: &PetriNet, max_states: Option<usize>) -> MarkovChain {
    let limit = max_states.unwrap_or(DEFAULT_MAX_STATES);
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
    while let Some(idx) = queue.pop_front() {
        let marking = states[idx].clone();
        let enabled = enabled_transition_candidates_from_marking(net, &marking);
        let edge_probability = if enabled.is_empty() {
            0.0
        } else {
            1.0 / enabled.len() as f64
        };
        let mut edges = Vec::new();
        for &t in &enabled {
            if let Some(next_marking) = apply_transition(net, &marking, t) {
                let state_id = if let Some(&id) = seen.get(&next_marking) {
                    id
                } else {
                    let id = states.len();
                    if id >= limit {
                        limit_reached = true;
                        break;
                    }
                    states.push(next_marking.clone());
                    transitions.push(Vec::new());
                    seen.insert(next_marking.clone(), id);
                    queue.push_back(id);
                    id
                };
                edges.push((state_id, edge_probability));
            }
        }
        transitions[idx] = edges;
        if limit_reached {
            break;
        }
    }

    let stationary_status = if limit_reached {
        StationaryStatus::StateLimitReached
    } else if net_has_timed_semantics(net) {
        StationaryStatus::TimedSemanticsUnsupported
    } else {
        let generator = build_generator_matrix(&transitions);
        if compute_stationary(&generator).is_some() {
            StationaryStatus::Available
        } else {
            StationaryStatus::SolverFailed
        }
    };

    let stationary = if stationary_status == StationaryStatus::Available {
        let generator = build_generator_matrix(&transitions);
        compute_stationary(&generator)
    } else {
        None
    };

    MarkovChain {
        states,
        transitions,
        stationary,
        limit_reached,
        stationary_status,
    }
}

fn net_has_timed_semantics(net: &PetriNet) -> bool {
    net.tables.mz.iter().any(|&delay| delay > 0.0)
        || net.places.iter().any(|place| place.stochastic != StochasticDistribution::None)
}

fn enabled_transition_candidates_from_marking(net: &PetriNet, marking: &[u32]) -> Vec<usize> {
    let places = net.places.len();
    let mut enabled = Vec::new();
    for t in 0..net.transitions.len() {
        let mut has_arc = false;
        for p in 0..places {
            if net.tables.pre[p][t] > 0
                || net.tables.post[p][t] > 0
                || net.tables.inhibitor[p][t] > 0
            {
                has_arc = true;
                break;
            }
        }
        if !has_arc {
            continue;
        }
        let mut ok = true;
        for p in 0..places {
            let need = net.tables.pre[p][t];
            if marking[p] < need {
                ok = false;
                break;
            }
            let inh = net.tables.inhibitor[p][t];
            if inh > 0 && marking[p] >= inh {
                ok = false;
                break;
            }
            if let Some(cap) = net.tables.mo[p] {
                let after = marking[p]
                    .saturating_sub(need)
                    .saturating_add(net.tables.post[p][t]);
                if after > cap {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            enabled.push(t);
        }
    }

    select_transition_candidates(net, &enabled)
}

fn select_transition_candidates(net: &PetriNet, enabled: &[usize]) -> Vec<usize> {
    if enabled.is_empty() {
        return Vec::new();
    }

    let mut best_priority = i32::MIN;
    let mut best_pre_weight = 0_u32;
    for &t in enabled {
        let priority = *net.tables.mpr.get(t).unwrap_or(&0);
        let pre_weight = transition_pre_weight(net, t);
        if priority > best_priority {
            best_priority = priority;
            best_pre_weight = pre_weight;
        } else if priority == best_priority {
            best_pre_weight = best_pre_weight.max(pre_weight);
        }
    }

    let mut candidates: Vec<usize> = enabled
        .iter()
        .copied()
        .filter(|&t| {
            *net.tables.mpr.get(t).unwrap_or(&0) == best_priority
                && transition_pre_weight(net, t) == best_pre_weight
        })
        .collect();
    candidates.sort_unstable();
    candidates
}

fn transition_pre_weight(net: &PetriNet, transition_idx: usize) -> u32 {
    net.tables
        .pre
        .iter()
        .filter_map(|row| row.get(transition_idx).copied())
        .sum()
}

fn apply_transition(net: &PetriNet, marking: &[u32], t: usize) -> Option<Vec<u32>> {
    let mut next = marking.to_vec();
    for p in 0..net.places.len() {
        next[p] = next[p].saturating_sub(net.tables.pre[p][t]);
    }
    for p in 0..net.places.len() {
        next[p] = next[p].saturating_add(net.tables.post[p][t]);
    }
    Some(next)
}

fn build_generator_matrix(transitions: &[Vec<(usize, f64)>]) -> Vec<Vec<f64>> {
    let n = transitions.len();
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        let mut sum = 0.0;
        for &(dest, rate) in &transitions[i] {
            matrix[i][dest] += rate;
            sum += rate;
        }
        matrix[i][i] = -sum;
    }
    matrix
}

fn compute_stationary(generator: &[Vec<f64>]) -> Option<Vec<f64>> {
    let n = generator.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = generator[j][i];
        }
    }
    let mut rhs = vec![0.0; n];
    for col in 0..n {
        matrix[n - 1][col] = 1.0;
    }
    rhs[n - 1] = 1.0;
    gaussian_elimination(&mut matrix, &mut rhs).map(|mut solution| {
        let sum: f64 = solution.iter().sum();
        if sum > 0.0 {
            for v in &mut solution {
                *v = (*v).max(0.0) / sum;
            }
        }
        solution
    })
}

fn gaussian_elimination(matrix: &mut [Vec<f64>], rhs: &mut [f64]) -> Option<Vec<f64>> {
    let n = matrix.len();
    for i in 0..n {
        let mut pivot = i;
        for row in (i + 1)..n {
            if matrix[row][i].abs() > matrix[pivot][i].abs() {
                pivot = row;
            }
        }
        if matrix[pivot][i].abs() < 1e-12 {
            return None;
        }
        if pivot != i {
            matrix.swap(pivot, i);
            rhs.swap(pivot, i);
        }
        let diag = matrix[i][i];
        for col in i..n {
            matrix[i][col] /= diag;
        }
        rhs[i] /= diag;
        for row in 0..n {
            if row == i {
                continue;
            }
            let factor = matrix[row][i];
            for col in i..n {
                matrix[row][col] -= factor * matrix[i][col];
            }
            rhs[row] -= factor * rhs[i];
        }
    }
    Some(rhs.to_vec())
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
        assert!(chain.stationary.as_ref().is_some_and(|v| (v.iter().sum::<f64>() - 1.0).abs() < 1e-6));
        assert_eq!(chain.stationary_status, StationaryStatus::Available);
    }

    #[test]
    fn priority_filter_matches_simulator_rules() {
        let mut net = PetriNet::new();
        net.set_counts(3, 2);
        net.tables.m0[0] = 1;
        net.tables.m0[1] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[2][0] = 1;
        net.tables.pre[0][1] = 1;
        net.tables.pre[1][1] = 1;
        net.tables.post[2][1] = 1;
        net.tables.mpr[0] = 5;
        net.tables.mpr[1] = 5;

        let enabled = enabled_transition_candidates_from_marking(&net, &net.tables.m0);
        assert_eq!(enabled, vec![1]);
    }

    #[test]
    fn timed_nets_disable_stationary_distribution() {
        let mut net = PetriNet::new();
        net.set_counts(2, 1);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[1][0] = 1;
        net.tables.mz[1] = 1.0;

        let chain = build_markov_chain(&net, Some(20));
        assert!(chain.stationary.is_none());
        assert_eq!(chain.stationary_status, StationaryStatus::TimedSemanticsUnsupported);
    }

    #[test]
    fn stationary_solver_handles_linear_system() {
        let generator = vec![vec![-0.5, 0.5], vec![0.25, -0.25]];
        let stationary = compute_stationary(&generator).expect("stationary computed");
        assert!((stationary.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!(stationary.iter().all(|v| *v >= 0.0));
    }
}
