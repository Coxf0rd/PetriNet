use eframe::egui;

/// Property window configuration and helpers.
///
/// This file defines a helper for creating property windows within the GUI.
/// The original implementation constrained the window's maximum size to the
/// viewport size.  To provide a bit of breathing room around the edges of
/// the application, we subtract a small margin from the viewport before
/// computing the maximum size.  This prevents property windows from
/// occupying the entire viewport, as requested by the user.

#[derive(Debug)]
pub(crate) struct PropertyWindowConfig<'a> {
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

pub(crate) fn viewport_rect(ctx: &egui::Context) -> egui::Rect {
    let screen_rect = ctx.input(|input| input.screen_rect());
    if screen_rect == egui::Rect::EVERYTHING {
        ctx.available_rect()
    } else {
        screen_rect
    }
}

/// Show a property window.
///
/// The window is constrained to the current viewport and its maximum size is
/// reduced by a small margin (20×20 points) so that it never completely
/// covers the application window.  This change preserves the overall
/// behaviour of the original implementation while satisfying the new
/// requirement that property windows leave a bit of space around the edges.
pub(crate) fn show_property_window<R>(
    ctx: &egui::Context,
    title: impl Into<egui::WidgetText>,
    open: &mut bool,
    mut config: PropertyWindowConfig<'_>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    let viewport = viewport_rect(ctx);
    // Introduce a small margin so that the max size is slightly smaller than the
    // viewport.  Without this adjustment, windows could be as large as the
    // viewport itself.
    // Apply the margin on all sides.  The window's maximum size should be
    // smaller than the viewport by twice the margin so that there is always a
    // visible gap between the window and each edge, even when the user
    // resizes it to the maximum allowed dimensions.  We shrink the viewport
    // rectangle by `margin` on all sides before calculating the max size.
    // Horizontal margin is set to 0.0 so that property windows can slide up to the
    // same right boundary as the results/statistics window.  We retain a
    // vertical margin (20.0) to keep a bit of space above and below the
    // window.  Previously a 20×20 margin was applied on both axes, which
    // limited how far the window could be dragged to the right.  Removing the
    // horizontal component unifies the drag boundary with other panels.
    let margin = egui::vec2(0.0, 20.0);
    // Shrink the viewport by the margin so that constraints are applied inside
    // the shrunken area.  `shrink()` returns a new rectangle reduced on all
    // sides.
    // Use shrink2 instead of shrink to apply the margin on both axes.  In egui
    // 0.22, `shrink2` reduces the rectangle by the given vector on all sides
    // (x on left/right and y on top/bottom).  Using `shrink` here would
    // incorrectly interpret the Vec2 as a uniform scalar and trigger a type
    // mismatch during compilation.  See build logs for details.
    let constrained_viewport = viewport.shrink2(margin);
    let mut max_size = constrained_viewport.size();
    // Clamp to zero in case the viewport is smaller than twice the margin.
    if max_size.x < 0.0 {
        max_size.x = 0.0;
    }
    if max_size.y < 0.0 {
        max_size.y = 0.0;
    }
    let default_size = config.default_size.min(max_size);
    let min_size = config.min_size.min(max_size);

    // Constrain the window to the shrunken viewport rather than the full
    // viewport.  This prevents the window from extending into the margin area.
    let mut window = egui::Window::new(title)
        .id(config.id)
        .constrain_to(constrained_viewport)
        .max_size(max_size)
        .resizable(config.resizable)
        .min_size(min_size)
        .open(open);

    if config.apply_default_size {
        window = window.default_size(default_size);
    }

    let response = window.show(ctx, |ui: &mut egui::Ui| {
        ui.set_max_width(ui.available_width());
        // Use a scroll area with hidden scrollbars for property windows.  We still
        // enable vertical scrolling but never display the scroll bar, even when the
        // content exceeds the available height.  This matches the requirement
        // that collapsible sections should scroll without showing a visible
        // scrollbar.
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui: &mut egui::Ui| {
                ui.set_min_width(0.0);
                ui.set_max_width(ui.available_width());
                add_contents(ui);
            });
    });

    if *open {
        if let (Some(size), Some(response)) = (config.remember_size.as_deref_mut(), response.as_ref()) {
            let actual_size = response.response.rect.size();
            if actual_size.x > 0.0 && actual_size.y > 0.0 {
                *size = actual_size;
            }
        }
    }
}