use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn load_legacy_sidecar_for_migration(&mut self, path: &std::path::Path) {
        if !self.text_blocks.is_empty() || !self.decorative_frames.is_empty() {
            return;
        }

        let sidecar_path = Self::ui_sidecar_path(path);
        let Ok(raw) = fs::read_to_string(&sidecar_path) else {
            return;
        };
        let Ok(sidecar) = serde_json::from_str::<LegacyUiSidecar>(&raw) else {
            return;
        };

        self.text_blocks = sidecar.text_blocks;
        self.decorative_frames = sidecar
            .decorative_frames
            .into_iter()
            .map(|frame| CanvasFrame {
                id: frame.id,
                pos: frame.pos,
                width: frame.side.max(Self::FRAME_MIN_SIDE),
                height: frame.side.max(Self::FRAME_MIN_SIDE),
            })
            .collect();
        self.next_text_id = sidecar.next_text_id.max(
            self.text_blocks
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
        self.next_frame_id = sidecar.next_frame_id.max(
            self.decorative_frames
                .iter()
                .map(|f| f.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );

        // Persist migrated overlays to GPN2 on next save.
        self.sync_model_overlays_from_canvas();
    }
}
