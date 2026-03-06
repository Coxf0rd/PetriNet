use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn save_file(&mut self) {
        if let Some(path) = self.file_path.clone() {
            let is_gpn2 = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("gpn2"))
                .unwrap_or(false);
            if !is_gpn2 {
                self.save_file_as();
                return;
            }
            self.sync_model_overlays_from_canvas();
            if let Err(e) = crate::io::gpn2::save_gpn2(&path, &self.net) {
                self.last_error = Some(e.to_string());
            } else {
                Self::cleanup_legacy_sidecar(&path);
            }
        } else {
            self.save_file_as();
        }
    }
}
