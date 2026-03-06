use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_place_transition_pair(
        from: NodeRef,
        to: NodeRef,
    ) -> Option<(u64, u64)> {
        match (from, to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid)) => Some((pid, tid)),
            _ => None,
        }
    }
}
