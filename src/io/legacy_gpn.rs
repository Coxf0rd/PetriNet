use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::model::{NodeColor, NodeRef, PetriNetModel, VisualSize};

const PLACE_RECORD_SIZE: usize = 231;
const PLACE_DELAY_OFFSET: usize = 77;
const PLACE_NAME_OFFSET: usize = 26;
const TRANSITION_RECORD_SIZE: usize = 105;
const TRANSITION_NAME_OFFSET: usize = 54;
const ARC_SECTION_HEADER_SIZE: usize = 6;
const ARC_RECORD_SIZE: usize = 46;

#[derive(Debug, Clone)]
pub struct LegacyDebugInfo {
    pub file_size: usize,
    pub candidate_counts: Vec<(usize, u32, u32)>,
    pub discovered_sections: Vec<String>,
    pub ascii_strings: Vec<(usize, String)>,
    pub utf16le_strings: Vec<(usize, String)>,
    pub candidate_float64_pairs: Vec<(usize, f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct LegacyImportResult {
    pub model: PetriNetModel,
    pub warnings: Vec<String>,
    pub debug: LegacyDebugInfo,
}

#[derive(Debug)]
pub enum LegacyImportError {
    Io(std::io::Error),
    Invalid(String),
}

impl fmt::Display for LegacyImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "РћС€РёР±РєР° РІРІРѕРґР°-РІС‹РІРѕРґР°: {e}"),
            Self::Invalid(msg) => write!(f, "РќРµРєРѕСЂСЂРµРєС‚РЅС‹Р№ legacy GPN: {msg}"),
        }
    }
}

impl std::error::Error for LegacyImportError {}

impl From<std::io::Error> for LegacyImportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, Copy)]
struct LegacyLayout {
    place_header_size: usize,
    places_offset: usize,
    transitions_offset: usize,
    arcs_offset: usize,
}

pub fn detect_legacy_gpn(bytes: &[u8]) -> bool {
    !bytes.starts_with(crate::model::GPN2_MAGIC.as_bytes())
}

pub fn import_legacy_gpn(path: &Path) -> Result<LegacyImportResult, LegacyImportError> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() {
        return Err(LegacyImportError::Invalid("РџСѓСЃС‚РѕР№ С„Р°Р№Р»".to_string()));
    }

    let candidate_counts = detect_counts(&bytes);
    let ascii_strings = extract_ascii_strings(&bytes, 4);
    let utf16le_strings = extract_utf16le_strings(&bytes, 4);
    let candidate_float64_pairs = extract_float64_pairs(&bytes, 256);

    let (places_count, transitions_count, mut warnings) =
        if let Some((p, t)) = header_counts_from_prefix(&bytes) {
            (p, t, Vec::new())
        } else if let Some((_, p, t)) = candidate_counts.first().copied() {
            (
                p.clamp(1, 2000) as usize,
                t.clamp(1, 2000) as usize,
                vec!["РСЃРїРѕР»СЊР·РѕРІР°РЅС‹ СЌРІСЂРёСЃС‚РёС‡РµСЃРєРёРµ counts".to_string()],
            )
        } else {
            (
                1,
                1,
                vec![
                    "РќРµ СѓРґР°Р»РѕСЃСЊ РЅР°РґРµР¶РЅРѕ РёР·РІР»РµС‡СЊ С‡РёСЃР»Р° РјРµСЃС‚/РїРµСЂРµС…РѕРґРѕРІ, РїСЂРёРјРµРЅРµРЅС‹ Р·РЅР°С‡РµРЅРёСЏ 1/1"
                        .to_string(),
                ],
            )
        };

    let mut model = PetriNetModel::new();
    model.set_counts(places_count, transitions_count);

    let mut used_fallback = false;
    let layout = detect_legacy_layout(&bytes, places_count, transitions_count);
    if let Some(layout) = layout {
        let parsed_place_nodes = parse_place_nodes_from_layout(&bytes, places_count, layout);
        let parsed_transition_nodes =
            parse_transition_nodes_from_layout(&bytes, transitions_count, layout);
        let mut arcs_applied = false;

        for (idx, place) in model.places.iter_mut().enumerate() {
            if let Some(first) = parsed_place_nodes.get(idx).cloned() {
                if !first.valid {
                    continue;
                }
                place.pos = [first.x, first.y];
                place.size = VisualSize::Small;
                if !first.name.is_empty() {
                    place.name = first.name.clone();
                    if place.note.trim().is_empty() {
                        place.note = first.name;
                    }
                }
                model.tables.m0[idx] = first.markers.max(0) as u32;
                model.tables.mo[idx] = if first.capacity > 0 {
                    Some(first.capacity as u32)
                } else {
                    None
                };
                model.tables.mz[idx] = first.delay_sec.max(0.0);
                place.color = map_legacy_color(first.color_raw);
            }
        }

        for (idx, tr) in model.transitions.iter_mut().enumerate() {
            if let Some(first) = parsed_transition_nodes.get(idx).cloned() {
                if !first.valid {
                    continue;
                }
                tr.size = VisualSize::Medium;
                let (w, h) = legacy_transition_dims(tr.size);
                tr.pos = [first.x - w * 0.5, first.y - h * 0.5];
                tr.angle_deg = first.angle_deg;
                if !first.name.is_empty() {
                    if tr.note.trim().is_empty() {
                        tr.note = first.name;
                    }
                }
                model.tables.mpr[idx] = first.priority;
                tr.color = map_legacy_color(first.color_raw);
            }
        }

        if let Some(arcs) = parse_arcs_from_section(&bytes, places_count, transitions_count, layout) {
            apply_legacy_arcs(&mut model, &arcs);
            arcs_applied = true;
        } else if let Some(arcs) = parse_arcs_by_signature(
            &bytes,
            places_count,
            transitions_count,
            &model.places,
            &model.transitions,
        ) {
            used_fallback = true;
            warnings.push("Дуги восстановлены по сигнатурам".to_string());
            apply_legacy_arcs(&mut model, &arcs);
            arcs_applied = true;
        } else {
            used_fallback = true;
            warnings.push("Не удалось извлечь дуги".to_string());
        }
        if arcs_applied {
            prune_legacy_ghost_nodes(&mut model);
            apply_legacy_read_arc_heuristics(&mut model);
        }
    } else {
        used_fallback = true;
        warnings.push("РќРµ СѓРґР°Р»РѕСЃСЊ РѕРїСЂРµРґРµР»РёС‚СЊ layout legacy СЃРµРєС†РёР№".to_string());
    }

    if used_fallback {
        warnings.push("РРјРїРѕСЂС‚ legacy GPN РІС‹РїРѕР»РЅРµРЅ РІ СЂРµР¶РёРјРµ best-effort".to_string());
    }

    let mut discovered_sections = detect_section_boundaries(&bytes);
    if let Some(layout) = layout {
        discovered_sections.push(format!(
            "layout: header={}, places@0x{:X}, transitions@0x{:X}, arcs@0x{:X}",
            layout.place_header_size,
            layout.places_offset,
            layout.transitions_offset,
            layout.arcs_offset
        ));
    }

    let debug = LegacyDebugInfo {
        file_size: bytes.len(),
        candidate_counts,
        discovered_sections,
        ascii_strings,
        utf16le_strings,
        candidate_float64_pairs,
    };

    Ok(LegacyImportResult {
        model,
        warnings,
        debug,
    })
}

