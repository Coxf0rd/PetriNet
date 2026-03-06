use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn color_to_egui(color: NodeColor, fallback: Color32) -> Color32 {
        match color {
            NodeColor::Default => fallback,
            NodeColor::Blue => Color32::from_rgb(25, 90, 220),
            NodeColor::Red => Color32::from_rgb(200, 40, 40),
            NodeColor::Green => Color32::from_rgb(40, 150, 60),
            NodeColor::Yellow => Color32::from_rgb(200, 160, 20),
        }
    }
}
