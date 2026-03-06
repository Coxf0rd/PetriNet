use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_text_ids(&self) -> Vec<u64> {
        let mut text_ids = self.canvas.selected_texts.clone();
        if let Some(id) = self.canvas.selected_text {
            text_ids.push(id);
        }
        text_ids.sort_unstable();
        text_ids.dedup();
        text_ids
    }
}
