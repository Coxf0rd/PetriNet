use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Файлы PetriNet (gpn2)", &["gpn2"])
            .set_file_name("модель.gpn2")
            .save_file()
        {
            self.file_path = Some(path.clone());
            self.sync_model_overlays_from_canvas();
            if let Err(e) = crate::io::gpn2::save_gpn2(&path, &self.net) {
                self.last_error = Some(e.to_string());
            } else {
                Self::cleanup_legacy_sidecar(&path);
            }
        }
    }
}
