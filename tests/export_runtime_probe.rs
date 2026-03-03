use std::path::Path;

use petri_net_legacy_editor::io::legacy_gpn::import_legacy_gpn;
use petri_net_legacy_editor::io::{load_gpn, save_gpn};
use petri_net_legacy_editor::sim::engine::{run_simulation, SimulationParams};

#[test]
fn probe_exported_manipulator_runtime() {
    let src = Path::new("манипулятор+2 станка.gpn");
    if !src.exists() {
        return;
    }

    let loaded = load_gpn(src).expect("load_gpn must succeed");
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("exported.gpn");
    save_gpn(&out, &loaded.model).expect("save_gpn must succeed");

    let imported = import_legacy_gpn(&out).expect("legacy reimport must succeed");
    let params = SimulationParams {
        use_pass_limit: true,
        pass_limit: 200,
        ..SimulationParams::default()
    };
    let result = run_simulation(&imported.model, &params, true, false);

    eprintln!(
        "exported/reloaded: places={} transitions={} arcs={} inhibitors={} fired={}",
        imported.model.places.len(),
        imported.model.transitions.len(),
        imported.model.arcs.len(),
        imported.model.inhibitor_arcs.len(),
        result.fired_count
    );
    assert_eq!(
        result.fired_count, 200,
        "expected to reach pass_limit after export/reload"
    );
}
