use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_font_candidates() -> &'static [&'static str] {
        &["MS Sans Serif", "Arial", "Courier New"]
    }
}
