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
            .add_filter("Файлы PetriNet", &["gpn2", "pn", "gpn"])
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
                                && !w.contains("восстановлены по сигнатурам")
                        })
                        .cloned()
                        .collect();
                    if filtered.is_empty() {
                        self.last_error = None;
                    } else {
                        self.last_error = Some(format!(
                            "Импорт с предупреждениями: {}",
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
            .add_filter("Файлы PetriNet", &["gpn2", "pn", "gpn"])
            .set_file_name("модель.gpn2")
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
            .add_filter("Файлы NetStar", &["gpn"])
            .set_file_name("экспорт_netstar.gpn")
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
            self.tr("Импорт CSV", "CSV import"),
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
            (LabelPosition::Top, true) => "Вверху",
            (LabelPosition::Bottom, true) => "Внизу",
            (LabelPosition::Left, true) => "Слева",
            (LabelPosition::Right, true) => "Справа",
            (LabelPosition::Center, true) => "По центру",
            (LabelPosition::Top, false) => "Top",
            (LabelPosition::Bottom, false) => "Bottom",
            (LabelPosition::Left, false) => "Left",
            (LabelPosition::Right, false) => "Right",
            (LabelPosition::Center, false) => "Center",
        }
    }

    fn node_color_text(color: NodeColor, is_ru: bool) -> &'static str {
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

    fn stochastic_text(dist: &StochasticDistribution, is_ru: bool) -> &'static str {
        match (dist, is_ru) {
            (StochasticDistribution::None, true) => "Нет",
            (StochasticDistribution::Uniform { .. }, true) => "Равномерное",
            (StochasticDistribution::Normal { .. }, true) => "Нормальное (Гаусса)",
            (StochasticDistribution::Exponential { .. }, true) => "Экспоненциальное",
            (StochasticDistribution::Poisson { .. }, true) => "Пуассона",
            (StochasticDistribution::CustomValue { .. }, true) => "Заданное пользователем",
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
            (ArcDisplayMode::All, true) => "Все",
            (ArcDisplayMode::OnlyColor, true) => "Только выбранный цвет",
            (ArcDisplayMode::Hidden, true) => "Скрыть все",
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
            self.status_hint = Some("Нечего копировать: нет выделения".to_string());
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
        self.status_hint = Some(format!("Скопировано объектов: {copied_count}"));
    }

    fn paste_copied_objects(&mut self) {
        if let Some(ext) = self.read_copy_buffer_from_system_clipboard() {
            self.clipboard = Some(ext);
        }
        let Some(buf) = self.clipboard.clone() else {
            self.status_hint = Some("Буфер пуст".to_string());
            return;
        };
        if buf.places.is_empty() && buf.transitions.is_empty() && buf.texts.is_empty() {
            self.status_hint = Some("Буфер пуст".to_string());
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
        self.status_hint = Some(format!("Вставлено объектов: {pasted_count}"));
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

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let mut do_new = false;
        let mut do_open = false;
        let mut do_save = false;
        let mut do_exit = false;
        let mut do_delete = false;
        let mut do_copy = false;
        let mut do_paste = false;
        let mut do_undo = false;

        ctx.input(|i| {
            do_new = i.modifiers.command && i.key_pressed(egui::Key::N);
            do_open = i.modifiers.command && i.key_pressed(egui::Key::O);
            do_save = i.modifiers.command && i.key_pressed(egui::Key::S);
            do_exit = i.modifiers.command && i.key_pressed(egui::Key::Q);
            do_delete = i.key_pressed(egui::Key::Delete);
            // Strict shortcuts: only Ctrl+key where Ctrl is already held.
            do_copy = i.modifiers.ctrl && i.key_pressed(egui::Key::C);
            do_paste = i.modifiers.ctrl && i.key_pressed(egui::Key::V);
            do_undo = i.modifiers.ctrl && i.key_pressed(egui::Key::Z);

            // Layout fallback (RU keyboard), still requiring Ctrl held.
            for e in &i.events {
                match e {
                    egui::Event::Copy => do_copy = true,
                    egui::Event::Paste(_) => do_paste = true,
                    _ => {}
                }
                if let egui::Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    modifiers,
                    ..
                } = e
                {
                    if modifiers.ctrl && (*key == egui::Key::C || *physical_key == Some(egui::Key::C)) {
                        do_copy = true;
                    }
                    if modifiers.ctrl && (*key == egui::Key::V || *physical_key == Some(egui::Key::V)) {
                        do_paste = true;
                    }
                    if modifiers.ctrl && (*key == egui::Key::Z || *physical_key == Some(egui::Key::Z)) {
                        do_undo = true;
                    }
                }
                if let egui::Event::Text(text) = e {
                    if i.modifiers.ctrl {
                        if text.eq_ignore_ascii_case("c") || text == "с" || text == "С" {
                            do_copy = true;
                        }
                        if text.eq_ignore_ascii_case("v") || text == "м" || text == "М" {
                            do_paste = true;
                        }
                        if text.eq_ignore_ascii_case("z") || text == "я" || text == "Я" {
                            do_undo = true;
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            {
                do_exit = do_exit || (i.modifiers.command && i.key_pressed(egui::Key::X));
            }
        });

        // Additional low-level key consumption to survive integrations where key_pressed/modifiers are flaky.
        ctx.input_mut(|i| {
            do_copy = do_copy || i.consume_key(egui::Modifiers::CTRL, egui::Key::C);
            do_paste = do_paste || i.consume_key(egui::Modifiers::CTRL, egui::Key::V);
            do_undo = do_undo || i.consume_key(egui::Modifiers::CTRL, egui::Key::Z);
        });

        if do_new {
            self.new_file();
        }
        if do_open {
            self.open_file();
        }
        if do_save {
            self.save_file();
        }
        if do_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if do_delete {
            self.delete_selected();
        }
        if do_copy {
            self.copy_selected_objects();
        }
        if do_paste {
            self.paste_copied_objects();
        }
        if do_undo {
            self.undo_last_action();
        }
    }

    fn draw_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Файл", |ui| {
                    if ui.button("Новый (Ctrl+N)").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Открыть (Ctrl+O)").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.menu_button("Импорт", |ui| {
                        ui.label("Импорт PeSim: TODO");
                    });
                    ui.menu_button("Экспорт", |ui| {
                        if ui.button("Экспортировать в NetStar").clicked() {
                            self.export_netstar_file();
                            ui.close_menu();
                        }
                    });
                    if ui.button("Сохранить (Ctrl+S)").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Сохранить как").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    if ui.button("Выход").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Опции", |ui| {
                    ui.menu_button("Язык", |ui| {
                        ui.radio_value(&mut self.net.ui.language, Language::Ru, "RU");
                        ui.radio_value(&mut self.net.ui.language, Language::En, "EN");
                    });
                    ui.checkbox(&mut self.net.ui.hide_grid, "Скрыть сетку");
                    ui.checkbox(&mut self.net.ui.snap_to_grid, "Привязка к сетке");
                    ui.checkbox(&mut self.net.ui.colored_petri_nets, "Цветные сети Петри");
                    ui.menu_button("Сбор статистики", |ui| {
                        ui.checkbox(&mut self.net.ui.marker_count_stats, "Статистика маркеров");
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Разработка").clicked() {
                            self.show_help_development = true;
                            ui.close_menu();
                        }
                        if ui.button("Помощь по управлению").clicked() {
                            self.show_help_controls = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Окно", |ui| {
                    if ui.button("Каскад").clicked() {
                        self.layout_mode = LayoutMode::Cascade;
                    }
                    if ui.button("Плитка по горизонтали").clicked() {
                        self.layout_mode = LayoutMode::TileHorizontal;
                    }
                    if ui.button("Плитка по вертикали").clicked() {
                        self.layout_mode = LayoutMode::TileVertical;
                    }
                    if ui.button("Свернуть все").clicked() {
                        self.layout_mode = LayoutMode::Minimized;
                    }
                    if ui.button("Упорядочить все").clicked() {
                        self.layout_mode = LayoutMode::TileVertical;
                        self.show_graph_view = true;
                    }
                });

                if ui.button("Параметры симуляции").clicked() {
                    self.reset_sim_stop_controls();
                    self.show_sim_params = true;
                }
                if ui.button("Структура сети").clicked() {
                    self.show_table_view = !self.show_table_view;
                    if !self.show_table_view {
                        self.table_fullscreen = false;
                    }
                }
                if ui
                    .button(self.tr("Результаты имитации", "Simulation Results"))
                    .clicked()
                {
                    self.show_results = self.sim_result.is_some();
                }
                if ui.button("Proof").clicked() && self.sim_result.is_some() {
                    self.show_proof = true;
                }
                if ui.button(self.tr("Режим отладки", "Debug Mode")).clicked()
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

    fn draw_tool_palette(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("tools").resizable(false).show(ctx, |ui| {
            ui.heading("Инструменты");
            ui.separator();
            ui.radio_value(&mut self.tool, Tool::Place, "Место");
            ui.radio_value(&mut self.tool, Tool::Transition, "Переход");
            ui.radio_value(&mut self.tool, Tool::Arc, "Дуга");
            ui.radio_value(&mut self.tool, Tool::Text, "Текст");
            ui.radio_value(&mut self.tool, Tool::Frame, "Рамка");
            ui.radio_value(&mut self.tool, Tool::Edit, "Редактировать");
            ui.radio_value(&mut self.tool, Tool::Delete, "Удалить");
            ui.radio_value(&mut self.tool, Tool::Run, "Запуск");

            if ui.button("СТАРТ").clicked() {
                self.reset_sim_stop_controls();
                self.show_sim_params = true;
            }

            ui.separator();
            ui.label(self.tr("Отображение связей", "Link visibility"));
            let is_ru = matches!(self.net.ui.language, Language::Ru);
            egui::ComboBox::from_label(self.tr("Режим", "Mode"))
                .selected_text(Self::arc_display_mode_text(self.arc_display_mode, is_ru))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.arc_display_mode,
                        ArcDisplayMode::All,
                        Self::arc_display_mode_text(ArcDisplayMode::All, is_ru),
                    );
                    ui.selectable_value(
                        &mut self.arc_display_mode,
                        ArcDisplayMode::OnlyColor,
                        Self::arc_display_mode_text(ArcDisplayMode::OnlyColor, is_ru),
                    );
                    ui.selectable_value(
                        &mut self.arc_display_mode,
                        ArcDisplayMode::Hidden,
                        Self::arc_display_mode_text(ArcDisplayMode::Hidden, is_ru),
                    );
                });

            if self.arc_display_mode == ArcDisplayMode::OnlyColor {
                let color_label = if is_ru { "Цвет" } else { "Color" };
                let c_default = if is_ru { "По умолчанию" } else { "Default" };
                let c_blue = if is_ru { "Синий" } else { "Blue" };
                let c_red = if is_ru { "Красный" } else { "Red" };
                let c_green = if is_ru { "Зеленый" } else { "Green" };
                let c_yellow = if is_ru { "Желтый" } else { "Yellow" };

                egui::ComboBox::from_label(color_label)
                    .selected_text(Self::node_color_text(self.arc_display_color, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Default, c_default);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Blue, c_blue);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Red, c_red);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Green, c_green);
                        ui.selectable_value(&mut self.arc_display_color, NodeColor::Yellow, c_yellow);
                    });
            }

            let selected_arc_ids = self.collect_selected_arc_ids();
            if !selected_arc_ids.is_empty() {
                ui.separator();
                let color_label = self.tr("Цвет", "Color");

                if selected_arc_ids.len() == 1 {
                    let arc_id = selected_arc_ids[0];
                    ui.label(self.tr("Выбранная связь", "Selected link"));

                    if let Some(arc) = self.net.arcs.iter_mut().find(|a| a.id == arc_id) {
                        egui::ComboBox::from_label(color_label)
                            .selected_text(Self::node_color_text(arc.color, is_ru))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut arc.color,
                                    NodeColor::Default,
                                    Self::node_color_text(NodeColor::Default, is_ru),
                                );
                                ui.selectable_value(
                                    &mut arc.color,
                                    NodeColor::Blue,
                                    Self::node_color_text(NodeColor::Blue, is_ru),
                                );
                                ui.selectable_value(
                                    &mut arc.color,
                                    NodeColor::Red,
                                    Self::node_color_text(NodeColor::Red, is_ru),
                                );
                                ui.selectable_value(
                                    &mut arc.color,
                                    NodeColor::Green,
                                    Self::node_color_text(NodeColor::Green, is_ru),
                                );
                                ui.selectable_value(
                                    &mut arc.color,
                                    NodeColor::Yellow,
                                    Self::node_color_text(NodeColor::Yellow, is_ru),
                                );
                            });
                    } else if let Some(inh) = self.net.inhibitor_arcs.iter_mut().find(|a| a.id == arc_id) {
                        egui::ComboBox::from_label(color_label)
                            .selected_text(Self::node_color_text(inh.color, is_ru))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut inh.color,
                                    NodeColor::Default,
                                    Self::node_color_text(NodeColor::Default, is_ru),
                                );
                                ui.selectable_value(
                                    &mut inh.color,
                                    NodeColor::Blue,
                                    Self::node_color_text(NodeColor::Blue, is_ru),
                                );
                                ui.selectable_value(
                                    &mut inh.color,
                                    NodeColor::Red,
                                    Self::node_color_text(NodeColor::Red, is_ru),
                                );
                                ui.selectable_value(
                                    &mut inh.color,
                                    NodeColor::Green,
                                    Self::node_color_text(NodeColor::Green, is_ru),
                                );
                                ui.selectable_value(
                                    &mut inh.color,
                                    NodeColor::Yellow,
                                    Self::node_color_text(NodeColor::Yellow, is_ru),
                                );
                            });
                    }
                } else {
                    let selected_label = if is_ru {
                        format!("Выбрано связей: {}", selected_arc_ids.len())
                    } else {
                        format!("Selected links: {}", selected_arc_ids.len())
                    };
                    ui.label(selected_label);

                    let mut bulk_color = selected_arc_ids
                        .iter()
                        .find_map(|id| {
                            self.net
                                .arcs
                                .iter()
                                .find(|a| a.id == *id)
                                .map(|a| a.color)
                                .or_else(|| {
                                    self.net
                                        .inhibitor_arcs
                                        .iter()
                                        .find(|a| a.id == *id)
                                        .map(|a| a.color)
                                })
                        })
                        .unwrap_or(NodeColor::Default);
                    let previous_color = bulk_color;

                    egui::ComboBox::from_label(color_label)
                        .selected_text(Self::node_color_text(bulk_color, is_ru))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut bulk_color,
                                NodeColor::Default,
                                Self::node_color_text(NodeColor::Default, is_ru),
                            );
                            ui.selectable_value(
                                &mut bulk_color,
                                NodeColor::Blue,
                                Self::node_color_text(NodeColor::Blue, is_ru),
                            );
                            ui.selectable_value(
                                &mut bulk_color,
                                NodeColor::Red,
                                Self::node_color_text(NodeColor::Red, is_ru),
                            );
                            ui.selectable_value(
                                &mut bulk_color,
                                NodeColor::Green,
                                Self::node_color_text(NodeColor::Green, is_ru),
                            );
                            ui.selectable_value(
                                &mut bulk_color,
                                NodeColor::Yellow,
                                Self::node_color_text(NodeColor::Yellow, is_ru),
                            );
                        });

                    if bulk_color != previous_color {
                        self.push_undo_snapshot();
                        let ids: HashSet<u64> = selected_arc_ids.iter().copied().collect();
                        for arc in &mut self.net.arcs {
                            if ids.contains(&arc.id) {
                                arc.color = bulk_color;
                            }
                        }
                        for inh in &mut self.net.inhibitor_arcs {
                            if ids.contains(&inh.id) {
                                inh.color = bulk_color;
                            }
                        }
                    }
                }
            }
        });
    }

    fn draw_graph_view(&mut self, ui: &mut egui::Ui) {

        ui.heading("Граф");
        let desired = ui.available_size_before_wrap();
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > f32::EPSILON {
            self.canvas.zoom = (self.canvas.zoom * zoom_delta).clamp(0.2, 3.0);
        }

        if response.dragged_by(egui::PointerButton::Secondary) {
            self.canvas.pan += response.drag_delta();
        }

        if !self.net.ui.hide_grid {
            // Draw grid aligned to world coordinates so snapped nodes land exactly on grid lines.
            let step_world = self.grid_step_world();
            let world_min = self.screen_to_world(rect, rect.left_top());
            let world_max = self.screen_to_world(rect, rect.right_bottom());
            let ppp = ui.ctx().pixels_per_point();
            let snap_to_pixel = |v: f32| (v * ppp).round() / ppp;

            let min_x = world_min[0].min(world_max[0]);
            let max_x = world_min[0].max(world_max[0]);
            let min_y = world_min[1].min(world_max[1]);
            let max_y = world_min[1].max(world_max[1]);

            // Start on the previous grid line so the first visible line is stable when panning.
            let mut xw = (min_x / step_world).floor() * step_world;
            while xw <= max_x + step_world {
                let xs = snap_to_pixel(self.world_to_screen(rect, [xw, 0.0]).x);
                painter.line_segment(
                    [Pos2::new(xs, rect.top()), Pos2::new(xs, rect.bottom())],
                    Stroke::new(1.0, Color32::from_gray(230)),
                );
                xw += step_world;
            }

            let mut yw = (min_y / step_world).floor() * step_world;
            while yw <= max_y + step_world {
                let ys = snap_to_pixel(self.world_to_screen(rect, [0.0, yw]).y);
                painter.line_segment(
                    [Pos2::new(rect.left(), ys), Pos2::new(rect.right(), ys)],
                    Stroke::new(1.0, Color32::from_gray(230)),
                );
                yw += step_world;
            }
        }

        if let Some(pos) = response.hover_pos() {
            self.canvas.cursor_world = self.screen_to_world(rect, pos);
            self.canvas.cursor_valid = true;
        }
        if response.hovered() {
            ui.output_mut(|o| {
                o.cursor_icon = match self.tool {
                    Tool::Place | Tool::Transition | Tool::Arc | Tool::Frame => egui::CursorIcon::Crosshair,
                    Tool::Text => egui::CursorIcon::Text,
                    Tool::Delete => egui::CursorIcon::NotAllowed,
                    Tool::Edit | Tool::Run => egui::CursorIcon::PointingHand,
                }
            });
        }

        if response.double_clicked_by(egui::PointerButton::Primary) {
            if let Some(click) = response.interact_pointer_pos() {
                if let Some(node) = self.node_at(rect, click) {
                    self.tool = Tool::Edit;
                    self.clear_selection();
                    match node {
                        NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                        NodeRef::Transition(t) => self.canvas.selected_transition = Some(t),
                    }
                }
            }
        }

        if response.clicked() {
            if let Some(click) = response.interact_pointer_pos() {
                let world = self.screen_to_world(rect, click);
                let snapped = self.snapped_world(world);

                match self.tool {
                    Tool::Place => {
                        self.push_undo_snapshot();
                        self.net.add_place(snapped);
                        if let Some(new_id) = self.net.places.iter().map(|p| p.id).max() {
                            self.assign_auto_name_for_place(new_id);
                        }
                    }
                    Tool::Transition => {
                        // Store transition position as top-left.
                        // Snap the top-left to the grid (not the center) so the rectangle aligns with the grid.
                        self.push_undo_snapshot();
                        let dims = Self::transition_dimensions(VisualSize::Medium);
                        let tl = self.snapped_world([world[0] - dims.x * 0.5, world[1] - dims.y * 0.5]);
                        self.net.add_transition(tl);
                    }
                    Tool::Arc => {
                    }
                    Tool::Text => {
                        self.push_undo_snapshot();
                        let id = self.next_text_id;
                        self.next_text_id = self.next_text_id.saturating_add(1);
                        self.text_blocks.push(CanvasTextBlock {
                            id,
                            pos: snapped,
                            text: self.tr("Текст", "Text").to_string(),
                        });
                        self.clear_selection();
                        self.canvas.selected_text = Some(id);
                    }
                    Tool::Frame => {}
                    Tool::Delete => {
                        if let Some(node) = self.node_at(rect, click) {
                            self.push_undo_snapshot();
                            match node {
                                NodeRef::Place(p) => {
                                    if let Some(idx) = self.place_idx_by_id(p) {
                                        self.net.places.remove(idx);
                                        self.net.set_counts(self.net.places.len(), self.net.transitions.len());
                                    }
                                }
                                NodeRef::Transition(t) => {
                                    if let Some(idx) = self.transition_idx_by_id(t) {
                                        self.net.transitions.remove(idx);
                                        self.net.set_counts(self.net.places.len(), self.net.transitions.len());
                                    }
                                }
                            }
                        } else if let Some(arc_id) = self.arc_at(rect, click) {
                            self.push_undo_snapshot();
                            self.net.arcs.retain(|a| a.id != arc_id);
                            self.net.inhibitor_arcs.retain(|a| a.id != arc_id);
                            self.net.rebuild_matrices_from_arcs();
                        } else if let Some(text_id) = self.text_at(rect, click) {
                            self.push_undo_snapshot();
                            self.text_blocks.retain(|item| item.id != text_id);
                        } else if let Some(frame_id) = self.frame_at(rect, click) {
                            self.push_undo_snapshot();
                            self.decorative_frames.retain(|item| item.id != frame_id);
                        }
                    }
                    Tool::Edit => {
                        self.clear_selection();
                        if let Some(n) = self.node_at(rect, click) {
                            match n {
                                NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                                NodeRef::Transition(t) => self.canvas.selected_transition = Some(t),
                            }
                        } else if let Some(arc_id) = self.arc_at(rect, click) {
                            self.canvas.selected_arc = Some(arc_id);
                            self.canvas.selected_arcs.clear();
                            self.canvas.selected_arcs.push(arc_id);
                        } else if let Some(text_id) = self.text_at(rect, click) {

                            self.canvas.selected_text = Some(text_id);
                        } else if let Some(frame_id) = self.frame_at(rect, click) {
                            self.canvas.selected_frame = Some(frame_id);
                        }
                    }
                    Tool::Run => {
                        self.reset_sim_stop_controls();
                        self.show_sim_params = true;
                    }
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) && self.tool == Tool::Arc {
            if let Some(pointer) = response.interact_pointer_pos() {
                self.canvas.arc_start = self.node_at(rect, pointer);
            }
        }
        if self.tool == Tool::Arc && response.drag_stopped() {
            if let Some(first) = self.canvas.arc_start.take() {
                if let Some(pointer) = response.interact_pointer_pos().or_else(|| response.hover_pos()) {
                    if let Some(last) = self.node_at(rect, pointer) {
                        if first != last {
                            self.push_undo_snapshot();
                            self.net.add_arc(first, last, 1);
                        }
                    }
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) && self.tool == Tool::Frame {
            if let Some(pointer) = response.interact_pointer_pos() {
                self.clear_selection();
                let start = self.snapped_world(self.screen_to_world(rect, pointer));
                self.canvas.frame_draw_start_world = Some(start);
                self.canvas.frame_draw_current_world = Some(start);
            }
        }

        if self.tool == Tool::Frame && response.dragged_by(egui::PointerButton::Primary) {
            if let Some(pointer) = response.interact_pointer_pos() {
                self.canvas.frame_draw_current_world = Some(self.snapped_world(self.screen_to_world(rect, pointer)));
            }
        }

        if self.tool == Tool::Frame && response.drag_stopped() {
            if let (Some(start), Some(current)) = (
                self.canvas.frame_draw_start_world.take(),
                self.canvas.frame_draw_current_world.take(),
            ) {
                let (mut pos, mut width, mut height) = Self::frame_from_drag(start, current);
                if width >= 1.0 || height >= 1.0 {
                    if self.net.ui.snap_to_grid {
                        pos = self.snap_point_to_grid(pos);
                        width = self.snap_scalar_to_grid(width);
                        height = self.snap_scalar_to_grid(height);
                    }
                    width = width.max(Self::FRAME_MIN_SIDE);
                    height = height.max(Self::FRAME_MIN_SIDE);
                    self.push_undo_snapshot();
                    let id = self.next_frame_id;
                    self.next_frame_id = self.next_frame_id.saturating_add(1);
                    self.decorative_frames.push(CanvasFrame {
                        id,
                        pos,
                        width,
                        height,
                    });
                    self.clear_selection();
                    self.canvas.selected_frame = Some(id);
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) && self.tool == Tool::Edit {
            if let Some(pointer) = response.interact_pointer_pos() {
                let mut handled_resize = false;
                if let Some(frame_id) = self.canvas.selected_frame {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        let handle = self.frame_resize_handle_rect(rect, &self.decorative_frames[idx]);
                        if handle.expand(4.0).contains(pointer) {
                            self.push_undo_snapshot();
                            self.canvas.frame_resize_id = Some(frame_id);
                            self.canvas.drag_prev_world = None;
                            self.canvas.move_drag_active = false;
                            self.canvas.selection_start = None;
                            self.canvas.selection_rect = None;
                            handled_resize = true;
                        }
                    }
                }

                if !handled_resize {
                    if let Some(node) = self.node_at(rect, pointer) {
                        let is_selected = match node {
                            NodeRef::Place(p) => {
                                self.canvas.selected_place == Some(p) || self.canvas.selected_places.contains(&p)
                            }
                            NodeRef::Transition(t) => {
                                self.canvas.selected_transition == Some(t) || self.canvas.selected_transitions.contains(&t)
                            }
                        };

                        if is_selected {
                            self.push_undo_snapshot();
                            self.canvas.drag_prev_world = Some(self.screen_to_world(rect, pointer));
                            self.canvas.move_drag_active = true;
                        } else {
                            self.clear_selection();
                            match node {
                                NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                                NodeRef::Transition(t) => self.canvas.selected_transition = Some(t),
                            }
                            self.canvas.drag_prev_world = None;
                            self.canvas.move_drag_active = false;
                        }
                    } else if let Some(text_id) = self.text_at(rect, pointer) {
                        if self.canvas.selected_text != Some(text_id) {
                            self.clear_selection();
                            self.canvas.selected_text = Some(text_id);
                        }
                        self.push_undo_snapshot();
                        self.canvas.drag_prev_world = Some(self.screen_to_world(rect, pointer));
                        self.canvas.move_drag_active = true;
                    } else if let Some(frame_id) = self.frame_at(rect, pointer) {
                        if self.canvas.selected_frame != Some(frame_id) {
                            self.clear_selection();
                            self.canvas.selected_frame = Some(frame_id);
                        }
                        self.push_undo_snapshot();
                        self.canvas.drag_prev_world = Some(self.screen_to_world(rect, pointer));
                        self.canvas.move_drag_active = true;
                    } else {
                        self.clear_selection();
                        self.canvas.selection_start = Some(pointer);
                        self.canvas.selection_rect = Some(Rect::from_two_pos(pointer, pointer));
                        self.canvas.drag_prev_world = None;
                        self.canvas.move_drag_active = false;
                    }
                }
            }
        }

        if self.tool == Tool::Edit && response.dragged_by(egui::PointerButton::Primary) {
            if let Some(frame_id) = self.canvas.frame_resize_id {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        let frame_pos = self.decorative_frames[idx].pos;
                        let world = self.screen_to_world(rect, pointer);
                        let mut width = world[0] - frame_pos[0];
                        let mut height = world[1] - frame_pos[1];
                        if self.net.ui.snap_to_grid {
                            width = self.snap_scalar_to_grid(width);
                            height = self.snap_scalar_to_grid(height);
                        }
                        self.decorative_frames[idx].width = width.max(Self::FRAME_MIN_SIDE);
                        self.decorative_frames[idx].height = height.max(Self::FRAME_MIN_SIDE);
                    }
                }
            } else if let Some(start) = self.canvas.selection_start {
                if let Some(pointer) = response.interact_pointer_pos() {
                    self.canvas.selection_rect = Some(Rect::from_two_pos(start, pointer));
                }
            } else if self.canvas.move_drag_active {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let world = self.screen_to_world(rect, pointer);
                    if let Some(prev) = self.canvas.drag_prev_world {
                        let dx = world[0] - prev[0];
                        let dy = world[1] - prev[1];
                        if dx.abs() > f32::EPSILON || dy.abs() > f32::EPSILON {
                            let move_place_ids: Vec<u64> = if self.canvas.selected_places.is_empty() {
                                self.canvas.selected_place.into_iter().collect()
                            } else {
                                self.canvas.selected_places.clone()
                            };
                            let move_transition_ids: Vec<u64> = if self.canvas.selected_transitions.is_empty() {
                                self.canvas.selected_transition.into_iter().collect()
                            } else {
                                self.canvas.selected_transitions.clone()
                            };

                            for pid in move_place_ids {
                                if let Some(idx) = self.place_idx_by_id(pid) {
                                    self.net.places[idx].pos[0] += dx;
                                    self.net.places[idx].pos[1] += dy;
                                }
                            }
                            for tid in move_transition_ids {
                                if let Some(idx) = self.transition_idx_by_id(tid) {
                                    self.net.transitions[idx].pos[0] += dx;
                                    self.net.transitions[idx].pos[1] += dy;
                                }
                            }
                            if let Some(text_id) = self.canvas.selected_text {
                                if let Some(idx) = self.text_idx_by_id(text_id) {
                                    self.text_blocks[idx].pos[0] += dx;
                                    self.text_blocks[idx].pos[1] += dy;
                                }
                            }
                            if let Some(frame_id) = self.canvas.selected_frame {
                                if let Some(idx) = self.frame_idx_by_id(frame_id) {
                                    self.decorative_frames[idx].pos[0] += dx;
                                    self.decorative_frames[idx].pos[1] += dy;
                                }
                            }
                        }
                    }
                    self.canvas.drag_prev_world = Some(world);
                }
            }
        }

        if self.tool == Tool::Edit && response.drag_stopped() {
            if self.canvas.move_drag_active && self.net.ui.snap_to_grid {
                let step = self.grid_step_world();
                let snap = |value: f32| (value / step).round() * step;
                let move_place_ids: Vec<u64> = if self.canvas.selected_places.is_empty() {
                    self.canvas.selected_place.into_iter().collect()
                } else {
                    self.canvas.selected_places.clone()
                };
                let move_transition_ids: Vec<u64> = if self.canvas.selected_transitions.is_empty() {
                    self.canvas.selected_transition.into_iter().collect()
                } else {
                    self.canvas.selected_transitions.clone()
                };
                for pid in move_place_ids {
                    if let Some(idx) = self.place_idx_by_id(pid) {
                        self.net.places[idx].pos[0] = snap(self.net.places[idx].pos[0]);
                        self.net.places[idx].pos[1] = snap(self.net.places[idx].pos[1]);
                    }
                }
                for tid in move_transition_ids {
                    if let Some(idx) = self.transition_idx_by_id(tid) {
                        self.net.transitions[idx].pos[0] = snap(self.net.transitions[idx].pos[0]);
                        self.net.transitions[idx].pos[1] = snap(self.net.transitions[idx].pos[1]);
                    }
                }
                if let Some(text_id) = self.canvas.selected_text {
                    if let Some(idx) = self.text_idx_by_id(text_id) {
                        self.text_blocks[idx].pos[0] = snap(self.text_blocks[idx].pos[0]);
                        self.text_blocks[idx].pos[1] = snap(self.text_blocks[idx].pos[1]);
                    }
                }
                if let Some(frame_id) = self.canvas.selected_frame {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        self.decorative_frames[idx].pos[0] = snap(self.decorative_frames[idx].pos[0]);
                        self.decorative_frames[idx].pos[1] = snap(self.decorative_frames[idx].pos[1]);
                    }
                }
            }
            if let Some(sel_rect) = self.canvas.selection_rect.take() {
                let norm = sel_rect.expand2(Vec2::ZERO);
                self.canvas.selected_places = self
                    .net
                    .places
                    .iter()
                    .filter(|p| norm.contains(self.world_to_screen(rect, p.pos)))
                    .map(|p| p.id)
                    .collect();
                self.canvas.selected_transitions = self
                    .net
                    .transitions
                    .iter()
                    .filter(|t| {
                        let pos = self.world_to_screen(rect, t.pos);
                        let tr_rect = Rect::from_min_size(pos, Self::transition_dimensions(t.size) * self.canvas.zoom);
                        norm.intersects(tr_rect)
                    })
                    .map(|t| t.id)
                    .collect();
                self.canvas.selected_arcs = self
                    .net
                    .arcs
                    .iter()
                    .filter(|arc| {
                        if !self.arc_visible_by_mode(arc.color, arc.visible) {
                            return false;
                        }
                        let Some((from, to)) = self.arc_screen_endpoints(rect, arc) else {
                            return false;
                        };
                        Self::arc_fully_inside_rect(norm, from, to)
                    })
                    .map(|arc| arc.id)
                    .collect();
                let selected_inhibitor_ids: Vec<u64> = self
                    .net
                    .inhibitor_arcs
                    .iter()
                    .filter(|inh| {
                        if !self.arc_visible_by_mode(inh.color, inh.visible) {
                            return false;
                        }
                        let Some((from, to)) = self.inhibitor_screen_endpoints(rect, inh) else {
                            return false;
                        };
                        norm.contains(from) && norm.contains(to)
                    })
                    .map(|inh| inh.id)
                    .collect();
                self.canvas.selected_arcs.extend(selected_inhibitor_ids);
                self.canvas.selected_arc = self.canvas.selected_arcs.first().copied();
                self.canvas.selected_place = None;
                self.canvas.selected_transition = None;
                self.canvas.selected_text = None;
                self.canvas.selected_frame = None;
            }
            self.canvas.selection_start = None;
            self.canvas.drag_prev_world = None;
            self.canvas.move_drag_active = false;
            self.canvas.frame_resize_id = None;
        }

        if response.clicked_by(egui::PointerButton::Secondary) {
            if let Some(click) = response.interact_pointer_pos() {
                if let Some(node) = self.node_at(rect, click) {
                    self.clear_selection();
                    match node {
                        NodeRef::Place(p) => {
                            self.canvas.selected_place = Some(p);
                            self.place_props_id = Some(p);
                            self.show_place_props = true;
                            self.show_transition_props = false;
                        }
                        NodeRef::Transition(t) => {
                            self.canvas.selected_transition = Some(t);
                            self.transition_props_id = Some(t);
                            self.show_transition_props = true;
                            self.show_place_props = false;
                        }
                    }
                } else if let Some(text_id) = self.text_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_text = Some(text_id);
                } else if let Some(frame_id) = self.frame_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_frame = Some(frame_id);
                }
            }
        }

        if let Some(sel) = self.canvas.selection_rect {
            painter.rect_stroke(sel, 0.0, Stroke::new(1.0, Color32::from_rgb(70, 120, 210)));
            painter.rect_filled(
                sel,
                0.0,
                Color32::from_rgba_premultiplied(70, 120, 210, 25),
            );
        }

        for frame in &self.decorative_frames {
            let min = self.world_to_screen(rect, frame.pos);
            let size = Vec2::new(
                frame.width.max(Self::FRAME_MIN_SIDE),
                frame.height.max(Self::FRAME_MIN_SIDE),
            ) * self.canvas.zoom;
            let r = Rect::from_min_size(min, size);
            let is_selected = self.canvas.selected_frame == Some(frame.id);
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(
                    if is_selected { 3.0 } else { 1.5 },
                    if is_selected {
                        Color32::from_rgb(255, 140, 0)
                    } else {
                        Color32::from_gray(90)
                    },
                ),
            );
            if is_selected {
                let handle = self.frame_resize_handle_rect(rect, frame);
                painter.rect_filled(handle, 0.0, Color32::from_rgb(255, 140, 0));
                painter.rect_stroke(handle, 0.0, Stroke::new(1.0, Color32::from_rgb(80, 40, 0)));
            }
        }
        for arc in &self.net.arcs {
            if !self.arc_visible_by_mode(arc.color, arc.visible) {
                continue;
            }
            let (from_center, from_radius, from_rect, to_center, to_radius, to_rect) = match (arc.from, arc.to) {
                (NodeRef::Place(p), NodeRef::Transition(t)) => {
                    if let (Some(pi), Some(ti)) = (self.place_idx_by_id(p), self.transition_idx_by_id(t)) {
                        let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                        let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                        let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                        let t_rect = Rect::from_min_size(t_min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom);
                        (p_center, Some(p_radius), None, t_rect.center(), None, Some(t_rect))
                    } else {
                        continue;
                    }
                }
                (NodeRef::Transition(t), NodeRef::Place(p)) => {
                    if let (Some(pi), Some(ti)) = (self.place_idx_by_id(p), self.transition_idx_by_id(t)) {
                        let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                        let t_rect = Rect::from_min_size(t_min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom);
                        let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                        let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                        (t_rect.center(), None, Some(t_rect), p_center, Some(p_radius), None)
                    } else {
                        continue;
                    }
                }
                _ => continue,
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

            let arc_color = Self::color_to_egui(arc.color, Color32::DARK_GRAY);
            let arc_stroke = if self.canvas.selected_arc == Some(arc.id) || self.canvas.selected_arcs.contains(&arc.id) {
                Stroke::new(3.0, Color32::from_rgb(255, 140, 0))
            } else {
                Stroke::new(2.0, arc_color)
            };
            painter.line_segment([from, to], arc_stroke);
            let arrow = to - from;
            if arrow.length_sq() <= f32::EPSILON {
                continue;
            }
            let dir = arrow.normalized();
            let tip = to;
            let left = tip - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
            let right = tip - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
            painter.line_segment([tip, left], arc_stroke);
            painter.line_segment([tip, right], arc_stroke);
        }

        for inh in &self.net.inhibitor_arcs {
            if !self.arc_visible_by_mode(inh.color, inh.visible) {
                continue;
            }
            if let (Some(pi), Some(ti)) = (
                self.place_idx_by_id(inh.place_id),
                self.transition_idx_by_id(inh.transition_id),
            ) {
                let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                let t_rect = Rect::from_min_size(t_min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom);
                let t_center = t_rect.center();
                let delta = t_center - p_center;
                let dir = if delta.length_sq() > 0.0 { delta.normalized() } else { Vec2::X };
                let from = p_center + dir * p_radius;
                let to = Self::rect_border_point(t_rect, -dir);
                let inh_color = Self::color_to_egui(inh.color, Color32::RED);
                let inh_stroke = if self.canvas.selected_arc == Some(inh.id) || self.canvas.selected_arcs.contains(&inh.id) {
                    Stroke::new(3.0, Color32::from_rgb(255, 140, 0))
                } else {
                    Stroke::new(1.5, inh_color)
                };
                painter.line_segment([from, to], inh_stroke);
                let mid = from + (to - from) * 0.5;
                painter.text(
                    mid,
                    egui::Align2::CENTER_CENTER,
                    format!("inh:{}", inh.threshold),
                    egui::TextStyle::Small.resolve(ui.style()),
                    Self::color_to_egui(inh.color, Color32::RED),
                );
            }
        }

        let (debug_marking, debug_touched_places) = if self.show_debug {
            self.sim_result
                .as_ref()
                .and_then(|res| {
                    let visible = Self::debug_visible_log_indices(res);
                    visible
                        .get(self.debug_step)
                        .and_then(|&log_idx| res.logs.get(log_idx))
                        .map(|entry| (entry.marking.clone(), entry.touched_places.clone()))
                })
                .unwrap_or_default()
        } else {
            (Vec::new(), Vec::new())
        };

        for (place_idx, place) in self.net.places.iter().enumerate() {
            let center = self.world_to_screen(rect, place.pos);
            let radius = Self::place_radius(place.size) * self.canvas.zoom;
            let place_color = Self::color_to_egui(place.color, Color32::BLACK);
            let is_selected = self.canvas.selected_place == Some(place.id) || self.canvas.selected_places.contains(&place.id);
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(if is_selected { 3.0 } else { 2.0 }, if is_selected { Color32::from_rgb(255, 140, 0) } else { place_color }),
            );
            let name_offset = Self::keep_label_inside(
                rect,
                center,
                Self::place_label_offset(place.text_position, radius, self.canvas.zoom),
            );
            painter.text(
                center + name_offset,
                egui::Align2::CENTER_CENTER,
                &place.name,
                egui::TextStyle::Small.resolve(ui.style()),
                if self.net.ui.colored_petri_nets {
                    Color32::from_rgb(0, 100, 180)
                } else {
                    place_color
                },
            );

            let tokens = if self.show_debug {
                debug_marking
                    .get(place_idx)
                    .copied()
                    .unwrap_or_else(|| self.net.tables.m0.get(place_idx).copied().unwrap_or(0))
            } else {
                self.net.tables.m0.get(place_idx).copied().unwrap_or(0)
            };
            let marker_color = if self.net.places[place_idx].marker_color_on_pass
                && self.show_debug
                && debug_touched_places.contains(&place_idx)
            {
                Self::color_to_egui(self.net.places[place_idx].color, Color32::from_rgb(200, 0, 0))
            } else {
                Color32::from_rgb(200, 0, 0)
            };
            if tokens > 0 {
                if tokens <= 4 {
                    let draw_tokens = tokens;
                    for i in 0..draw_tokens {
                        let angle = (i as f32) * std::f32::consts::TAU / (draw_tokens.max(1) as f32);
                        let dot_pos = center + Vec2::new(angle.cos(), angle.sin()) * (radius * 0.55);
                        painter.circle_filled(dot_pos, 3.0 * self.canvas.zoom.clamp(0.7, 1.2), marker_color);
                    }
                } else {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("{tokens}"),
                        egui::TextStyle::Body.resolve(ui.style()),
                        marker_color,
                    );
                }
            }
        }

        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let dims = Self::transition_dimensions(tr.size) * self.canvas.zoom;
            let r = Rect::from_min_size(p, dims);
            let tr_color = Self::color_to_egui(tr.color, Color32::BLACK);
            let is_selected =
                self.canvas.selected_transition == Some(tr.id) || self.canvas.selected_transitions.contains(&tr.id);
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(if is_selected { 3.0 } else { 2.0 }, if is_selected { Color32::from_rgb(255, 140, 0) } else { tr_color }),
            );
            painter.text(
                r.center() + Self::label_offset(tr.label_position, self.canvas.zoom),
                egui::Align2::CENTER_CENTER,
                &tr.name,
                egui::TextStyle::Small.resolve(ui.style()),
                tr_color,
            );
        }

        for text in &self.text_blocks {
            let center = self.world_to_screen(rect, text.pos);
            let color = if self.canvas.selected_text == Some(text.id) {
                Color32::from_rgb(255, 140, 0)
            } else {
                Color32::from_rgb(40, 40, 40)
            };
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                &text.text,
                egui::TextStyle::Body.resolve(ui.style()),
                color,
            );
        }

        let preview_pos = response.hover_pos().map(|pointer| {
            let world = self.screen_to_world(rect, pointer);
            self.world_to_screen(rect, self.snapped_world(world))
        });
        if let Some(preview) = preview_pos {
            match self.tool {
                Tool::Place => {
                    painter.circle_stroke(
                        preview,
                        Self::place_radius(VisualSize::Medium) * self.canvas.zoom,
                        Stroke::new(2.0, Color32::from_rgb(60, 120, 220)),
                    );
                }
                Tool::Transition => {
                    let dims = Self::transition_dimensions(VisualSize::Medium) * self.canvas.zoom;
                    let r = Rect::from_center_size(preview, dims);
                    painter.rect_stroke(r, 0.0, Stroke::new(2.0, Color32::from_rgb(60, 120, 220)));
                }
                Tool::Text => {
                    painter.text(
                        preview,
                        egui::Align2::CENTER_CENTER,
                        self.tr("Текст", "Text"),
                        egui::TextStyle::Body.resolve(ui.style()),
                        Color32::from_rgb(60, 120, 220),
                    );
                }
                Tool::Frame => {
                    if let (Some(start), Some(current)) = (
                        self.canvas.frame_draw_start_world,
                        self.canvas.frame_draw_current_world,
                    ) {
                        let (pos, width, height) = Self::frame_from_drag(start, current);
                        if width >= 1.0 || height >= 1.0 {
                            let min = self.world_to_screen(rect, pos);
                            let r = Rect::from_min_size(
                                min,
                                Vec2::new(
                                    width.max(Self::FRAME_MIN_SIDE),
                                    height.max(Self::FRAME_MIN_SIDE),
                                ) * self.canvas.zoom,
                            );
                            painter.rect_stroke(
                                r,
                                0.0,
                                Stroke::new(2.0, Color32::from_rgb(60, 120, 220)),
                            );
                        }
                    }
                }
                Tool::Delete => {
                    let s = 8.0 * self.canvas.zoom;
                    let a = preview + Vec2::new(-s, -s);
                    let b = preview + Vec2::new(s, s);
                    let c = preview + Vec2::new(-s, s);
                    let d = preview + Vec2::new(s, -s);
                    let stroke = Stroke::new(2.0, Color32::from_rgb(220, 60, 60));
                    painter.line_segment([a, b], stroke);
                    painter.line_segment([c, d], stroke);
                }
                _ => {}
            }
        }
        if self.tool == Tool::Arc {
            if let (Some(first), Some(pointer)) = (self.canvas.arc_start, response.hover_pos()) {
                let start = match first {
                    NodeRef::Place(pid) => {
                        if let Some(pi) = self.place_idx_by_id(pid) {
                            self.world_to_screen(rect, self.net.places[pi].pos)
                        } else {
                            pointer
                        }
                    }
                    NodeRef::Transition(tid) => {
                        if let Some(ti) = self.transition_idx_by_id(tid) {
                            let min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                            Rect::from_min_size(min, Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom).center()
                        } else {
                            pointer
                        }
                    }
                };
                let stroke = Stroke::new(2.0, Color32::from_rgb(80, 130, 230));
                painter.line_segment([start, pointer], stroke);
                let dir_vec = pointer - start;
                if dir_vec.length_sq() > 1.0 {
                    let dir = dir_vec.normalized();
                    let left = pointer - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
                    let right = pointer - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
                    painter.line_segment([pointer, left], stroke);
                    painter.line_segment([pointer, right], stroke);
                }
            }
        }

        if let Some(p) = self.canvas.selected_place {
            if let Some(idx) = self.place_idx_by_id(p) {
                let place = &mut self.net.places[idx];
                ui.separator();
                ui.label("Выбранное место");
                ui.text_edit_singleline(&mut place.name);
            }
        }
        if let Some(t) = self.canvas.selected_transition {
            if let Some(idx) = self.transition_idx_by_id(t) {
                let tr = &mut self.net.transitions[idx];
                ui.separator();
                ui.label("Выбранный переход");
                ui.text_edit_singleline(&mut tr.name);
            }
        }
        if let Some(text_id) = self.canvas.selected_text {

            if let Some(idx) = self.text_idx_by_id(text_id) {
                ui.separator();
                ui.label("Выбранный текст");
                ui.text_edit_singleline(&mut self.text_blocks[idx].text);
            }
        }
        if let Some(frame_id) = self.canvas.selected_frame {
            if let Some(idx) = self.frame_idx_by_id(frame_id) {
                ui.separator();
                ui.label("Выбранная рамка");
                                ui.horizontal(|ui| {
                    ui.label("Ширина");
                    ui.add(
                        egui::DragValue::new(&mut self.decorative_frames[idx].width)
                            .speed(1.0)
                            .range(10.0..=5000.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Высота");
                    ui.add(
                        egui::DragValue::new(&mut self.decorative_frames[idx].height)
                            .speed(1.0)
                            .range(10.0..=5000.0),
                    );
                });
            }
        }
    }

    fn draw_table_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Структура сети");
        ui.horizontal(|ui| {
            if ui.button("Скрыть структуру").clicked() {
                self.show_table_view = false;
                self.table_fullscreen = false;
            }
            if ui
                .button(if self.table_fullscreen {
                    "Обычный режим"
                } else {
                    "Полный экран"
                })
                .clicked()
            {
                self.table_fullscreen = !self.table_fullscreen;
            }
        });
        ui.separator();
        if !self.show_table_view {
            return;
        }

        let mut p_count = self.net.places.len() as i32;
        let mut t_count = self.net.transitions.len() as i32;
        ui.horizontal(|ui| {
            ui.label("Места:");
            ui.add(egui::DragValue::new(&mut p_count).range(0..=200));
            ui.label("Переходы:");
            ui.add(egui::DragValue::new(&mut t_count).range(0..=200));
            if ui.button("Применить количество").clicked() {
                self.net.set_counts(p_count.max(0) as usize, t_count.max(0) as usize);
            }
        });

        let row_label_w = 46.0;
        let cell_w = 42.0;
        egui::ScrollArea::both().show(ui, |ui| {
            ui.separator();
            ui.label("Вектор начальной маркировки (M0)");
            egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                for i in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", i + 1)));
                    ui.add_sized(
                        [cell_w * 1.4, 0.0],
                        egui::DragValue::new(&mut self.net.tables.m0[i]).range(0..=u32::MAX),
                    );
                    ui.end_row();
                }
            });

            ui.separator();
            ui.label("Вектор максимальных емкостей (Mo)");
            egui::Grid::new("mo_grid").striped(true).show(ui, |ui| {
                for i in 0..self.net.places.len() {
                    let mut cap = self.net.tables.mo[i].unwrap_or(0);
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", i + 1)));
                    if ui
                        .add_sized(
                            [cell_w * 1.4, 0.0],
                            egui::DragValue::new(&mut cap).range(0..=u32::MAX),
                        )
                        .changed()
                    {
                        self.net.tables.mo[i] = if cap == 0 { None } else { Some(cap) };
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.label("Вектор временных задержек в позициях (Mz)");
            egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                for i in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", i + 1)));
                    ui.add_sized(
                        [cell_w * 1.8, 0.0],
                        egui::DragValue::new(&mut self.net.tables.mz[i]).speed(0.1).range(0.0..=10_000.0),
                    );
                    ui.end_row();
                }
            });

            ui.separator();
            ui.label("Вектор приоритетов переходов (Mpr)");
            egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                    ui.add_sized(
                        [cell_w * 1.8, 0.0],
                        egui::DragValue::new(&mut self.net.tables.mpr[t]).speed(1),
                    );
                    ui.end_row();
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Матрица инциденций Pre");
                if ui.small_button(self.tr("Импорт CSV", "Import CSV")).clicked() {
                    self.import_matrix_csv(MatrixCsvTarget::Pre);
                }
            });
            let mut changed = false;
            egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([cell_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                }
                ui.end_row();
                for p in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", p + 1)));
                    for t in 0..self.net.transitions.len() {
                        changed |= ui
                            .add_sized(
                                [cell_w, 0.0],
                                egui::DragValue::new(&mut self.net.tables.pre[p][t]).range(0..=u32::MAX).speed(1),
                            )
                            .changed();
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Матрица инциденций Post");
                if ui.small_button(self.tr("Импорт CSV", "Import CSV")).clicked() {
                    self.import_matrix_csv(MatrixCsvTarget::Post);
                }
            });
            egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([cell_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                }
                ui.end_row();
                for p in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", p + 1)));
                    for t in 0..self.net.transitions.len() {
                        changed |= ui
                            .add_sized(
                                [cell_w, 0.0],
                                egui::DragValue::new(&mut self.net.tables.post[p][t]).range(0..=u32::MAX).speed(1),
                            )
                            .changed();
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Матрица ингибиторных дуг");
                if ui.small_button(self.tr("Импорт CSV", "Import CSV")).clicked() {
                    self.import_matrix_csv(MatrixCsvTarget::Inhibitor);
                }
            });
            egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                for t in 0..self.net.transitions.len() {
                    ui.add_sized([cell_w, 0.0], egui::Label::new(format!("T{}", t + 1)));
                }
                ui.end_row();
                for p in 0..self.net.places.len() {
                    ui.add_sized([row_label_w, 0.0], egui::Label::new(format!("P{}", p + 1)));
                    for t in 0..self.net.transitions.len() {
                        changed |= ui
                            .add_sized(
                                [cell_w, 0.0],
                                egui::DragValue::new(&mut self.net.tables.inhibitor[p][t]).range(0..=u32::MAX).speed(1),
                            )
                            .changed();
                    }
                    ui.end_row();
                }
            });

            if changed {
                self.net.rebuild_arcs_from_matrices();
            }
        });
    }

    fn draw_sim_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_sim_params;
        let mut close_now = false;
        egui::Window::new("Параметры симуляции")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.sim_params.use_time_limit, "Лимит времени (сек)");
                ui.add_enabled(
                    self.sim_params.use_time_limit,
                    egui::DragValue::new(&mut self.sim_params.time_limit_sec).speed(0.1).range(0.0..=1_000_000.0),
                );

                ui.checkbox(&mut self.sim_params.use_pass_limit, "Лимит срабатываний");
                ui.add_enabled(
                    self.sim_params.use_pass_limit,
                    egui::DragValue::new(&mut self.sim_params.pass_limit).range(0..=u64::MAX),
                );


                ui.horizontal(|ui| {
                    ui.label("Диапазон мест для вывода маркировки");
                    ui.add(egui::DragValue::new(&mut self.sim_params.display_range_start).range(0..=10000));
                    ui.add(egui::DragValue::new(&mut self.sim_params.display_range_end).range(0..=10000));
                });

                ui.separator();
                ui.label("Условия остановки");
                let mut stop_place_enabled = self.sim_params.stop.through_place.is_some();
                ui.checkbox(&mut stop_place_enabled, "Через место Pk прошло N маркеров");
                if stop_place_enabled {
                    let (mut p, mut n) = self.sim_params.stop.through_place.unwrap_or((0, 1));
                    ui.horizontal(|ui| {
                        ui.label("Pk");
                        ui.add(egui::DragValue::new(&mut p).range(0..=10000));
                        ui.label("N");
                        ui.add(egui::DragValue::new(&mut n).range(1..=u64::MAX));
                    });
                    self.sim_params.stop.through_place = Some((p, n));
                } else {
                    self.sim_params.stop.through_place = None;
                }

                let mut stop_time_enabled = self.sim_params.stop.sim_time.is_some();
                ui.checkbox(&mut stop_time_enabled, "Время симуляции достигло T секунд");
                if stop_time_enabled {
                    let mut t = self.sim_params.stop.sim_time.unwrap_or(1.0);
                    ui.add(egui::DragValue::new(&mut t).speed(0.1).range(0.0..=1_000_000.0));
                    self.sim_params.stop.sim_time = Some(t);
                } else {
                    self.sim_params.stop.sim_time = None;
                }

                if ui.button("СТАРТ").clicked() {
                    self.net.rebuild_matrices_from_arcs();
                    self.sim_result = Some(run_simulation(
                        &self.net,
                        &self.sim_params,
                        false,
                        self.net.ui.marker_count_stats,
                    ));
                    self.debug_step = 0;
                    self.debug_playing = false;
                    self.last_debug_tick = None;
                    self.show_results = true;
                    self.show_place_stats_window = false;
                    self.show_sim_params = false;
                    close_now = true;
                }
            });
        if close_now {
            open = false;
        }
        self.show_sim_params = open;
    }

    fn draw_results(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.sim_result.clone() {
            let mut open = self.show_results;
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(match result.cycle_time {
                        Some(t) => format!("{}: {:.6} {}", self.tr("Время цикла", "Cycle time"), t, self.tr("сек", "sec")),
                        None => format!("{}: N/A", self.tr("Время цикла", "Cycle time")),
                    });
                    ui.label(format!("{}: {}", self.tr("Сработало переходов", "Fired transitions"), result.fired_count));

                    let stats_places: Vec<usize> = self
                        .net
                        .places
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, place)| if place.stats.any_enabled() { Some(idx) } else { None })
                        .collect();
                    if !stats_places.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label(self.tr("Детальная статистика по позициям доступна", "Detailed per-place statistics available"));
                            if ui.button(self.tr("Статистика", "Statistics")).clicked() {
                                let selected = stats_places
                                    .iter()
                                    .position(|&p| p == self.place_stats_view_place)
                                    .unwrap_or(0);
                                self.place_stats_view_place = stats_places[selected];
                                self.show_place_stats_window = true;
                            }
                        });
                    }

                    ui.separator();
                    ui.label(self.tr("Журнал (таблица)", "Log (table)"));
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                        egui::Grid::new("sim_log_grid_header").striped(true).show(ui, |ui| {
                            ui.label(self.tr("Время", "Time"));
                            for (p, _) in self.net.places.iter().enumerate() {
                                ui.label(format!("P{}", p + 1));
                            }
                            ui.end_row();
                        });

                        egui::ScrollArea::vertical().max_height(320.0).show_rows(
                            ui,
                            row_h,
                            result.logs.len(),
                            |ui, range| {
                                egui::Grid::new("sim_log_grid_rows").striped(true).show(ui, |ui| {
                                    for idx in range {
                                        let entry = &result.logs[idx];
                                        ui.label(format!("{:.3}", entry.time));
                                        for token in &entry.marking {
                                            ui.label(token.to_string());
                                        }
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                    });

                    let any_place_stats_selected = self.net.places.iter().any(|p| p.stats.any_enabled());
                    let show_all_places_in_stats = !any_place_stats_selected;

                    if let Some(stats) = &result.place_stats {
                        ui.separator();
                        ui.label(self.tr("Статистика маркеров (min/max/avg)", "Token statistics (min/max/avg)"));
                        egui::Grid::new("stats_grid").striped(true).show(ui, |ui| {
                            ui.label(self.tr("Позиция", "Place"));
                            ui.label("Min");
                            ui.label("Max");
                            ui.label("Avg");
                            ui.end_row();
                            for (p, st) in stats.iter().enumerate() {
                                let selected = self.net.places.get(p).map(|pl| pl.stats.markers_total).unwrap_or(false);
                                if !show_all_places_in_stats && !selected {
                                    continue;
                                }
                                ui.label(format!("P{}", p + 1));
                                ui.label(st.min.to_string());
                                ui.label(st.max.to_string());
                                ui.label(format!("{:.3}", st.avg));
                                ui.end_row();
                            }
                        });
                    }

                    if let Some(flow) = &result.place_flow {
                        let want_flow = show_all_places_in_stats
                            || self.net.places.iter().any(|p| p.stats.markers_input || p.stats.markers_output);
                        if want_flow {
                            ui.separator();
                            ui.label(self.tr("Потоки (вход/выход)", "Flows (in/out)"));
                            egui::Grid::new("flow_grid").striped(true).show(ui, |ui| {
                                ui.label(self.tr("Позиция", "Place"));
                                ui.label(self.tr("Вход", "In"));
                                ui.label(self.tr("Выход", "Out"));
                                ui.end_row();
                                for (p, st) in flow.iter().enumerate() {
                                    let selected = self.net.places.get(p).map(|pl| pl.stats.markers_input || pl.stats.markers_output).unwrap_or(false);
                                    if !show_all_places_in_stats && !selected {
                                        continue;
                                    }
                                    ui.label(format!("P{}", p + 1));
                                    ui.label(st.in_tokens.to_string());
                                    ui.label(st.out_tokens.to_string());
                                    ui.end_row();
                                }
                            });
                        }
                    }

                    if let Some(load) = &result.place_load {
                        let want_load = show_all_places_in_stats
                            || self.net.places.iter().any(|p| p.stats.load_total || p.stats.load_input || p.stats.load_output);
                        if want_load {
                            ui.separator();
                            ui.label(self.tr("Загруженность", "Load"));
                            egui::Grid::new("load_grid").striped(true).show(ui, |ui| {
                                ui.label(self.tr("Позиция", "Place"));
                                ui.label(self.tr("Общая", "Total"));
                                ui.label(self.tr("Вход", "Input"));
                                ui.label(self.tr("Выход", "Output"));
                                ui.end_row();
                                for (p, st) in load.iter().enumerate() {
                                    let selected = self.net.places.get(p).map(|pl| {
                                        pl.stats.load_total || pl.stats.load_input || pl.stats.load_output
                                    }).unwrap_or(false);
                                    if !show_all_places_in_stats && !selected {
                                        continue;
                                    }
                                    ui.label(format!("P{}", p + 1));
                                    ui.label(match st.avg_over_capacity {
                                        Some(v) => format!("{:.3}", v),
                                        None => "N/A".to_string(),
                                    });
                                    ui.label(match st.in_rate {
                                        Some(v) => format!("{:.3}", v),
                                        None => "N/A".to_string(),
                                    });
                                    ui.label(match st.out_rate {
                                        Some(v) => format!("{:.3}", v),
                                        None => "N/A".to_string(),
                                    });
                                    ui.end_row();
                                }
                            });
                        }
                    }
                });
            self.show_results = open;
        }
    }

    fn draw_place_statistics_window(&mut self, ctx: &egui::Context) {
        if !self.show_place_stats_window {
            return;
        }
        let Some(result) = self.sim_result.clone() else {
            self.show_place_stats_window = false;
            return;
        };

        let available_places: Vec<usize> = self
            .net
            .places
            .iter()
            .enumerate()
            .filter_map(|(idx, place)| if place.stats.any_enabled() { Some(idx) } else { None })
            .collect();
        if available_places.is_empty() {
            self.show_place_stats_window = false;
            return;
        }
        if !available_places.contains(&self.place_stats_view_place) {
            self.place_stats_view_place = available_places[0];
        }
        let place_idx = self.place_stats_view_place;

        let mut open = self.show_place_stats_window;
        egui::Window::new(self.tr("Статистика", "Statistics"))
            .id(egui::Id::new("results_place_stats_window"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                let place_name = self
                    .net
                    .places
                    .get(place_idx)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("P{}", place_idx + 1));

                ui.horizontal(|ui| {
                    ui.label(self.tr("Позиция", "Place"));
                    let mut selected_ordinal = available_places
                        .iter()
                        .position(|&idx| idx == place_idx)
                        .unwrap_or(0);
                    ui.add(egui::DragValue::new(&mut selected_ordinal).range(0..=available_places.len().saturating_sub(1)));
                    self.place_stats_view_place = available_places[selected_ordinal];
                    ui.label(format!("P{}", self.place_stats_view_place + 1));
                    ui.separator();
                    ui.label(place_name);
                });

                let mut values = Vec::<f64>::new();
                let mut times = Vec::<f64>::new();
                for (idx, entry) in result.logs.iter().enumerate() {
                    if let Some(value) = entry.marking.get(place_idx) {
                        values.push(*value as f64);
                        let t = if entry.time.is_finite() {
                            entry.time
                        } else {
                            idx as f64
                        };
                        times.push(t);
                    }
                }
                if values.is_empty() {
                    ui.label(self.tr("Нет данных для отображения", "No data to display"));
                    return;
                }

                let mut max_v = values[0];
                let mut min_v = values[0];
                let mut max_t = times[0];
                let mut min_t = times[0];
                let mut sum = 0.0;
                for (v, t) in values.iter().zip(times.iter()) {
                    sum += *v;
                    if *v > max_v {
                        max_v = *v;
                        max_t = *t;
                    }
                    if *v < min_v {
                        min_v = *v;
                        min_t = *t;
                    }
                }
                let avg = sum / values.len() as f64;
                let utilization = result
                    .place_load
                    .as_ref()
                    .and_then(|load| load.get(place_idx))
                    .and_then(|l| l.avg_over_capacity)
                    .map(|v| v * 100.0)
                    .unwrap_or(0.0);

                ui.horizontal(|ui| {
                    ui.label(format!("{} {:.3}", self.tr("Максимум", "Maximum"), max_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), max_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Минимум", "Minimum"), min_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), min_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Среднее", "Average"), avg));
                    ui.label(format!("{} {:.3}%", self.tr("Утилизация", "Utilization"), utilization));
                });

                if let Some(place) = self.net.places.get(place_idx) {
                    ui.horizontal(|ui| {
                        let mut markers_total = place.stats.markers_total;
                        let mut markers_input = place.stats.markers_input;
                        let mut markers_output = place.stats.markers_output;
                        ui.add_enabled(false, egui::Checkbox::new(&mut markers_total, self.tr("Общая", "Total")));
                        ui.add_enabled(false, egui::Checkbox::new(&mut markers_input, self.tr("На входе", "On input")));
                        ui.add_enabled(false, egui::Checkbox::new(&mut markers_output, self.tr("На выходе", "On output")));
                    });
                }

                let desired_size = egui::Vec2::new(ui.available_width(), 320.0);
                let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));

                let x_min = times.first().copied().unwrap_or(0.0);
                let mut x_max = times.last().copied().unwrap_or(1.0);
                if x_max <= x_min {
                    x_max = x_min + (values.len().max(1) as f64);
                }
                let y_min = 0.0;
                let mut y_max = max_v.max(1.0);
                if y_max <= y_min {
                    y_max = y_min + 1.0;
                }

                for i in 1..10 {
                    let x = rect.left() + rect.width() * (i as f32 / 10.0);
                    painter.line_segment(
                        [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                        Stroke::new(0.5, Color32::LIGHT_GRAY),
                    );
                }

                let to_screen = |x: f64, y: f64| -> Pos2 {
                    let xr = ((x - x_min) / (x_max - x_min)).clamp(0.0, 1.0) as f32;
                    let yr = ((y - y_min) / (y_max - y_min)).clamp(0.0, 1.0) as f32;
                    Pos2::new(
                        rect.left() + xr * rect.width(),
                        rect.bottom() - yr * rect.height(),
                    )
                };

                let mut points = Vec::with_capacity(values.len());
                for (x, y) in times.iter().zip(values.iter()) {
                    points.push(to_screen(*x, *y));
                }
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(points, Stroke::new(1.5, Color32::BLUE)));
                }

                painter.text(
                    Pos2::new(rect.left() + 4.0, rect.top() + 4.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.0}", y_max),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
                painter.text(
                    Pos2::new(rect.left() + 4.0, rect.bottom() - 4.0),
                    egui::Align2::LEFT_BOTTOM,
                    "0",
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
            });

        self.show_place_stats_window = open;
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
                    ui.label(t("Число маркеров", "Markers"));
                    ui.add(egui::DragValue::new(&mut self.net.tables.m0[place_idx]).range(0..=u32::MAX));
                });

                let mut cap = self.net.tables.mo[place_idx].unwrap_or(0);
                ui.horizontal(|ui| {
                    ui.label(t("Макс. емкость (0 = без ограничений)", "Capacity (0 = unlimited)"));
                    if ui.add(egui::DragValue::new(&mut cap).range(0..=u32::MAX)).changed() {
                        self.net.tables.mo[place_idx] = if cap == 0 { None } else { Some(cap) };
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(t("Время задержки (сек)", "Delay (sec)"));
                    ui.add(
                        egui::DragValue::new(&mut self.net.tables.mz[place_idx])
                            .speed(0.1)
                            .range(0.0..=10_000.0),
                    );
                });

                ui.separator();
                ui.label(t("Размер позиции", "Place size"));
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.net.places[place_idx].size, VisualSize::Small, t("Малый", "Small"));
                    ui.radio_value(&mut self.net.places[place_idx].size, VisualSize::Medium, t("Средний", "Medium"));
                    ui.radio_value(&mut self.net.places[place_idx].size, VisualSize::Large, t("Большой", "Large"));
                });

                egui::ComboBox::from_label(t("Положение метки", "Marker label position"))
                    .selected_text(Self::label_pos_text(self.net.places[place_idx].marker_label_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Top, t("Вверху", "Top"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Bottom, t("Внизу", "Bottom"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Left, t("Слева", "Left"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Right, t("Справа", "Right"));
                        ui.selectable_value(&mut self.net.places[place_idx].marker_label_position, LabelPosition::Center, t("По центру", "Center"));
                    });

                egui::ComboBox::from_label(t("Положение текста", "Text position"))
                    .selected_text(Self::label_pos_text(self.net.places[place_idx].text_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Top, t("Вверху", "Top"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Bottom, t("Внизу", "Bottom"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Left, t("Слева", "Left"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Right, t("Справа", "Right"));
                        ui.selectable_value(&mut self.net.places[place_idx].text_position, LabelPosition::Center, t("По центру", "Center"));
                    });

                egui::ComboBox::from_label(t("Цвет", "Color"))
                    .selected_text(Self::node_color_text(self.net.places[place_idx].color, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Default, t("По умолчанию", "Default"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Blue, t("Синий", "Blue"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Red, t("Красный", "Red"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Green, t("Зеленый", "Green"));
                        ui.selectable_value(&mut self.net.places[place_idx].color, NodeColor::Yellow, t("Желтый", "Yellow"));
                    });

                ui.separator();
                ui.checkbox(
                    &mut self.net.places[place_idx].marker_color_on_pass,
                    t(
                        "Изменять цвет маркера при прохождении через позицию",
                        "Change marker color when token passes this place",
                    ),
                );
                ui.checkbox(
                    &mut self.net.places[place_idx].input_module,
                    t("Определить позицию как вход модуля", "Define place as module input"),
                );
                if self.net.places[place_idx].input_module {
                    ui.horizontal(|ui| {
                        ui.label(t("Номер входа", "Input number"));
                        ui.add(
                            egui::DragValue::new(&mut self.net.places[place_idx].input_number)
                                .range(1..=u32::MAX),
                        );
                    });
                    ui.label(t("Описание входа", "Input description"));
                    ui.text_edit_singleline(&mut self.net.places[place_idx].input_description);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(t("Стохастичестие процессы", "Stochastic processes"));
                    let stats_enabled = self.net.ui.marker_count_stats;
                    if ui
                        .add_enabled(stats_enabled, egui::Button::new(t("Сбор статистики", "Collect statistics")))
                        .clicked()
                    {
                        self.place_stats_dialog_place_id = Some(place_id);
                        self.place_stats_dialog_backup =
                            Some((place_id, self.net.places[place_idx].stats));
                    }
                });
                egui::ComboBox::from_label(t("Распределение", "Distribution"))
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
                            ui.label(t("Значение", "Value"));
                            ui.add(egui::DragValue::new(value).speed(0.1).range(0.0..=10_000.0));
                        });
                    }
                }

                ui.separator();
                ui.label(t("Название", "Name"));
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
            let title = self.tr("Свойства позиции", "Place Properties").to_owned();
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
                    ui.label(t("Приоритет", "Priority"));
                    ui.add(egui::DragValue::new(&mut self.net.tables.mpr[transition_idx]));
                });
                ui.horizontal(|ui| {
                    ui.label(t("Угол наклона", "Angle"));
                    ui.add(egui::DragValue::new(&mut self.net.transitions[transition_idx].angle_deg).range(-180..=180));
                });

                ui.label(t("Размер перехода", "Transition size"));
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.net.transitions[transition_idx].size, VisualSize::Small, t("Малый", "Small"));
                    ui.radio_value(&mut self.net.transitions[transition_idx].size, VisualSize::Medium, t("Средний", "Medium"));
                    ui.radio_value(&mut self.net.transitions[transition_idx].size, VisualSize::Large, t("Большой", "Large"));
                });

                egui::ComboBox::from_label(t("Положение метки", "Label position"))
                    .selected_text(Self::label_pos_text(self.net.transitions[transition_idx].label_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Top, t("Вверху", "Top"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Bottom, t("Внизу", "Bottom"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Left, t("Слева", "Left"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Right, t("Справа", "Right"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].label_position, LabelPosition::Center, t("По центру", "Center"));
                    });

                egui::ComboBox::from_label(t("Положение текста", "Text position"))
                    .selected_text(Self::label_pos_text(self.net.transitions[transition_idx].text_position, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Top, t("Вверху", "Top"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Bottom, t("Внизу", "Bottom"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Left, t("Слева", "Left"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Right, t("Справа", "Right"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].text_position, LabelPosition::Center, t("По центру", "Center"));
                    });

                egui::ComboBox::from_label(t("Цвет", "Color"))
                    .selected_text(Self::node_color_text(self.net.transitions[transition_idx].color, is_ru))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Default, t("По умолчанию", "Default"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Blue, t("Синий", "Blue"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Red, t("Красный", "Red"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Green, t("Зеленый", "Green"));
                        ui.selectable_value(&mut self.net.transitions[transition_idx].color, NodeColor::Yellow, t("Желтый", "Yellow"));
                    });

                ui.separator();
                ui.label(t("Название", "Name"));
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
                .tr("Свойства перехода", "Transition Properties")
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
        egui::Window::new(t("Режим отладки", "Debug Mode"))
            .open(&mut open)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.clone() else {
                    ui.label(t("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };
                let visible_steps = Self::debug_visible_log_indices(&result);
                let steps = visible_steps.len();
                if steps == 0 {
                    ui.label(t("Пустой журнал.", "Empty log."));
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
                    if ui.button(if self.debug_playing { t("Пауза", "Pause") } else { t("Пуск", "Play") }).clicked() {
                        self.debug_playing = !self.debug_playing;
                        self.last_debug_tick = Some(Instant::now());
                    }
                    if ui.button(">>").clicked() {
                        self.debug_step = (self.debug_step + 1).min(steps - 1);
                    }
                    ui.label(t("Скорость (мс):", "Speed (ms):"));
                    ui.add(egui::DragValue::new(&mut self.debug_interval_ms).range(50..=5_000));
                });

                ui.add(egui::Slider::new(&mut self.debug_step, 0..=steps - 1).text(t("Шаг", "Step")));
                if let Some(&log_idx) = visible_steps.get(self.debug_step) {
                    if let Some(entry) = result.logs.get(log_idx) {
                    ui.separator();
                    ui.label(format!("t = {:.3}", entry.time));
                    ui.label(format!(
                        "{}: {}",
                        t("Переход", "Transition"),
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
                    ui.label(self.tr("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };
                ui.label(self.tr(
                    "Доказательство построено по журналу состояний (trace).",
                    "Proof is generated from simulation trace.",
                ));
                ui.separator();
                egui::Grid::new("proof_grid").striped(true).show(ui, |ui| {
                    ui.label(self.tr("Шаг", "Step"));
                    ui.label(self.tr("Время", "Time"));
                    ui.label(self.tr("Сработал переход", "Fired transition"));
                    ui.label(self.tr("Маркировка", "Marking"));
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
                        ui.label("Левая область");
                        ui.horizontal(|ui| {
                            ui.label("P:");
                            ui.add(egui::DragValue::new(&mut self.atf_selected_place).range(0..=10000));
                            if ui.button("OK").clicked() {
                                self.atf_text = generate_atf(&self.net, self.atf_selected_place.min(self.net.places.len().saturating_sub(1)));
                            }
                        });
                        if ui.button("Сгенерировать ATF").clicked() {
                            self.atf_text = generate_atf(&self.net, self.atf_selected_place.min(self.net.places.len().saturating_sub(1)));
                        }
                        if ui.button("Открыть ATF файл").clicked() {
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
        egui::Window::new("Help: Разработка")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Информация о приложении");
                ui.separator();
                ui.label(egui::RichText::new(format!("Версия: {}", env!("CARGO_PKG_VERSION"))).size(20.0));
                ui.label(egui::RichText::new("Разработчик: Вайбкод + вылеты NetStar").size(18.0));
                ui.separator();
                ui.label("Редактор сетей Петри с совместимостью с форматом NetStar и инструментами имитации.");
            });
        self.show_help_development = open;
    }

    fn draw_help_controls(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_controls;
        egui::Window::new("Help: Помощь по управлению")
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading("Основные кнопки и комбинации");
                ui.separator();
                ui.label("ЛКМ: создать/выбрать элемент (в зависимости от активного инструмента)");
                ui.label("ПКМ + перетаскивание: двигать рабочую область");
                ui.label("Delete: удалить выделенное");
                ui.separator();
                ui.label("Ctrl+N: новый файл");
                ui.label("Ctrl+O: открыть файл");
                ui.label("Ctrl+S: сохранить файл");
                ui.label("Ctrl+C: копировать выделенное");
                ui.label("Ctrl+V: вставить");
                ui.label("Ctrl+Z: отменить последнее действие");
                ui.label("Ctrl+Q: выход");
            });
        self.show_help_controls = open;
    }

    fn draw_status(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Курсор: x={:.2}, y={:.2}",
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
                ui.heading("Все окна свернуты");
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
        egui::Window::new(t("Статистика", "Statistics"))
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
                        ui.label(t("Число маркеров", "Tokens"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.markers_total, t("Общая", "Total"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.markers_input, t("На входе", "On input"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.markers_output, t("На выходе", "On output"));
                    });
                    cols[1].group(|ui| {
                        ui.label(t("Загруженность", "Load"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.load_total, t("Общая", "Total"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.load_input, t("Вход", "Input"));
                        ui.checkbox(&mut self.net.places[place_idx].stats.load_output, t("Выход", "Output"));
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
        raw.events.push(egui::Event::Text("с".to_string()));

        ctx.begin_frame(raw);
        app.handle_shortcuts(&ctx);
        let _ = ctx.end_frame();

        assert!(
            app.clipboard.is_some(),
            "clipboard should be populated by Ctrl+С (RU layout fallback)"
        );
        let copied = app.clipboard.as_ref().unwrap();
        assert_eq!(copied.places.len(), 1);
    }
}
