use std::borrow::Cow;
use std::fs;

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};

use crate::formats::atf::generate_atf;
use crate::io::{load_gpn, save_gpn_with_hints, LegacyExportHints};
use crate::markov::{build_markov_chain, MarkovChain};
use crate::model::{
    LabelPosition, Language, NodeColor, NodeRef, PetriNet, Place, PlaceStatisticsSelection,
    StochasticDistribution, Tool, Transition, UiDecorativeFrame, UiTextBlock, VisualSize,
};
use crate::sim::engine::{run_simulation, SimulationParams, SimulationResult};

mod graph_view;
mod petri_app;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlaceStatsSeries {
    Total,
    Input,
    Output,
}

#[derive(Debug, Clone, Default)]
struct NetstarExportValidationReport {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl NetstarExportValidationReport {
    fn error_count(&self) -> usize {
        self.errors.len()
    }

    fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
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
    selected_texts: Vec<u64>,
    selected_frame: Option<u64>,
    selected_frames: Vec<u64>,
    arc_start: Option<NodeRef>,
    cursor_world: [f32; 2],
    selection_start: Option<Pos2>,
    selection_rect: Option<Rect>,
    selection_toggle_mode: bool,
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
            selected_texts: Vec::new(),
            selected_frame: None,
            selected_frames: Vec::new(),
            arc_start: None,
            cursor_world: [0.0, 0.0],
            selection_start: None,
            selection_rect: None,
            selection_toggle_mode: false,
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
#[serde(default)]
struct CanvasTextBlock {
    id: u64,
    pos: [f32; 2],
    text: String,
    font_name: String,
    font_size: f32,
    color: NodeColor,
}

impl Default for CanvasTextBlock {
    fn default() -> Self {
        Self {
            id: 0,
            pos: [0.0, 0.0],
            text: String::new(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        }
    }
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
}
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(default)]
struct CopiedTextBlock {
    pos: [f32; 2],
    text: String,
    font_name: String,
    font_size: f32,
    color: NodeColor,
}

impl Default for CopiedTextBlock {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0],
            text: String::new(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        }
    }
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

#[derive(Debug, Clone)]
struct DebugAnimationArc {
    arc_id: u64,
    weight: u32,
}

#[derive(Debug, Clone)]
struct DebugAnimationEvent {
    transition_idx: usize,
    step_idx: usize,
    duration: f64,
    entry_color: Color32,
    exit_color: Color32,
    pre_arcs: Vec<DebugAnimationArc>,
    post_arcs: Vec<DebugAnimationArc>,
}

impl DebugAnimationEvent {
    fn duration(&self) -> f64 {
        self.duration
    }
}

fn sanitize_f64(value: &mut f64, min: f64, max: f64) -> bool {
    if !value.is_finite() {
        *value = min;
        return true;
    }
    let clamped = value.clamp(min, max);
    let changed = (clamped - *value).abs() > f64::EPSILON;
    if changed {
        *value = clamped;
    }
    changed
}

fn sanitize_bounded<T: PartialOrd + Copy>(value: &mut T, min: T, max: T) -> bool {
    let mut changed = false;
    if *value < min {
        *value = min;
        changed = true;
    }
    if *value > max {
        *value = max;
        changed = true;
    }
    changed
}

fn sanitize_u64(value: &mut u64, min: u64, max: u64) -> bool {
    sanitize_bounded(value, min, max)
}

fn sanitize_usize(value: &mut usize, min: usize, max: usize) -> bool {
    sanitize_bounded(value, min, max)
}

fn sanitize_u32(value: &mut u32, min: u32, max: u32) -> bool {
    sanitize_bounded(value, min, max)
}

fn sanitize_i32(value: &mut i32, min: i32, max: i32) -> bool {
    sanitize_bounded(value, min, max)
}

fn validation_hint(ui: &mut egui::Ui, corrected: bool, msg: &str) {
    if corrected {
        ui.colored_label(Color32::from_rgb(190, 40, 40), msg);
    }
}

pub struct PetriApp {
    net: PetriNet,
    tool: Tool,
    canvas: CanvasState,
    sim_params: SimulationParams,
    sim_result: Option<Arc<SimulationResult>>,
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
    show_struct_vectors: bool,
    show_struct_pre: bool,
    show_struct_post: bool,
    show_struct_inhibitor: bool,
    place_props_id: Option<u64>,
    transition_props_id: Option<u64>,
    show_place_props: bool,
    show_transition_props: bool,
    text_props_id: Option<u64>,
    show_text_props: bool,
    arc_props_id: Option<u64>,
    show_arc_props: bool,
    show_debug: bool,
    debug_step: usize,
    debug_playing: bool,
    debug_interval_ms: u64,
    debug_arc_animation: bool,
    debug_animation_enabled: bool,
    debug_animation_local_clock: f64,
    debug_animation_current_duration: f64,
    debug_animation_last_update: Option<Instant>,
    debug_animation_events: Vec<DebugAnimationEvent>,
    debug_animation_active_event: Option<usize>,
    debug_animation_step_active: bool,
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
    place_stats_series: PlaceStatsSeries,
    place_stats_zoom_x: f32,
    place_stats_pan_x: f32,
    place_stats_show_grid: bool,
    arc_display_mode: ArcDisplayMode,
    arc_display_color: NodeColor,
    show_netstar_export_validation: bool,
    pending_netstar_export_path: Option<PathBuf>,
    netstar_export_validation: Option<NetstarExportValidationReport>,
    show_new_element_props: bool,
    show_markov_window: bool,
    markov_model: Option<MarkovChain>,
    markov_limit_reached: bool,
    markov_annotations: HashMap<u64, String>,
    new_place_size: VisualSize,
    new_place_color: NodeColor,
    new_place_marking: u32,
    new_place_capacity: Option<u32>,
    new_place_delay: f64,
    new_transition_size: VisualSize,
    new_transition_color: NodeColor,
    new_transition_priority: i32,
    new_arc_weight: u32,
    new_arc_color: NodeColor,
    new_arc_inhibitor: bool,
    new_arc_inhibitor_threshold: u32,
    new_element_props_window_size: Vec2,
    new_element_props_window_was_open: bool,
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
    const MAX_PLOT_POINTS: usize = 2_000;
    const DEBUG_ANIMATION_MIN_DURATION: f64 = 0.1;