pub fn export_legacy_gpn(path: &Path, model: &PetriNetModel) -> std::io::Result<()> {
    let mut normalized = model.clone();
    normalized.rebuild_matrices_from_arcs();

    let places_count = normalized.places.len();
    let transitions_count = normalized.transitions.len();
    let place_legacy_idx: HashMap<u64, usize> = normalized
        .places
        .iter()
        .enumerate()
        .map(|(idx, place)| (place.id, idx + 1))
        .collect();
    let transition_legacy_idx: HashMap<u64, usize> = normalized
        .transitions
        .iter()
        .enumerate()
        .map(|(idx, transition)| (transition.id, idx + 1))
        .collect();

    let mut bytes = Vec::new();
    push_i32(&mut bytes, places_count as i32);
    push_i32(&mut bytes, transitions_count as i32);
    push_i32(&mut bytes, 0x20);
    push_i32(&mut bytes, 0);

    let looks_like_auto_place_name = |name: &str| -> bool {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !['P', 'p', 'Р', 'р'].contains(&first) {
            return false;
        }
        let rest: String = chars.collect();
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    };

    for idx in 0..places_count {
        let mut record = [0u8; PLACE_RECORD_SIZE];
        let place = &normalized.places[idx];
        write_i32(&mut record, 0, round_i32(place.pos[0]));
        write_i32(&mut record, 4, round_i32(place.pos[1]));
        write_i32(&mut record, 8, 10);
        write_i32(
            &mut record,
            12,
            normalized.tables.m0.get(idx).copied().unwrap_or(0) as i32,
        );
        write_i32(
            &mut record,
            16,
            normalized
                .tables
                .mo
                .get(idx)
                .and_then(|value| *value)
                .unwrap_or(0) as i32,
        );
        write_i32(&mut record, 20, map_color_to_legacy(place.color));
        write_f64(
            &mut record,
            PLACE_DELAY_OFFSET,
            normalized
                .tables
                .mz
                .get(idx)
                .copied()
                .unwrap_or(0.0)
                .max(0.0),
        );

        // NetStar displays the place label from this legacy field.
        // Prefer the explicit name; if it's an auto-name (P1, P2, ...) and note is filled,
        // export note instead so "Текст/Описание" is visible in NetStar.
        let place_label = if looks_like_auto_place_name(&place.name) && !place.note.trim().is_empty() {
            place.note.as_str()
        } else {
            place.name.as_str()
        };
        write_legacy_name(&mut record, PLACE_NAME_OFFSET, place_label);
        bytes.extend_from_slice(&record);
    }

    for idx in 0..transitions_count {
        let mut record = [0u8; TRANSITION_RECORD_SIZE];
        let transition = &normalized.transitions[idx];
        let (w, h) = legacy_transition_dims(transition.size);
        write_i32(&mut record, 0, round_i32(transition.pos[0] + w * 0.5));
        write_i32(&mut record, 4, round_i32(transition.pos[1] + h * 0.5));
        write_i32(
            &mut record,
            8,
            normalized.tables.mpr.get(idx).copied().unwrap_or(1).max(0),
        );
        write_i32(&mut record, 12, transition.angle_deg.clamp(-360, 360));
        write_i32(&mut record, 16, -131072);
        write_i32(&mut record, 20, -589825);
        write_i32(&mut record, 24, 196607);
        write_i32(&mut record, 28, -655360);
        write_i32(&mut record, 32, 196607);
        write_i32(&mut record, 36, 655360);
        write_i32(&mut record, 40, -131072);
        write_i32(&mut record, 44, 720895);
        write_i32(&mut record, 52, map_color_to_legacy(transition.color));

        // NetStar displays the transition label from this legacy field.
        let tr_label = if transition.note.trim().is_empty() {
            transition.name.as_str()
        } else {
            transition.note.as_str()
        };
        write_legacy_name(&mut record, TRANSITION_NAME_OFFSET, tr_label);
        bytes.extend_from_slice(&record);
    }

    let mut arc_records = Vec::<(bool, i32, usize, usize, NodeRef, NodeRef)>::new();

    for arc in &normalized.arcs {
        match (arc.from, arc.to) {
            (NodeRef::Place(place_id), NodeRef::Transition(transition_id)) => {
                let Some(&place_idx) = place_legacy_idx.get(&place_id) else {
                    continue;
                };
                let Some(&transition_idx) = transition_legacy_idx.get(&transition_id) else {
                    continue;
                };
                for _ in 0..arc.weight.max(1) {
                    arc_records.push((
                        false,
                        -1,
                        place_idx,
                        transition_idx,
                        NodeRef::Place(place_id),
                        NodeRef::Transition(transition_id),
                    ));
                }
            }
            (NodeRef::Transition(transition_id), NodeRef::Place(place_id)) => {
                let Some(&place_idx) = place_legacy_idx.get(&place_id) else {
                    continue;
                };
                let Some(&transition_idx) = transition_legacy_idx.get(&transition_id) else {
                    continue;
                };
                for _ in 0..arc.weight.max(1) {
                    arc_records.push((
                        false,
                        1,
                        transition_idx,
                        place_idx,
                        NodeRef::Transition(transition_id),
                        NodeRef::Place(place_id),
                    ));
                }
            }
            _ => {}
        }
    }

    for inhibitor in &normalized.inhibitor_arcs {
        let Some(&place_idx) = place_legacy_idx.get(&inhibitor.place_id) else {
            continue;
        };
        let Some(&transition_idx) = transition_legacy_idx.get(&inhibitor.transition_id) else {
            continue;
        };
        for _ in 0..inhibitor.threshold.max(1) {
            arc_records.push((
                true,
                -1,
                place_idx,
                transition_idx,
                NodeRef::Place(inhibitor.place_id),
                NodeRef::Transition(inhibitor.transition_id),
            ));
        }
    }
    arc_records.sort_by_key(|(inhibitor, direction, a, b, _, _)| (*inhibitor, *direction, *a, *b));

    let mut encoded_arcs = Vec::new();
    for (inhibitor, direction, source_idx, target_idx, from_node, to_node) in arc_records {
        let points = legacy_arc_polyline_points(&normalized, from_node, to_node)
            .unwrap_or(([0.0, 0.0], [0.0, 0.0], [0.0, 0.0]));
        encoded_arcs.push((inhibitor, direction, source_idx, target_idx, points));
    }

    let arc_max_index = encoded_arcs
        .len()
        .checked_sub(1)
        .map(|value| value as i32)
        .unwrap_or(-1);
    push_i32(&mut bytes, arc_max_index);
    bytes.extend_from_slice(&[0, 0]);
    for (inhibitor, direction, source_idx, target_idx, (p1, p2, p3)) in encoded_arcs {
        let mut record = [0u8; ARC_RECORD_SIZE];
        let p1x = clamp_u16(p1[0]);
        let p1y = clamp_u16(p1[1]);
        let p2x = clamp_u16(p2[0]);
        let p2y = clamp_u16(p2[1]);
        let p3x = clamp_u16(p3[0]);
        let p3y = clamp_u16(p3[1]);
        write_u16(&mut record, 2, p1y);
        write_u16(&mut record, 6, p2y);
        write_u16(&mut record, 10, p1x);
        write_u16(&mut record, 14, p3y);
        write_i32(&mut record, 24, if inhibitor { 0 } else { 1 });
        write_i32(&mut record, 28, direction);
        write_i32(&mut record, 32, source_idx as i32);
        write_i32(&mut record, 36, target_idx as i32);
        write_i32(&mut record, 40, p3x as i32);
        write_u16(&mut record, 44, p2x);
        bytes.extend_from_slice(&record);
    }
    bytes.extend_from_slice(&legacy_footer_template());

    fs::write(path, bytes)
}

