use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn copy_selected_objects(&mut self) {
        let mut place_ids = self.collect_selected_place_ids();
        let mut transition_ids = self.collect_selected_transition_ids();
        let text_ids = self.collect_selected_text_ids();

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
                    font_name: self.text_blocks[idx].font_name.clone(),
                    font_size: self.text_blocks[idx].font_size,
                    color: self.text_blocks[idx].color,
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
                    show_weight: arc.show_weight,
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
                    show_weight: inh.show_weight,
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

        let copied_count = place_ids.len()
            + transition_ids.len()
            + text_ids.len()
            + copied_arcs.len()
            + copied_inhibitors.len();
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
}
