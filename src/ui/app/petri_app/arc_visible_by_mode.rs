use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_visible_by_mode(
        &self,
        color: NodeColor,
        per_arc_visible: bool,
    ) -> bool {
        if !per_arc_visible {
            return false;
        }
        match self.arc_display_mode {
            ArcDisplayMode::All => true,
            ArcDisplayMode::OnlyColor => color == self.arc_display_color,
            ArcDisplayMode::Hidden => false,
        }
    }
}
