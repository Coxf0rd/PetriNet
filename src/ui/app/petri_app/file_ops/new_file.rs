use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn new_file(&mut self) {
        self.net = PetriNet::new();
        self.net.set_counts(0, 0);
        self.file_path = None;
        self.text_blocks.clear();
        self.next_text_id = 1;
        self.decorative_frames.clear();
        self.next_frame_id = 1;
        self.undo_stack.clear();
        self.legacy_export_hints = None;
        self.status_hint = None;
        self.show_netstar_export_validation = false;
        self.pending_netstar_export_path = None;
        self.netstar_export_validation = None;
        self.sim_result = None;
        self.show_results = false;
        self.show_debug = false;
        self.show_proof = false;
        self.show_place_stats_window = false;
        self.debug_animation_enabled = false;
        self.debug_arc_animation = false;
        self.debug_animation_events.clear();
        self.debug_place_colors.clear();
        self.markov_model = None;
        self.markov_model_pending_compute = false;
        self.sim_run_serial = 0;
        self.markov_computed_for_sim_serial = None;
        self.markov_recompute_attempts = 0;
        self.markov_limit_reached = false;
        self.markov_annotations.clear();
        self.markov_place_arcs.clear();
        self.markov_arc_view_mode = MarkovArcViewMode::AggregatedWeighted;
        self.selected_markov_arc = None;
        self.show_markov_window = false;
        self.markov_model_enabled = false;
        self.canvas.cursor_valid = false;
    }
}
