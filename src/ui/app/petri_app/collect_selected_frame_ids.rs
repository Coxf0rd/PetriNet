use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_frame_ids(&self) -> Vec<u64> {
        let mut frame_ids = self.canvas.selected_frames.clone();
        if let Some(id) = self.canvas.selected_frame {
            frame_ids.push(id);
        }
        frame_ids.sort_unstable();
        frame_ids.dedup();
        frame_ids
    }
}
