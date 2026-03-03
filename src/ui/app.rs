use std::fs;

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};

use crate::formats::atf::generate_atf;
use crate::io::{LegacyExportHints, load_gpn, save_gpn_with_hints};
use crate::model::{LabelPosition, Language, NodeColor, NodeRef, PetriNet, Place, PlaceStatisticsSelection, StochasticDistribution, Tool, Transition, UiDecorativeFrame, UiTextBlock, VisualSize};
use crate::sim::engine::{run_simulation, SimulationParams, SimulationResult};

mod graph_view;
mod shortcuts;
mod table_view;
mod tool_palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Cascade,
    TileHorizontal,
    TileVertical,
    Minimized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArcDisplayMode {
    All,
    OnlyColor,
    Hidden,
}

#[derive(Debug, Clone)]
struct CanvasState {
    zoom: f32,
    pan: Vec2,
    selected_place: Option<u64>,
    selected_transition: Option<u64>,
    selected_places: Vec<u64>,
    selected_transitions: Vec<u64>,
    selected_arc: Option<u64>,
    selected_arcs: Vec<u64>,
    selected_text: Option<u64>,
    selected_frame: Option<u64>,
    arc_start: Option<NodeRef>,
    cursor_world: [f32; 2],
    selection_start: Option<Pos2>,
    selection_rect: Option<Rect>,
    drag_prev_world: Option<[f32; 2]>,
    move_drag_active: bool,
    frame_draw_start_world: Option<[f32; 2]>,
    frame_draw_current_world: Option<[f32; 2]>,
    frame_resize_id: Option<u64>,
    cursor_valid: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::new(0.0, 0.0),
            selected_place: None,
            selected_transition: None,
            selected_places: Vec::new(),
            selected_transitions: Vec::new(),
            selected_arc: None,
            selected_arcs: Vec::new(),
            selected_text: None,
            selected_frame: None,
            arc_start: None,
            cursor_world: [0.0, 0.0],
            selection_start: None,
            selection_rect: None,
            drag_prev_world: None,
            move_drag_active: false,
            frame_draw_start_world: None,
            frame_draw_current_world: None,
            frame_resize_id: None,
            cursor_valid: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanvasTextBlock {
    id: u64,
    pos: [f32; 2],
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanvasFrame {
    id: u64,
    pos: [f32; 2],
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyCanvasFrame {
    id: u64,
    pos: [f32; 2],
    side: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyUiSidecar {
    version: u32,
    #[serde(default)]
    text_blocks: Vec<CanvasTextBlock>,
    #[serde(default)]
    decorative_frames: Vec<LegacyCanvasFrame>,
    #[serde(default)]
    next_text_id: u64,
    #[serde(default)]
    next_frame_id: u64,
}#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedPlace {
    place: Place,
    m0: u32,
    mo: Option<u32>,
    mz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedTransition {
    transition: Transition,
    mpr: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedArc {
    from: NodeRef,
    to: NodeRef,
    weight: u32,
    color: NodeColor,
    visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedInhibitorArc {
    place_id: u64,
    transition_id: u64,
    threshold: u32,
    color: NodeColor,
    visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedTextBlock {
    pos: [f32; 2],
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopyBuffer {
    origin: [f32; 2],
    places: Vec<CopiedPlace>,
    transitions: Vec<CopiedTransition>,
    arcs: Vec<CopiedArc>,
    inhibitors: Vec<CopiedInhibitorArc>,
    texts: Vec<CopiedTextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardPayload {
    version: u32,
    buffer: CopyBuffer,
}

#[derive(Debug, Clone)]
struct UndoSnapshot {
    net: PetriNet,
    text_blocks: Vec<CanvasTextBlock>,
    next_text_id: u64,
    decorative_frames: Vec<CanvasFrame>,
    next_frame_id: u64,
}

pub struct PetriApp {
    net: PetriNet,
    tool: Tool,
    canvas: CanvasState,
    sim_params: SimulationParams,
    sim_result: Option<SimulationResult>,
    show_sim_params: bool,
    show_results: bool,
    show_atf: bool,
    atf_selected_place: usize,
    atf_text: String,
    file_path: Option<PathBuf>,
    last_error: Option<String>,
    layout_mode: LayoutMode,
    show_graph_view: bool,
    show_table_view: bool,
    table_fullscreen: bool,
    place_props_id: Option<u64>,
    transition_props_id: Option<u64>,
    show_place_props: bool,
    show_transition_props: bool,
    show_debug: bool,
    debug_step: usize,
    debug_playing: bool,
    debug_interval_ms: u64,
    last_debug_tick: Option<Instant>,
    show_proof: bool,
    text_blocks: Vec<CanvasTextBlock>,
    next_text_id: u64,
    decorative_frames: Vec<CanvasFrame>,
    next_frame_id: u64,
    clipboard: Option<CopyBuffer>,
    paste_serial: u32,
    undo_stack: Vec<UndoSnapshot>,
    legacy_export_hints: Option<LegacyExportHints>,
    status_hint: Option<String>,
    show_help_development: bool,
    show_help_controls: bool,
    place_stats_dialog_place_id: Option<u64>,
    place_stats_dialog_backup: Option<(u64, PlaceStatisticsSelection)>,
    show_place_stats_window: bool,
    place_stats_view_place: usize,
    arc_display_mode: ArcDisplayMode,
    arc_display_color: NodeColor,
}

#[derive(Clone, Copy, Debug)]
enum MatrixCsvTarget {
    Pre,
    Post,
    Inhibitor,
}

impl PetriApp {
    const GRID_STEP_SNAP: f32 = 10.0;
    const GRID_STEP_FREE: f32 = 25.0;
    const CLIPBOARD_PREFIX: &'static str = "PETRINET_COPY_V1:";
    const FRAME_MIN_SIDE: f32 = 10.0;
    const FRAME_RESIZE_HANDLE_PX: f32 = 10.0;

    fn grid_step_world(&self) -> f32 {
        if self.net.ui.snap_to_grid {
            Self::GRID_STEP_SNAP
        } else {
            Self::GRID_STEP_FREE
        }
    }

    fn write_copy_buffer_to_system_clipboard(&mut self, buf: &CopyBuffer) {
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

    fn read_copy_buffer_from_system_clipboard(&self) -> Option<CopyBuffer> {
        let mut clipboard = arboard::Clipboard::new().ok()?;
        let text = clipboard.get_text().ok()?;
        // Guard against accidental huge clipboard payloads that can freeze UI on parse.
        if text.len() > 4 * 1024 * 1024 {
            return None;
        }
        let payload = text.strip_prefix(Self::CLIPBOARD_PREFIX)?;
        let parsed: ClipboardPayload = serde_json::from_str(payload).ok()?;
        Some(parsed.buffer)
    }

    fn snap_scalar_to_grid(&self, v: f32) -> f32 {
        let step = self.grid_step_world();
        (v / step).round() * step
    }

    fn snap_point_to_grid(&self, p: [f32; 2]) -> [f32; 2] {
        [self.snap_scalar_to_grid(p[0]), self.snap_scalar_to_grid(p[1])]
    }

    #[cfg(test)]
    fn new_for_tests() -> Self {
        let mut net = PetriNet::new();
        net.set_counts(2, 1);
        net.places[0].pos = [120.0, 150.0];
        net.places[1].pos = [340.0, 150.0];
        net.transitions[0].pos = [240.0, 145.0];

        Self {
            net,
            tool: Tool::Edit,
            canvas: CanvasState::default(),
            sim_params: SimulationParams::default(),
            sim_result: None,
            show_sim_params: false,
            show_results: false,
            show_atf: false,
            atf_selected_place: 0,
            atf_text: String::new(),
            file_path: None,
            last_error: None,
            layout_mode: LayoutMode::TileVertical,
            show_graph_view: true,
            show_table_view: false,
            table_fullscreen: false,
            place_props_id: None,
            transition_props_id: None,
            show_place_props: false,
            show_transition_props: false,
            show_debug: false,
            debug_step: 0,
            debug_playing: false,
            debug_interval_ms: 400,
            last_debug_tick: None,
            show_proof: false,
            text_blocks: Vec::new(),
            next_text_id: 1,
            decorative_frames: Vec::new(),
            next_frame_id: 1,
            clipboard: None,
            paste_serial: 0,
            undo_stack: Vec::new(),
            legacy_export_hints: None,
            status_hint: None,
            show_help_development: false,
            show_help_controls: false,
            place_stats_dialog_place_id: None,
            place_stats_dialog_backup: None,
            show_place_stats_window: false,
            place_stats_view_place: 0,
            arc_display_mode: ArcDisplayMode::All,
            arc_display_color: NodeColor::Default,
        }
    }

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(test)]
        {
            Self::new_for_tests()
        }
        #[cfg(not(test))]
        {
            let mut net = PetriNet::new();
            net.set_counts(2, 1);
            net.places[0].pos = [120.0, 150.0];
            net.places[1].pos = [340.0, 150.0];
            net.transitions[0].pos = [240.0, 145.0];

            Self {
                net,
                tool: Tool::Edit,
                canvas: CanvasState::default(),
                sim_params: SimulationParams::default(),
                sim_result: None,
                show_sim_params: false,
                show_results: false,
                show_atf: false,
                atf_selected_place: 0,
                atf_text: String::new(),
                file_path: None,
                last_error: None,
                layout_mode: LayoutMode::TileVertical,
                show_graph_view: true,
                show_table_view: false,
                table_fullscreen: false,
                place_props_id: None,
                transition_props_id: None,
                show_place_props: false,
                show_transition_props: false,
                show_debug: false,
                debug_step: 0,
                debug_playing: false,
                debug_interval_ms: 400,
                last_debug_tick: None,
                show_proof: false,
                text_blocks: Vec::new(),
                next_text_id: 1,
            decorative_frames: Vec::new(),
            next_frame_id: 1,
            clipboard: None,
                paste_serial: 0,
                undo_stack: Vec::new(),
                legacy_export_hints: None,
                status_hint: None,
                show_help_development: false,
                show_help_controls: false,
            place_stats_dialog_place_id: None,
            place_stats_dialog_backup: None,
            show_place_stats_window: false,
            place_stats_view_place: 0,
            arc_display_mode: ArcDisplayMode::All,
            arc_display_color: NodeColor::Default,
            }
        }
    }

    fn new_file(&mut self) {
        self.net = PetriNet::new();
        self.net.set_counts(1, 1);
        self.file_path = None;
        self.text_blocks.clear();
        self.next_text_id = 1;
        self.decorative_frames.clear();
        self.next_frame_id = 1;
        self.undo_stack.clear();
        self.legacy_export_hints = None;
        self.status_hint = None;
        self.canvas.cursor_valid = false;
    }

    fn reset_sim_stop_controls(&mut self) {
        self.sim_params.use_time_limit = false;
        self.sim_params.use_pass_limit = false;
        self.sim_params.stop.through_place = None;
        self.sim_params.stop.sim_time = None;
    }
    fn ui_sidecar_path(path: &std::path::Path) -> PathBuf {
        let mut os = path.as_os_str().to_os_string();
        os.push(".petriui.json");
        PathBuf::from(os)
    }

    fn sync_canvas_overlays_from_model(&mut self) {
        self.text_blocks = self
            .net
            .ui
            .text_blocks
            .iter()
            .map(|item| CanvasTextBlock {
                id: item.id,
                pos: item.pos,
                text: item.text.clone(),
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

        self.next_text_id = self
            .net
            .ui
            .next_text_id
            .max(self.text_blocks.iter().map(|t| t.id).max().unwrap_or(0).saturating_add(1));
        self.next_frame_id = self
            .net
            .ui
            .next_frame_id
            .max(self.decorative_frames.iter().map(|f| f.id).max().unwrap_or(0).saturating_add(1));
    }

    fn sync_model_overlays_from_canvas(&mut self) {
        self.net.ui.text_blocks = self
            .text_blocks
            .iter()
            .map(|item| UiTextBlock {
                id: item.id,
                pos: item.pos,
                text: item.text.clone(),
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

    fn load_legacy_sidecar_for_migration(&mut self, path: &std::path::Path) {
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
        self.next_text_id = sidecar
            .next_text_id
            .max(self.text_blocks.iter().map(|t| t.id).max().unwrap_or(0).saturating_add(1));
        self.next_frame_id = sidecar
            .next_frame_id
            .max(self.decorative_frames.iter().map(|f| f.id).max().unwrap_or(0).saturating_add(1));

        // Persist migrated overlays to GPN2 on next save.
        self.sync_model_overlays_from_canvas();
    }

    fn cleanup_legacy_sidecar(path: &std::path::Path) {
        let sidecar_path = Self::ui_sidecar_path(path);
        if sidecar_path.exists() {
            let _ = fs::remove_file(sidecar_path);
        }
    }

    fn extract_legacy_export_hints(path: &std::path::Path) -> Option<LegacyExportHints> {
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
        let arc_header_extra = Some(u16::from_le_bytes([bytes[arcs_off + 4], bytes[arcs_off + 5]]));
        Some(LegacyExportHints {
            places_count: Some(p),
            transitions_count: Some(t),
            arc_topology_fingerprint: None,
            arc_header_extra,
            footer_bytes,
            raw_arc_and_tail: Some(bytes[arcs_off..].to_vec()),
        })
    }

    fn arc_topology_fingerprint(net: &PetriNet) -> u64 {
        let mut place_idx = HashMap::<u64, usize>::new();
        for (idx, place) in net.places.iter().enumerate() {
            place_idx.insert(place.id, idx + 1);
        }
        let mut transition_idx = HashMap::<u64, usize>::new();
        for (idx, transition) in net.transitions.iter().enumerate() {
            transition_idx.insert(transition.id, idx + 1);
        }

        let mut edges = Vec::<(u8, i8, usize, usize, u32)>::new();
        for arc in &net.arcs {
            match (arc.from, arc.to) {
                (NodeRef::Place(place_id), NodeRef::Transition(transition_id)) => {
                    if let (Some(&p), Some(&t)) =
                        (place_idx.get(&place_id), transition_idx.get(&transition_id))
                    {
                        edges.push((0, -1, p, t, arc.weight.max(1)));
                    }
                }
                (NodeRef::Transition(transition_id), NodeRef::Place(place_id)) => {
                    if let (Some(&t), Some(&p)) =
                        (transition_idx.get(&transition_id), place_idx.get(&place_id))
                    {
                        edges.push((0, 1, t, p, arc.weight.max(1)));
                    }
                }
                _ => {}
            }
        }
        for inh in &net.inhibitor_arcs {
            if let (Some(&p), Some(&t)) = (
                place_idx.get(&inh.place_id),
                transition_idx.get(&inh.transition_id),
            ) {
                edges.push((1, -1, p, t, inh.threshold.max(1)));
            }
        }
        edges.sort_unstable();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        net.places.len().hash(&mut hasher);
        net.transitions.len().hash(&mut hasher);
        edges.hash(&mut hasher);
        hasher.finish()
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Р¤Р°Р№Р»С‹ PetriNet", &["gpn2", "pn", "gpn"])
            .pick_file()
        {
            match load_gpn(&path) {
                Ok(result) => {
                    let legacy_hints = if result.legacy_debug.is_some() {
                        let mut hints = Self::extract_legacy_export_hints(&path);
                        if let Some(h) = hints.as_mut() {
                            h.arc_topology_fingerprint =
                                Some(Self::arc_topology_fingerprint(&result.model));
                        }
                        hints
                    } else {
                        None
                    };
                    self.net = result.model;
                    self.net.set_counts(self.net.places.len(), self.net.transitions.len());
                    self.file_path = Some(path.clone());
                    self.undo_stack.clear();
                    self.sync_canvas_overlays_from_model();
                    self.load_legacy_sidecar_for_migration(&path);
                    self.legacy_export_hints = legacy_hints;
                    self.status_hint = None;
                    self.canvas.cursor_valid = false;
                    let filtered: Vec<String> = result
                        .warnings
                        .iter()
                        .filter(|w| {
                            !w.contains("best-effort")
                                && !w.contains("signature heuristic")
                                && !w.contains("РІРѕСЃСЃС‚Р°РЅРѕРІР»РµРЅС‹ РїРѕ СЃРёРіРЅР°С‚СѓСЂР°Рј")
                        })
                        .cloned()
                        .collect();
                    if filtered.is_empty() {
                        self.last_error = None;
                    } else {
                        self.last_error = Some(format!(
                            "РРјРїРѕСЂС‚ СЃ РїСЂРµРґСѓРїСЂРµР¶РґРµРЅРёСЏРјРё: {}",
                            filtered.join("; ")
                        ));
                    }
                }
                Err(e) => self.last_error = Some(e.to_string()),
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = self.file_path.clone() {
            self.sync_model_overlays_from_canvas();
            if let Err(e) = crate::io::gpn2::save_gpn2(&path, &self.net) {
                self.last_error = Some(e.to_string());
            } else {
                Self::cleanup_legacy_sidecar(&path);
            }
        } else {
            self.save_file_as();
        }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Р¤Р°Р№Р»С‹ PetriNet", &["gpn2", "pn", "gpn"])
            .set_file_name("РјРѕРґРµР»СЊ.gpn2")
            .save_file()
        {
            self.file_path = Some(path.clone());
            self.sync_model_overlays_from_canvas();
            if let Err(e) = crate::io::gpn2::save_gpn2(&path, &self.net) {
                self.last_error = Some(e.to_string());
            } else {
                Self::cleanup_legacy_sidecar(&path);
            }
        }
    }

    fn export_netstar_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Р¤Р°Р№Р»С‹ NetStar", &["gpn"])
            .set_file_name("СЌРєСЃРїРѕСЂС‚_netstar.gpn")
            .save_file()
        {
            if let Err(e) = save_gpn_with_hints(&path, &self.net, self.legacy_export_hints.as_ref()) {
                self.last_error = Some(e.to_string());
            }
        }
    }
    fn place_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.places.iter().position(|p| p.id == id)
    }

    fn transition_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.transitions.iter().position(|t| t.id == id)
    }

    fn parse_place_auto_index(name: &str) -> Option<usize> {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let first = chars.next()?;
        if !['P', 'p'].contains(&first) {
            return None;
        }
        let rest: String = chars.collect();
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        rest.parse::<usize>().ok()
    }

    fn parse_transition_auto_index(name: &str) -> Option<usize> {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let first = chars.next()?;
        if !['T', 't'].contains(&first) {
            return None;
        }
        let rest: String = chars.collect();
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        rest.parse::<usize>().ok()
    }


    fn assign_auto_name_for_place(&mut self, place_id: u64) {
        let mut ids: Vec<u64> = self.net.places.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        let rank = ids
            .iter()
            .position(|&id| id == place_id)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| self.net.places.len().max(1));
        let new_name = format!("P{rank}");
        if let Some(index) = self.place_idx_by_id(place_id) {
            self.net.places[index].name = new_name;
        }
    }

    fn assign_auto_name_for_transition(&mut self, transition_id: u64) {
        let mut ids: Vec<u64> = self.net.transitions.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let rank = ids
            .iter()
            .position(|&id| id == transition_id)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| self.net.transitions.len().max(1));
        let new_name = format!("T{rank}");
        if let Some(index) = self.transition_idx_by_id(transition_id) {
            self.net.transitions[index].name = new_name;
        }
    }

    fn tr<'a>(&self, ru: &'a str, en: &'a str) -> &'a str {
        match self.net.ui.language {
            Language::Ru => ru,
            Language::En => en,
        }
    }

    fn import_matrix_csv(&mut self, target: MatrixCsvTarget) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file()
        else {
            return;
        };

        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                self.last_error = Some(format!("CSV read error: {e}"));
                return;
            }
        };

        let first_line = text.lines().next().unwrap_or_default();
        let semi = first_line.matches(';').count();
        let comma = first_line.matches(',').count();
        let delim = if semi >= comma { ';' } else { ',' };

        let mut lines = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty());
        let Some(header) = lines.next() else {
            self.last_error = Some("CSV parse error: empty file".to_string());
            return;
        };

        let header_cells: Vec<&str> = header.split(delim).map(|c| c.trim()).collect();
        if header_cells.len() < 2 {
            self.last_error = Some("CSV parse error: missing header columns".to_string());
            return;
        }

        let parse_ordinal = |s: &str, prefix: char| -> Option<usize> {
            let s = s.trim();
            let s = s.strip_prefix(prefix)?;
            let n: usize = s.parse().ok()?;
            n.checked_sub(1)
        };

        let mut col_map: Vec<usize> = Vec::new();
        for (col_idx, raw) in header_cells.iter().skip(1).enumerate() {
            col_map.push(parse_ordinal(raw, 'T').unwrap_or(col_idx));
        }

        let mut entries: Vec<(usize, usize, u32)> = Vec::new();
        let mut required_p = 0usize;
        let mut required_t = col_map.iter().copied().max().unwrap_or(0).saturating_add(1);

        for (row_idx, line) in lines.enumerate() {
            let cells: Vec<&str> = line.split(delim).map(|c| c.trim()).collect();
            if cells.len() < 2 {
                continue;
            }
            let p_idx = parse_ordinal(cells[0], 'P').unwrap_or(row_idx);
            required_p = required_p.max(p_idx + 1);

            for (ci, raw_val) in cells.iter().skip(1).enumerate() {
                let t_idx = *col_map.get(ci).unwrap_or(&ci);
                required_t = required_t.max(t_idx + 1);

                if raw_val.is_empty() {
                    continue;
                }

                let parsed: i64 = match raw_val.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.last_error = Some(format!("CSV parse error: invalid number '{raw_val}'"));
                        return;
                    }
                };
                if parsed < 0 {
                    self.last_error = Some(format!("CSV parse error: negative value '{raw_val}'"));
                    return;
                }
                let val: u32 = match parsed.try_into() {
                    Ok(v) => v,
                    Err(_) => {
                        self.last_error = Some(format!("CSV parse error: value too large '{raw_val}'"));
                        return;
                    }
                };
                entries.push((p_idx, t_idx, val));
            }
        }

        if required_p == 0 || required_t == 0 {
            self.last_error = Some("CSV parse error: empty matrix".to_string());
            return;
        }

        let cur_p = self.net.places.len();
        let cur_t = self.net.transitions.len();
        if required_p > cur_p || required_t > cur_t {
            self.net.set_counts(cur_p.max(required_p), cur_t.max(required_t));
        }

        match target {
            MatrixCsvTarget::Pre => {
                for (p, t, v) in entries {
                    if p < self.net.tables.pre.len() && t < self.net.tables.pre[p].len() {
                        self.net.tables.pre[p][t] = v;
                    }
                }
            }
            MatrixCsvTarget::Post => {
                for (p, t, v) in entries {
                    if p < self.net.tables.post.len() && t < self.net.tables.post[p].len() {
                        self.net.tables.post[p][t] = v;
                    }
                }
            }
            MatrixCsvTarget::Inhibitor => {
                for (p, t, v) in entries {
                    if p < self.net.tables.inhibitor.len() && t < self.net.tables.inhibitor[p].len() {
                        self.net.tables.inhibitor[p][t] = v;
                    }
                }
            }
        }

        self.net.rebuild_arcs_from_matrices();
        self.last_error = None;
        let target_name = match target {
            MatrixCsvTarget::Pre => "Pre",
            MatrixCsvTarget::Post => "Post",
            MatrixCsvTarget::Inhibitor => "Inhibitor",
        };
        self.status_hint = Some(format!(
            "{}: {}x{} -> {}",
            self.tr("РРјРїРѕСЂС‚ CSV", "CSV import"),
            required_p,
            required_t,
            target_name
        ));
    }

    fn debug_visible_log_indices(result: &SimulationResult) -> Vec<usize> {
        if result.logs.is_empty() {
            return Vec::new();
        }

        // Step 0 in debug must always point to the initial state.
        let mut indices = vec![0usize];
        let mut previous_marking = result.logs[0].marking.as_slice();
        for (idx, entry) in result.logs.iter().enumerate().skip(1) {
            let marking_changed = previous_marking != entry.marking.as_slice();
            if entry.fired_transition.is_some() || marking_changed {
                indices.push(idx);
            }
            previous_marking = entry.marking.as_slice();
        }
        indices
    }

    fn label_pos_text(pos: LabelPosition, is_ru: bool) -> &'static str {
        match (pos, is_ru) {
            (LabelPosition::Top, true) => "Р’РІРµСЂС…Сѓ",
            (LabelPosition::Bottom, true) => "Р’РЅРёР·Сѓ",
            (LabelPosition::Left, true) => "РЎР»РµРІР°",
            (LabelPosition::Right, true) => "РЎРїСЂР°РІР°",
            (LabelPosition::Center, true) => "РџРѕ С†РµРЅС‚СЂСѓ",
            (LabelPosition::Top, false) => "Top",
            (LabelPosition::Bottom, false) => "Bottom",
            (LabelPosition::Left, false) => "Left",
            (LabelPosition::Right, false) => "Right",
            (LabelPosition::Center, false) => "Center",
        }
    }

    fn node_color_text(color: NodeColor, is_ru: bool) -> &'static str {
        match (color, is_ru) {
            (NodeColor::Default, true) => "РџРѕ СѓРјРѕР»С‡Р°РЅРёСЋ",
            (NodeColor::Blue, true) => "РЎРёРЅРёР№",
            (NodeColor::Red, true) => "РљСЂР°СЃРЅС‹Р№",
            (NodeColor::Green, true) => "Р—РµР»РµРЅС‹Р№",
            (NodeColor::Yellow, true) => "Р–РµР»С‚С‹Р№",
            (NodeColor::Default, false) => "Default",
            (NodeColor::Blue, false) => "Blue",
            (NodeColor::Red, false) => "Red",
            (NodeColor::Green, false) => "Green",
            (NodeColor::Yellow, false) => "Yellow",
        }
    }

    fn stochastic_text(dist: &StochasticDistribution, is_ru: bool) -> &'static str {
        match (dist, is_ru) {
            (StochasticDistribution::None, true) => "РќРµС‚",
            (StochasticDistribution::Uniform { .. }, true) => "Р Р°РІРЅРѕРјРµСЂРЅРѕРµ",
            (StochasticDistribution::Normal { .. }, true) => "РќРѕСЂРјР°Р»СЊРЅРѕРµ (Р“Р°СѓСЃСЃР°)",
            (StochasticDistribution::Exponential { .. }, true) => "Р­РєСЃРїРѕРЅРµРЅС†РёР°Р»СЊРЅРѕРµ",
            (StochasticDistribution::Poisson { .. }, true) => "РџСѓР°СЃСЃРѕРЅР°",
            (StochasticDistribution::CustomValue { .. }, true) => "Р—Р°РґР°РЅРЅРѕРµ РїРѕР»СЊР·РѕРІР°С‚РµР»РµРј",
            (StochasticDistribution::None, false) => "None",
            (StochasticDistribution::Uniform { .. }, false) => "Uniform",
            (StochasticDistribution::Normal { .. }, false) => "Normal (Gaussian)",
            (StochasticDistribution::Exponential { .. }, false) => "Exponential",
            (StochasticDistribution::Poisson { .. }, false) => "Poisson",
            (StochasticDistribution::CustomValue { .. }, false) => "User-defined",
        }
    }

    fn color_to_egui(color: NodeColor, fallback: Color32) -> Color32 {
        match color {
            NodeColor::Default => fallback,
            NodeColor::Blue => Color32::from_rgb(25, 90, 220),
            NodeColor::Red => Color32::from_rgb(200, 40, 40),
            NodeColor::Green => Color32::from_rgb(40, 150, 60),
            NodeColor::Yellow => Color32::from_rgb(200, 160, 20),
        }
    }

    fn arc_display_mode_text(mode: ArcDisplayMode, is_ru: bool) -> &'static str {
        match (mode, is_ru) {
            (ArcDisplayMode::All, true) => "Р’СЃРµ",
            (ArcDisplayMode::OnlyColor, true) => "РўРѕР»СЊРєРѕ РІС‹Р±СЂР°РЅРЅС‹Р№ С†РІРµС‚",
            (ArcDisplayMode::Hidden, true) => "РЎРєСЂС‹С‚СЊ РІСЃРµ",
            (ArcDisplayMode::All, false) => "All",
            (ArcDisplayMode::OnlyColor, false) => "Only selected color",
            (ArcDisplayMode::Hidden, false) => "Hide all",
        }
    }

