use std::path::Path;

use petri_net_legacy_editor::io::load_gpn;
use petri_net_legacy_editor::sim::engine::{run_simulation, SimulationParams};

#[test]
fn probe_manipulator_file_runtime() {
    let path = Path::new("манипулятор+2 станка.gpn");
    if !path.exists() {
        return;
    }

    let loaded = load_gpn(path).expect("load_gpn must succeed");
    let params = SimulationParams {
        use_pass_limit: true,
        pass_limit: 200,
        ..SimulationParams::default()
    };
    let result = run_simulation(&loaded.model, &params, true, false);

    eprintln!(
        "loaded: places={} transitions={} arcs={} inhibitors={} fired={} logs={}",
        loaded.model.places.len(),
        loaded.model.transitions.len(),
        loaded.model.arcs.len(),
        loaded.model.inhibitor_arcs.len(),
        result.fired_count,
        result.logs.len()
    );
    if let Some(last) = result.logs.last() {
        eprintln!(
            "last: t={:.3} fired={:?} marking={:?}",
            last.time, last.fired_transition, last.marking
        );
    }
    assert_eq!(
        result.fired_count, 200,
        "expected to reach pass_limit on source gpn2 file"
    );
}
