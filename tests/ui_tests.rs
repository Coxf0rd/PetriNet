//! Simple unit tests to verify that patched helper functions compile and behave as expected.
//!
//! These tests are basic compile-time checks rather than full UI tests.  They
//! ensure that calls to `shrink2` compile and that the scroll utility
//! functions are accessible.  To run these tests, execute `cargo test` in the
//! project root.

#[cfg(test)]
mod tests {
    use egui::{Rect, Pos2, vec2};

    /// Verify that `shrink2` reduces a rectangle's width and height by the
    /// specified margin.  This test compiles against the patched
    /// `property_window` helper which uses `shrink2` internally.
    #[test]
    fn test_shrink2_margin() {
        let r = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
        let margin = vec2(20.0, 20.0);
        let shr = r.shrink2(margin);
        // The shrunken rectangle should be strictly smaller in both dimensions.
        assert!(shr.width() < r.width());
        assert!(shr.height() < r.height());
        // It should also maintain the same center.
        assert_eq!(r.center(), shr.center());
    }

    /// Ensure that the scroll utilities module can be referenced.  This test
    /// performs no runtime assertions but guarantees that the module and
    /// functions are available and can be invoked without errors.
    #[test]
    fn test_scroll_utils_access() {
        use crate::ui::scroll_utils::{show_hidden_vertical_scroll, show_list_with_scroll, show_virtualized_rows};
        // Due to egui's requirements for running UI, we can't construct a full
        // `egui::Ui` here.  Instead, we simply ensure that the functions can
        // be referenced.  This test will fail to compile if the paths are
        // incorrect.
        let _ = (show_hidden_vertical_scroll as fn(&mut egui::Ui, _, f32, _));
        let _ = (show_list_with_scroll as fn(&mut egui::Ui, _, f32, _));
        let _ = (show_virtualized_rows as fn(&mut egui::Ui, _, f32, f32, usize, _));
    }
}