#[derive(Debug, Clone)]
struct LegacyTransitionNode {
    valid: bool,
    x: f32,
    y: f32,
    priority: i32,
    angle_deg: i32,
    color_raw: i32,
    name: String,
}

fn header_counts_from_prefix(bytes: &[u8]) -> Option<(usize, usize)> {
    let places = read_i32(bytes, 0)?;
    let transitions = read_i32(bytes, 4)?;
    if !(1..=10_000).contains(&places) || !(1..=10_000).contains(&transitions) {
        return None;
    }
    Some((places as usize, transitions as usize))
}

fn detect_legacy_layout(
    bytes: &[u8],
    places_count: usize,
    transitions_count: usize,
) -> Option<LegacyLayout> {
    if places_count == 0 || transitions_count == 0 {
        return None;
    }

    let mut best: Option<(LegacyLayout, i32)> = None;
    for place_header_size in [16usize, 247usize] {
        let places_offset = place_header_size;
        let transitions_offset =
            places_offset.saturating_add(places_count.saturating_mul(PLACE_RECORD_SIZE));
        let arcs_offset = transitions_offset
            .saturating_add(transitions_count.saturating_mul(TRANSITION_RECORD_SIZE));
        if arcs_offset + 4 > bytes.len() {
            continue;
        }

        let mut score = 0i32;
        for idx in 0..places_count {
            let off = places_offset + idx * PLACE_RECORD_SIZE;
            let Some(x) = read_i32(bytes, off) else {
                break;
            };
            let Some(y) = read_i32(bytes, off + 4) else {
                break;
            };
            if (0..=50_000).contains(&x) && (0..=50_000).contains(&y) {
                score += 1;
            }
        }

        for idx in 0..transitions_count {
            let off = transitions_offset + idx * TRANSITION_RECORD_SIZE;
            let Some(x) = read_i32(bytes, off) else {
                break;
            };
            let Some(y) = read_i32(bytes, off + 4) else {
                break;
            };
            if (-50_000..=50_000).contains(&x) && (-50_000..=50_000).contains(&y) {
                score += 1;
            }
        }

        let layout = LegacyLayout {
            place_header_size,
            places_offset,
            transitions_offset,
            arcs_offset,
        };
        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((layout, score)),
        }
    }
    best.map(|(layout, _)| layout)
}

fn parse_transition_nodes_from_layout(
    bytes: &[u8],
    needed: usize,
    layout: LegacyLayout,
) -> Vec<LegacyTransitionNode> {
    let mut result = Vec::new();
    for idx in 0..needed {
        let off = layout.transitions_offset + idx * TRANSITION_RECORD_SIZE;
        let Some(x) = read_i32(bytes, off) else {
            break;
        };
        let Some(y) = read_i32(bytes, off + 4) else {
            break;
        };
        let priority = read_i32(bytes, off + 8).unwrap_or(1);
        let angle_deg = read_i32(bytes, off + 12).unwrap_or(90);
        let color_raw = read_i32(bytes, off + 52).unwrap_or(0);
        let name = read_legacy_name(bytes, off, TRANSITION_RECORD_SIZE, TRANSITION_NAME_OFFSET);
        let valid = (-50_000..=50_000).contains(&x) && (-50_000..=50_000).contains(&y);
        result.push(LegacyTransitionNode {
            valid,
            x: x as f32,
            y: y as f32,
            priority: priority.clamp(0, 1_000_000),
            angle_deg: angle_deg.clamp(-360, 360),
            color_raw,
            name,
        });
    }
    result
}

