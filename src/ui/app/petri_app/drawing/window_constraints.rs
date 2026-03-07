use egui::{Context, Rect, Window};

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
        let size = viewport.size();
        self.max_size(size).constrain_to(viewport)
    }
}
