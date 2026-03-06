use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn debug_visible_log_indices(result: &SimulationResult) -> Vec<usize> {
        if result.logs.is_empty() {
            return Vec::new();
        }

        // Step 0 in debug must always point to the initial state.
        let mut indices = vec![0usize];
        let mut previous_marking = result.logs[0].marking.as_slice();
        for (idx, entry) in result.logs.iter().enumerate().skip(1) {
            let marking_changed = previous_marking != entry.marking.as_slice();
            if entry.fired_transition.is_some() || marking_changed {
                indices.push(idx);
            }
            previous_marking = entry.marking.as_slice();
        }
        indices
    }
}
