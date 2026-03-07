use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn node_color_text(color: NodeColor, is_ru: bool) -> &'static str {
        match (color, is_ru) {
            (NodeColor::Default, true) => "По умолчанию",
            (NodeColor::Blue, true) => "Синий",
            (NodeColor::Red, true) => "Красный",
            (NodeColor::Green, true) => "Зеленый",
            (NodeColor::Yellow, true) => "Желтый",
            (NodeColor::Default, false) => "Default",
            (NodeColor::Blue, false) => "Blue",
            (NodeColor::Red, false) => "Red",
            (NodeColor::Green, false) => "Green",
            (NodeColor::Yellow, false) => "Yellow",
        }
    }
}
