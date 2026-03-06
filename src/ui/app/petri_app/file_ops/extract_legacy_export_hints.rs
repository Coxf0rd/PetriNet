use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn extract_legacy_export_hints(
        path: &std::path::Path,
    ) -> Option<LegacyExportHints> {
        const PLACE_RECORD_SIZE: usize = 231;
        const TRANSITION_RECORD_SIZE: usize = 105;
        let bytes = fs::read(path).ok()?;
        if bytes.starts_with(crate::model::GPN2_MAGIC.as_bytes()) || bytes.len() < 16 {
            return None;
        }
        let read_i32 = |off: usize| -> Option<i32> {
            if off + 4 > bytes.len() {
                return None;
            }
            Some(i32::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
            ]))
        };
        let p = read_i32(0)?.max(0) as usize;
        let t = read_i32(4)?.max(0) as usize;
        let arcs_off = 16usize
            .saturating_add(p.saturating_mul(PLACE_RECORD_SIZE))
            .saturating_add(t.saturating_mul(TRANSITION_RECORD_SIZE));
        if arcs_off + 6 > bytes.len() {
            return None;
        }
        let footer_bytes = None;
        let arc_header_extra = Some(u16::from_le_bytes([
            bytes[arcs_off + 4],
            bytes[arcs_off + 5],
        ]));
        Some(LegacyExportHints {
            places_count: Some(p),
            transitions_count: Some(t),
            arc_topology_fingerprint: None,
            arc_header_extra,
            footer_bytes,
            raw_arc_and_tail: Some(bytes[arcs_off..].to_vec()),
        })
    }
}
