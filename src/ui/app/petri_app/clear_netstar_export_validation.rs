use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn clear_netstar_export_validation(&mut self) {
        self.show_netstar_export_validation = false;
        self.pending_netstar_export_path = None;
        self.netstar_export_validation = None;
    }
}
