use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn markov_placement_text(
        placement: MarkovPlacement,
        is_ru: bool,
    ) -> &'static str {
        match (placement, is_ru) {
            (MarkovPlacement::Bottom, true) => "Внизу",
            (MarkovPlacement::Top, true) => "Вверху",
            (MarkovPlacement::Bottom, false) => "Bottom",
            (MarkovPlacement::Top, false) => "Top",
        }
    }
}
