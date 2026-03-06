use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sampled_indices(total: usize, max_points: usize) -> Vec<usize> {
        if total == 0 {
            return Vec::new();
        }
        if max_points <= 1 || total <= max_points {
            return (0..total).collect();
        }

        let mut out = Vec::with_capacity(max_points);
        let last_idx = total - 1;
        let step = last_idx as f64 / (max_points - 1) as f64;
        for i in 0..max_points {
            let mut idx = (i as f64 * step).round() as usize;
            if idx > last_idx {
                idx = last_idx;
            }
            if out.last().copied() != Some(idx) {
                out.push(idx);
            }
        }
        if out.last().copied() != Some(last_idx) {
            out.push(last_idx);
        }
        out
    }
}
