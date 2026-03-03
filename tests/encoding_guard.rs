use std::fs;
use std::path::Path;

fn assert_no_mojibake(path: &Path) {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let bad_markers = ["Р В¤", "вЂў", "Р вЂ", "РЎвЂ", "вљ", "РВ", "в„"];
    for marker in bad_markers {
        assert!(
            !text.contains(marker),
            "mojibake marker '{marker}' found in {}",
            path.display()
        );
    }
}

#[test]
fn ui_files_have_no_mojibake_markers() {
    assert_no_mojibake(Path::new("src/ui/app.rs"));
    assert_no_mojibake(Path::new("src/ui/app/table_view.rs"));
    assert_no_mojibake(Path::new("src/ui/app/shortcuts.rs"));
}
