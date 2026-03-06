use std::collections::HashSet;

use super::*;

impl PetriApp {
    pub(super) fn draw_graph_view(&mut self, ui: &mut egui::Ui) {
        self.update_debug_animation_clock(ui.ctx());
        ui.heading("Граф");
        let desired = ui.available_size_before_wrap();
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > f32::EPSILON {
            self.canvas.zoom = (self.canvas.zoom * zoom_delta).clamp(0.2, 3.0);
        }

        if response.dragged_by(egui::PointerButton::Middle) {
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
                    Tool::Place | Tool::Transition | Tool::Arc | Tool::Frame => {
                        egui::CursorIcon::Crosshair
                    }
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
                            if let Some(idx) = self.place_idx_by_id(new_id) {
                                self.net.places[idx].size = self.new_place_size;
                                self.net.places[idx].color = self.new_place_color;
                                if idx < self.net.tables.m0.len() {
                                    self.net.tables.m0[idx] = self.new_place_marking;
                                }
                                if idx < self.net.tables.mo.len() {
                                    self.net.tables.mo[idx] = self.new_place_capacity;
                                }
                                if idx < self.net.tables.mz.len() {
                                    self.net.tables.mz[idx] = self.new_place_delay.max(0.0);
                                }
                            }
                        }
                    }
                    Tool::Transition => {
                        // Store transition position as top-left.
                        // Snap the top-left to the grid (not the center) so the rectangle aligns with the grid.
                        self.push_undo_snapshot();
                        let dims = Self::transition_dimensions(self.new_transition_size);
                        let tl =
                            self.snapped_world([world[0] - dims.x * 0.5, world[1] - dims.y * 0.5]);
                        self.net.add_transition(tl);
                        if let Some(new_id) = self.net.transitions.iter().map(|t| t.id).max() {
                            if let Some(idx) = self.transition_idx_by_id(new_id) {
                                self.net.transitions[idx].size = self.new_transition_size;
                                self.net.transitions[idx].color = self.new_transition_color;
                                if idx < self.net.tables.mpr.len() {
                                    self.net.tables.mpr[idx] = self.new_transition_priority;
                                }
                            }
                        }
                    }
                    Tool::Arc => {}
                    Tool::Text => {
                        self.push_undo_snapshot();
                        let id = self.next_text_id;
                        self.next_text_id = self.next_text_id.saturating_add(1);
                        self.text_blocks.push(CanvasTextBlock {
                            id,
                            pos: snapped,
                            text: self
                                .tr("\u{422}\u{435}\u{43A}\u{441}\u{442}", "Text")
                                .to_string(),
                            font_name: "MS Sans Serif".to_string(),
                            font_size: 10.0,
                            color: NodeColor::Default,
                        });
                        self.clear_selection();
                        self.canvas.selected_text = Some(id);
                        self.text_props_id = Some(id);
                        self.show_text_props = true;
                        self.show_place_props = false;
                        self.show_transition_props = false;
                    }
                    Tool::Frame => {}
                    Tool::Delete => {
                        if let Some(node) = self.node_at(rect, click) {
                            self.push_undo_snapshot();
                            match node {
                                NodeRef::Place(p) => {
                                    if let Some(idx) = self.place_idx_by_id(p) {
                                        self.net.tables.remove_place_row(idx);
                                        self.net.places.remove(idx);
                                        self.net.set_counts(
                                            self.net.places.len(),
                                            self.net.transitions.len(),
                                        );
                                    }
                                }
                                NodeRef::Transition(t) => {
                                    if let Some(idx) = self.transition_idx_by_id(t) {
                                        self.net.tables.remove_transition_column(idx);
                                        self.net.transitions.remove(idx);
                                        self.net.set_counts(
                                            self.net.places.len(),
                                            self.net.transitions.len(),
                                        );
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
                        let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
                        if shift_pressed {
                            self.promote_single_selection_to_multi();
                            if let Some(text_id) = self.text_at(rect, click) {
                                let added = Self::toggle_selected_id(
                                    &mut self.canvas.selected_texts,
                                    text_id,
                                );
                                self.canvas.selected_frames.clear();
                                self.canvas.selected_frame = None;
                                self.canvas.selected_text = if added {
                                    Some(text_id)
                                } else {
                                    self.canvas.selected_texts.last().copied()
                                };
                            } else if let Some(frame_id) = self.frame_at(rect, click) {
                                let added = Self::toggle_selected_id(
                                    &mut self.canvas.selected_frames,
                                    frame_id,
                                );
                                self.canvas.selected_texts.clear();
                                self.canvas.selected_text = None;
                                self.canvas.selected_frame = if added {
                                    Some(frame_id)
                                } else {
                                    self.canvas.selected_frames.last().copied()
                                };
                            } else if let Some(n) = self.node_at(rect, click) {
                                match n {
                                    NodeRef::Place(p) => {
                                        Self::toggle_selected_id(
                                            &mut self.canvas.selected_places,
                                            p,
                                        );
                                    }
                                    NodeRef::Transition(t) => {
                                        Self::toggle_selected_id(
                                            &mut self.canvas.selected_transitions,
                                            t,
                                        );
                                    }
                                }
                                self.canvas.selected_text = None;
                                self.canvas.selected_texts.clear();
                                self.canvas.selected_frame = None;
                                self.canvas.selected_frames.clear();
                            } else if let Some(arc_id) = self.arc_at(rect, click) {
                                Self::toggle_selected_id(&mut self.canvas.selected_arcs, arc_id);
                                self.canvas.selected_text = None;
                                self.canvas.selected_texts.clear();
                                self.canvas.selected_frame = None;
                                self.canvas.selected_frames.clear();
                            }
                            self.sync_primary_selection_from_multi();
                        } else {
                            self.clear_selection();
                            if let Some(text_id) = self.text_at(rect, click) {
                                self.canvas.selected_text = Some(text_id);
                            } else if let Some(frame_id) = self.frame_at(rect, click) {
                                self.canvas.selected_frame = Some(frame_id);
                            } else if let Some(n) = self.node_at(rect, click) {
                                match n {
                                    NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                                    NodeRef::Transition(t) => {
                                        self.canvas.selected_transition = Some(t)
                                    }
                                }
                            } else if let Some(arc_id) = self.arc_at(rect, click) {
                                self.canvas.selected_arc = Some(arc_id);
                                self.canvas.selected_arcs.clear();
                                self.canvas.selected_arcs.push(arc_id);
                            }
                        }
                    }
                    Tool::Run => {}
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
                if let Some(pointer) = response
                    .interact_pointer_pos()
                    .or_else(|| response.hover_pos())
                {
                    if let Some(last) = self.node_at(rect, pointer) {
                        if first != last {
                            self.push_undo_snapshot();
                            if self.new_arc_inhibitor {
                                let pair = match (first, last) {
                                    (NodeRef::Place(pid), NodeRef::Transition(tid)) => {
                                        Some((pid, tid))
                                    }
                                    (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                                        Some((pid, tid))
                                    }
                                    _ => None,
                                };
                                if let Some((place_id, transition_id)) = pair {
                                    self.net.add_inhibitor_arc(
                                        place_id,
                                        transition_id,
                                        self.new_arc_inhibitor_threshold.max(1),
                                    );
                                    if let Some(last_inh) = self.net.inhibitor_arcs.last_mut() {
                                        last_inh.color = self.new_arc_color;
                                        last_inh.visible = true;
                                    }
                                }
                            } else {
                                self.net.add_arc(first, last, self.new_arc_weight.max(1));
                                if let Some(last_arc) = self.net.arcs.last_mut() {
                                    last_arc.color = self.new_arc_color;
                                    last_arc.visible = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.tool == Tool::Arc && !ui.ctx().input(|i| i.pointer.any_down()) {
            self.canvas.arc_start = None;
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
                self.canvas.frame_draw_current_world =
                    Some(self.snapped_world(self.screen_to_world(rect, pointer)));
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
                        let handle =
                            self.frame_resize_handle_rect(rect, &self.decorative_frames[idx]);
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
                    let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
                    if shift_pressed {
                        self.promote_single_selection_to_multi();
                        self.canvas.selection_toggle_mode = true;
                        self.canvas.selection_start = Some(pointer);
                        self.canvas.selection_rect = Some(Rect::from_two_pos(pointer, pointer));
                        self.canvas.drag_prev_world = None;
                        self.canvas.move_drag_active = false;
                    } else if let Some(node) = self.node_at(rect, pointer) {
                        let is_selected = match node {
                            NodeRef::Place(p) => {
                                self.canvas.selected_place == Some(p)
                                    || self.canvas.selected_places.contains(&p)
                            }
                            NodeRef::Transition(t) => {
                                self.canvas.selected_transition == Some(t)
                                    || self.canvas.selected_transitions.contains(&t)
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
                        self.canvas.selection_toggle_mode = false;
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
                            let move_place_ids: Vec<u64> = if self.canvas.selected_places.is_empty()
                            {
                                self.canvas.selected_place.into_iter().collect()
                            } else {
                                self.canvas.selected_places.clone()
                            };
                            let move_transition_ids: Vec<u64> =
                                if self.canvas.selected_transitions.is_empty() {
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
                            for text_id in self.collect_selected_text_ids() {
                                if let Some(idx) = self.text_idx_by_id(text_id) {
                                    self.text_blocks[idx].pos[0] += dx;
                                    self.text_blocks[idx].pos[1] += dy;
                                }
                            }
                            for frame_id in self.collect_selected_frame_ids() {
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
                for text_id in self.collect_selected_text_ids() {
                    if let Some(idx) = self.text_idx_by_id(text_id) {
                        self.text_blocks[idx].pos[0] = snap(self.text_blocks[idx].pos[0]);
                        self.text_blocks[idx].pos[1] = snap(self.text_blocks[idx].pos[1]);
                    }
                }
                for frame_id in self.collect_selected_frame_ids() {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        self.decorative_frames[idx].pos[0] =
                            snap(self.decorative_frames[idx].pos[0]);
                        self.decorative_frames[idx].pos[1] =
                            snap(self.decorative_frames[idx].pos[1]);
                    }
                }
            }
            if let Some(sel_rect) = self.canvas.selection_rect.take() {
                let norm = sel_rect.expand2(Vec2::ZERO);
                let hit_places: Vec<u64> = self
                    .net
                    .places
                    .iter()
                    .filter(|p| norm.contains(self.world_to_screen(rect, p.pos)))
                    .map(|p| p.id)
                    .collect();
                let hit_transitions: Vec<u64> = self
                    .net
                    .transitions
                    .iter()
                    .filter(|t| {
                        let pos = self.world_to_screen(rect, t.pos);
                        let tr_rect = Rect::from_min_size(
                            pos,
                            Self::transition_dimensions(t.size) * self.canvas.zoom,
                        );
                        norm.intersects(tr_rect)
                    })
                    .map(|t| t.id)
                    .collect();
                let mut hit_arcs: Vec<u64> = self
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
                hit_arcs.extend(selected_inhibitor_ids);
                let hit_text_ids: Vec<u64> = self
                    .text_blocks
                    .iter()
                    .filter(|text| norm.contains(self.world_to_screen(rect, text.pos)))
                    .map(|text| text.id)
                    .collect();
                let hit_frame_ids: Vec<u64> = self
                    .decorative_frames
                    .iter()
                    .filter(|frame| {
                        let min = self.world_to_screen(rect, frame.pos);
                        let size = Vec2::new(
                            frame.width.max(Self::FRAME_MIN_SIDE),
                            frame.height.max(Self::FRAME_MIN_SIDE),
                        ) * self.canvas.zoom;
                        let frame_rect = Rect::from_min_size(min, size);
                        norm.intersects(frame_rect)
                    })
                    .map(|frame| frame.id)
                    .collect();

                if self.canvas.selection_toggle_mode {
                    self.promote_single_selection_to_multi();
                    for place_id in hit_places {
                        Self::toggle_selected_id(&mut self.canvas.selected_places, place_id);
                    }
                    for transition_id in hit_transitions {
                        Self::toggle_selected_id(
                            &mut self.canvas.selected_transitions,
                            transition_id,
                        );
                    }
                    for arc_id in hit_arcs {
                        Self::toggle_selected_id(&mut self.canvas.selected_arcs, arc_id);
                    }
                    for text_id in hit_text_ids {
                        Self::toggle_selected_id(&mut self.canvas.selected_texts, text_id);
                    }
                    for frame_id in hit_frame_ids {
                        Self::toggle_selected_id(&mut self.canvas.selected_frames, frame_id);
                    }
                    self.sync_primary_selection_from_multi();
                } else {
                    self.canvas.selected_places = hit_places;
                    self.canvas.selected_transitions = hit_transitions;
                    self.canvas.selected_arcs = hit_arcs;
                    self.canvas.selected_texts = hit_text_ids;
                    self.canvas.selected_frames = hit_frame_ids;
                    self.canvas.selected_place = None;
                    self.canvas.selected_transition = None;
                    self.canvas.selected_arc = self.canvas.selected_arcs.first().copied();
                    self.canvas.selected_text = self.canvas.selected_texts.first().copied();
                    self.canvas.selected_frame = self.canvas.selected_frames.first().copied();
                }
                self.canvas.selection_toggle_mode = false;
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
                            self.show_text_props = false;
                        }
                        NodeRef::Transition(t) => {
                            self.canvas.selected_transition = Some(t);
                            self.transition_props_id = Some(t);
                            self.show_transition_props = true;
                            self.show_place_props = false;
                            self.show_text_props = false;
                        }
                    }
                } else if let Some(text_id) = self.text_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_text = Some(text_id);
                    self.text_props_id = Some(text_id);
                    self.show_text_props = true;
                    self.show_place_props = false;
                    self.show_transition_props = false;
                } else if let Some(arc_id) = self.arc_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_arc = Some(arc_id);
                    self.canvas.selected_arcs.clear();
                    self.canvas.selected_arcs.push(arc_id);
                    self.arc_props_id = Some(arc_id);
                    self.show_arc_props = true;
                    self.show_place_props = false;
                    self.show_transition_props = false;
                    self.show_text_props = false;
                } else if let Some(frame_id) = self.frame_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_frame = Some(frame_id);
                    self.show_text_props = false;
                }
            }
        }

        if let Some(sel) = self.canvas.selection_rect {
            painter.rect_stroke(sel, 0.0, Stroke::new(1.0, Color32::from_rgb(70, 120, 210)));
            painter.rect_filled(sel, 0.0, Color32::from_rgba_premultiplied(70, 120, 210, 25));
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
        let active_event = if self.show_debug && self.debug_animation_enabled {
            self.debug_animation_active_event
                .and_then(|idx| self.debug_animation_events.get(idx))
        } else {
            None
        };
        let (active_pre_arc_ids, active_post_arc_ids) = if let Some(event) = active_event {
            (
                event
                    .pre_arcs
                    .iter()
                    .map(|arc| arc.arc_id)
                    .collect::<HashSet<_>>(),
                event
                    .post_arcs
                    .iter()
                    .map(|arc| arc.arc_id)
                    .collect::<HashSet<_>>(),
            )
        } else {
            (HashSet::new(), HashSet::new())
        };

        for arc in &self.net.arcs {
            if !self.arc_visible_by_mode(arc.color, arc.visible) {
                continue;
            }
            let (from_center, from_radius, from_rect, to_center, to_radius, to_rect) =
                match (arc.from, arc.to) {
                    (NodeRef::Place(p), NodeRef::Transition(t)) => {
                        if let (Some(pi), Some(ti)) =
                            (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                        {
                            let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                            let p_radius =
                                Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                            let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                            let t_rect = Rect::from_min_size(
                                t_min,
                                Self::transition_dimensions(self.net.transitions[ti].size)
                                    * self.canvas.zoom,
                            );
                            (
                                p_center,
                                Some(p_radius),
                                None,
                                t_rect.center(),
                                None,
                                Some(t_rect),
                            )
                        } else {
                            continue;
                        }
                    }
                    (NodeRef::Transition(t), NodeRef::Place(p)) => {
                        if let (Some(pi), Some(ti)) =
                            (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                        {
                            let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                            let t_rect = Rect::from_min_size(
                                t_min,
                                Self::transition_dimensions(self.net.transitions[ti].size)
                                    * self.canvas.zoom,
                            );
                            let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                            let p_radius =
                                Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                            (
                                t_rect.center(),
                                None,
                                Some(t_rect),
                                p_center,
                                Some(p_radius),
                                None,
                            )
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

            let mut from = from_center;
            let mut to = to_center;
            let delta = to_center - from_center;
            let dir = if delta.length_sq() > 0.0 {
                delta.normalized()
            } else {
                Vec2::X
            };

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
            let mut arc_stroke = if self.canvas.selected_arc == Some(arc.id)
                || self.canvas.selected_arcs.contains(&arc.id)
            {
                Stroke::new(3.0, Color32::from_rgb(255, 140, 0))
            } else {
                Stroke::new(2.0, arc_color)
            };
            if self.debug_arc_animation
                && self.debug_animation_enabled
                && self.canvas.selected_arc != Some(arc.id)
                && !self.canvas.selected_arcs.contains(&arc.id)
            {
                let is_pre_arc = active_pre_arc_ids.contains(&arc.id);
                let is_post_arc = active_post_arc_ids.contains(&arc.id);
                if is_pre_arc || is_post_arc {
                    if let Some(event) = active_event {
                        let highlight_color = if is_pre_arc {
                            event.entry_color
                        } else {
                            event.exit_color
                        };
                        arc_stroke = Stroke::new(3.0, highlight_color);
                    }
                }
            }
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
                let t_rect = Rect::from_min_size(
                    t_min,
                    Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom,
                );
                let t_center = t_rect.center();
                let delta = t_center - p_center;
                let dir = if delta.length_sq() > 0.0 {
                    delta.normalized()
                } else {
                    Vec2::X
                };
                let from = p_center + dir * p_radius;
                let to = Self::rect_border_point(t_rect, -dir);
                let inh_color = Self::color_to_egui(inh.color, Color32::RED);
                let inh_stroke = if self.canvas.selected_arc == Some(inh.id)
                    || self.canvas.selected_arcs.contains(&inh.id)
                {
                    Stroke::new(3.0, Color32::from_rgb(255, 140, 0))
                } else {
                    Stroke::new(1.5, inh_color)
                };
                painter.line_segment([from, to], inh_stroke);
                let mid = from + (to - from) * 0.5;
                if inh.show_weight {
                    let multiplicity_label = self.tr("Кратность", "Multiplicity");
                    painter.text(
                        mid,
                        egui::Align2::CENTER_CENTER,
                        format!("{multiplicity_label}: {}", inh.threshold),
                        egui::TextStyle::Small.resolve(ui.style()),
                        Self::color_to_egui(inh.color, Color32::RED),
                    );
                }
            }
        }

        let debug_marking = if self.show_debug {
            self.sim_result
                .as_ref()
                .and_then(|res| {
                    let visible = Self::debug_visible_log_indices(res);
                    visible
                        .get(self.debug_step)
                        .and_then(|&log_idx| res.logs.get(log_idx))
                        .map(|entry| entry.marking.clone())
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let debug_place_colors = self
            .debug_place_colors
            .get(self.debug_step)
            .cloned()
            .unwrap_or_else(|| Vec::new());

        for (place_idx, place) in self.net.places.iter().enumerate() {
            let center = self.world_to_screen(rect, place.pos);
            let radius = Self::place_radius(place.size) * self.canvas.zoom;
            let place_color = Self::color_to_egui(place.color, Color32::BLACK);
            let is_selected = self.canvas.selected_place == Some(place.id)
                || self.canvas.selected_places.contains(&place.id);
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(
                    if is_selected { 3.0 } else { 2.0 },
                    if is_selected {
                        Color32::from_rgb(255, 140, 0)
                    } else {
                        place_color
                    },
                ),
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

            let (tokens, token_colors) = if self.show_debug {
                (
                    debug_place_colors
                        .get(place_idx)
                        .map(|colors| colors.len() as u32)
                        .unwrap_or_else(|| {
                            debug_marking.get(place_idx).copied().unwrap_or_else(|| {
                                self.net.tables.m0.get(place_idx).copied().unwrap_or(0)
                            })
                        }),
                    debug_place_colors
                        .get(place_idx)
                        .cloned()
                        .unwrap_or_else(|| Vec::new()),
                )
            } else {
                (
                    self.net.tables.m0.get(place_idx).copied().unwrap_or(0),
                    Vec::new(),
                )
            };
            if tokens > 0 {
                if self.show_debug {
                    if tokens > 5 {
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            format!("{tokens}"),
                            egui::TextStyle::Body.resolve(ui.style()),
                            token_colors
                                .get(0)
                                .copied()
                                .unwrap_or(Color32::from_rgb(200, 0, 0)),
                        );
                    } else {
                        let draw_tokens = tokens as usize;
                        for i in 0..draw_tokens {
                            let angle =
                                (i as f32) * std::f32::consts::TAU / (draw_tokens.max(1) as f32);
                            let dot_pos =
                                center + Vec2::new(angle.cos(), angle.sin()) * (radius * 0.55);
                            let color = token_colors
                                .get(i)
                                .copied()
                                .unwrap_or(Color32::from_rgb(200, 0, 0));
                            painter.circle_filled(
                                dot_pos,
                                3.0 * self.canvas.zoom.clamp(0.7, 1.2),
                                color,
                            );
                        }
                    }
                } else if tokens <= 4 {
                    let draw_tokens = tokens;
                    for i in 0..draw_tokens {
                        let angle =
                            (i as f32) * std::f32::consts::TAU / (draw_tokens.max(1) as f32);
                        let dot_pos =
                            center + Vec2::new(angle.cos(), angle.sin()) * (radius * 0.55);
                        painter.circle_filled(
                            dot_pos,
                            3.0 * self.canvas.zoom.clamp(0.7, 1.2),
                            Color32::from_rgb(200, 0, 0),
                        );
                    }
                } else {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("{tokens}"),
                        egui::TextStyle::Body.resolve(ui.style()),
                        Color32::from_rgb(200, 0, 0),
                    );
                }
            }
            if let Some(annotation) = self.markov_annotations.get(&place.id) {
                let annotation_offset = Vec2::new(0.0, radius + 8.0);
                painter.text(
                    center + annotation_offset,
                    egui::Align2::CENTER_TOP,
                    annotation,
                    egui::TextStyle::Small.resolve(ui.style()),
                    Color32::from_rgb(100, 100, 100),
                );
            }
        }

        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let dims = Self::transition_dimensions(tr.size) * self.canvas.zoom;
            let r = Rect::from_min_size(p, dims);
            let tr_color = Self::color_to_egui(tr.color, Color32::BLACK);
            let is_selected = self.canvas.selected_transition == Some(tr.id)
                || self.canvas.selected_transitions.contains(&tr.id);
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(
                    if is_selected { 3.0 } else { 2.0 },
                    if is_selected {
                        Color32::from_rgb(255, 140, 0)
                    } else {
                        tr_color
                    },
                ),
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
            let draw_color = if self.canvas.selected_text == Some(text.id) {
                Color32::from_rgb(255, 140, 0)
            } else {
                Self::color_to_egui(text.color, Color32::from_rgb(40, 40, 40))
            };
            let family = Self::text_family_from_name(&text.font_name);
            let font_id = egui::FontId::new(text.font_size.max(6.0) * self.canvas.zoom, family);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                &text.text,
                font_id,
                draw_color,
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
                        self.tr("\u{422}\u{435}\u{43A}\u{441}\u{442}", "Text"),
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
                            Rect::from_min_size(
                                min,
                                Self::transition_dimensions(self.net.transitions[ti].size)
                                    * self.canvas.zoom,
                            )
                            .center()
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

        self.draw_debug_animation_overlay(rect, &painter);

        if let Some(p) = self.canvas.selected_place {
            if let Some(idx) = self.place_idx_by_id(p) {
                let place = &mut self.net.places[idx];
                ui.separator();
                ui.label("Выбранная позиция");
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

    fn update_debug_animation_clock(&mut self, ctx: &egui::Context) {
        if !self.show_debug || !self.debug_animation_enabled {
            self.debug_animation_last_update = None;
            return;
        }
        if self.debug_animation_events.is_empty() {
            self.debug_animation_last_update = None;
            return;
        }
        if !self.debug_animation_step_active && !self.debug_playing {
            self.debug_animation_last_update = None;
            return;
        }
        let now = Instant::now();
        let delta = if let Some(last) = self.debug_animation_last_update {
            now.duration_since(last).as_secs_f64()
        } else {
            0.0
        };
        self.debug_animation_last_update = Some(now);
        let speed = self.debug_animation_playback_speed();
        self.debug_animation_local_clock += delta * speed;
        let duration = self
            .debug_animation_current_duration
            .max(Self::DEBUG_ANIMATION_MIN_DURATION);
        if self.debug_animation_local_clock >= duration {
            self.debug_animation_local_clock = duration;
            if self.debug_playing {
                let visible_len = self
                    .sim_result
                    .as_ref()
                    .map(|result| Self::debug_visible_log_indices(result).len())
                    .unwrap_or(0);
                if self.debug_step + 1 < visible_len {
                    self.debug_step += 1;
                    self.sync_debug_animation_for_step();
                    return;
                }
                self.debug_playing = false;
            }
            self.debug_animation_step_active = false;
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn draw_debug_animation_overlay(&self, rect: Rect, painter: &egui::Painter) {
        if !self.show_debug || !self.debug_animation_enabled {
            return;
        }
        let event_idx = match self.debug_animation_active_event {
            Some(idx) => idx,
            None => return,
        };
        let event = match self.debug_animation_events.get(event_idx) {
            Some(event) => event,
            None => return,
        };
        let relative = self.debug_animation_relative(event);
        self.draw_debug_animation_event(event, relative, rect, painter);
    }

    fn debug_animation_relative(&self, _event: &DebugAnimationEvent) -> f32 {
        let duration = self
            .debug_animation_current_duration
            .max(Self::DEBUG_ANIMATION_MIN_DURATION);
        if duration <= 0.0 {
            return 0.0;
        }
        (self.debug_animation_local_clock / duration).clamp(0.0, 1.0) as f32
    }

    fn draw_debug_animation_event(
        &self,
        event: &DebugAnimationEvent,
        relative: f32,
        rect: Rect,
        painter: &egui::Painter,
    ) {
        const PRE_FRACTION: f32 = 0.35;
        const TRANSITION_FRACTION: f32 = 0.2;
        const POST_FRACTION: f32 = 1.0 - PRE_FRACTION - TRANSITION_FRACTION;
        let transition = match self.net.transitions.get(event.transition_idx) {
            Some(tr) => tr,
            None => return,
        };
        let tr_pos = self.world_to_screen(rect, transition.pos);
        let tr_dims = Self::transition_dimensions(transition.size) * self.canvas.zoom;
        let tr_rect = Rect::from_min_size(tr_pos, tr_dims);
        let tr_center = tr_rect.center();
        let entry_color = event.entry_color;
        let exit_color = event.exit_color;
        let transition_token_color = event
            .color_change_place_idx
            .and_then(|idx| self.net.places.get(idx))
            .map(|place| Self::color_to_egui(place.color, entry_color))
            .unwrap_or(exit_color);
        let token_radius = 4.0 * self.canvas.zoom;
        let token_spacing = token_radius * 2.2;

        if relative < PRE_FRACTION {
            if self.debug_arc_animation {
                let progress = (relative / PRE_FRACTION).clamp(0.0, 1.0);
                self.draw_debug_animation_tokens_along_arcs(
                    &event.pre_arcs,
                    rect,
                    painter,
                    tr_rect,
                    tr_center,
                    progress,
                    token_radius,
                    token_spacing,
                    true,
                    entry_color,
                );
            }
            return;
        }
        if relative < PRE_FRACTION + TRANSITION_FRACTION {
            let progress = ((relative - PRE_FRACTION) / TRANSITION_FRACTION).clamp(0.0, 1.0);
            let count = event
                .pre_arcs
                .iter()
                .map(|arc| arc.weight as usize)
                .sum::<usize>()
                .max(1)
                .min(4);
            let angle_offset = progress * std::f32::consts::TAU;
            let radius = token_spacing * 0.35;
            for i in 0..count {
                let angle = (i as f32) * (std::f32::consts::TAU / count as f32) + angle_offset;
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                painter.circle_filled(tr_center + offset, token_radius, transition_token_color);
            }
            return;
        }
        let post_progress =
            ((relative - PRE_FRACTION - TRANSITION_FRACTION) / POST_FRACTION).clamp(0.0, 1.0);
        if self.debug_arc_animation {
            self.draw_debug_animation_tokens_along_arcs(
                &event.post_arcs,
                rect,
                painter,
                tr_rect,
                tr_center,
                post_progress,
                token_radius,
                token_spacing,
                false,
                exit_color,
            );
        }
    }

    fn draw_debug_animation_tokens_along_arcs(
        &self,
        arcs: &[DebugAnimationArc],
        rect: Rect,
        painter: &egui::Painter,
        tr_rect: Rect,
        tr_center: Pos2,
        progress: f32,
        token_radius: f32,
        token_spacing: f32,
        toward_transition: bool,
        token_color: Color32,
    ) {
        for arc in arcs {
            if arc.weight == 0 {
                continue;
            }
            let place = match self.net.places.get(arc.place_idx) {
                Some(place) => place,
                None => continue,
            };
            let place_center = self.world_to_screen(rect, place.pos);
            let place_radius = Self::place_radius(place.size) * self.canvas.zoom;
            let dir = if toward_transition {
                Self::normalized_direction(tr_center - place_center)
            } else {
                Self::normalized_direction(place_center - tr_center)
            };
            let (start, end) = if toward_transition {
                (
                    place_center + dir * place_radius,
                    Self::rect_border_point(tr_rect, -dir),
                )
            } else {
                (
                    Self::rect_border_point(tr_rect, dir),
                    place_center - dir * place_radius,
                )
            };
            let perp = Vec2::new(-dir.y, dir.x);
            let count = (arc.weight as usize).min(3).max(1);
            let offset_base = (count as f32 - 1.0) * 0.5;
            let travel = start + (end - start) * progress;
            for i in 0..count {
                let offset = perp * token_spacing * (i as f32 - offset_base);
                let color = arc.token_colors.get(i).copied().unwrap_or(token_color);
                painter.circle_filled(travel + offset, token_radius, color);
            }
        }
    }

    fn normalized_direction(delta: Vec2) -> Vec2 {
        if delta.length_sq() < f32::EPSILON {
            Vec2::X
        } else {
            delta.normalized()
        }
    }
}
