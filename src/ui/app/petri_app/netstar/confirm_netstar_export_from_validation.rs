use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn confirm_netstar_export_from_validation(&mut self) {
        let Some(path) = self.pending_netstar_export_path.clone() else {
            self.clear_netstar_export_validation();
            return;
        };

        self.sync_model_overlays_from_canvas();
        if let Err(e) = save_gpn_with_hints(&path, &self.net, self.legacy_export_hints.as_ref()) {
            self.last_error = Some(e.to_string());
        } else {
            self.last_error = None;
            self.status_hint = Some(
                self.tr("Экспорт в NetStar завершен", "NetStar export completed")
                    .to_string(),
            );
        }
        self.clear_netstar_export_validation();
    }
}
