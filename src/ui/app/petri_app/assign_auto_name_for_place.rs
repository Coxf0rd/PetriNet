use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn assign_auto_name_for_place(&mut self, place_id: u64) {
        let mut ids: Vec<u64> = self.net.places.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        let rank = ids
            .iter()
            .position(|&id| id == place_id)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| self.net.places.len().max(1));
        let new_name = format!("P{rank}");
        if let Some(index) = self.place_idx_by_id(place_id) {
            self.net.places[index].name = new_name;
        }
    }
}