    fn arc_visible_by_mode(&self, color: NodeColor, per_arc_visible: bool) -> bool {
        if !per_arc_visible {
            return false;
        }
        match self.arc_display_mode {
            ArcDisplayMode::All => true,
            ArcDisplayMode::OnlyColor => color == self.arc_display_color,
            ArcDisplayMode::Hidden => false,
        }
    }

    fn place_radius(size: VisualSize) -> f32 {
        match size {
            VisualSize::Small => 14.0,
            VisualSize::Medium => 20.0,
            VisualSize::Large => 28.0,
        }
    }

    fn transition_dimensions(size: VisualSize) -> Vec2 {
        match size {
            VisualSize::Small => Vec2::new(10.0, 18.0),
            VisualSize::Medium => Vec2::new(12.0, 28.0),
            VisualSize::Large => Vec2::new(16.0, 38.0),
        }
    }

    fn snapped_world(&self, world: [f32; 2]) -> [f32; 2] {
        if self.net.ui.snap_to_grid {
            self.snap_point_to_grid(world)
        } else {
            world
        }
    }

    fn label_offset(pos: LabelPosition, scale: f32) -> Vec2 {
        match pos {
            LabelPosition::Top => Vec2::new(0.0, -24.0 * scale),
            LabelPosition::Bottom => Vec2::new(0.0, 24.0 * scale),
            LabelPosition::Left => Vec2::new(-28.0 * scale, 0.0),
            LabelPosition::Right => Vec2::new(28.0 * scale, 0.0),
            LabelPosition::Center => Vec2::ZERO,
        }
    }

