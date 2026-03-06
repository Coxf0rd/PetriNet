use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_arc_ids(&self) -> Vec<u64> {
        let mut arc_ids = self.canvas.selected_arcs.clone();
        if let Some(id) = self.canvas.selected_arc {
            arc_ids.push(id);
        }
        arc_ids.sort_unstable();
        arc_ids.dedup();
        arc_ids
    }
}
