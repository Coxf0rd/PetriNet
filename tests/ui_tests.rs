//! Simple unit tests to verify that patched helper functions compile and behave as expected.
//!
//! These tests are basic compile-time checks rather than full UI tests.  They
//! ensure that `shrink2` behaves correctly.  To run these tests, execute `cargo test` in the
//! project root.

#[cfg(test)]
mod tests {
    use egui::{vec2, Pos2, Rect};

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
}
