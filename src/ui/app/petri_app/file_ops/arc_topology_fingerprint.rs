use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_topology_fingerprint(net: &PetriNet) -> u64 {
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
}
