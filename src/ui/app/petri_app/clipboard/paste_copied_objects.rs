use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn paste_copied_objects(&mut self) {
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
            let rel = [
                cp.place.pos[0] - buf.origin[0],
                cp.place.pos[1] - buf.origin[1],
            ];
            let pos =
                self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

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

                if Self::parse_place_auto_index(&cp.place.name).is_some()
                    || cp.place.name.trim().is_empty()
                {
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
            let pos =
                self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

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

                if Self::parse_transition_auto_index(&ct.transition.name).is_some()
                    || ct.transition.name.trim().is_empty()
                {
                    self.net.transitions[idx].name.clear();
                    self.assign_auto_name_for_transition(new_id);
                } else {
                    let desired = self.net.transitions[idx].name.clone();
                    self.net.transitions[idx].name =
                        self.ensure_unique_transition_name(&desired, new_id);
                }
            }
        }

        let mut new_text_ids = Vec::new();
        for tt in &buf.texts {
            let rel = [tt.pos[0] - buf.origin[0], tt.pos[1] - buf.origin[1]];
            let pos =
                self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let id = self.next_text_id;
            self.next_text_id = self.next_text_id.saturating_add(1);
            self.text_blocks.push(CanvasTextBlock {
                id,
                pos,
                text: tt.text.clone(),
                font_name: tt.font_name.clone(),
                font_size: tt.font_size,
                color: tt.color,
            });
            new_text_ids.push(id);
        }

        for arc in &buf.arcs {
            let remap = |n: NodeRef| -> Option<NodeRef> {
                match n {
                    NodeRef::Place(id) => place_map.get(&id).copied().map(NodeRef::Place),
                    NodeRef::Transition(id) => {
                        transition_map.get(&id).copied().map(NodeRef::Transition)
                    }
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
            let (Some(&pid), Some(&tid)) = (
                place_map.get(&inh.place_id),
                transition_map.get(&inh.transition_id),
            ) else {
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
}
