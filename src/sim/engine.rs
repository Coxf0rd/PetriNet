use std::collections::HashMap;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};

use crate::model::{PetriNet, StochasticDistribution};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StopConditions {
    pub through_place: Option<(usize, u64)>,
    pub sim_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    pub use_time_limit: bool,
    pub time_limit_sec: f64,
    pub use_pass_limit: bool,
    pub pass_limit: u64,
    pub dt: f64,
    pub display_range_start: usize,
    pub display_range_end: usize,
    pub stop: StopConditions,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            use_time_limit: false,
            time_limit_sec: 100.0,
            use_pass_limit: false,
            pass_limit: 1000,
            dt: 0.1,
            display_range_start: 0,
            display_range_end: 0,
            stop: StopConditions::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: f64,
    pub fired_transition: Option<usize>,
    pub marking: Vec<u32>,
    pub touched_places: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceStats {
    pub min: u32,
    pub max: u32,
    pub avg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub cycle_time: Option<f64>,
    pub logs: Vec<LogEntry>,
    pub place_stats: Option<Vec<PlaceStats>>,
    pub fired_count: u64,
    pub final_marking: Vec<u32>,
}

#[derive(Debug, Clone)]
struct SimState {
    available: Vec<u32>,
    pending_release: Vec<Vec<f64>>,
    through_place_counter: Vec<u64>,
}

impl SimState {
    fn total_marking(&self) -> Vec<u32> {
        self.available
            .iter()
            .enumerate()
            .map(|(p, a)| *a + self.pending_release[p].len() as u32)
            .collect()
    }

    fn process_releases(&mut self, now: f64) {
        for p in 0..self.pending_release.len() {
            let mut still_pending = Vec::with_capacity(self.pending_release[p].len());
            for release_time in self.pending_release[p].drain(..) {
                if release_time <= now {
                    self.available[p] = self.available[p].saturating_add(1);
                } else {
                    still_pending.push(release_time);
                }
            }
            self.pending_release[p] = still_pending;
        }
    }

    fn next_release_time(&self) -> Option<f64> {
        self.pending_release
            .iter()
            .flat_map(|items| items.iter().copied())
            .reduce(f64::min)
    }
}

pub fn run_simulation(net: &PetriNet, params: &SimulationParams, _fixed_step: bool, collect_stats: bool) -> SimulationResult {
    let places = net.places.len();
    let mut state = SimState {
        available: net.tables.m0.clone(),
        pending_release: vec![Vec::new(); places],
        through_place_counter: vec![0; places],
    };

    let mut now = 0.0;
    let mut passes = 0_u64;
    let mut logs = Vec::new();
    // Deterministic by default: makes tests and bug reports reproducible.
    let mut rng = SmallRng::seed_from_u64(0x5EED_5EED);
    let mut seen_markings: HashMap<Vec<u32>, f64> = HashMap::new();
    let mut cycle_time = None;

    let mut stats_acc = vec![0_f64; places];
    let mut stats_min = vec![u32::MAX; places];
    let mut stats_max = vec![0_u32; places];

    loop {
        state.process_releases(now);
        let marking = state.total_marking();

        if cycle_time.is_none() {
            if let Some(prev) = seen_markings.insert(marking.clone(), now) {
                cycle_time = Some((now - prev).max(0.0));
            }
        }

        if collect_stats {
            for p in 0..places {
                let m = marking[p];
                stats_min[p] = stats_min[p].min(m);
                stats_max[p] = stats_max[p].max(m);
                stats_acc[p] += m as f64;
            }
        }

        let enabled = enabled_transitions(net, &state);
        if enabled.is_empty() {
            logs.push(LogEntry {
                time: now,
                fired_transition: None,
                marking,
                touched_places: Vec::new(),
            });
            if let Some(next_release) = state.next_release_time() {
                let next_time = next_release;
                if next_time > now {
                    now = next_time;
                    if should_stop(net, &state, params, now, passes) {
                        break;
                    }
                    continue;
                }
            }
            break;
        }

        let fired = pick_transition(net, &enabled, &mut rng);
        let touched_places = fire_transition(net, &mut state, fired, now, &mut rng);
        passes = passes.saturating_add(1);

        logs.push(LogEntry {
            time: now,
            fired_transition: Some(fired),
            marking: state.total_marking(),
            touched_places,
        });

        if should_stop(net, &state, params, now, passes) {
            break;
        }
    }

    let place_stats = if collect_stats && !logs.is_empty() {
        let n = logs.len() as f64;
        Some(
            (0..places)
                .map(|p| PlaceStats {
                    min: if stats_min[p] == u32::MAX { 0 } else { stats_min[p] },
                    max: stats_max[p],
                    avg: stats_acc[p] / n,
                })
                .collect(),
        )
    } else {
        None
    };

    SimulationResult {
        cycle_time,
        logs,
        place_stats,
        fired_count: passes,
        final_marking: state.total_marking(),
    }
}

fn enabled_transitions(net: &PetriNet, state: &SimState) -> Vec<usize> {
    let mut enabled = Vec::new();
    let places = net.places.len();

    for t in 0..net.transitions.len() {
        let mut has_incident_arc = false;
        for p in 0..places {
            if net.tables.pre[p][t] > 0 || net.tables.post[p][t] > 0 || net.tables.inhibitor[p][t] > 0 {
                has_incident_arc = true;
                break;
            }
        }
        if !has_incident_arc {
            continue;
        }

        let mut ok = true;

        for p in 0..places {
            let need = net.tables.pre[p][t];
            if state.available[p] < need {
                ok = false;
                break;
            }

            let inh = net.tables.inhibitor[p][t];
            if inh > 0 {
                let marking_total = state.available[p] + state.pending_release[p].len() as u32;
                if marking_total >= inh {
                    ok = false;
                    break;
                }
            }
        }

        if !ok {
            continue;
        }

        for p in 0..places {
            if let Some(cap) = net.tables.mo[p] {
                let current_total = state.available[p] + state.pending_release[p].len() as u32;
                let after = current_total
                    .saturating_sub(net.tables.pre[p][t])
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

fn pick_transition(net: &PetriNet, enabled: &[usize], rng: &mut SmallRng) -> usize {
    let mut best_priority = *net.tables.mpr.get(enabled[0]).unwrap_or(&0);
    for &t in enabled.iter().skip(1) {
        let p = *net.tables.mpr.get(t).unwrap_or(&0);
        best_priority = best_priority.max(p);
    }

    let mut candidates: Vec<usize> = enabled
        .iter()
        .copied()
        .filter(|&t| *net.tables.mpr.get(t).unwrap_or(&0) == best_priority)
        .collect();
    candidates.sort_unstable();
    let idx = rng.gen_range(0..candidates.len());
    candidates[idx]
}

fn fire_transition(
    net: &PetriNet,
    state: &mut SimState,
    t: usize,
    now: f64,
    rng: &mut SmallRng,
) -> Vec<usize> {
    let mut touched_places = Vec::new();
    let mut push_touched = |p: usize| {
        if !touched_places.contains(&p) {
            touched_places.push(p);
        }
    };

    for p in 0..net.places.len() {
        let pre = net.tables.pre[p][t];
        if pre > 0 {
            push_touched(p);
        }
        state.available[p] = state.available[p].saturating_sub(pre);
    }

    for p in 0..net.places.len() {
        let post = net.tables.post[p][t];
        if post == 0 {
            continue;
        }

        push_touched(p);
        let delay = sample_place_delay(net, p, net.tables.mz[p].max(0.0), rng);
        for _ in 0..post {
            if delay > 0.0 {
                state.pending_release[p].push(now + delay);
            } else {
                state.available[p] = state.available[p].saturating_add(1);
            }
            state.through_place_counter[p] = state.through_place_counter[p].saturating_add(1);
        }
    }
    touched_places
}

fn sample_place_delay(net: &PetriNet, place_index: usize, base_delay: f64, rng: &mut SmallRng) -> f64 {
    let Some(place) = net.places.get(place_index) else {
        return base_delay.max(0.0);
    };
    let value = match place.stochastic {
        StochasticDistribution::None => base_delay,
        StochasticDistribution::Uniform { min, max } => {
            let lo = min.min(max);
            let hi = min.max(max);
            if (hi - lo).abs() < f64::EPSILON {
                lo
            } else {
                rng.gen_range(lo..=hi)
            }
        }
        StochasticDistribution::Normal { mean, std_dev } => {
            let sigma = std_dev.max(0.0);
            if sigma <= f64::EPSILON {
                mean
            } else {
                let u1 = (1.0 - rng.gen::<f64>()).clamp(1e-12, 1.0);
                let u2 = rng.gen::<f64>();
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                mean + sigma * z
            }
        }
        StochasticDistribution::Exponential { lambda } => {
            let l = lambda.max(1e-9);
            let u = (1.0 - rng.gen::<f64>()).clamp(1e-12, 1.0);
            -u.ln() / l
        }
        StochasticDistribution::Poisson { lambda } => {
            let l = lambda.max(0.0);
            if l <= f64::EPSILON {
                0.0
            } else {
                let limit = (-l).exp();
                let mut k = 0_u32;
                let mut p = 1.0_f64;
                loop {
                    k = k.saturating_add(1);
                    p *= rng.gen::<f64>();
                    if p <= limit {
                        break;
                    }
                }
                (k.saturating_sub(1)) as f64
            }
        }
        StochasticDistribution::CustomValue { value } => value,
    };
    if value.is_finite() { value.max(0.0) } else { 0.0 }
}

fn should_stop(net: &PetriNet, state: &SimState, params: &SimulationParams, now: f64, passes: u64) -> bool {
    if params.use_time_limit && now >= params.time_limit_sec.max(0.0) {
        return true;
    }
    if params.use_pass_limit && passes >= params.pass_limit {
        return true;
    }

    if let Some((pk, n)) = params.stop.through_place {
        if pk < net.places.len() && state.through_place_counter[pk] >= n {
            return true;
        }
    }
    if let Some(t) = params.stop.sim_time {
        if now >= t.max(0.0) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeRef, PetriNet};

    #[test]
    fn firing_rules_with_priority() {
        let mut net = PetriNet::new();
        net.set_counts(1, 2);
        net.tables.m0[0] = 2;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.tables.pre[0][1] = 1;
        net.tables.post[0][1] = 0;
        net.tables.mpr[0] = 1;
        net.tables.mpr[1] = 5;
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 1,
            ..SimulationParams::default()
        };
        let res = run_simulation(&net, &p, true, false);
        assert_eq!(res.logs[0].fired_transition, Some(1));
    }

    #[test]
    fn timed_tokens_become_available_after_delay() {
        let mut net = PetriNet::new();
        net.add_place([0.0, 0.0]);
        net.add_place([100.0, 0.0]);
        net.add_transition([50.0, 0.0]);
        net.tables.m0[0] = 1;
        net.tables.mz[1] = 1.0;
        let p1 = net.places[0].id;
        let p2 = net.places[1].id;
        let t1 = net.transitions[0].id;
        net.add_arc(NodeRef::Place(p1), NodeRef::Transition(t1), 1);
        net.add_arc(NodeRef::Transition(t1), NodeRef::Place(p2), 1);

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 1,
            dt: 0.5,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, true, false);
        assert_eq!(res.final_marking[1], 1);
        assert!(res.logs[0].marking[1] <= 1);
    }

    #[test]
    fn isolated_transition_is_ignored() {
        let mut net = PetriNet::new();
        net.set_counts(1, 2);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.tables.mpr[0] = 1;
        net.tables.mpr[1] = 100; // isolated but higher priority
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 1,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, true, false);
        assert_eq!(res.logs[0].fired_transition, Some(0));
    }

    #[test]
    fn simulation_waits_for_delayed_tokens_instead_of_stopping() {
        let mut net = PetriNet::new();
        net.set_counts(3, 2);
        net.tables.m0[0] = 1;
        net.tables.mz[1] = 1.0;
        net.tables.pre[0][0] = 1; // P1 -> T1
        net.tables.post[1][0] = 1; // T1 -> P2 (delayed)
        net.tables.pre[1][1] = 1; // P2 -> T2
        net.tables.post[2][1] = 1; // T2 -> P3
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 2,
            dt: 0.1,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, true, false);
        assert_eq!(res.fired_count, 2);
        assert_eq!(res.final_marking[2], 1);
    }
    #[test]
    fn zero_delay_transitions_do_not_advance_time() {
        let mut net = PetriNet::new();
        net.set_counts(1, 1);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 3,
            dt: 0.1,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, false, false);
        assert_eq!(res.fired_count, 3);
        assert!(res.logs.iter().all(|entry| (entry.time - 0.0).abs() < f64::EPSILON));
    }
}
