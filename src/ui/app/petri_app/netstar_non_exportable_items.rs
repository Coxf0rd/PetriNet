use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn netstar_non_exportable_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        if !self.text_blocks.is_empty() {
            items.push(format!(
                "{}: {}",
                self.tr("Текстовые блоки", "Text blocks"),
                self.text_blocks.len()
            ));
        }
        if !self.decorative_frames.is_empty() {
            items.push(format!(
                "{}: {}",
                self.tr("Декоративные рамки", "Decorative frames"),
                self.decorative_frames.len()
            ));
        }
        let has_arc_style_data = self
            .net
            .arcs
            .iter()
            .any(|arc| arc.color != NodeColor::Default || !arc.visible)
            || self
                .net
                .inhibitor_arcs
                .iter()
                .any(|arc| arc.color != NodeColor::Red || !arc.visible);
        if has_arc_style_data {
            items.push(
                self.tr("Цвет/скрытие дуг", "Arc color/visibility")
                    .to_string(),
            );
        }
        items
    }
}
