use std::collections::{HashMap, VecDeque};

use crate::model::PetriNet;

const DEFAULT_MAX_STATES: usize = 500;

/// Результат распределения цепи Маркова и её графа состояний.
pub struct MarkovChain {
    pub states: Vec<Vec<u32>>,
    pub transitions: Vec<Vec<(usize, f64)>>,
    pub stationary: Option<Vec<f64>>,
    pub limit_reached: bool,
}

impl MarkovChain {
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
}

/// Построить граф состояний и решить уравнение Кольмогорова для стационарного распределения.
/// Колмогоровы уравнения описывают эволюцию вероятностей дискретных цепей Маркова [Kolmogorov equations].
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
        if states.len() >= limit {
            limit_reached = true;
            break;
        }
        let marking = states[idx].clone();
        let enabled = enabled_transitions_from_marking(net, &marking);
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
                edges.push((state_id, 1.0));
            }
        }
        transitions[idx] = edges;
        if limit_reached {
            break;
        }
    }

    let generator = build_generator_matrix(&transitions);
    let stationary = compute_stationary(&generator);
    MarkovChain {
        states,
        transitions,
        stationary,
        limit_reached,
    }
}

fn enabled_transitions_from_marking(net: &PetriNet, marking: &[u32]) -> Vec<usize> {
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
    enabled
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
    for row in 0..n - 1 {
        rhs[row] = 0.0;
    }
    for col in 0..n {
        matrix[n - 1][col] = 1.0;
    }
    rhs[n - 1] = 1.0;
    gaussian_elimination(&mut matrix, &mut rhs)
        .map(|mut solution| {
            let sum: f64 = solution.iter().sum();
            if sum > 0.0 {
                for v in solution.iter_mut() {
                    *v = (*v).max(0.0) / sum;
                }
            }
            solution
        })
        .or_else(|| uniform_stationary(n))
}

fn uniform_stationary(n: usize) -> Option<Vec<f64>> {
    if n == 0 {
        Some(Vec::new())
    } else {
        Some(vec![1.0 / (n as f64); n])
    }
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
        assert!(chain
            .stationary
            .as_ref()
            .map_or(false, |v| (v.iter().sum::<f64>() - 1.0).abs() < 1e-6));
    }

    #[test]
    fn stationary_solver_handles_linear_system() {
        let generator = vec![vec![-0.5, 0.5], vec![0.25, -0.25]];
        let stationary = compute_stationary(&generator).expect("stationary computed");
        assert!((stationary.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!(stationary.iter().all(|v| *v >= 0.0));
    }
}
