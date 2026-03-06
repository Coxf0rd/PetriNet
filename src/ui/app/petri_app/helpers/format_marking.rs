use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn format_marking(marking: &[u32]) -> String {
        marking
            .iter()
            .enumerate()
            .map(|(idx, value)| format!("P{}={}", idx + 1, value))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
