use super::*;
use crate::markov::BuildStopReason;

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
        let chain = build_markov_chain(&self.net, None);
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
        };
        eprintln!(
            "[markov] calculated: sim_run_serial={}, states={}, transitions(before_merge)={}, transitions(after_merge)={}, stop_reason={}, stationary_status={:?}, repeated_recalc_attempts={}",
            self.sim_run_serial,
            chain.state_count(),
            chain.transition_count_before_merge,
            chain.transition_count_after_merge,
            stop_reason,
            chain.stationary_status,
            self.markov_recompute_attempts
        );
        self.markov_limit_reached = chain.limit_reached;
        self.markov_model = Some(chain);
        self.markov_computed_for_sim_serial = Some(self.sim_run_serial);
        self.markov_model_pending_compute = false;
        self.update_markov_annotations();
        self.refresh_markov_place_arcs();
    }
}