    pub(in crate::ui::app) fn refresh_debug_animation_state(&mut self) {
        if let Some(result) = self.sim_result.as_ref() {
            self.debug_animation_events = Self::build_debug_animation_events(&self.net, result);
        } else {
            self.debug_animation_events.clear();
        }
        self.sync_debug_animation_for_step();
    }

    fn sync_debug_animation_for_step(&mut self) {
        self.debug_animation_last_update = None;
        if !self.debug_animation_enabled || self.debug_animation_events.is_empty() {
            self.clear_debug_animation_state();
            return;
        }
        let Some(result) = self.sim_result.as_ref() else {
            self.clear_debug_animation_state();
            return;
        };
        let visible_steps = Self::debug_visible_log_indices(result);
        if visible_steps.is_empty() {
            self.clear_debug_animation_state();
            return;
        }
        if self.debug_step >= visible_steps.len() {
            self.debug_step = visible_steps.len() - 1;
        }
        let target_step = self
            .debug_step
            .checked_add(1)
            .filter(|next| *next < visible_steps.len());
        let event_idx = target_step.and_then(|step| {
            self.debug_animation_events
                .iter()
                .position(|event| event.step_idx == step)
        });
        self.set_active_debug_animation_event(event_idx, visible_steps.len());
    }
    fn set_active_debug_animation_event(&mut self, event_idx: Option<usize>, visible_len: usize) {
        self.debug_animation_active_event = event_idx;
        if let Some(idx) = event_idx {
            if visible_len > 0 && self.debug_step >= visible_len {
                self.debug_step = visible_len - 1;
            }
            let duration = self.debug_animation_events[idx]
                .duration()
                .max(Self::DEBUG_ANIMATION_MIN_DURATION);
            self.debug_animation_current_duration = duration;
            self.debug_animation_local_clock = 0.0;
            self.debug_animation_step_active = self.debug_playing && duration > 0.0;
            self.debug_animation_last_update = None;
        } else {
            self.debug_animation_local_clock = 0.0;
            self.debug_animation_current_duration = 0.0;
            self.debug_animation_step_active = false;
        }
    }

    fn clear_debug_animation_state(&mut self) {
        self.debug_animation_active_event = None;
        self.debug_animation_local_clock = 0.0;
        self.debug_animation_last_update = None;
        self.debug_playing = false;
        self.debug_animation_current_duration = 0.0;
        self.debug_animation_step_active = false;
    }

    fn debug_animation_playback_speed(&self) -> f64 {
        let interval = self.debug_interval_ms.max(1);
        1000.0 / interval as f64
    }

