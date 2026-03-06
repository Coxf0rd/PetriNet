use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn start_netstar_export_validation(&mut self, path: PathBuf) {
        self.sync_model_overlays_from_canvas();
        self.pending_netstar_export_path = Some(path);
        self.netstar_export_validation = Some(self.validate_netstar_export());
        self.show_netstar_export_validation = true;
    }
}