    fn place_label_offset(pos: LabelPosition, radius: f32, scale: f32) -> Vec2 {
        if pos == LabelPosition::Center {
            return Vec2::ZERO;
        }
        let distance = radius + 10.0 * scale;
        match pos {
            LabelPosition::Top => Vec2::new(0.0, -distance),
            LabelPosition::Bottom => Vec2::new(0.0, distance),
            LabelPosition::Left => Vec2::new(-distance, 0.0),
            LabelPosition::Right => Vec2::new(distance, 0.0),
            LabelPosition::Center => Vec2::ZERO,
        }
    }

    fn keep_label_inside(rect: Rect, center: Pos2, mut offset: Vec2) -> Vec2 {
        let candidate = center + offset;
        let margin = 8.0;
        if candidate.y > rect.bottom() - margin {
            offset.y = -offset.y.abs();
        } else if candidate.y < rect.top() + margin {
            offset.y = offset.y.abs();
        }
        if candidate.x > rect.right() - margin {
            offset.x = -offset.x.abs();
        } else if candidate.x < rect.left() + margin {
            offset.x = offset.x.abs();
        }
        offset
    }

    fn rect_border_point(rect: Rect, dir: Vec2) -> Pos2 {
        let center = rect.center();
        let nx = if dir.x.abs() < f32::EPSILON { 0.0 } else { dir.x };
        let ny = if dir.y.abs() < f32::EPSILON { 0.0 } else { dir.y };
        let half_w = rect.width() * 0.5;
        let half_h = rect.height() * 0.5;
        let tx = if nx.abs() < f32::EPSILON { f32::INFINITY } else { half_w / nx.abs() };
        let ty = if ny.abs() < f32::EPSILON { f32::INFINITY } else { half_h / ny.abs() };
        let t = tx.min(ty);
        center + Vec2::new(nx * t, ny * t)
    }

    fn world_to_screen(&self, rect: Rect, p: [f32; 2]) -> Pos2 {
        Pos2::new(
            rect.left() + self.canvas.pan.x + p[0] * self.canvas.zoom,
            rect.top() + self.canvas.pan.y + p[1] * self.canvas.zoom,
        )
    }

    fn approx_text_rect(center: Pos2, text: &str, zoom: f32) -> Rect {
        let scale = zoom.clamp(0.75, 2.0);
        let width = (text.chars().count().max(1) as f32 * 7.0 * scale).max(24.0);
        let height = 16.0 * scale;
        Rect::from_center_size(center, Vec2::new(width, height))
    }

    fn screen_to_world(&self, rect: Rect, p: Pos2) -> [f32; 2] {
        [
            (p.x - rect.left() - self.canvas.pan.x) / self.canvas.zoom,
            (p.y - rect.top() - self.canvas.pan.y) / self.canvas.zoom,
        ]
    }

