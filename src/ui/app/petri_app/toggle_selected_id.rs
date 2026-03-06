use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn toggle_selected_id(ids: &mut Vec<u64>, id: u64) -> bool {
        if let Some(idx) = ids.iter().position(|&value| value == id) {
            ids.remove(idx);
            false
        } else {
            ids.push(id);
            true
        }
    }
}
