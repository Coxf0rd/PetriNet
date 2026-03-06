use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn stochastic_text(
        dist: &StochasticDistribution,
        is_ru: bool,
    ) -> &'static str {
        match (dist, is_ru) {
            (StochasticDistribution::None, true) => "Нет",
            (StochasticDistribution::Uniform { .. }, true) => "Равномерное",
            (StochasticDistribution::Normal { .. }, true) => "Нормальное (Гаусса)",
            (StochasticDistribution::Exponential { .. }, true) => "Экспоненциальное",
            (StochasticDistribution::Gamma { .. }, true) => "Гамма",
            (StochasticDistribution::Poisson { .. }, true) => "Пуассона",
            (StochasticDistribution::None, false) => "None",
            (StochasticDistribution::Uniform { .. }, false) => "Uniform",
            (StochasticDistribution::Normal { .. }, false) => "Normal (Gaussian)",
            (StochasticDistribution::Exponential { .. }, false) => "Exponential",
            (StochasticDistribution::Gamma { .. }, false) => "Gamma",
            (StochasticDistribution::Poisson { .. }, false) => "Poisson",
        }
    }
}