    fn node_at(&self, rect: Rect, pos: Pos2) -> Option<NodeRef> {
        for place in &self.net.places {
            let center = self.world_to_screen(rect, place.pos);
            if center.distance(pos) <= Self::place_radius(place.size) * self.canvas.zoom {
                return Some(NodeRef::Place(place.id));
            }
        }
        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let r = Rect::from_min_size(p, Self::transition_dimensions(tr.size) * self.canvas.zoom);
            if r.contains(pos) {
                return Some(NodeRef::Transition(tr.id));
            }
        }
        for place in &self.net.places {
            let center = self.world_to_screen(rect, place.pos);
            let radius = Self::place_radius(place.size) * self.canvas.zoom;
            let name_offset = Self::keep_label_inside(
                rect,
                center,
                Self::place_label_offset(place.text_position, radius, self.canvas.zoom),
            );
            let label_center = center + name_offset;
            let label_rect = Self::approx_text_rect(label_center, &place.name, self.canvas.zoom);
            if label_rect.contains(pos) {
                return Some(NodeRef::Place(place.id));
            }
        }
        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let dims = Self::transition_dimensions(tr.size) * self.canvas.zoom;
            let r = Rect::from_min_size(p, dims);
            let label_center = r.center() + Self::label_offset(tr.label_position, self.canvas.zoom);
            let label_rect = Self::approx_text_rect(label_center, &tr.name, self.canvas.zoom);
            if label_rect.contains(pos) {
                return Some(NodeRef::Transition(tr.id));
            }
        }
        None
    }

    fn text_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        self.text_blocks
            .iter()
            .rev()
            .find(|item| {
                let center = self.world_to_screen(rect, item.pos);
                Self::approx_text_rect(center, &item.text, self.canvas.zoom).contains(pos)
            })
            .map(|item| item.id)
    }

    fn text_idx_by_id(&self, id: u64) -> Option<usize> {
        self.text_blocks.iter().position(|item| item.id == id)
    }

    fn frame_idx_by_id(&self, id: u64) -> Option<usize> {
        self.decorative_frames.iter().position(|item| item.id == id)
    }
    fn frame_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        self.decorative_frames
            .iter()
            .rev()
            .find(|frame| {
                let min = self.world_to_screen(rect, frame.pos);
                let size =
                    Vec2::new(frame.width.max(Self::FRAME_MIN_SIDE), frame.height.max(Self::FRAME_MIN_SIDE))
                        * self.canvas.zoom;
                let r = Rect::from_min_size(min, size);
                let tolerance = (6.0 * self.canvas.zoom).max(4.0);
                r.expand(tolerance).contains(pos) && !r.shrink(tolerance).contains(pos)
            })
            .map(|frame| frame.id)
    }

    fn frame_from_drag(start: [f32; 2], current: [f32; 2]) -> ([f32; 2], f32, f32) {
        let min_x = start[0].min(current[0]);
        let min_y = start[1].min(current[1]);
        let width = (current[0] - start[0]).abs();
        let height = (current[1] - start[1]).abs();
        ([min_x, min_y], width, height)
    }

    fn frame_resize_handle_rect(&self, rect: Rect, frame: &CanvasFrame) -> Rect {
        let min = self.world_to_screen(rect, frame.pos);
        let width = frame.width.max(Self::FRAME_MIN_SIDE) * self.canvas.zoom;
        let height = frame.height.max(Self::FRAME_MIN_SIDE) * self.canvas.zoom;
        let handle = Self::FRAME_RESIZE_HANDLE_PX;
        let center = Pos2::new(min.x + width, min.y + height);
        Rect::from_center_size(center, Vec2::splat(handle))
    }
    fn clear_selection(&mut self) {
        self.canvas.selected_place = None;
        self.canvas.selected_transition = None;
        self.canvas.selected_places.clear();
        self.canvas.selected_transitions.clear();
        self.canvas.selected_arc = None;
        self.canvas.selected_arcs.clear();
        self.canvas.selected_text = None;
        self.canvas.selected_frame = None;
        self.canvas.frame_draw_start_world = None;
        self.canvas.frame_draw_current_world = None;
        self.canvas.frame_resize_id = None;
    }

    fn push_undo_snapshot(&mut self) {
        self.undo_stack.push(UndoSnapshot {
            net: self.net.clone(),
            text_blocks: self.text_blocks.clone(),
            next_text_id: self.next_text_id,
            decorative_frames: self.decorative_frames.clone(),
            next_frame_id: self.next_frame_id,
        });
        // Keep memory bounded.
        if self.undo_stack.len() > 64 {
            self.undo_stack.remove(0);
        }
    }

    fn undo_last_action(&mut self) {
        let Some(state) = self.undo_stack.pop() else {
            return;
        };
        self.net = state.net;
        self.text_blocks = state.text_blocks;
        self.next_text_id = state.next_text_id;
        self.decorative_frames = state.decorative_frames;
        self.next_frame_id = state.next_frame_id;
        self.clear_selection();
    }

    fn delete_selected(&mut self) {
        if let Some(text_id) = self.canvas.selected_text.take() {
            self.push_undo_snapshot();
            self.text_blocks.retain(|item| item.id != text_id);
            return;
        }
        let mut arc_ids = self.canvas.selected_arcs.clone();
        if let Some(arc_id) = self.canvas.selected_arc.take() {
            arc_ids.push(arc_id);
        }
        arc_ids.sort_unstable();
        arc_ids.dedup();
        if !arc_ids.is_empty() {
            self.canvas.selected_arcs.clear();
            self.push_undo_snapshot();
            self.net.arcs.retain(|a| !arc_ids.contains(&a.id));
            self.net.inhibitor_arcs.retain(|a| !arc_ids.contains(&a.id));
            self.net.rebuild_matrices_from_arcs();
            return;
        }
        if let Some(frame_id) = self.canvas.selected_frame.take() {
            self.push_undo_snapshot();
            self.decorative_frames.retain(|item| item.id != frame_id);
            return;
        }

        let mut place_ids = self.canvas.selected_places.clone();
        let mut transition_ids = self.canvas.selected_transitions.clone();
        if let Some(id) = self.canvas.selected_place {
            place_ids.push(id);
        }
        if let Some(id) = self.canvas.selected_transition {
            transition_ids.push(id);
        }
        place_ids.sort_unstable();
        place_ids.dedup();
        transition_ids.sort_unstable();
        transition_ids.dedup();

        if !place_ids.is_empty() || !transition_ids.is_empty() {
            self.push_undo_snapshot();
            self.net.places.retain(|p| !place_ids.contains(&p.id));
            self.net.transitions.retain(|t| !transition_ids.contains(&t.id));
            self.net.set_counts(self.net.places.len(), self.net.transitions.len());
            self.clear_selection();
        }
    }

    fn collect_selected_place_ids(&self) -> Vec<u64> {
        let mut place_ids = self.canvas.selected_places.clone();
        if let Some(id) = self.canvas.selected_place {
            place_ids.push(id);
        }
        place_ids.sort_unstable();
        place_ids.dedup();
        place_ids
    }

    fn collect_selected_transition_ids(&self) -> Vec<u64> {
        let mut transition_ids = self.canvas.selected_transitions.clone();
        if let Some(id) = self.canvas.selected_transition {
            transition_ids.push(id);
        }
        transition_ids.sort_unstable();
        transition_ids.dedup();
        transition_ids
    }

    fn collect_selected_arc_ids(&self) -> Vec<u64> {
        let mut arc_ids = self.canvas.selected_arcs.clone();
        if let Some(id) = self.canvas.selected_arc {
            arc_ids.push(id);
        }
        arc_ids.sort_unstable();
        arc_ids.dedup();
        arc_ids
    }

    fn ensure_unique_place_name(&self, desired: &str, exclude_id: u64) -> String {
        let base = desired.trim();
        if base.is_empty() {
            return String::new();
        }
        let mut candidate = base.to_string();
        let mut n = 2u32;
        while self
            .net
            .places
            .iter()
            .any(|p| p.id != exclude_id && p.name.trim() == candidate.as_str())
        {
            candidate = format!("{base} ({n})");
            n = n.saturating_add(1);
        }
        candidate
    }

    fn ensure_unique_transition_name(&self, desired: &str, exclude_id: u64) -> String {
        let base = desired.trim();
        if base.is_empty() {
            return String::new();
        }
        let mut candidate = base.to_string();
        let mut n = 2u32;
        while self
            .net
            .transitions
            .iter()
            .any(|t| t.id != exclude_id && t.name.trim() == candidate.as_str())
        {
            candidate = format!("{base} ({n})");
            n = n.saturating_add(1);
        }
        candidate
    }

    fn copy_selected_objects(&mut self) {
        let mut place_ids = self.collect_selected_place_ids();
        let mut transition_ids = self.collect_selected_transition_ids();
        let text_ids: Vec<u64> = self.canvas.selected_text.into_iter().collect();

        // Fallback: if nothing is selected on canvas, allow copying currently opened properties target.
        if place_ids.is_empty() && transition_ids.is_empty() && text_ids.is_empty() {
            if self.show_place_props {
                if let Some(pid) = self.place_props_id {
                    place_ids.push(pid);
                }
            } else if self.show_transition_props {
                if let Some(tid) = self.transition_props_id {
                    transition_ids.push(tid);
                }
            }
        }

        if place_ids.is_empty() && transition_ids.is_empty() && text_ids.is_empty() {
            self.status_hint = Some("РќРµС‡РµРіРѕ РєРѕРїРёСЂРѕРІР°С‚СЊ: РЅРµС‚ РІС‹РґРµР»РµРЅРёСЏ".to_string());
            return;
        }

        let place_set: HashSet<u64> = place_ids.iter().copied().collect();
        let transition_set: HashSet<u64> = transition_ids.iter().copied().collect();
        let pmap = self.net.place_index_map();
        let tmap = self.net.transition_index_map();

        let mut copied_places = Vec::new();
        for pid in &place_ids {
            let Some(&idx) = pmap.get(pid) else {
                continue;
            };
            copied_places.push(CopiedPlace {
                place: self.net.places[idx].clone(),
                m0: self.net.tables.m0.get(idx).copied().unwrap_or(0),
                mo: self.net.tables.mo.get(idx).copied().unwrap_or(None),
                mz: self.net.tables.mz.get(idx).copied().unwrap_or(0.0),
            });
        }

        let mut copied_transitions = Vec::new();
        for tid in &transition_ids {
            let Some(&idx) = tmap.get(tid) else {
                continue;
            };
            copied_transitions.push(CopiedTransition {
                transition: self.net.transitions[idx].clone(),
                mpr: self.net.tables.mpr.get(idx).copied().unwrap_or(0),
            });
        }

        let mut copied_texts = Vec::new();
        for text_id in &text_ids {
            if let Some(idx) = self.text_idx_by_id(*text_id) {
                copied_texts.push(CopiedTextBlock {
                    pos: self.text_blocks[idx].pos,
                    text: self.text_blocks[idx].text.clone(),
                });
            }
        }

        let mut copied_arcs = Vec::new();
        let in_sel = |n: NodeRef| match n {
            NodeRef::Place(id) => place_set.contains(&id),
            NodeRef::Transition(id) => transition_set.contains(&id),
        };

        for arc in &self.net.arcs {
            if in_sel(arc.from) && in_sel(arc.to) {
                copied_arcs.push(CopiedArc {
                    from: arc.from,
                    to: arc.to,
                    weight: arc.weight,
                    color: arc.color,
                    visible: arc.visible,
                });
            }
        }

        let mut copied_inhibitors = Vec::new();
        for inh in &self.net.inhibitor_arcs {
            if place_set.contains(&inh.place_id) && transition_set.contains(&inh.transition_id) {
                copied_inhibitors.push(CopiedInhibitorArc {
                    place_id: inh.place_id,
                    transition_id: inh.transition_id,
                    threshold: inh.threshold,
                    color: inh.color,
                    visible: inh.visible,
                });
            }
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        for p in &copied_places {
            min_x = min_x.min(p.place.pos[0]);
            min_y = min_y.min(p.place.pos[1]);
        }
        for t in &copied_transitions {
            min_x = min_x.min(t.transition.pos[0]);
            min_y = min_y.min(t.transition.pos[1]);
        }
        for t in &copied_texts {
            min_x = min_x.min(t.pos[0]);
            min_y = min_y.min(t.pos[1]);
        }
        if !min_x.is_finite() || !min_y.is_finite() {
            min_x = self.canvas.cursor_world[0];
            min_y = self.canvas.cursor_world[1];
        }

        let copied_count =
            place_ids.len() + transition_ids.len() + text_ids.len() + copied_arcs.len() + copied_inhibitors.len();
        let clip = CopyBuffer {
            origin: [min_x, min_y],
            places: copied_places,
            transitions: copied_transitions,
            arcs: copied_arcs,
            inhibitors: copied_inhibitors,
            texts: copied_texts,
        };
        self.write_copy_buffer_to_system_clipboard(&clip);
        self.clipboard = Some(clip);
        // Keep first paste visibly offset from original selection.
        self.paste_serial = 1;
        self.status_hint = Some(format!("РЎРєРѕРїРёСЂРѕРІР°РЅРѕ РѕР±СЉРµРєС‚РѕРІ: {copied_count}"));
    }

    fn paste_copied_objects(&mut self) {
        if let Some(ext) = self.read_copy_buffer_from_system_clipboard() {
            self.clipboard = Some(ext);
        }
        let Some(buf) = self.clipboard.clone() else {
            self.status_hint = Some("Р‘СѓС„РµСЂ РїСѓСЃС‚".to_string());
            return;
        };
        if buf.places.is_empty() && buf.transitions.is_empty() && buf.texts.is_empty() {
            self.status_hint = Some("Р‘СѓС„РµСЂ РїСѓСЃС‚".to_string());
            return;
        }
        self.push_undo_snapshot();

        let base = if self.canvas.cursor_valid {
            self.snapped_world(self.canvas.cursor_world)
        } else {
            self.snapped_world(buf.origin)
        };
        let step = self.grid_step_world();
        let delta = self.paste_serial as f32 * step;
        let offset = [delta, delta];

        let mut place_map = HashMap::<u64, u64>::new();
        let mut transition_map = HashMap::<u64, u64>::new();

        for cp in &buf.places {
            let rel = [cp.place.pos[0] - buf.origin[0], cp.place.pos[1] - buf.origin[1]];
            let pos = self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let before_max = self.net.places.iter().map(|p| p.id).max().unwrap_or(0);
            self.net.add_place(pos);
            let new_id = self.net.places.iter().map(|p| p.id).max().unwrap_or(0);
            if new_id <= before_max {
                continue;
            }
            place_map.insert(cp.place.id, new_id);

            if let Some(idx) = self.place_idx_by_id(new_id) {
                let mut place = cp.place.clone();
                place.id = new_id;
                place.pos = pos;
                self.net.places[idx] = place;

                self.net.tables.m0[idx] = cp.m0;
                self.net.tables.mo[idx] = cp.mo;
                self.net.tables.mz[idx] = cp.mz;

                if Self::parse_place_auto_index(&cp.place.name).is_some() || cp.place.name.trim().is_empty() {
                    self.net.places[idx].name.clear();
                    self.assign_auto_name_for_place(new_id);
                } else {
                    let desired = self.net.places[idx].name.clone();
                    self.net.places[idx].name = self.ensure_unique_place_name(&desired, new_id);
                }
            }
        }

        for ct in &buf.transitions {
            let rel = [
                ct.transition.pos[0] - buf.origin[0],
                ct.transition.pos[1] - buf.origin[1],
            ];
            let pos = self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let before_max = self.net.transitions.iter().map(|t| t.id).max().unwrap_or(0);
            self.net.add_transition(pos);
            let new_id = self.net.transitions.iter().map(|t| t.id).max().unwrap_or(0);
            if new_id <= before_max {
                continue;
            }
            transition_map.insert(ct.transition.id, new_id);

            if let Some(idx) = self.transition_idx_by_id(new_id) {
                let mut tr = ct.transition.clone();
                tr.id = new_id;
                tr.pos = pos;
                self.net.transitions[idx] = tr;
                self.net.tables.mpr[idx] = ct.mpr;

                if Self::parse_transition_auto_index(&ct.transition.name).is_some() || ct.transition.name.trim().is_empty() {
                    self.net.transitions[idx].name.clear();
                    self.assign_auto_name_for_transition(new_id);
                } else {
                    let desired = self.net.transitions[idx].name.clone();
                    self.net.transitions[idx].name = self.ensure_unique_transition_name(&desired, new_id);
                }
            }
        }

        let mut new_text_ids = Vec::new();
        for tt in &buf.texts {
            let rel = [tt.pos[0] - buf.origin[0], tt.pos[1] - buf.origin[1]];
            let pos = self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let id = self.next_text_id;
            self.next_text_id = self.next_text_id.saturating_add(1);
            self.text_blocks.push(CanvasTextBlock {
                id,
                pos,
                text: tt.text.clone(),
            });
            new_text_ids.push(id);
        }

        for arc in &buf.arcs {
            let remap = |n: NodeRef| -> Option<NodeRef> {
                match n {
                    NodeRef::Place(id) => place_map.get(&id).copied().map(NodeRef::Place),
                    NodeRef::Transition(id) => transition_map.get(&id).copied().map(NodeRef::Transition),
                }
            };
            let (Some(from), Some(to)) = (remap(arc.from), remap(arc.to)) else {
                continue;
            };
            self.net.add_arc(from, to, arc.weight);
            if let Some(last) = self.net.arcs.last_mut() {
                last.color = arc.color;
                last.visible = arc.visible;
            }
        }
        for inh in &buf.inhibitors {
            let (Some(&pid), Some(&tid)) = (place_map.get(&inh.place_id), transition_map.get(&inh.transition_id)) else {
                continue;
            };
            self.net.add_inhibitor_arc(pid, tid, inh.threshold);
            if let Some(last) = self.net.inhibitor_arcs.last_mut() {
                last.color = inh.color;
                last.visible = inh.visible;
            }
        }

        self.clear_selection();
        self.canvas.selected_places = place_map.values().copied().collect();
        self.canvas.selected_transitions = transition_map.values().copied().collect();
        self.canvas.selected_text = new_text_ids.last().copied();

        self.paste_serial = self.paste_serial.saturating_add(1);
        let pasted_count = place_map.len() + transition_map.len() + new_text_ids.len();
        self.status_hint = Some(format!("Р’СЃС‚Р°РІР»РµРЅРѕ РѕР±СЉРµРєС‚РѕРІ: {pasted_count}"));
    }

    fn arc_screen_endpoints(&self, rect: Rect, arc: &crate::model::Arc) -> Option<(Pos2, Pos2)> {
        let (from_center, from_radius, from_rect, to_center, to_radius, to_rect) = match (arc.from, arc.to) {
            (NodeRef::Place(p), NodeRef::Transition(t)) => {
                let (Some(pi), Some(ti)) = (self.place_idx_by_id(p), self.transition_idx_by_id(t)) else {
                    return None;
                };
                let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                let t_rect = Rect::from_min_size(t_min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom);
                (p_center, Some(p_radius), None, t_rect.center(), None, Some(t_rect))
            }
            (NodeRef::Transition(t), NodeRef::Place(p)) => {
                let (Some(pi), Some(ti)) = (self.place_idx_by_id(p), self.transition_idx_by_id(t)) else {
                    return None;
                };
                let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                let t_rect = Rect::from_min_size(t_min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom);
                let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                (t_rect.center(), None, Some(t_rect), p_center, Some(p_radius), None)
            }
            _ => return None,
        };

        let mut from = from_center;
        let mut to = to_center;
        let delta = to_center - from_center;
        let dir = if delta.length_sq() > 0.0 { delta.normalized() } else { Vec2::X };

        if let Some(radius) = from_radius {
            from += dir * radius;
        } else if let Some(r) = from_rect {
            from = Self::rect_border_point(r, dir);
        }

        if let Some(radius) = to_radius {
            to -= dir * radius;
        } else if let Some(r) = to_rect {
            to = Self::rect_border_point(r, -dir);
        }

        Some((from, to))
    }

    fn inhibitor_screen_endpoints(&self, rect: Rect, inh: &crate::model::InhibitorArc) -> Option<(Pos2, Pos2)> {
        let (Some(pi), Some(ti)) = (
            self.place_idx_by_id(inh.place_id),
            self.transition_idx_by_id(inh.transition_id),
        ) else {
            return None;
        };

        let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
        let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
        let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
        let t_rect = Rect::from_min_size(t_min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom);
        let t_center = t_rect.center();
        let delta = t_center - p_center;
        let dir = if delta.length_sq() > 0.0 { delta.normalized() } else { Vec2::X };
        let from = p_center + dir * p_radius;
        let to = Self::rect_border_point(t_rect, -dir);

        Some((from, to))
    }

    fn segment_distance_to_point(pos: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        if ab.length_sq() <= f32::EPSILON {
            return pos.distance(a);
        }
        let t = ((pos - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        let proj = a + ab * t;
        proj.distance(pos)
    }

    fn arc_fully_inside_rect(sel: Rect, from: Pos2, to: Pos2) -> bool {
        if !sel.contains(from) || !sel.contains(to) {
            return false;
        }

        let arrow = to - from;
        if arrow.length_sq() <= f32::EPSILON {
            return true;
        }

        let dir = arrow.normalized();
        let left = to - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
        let right = to - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
        sel.contains(left) && sel.contains(right)
    }

    fn arc_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        let mut best_id = None;
        // Keep arc hit-test tighter so node clicks near edges still select the node.
        let mut best_dist = 12.0_f32;

        for arc in &self.net.arcs {
            if !self.arc_visible_by_mode(arc.color, arc.visible) {
                continue;
            }
            let Some((a, b)) = self.arc_screen_endpoints(rect, arc) else {
                continue;
            };
            let dist = Self::segment_distance_to_point(pos, a, b);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(arc.id);
            }
        }

        for inh in &self.net.inhibitor_arcs {
            if !self.arc_visible_by_mode(inh.color, inh.visible) {
                continue;
            }
            let Some((a, b)) = self.inhibitor_screen_endpoints(rect, inh) else {
                continue;
            };
            let dist = Self::segment_distance_to_point(pos, a, b);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(inh.id);
            }
        }

        best_id
    }


    fn draw_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Р¤Р°Р№Р»", |ui| {
                    if ui.button("РќРѕРІС‹Р№ (Ctrl+N)").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("РћС‚РєСЂС‹С‚СЊ (Ctrl+O)").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.menu_button("РРјРїРѕСЂС‚", |ui| {
                        ui.label("РРјРїРѕСЂС‚ PeSim: TODO");
                    });
                    ui.menu_button("Р­РєСЃРїРѕСЂС‚", |ui| {
                        if ui.button("Р­РєСЃРїРѕСЂС‚РёСЂРѕРІР°С‚СЊ РІ NetStar").clicked() {
                            self.export_netstar_file();
                            ui.close_menu();
                        }
                    });
                    if ui.button("РЎРѕС…СЂР°РЅРёС‚СЊ (Ctrl+S)").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("РЎРѕС…СЂР°РЅРёС‚СЊ РєР°Рє").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    if ui.button("Р’С‹С…РѕРґ").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("РћРїС†РёРё", |ui| {
                    ui.menu_button("РЇР·С‹Рє", |ui| {
                        ui.radio_value(&mut self.net.ui.language, Language::Ru, "RU");
                        ui.radio_value(&mut self.net.ui.language, Language::En, "EN");
                    });
                    ui.checkbox(&mut self.net.ui.hide_grid, "РЎРєСЂС‹С‚СЊ СЃРµС‚РєСѓ");
                    ui.checkbox(&mut self.net.ui.snap_to_grid, "РџСЂРёРІСЏР·РєР° Рє СЃРµС‚РєРµ");
                    ui.checkbox(&mut self.net.ui.colored_petri_nets, "Р¦РІРµС‚РЅС‹Рµ СЃРµС‚Рё РџРµС‚СЂРё");
                    ui.menu_button("РЎР±РѕСЂ СЃС‚Р°С‚РёСЃС‚РёРєРё", |ui| {
                        ui.checkbox(&mut self.net.ui.marker_count_stats, "РЎС‚Р°С‚РёСЃС‚РёРєР° РјР°СЂРєРµСЂРѕРІ");
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Р Р°Р·СЂР°Р±РѕС‚РєР°").clicked() {
                            self.show_help_development = true;
                            ui.close_menu();
                        }
                        if ui.button("РџРѕРјРѕС‰СЊ РїРѕ СѓРїСЂР°РІР»РµРЅРёСЋ").clicked() {
                            self.show_help_controls = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("РћРєРЅРѕ", |ui| {
                    if ui.button("РљР°СЃРєР°Рґ").clicked() {
                        self.layout_mode = LayoutMode::Cascade;
                    }
                    if ui.button("РџР»РёС‚РєР° РїРѕ РіРѕСЂРёР·РѕРЅС‚Р°Р»Рё").clicked() {
                        self.layout_mode = LayoutMode::TileHorizontal;
                    }
                    if ui.button("РџР»РёС‚РєР° РїРѕ РІРµСЂС‚РёРєР°Р»Рё").clicked() {
                        self.layout_mode = LayoutMode::TileVertical;
                    }
                    if ui.button("РЎРІРµСЂРЅСѓС‚СЊ РІСЃРµ").clicked() {
                        self.layout_mode = LayoutMode::Minimized;
                    }
                    if ui.button("РЈРїРѕСЂСЏРґРѕС‡РёС‚СЊ РІСЃРµ").clicked() {
                        self.layout_mode = LayoutMode::TileVertical;
                        self.show_graph_view = true;
                    }
                });

                if ui.button("РџР°СЂР°РјРµС‚СЂС‹ СЃРёРјСѓР»СЏС†РёРё").clicked() {
                    self.reset_sim_stop_controls();
                    self.show_sim_params = true;
                }
                if ui.button("РЎС‚СЂСѓРєС‚СѓСЂР° СЃРµС‚Рё").clicked() {
                    self.show_table_view = !self.show_table_view;
                    if !self.show_table_view {
                        self.table_fullscreen = false;
                    }
                }
                if ui
                    .button(self.tr("Р РµР·СѓР»СЊС‚Р°С‚С‹ РёРјРёС‚Р°С†РёРё", "Simulation Results"))
                    .clicked()
                {
                    self.show_results = self.sim_result.is_some();
                }
                if ui.button("Proof").clicked() && self.sim_result.is_some() {
                    self.show_proof = true;
                }
                if ui.button(self.tr("Р РµР¶РёРј РѕС‚Р»Р°РґРєРё", "Debug Mode")).clicked()
                    && self.sim_result.is_some()
                {
                    self.show_debug = true;
                }
                if ui.button("ATF").clicked() {
                    self.show_atf = true;
                }
            });
        });
    }


    fn draw_place_props_window(
        &mut self,
        ctx: &egui::Context,
        place_id: u64,
        title: String,
    ) -> bool {
        let Some(place_idx) = self.place_idx_by_id(place_id) else {
            return false;
        };
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };
        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::new("place_props_window"))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("ID: P{}", place_id));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(t("Р§РёСЃР»Рѕ РјР°СЂРєРµСЂРѕРІ", "Markers"));
                    ui.add(egui::DragValue::new(&mut self.net.tables.m0[place_idx]).range(0..=u32::MAX));
                });

                let mut cap = self.net.tables.mo[place_idx].unwrap_or(0);
                ui.horizontal(|ui| {
                    ui.label(t("РњР°РєСЃ. РµРјРєРѕСЃС‚СЊ (0 = Р±РµР· РѕРіСЂР°РЅРёС‡РµРЅРёР№)", "Capacity (0 = unlimited)"));
                    if ui.add(egui::DragValue::new(&mut cap).range(0..=u32::MAX)).changed() {
                        self.net.tables.mo[place_idx] = if cap == 0 { None } else { Some(cap) };
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(t("Р’СЂРµРјСЏ Р·Р°РґРµСЂР¶РєРё (СЃРµРє)", "Delay (sec)"));
                    ui.add(
                        egui::DragValue::new(&mut self.net.tables.mz[place_idx])
                            .speed(0.1)
                            .range(0.0..=10_000.0),
                    );
                });

                ui.separator();
                ui.label(t("Р Р°Р·РјРµСЂ РїРѕР·РёС†РёРё", "Place size"));
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.net.places[place_idx].size, VisualSize::Small, t("РњР°Р»С‹Р№", "Small"));
                    ui.radio_value(&mut self.net.places[place_idx].size, VisualSize::Medium, t("РЎСЂРµРґРЅРёР№", "Medium"));
                    ui.radio_value(&mut self.net.places[place_idx].size, VisualSize::Large, t("Р‘РѕР»СЊС€РѕР№", "Large"));
                });

                egui::ComboBox::from_label(t("РџРѕР»РѕР¶РµРЅРёРµ РјРµС‚РєРё", "Marker label position"))
                    .selected_text(Self::label_pos_text(self.net.places[place_idx].marker_label_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Top, t("Р’РІРµСЂС…Сѓ", "Top"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Bottom, t("Р’РЅРёР·Сѓ", "Bottom"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Left, t("РЎР»РµРІР°", "Left"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Right, t("РЎРїСЂР°РІР°", "Right"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Center, t("РџРѕ С†РµРЅС‚СЂСѓ", "Center"));
                    });

                egui::ComboBox::from_label(t("РџРѕР»РѕР¶РµРЅРёРµ С‚РµРєСЃС‚Р°", "Text position"))
                    .selected_text(Self::label_pos_text(self.net.places[place_idx].text_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Top, t("Р’РІРµСЂС…Сѓ", "Top"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Bottom, t("Р’РЅРёР·Сѓ", "Bottom"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Left, t("РЎР»РµРІР°", "Left"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Right, t("РЎРїСЂР°РІР°", "Right"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Center, t("РџРѕ С†РµРЅС‚СЂСѓ", "Center"));
                    });

                egui::ComboBox::from_label(t("Р¦РІРµС‚", "Color"))
                    .selected_text(Self::node_color_text(self.net.places[place_idx].color, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Default, t("РџРѕ СѓРјРѕР»С‡Р°РЅРёСЋ", "Default"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Blue, t("РЎРёРЅРёР№", "Blue"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Red, t("РљСЂР°СЃРЅС‹Р№", "Red"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Green, t("Р—РµР»РµРЅС‹Р№", "Green"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Yellow, t("Р–РµР»С‚С‹Р№", "Yellow"));
                    });

                ui.separator();
                ui.checkbox(
                    &mut self.net.places[place_idx].marker_color_on_pass,
                    t(
                        "РР·РјРµРЅСЏС‚СЊ С†РІРµС‚ РјР°СЂРєРµСЂР° РїСЂРё РїСЂРѕС…РѕР¶РґРµРЅРёРё С‡РµСЂРµР· РїРѕР·РёС†РёСЋ",
                        "Change marker color when token passes this place",
                    ),
                );
                ui.checkbox(
                    &mut self.net.places[place_idx].input_module,
                    t("РћРїСЂРµРґРµР»РёС‚СЊ РїРѕР·РёС†РёСЋ РєР°Рє РІС…РѕРґ РјРѕРґСѓР»СЏ", "Define place as module input"),
                );
                if self.net.places[place_idx].input_module {
                    ui.horizontal(|ui| {
                        ui.label(t("РќРѕРјРµСЂ РІС…РѕРґР°", "Input number"));
                        ui.add(
                            egui::DragValue::new(&mut self.net.places[place_idx].input_number)
                                .range(1..=u32::MAX),
                        );
                    });
                    ui.label(t("РћРїРёСЃР°РЅРёРµ РІС…РѕРґР°", "Input description"));
                    ui.text_edit_singleline(&mut self.net.places[place_idx].input_description);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(t("РЎС‚РѕС…Р°СЃС‚РёС‡РµСЃС‚РёРµ РїСЂРѕС†РµСЃСЃС‹", "Stochastic processes"));
                    let stats_enabled = self.net.ui.marker_count_stats;
                    if ui
                        .add_enabled(stats_enabled, egui::Button::new(t("РЎР±РѕСЂ СЃС‚Р°С‚РёСЃС‚РёРєРё", "Collect statistics")))
                        .clicked()
                    {
                        self.place_stats_dialog_place_id = Some(place_id);
                        self.place_stats_dialog_backup =
                            Some((place_id, self.net.places[place_idx].stats));
                    }
                });
                egui::ComboBox::from_label(t("Р Р°СЃРїСЂРµРґРµР»РµРЅРёРµ", "Distribution"))
                    .selected_text(Self::stochastic_text(&self.net.places[place_idx].stochastic, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::None,
                            Self::stochastic_text(&StochasticDistribution::None, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Uniform { min: 0.0, max: 1.0 },
                            Self::stochastic_text(&StochasticDistribution::Uniform { min: 0.0, max: 1.0 }, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Normal { mean: 1.0, std_dev: 0.2 },
                            Self::stochastic_text(&StochasticDistribution::Normal { mean: 1.0, std_dev: 0.2 }, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Exponential { lambda: 1.0 },
                            Self::stochastic_text(&StochasticDistribution::Exponential { lambda: 1.0 }, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Poisson { lambda: 1.0 },
                            Self::stochastic_text(&StochasticDistribution::Poisson { lambda: 1.0 }, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::CustomValue { value: 1.0 },
                            Self::stochastic_text(&StochasticDistribution::CustomValue { value: 1.0 }, is_ru),
                        );
                    });

                match &mut self.net.places[place_idx].stochastic {
                    StochasticDistribution::None => {}
                    StochasticDistribution::Uniform { min, max } => {
                        ui.horizontal(|ui| {
                            ui.label(t("min", "min"));
                            ui.add(egui::DragValue::new(min).speed(0.1).range(0.0..=10_000.0));
                            ui.label(t("max", "max"));
                            ui.add(egui::DragValue::new(max).speed(0.1).range(0.0..=10_000.0));
                        });
                    }
                    StochasticDistribution::Normal { mean, std_dev } => {
                        ui.horizontal(|ui| {
                            ui.label(t("mean", "mean"));
                            ui.add(egui::DragValue::new(mean).speed(0.1).range(0.0..=10_000.0));
                            ui.label(t("std", "std"));
                            ui.add(egui::DragValue::new(std_dev).speed(0.1).range(0.0..=10_000.0));
                        });
                    }
                    StochasticDistribution::Exponential { lambda } | StochasticDistribution::Poisson { lambda } => {
                        ui.horizontal(|ui| {
                            ui.label(t("lambda", "lambda"));
                            ui.add(egui::DragValue::new(lambda).speed(0.1).range(0.0001..=10_000.0));
                        });
                    }
                    StochasticDistribution::CustomValue { value } => {
                        ui.horizontal(|ui| {
                            ui.label(t("Р—РЅР°С‡РµРЅРёРµ", "Value"));
                            ui.add(egui::DragValue::new(value).speed(0.1).range(0.0..=10_000.0));
                        });
                    }
                }

                ui.separator();
                ui.label(t("РќР°Р·РІР°РЅРёРµ", "Name"));
                ui.text_edit_singleline(&mut self.net.places[place_idx].name);
            });
        open
    }

    fn draw_place_properties(&mut self, ctx: &egui::Context) {
        if !self.show_place_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_place
            .or_else(|| self.canvas.selected_places.last().copied())
        {
            self.place_props_id = Some(id);
        }
        if let Some(place_id) = self.place_props_id {
            let title = self.tr("РЎРІРѕР№СЃС‚РІР° РїРѕР·РёС†РёРё", "Place Properties").to_owned();
            self.show_place_props = self.draw_place_props_window(ctx, place_id, title);
        } else {
            self.show_place_props = false;
        }
    }

    fn draw_transition_props_window(
        &mut self,
        ctx: &egui::Context,
        transition_id: u64,
        title: String,
    ) -> bool {
        let Some(transition_idx) = self.transition_idx_by_id(transition_id) else {
            return false;
        };

        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::new("transition_props_window"))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("ID: T{}", transition_id));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(t("РџСЂРёРѕСЂРёС‚РµС‚", "Priority"));
                    ui.add(egui::DragValue::new(&mut self.net.tables.mpr[transition_idx]));
                });
                ui.horizontal(|ui| {
                    ui.label(t("РЈРіРѕР» РЅР°РєР»РѕРЅР°", "Angle"));
                    ui.add(egui::DragValue::new(&mut self.net.transitions[transition_idx].angle_deg).range(-180..=180));
                });

                ui.label(t("Р Р°Р·РјРµСЂ РїРµСЂРµС…РѕРґР°", "Transition size"));
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.net.transitions[transition_idx].size, VisualSize::Small, t("РњР°Р»С‹Р№", "Small"));
                    ui.radio_value(&mut self.net.transitions[transition_idx].size, VisualSize::Medium, t("РЎСЂРµРґРЅРёР№", "Medium"));
                    ui.radio_value(&mut self.net.transitions[transition_idx].size, VisualSize::Large, t("Р‘РѕР»СЊС€РѕР№", "Large"));
                });

                egui::ComboBox::from_label(t("РџРѕР»РѕР¶РµРЅРёРµ РјРµС‚РєРё", "Label position"))
                    .selected_text(Self::label_pos_text(self.net.transitions[transition_idx].label_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Top, t("Р’РІРµСЂС…Сѓ", "Top"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Bottom, t("Р’РЅРёР·Сѓ", "Bottom"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Left, t("РЎР»РµРІР°", "Left"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Right, t("РЎРїСЂР°РІР°", "Right"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Center, t("РџРѕ С†РµРЅС‚СЂСѓ", "Center"));
                    });

                egui::ComboBox::from_label(t("РџРѕР»РѕР¶РµРЅРёРµ С‚РµРєСЃС‚Р°", "Text position"))
                    .selected_text(Self::label_pos_text(self.net.transitions[transition_idx].text_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Top, t("Р’РІРµСЂС…Сѓ", "Top"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Bottom, t("Р’РЅРёР·Сѓ", "Bottom"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Left, t("РЎР»РµРІР°", "Left"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Right, t("РЎРїСЂР°РІР°", "Right"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Center, t("РџРѕ С†РµРЅС‚СЂСѓ", "Center"));
                    });

                egui::ComboBox::from_label(t("Р¦РІРµС‚", "Color"))
                    .selected_text(Self::node_color_text(self.net.transitions[transition_idx].color, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Default, t("РџРѕ СѓРјРѕР»С‡Р°РЅРёСЋ", "Default"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Blue, t("РЎРёРЅРёР№", "Blue"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Red, t("РљСЂР°СЃРЅС‹Р№", "Red"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Green, t("Р—РµР»РµРЅС‹Р№", "Green"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Yellow, t("Р–РµР»С‚С‹Р№", "Yellow"));
                    });

                ui.separator();
                ui.label(t("РќР°Р·РІР°РЅРёРµ", "Name"));
                ui.text_edit_singleline(&mut self.net.transitions[transition_idx].name);
            });
        open
    }

    fn draw_transition_properties(&mut self, ctx: &egui::Context) {
        if !self.show_transition_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_transition
            .or_else(|| self.canvas.selected_transitions.last().copied())
        {
            self.transition_props_id = Some(id);
        }
        if let Some(transition_id) = self.transition_props_id {
            let title = self
                .tr("РЎРІРѕР№СЃС‚РІР° РїРµСЂРµС…РѕРґР°", "Transition Properties")
                .to_owned();
            self.show_transition_props =
                self.draw_transition_props_window(ctx, transition_id, title);
        } else {
            self.show_transition_props = false;
        }
    }
    fn draw_debug_window(&mut self, ctx: &egui::Context) {
        if !self.show_debug {
            return;
        }
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = self.show_debug;
        egui::Window::new(t("Р РµР¶РёРј РѕС‚Р»Р°РґРєРё", "Debug Mode"))
            .open(&mut open)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.clone() else {
                    ui.label(t("РЎРЅР°С‡Р°Р»Р° Р·Р°РїСѓСЃС‚РёС‚Рµ РёРјРёС‚Р°С†РёСЋ.", "Run simulation first."));
                    return;
                };
                let visible_steps = Self::debug_visible_log_indices(&result);
                let steps = visible_steps.len();
                if steps == 0 {
                    ui.label(t("РџСѓСЃС‚РѕР№ Р¶СѓСЂРЅР°Р».", "Empty log."));
                    return;
                }
                if self.debug_step >= steps {
                    self.debug_step = steps - 1;
                }

                if self.debug_playing {
                    let now = Instant::now();
                    let should_tick = self
                        .last_debug_tick
                        .map(|tick| now.duration_since(tick) >= Duration::from_millis(self.debug_interval_ms))
                        .unwrap_or(true);
                    if should_tick {
                        if self.debug_step + 1 < steps {
                            self.debug_step += 1;
                            self.last_debug_tick = Some(now);
                            ctx.request_repaint_after(Duration::from_millis(16));
                        } else {
                            self.debug_playing = false;
                        }
                    } else {
                        ctx.request_repaint_after(Duration::from_millis(16));
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button("<<").clicked() {
                        self.debug_step = self.debug_step.saturating_sub(1);
                    }
                    if ui.button(if self.debug_playing { t("РџР°СѓР·Р°", "Pause") } else { t("РџСѓСЃРє", "Play") }).clicked() {
                        self.debug_playing = !self.debug_playing;
                        self.last_debug_tick = Some(Instant::now());
                    }
                    if ui.button(">>").clicked() {
                        self.debug_step = (self.debug_step + 1).min(steps - 1);
                    }
                    ui.label(t("РЎРєРѕСЂРѕСЃС‚СЊ (РјСЃ):", "Speed (ms):"));
                    ui.add(egui::DragValue::new(&mut self.debug_interval_ms).range(50..=5_000));
                });

                ui.add(egui::Slider::new(&mut self.debug_step, 0..=steps - 1).text(t("РЁР°Рі", "Step")));
                if let Some(&log_idx) = visible_steps.get(self.debug_step) {
                    if let Some(entry) = result.logs.get(log_idx) {
                    ui.separator();
                    ui.label(format!("t = {:.3}", entry.time));
                    ui.label(format!(
                        "{}: {}",
                        t("РџРµСЂРµС…РѕРґ", "Transition"),
                        entry
                            .fired_transition
                            .map(|i| format!("T{}", i + 1))
                            .unwrap_or_else(|| "-".to_string())
                    ));
                    egui::Grid::new("debug_marking_grid").striped(true).show(ui, |ui| {
                        for (idx, marking) in entry.marking.iter().enumerate() {
                            ui.label(format!("P{}", idx + 1));
                            ui.label(marking.to_string());
                            ui.end_row();
                        }
                    });
                    }
                }
            });
        self.show_debug = open;
    }
    fn draw_proof_window(&mut self, ctx: &egui::Context) {
        if !self.show_proof {
            return;
        }
        let mut open = self.show_proof;
        egui::Window::new("Proof")
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.as_ref() else {
                    ui.label(self.tr("РЎРЅР°С‡Р°Р»Р° Р·Р°РїСѓСЃС‚РёС‚Рµ РёРјРёС‚Р°С†РёСЋ.", "Run simulation first."));
                    return;
                };
                ui.label(self.tr(
                    "Р”РѕРєР°Р·Р°С‚РµР»СЊСЃС‚РІРѕ РїРѕСЃС‚СЂРѕРµРЅРѕ РїРѕ Р¶СѓСЂРЅР°Р»Сѓ СЃРѕСЃС‚РѕСЏРЅРёР№ (trace).",
                    "Proof is generated from simulation trace.",
                ));
                ui.separator();
                egui::Grid::new("proof_grid").striped(true).show(ui, |ui| {
                    ui.label(self.tr("РЁР°Рі", "Step"));
                    ui.label(self.tr("Р’СЂРµРјСЏ", "Time"));
                    ui.label(self.tr("РЎСЂР°Р±РѕС‚Р°Р» РїРµСЂРµС…РѕРґ", "Fired transition"));
                    ui.label(self.tr("РњР°СЂРєРёСЂРѕРІРєР°", "Marking"));
                    ui.end_row();
                    for (step, entry) in result.logs.iter().enumerate() {
                        ui.label(step.to_string());
                        ui.label(format!("{:.3}", entry.time));
                        ui.label(
                            entry
                                .fired_transition
                                .map(|i| format!("T{}", i + 1))
                                .unwrap_or_else(|| "-".to_string()),
                        );
                        ui.label(format!("{:?}", entry.marking));
                        ui.end_row();
                    }
                });
            });
        self.show_proof = open;
    }

    fn draw_atf_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_atf;
        egui::Window::new("ATF")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Р›РµРІР°СЏ РѕР±Р»Р°СЃС‚СЊ");
                        ui.horizontal(|ui| {
                            ui.label("P:");
                            ui.add(egui::DragValue::new(&mut self.atf_selected_place).range(0..=10000));
                            if ui.button("OK").clicked() {
                                self.atf_text = generate_atf(&self.net, self.atf_selected_place.min(self.net.places.len().saturating_sub(1)));
                            }
                        });
                        if ui.button("РЎРіРµРЅРµСЂРёСЂРѕРІР°С‚СЊ ATF").clicked() {
                            self.atf_text = generate_atf(&self.net, self.atf_selected_place.min(self.net.places.len().saturating_sub(1)));
                        }
                        if ui.button("РћС‚РєСЂС‹С‚СЊ ATF С„Р°Р№Р»").clicked() {
                            if let Some(path) = rfd::FileDialog::new().add_filter("ATF", &["atf", "txt"]).pick_file() {
                                match fs::read_to_string(&path) {
                                    Ok(text) => self.atf_text = text,
                                    Err(e) => self.last_error = Some(e.to_string()),
                                }
                            }
                        }
                    });
                    ui.separator();
                    ui.add(
                        egui::TextEdit::multiline(&mut self.atf_text)
                            .desired_rows(30)
                            .desired_width(700.0),
                    );
                });
            });
        self.show_atf = open;
    }

    fn draw_help_development(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_development;
        egui::Window::new("Help: Р Р°Р·СЂР°Р±РѕС‚РєР°")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("РРЅС„РѕСЂРјР°С†РёСЏ Рѕ РїСЂРёР»РѕР¶РµРЅРёРё");
                ui.separator();
                ui.label(egui::RichText::new(format!("Р’РµСЂСЃРёСЏ: {}", env!("CARGO_PKG_VERSION"))).size(20.0));
                ui.label(egui::RichText::new("Р Р°Р·СЂР°Р±РѕС‚С‡РёРє: Р’Р°Р№Р±РєРѕРґ + РІС‹Р»РµС‚С‹ NetStar").size(18.0));
                ui.separator();
                ui.label("Р РµРґР°РєС‚РѕСЂ СЃРµС‚РµР№ РџРµС‚СЂРё СЃ СЃРѕРІРјРµСЃС‚РёРјРѕСЃС‚СЊСЋ СЃ С„РѕСЂРјР°С‚РѕРј NetStar Рё РёРЅСЃС‚СЂСѓРјРµРЅС‚Р°РјРё РёРјРёС‚Р°С†РёРё.");
            });
        self.show_help_development = open;
    }

    fn draw_help_controls(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_controls;
        egui::Window::new("Help: РџРѕРјРѕС‰СЊ РїРѕ СѓРїСЂР°РІР»РµРЅРёСЋ")
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading("РћСЃРЅРѕРІРЅС‹Рµ РєРЅРѕРїРєРё Рё РєРѕРјР±РёРЅР°С†РёРё");
                ui.separator();
                ui.label("Р›РљРњ: СЃРѕР·РґР°С‚СЊ/РІС‹Р±СЂР°С‚СЊ СЌР»РµРјРµРЅС‚ (РІ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РѕС‚ Р°РєС‚РёРІРЅРѕРіРѕ РёРЅСЃС‚СЂСѓРјРµРЅС‚Р°)");
                ui.label("РџРљРњ + РїРµСЂРµС‚Р°СЃРєРёРІР°РЅРёРµ: РґРІРёРіР°С‚СЊ СЂР°Р±РѕС‡СѓСЋ РѕР±Р»Р°СЃС‚СЊ");
                ui.label("Delete: СѓРґР°Р»РёС‚СЊ РІС‹РґРµР»РµРЅРЅРѕРµ");
                ui.separator();
                ui.label("Ctrl+N: РЅРѕРІС‹Р№ С„Р°Р№Р»");
                ui.label("Ctrl+O: РѕС‚РєСЂС‹С‚СЊ С„Р°Р№Р»");
                ui.label("Ctrl+S: СЃРѕС…СЂР°РЅРёС‚СЊ С„Р°Р№Р»");
                ui.label("Ctrl+C: РєРѕРїРёСЂРѕРІР°С‚СЊ РІС‹РґРµР»РµРЅРЅРѕРµ");
                ui.label("Ctrl+V: РІСЃС‚Р°РІРёС‚СЊ");
                ui.label("Ctrl+Z: РѕС‚РјРµРЅРёС‚СЊ РїРѕСЃР»РµРґРЅРµРµ РґРµР№СЃС‚РІРёРµ");
                ui.label("Ctrl+Q: РІС‹С…РѕРґ");
            });
        self.show_help_controls = open;
    }

    fn draw_status(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "РљСѓСЂСЃРѕСЂ: x={:.2}, y={:.2}",
                    self.canvas.cursor_world[0], self.canvas.cursor_world[1]
                ));
                if let Some(path) = &self.file_path {
                    ui.separator();
                    ui.label(format!("File: {}", path.display()));
                }
                if let Some(err) = &self.last_error {
                    ui.separator();
                    ui.colored_label(Color32::RED, format!("Error: {err}"));
                }
                if let Some(hint) = &self.status_hint {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(0, 90, 170), hint);
                }
            });
        });
    }

    fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let desired = ui.available_size_before_wrap();
        let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
        let painter = ui.painter_at(rect);

        let step = self.grid_step_world();
        let mut x = rect.left();
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(225)),
            );
            x += step;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(225)),
            );
            y += step;
        }

        ui.allocate_ui_at_rect(rect.shrink(6.0), |ui| {
            if self.show_table_view {
                self.draw_table_view(ui);
            }
        });
    }

    fn draw_layout(&mut self, ctx: &egui::Context) {
        if self.show_table_view && self.table_fullscreen {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.draw_table_workspace(ui);
            });
            return;
        }

        if self.layout_mode == LayoutMode::Minimized {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Р’СЃРµ РѕРєРЅР° СЃРІРµСЂРЅСѓС‚С‹");
            });
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.layout_mode {
            LayoutMode::Cascade => {
                if self.show_graph_view {
                    self.draw_graph_view(ui);
                }
                if self.show_table_view {
                    self.draw_table_workspace(ui);
                }
            }
            LayoutMode::TileHorizontal => {
                if !self.show_table_view {
                    if self.show_graph_view {
                        self.draw_graph_view(ui);
                    }
                    return;
                }
                ui.vertical(|ui| {
                    if self.show_graph_view {
                        ui.allocate_ui_with_layout(
                            Vec2::new(ui.available_width(), ui.available_height() * 0.55),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| self.draw_graph_view(ui),
                        );
                    }
                    ui.separator();
                    self.draw_table_workspace(ui);
                });
            }
            LayoutMode::TileVertical => {
                if !self.show_table_view {
                    if self.show_graph_view {
                        self.draw_graph_view(ui);
                    }
                    return;
                }
                ui.columns(2, |columns| {
                    if self.show_graph_view {
                        self.draw_graph_view(&mut columns[0]);
                    }
                    self.draw_table_workspace(&mut columns[1]);
                });
            }
            LayoutMode::Minimized => {}
        });
    }
