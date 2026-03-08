use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_place_props_window(
        &mut self,
        ctx: &egui::Context,
        place_id: u64,
        title: String,
    ) -> bool {
        let Some(place_idx) = self.place_idx_by_id(place_id) else {
            return false;
        };
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };
        let mut open = true;
        show_property_window(
            ctx,
            title,
            &mut open,
            PropertyWindowConfig::new("place_props_window"),
            |ui: &mut egui::Ui| {
                let mut corrected_inputs = false;
                ui.label(format!("ID: P{}", place_id));
                ui.separator();

                let mut markers = self.net.tables.m0[place_idx];
                corrected_inputs |= sanitize_u32(&mut markers, 0, u32::MAX);
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(t("Число маркеров", "Markers"));
                    if ui
                        .add(egui::DragValue::new(&mut markers).range(0..=u32::MAX))
                        .changed()
                    {
                        corrected_inputs |= sanitize_u32(&mut markers, 0, u32::MAX);
                    }
                });
                self.net.tables.m0[place_idx] = markers;

                let mut cap = self.net.tables.mo[place_idx].unwrap_or(0);
                corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(t(
                        "Макс. емкость (0 = без ограничений)",
                        "Capacity (0 = unlimited)",
                    ));
                    if ui
                        .add(egui::DragValue::new(&mut cap).range(0..=u32::MAX))
                        .changed()
                    {
                        corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                    }
                });
                self.net.tables.mo[place_idx] = if cap == 0 { None } else { Some(cap) };

                let mut delay = self.net.tables.mz[place_idx];
                corrected_inputs |= sanitize_f64(&mut delay, 0.0, 10_000.0);
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(t("Время задержки (сек)", "Delay (sec)"));
                    if ui
                        .add(
                            egui::DragValue::new(&mut delay)
                                .speed(0.1)
                                .range(0.0..=10_000.0),
                        )
                        .changed()
                    {
                        corrected_inputs |= sanitize_f64(&mut delay, 0.0, 10_000.0);
                    }
                });
                self.net.tables.mz[place_idx] = delay;

                ui.separator();
                ui.label(t("Размер позиции", "Place size"));
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.radio_value(
                        &mut self.net.places[place_idx].size,
                        VisualSize::Small,
                        t("Малый", "Small"),
                    );
                    ui.radio_value(
                        &mut self.net.places[place_idx].size,
                        VisualSize::Medium,
                        t("Средний", "Medium"),
                    );
                    ui.radio_value(
                        &mut self.net.places[place_idx].size,
                        VisualSize::Large,
                        t("Большой", "Large"),
                    );
                });

                egui::ComboBox::from_label(t("Положение метки", "Marker label position"))
                    .selected_text(Self::label_pos_text(
                        self.net.places[place_idx].marker_label_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Положение текста", "Text position"))
                    .selected_text(Self::label_pos_text(
                        self.net.places[place_idx].text_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Цвет", "Color"))
                    .selected_text(Self::node_color_text(
                        self.net.places[place_idx].color,
                        is_ru,
                    ))
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Default,
                            t("По умолчанию", "Default"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Blue,
                            t("Синий", "Blue"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Red,
                            t("Красный", "Red"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Green,
                            t("Зеленый", "Green"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Yellow,
                            t("Желтый", "Yellow"),
                        );
                    });

                ui.separator();
                ui.checkbox(
                    &mut self.net.places[place_idx].marker_color_on_pass,
                    t(
                        "Изменять цвет маркера при прохождении через позицию",
                        "Change marker color when token passes this place",
                    ),
                );
                ui.checkbox(
                    &mut self.net.places[place_idx].input_module,
                    t(
                        "Определить позицию как вход модуля",
                        "Define place as module input",
                    ),
                );
                if self.net.places[place_idx].input_module {
                    ui.horizontal(|ui: &mut egui::Ui| {
                        ui.label(t("Номер входа", "Input number"));
                        let mut input_number = self.net.places[place_idx].input_number;
                        corrected_inputs |= sanitize_u32(&mut input_number, 1, u32::MAX);
                        if ui
                            .add(egui::DragValue::new(&mut input_number).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut input_number, 1, u32::MAX);
                        }
                        self.net.places[place_idx].input_number = input_number;
                    });
                    ui.label(t("Описание входа", "Input description"));
                    ui.text_edit_singleline(&mut self.net.places[place_idx].input_description);
                }

                ui.separator();
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(t("Стохастичестие процессы", "Stochastic processes"));
                    let stats_enabled = self.net.ui.marker_count_stats;
                    if ui
                        .add_enabled(
                            stats_enabled,
                            egui::Button::new(t("Сбор статистики", "Collect statistics")),
                        )
                        .clicked()
                    {
                        self.place_stats_dialog_place_id = Some(place_id);
                        self.place_stats_dialog_backup =
                            Some((place_id, self.net.places[place_idx].stats));
                    }
                });

                egui::ComboBox::from_label(t("Распределение", "Distribution"))
                    .selected_text(Self::stochastic_text(
                        &self.net.places[place_idx].stochastic,
                        is_ru,
                    ))
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::None,
                            Self::stochastic_text(&StochasticDistribution::None, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Uniform { min: 0.0, max: 1.0 },
                            Self::stochastic_text(
                                &StochasticDistribution::Uniform { min: 0.0, max: 1.0 },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Normal {
                                mean: 1.0,
                                std_dev: 0.2,
                            },
                            Self::stochastic_text(
                                &StochasticDistribution::Normal {
                                    mean: 1.0,
                                    std_dev: 0.2,
                                },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Gamma {
                                shape: 2.0,
                                scale: 1.0,
                            },
                            Self::stochastic_text(
                                &StochasticDistribution::Gamma {
                                    shape: 2.0,
                                    scale: 1.0,
                                },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Exponential { lambda: 1.0 },
                            Self::stochastic_text(
                                &StochasticDistribution::Exponential { lambda: 1.0 },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Poisson { lambda: 1.0 },
                            Self::stochastic_text(
                                &StochasticDistribution::Poisson { lambda: 1.0 },
                                is_ru,
                            ),
                        );
                    });

                match &mut self.net.places[place_idx].stochastic {
                    StochasticDistribution::None => {}
                    StochasticDistribution::Uniform { min, max } => {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label(t("min", "min"));
                            ui.add(egui::DragValue::new(min).speed(0.1).range(0.0..=10_000.0));
                            ui.label(t("max", "max"));
                            ui.add(egui::DragValue::new(max).speed(0.1).range(0.0..=10_000.0));
                        });
                        corrected_inputs |= sanitize_f64(min, 0.0, 10_000.0);
                        corrected_inputs |= sanitize_f64(max, 0.0, 10_000.0);
                        if *max < *min {
                            *max = *min;
                            corrected_inputs = true;
                        }
                    }
                    StochasticDistribution::Normal { mean, std_dev } => {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label(t("mean", "mean"));
                            ui.add(egui::DragValue::new(mean).speed(0.1).range(0.0..=10_000.0));
                            ui.label(t("std", "std"));
                            ui.add(
                                egui::DragValue::new(std_dev)
                                    .speed(0.1)
                                    .range(0.0..=10_000.0),
                            );
                        });
                        corrected_inputs |= sanitize_f64(mean, 0.0, 10_000.0);
                        corrected_inputs |= sanitize_f64(std_dev, 0.0, 10_000.0);
                    }
                    StochasticDistribution::Gamma { shape, scale } => {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label(t("shape", "shape"));
                            ui.add(
                                egui::DragValue::new(shape)
                                    .speed(0.1)
                                    .range(0.0001..=10_000.0),
                            );
                            ui.label(t("scale", "scale"));
                            ui.add(
                                egui::DragValue::new(scale)
                                    .speed(0.1)
                                    .range(0.0001..=10_000.0),
                            );
                        });
                        corrected_inputs |= sanitize_f64(shape, 0.0001, 10_000.0);
                        corrected_inputs |= sanitize_f64(scale, 0.0001, 10_000.0);
                    }
                    StochasticDistribution::Exponential { lambda }
                    | StochasticDistribution::Poisson { lambda } => {
                        ui.horizontal(|ui: &mut egui::Ui| {
                            ui.label(t("lambda", "lambda"));
                            ui.add(
                                egui::DragValue::new(lambda)
                                    .speed(0.1)
                                    .range(0.0001..=10_000.0),
                            );
                        });
                        corrected_inputs |= sanitize_f64(lambda, 0.0001, 10_000.0);
                    }
                }

                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );

                let mut markov_enabled = self.net.places[place_idx].markov_highlight;
                if ui
                    .checkbox(
                        &mut markov_enabled,
                        t("Марковская метка", "Markov annotation"),
                    )
                    .changed()
                {
                    self.net.places[place_idx].markov_highlight = markov_enabled;
                    self.update_markov_annotations();
                }

                let mut markov_placement = self.net.places[place_idx].markov_placement;
                egui::ComboBox::from_label(t(
                    "Положение марковской метки",
                    "Markov highlight placement",
                ))
                .selected_text(Self::markov_placement_text(markov_placement, is_ru))
                .show_ui(ui, |ui: &mut egui::Ui| {
                    ui.selectable_value(
                        &mut markov_placement,
                        MarkovPlacement::Bottom,
                        Self::markov_placement_text(MarkovPlacement::Bottom, is_ru),
                    );
                    ui.selectable_value(
                        &mut markov_placement,
                        MarkovPlacement::Top,
                        Self::markov_placement_text(MarkovPlacement::Top, is_ru),
                    );
                });
                self.net.places[place_idx].markov_placement = markov_placement;

                ui.separator();
                ui.label(t("Название", "Name"));
                ui.text_edit_singleline(&mut self.net.places[place_idx].name);
            },
        );
        open
    }
}