#[derive(Debug, Clone)]
struct LegacyPlaceNode {
    valid: bool,
    x: f32,
    y: f32,
    markers: i32,
    delay_sec: f64,
    capacity: i32,
    color_raw: i32,
    name: String,
}

fn detect_place_capacity_offset(bytes: &[u8], needed: usize, layout: LegacyLayout) -> usize {
    // Legacy variants exist: in some files the "capacity/Mo" field is not at +16.
    // We pick the most plausible offset by sampling first records and scoring candidates.
    const CANDIDATES: [usize; 3] = [16, 24, 28];
    let sample_n = needed.min(64);

    let mut best_off = 16usize;
    let mut best_score: i64 = i64::MIN;

    for cap_off in CANDIDATES {
        let mut ok = 0i64;
        let mut nonzero = 0i64;
        let mut invalid = 0i64;

        for idx in 0..sample_n {
            let off = layout.places_offset + idx * PLACE_RECORD_SIZE + cap_off;
            let Some(v) = read_i32(bytes, off) else {
                invalid += 1;
                continue;
            };
            if v < 0 || v > 1_000_000 {
                invalid += 1;
                continue;
            }
            ok += 1;
            if v != 0 {
                nonzero += 1;
            }
        }

        // Favor offsets that decode "reasonable" non-negative integers and aren't mostly invalid.
        let score = ok * 2 + nonzero * 3 - invalid * 5;
        if score > best_score || (score == best_score && cap_off == 16) {
            best_score = score;
            best_off = cap_off;
        }
    }

    best_off
}

fn parse_place_nodes_from_layout(
    bytes: &[u8],
    needed: usize,
    layout: LegacyLayout,
) -> Vec<LegacyPlaceNode> {
    #[derive(Debug, Clone)]
    struct RawPlaceNode {
        valid: bool,
        x: f32,
        y: f32,
        marker8: i32,
        marker12: i32,
        delay_sec: f64,
        capacity: i32,
        color_raw: i32,
        name: String,
    }

    let mut raw = Vec::<RawPlaceNode>::new();
    let capacity_off = detect_place_capacity_offset(bytes, needed, layout);
    for idx in 0..needed {
        let off = layout.places_offset + idx * PLACE_RECORD_SIZE;
        let Some(x) = read_i32(bytes, off) else {
            break;
        };
        let Some(y) = read_i32(bytes, off + 4) else {
            break;
        };
        let valid = (0..=20_000).contains(&x) && (0..=20_000).contains(&y);
        let marker8 = read_i32(bytes, off + 8).unwrap_or(0);
        let marker12 = read_i32(bytes, off + 12).unwrap_or(0);
        let delay_raw = read_f64(bytes, off + PLACE_DELAY_OFFSET);
        let delay_fallback = marker12 as f64;
        let capacity = read_i32(bytes, off + capacity_off).unwrap_or(0);
        let color_raw = read_i32(bytes, off + 20).unwrap_or(0);
        let name = read_legacy_name(bytes, off, PLACE_RECORD_SIZE, PLACE_NAME_OFFSET);
        raw.push(RawPlaceNode {
            valid,
            x: x as f32,
            y: y as f32,
            marker8,
            marker12,
            delay_sec: delay_raw
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(delay_fallback.max(0.0)),
            capacity,
            color_raw,
            name,
        });
    }

    let marker_pairs: Vec<(i32, i32)> = raw
        .iter()
        .filter(|node| node.valid)
        .map(|node| (node.marker8, node.marker12))
        .collect();
    let use_marker12 = should_use_marker12(&marker_pairs);
    raw.into_iter()
        .map(|node| LegacyPlaceNode {
            valid: node.valid,
            x: node.x,
            y: node.y,
            markers: if node.valid {
                if use_marker12 {
                    node.marker12
                } else {
                    node.marker8
                }
                .clamp(0, 1_000_000)
            } else {
                0
            },
            delay_sec: node.delay_sec,
            capacity: node.capacity,
            color_raw: node.color_raw,
            name: node.name,
        })
        .collect()
}

fn should_use_marker12(marker_pairs: &[(i32, i32)]) -> bool {
    if marker_pairs.is_empty() {
        return false;
    }
    let total = marker_pairs.len();
    let mut marker12_has_large_value = false;
    let mut marker8_is_legacy_sentinel = 0usize;
    let mut marker12_is_binary = 0usize;

    for (marker8, marker12) in marker_pairs.iter().copied() {
        if marker12 > 1 {
            marker12_has_large_value = true;
        }
        if marker8 == 10 {
            marker8_is_legacy_sentinel += 1;
        }
        if marker12 == 0 || marker12 == 1 {
            marker12_is_binary += 1;
        }
    }

    if marker12_has_large_value {
        return true;
    }

    marker8_is_legacy_sentinel * 10 >= total * 8 && marker12_is_binary * 10 >= total * 8
}

#[derive(Debug, Clone, Copy)]
struct LegacyArcRecord {
    place_idx: usize,
    transition_idx: usize,
    place_to_transition: bool,
    weight: u32,
    inhibitor: bool,
}

