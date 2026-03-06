use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn validate_netstar_export(&self) -> NetstarExportValidationReport {
        let mut report = NetstarExportValidationReport::default();

        let place_ids: HashSet<u64> = self.net.places.iter().map(|p| p.id).collect();
        let transition_ids: HashSet<u64> = self.net.transitions.iter().map(|t| t.id).collect();

        if self.net.tables.m0.len() != self.net.places.len()
            || self.net.tables.mo.len() != self.net.places.len()
            || self.net.tables.mz.len() != self.net.places.len()
        {
            report.errors.push(
                self.tr(
                    "Таблицы M0/Mo/Mz имеют неверный размер относительно числа мест.",
                    "M0/Mo/Mz table sizes do not match the places count.",
                )
                .to_string(),
            );
        }
        if self.net.tables.mpr.len() != self.net.transitions.len() {
            report.errors.push(
                self.tr(
                    "Таблица приоритетов переходов (Mpr) имеет неверный размер.",
                    "Mpr table size does not match the transitions count.",
                )
                .to_string(),
            );
        }
        for (name, matrix) in [
            ("Pre", &self.net.tables.pre),
            ("Post", &self.net.tables.post),
            ("Inhibitor", &self.net.tables.inhibitor),
        ] {
            if matrix.len() != self.net.places.len() {
                report.errors.push(format!(
                    "{}: {}",
                    self.tr(
                        "Некорректное число строк в матрице",
                        "Invalid matrix row count"
                    ),
                    name
                ));
                continue;
            }
            if matrix
                .iter()
                .any(|row| row.len() != self.net.transitions.len())
            {
                report.errors.push(format!(
                    "{}: {}",
                    self.tr(
                        "Некорректное число столбцов в матрице",
                        "Invalid matrix column count"
                    ),
                    name
                ));
            }
        }

        for id in Self::duplicate_ids(self.net.places.iter().map(|p| p.id)) {
            report.errors.push(format!(
                "{} P{}",
                self.tr("Дубликат ID позиции:", "Duplicate position ID:"),
                id
            ));
        }
        for id in Self::duplicate_ids(self.net.transitions.iter().map(|t| t.id)) {
            report.errors.push(format!(
                "{} T{}",
                self.tr("Дубликат ID перехода:", "Duplicate transition ID:"),
                id
            ));
        }
        let mut arc_like_ids: Vec<u64> = self.net.arcs.iter().map(|a| a.id).collect();
        arc_like_ids.extend(self.net.inhibitor_arcs.iter().map(|a| a.id));
        for id in Self::duplicate_ids(arc_like_ids) {
            report.errors.push(format!(
                "{} A{}",
                self.tr("Дубликат ID дуги:", "Duplicate arc ID:"),
                id
            ));
        }

        for arc in &self.net.arcs {
            if arc.weight == 0 {
                report.errors.push(format!(
                    "{} A{}",
                    self.tr(
                        "Вес дуги должен быть больше 0:",
                        "Arc weight must be greater than 0:"
                    ),
                    arc.id
                ));
            }
            if arc.weight > 1024 {
                report.warnings.push(format!(
                    "{} A{} ({} -> 1024)",
                    self.tr(
                        "Вес дуги будет ограничен при экспорте:",
                        "Arc weight will be clamped during export:"
                    ),
                    arc.id,
                    arc.weight
                ));
            }
            match (arc.from, arc.to) {
                (NodeRef::Place(place_id), NodeRef::Transition(transition_id))
                | (NodeRef::Transition(transition_id), NodeRef::Place(place_id)) => {
                    if !place_ids.contains(&place_id) || !transition_ids.contains(&transition_id) {
                        report.errors.push(format!(
                            "{} A{}",
                            self.tr(
                                "Дуга ссылается на несуществующую позицию/переход:",
                                "Arc references a missing position/transition:"
                            ),
                            arc.id
                        ));
                    }
                }
                _ => {
                    report.errors.push(format!(
                        "{} A{}",
                        self.tr(
                            "Дуга нарушает двудольность графа:",
                            "Arc breaks graph bipartiteness:"
                        ),
                        arc.id
                    ));
                }
            }
        }

        for inh in &self.net.inhibitor_arcs {
            if inh.threshold == 0 {
                report.errors.push(format!(
                    "{} A{}",
                    self.tr(
                        "Порог ингибиторной дуги должен быть больше 0:",
                        "Inhibitor threshold must be greater than 0:"
                    ),
                    inh.id
                ));
            }
            if inh.threshold > 1024 {
                report.warnings.push(format!(
                    "{} A{} ({} -> 1024)",
                    self.tr(
                        "Порог ингибиторной дуги будет ограничен при экспорте:",
                        "Inhibitor threshold will be clamped during export:"
                    ),
                    inh.id,
                    inh.threshold
                ));
            }
            if !place_ids.contains(&inh.place_id) || !transition_ids.contains(&inh.transition_id) {
                report.errors.push(format!(
                    "{} A{}",
                    self.tr(
                        "Ингибиторная дуга ссылается на несуществующую позицию/переход:",
                        "Inhibitor arc references a missing position/transition:"
                    ),
                    inh.id
                ));
            }
        }

        for (idx, place) in self.net.places.iter().enumerate() {
            let m0 = self.net.tables.m0.get(idx).copied().unwrap_or(0);
            let mo = self.net.tables.mo.get(idx).copied().flatten();
            let mz = self.net.tables.mz.get(idx).copied().unwrap_or(0.0);

            if !place.pos[0].is_finite() || !place.pos[1].is_finite() {
                report.errors.push(format!(
                    "{} P{}",
                    self.tr(
                        "Некорректные координаты позиции:",
                        "Invalid position coordinates:"
                    ),
                    idx + 1
                ));
            } else if place.pos[0] < 0.0
                || place.pos[1] < 0.0
                || place.pos[0] > 65535.0
                || place.pos[1] > 65535.0
            {
                report.warnings.push(format!(
                    "{} P{}",
                    self.tr(
                        "Координаты места могут выйти за диапазон legacy-формата:",
                        "Place coordinates may exceed legacy format limits:"
                    ),
                    idx + 1
                ));
            }

            if let Some(cap) = mo {
                if cap > 1_000_000 {
                    report.warnings.push(format!(
                        "{} P{} ({} -> 1000000)",
                        self.tr(
                            "Максимальная емкость позиции будет ограничена при экспорте:",
                            "Place capacity will be clamped during export:"
                        ),
                        idx + 1,
                        cap
                    ));
                }
            } else {
                report.warnings.push(format!(
                    "{} P{}",
                    self.tr(
                        "Безлимитная емкость позиции не поддерживается, будет заменена на 1:",
                        "Unlimited position capacity is not supported and will be replaced with 1:"
                    ),
                    idx + 1
                ));
            }

            let cap_for_export = mo.unwrap_or(1).clamp(1, 1_000_000);
            if m0 > cap_for_export || m0 > 1_000_000 {
                report.warnings.push(format!(
                    "{} P{}",
                    self.tr(
                        "Число маркеров места будет ограничено при экспорте:",
                        "Place markers count will be clamped during export:"
                    ),
                    idx + 1
                ));
            }

            if !mz.is_finite() {
                report.errors.push(format!(
                    "{} P{}",
                    self.tr(
                        "Задержка места имеет нечисловое значение:",
                        "Place delay has a non-finite value:"
                    ),
                    idx + 1
                ));
            } else if !(0.0..=86_400.0).contains(&mz) {
                report.warnings.push(format!(
                    "{} P{} ({:.3})",
                    self.tr(
                        "Задержка места будет ограничена диапазоном [0; 86400]:",
                        "Place delay will be clamped to [0; 86400]:"
                    ),
                    idx + 1,
                    mz
                ));
            }
        }

        for (idx, transition) in self.net.transitions.iter().enumerate() {
            let mpr = self.net.tables.mpr.get(idx).copied().unwrap_or(1);

            if !transition.pos[0].is_finite() || !transition.pos[1].is_finite() {
                report.errors.push(format!(
                    "{} T{}",
                    self.tr(
                        "Некорректные координаты перехода:",
                        "Invalid transition coordinates:"
                    ),
                    idx + 1
                ));
            } else if transition.pos[0] < 0.0
                || transition.pos[1] < 0.0
                || transition.pos[0] > 65535.0
                || transition.pos[1] > 65535.0
            {
                report.warnings.push(format!(
                    "{} T{}",
                    self.tr(
                        "Координаты перехода могут выйти за диапазон legacy-формата:",
                        "Transition coordinates may exceed legacy format limits:"
                    ),
                    idx + 1
                ));
            }

            if !(0..=1_000_000).contains(&mpr) {
                report.warnings.push(format!(
                    "{} T{} ({} -> диапазон 0..1000000)",
                    self.tr(
                        "Приоритет перехода будет ограничен при экспорте:",
                        "Transition priority will be clamped during export:"
                    ),
                    idx + 1,
                    mpr
                ));
            }

            if transition.angle_deg < -360 || transition.angle_deg > 360 {
                report.warnings.push(format!(
                    "{} T{} ({} -> диапазон -360..360)",
                    self.tr(
                        "Угол перехода будет ограничен при экспорте:",
                        "Transition angle will be clamped during export:"
                    ),
                    idx + 1,
                    transition.angle_deg
                ));
            }
        }

        let non_exportable_items = self.netstar_non_exportable_items();
        if !non_exportable_items.is_empty() {
            report.warnings.push(
                self.tr(
                    "Есть элементы, которые не экспортируются в NetStar.",
                    "There are elements that are not exported to NetStar.",
                )
                .to_string(),
            );
            for item in non_exportable_items {
                report.warnings.push(format!("- {}", item));
            }
        }

        report
    }
}
