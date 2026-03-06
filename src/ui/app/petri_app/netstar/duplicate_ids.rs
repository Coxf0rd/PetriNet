use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn duplicate_ids<I>(ids: I) -> Vec<u64>
    where
        I: IntoIterator<Item = u64>,
    {
        let mut counts: HashMap<u64, usize> = HashMap::new();
        for id in ids {
            *counts.entry(id).or_insert(0) += 1;
        }
        let mut duplicates: Vec<u64> = counts
            .into_iter()
            .filter_map(|(id, count)| (count > 1).then_some(id))
            .collect();
        duplicates.sort_unstable();
        duplicates
    }
}