fn draw_place_stats_dialog(&mut self, ctx: &egui::Context) {
        let Some(place_id) = self.place_stats_dialog_place_id else {
            self.place_stats_dialog_backup = None;
            return;
        };
        if !self.net.ui.marker_count_stats {
            self.place_stats_dialog_place_id = None;
            self.place_stats_dialog_backup = None;
            return;
        }
        let Some(place_idx) = self.place_idx_by_id(place_id) else {
            self.place_stats_dialog_place_id = None;
            self.place_stats_dialog_backup = None;
            return;
        };

        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(t("РЎС‚Р°С‚РёСЃС‚РёРєР°", "Statistics"))
            .id(egui::Id::new(("place_stats_dialog", place_id)))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("ID: P{}", place_id));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            if let Some((backup_id, backup)) = self.place_stats_dialog_backup.take() {
                                if backup_id == place_id {
                                    self.net.places[place_idx].stats = backup;
                                }
                            }
                            self.place_stats_dialog_place_id = None;
                        }
                        if ui.button("Ok").clicked() {
                            self.place_stats_dialog_backup = None;
                            self.place_stats_dialog_place_id = None;
                        }
                    });
                });
                ui.separator();

                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
                        ui.label(t("Р§РёСЃР»Рѕ РјР°СЂРєРµСЂРѕРІ", "Tokens"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.markers_total, t("РћР±С‰Р°СЏ", "Total"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.markers_input, t("РќР° РІС…РѕРґРµ", "On input"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.markers_output, t("РќР° РІС‹С…РѕРґРµ", "On output"));
                    });
                    cols[1].group(|ui| {
                        ui.label(t("Р—Р°РіСЂСѓР¶РµРЅРЅРѕСЃС‚СЊ", "Load"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.load_total, t("РћР±С‰Р°СЏ", "Total"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.load_input, t("Р’С…РѕРґ", "Input"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.load_output, t("Р’С‹С…РѕРґ", "Output"));
                    });
                });
            });

        if !open {
            // Treat closing via X as cancel.
            if let Some((backup_id, backup)) = self.place_stats_dialog_backup.take() {
                if backup_id == place_id {
                    self.net.places[place_idx].stats = backup;
                }
            }
            self.place_stats_dialog_place_id = None;
        }
    }

}


    impl eframe::App for PetriApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        self.draw_menu(ctx);
        self.draw_tool_palette(ctx);
        self.draw_layout(ctx);
        self.draw_status(ctx);

        if self.show_sim_params {
            self.draw_sim_dialog(ctx);
        }
        if self.show_results {
            self.draw_results(ctx);
        }
        self.draw_place_statistics_window(ctx);
        if self.show_place_props {
            self.draw_place_properties(ctx);
        }
        self.draw_place_stats_dialog(ctx);
        if self.show_transition_props {
            self.draw_transition_properties(ctx);
        }
        if self.show_debug {
            self.draw_debug_window(ctx);
        }
        if self.show_proof {
            self.draw_proof_window(ctx);
        }
        if self.show_atf {
            self.draw_atf_window(ctx);
        }
        if self.show_help_development {
            self.draw_help_development(ctx);
        }
        if self.show_help_controls {
            self.draw_help_controls(ctx);
        }
        self.handle_shortcuts(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_shortcut_copies_selected_place() {
        let mut app = PetriApp::new_for_tests();
        let selected = app.net.places[0].id;
        app.canvas.selected_place = Some(selected);

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: egui::Key::C,
            physical_key: Some(egui::Key::C),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        });

        ctx.begin_frame(raw);
        app.handle_shortcuts(&ctx);
        let _ = ctx.end_frame();

        assert!(app.clipboard.is_some(), "clipboard should be populated by Ctrl+C");
        let copied = app.clipboard.as_ref().unwrap();
        assert_eq!(copied.places.len(), 1);
    }

    #[test]
    fn ctrl_c_ru_layout_text_event_copies_selected_place() {
        let mut app = PetriApp::new_for_tests();
        let selected = app.net.places[0].id;
        app.canvas.selected_place = Some(selected);

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput {
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
            ..Default::default()
        };
        raw.events.push(egui::Event::Text("СЃ".to_string()));

        ctx.begin_frame(raw);
        app.handle_shortcuts(&ctx);
        let _ = ctx.end_frame();

        assert!(
            app.clipboard.is_some(),
            "clipboard should be populated by Ctrl+РЎ (RU layout fallback)"
        );
        let copied = app.clipboard.as_ref().unwrap();
        assert_eq!(copied.places.len(), 1);
    }
}
