use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn write_copy_buffer_to_system_clipboard(&mut self, buf: &CopyBuffer) {
        let payload = ClipboardPayload {
            version: 1,
            buffer: buf.clone(),
        };
        let Ok(json) = serde_json::to_string(&payload) else {
            return;
        };
        let text = format!("{}{}", Self::CLIPBOARD_PREFIX, json);
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }
}
