use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn ensure_unique_place_name(
        &self,
        desired: &str,
        exclude_id: u64,
    ) -> String {
        let base = desired.trim();
        if base.is_empty() {
            return String::new();
        }
        let mut candidate = base.to_string();
        let mut n = 2u32;
        while self
            .net
            .places
            .iter()
            .any(|p| p.id != exclude_id && p.name.trim() == candidate.as_str())
        {
            candidate = format!("{base} ({n})");
            n = n.saturating_add(1);
        }
        candidate
    }
}
