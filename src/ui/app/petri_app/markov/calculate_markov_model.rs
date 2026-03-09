use super::*;
use crate::markov::{build_markov_chain_approximate, BuildStopReason, MarkovComputationMode};

const EXACT_MARKOV_MAX_STATES: usize = 250_000;
const APPROX_MARKOV_MAX_STATES: usize = 250_000;

impl PetriApp {
    pub(in crate::ui::app) fn invalidate_markov_model(&mut self) {
        self.markov_model = None;
        self.markov_model_pending_compute = false;
        self.markov_computed_for_sim_serial = None;
        self.markov_recompute_attempts = 0;
        self.markov_limit_reached = false;
        self.markov_annotations.clear();
        self.markov_place_arcs.clear();
        self.selected_markov_arc = None;
        self.markov_stationary_row_offsets.clear();
        self.markov_state_graph_row_offsets.clear();
    }

    fn rebuild_markov_virtual_row_offsets(&mut self) {
        self.markov_stationary_row_offsets.clear();
        self.markov_state_graph_row_offsets.clear();

        let Some(chain) = self.markov_model.as_ref() else {
            return;
        };

        self.markov_stationary_row_offsets = Self::markov_build_stationary_row_offsets(chain);
        self.markov_state_graph_row_offsets = Self::markov_build_state_graph_row_offsets(chain);
    }

    pub(in crate::ui::app) fn calculate_markov_model(&mut self) {
        if self.markov_model.is_some()
            && self.markov_computed_for_sim_serial == Some(self.sim_run_serial)
        {
            self.markov_recompute_attempts = self.markov_recompute_attempts.saturating_add(1);
            eprintln!(
                "[markov] repeated recalculation blocked: sim_run_serial={}, attempts={}",
                self.sim_run_serial, self.markov_recompute_attempts
            );
            return;
        }

        self.net.sanitize_values();
        let mut chain = build_markov_chain(&self.net, Some(EXACT_MARKOV_MAX_STATES));
        if chain.limit_reached {
            if let Some(sim_result) = self.sim_result.as_deref() {
                if let Some(approx) =
                    build_markov_chain_approximate(sim_result, APPROX_MARKOV_MAX_STATES)
                {
                    eprintln!(
                        "[markov] exact model reached state limit ({}), switched to approximate mode from simulation logs",
                        EXACT_MARKOV_MAX_STATES
                    );
                    chain = approx;
                }
            }
        }
        let stop_reason = match &chain.build_stop_reason {
            BuildStopReason::ExhaustedStateSpace { explored_states } => {
                format!("state-space exhausted, explored_states={}", explored_states)
            }
            BuildStopReason::StateLimitReached {
                explored_states,
                limit,
            } => format!(
                "state limit reached, explored_states={}, limit={}",
                explored_states, limit
            ),
            BuildStopReason::ApproximationFromSimulation {
                sampled_states,
                sampled_steps,
            } => format!(
                "approximation from simulation logs, sampled_states={}, sampled_steps={}",
                sampled_states, sampled_steps
            ),
        };
        eprintln!(
            "[markov] calculated: sim_run_serial={}, mode={:?}, states={}, transitions(before_merge)={}, transitions(after_merge)={}, stop_reason={}, stationary_status={:?}, repeated_recalc_attempts={}",
            self.sim_run_serial,
            chain.computation_mode,
            chain.state_count(),
            chain.transition_count_before_merge,
            chain.transition_count_after_merge,
            stop_reason,
            chain.stationary_status,
            self.markov_recompute_attempts
        );
        self.markov_limit_reached =
            chain.limit_reached && chain.computation_mode == MarkovComputationMode::Exact;
        self.markov_model = Some(chain);
        self.rebuild_markov_virtual_row_offsets();
        self.markov_computed_for_sim_serial = Some(self.sim_run_serial);
        self.markov_model_pending_compute = false;
        self.update_markov_annotations();
        self.refresh_markov_place_arcs();
    }
}
