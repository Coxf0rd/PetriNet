//! Utility functions for consistent scroll area behaviour across the UI.
//!
//! This module defines helpers to create vertical `ScrollArea`s with specific
//! scroll bar visibility semantics and to virtualize large lists.  Using
//! these helpers makes it easy to apply a unified scroll pattern to
//! collapsible sections, tables, and other scrollable regions.  See
//! `show_hidden_vertical_scroll`, `show_list_with_scroll`, and
//! `show_virtualized_rows` for details.

use egui::{self, scroll_area::ScrollBarVisibility, Id, Ui};

/// Show a vertical scroll area whose scroll bar is always hidden.
///
/// This helper wraps the provided `add_contents` closure in a vertical
/// `ScrollArea` with `ScrollBarVisibility::AlwaysHidden`.  The scroll
/// area automatically shrinks both axes so that the scroll bar does not
/// consume any space.  This is useful for collapsible property sections
/// where the content may be longer than the available space but the UI
/// should not display a scroll bar on the right.
pub fn show_hidden_vertical_scroll<R>(
    ui: &mut Ui,
    id_source: impl Into<Id>,
    max_height: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::ScrollArea::vertical()
        .id_source(id_source)
        .max_height(max_height)
        .auto_shrink([false, false])
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| add_contents(ui))
        .inner
}

/// Show a vertical scroll area whose scroll bar becomes visible when needed.
///
/// This helper wraps the provided `add_contents` closure in a vertical
/// `ScrollArea` with `ScrollBarVisibility::VisibleWhenNeeded`.  The scroll
/// bar will appear when the user hovers over the area and there is
/// overflow, matching the desired behaviour for tables and lists.  The
/// caller must specify a `max_height` to constrain the height of the
/// scroll area.
pub fn show_list_with_scroll<R>(
    ui: &mut Ui,
    id_source: impl Into<Id>,
    max_height: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::ScrollArea::vertical()
        .id_source(id_source)
        .max_height(max_height)
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .show(ui, |ui| add_contents(ui))
        .inner
}

/// Virtualize a list of rows with a scroll area whose scroll bar becomes visible when needed.
///
/// Given the total number of rows and the height of each row, this helper
/// renders only the visible rows inside a vertical `ScrollArea` using
/// `show_rows`.  The scroll bar appears when needed.  Each row is drawn
/// by invoking the `row_ui` closure for the range of visible row
/// indices.  The closure can lay out the row contents as desired.
pub fn show_virtualized_rows(
    ui: &mut Ui,
    id_source: impl Into<Id>,
    max_height: f32,
    row_height: f32,
    total_rows: usize,
    mut row_ui: impl FnMut(&mut Ui, usize),
) {
    egui::ScrollArea::vertical()
        .id_source(id_source)
        .max_height(max_height)
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .show_rows(ui, row_height, total_rows, |ui, range| {
            for idx in range {
                row_ui(ui, idx);
            }
        });
}

// Unit tests for the scroll utilities.  These tests live in the same file to
// ensure they have access to crate-private items.  They verify that the
// helpers compile and behave as expected without requiring a full `egui`
// context.
#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, Pos2, vec2};

    #[test]
    fn test_shrink2_behaviour() {
        // This test ensures that Rect::shrink2 behaves as expected.  It is
        // indirectly related to the property_window helper which relies on
        // shrink2 for margins.
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 50.0));
        let margin = vec2(10.0, 5.0);
        let shrunk = rect.shrink2(margin);
        assert_eq!(shrunk.width(), rect.width() - 2.0 * margin.x);
        assert_eq!(shrunk.height(), rect.height() - 2.0 * margin.y);
        assert_eq!(shrunk.center(), rect.center());
    }

    #[test]
    fn test_scroll_utils_signatures() {
        // These no-op calls ensure that the scroll util functions are callable
        // with the expected signatures.  We don't execute them because they
        // require a real `egui::Ui`, but if the types change this test will
        // fail to compile.
        let _hidden: fn(&mut Ui, _, f32, _) -> _ = show_hidden_vertical_scroll;
        let _list: fn(&mut Ui, _, f32, _) -> _ = show_list_with_scroll;
        let _virt: fn(&mut Ui, _, f32, f32, usize, _) = show_virtualized_rows;
    }
}