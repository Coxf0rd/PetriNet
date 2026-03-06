use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn ui_sidecar_path(path: &std::path::Path) -> PathBuf {
        let mut os = path.as_os_str().to_os_string();
        os.push(".petriui.json");
        PathBuf::from(os)
    }
}
