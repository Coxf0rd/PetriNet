use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sync_canvas_overlays_from_model(&mut self) {
        self.text_blocks = self
            .net
            .ui
            .text_blocks
            .iter()
            .map(|item| CanvasTextBlock {
                id: item.id,
                pos: item.pos,
                text: item.text.clone(),
                font_name: item.font_name.clone(),
                font_size: item.font_size,
                color: item.color,
            })
            .collect();
        self.decorative_frames = self
            .net
            .ui
            .decorative_frames
            .iter()
            .map(|frame| CanvasFrame {
                id: frame.id,
                pos: frame.pos,
                width: frame.width.max(Self::FRAME_MIN_SIDE),
                height: frame.height.max(Self::FRAME_MIN_SIDE),
            })
            .collect();

        self.next_text_id = self.net.ui.next_text_id.max(
            self.text_blocks
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
        self.next_frame_id = self.net.ui.next_frame_id.max(
            self.decorative_frames
                .iter()
                .map(|f| f.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
    }
}
