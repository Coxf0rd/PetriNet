use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Файлы PetriNet", &["gpn2", "pn", "gpn"])
            .pick_file()
        {
            match load_gpn(&path) {
                Ok(result) => {
                    let legacy_hints = if result.legacy_debug.is_some() {
                        let mut hints = Self::extract_legacy_export_hints(&path);
                        if let Some(h) = hints.as_mut() {
                            h.arc_topology_fingerprint =
                                Some(Self::arc_topology_fingerprint(&result.model));
                        }
                        hints
                    } else {
                        None
                    };
                    self.net = result.model;
                    self.net.normalize_arc_ids();
                    self.net
                        .set_counts(self.net.places.len(), self.net.transitions.len());
                    self.file_path = Some(path.clone());
                    self.undo_stack.clear();
                    self.sync_canvas_overlays_from_model();
                    self.load_legacy_sidecar_for_migration(&path);
                    self.legacy_export_hints = legacy_hints;
                    self.status_hint = None;
                    self.canvas.cursor_valid = false;
                    self.sim_result = None;
                    self.show_results = false;
                    self.show_debug = false;
                    self.show_proof = false;
                    self.show_place_stats_window = false;
                    self.debug_playing = false;
                    self.debug_animation_enabled = false;
                    self.debug_arc_animation = false;
                    self.debug_animation_events.clear();
                    self.debug_place_colors.clear();
                    self.invalidate_markov_model();
                    self.markov_arc_min_weight_percent = Self::MARKOV_ARC_MIN_PERCENT;
                    self.markov_model_enabled = false;
                    self.show_markov_window = false;
                    self.sim_run_serial = 0;
                    let filtered: Vec<String> = result
                        .warnings
                        .iter()
                        .filter(|w| {
                            !w.contains("best-effort")
                                && !w.contains("signature heuristic")
                                && !w.contains("восстановлены по сигнатурам")
                        })
                        .cloned()
                        .collect();
                    if filtered.is_empty() {
                        self.last_error = None;
                    } else {
                        self.last_error = Some(format!(
                            "Импорт с предупреждениями: {}",
                            filtered.join("; ")
                        ));
                    }
                }
                Err(e) => self.last_error = Some(e.to_string()),
            }
        }
    }
}