    fn build_debug_animation_events(
        net: &PetriNet,
        result: &SimulationResult,
    ) -> Vec<DebugAnimationEvent> {
        let mut events = Vec::new();
        let mut current_marker_color = Color32::from_rgb(200, 0, 0);
        let visible_steps = Self::debug_visible_log_indices(result);
        let log_to_step: HashMap<usize, usize> = visible_steps
            .iter()
            .copied()
            .enumerate()
            .map(|(step, log_idx)| (log_idx, step))
            .collect();
        for (idx, entry) in result.logs.iter().enumerate() {
            let Some(transition_idx) = entry.fired_transition else {
                continue;
            };
            let mut next_time = entry.time + Self::DEBUG_ANIMATION_MIN_DURATION;
            for next_entry in result.logs.iter().skip(idx + 1) {
                if next_entry.time > entry.time {
                    next_time = next_entry.time;
                    break;
                }
            }
            let duration = (next_time - entry.time).max(Self::DEBUG_ANIMATION_MIN_DURATION);
            let step_idx = *log_to_step.get(&idx).unwrap_or(&visible_steps.len());
            let pre_arcs = Self::transition_arcs(net, transition_idx, true);
            let post_arcs = Self::transition_arcs(net, transition_idx, false);
            let entry_color = current_marker_color;
            let exit_color =
                Self::marker_color_for_arcs(net, &post_arcs, entry_color).unwrap_or(entry_color);
            events.push(DebugAnimationEvent {
                transition_idx,
                step_idx,
                duration,
                entry_color,
                exit_color,
                pre_arcs,
                post_arcs,
            });
            current_marker_color = exit_color;
        }
        events
    }

    fn transition_arcs(
        net: &PetriNet,
        transition_idx: usize,
        incoming: bool,
    ) -> Vec<DebugAnimationArc> {
        let Some(transition) = net.transitions.get(transition_idx) else {
            return Vec::new();
        };
        let transition_id = transition.id;
        net.arcs
            .iter()
            .filter(|arc| arc.weight > 0)
            .filter_map(|arc| {
                if incoming {
                    match (&arc.from, &arc.to) {
                        (NodeRef::Place(_), NodeRef::Transition(id)) if *id == transition_id => {
                            Some(DebugAnimationArc {
                                arc_id: arc.id,
                                weight: arc.weight,
                            })
                        }
                        _ => None,
                    }
                } else {
                    match (&arc.from, &arc.to) {
                        (NodeRef::Transition(id), NodeRef::Place(_)) if *id == transition_id => {
                            Some(DebugAnimationArc {
                                arc_id: arc.id,
                                weight: arc.weight,
                            })
                        }
                        _ => None,
                    }
                }
            })
            .collect()
    }

    fn marker_color_for_arcs(
        net: &PetriNet,
        arcs: &[DebugAnimationArc],
        fallback: Color32,
    ) -> Option<Color32> {
        for arc in arcs {
            let arc_idx = net.arcs.iter().position(|entry| entry.id == arc.arc_id)?;
            let arc_data = net.arcs.get(arc_idx)?;
            let place_id = match arc_data.from {
                NodeRef::Place(id) => id,
                _ => continue,
            };
            let place_idx = net.places.iter().position(|place| place.id == place_id)?;
            let place = &net.places[place_idx];
            if place.marker_color_on_pass {
                return Some(Self::color_to_egui(place.color, fallback));
            }
        }
        None
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
        if self.show_arc_props {
            self.draw_arc_properties(ctx);
        }
        if self.show_text_props {
            self.draw_text_properties(ctx);
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
        if self.show_markov_window {
            self.draw_markov_window(ctx);
        }
        self.draw_netstar_export_validation(ctx);
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

        assert!(
            app.clipboard.is_some(),
            "clipboard should be populated by Ctrl+C"
        );
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

    #[test]
    fn netstar_export_validation_has_error_for_broken_arc_link() {
        let mut app = PetriApp::new_for_tests();
        app.net.arcs.push(crate::model::Arc {
            id: 999,
            from: NodeRef::Place(999_999),
            to: NodeRef::Transition(app.net.transitions[0].id),
            weight: 1,
            color: NodeColor::Default,
            visible: true,
        });

        let report = app.validate_netstar_export();
        assert!(
            report.error_count() > 0,
            "broken arc link must produce a blocking export error"
        );
    }

    #[test]
    fn netstar_export_validation_warns_for_non_exportable_ui_elements() {
        let mut app = PetriApp::new_for_tests();
        app.text_blocks.push(CanvasTextBlock {
            id: 1,
            pos: [10.0, 10.0],
            text: "x".to_string(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        });
        app.decorative_frames.push(CanvasFrame {
            id: 1,
            pos: [20.0, 20.0],
            width: 120.0,
            height: 80.0,
        });

        let report = app.validate_netstar_export();
        assert!(
            report.warning_count() >= 2,
            "text blocks and frames should be reported as export warnings"
        );
    }
}