fn parse_arcs_from_section(
    bytes: &[u8],
    places: usize,
    transitions: usize,
    layout: LegacyLayout,
) -> Option<Vec<LegacyArcRecord>> {
    let arc_counter = read_i32(bytes, layout.arcs_offset)?;
    if arc_counter < -1 {
        return None;
    }
    let mut arc_count = (arc_counter + 1).max(0) as usize;
    let section_start = layout.arcs_offset + ARC_SECTION_HEADER_SIZE;
    if section_start > bytes.len() {
        return None;
    }
    let max_records = (bytes.len().saturating_sub(section_start)) / ARC_RECORD_SIZE;
    arc_count = arc_count.min(max_records);

    let mut counts = HashMap::<(usize, usize, bool, bool), u32>::new();
    let parse_one = |off: usize| -> Option<(usize, usize, bool, bool)> {
        let marker = read_i32(bytes, off + 24)?;
        if marker != 0 && marker != 1 {
            return None;
        }
        let direction = read_i32(bytes, off + 28)?;
        if direction != -1 && direction != 1 {
            return None;
        }
        let source_raw = read_i32(bytes, off + 32)?;
        let target_raw = read_i32(bytes, off + 36)?;
        if source_raw < 1 || target_raw < 1 {
            return None;
        }
        let (place_idx, transition_idx, place_to_transition) = if direction == -1 {
            if source_raw > places as i32 || target_raw > transitions as i32 {
                return None;
            }
            ((source_raw - 1) as usize, (target_raw - 1) as usize, true)
        } else {
            if source_raw > transitions as i32 || target_raw > places as i32 {
                return None;
            }
            ((target_raw - 1) as usize, (source_raw - 1) as usize, false)
        };
        Some((place_idx, transition_idx, place_to_transition, marker == 0))
    };

    let mut parsed_records = 0usize;
    for index in 0..arc_count {
        let off = section_start + index * ARC_RECORD_SIZE;
        if let Some((place_idx, transition_idx, place_to_transition, inhibitor)) = parse_one(off) {
            *counts
                .entry((place_idx, transition_idx, place_to_transition, inhibitor))
                .or_insert(0) += 1;
            parsed_records += 1;
        }
    }

    if parsed_records == 0 {
        return None;
    }

    let mut arcs = counts
        .into_iter()
        .map(
            |((place_idx, transition_idx, place_to_transition, inhibitor), weight)| {
                LegacyArcRecord {
                    place_idx,
                    transition_idx,
                    place_to_transition,
                    weight: weight.max(1),
                    inhibitor,
                }
            },
        )
        .collect::<Vec<_>>();
    arcs.sort_by_key(|arc| {
        (
            arc.place_idx,
            arc.transition_idx,
            arc.place_to_transition,
            arc.inhibitor,
        )
    });
    if arcs.is_empty() {
        None
    } else {
        Some(arcs)
    }
}

fn apply_legacy_arcs(model: &mut PetriNetModel, arcs: &[LegacyArcRecord]) {
    model.arcs.clear();
    model.inhibitor_arcs.clear();

    let mut next_arc_id = 1_u64;
    let mut next_inh_id = 1_u64;
    for arc in arcs {
        if arc.place_idx >= model.places.len() || arc.transition_idx >= model.transitions.len() {
            continue;
        }
        let place_id = model.places[arc.place_idx].id;
        let transition_id = model.transitions[arc.transition_idx].id;
        if arc.inhibitor {
            model.inhibitor_arcs.push(crate::model::InhibitorArc {
                id: next_inh_id,
                place_id,
                transition_id,
                threshold: arc.weight.max(1),
            });
            next_inh_id = next_inh_id.saturating_add(1);
        } else {
            let (from, to) = if arc.place_to_transition {
                (NodeRef::Place(place_id), NodeRef::Transition(transition_id))
            } else {
                (NodeRef::Transition(transition_id), NodeRef::Place(place_id))
            };
            model.arcs.push(crate::model::Arc {
                id: next_arc_id,
                from,
                to,
                weight: arc.weight.max(1),
            });
            next_arc_id = next_arc_id.saturating_add(1);
        }
    }
    model.rebuild_matrices_from_arcs();
}

fn prune_legacy_ghost_nodes(model: &mut PetriNetModel) {
    let place_count = model.places.len();
    let transition_count = model.transitions.len();
    if place_count == 0 || transition_count == 0 {
        return;
    }

    let mut place_incident = vec![false; place_count];
    let mut transition_incident = vec![false; transition_count];
    let place_index = model.place_index_map();
    let transition_index = model.transition_index_map();

    for arc in &model.arcs {
        match (arc.from, arc.to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid)) => {
                if let Some(&pi) = place_index.get(&pid) {
                    place_incident[pi] = true;
                }
                if let Some(&ti) = transition_index.get(&tid) {
                    transition_incident[ti] = true;
                }
            }
            (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                if let Some(&pi) = place_index.get(&pid) {
                    place_incident[pi] = true;
                }
                if let Some(&ti) = transition_index.get(&tid) {
                    transition_incident[ti] = true;
                }
            }
            _ => {}
        }
    }
    for inh in &model.inhibitor_arcs {
        if let Some(&pi) = place_index.get(&inh.place_id) {
            place_incident[pi] = true;
        }
        if let Some(&ti) = transition_index.get(&inh.transition_id) {
            transition_incident[ti] = true;
        }
    }

    let mut keep_places = vec![true; place_count];
    for idx in 0..place_count {
        if place_incident[idx] {
            continue;
        }
        let node = &model.places[idx];
        let duplicate_connected_exists = model
            .places
            .iter()
            .enumerate()
            .any(|(other_idx, other)| {
                other_idx != idx
                    && place_incident[other_idx]
                    && near_point(other.pos, node.pos[0], node.pos[1], 0.5)
            });
        if duplicate_connected_exists {
            keep_places[idx] = false;
        }
    }

    let mut keep_transitions = vec![true; transition_count];
    for idx in 0..transition_count {
        if transition_incident[idx] {
            continue;
        }
        let node = &model.transitions[idx];
        let duplicate_connected_exists = model
            .transitions
            .iter()
            .enumerate()
            .any(|(other_idx, other)| {
                other_idx != idx
                    && transition_incident[other_idx]
                    && near_point(other.pos, node.pos[0], node.pos[1], 0.5)
            });
        if duplicate_connected_exists {
            keep_transitions[idx] = false;
        }
    }

    if keep_places.iter().all(|keep| *keep) && keep_transitions.iter().all(|keep| *keep) {
        return;
    }

    let old_places = model.places.clone();
    let old_transitions = model.transitions.clone();
    let old_tables = model.tables.clone();

    let mut place_old_to_new = vec![None; old_places.len()];
    let mut transition_old_to_new = vec![None; old_transitions.len()];

    model.places.clear();
    for (old_idx, place) in old_places.into_iter().enumerate() {
        if keep_places[old_idx] {
            place_old_to_new[old_idx] = Some(model.places.len());
            model.places.push(place);
        }
    }

    model.transitions.clear();
    for (old_idx, tr) in old_transitions.into_iter().enumerate() {
        if keep_transitions[old_idx] {
            transition_old_to_new[old_idx] = Some(model.transitions.len());
            model.transitions.push(tr);
        }
    }

    let keep_place_ids = model.place_index_map();
    let keep_transition_ids = model.transition_index_map();
    model.arcs.retain(|arc| match (arc.from, arc.to) {
        (NodeRef::Place(pid), NodeRef::Transition(tid))
        | (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
            keep_place_ids.contains_key(&pid) && keep_transition_ids.contains_key(&tid)
        }
        _ => false,
    });
    model
        .inhibitor_arcs
        .retain(|arc| keep_place_ids.contains_key(&arc.place_id) && keep_transition_ids.contains_key(&arc.transition_id));

    model.tables.resize(model.places.len(), model.transitions.len());
    for (old_idx, maybe_new_idx) in place_old_to_new.into_iter().enumerate() {
        let Some(new_idx) = maybe_new_idx else {
            continue;
        };
        model.tables.m0[new_idx] = old_tables.m0.get(old_idx).copied().unwrap_or(0);
        model.tables.mo[new_idx] = old_tables.mo.get(old_idx).copied().unwrap_or(None);
        model.tables.mz[new_idx] = old_tables.mz.get(old_idx).copied().unwrap_or(0.0);
    }
    for (old_idx, maybe_new_idx) in transition_old_to_new.into_iter().enumerate() {
        let Some(new_idx) = maybe_new_idx else {
            continue;
        };
        model.tables.mpr[new_idx] = old_tables.mpr.get(old_idx).copied().unwrap_or(1);
    }

    model.rebuild_matrices_from_arcs();
}

