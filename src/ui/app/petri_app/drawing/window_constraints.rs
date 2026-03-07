use egui::{Context, Rect, Vec2, Window};

const WINDOW_MARGIN: f32 = 18.0;

/// Экстеншн для `egui::Window`, ограничивающий размер окна видимой областью контекста.
pub trait WindowExt {
    fn constrained_to_viewport(self, ctx: &Context) -> Self;
}

impl<'a> WindowExt for Window<'a> {
    fn constrained_to_viewport(self, ctx: &Context) -> Self {
        let screen_rect = ctx.input(|input| input.screen_rect());
        let viewport = if screen_rect == Rect::EVERYTHING {
            ctx.available_rect()
        } else {
            screen_rect
        };
        let margin = Vec2::splat(WINDOW_MARGIN);
        let shrunken = Rect::from_min_max(viewport.min + margin, viewport.max - margin);
        let inner = if shrunken.min.x < shrunken.max.x && shrunken.min.y < shrunken.max.y {
            shrunken
        } else {
            viewport
        };
        let size = inner.size();
        self.default_size(size).max_size(size).constrain_to(inner)
    }
}
