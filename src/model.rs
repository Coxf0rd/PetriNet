use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const GPN2_MAGIC: &str = "GPN2\n";
pub const GPN2_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Ru,
    En,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tool {
    Place,
    Transition,
    Arc,
    Text,
    Frame,
    Edit,
    Delete,
    Run,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum VisualSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LabelPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum NodeColor {
    #[default]
    Default,
    Blue,
    Red,
    Green,
    Yellow,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarkovPlacement {
    Bottom,
    Top,
}

impl Default for MarkovPlacement {
    fn default() -> Self {
        Self::Bottom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaInfo {
    pub name: String,
    pub author: String,
    pub description: String,
}

impl Default for MetaInfo {
    fn default() -> Self {
        Self {
            name: "Без названия".to_string(),
            author: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiTextBlock {
    pub id: u64,
    pub pos: [f32; 2],
    pub text: String,
    pub font_name: String,
    pub font_size: f32,
    pub color: NodeColor,
}

impl Default for UiTextBlock {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiDecorativeFrame {
    pub id: u64,
    pub pos: [f32; 2],
    pub width: f32,
    pub height: f32,
}

impl Default for UiDecorativeFrame {
    fn default() -> Self {
        Self {
            id: 0,
            pos: [0.0, 0.0],
            width: 120.0,
            height: 120.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiSettings {
    pub language: Language,
    pub hide_grid: bool,
    pub snap_to_grid: bool,
    pub colored_petri_nets: bool,
    pub fix_time_step: bool,
    pub marker_count_stats: bool,
    pub light_theme: bool,
    pub text_blocks: Vec<UiTextBlock>,
    pub decorative_frames: Vec<UiDecorativeFrame>,
    pub next_text_id: u64,
    pub next_frame_id: u64,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            hide_grid: false,
            snap_to_grid: true,
            colored_petri_nets: false,
            fix_time_step: true,
            marker_count_stats: true,
            light_theme: true,
            text_blocks: Vec::new(),
            decorative_frames: Vec::new(),
            next_text_id: 1,
            next_frame_id: 1,
        }
    }
}

fn default_visible_true() -> bool {
    true
}

fn default_inhibitor_color() -> NodeColor {
    NodeColor::Red
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Place {
    pub id: u64,
    pub name: String,
    pub pos: [f32; 2],
    pub note: String,
    pub color: NodeColor,
    pub marker_label_position: LabelPosition,
    pub text_position: LabelPosition,
    pub size: VisualSize,
    pub marker_color_on_pass: bool,
    pub input_module: bool,
    pub input_number: u32,
    pub input_description: String,
    pub stochastic: StochasticDistribution,
    pub stochastic_seed: u64,
    pub stats: PlaceStatisticsSelection,
    pub markov_highlight: bool,
    pub markov_placement: MarkovPlacement,
    pub show_markov_model: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct PlaceStatisticsSelection {
    pub markers_total: bool,
    pub markers_input: bool,
    pub markers_output: bool,
    pub load_total: bool,
    pub load_input: bool,
    pub load_output: bool,
}

impl PlaceStatisticsSelection {
    pub fn any_enabled(&self) -> bool {
        self.markers_total
            || self.markers_input
            || self.markers_output
            || self.load_total
            || self.load_input
            || self.load_output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum StochasticDistribution {
    #[default]
    None,
    Uniform {
        min: f64,
        max: f64,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Exponential {
        lambda: f64,
    },
    Gamma {
        shape: f64,
        scale: f64,
    },
    Poisson {
        lambda: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Transition {
    pub id: u64,
    pub name: String,
    pub pos: [f32; 2],
    pub note: String,
    pub color: NodeColor,
    pub label_position: LabelPosition,
    pub text_position: LabelPosition,
    pub size: VisualSize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "id")]
pub enum NodeRef {
    Place(u64),
    Transition(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Arc {
    pub id: u64,
    pub from: NodeRef,
    pub to: NodeRef,
    pub weight: u32,
    pub color: NodeColor,
    #[serde(default = "default_visible_true")]
    pub visible: bool,
    #[serde(default)]
    pub show_weight: bool,
}

impl Default for Arc {
    fn default() -> Self {
        Self {
            id: 0,
            from: NodeRef::Place(0),
            to: NodeRef::Transition(0),
            weight: 1,
            color: NodeColor::Default,
            visible: true,
            show_weight: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct InhibitorArc {
    pub id: u64,
    pub place_id: u64,
    pub transition_id: u64,
    pub threshold: u32,
    pub show_weight: bool,
    #[serde(default = "default_inhibitor_color")]
    pub color: NodeColor,
    #[serde(default = "default_visible_true")]
    pub visible: bool,
}

impl Default for InhibitorArc {
    fn default() -> Self {
        Self {
            id: 0,
            place_id: 0,
            transition_id: 0,
            threshold: 1,
            color: NodeColor::Red,
            visible: true,
            show_weight: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Tables {
    pub m0: Vec<u32>,
    pub mo: Vec<Option<u32>>,
    pub mz: Vec<f64>,
    pub mpr: Vec<i32>,
    pub pre: Vec<Vec<u32>>,
    pub post: Vec<Vec<u32>>,
    pub inhibitor: Vec<Vec<u32>>,
}

impl Tables {
    pub fn resize(&mut self, places: usize, transitions: usize) {
        self.m0.resize(places, 0);
        // Default place capacity is 1 (Mo=1). Use None only when explicitly set to unlimited.
        self.mo.resize_with(places, || Some(1));
        self.mz.resize(places, 0.0);
        self.mpr.resize(transitions, 0);

        self.pre.resize_with(places, || vec![0; transitions]);
        self.post.resize_with(places, || vec![0; transitions]);
        self.inhibitor.resize_with(places, || vec![0; transitions]);

        for row in &mut self.pre {
            row.resize(transitions, 0);
        }
        for row in &mut self.post {
            row.resize(transitions, 0);
        }
        for row in &mut self.inhibitor {
            row.resize(transitions, 0);
        }
    }

    pub(crate) fn remove_place_row(&mut self, idx: usize) {
        if idx < self.m0.len() {
            self.m0.remove(idx);
        }
        if idx < self.mo.len() {
            self.mo.remove(idx);
        }
        if idx < self.mz.len() {
            self.mz.remove(idx);
        }
        if idx < self.pre.len() {
            self.pre.remove(idx);
        }
        if idx < self.post.len() {
            self.post.remove(idx);
        }
        if idx < self.inhibitor.len() {
            self.inhibitor.remove(idx);
        }
    }

    pub(crate) fn remove_transition_column(&mut self, idx: usize) {
        if idx < self.mpr.len() {
            self.mpr.remove(idx);
        }
        for row in &mut self.pre {
            if idx < row.len() {
                row.remove(idx);
            }
        }
        for row in &mut self.post {
            if idx < row.len() {
                row.remove(idx);
            }
        }
        for row in &mut self.inhibitor {
            if idx < row.len() {
                row.remove(idx);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PetriNetModel {
    pub format_version: u32,
    pub meta: MetaInfo,
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
    pub inhibitor_arcs: Vec<InhibitorArc>,
    pub tables: Tables,
    pub ui: UiSettings,
}

pub type PetriNet = PetriNetModel;

impl Default for PetriNetModel {
    fn default() -> Self {
        Self::new()
    }
}

impl PetriNetModel {
    fn is_auto_name(name: &str, prefixes: &[char]) -> bool {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !prefixes.contains(&first) {
            return false;
        }
        let digits: String = chars.collect();
        !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
    }

    pub fn new() -> Self {
        Self {
            format_version: GPN2_FORMAT_VERSION,
            meta: MetaInfo::default(),
            places: Vec::new(),
            transitions: Vec::new(),
            arcs: Vec::new(),
            inhibitor_arcs: Vec::new(),
            tables: Tables::default(),
            ui: UiSettings::default(),
        }
    }

    fn next_place_id(&self) -> u64 {
        self.places.iter().map(|p| p.id).max().unwrap_or(0) + 1
    }

    fn next_transition_id(&self) -> u64 {
        self.transitions.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    fn next_arc_id(&self) -> u64 {
        let max_arc = self.arcs.iter().map(|a| a.id).max().unwrap_or(0);
        let max_inh = self.inhibitor_arcs.iter().map(|a| a.id).max().unwrap_or(0);
        max_arc.max(max_inh) + 1
    }

    fn next_inhibitor_id(&self) -> u64 {
        let max_arc = self.arcs.iter().map(|a| a.id).max().unwrap_or(0);
        let max_inh = self.inhibitor_arcs.iter().map(|a| a.id).max().unwrap_or(0);
        max_arc.max(max_inh) + 1
    }

    pub fn normalize_arc_ids(&mut self) {
        let mut next_id = 1_u64;
        for arc in &mut self.arcs {
            arc.id = next_id;
            next_id = next_id.saturating_add(1);
        }
        for arc in &mut self.inhibitor_arcs {
            arc.id = next_id;
            next_id = next_id.saturating_add(1);
        }
    }

    fn default_place_pos(index: usize) -> [f32; 2] {
        let col = (index % 8) as f32;
        let row = (index / 8) as f32;
        [40.0 + col * 140.0, 40.0 + row * 140.0]
    }

    fn default_transition_pos(index: usize) -> [f32; 2] {
        let col = (index % 8) as f32;
        let row = (index / 8) as f32;
        [110.0 + col * 140.0, 40.0 + row * 140.0]
    }

    pub fn set_counts(&mut self, places: usize, transitions: usize) {
        let old_places = self.places.len();
        if places >= old_places {
            for index in old_places..places {
                self.places.push(Place {
                    id: 0,
                    name: String::new(),
                    pos: Self::default_place_pos(index),
                    ..Default::default()
                });
            }
        } else {
            self.places.truncate(places);
        }

        let old_transitions = self.transitions.len();
        if transitions >= old_transitions {
            for index in old_transitions..transitions {
                self.transitions.push(Transition {
                    id: 0,
                    name: String::new(),
                    pos: Self::default_transition_pos(index),
                    size: VisualSize::Medium,
                    ..Default::default()
                });
            }
        } else {
            self.transitions.truncate(transitions);
        }

        let mut used_place_ids = HashSet::new();
        for (i, place) in self.places.iter_mut().enumerate() {
            if place.id == 0 || !used_place_ids.insert(place.id) {
                place.id = (i + 1) as u64;
                used_place_ids.insert(place.id);
            }
        }
        for place in &mut self.places {
            if place.name.is_empty() {
                place.name = format!("P{}", place.id);
            }
        }

        let mut used_transition_ids = HashSet::new();
        for (i, tr) in self.transitions.iter_mut().enumerate() {
            if tr.id == 0 || !used_transition_ids.insert(tr.id) {
                tr.id = (i + 1) as u64;
                used_transition_ids.insert(tr.id);
            }
        }
        let mut sorted_transition_ids: Vec<u64> = self.transitions.iter().map(|t| t.id).collect();
        sorted_transition_ids.sort_unstable();
        let transition_rank: HashMap<u64, usize> = sorted_transition_ids
            .into_iter()
            .enumerate()
            .map(|(idx, id)| (id, idx + 1))
            .collect();
        for tr in &mut self.transitions {
            if tr.name.is_empty() || Self::is_auto_name(&tr.name, &['T', 't']) {
                if let Some(rank) = transition_rank.get(&tr.id) {
                    tr.name = format!("T{}", rank);
                }
            }
        }

        self.tables.resize(places, transitions);
        self.rebuild_matrices_from_arcs();
    }

    pub fn add_place(&mut self, pos: [f32; 2]) {
        let id = self.next_place_id();
        let idx = self.places.len();
        self.places.push(Place {
            id,
            name: format!("P{}", idx + 1),
            pos,
            ..Default::default()
        });
        self.set_counts(self.places.len(), self.transitions.len());
    }

    pub fn add_transition(&mut self, pos: [f32; 2]) {
        let id = self.next_transition_id();
        let idx = self.transitions.len();
        self.transitions.push(Transition {
            id,
            name: format!("T{}", idx + 1),
            pos,
            size: VisualSize::Medium,
            ..Default::default()
        });
        self.set_counts(self.places.len(), self.transitions.len());
    }

    pub fn add_arc(&mut self, from: NodeRef, to: NodeRef, weight: u32) {
        if matches!(
            (from, to),
            (NodeRef::Place(_), NodeRef::Transition(_))
                | (NodeRef::Transition(_), NodeRef::Place(_))
        ) {
            self.arcs.push(Arc {
                id: self.next_arc_id(),
                from,
                to,
                weight: weight.max(1),
                color: NodeColor::Default,
                visible: true,
                show_weight: false,
            });
            self.rebuild_matrices_from_arcs();
        }
    }

    pub fn add_inhibitor_arc(&mut self, place_id: u64, transition_id: u64, threshold: u32) {
        if self.places.iter().any(|p| p.id == place_id)
            && self.transitions.iter().any(|t| t.id == transition_id)
        {
            self.inhibitor_arcs.push(InhibitorArc {
                id: self.next_inhibitor_id(),
                place_id,
                transition_id,
                threshold: threshold.max(1),
                color: NodeColor::Red,
                visible: true,
                show_weight: false,
            });
            self.rebuild_matrices_from_arcs();
        }
    }

    pub fn place_index_map(&self) -> HashMap<u64, usize> {
        self.places
            .iter()
            .enumerate()
            .map(|(idx, p)| (p.id, idx))
            .collect()
    }

    pub fn transition_index_map(&self) -> HashMap<u64, usize> {
        self.transitions
            .iter()
            .enumerate()
            .map(|(idx, t)| (t.id, idx))
            .collect()
    }

    pub fn rebuild_matrices_from_arcs(&mut self) {
        self.tables
            .resize(self.places.len(), self.transitions.len());

        for p in 0..self.places.len() {
            for t in 0..self.transitions.len() {
                self.tables.pre[p][t] = 0;
                self.tables.post[p][t] = 0;
                self.tables.inhibitor[p][t] = 0;
            }
        }

        let pmap = self.place_index_map();
        let tmap = self.transition_index_map();

        self.arcs.retain(|arc| match (arc.from, arc.to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid))
            | (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                pmap.contains_key(&pid) && tmap.contains_key(&tid)
            }
            _ => false,
        });

        for arc in &self.arcs {
            match (arc.from, arc.to) {
                (NodeRef::Place(pid), NodeRef::Transition(tid)) => {
                    if let (Some(&p), Some(&t)) = (pmap.get(&pid), tmap.get(&tid)) {
                        self.tables.pre[p][t] =
                            self.tables.pre[p][t].saturating_add(arc.weight.max(1));
                    }
                }
                (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                    if let (Some(&p), Some(&t)) = (pmap.get(&pid), tmap.get(&tid)) {
                        self.tables.post[p][t] =
                            self.tables.post[p][t].saturating_add(arc.weight.max(1));
                    }
                }
                _ => {}
            }
        }

        self.inhibitor_arcs
            .retain(|a| pmap.contains_key(&a.place_id) && tmap.contains_key(&a.transition_id));
        for inh in &self.inhibitor_arcs {
            if let (Some(&p), Some(&t)) = (pmap.get(&inh.place_id), tmap.get(&inh.transition_id)) {
                self.tables.inhibitor[p][t] = inh.threshold.max(1);
            }
        }
    }

    pub fn rebuild_arcs_from_matrices(&mut self) {
        self.arcs.clear();
        self.inhibitor_arcs.clear();
        let mut next_id = 1_u64;

        let place_ids: Vec<u64> = self.places.iter().map(|p| p.id).collect();
        let transition_ids: Vec<u64> = self.transitions.iter().map(|t| t.id).collect();

        for (pi, place_id) in place_ids.iter().enumerate() {
            for (ti, tr_id) in transition_ids.iter().enumerate() {
                let pre = self.tables.pre[pi][ti];
                let post = self.tables.post[pi][ti];
                let inh = self.tables.inhibitor[pi][ti];

                if pre > 0 {
                    self.arcs.push(Arc {
                        id: next_id,
                        from: NodeRef::Place(*place_id),
                        to: NodeRef::Transition(*tr_id),
                        weight: pre,
                        color: NodeColor::Default,
                        visible: true,
                        show_weight: false,
                    });
                    next_id = next_id.saturating_add(1);
                }
                if post > 0 {
                    self.arcs.push(Arc {
                        id: next_id,
                        from: NodeRef::Transition(*tr_id),
                        to: NodeRef::Place(*place_id),
                        weight: post,
                        color: NodeColor::Default,
                        visible: true,
                        show_weight: false,
                    });
                    next_id = next_id.saturating_add(1);
                }
                if inh > 0 {
                    self.inhibitor_arcs.push(InhibitorArc {
                        id: next_id,
                        place_id: *place_id,
                        transition_id: *tr_id,
                        threshold: inh.max(1),
                        color: NodeColor::Red,
                        visible: true,
                        show_weight: false,
                    });
                    next_id = next_id.saturating_add(1);
                }
            }
        }
    }

    pub fn sanitize_values(&mut self) {
        for value in &mut self.tables.mz {
            if !value.is_finite() || *value < 0.0 {
                *value = 0.0;
            }
        }
        for cap in &mut self.tables.mo {
            if let Some(inner) = cap {
                if *inner == 0 {
                    *cap = None;
                }
            }
        }
        for arc in &mut self.arcs {
            arc.weight = arc.weight.max(1);
        }
        for inh in &mut self.inhibitor_arcs {
            inh.threshold = inh.threshold.max(1);
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.format_version != GPN2_FORMAT_VERSION {
            return Err(anyhow!(
                "Неподдерживаемая версия формата: {}",
                self.format_version
            ));
        }

        let mut place_ids = HashSet::new();
        for place in &self.places {
            if !place_ids.insert(place.id) {
                return Err(anyhow!("Дублирующийся id места: {}", place.id));
            }
            if !place.pos[0].is_finite() || !place.pos[1].is_finite() {
                return Err(anyhow!("Координаты места {} невалидны", place.id));
            }
        }

        let mut transition_ids = HashSet::new();
        for tr in &self.transitions {
            if !transition_ids.insert(tr.id) {
                return Err(anyhow!("Дублирующийся id перехода: {}", tr.id));
            }
            if !tr.pos[0].is_finite() || !tr.pos[1].is_finite() {
                return Err(anyhow!("Координаты перехода {} невалидны", tr.id));
            }
        }

        for (row_name, matrix) in [
            ("pre", &self.tables.pre),
            ("post", &self.tables.post),
            ("inhibitor", &self.tables.inhibitor),
        ] {
            if matrix.len() != self.places.len() {
                return Err(anyhow!(
                    "Матрица {} имеет некорректное число строк: {} вместо {}",
                    row_name,
                    matrix.len(),
                    self.places.len()
                ));
            }
            for row in matrix {
                if row.len() != self.transitions.len() {
                    return Err(anyhow!(
                        "Матрица {} имеет некорректное число столбцов",
                        row_name
                    ));
                }
            }
        }

        if self.tables.m0.len() != self.places.len()
            || self.tables.mo.len() != self.places.len()
            || self.tables.mz.len() != self.places.len()
            || self.tables.mpr.len() != self.transitions.len()
        {
            return Err(anyhow!(
                "Размеры таблиц не согласованы с числами мест/переходов"
            ));
        }

        for (idx, v) in self.tables.mz.iter().enumerate() {
            if !v.is_finite() || *v < 0.0 {
                return Err(anyhow!("Mz[{}] содержит недопустимое значение", idx));
            }
        }

        for arc in &self.arcs {
            if arc.weight == 0 {
                return Err(anyhow!("Вес дуги {} должен быть > 0", arc.id));
            }
            match (arc.from, arc.to) {
                (NodeRef::Place(p), NodeRef::Transition(t)) => {
                    if !place_ids.contains(&p) || !transition_ids.contains(&t) {
                        return Err(anyhow!(
                            "Дуга {} ссылается на отсутствующие вершины",
                            arc.id
                        ));
                    }
                }
                (NodeRef::Transition(t), NodeRef::Place(p)) => {
                    if !place_ids.contains(&p) || !transition_ids.contains(&t) {
                        return Err(anyhow!(
                            "Дуга {} ссылается на отсутствующие вершины",
                            arc.id
                        ));
                    }
                }
                _ => return Err(anyhow!("Дуга {} нарушает двудольность графа", arc.id)),
            }
        }

        for inh in &self.inhibitor_arcs {
            if inh.threshold == 0 {
                return Err(anyhow!(
                    "Порог ингибиторной дуги {} должен быть > 0",
                    inh.id
                ));
            }
            if !place_ids.contains(&inh.place_id) || !transition_ids.contains(&inh.transition_id) {
                return Err(anyhow!(
                    "Ингибиторная дуга {} ссылается на отсутствующие вершины",
                    inh.id
                ));
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_values_resets_invalid_inputs() {
        let mut net = PetriNet::new();
        net.set_counts(1, 1);
        net.tables.mz[0] = f64::NAN;
        net.tables.mo[0] = Some(0);
        let place_id = net.places[0].id;
        let transition_id = net.transitions[0].id;
        net.arcs.push(Arc {
            id: 1,
            from: NodeRef::Place(place_id),
            to: NodeRef::Transition(transition_id),
            weight: 0,
            color: NodeColor::Default,
            visible: true,
            show_weight: false,
        });
        net.inhibitor_arcs.push(InhibitorArc {
            id: 2,
            place_id,
            transition_id,
            threshold: 0,
            color: NodeColor::Red,
            visible: true,
            show_weight: false,
        });

        net.sanitize_values();

        assert_eq!(net.tables.mz[0], 0.0);
        assert_eq!(net.tables.mo[0], None);
        assert_eq!(net.arcs[0].weight, 1);
        assert_eq!(net.inhibitor_arcs[0].threshold, 1);
    }
}
