use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn cleanup_legacy_sidecar(path: &std::path::Path) {
        let sidecar_path = Self::ui_sidecar_path(path);
        if sidecar_path.exists() {
            let _ = fs::remove_file(sidecar_path);
        }
    }
}
