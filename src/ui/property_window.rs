use eframe::egui;

#[derive(Debug)]
pub(super) struct PropertyWindowConfig<'a> {
    pub id: egui::Id,
    pub default_size: egui::Vec2,
    pub min_size: egui::Vec2,
    pub resizable: bool,
    pub remember_size: Option<&'a mut egui::Vec2>,
    pub apply_default_size: bool,
}

impl<'a> PropertyWindowConfig<'a> {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            default_size: egui::vec2(420.0, 520.0),
            min_size: egui::vec2(320.0, 360.0),
            resizable: true,
            remember_size: None,
            apply_default_size: true,
        }
    }

    pub fn default_size(mut self, size: egui::Vec2) -> Self {
        self.default_size = size;
        self
    }

    pub fn min_size(mut self, size: egui::Vec2) -> Self {
        self.min_size = size;
        self
    }

    pub fn resizable(mut self, value: bool) -> Self {
        self.resizable = value;
        self
    }

    pub fn remember_size(mut self, size: &'a mut egui::Vec2) -> Self {
        self.remember_size = Some(size);
        self
    }

    pub fn apply_default_size(mut self, value: bool) -> Self {
        self.apply_default_size = value;
        self
    }
}

pub(super) fn viewport_rect(ctx: &egui::Context) -> egui::Rect {
    let screen_rect = ctx.input(|input| input.screen_rect());
    if screen_rect == egui::Rect::EVERYTHING {
        ctx.available_rect()
    } else {
        screen_rect
    }
}

pub(super) fn show_property_window<R>(
    ctx: &egui::Context,
    title: impl Into<egui::WidgetText>,
    open: &mut bool,
    mut config: PropertyWindowConfig<'_>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<egui::InnerResponse<R>> {
    let viewport = viewport_rect(ctx);
    let max_size = viewport.size();
    let default_size = config.default_size.min(max_size);
    let min_size = config.min_size.min(max_size);

    let mut window = egui::Window::new(title)
        .id(config.id)
        .constrain_to(viewport)
        .max_size(max_size)
        .resizable(config.resizable)
        .min_size(min_size)
        .open(open);

    if config.apply_default_size {
        window = window.default_size(default_size);
    }

    let response = window.show(ctx, |ui| {
        ui.set_max_width(ui.available_width());
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(0.0);
                ui.set_max_width(ui.available_width());
                add_contents(ui)
            })
            .inner
    });

    if *open {
        if let (Some(size), Some(response)) = (config.remember_size.as_deref_mut(), response.as_ref()) {
            let actual_size = response.response.rect.size();
            if actual_size.x > 0.0 && actual_size.y > 0.0 {
                *size = actual_size;
            }
        }
    }

    response
}