fn apply_legacy_read_arc_heuristics(model: &mut PetriNetModel) {
    // NetStar legacy files can encode "resource" places as ordinary arcs, but semantically those arcs
    // may behave like read-arcs (test for token without consuming it). Without this, some imported
    // networks deadlock immediately.
    let places = model.places.len();
    let transitions = model.transitions.len();
    if places == 0 || transitions == 0 {
        return;
    }

    let mut changed = false;
    for p in 0..places {
        let name = model.places[p].name.to_lowercase();
        let looks_like_free_resource = name.contains("свобод") || name.contains("free");
        if !looks_like_free_resource {
            continue;
        }

        // Heuristic: 1 token resource used by many transitions, and at least one transition returns it.
        if model.tables.m0.get(p).copied().unwrap_or(0) != 1 {
            continue;
        }
        let outgoing = (0..transitions).filter(|&t| model.tables.pre[p][t] > 0).count();
        let incoming = (0..transitions).filter(|&t| model.tables.post[p][t] > 0).count();
        if outgoing < 3 || incoming == 0 {
            continue;
        }

        for t in 0..transitions {
            let pre = model.tables.pre[p][t];
            if pre > 0 && model.tables.post[p][t] == 0 {
                model.tables.post[p][t] = pre;
                changed = true;
            }
        }
    }

    if changed {
        model.rebuild_arcs_from_matrices();
    }
}

fn parse_arcs_by_signature(
    bytes: &[u8],
    places: usize,
    transitions: usize,
    place_nodes: &[crate::model::Place],
    transition_nodes: &[crate::model::Transition],
) -> Option<Vec<LegacyArcRecord>> {
    if bytes.len() < 64 || places == 0 || transitions == 0 {
        return None;
    }

    let read_i32 = |off: usize| -> Option<i32> {
        if off + 4 > bytes.len() {
            None
        } else {
            Some(i32::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
            ]))
        }
    };

    let mut counts_all = HashMap::<(usize, usize, bool, bool), u32>::new();
    let mut counts_filtered = HashMap::<(usize, usize, bool, bool), u32>::new();
    for off in 0..bytes.len().saturating_sub(64) {
        let Some(marker) = read_i32(off + 24) else {
            continue;
        };
        if marker != 1 && marker != 0 {
            continue;
        }
        let Some(direction_raw) = read_i32(off + 28) else {
            continue;
        };
        if direction_raw != -1 && direction_raw != 1 {
            continue;
        }
        let Some(source_raw) = read_i32(off + 32) else {
            continue;
        };
        let Some(target_raw) = read_i32(off + 36) else {
            continue;
        };
        if source_raw < 1 || target_raw < 1 {
            continue;
        }
        let x1 = read_i32(off + 40).unwrap_or(0);
        let y1 = read_i32(off + 44).unwrap_or(0);
        let x2 = read_i32(off + 56).unwrap_or(0);
        let y2 = read_i32(off + 60).unwrap_or(0);
        if !(-50_000..=50_000).contains(&x1)
            || !(-50_000..=50_000).contains(&y1)
            || !(-50_000..=50_000).contains(&x2)
            || !(-50_000..=50_000).contains(&y2)
        {
            continue;
        }
        let (place_idx, transition_idx, place_to_transition) = if direction_raw == -1 {
            if source_raw > places as i32 || target_raw > transitions as i32 {
                continue;
            }
            ((source_raw - 1) as usize, (target_raw - 1) as usize, true)
        } else {
            if source_raw > transitions as i32 || target_raw > places as i32 {
                continue;
            }
            ((target_raw - 1) as usize, (source_raw - 1) as usize, false)
        };
        let inhibitor = marker == 0;
        let dedup_key = (place_idx, transition_idx, place_to_transition, inhibitor);
        *counts_all.entry(dedup_key).or_insert(0) += 1;

        if place_idx < place_nodes.len() && transition_idx < transition_nodes.len() {
            let pp = place_nodes[place_idx].pos;
            let tp = transition_nodes[transition_idx].pos;
            let end_a_ok = near_point(pp, x1 as f32, y1 as f32, 220.0)
                && near_point(tp, x2 as f32, y2 as f32, 220.0);
            let end_b_ok = near_point(pp, x2 as f32, y2 as f32, 220.0)
                && near_point(tp, x1 as f32, y1 as f32, 220.0);
            if end_a_ok || end_b_ok {
                *counts_filtered.entry(dedup_key).or_insert(0) += 1;
            }
        }
    }

    let chosen = if counts_filtered.len() > counts_all.len() {
        counts_filtered
    } else {
        counts_all
    };

    let mut arcs = chosen
        .into_iter()
        .map(
            |((place_idx, transition_idx, place_to_transition, inhibitor), weight)| {
                LegacyArcRecord {
                    place_idx,
                    transition_idx,
                    place_to_transition,
                    weight: weight.max(1),
                    inhibitor,
                }
            },
        )
        .collect::<Vec<_>>();
    arcs.sort_by_key(|arc| {
        (
            arc.place_idx,
            arc.transition_idx,
            arc.place_to_transition,
            arc.inhibitor,
        )
    });

    if arcs.is_empty() {
        None
    } else {
        Some(arcs)
    }
}

