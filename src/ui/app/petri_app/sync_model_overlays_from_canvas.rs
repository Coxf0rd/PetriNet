use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sync_model_overlays_from_canvas(&mut self) {
        self.net.ui.text_blocks = self
            .text_blocks
            .iter()
            .map(|item| UiTextBlock {
                id: item.id,
                pos: item.pos,
                text: item.text.clone(),
                font_name: item.font_name.clone(),
                font_size: item.font_size,
                color: item.color,
            })
            .collect();
        self.net.ui.decorative_frames = self
            .decorative_frames
            .iter()
            .map(|frame| UiDecorativeFrame {
                id: frame.id,
                pos: frame.pos,
                width: frame.width.max(Self::FRAME_MIN_SIDE),
                height: frame.height.max(Self::FRAME_MIN_SIDE),
            })
            .collect();
        self.net.ui.next_text_id = self.next_text_id;
        self.net.ui.next_frame_id = self.next_frame_id;
    }
}
