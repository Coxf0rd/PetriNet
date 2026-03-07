use egui::{Context, Window};

/// Экстеншн для `egui::Window`, ограничивающий размер окна видимой областью контекста.
pub trait WindowExt {
    fn constrained_to_viewport(self, ctx: &Context) -> Self;
}

impl<'a> WindowExt for Window<'a> {
    fn constrained_to_viewport(self, ctx: &Context) -> Self {
        let rect = ctx.available_rect();
        self.constrain_to(rect)
    }
}