fn detect_counts(bytes: &[u8]) -> Vec<(usize, u32, u32)> {
    let mut result = Vec::new();
    let scan_limit = bytes.len().min(4096);
    let mut offset = 0usize;

    while offset + 8 <= scan_limit {
        let p = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let t = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        if (1..=10_000).contains(&p) && (1..=10_000).contains(&t) {
            result.push((offset, p, t));
            if result.len() >= 10 {
                break;
            }
        }
        offset += 4;
    }

    result
}

fn legacy_arc_polyline_points(
    model: &PetriNetModel,
    from: NodeRef,
    to: NodeRef,
) -> Option<([f32; 2], [f32; 2], [f32; 2])> {
    let from_center = legacy_node_center(model, from)?;
    let to_center = legacy_node_center(model, to)?;
    let mut dir = [to_center[0] - from_center[0], to_center[1] - from_center[1]];
    let dir_len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
    if dir_len > f32::EPSILON {
        dir[0] /= dir_len;
        dir[1] /= dir_len;
    } else {
        dir = [1.0, 0.0];
    }

    let from_anchor = legacy_node_anchor(model, from, dir)?;
    let to_anchor = legacy_node_anchor(model, to, [-dir[0], -dir[1]])?;
    let middle = [
        (from_anchor[0] + to_anchor[0]) * 0.5,
        (from_anchor[1] + to_anchor[1]) * 0.5,
    ];
    Some((from_anchor, middle, to_anchor))
}

fn legacy_node_center(model: &PetriNetModel, node: NodeRef) -> Option<[f32; 2]> {
    match node {
        NodeRef::Place(id) => model.places.iter().find(|item| item.id == id).map(|item| item.pos),
        NodeRef::Transition(id) => model.transitions.iter().find(|item| item.id == id).map(|item| {
            let (w, h) = legacy_transition_dims(item.size);
            [item.pos[0] + w * 0.5, item.pos[1] + h * 0.5]
        }),
    }
}

fn legacy_node_anchor(model: &PetriNetModel, node: NodeRef, dir: [f32; 2]) -> Option<[f32; 2]> {
    match node {
        NodeRef::Place(id) => model.places.iter().find(|item| item.id == id).map(|item| {
            let r = legacy_place_radius(item.size);
            [item.pos[0] + dir[0] * r, item.pos[1] + dir[1] * r]
        }),
        NodeRef::Transition(id) => model.transitions.iter().find(|item| item.id == id).map(|item| {
            let (w, h) = legacy_transition_dims(item.size);
            let center = [item.pos[0] + w * 0.5, item.pos[1] + h * 0.5];
            let half_w = w * 0.5;
            let half_h = h * 0.5;
            let tx = if dir[0].abs() > f32::EPSILON {
                half_w / dir[0].abs()
            } else {
                f32::INFINITY
            };
            let ty = if dir[1].abs() > f32::EPSILON {
                half_h / dir[1].abs()
            } else {
                f32::INFINITY
            };
            let t = tx.min(ty);
            if t.is_finite() {
                [center[0] + dir[0] * t, center[1] + dir[1] * t]
            } else {
                center
            }
        }),
    }
}

fn legacy_place_radius(size: VisualSize) -> f32 {
    match size {
        VisualSize::Small => 14.0,
        VisualSize::Medium => 20.0,
        VisualSize::Large => 28.0,
    }
}

fn legacy_transition_dims(size: VisualSize) -> (f32, f32) {
    match size {
        VisualSize::Small => (10.0, 18.0),
        VisualSize::Medium => (12.0, 28.0),
        VisualSize::Large => (16.0, 38.0),
    }
}

fn map_color_to_legacy(color: NodeColor) -> i32 {
    match color {
        NodeColor::Blue => 0x000000FF,
        NodeColor::Green => 0x0000FF00,
        NodeColor::Red => 0x00FF0000,
        NodeColor::Yellow => 0x00FF0100,
        NodeColor::Default => 0,
    }
}

fn legacy_footer_template() -> &'static [u8] {
    &[
        0x58, 0x81, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x12, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x28, 0x63, 0x29, 0x20,
        0x4D, 0x69, 0x6B, 0x68, 0x61, 0x79, 0x6C, 0x69,
        0x73, 0x68, 0x69, 0x6E,
    ]
}

