use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_display_mode_text(
        mode: ArcDisplayMode,
        is_ru: bool,
    ) -> &'static str {
        match (mode, is_ru) {
            (ArcDisplayMode::All, true) => "Все",
            (ArcDisplayMode::OnlyColor, true) => "Только выбранный цвет",
            (ArcDisplayMode::Hidden, true) => "Скрыть все",
            (ArcDisplayMode::All, false) => "All",
            (ArcDisplayMode::OnlyColor, false) => "Only selected color",
            (ArcDisplayMode::Hidden, false) => "Hide all",
        }
    }
}