fn read_legacy_name(
    bytes: &[u8],
    record_offset: usize,
    record_size: usize,
    field_offset: usize,
) -> String {
    if field_offset + 1 >= record_size {
        return String::new();
    }
    let len_off = record_offset.saturating_add(field_offset);
    if len_off >= bytes.len() {
        return String::new();
    }

    let len = bytes[len_off] as usize;
    if len == 0 {
        return String::new();
    }

    let value_off = len_off.saturating_add(1);
    let record_end = record_offset.saturating_add(record_size).min(bytes.len());
    if value_off >= record_end {
        return String::new();
    }
    let max_len = record_end.saturating_sub(value_off);
    let len = len.min(max_len);
    let raw = &bytes[value_off..value_off + len];
    decode_legacy_cp1251(raw)
        .trim_matches(|ch: char| ch.is_whitespace() || ch.is_control())
        .to_string()
}

fn decode_legacy_cp1251(raw: &[u8]) -> String {
    raw.iter()
        .copied()
        .map(|b| match b {
            0x00..=0x7F => b as char,
            0xA8 => '\u{0401}',
            0xB8 => '\u{0451}',
            0xC0..=0xFF => {
                let code = 0x0410 + (b - 0xC0) as u32;
                char::from_u32(code).unwrap_or('\u{FFFD}')
            }
            _ => '\u{FFFD}',
        })
        .collect()
}

fn encode_legacy_cp1251(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let b = match ch {
            '\u{0000}'..='\u{007F}' => ch as u8,
            '\u{0401}' => 0xA8, // Ё
            '\u{0451}' => 0xB8, // ё
            '\u{0410}'..='\u{044F}' => (0xC0u32 + (ch as u32 - 0x0410)) as u8, // А..я
            _ => b'?', // unsupported in our legacy subset
        };
        out.push(b);
    }
    out
}

fn write_legacy_name(record: &mut [u8], field_offset: usize, value: &str) {
    if field_offset + 1 >= record.len() {
        return;
    }
    let trimmed = value.trim();
    if trimmed.is_empty() {
        record[field_offset] = 0;
        return;
    }

    let encoded = encode_legacy_cp1251(trimmed);
    let max_len = record.len().saturating_sub(field_offset + 1);
    let len = encoded.len().min(max_len).min(255);
    record[field_offset] = len as u8;
    record[field_offset + 1..field_offset + 1 + len].copy_from_slice(&encoded[..len]);
}

fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    if offset + 4 > bytes.len() {
        return None;
    }
    Some(i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

fn read_f64(bytes: &[u8], offset: usize) -> Option<f64> {
    if offset + 8 > bytes.len() {
        return None;
    }
    Some(f64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ]))
}

fn write_i32(target: &mut [u8], offset: usize, value: i32) {
    if offset + 4 <= target.len() {
        target[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

fn write_u16(target: &mut [u8], offset: usize, value: u16) {
    if offset + 2 <= target.len() {
        target[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn write_f64(target: &mut [u8], offset: usize, value: f64) {
    if offset + 8 <= target.len() {
        target[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
}

fn push_i32(target: &mut Vec<u8>, value: i32) {
    target.extend_from_slice(&value.to_le_bytes());
}

fn round_i32(value: f32) -> i32 {
    if value.is_finite() {
        value.round().clamp(-2_000_000_000.0, 2_000_000_000.0) as i32
    } else {
        0
    }
}

fn clamp_u16(value: f32) -> u16 {
    if value.is_finite() {
        value.round().clamp(0.0, u16::MAX as f32) as u16
    } else {
        0
    }
}

fn map_legacy_color(raw: i32) -> NodeColor {
    let value = raw as u32;
    match value {
        0x000000FF => NodeColor::Blue,
        0x0000FF00 => NodeColor::Green,
        0x00FF0000 => NodeColor::Red,
        0x00FFFF00 | 0x00FF0100 => NodeColor::Yellow,
        _ => NodeColor::Default,
    }
}

fn near_point(center: [f32; 2], x: f32, y: f32, max_dist: f32) -> bool {
    let dx = center[0] - x;
    let dy = center[1] - y;
    dx * dx + dy * dy <= max_dist * max_dist
}

pub fn extract_ascii_strings(bytes: &[u8], min_len: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut start = None;

    for (i, b) in bytes.iter().copied().enumerate() {
        if b.is_ascii_graphic() || b == b' ' {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            if i - s >= min_len {
                out.push((s, String::from_utf8_lossy(&bytes[s..i]).to_string()));
            }
        }
    }

    if let Some(s) = start {
        if bytes.len() - s >= min_len {
            out.push((s, String::from_utf8_lossy(&bytes[s..]).to_string()));
        }
    }

    out
}

pub fn extract_utf16le_strings(bytes: &[u8], min_len: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;

    while i + 2 <= bytes.len() {
        let start = i;
        let mut data = Vec::new();

        while i + 2 <= bytes.len() {
            let lo = bytes[i];
            let hi = bytes[i + 1];
            if hi == 0 && (lo.is_ascii_graphic() || lo == b' ') {
                data.push(lo as u16);
                i += 2;
            } else {
                break;
            }
        }

        if data.len() >= min_len {
            if let Ok(s) = String::from_utf16(&data) {
                out.push((start, s));
            }
        }

        i = if i == start { i + 1 } else { i + 2 };
    }

    out
}

pub fn extract_float64_pairs(bytes: &[u8], max_items: usize) -> Vec<(usize, f64, f64)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 16 <= bytes.len() && out.len() < max_items {
        let a = f64::from_le_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        let b = f64::from_le_bytes([
            bytes[i + 8],
            bytes[i + 9],
            bytes[i + 10],
            bytes[i + 11],
            bytes[i + 12],
            bytes[i + 13],
            bytes[i + 14],
            bytes[i + 15],
        ]);
        if a.is_finite() && b.is_finite() && a.abs() <= 1.0e8 && b.abs() <= 1.0e8 {
            out.push((i, a, b));
        }
        i += 8;
    }
    out
}

fn detect_section_boundaries(bytes: &[u8]) -> Vec<String> {
    let mut sections = Vec::new();
    for (off, p, t) in detect_counts(bytes).into_iter().take(5) {
        sections.push(format!(
            "РљР°РЅРґРёРґР°С‚ СЃРµРєС†РёРё counts @0x{off:08X}: places={p}, transitions={t}"
        ));
    }
    sections
}
