use std::path::{Path, PathBuf};
use std::process::Command;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use petri_net_legacy_editor::io::legacy_gpn::{
    export_legacy_gpn, export_legacy_gpn_with_hints, import_legacy_gpn, LegacyExportHints,
};
use petri_net_legacy_editor::io::save_gpn;
use petri_net_legacy_editor::model::{NodeRef, PetriNetModel};
use petri_net_legacy_editor::sim::engine::{run_simulation, SimulationParams};

fn legacy_fixture_path() -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("gpn") {
                candidates.push(path);
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir("fixtures/legacy") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("gpn") {
                candidates.push(path);
            }
        }
    }
    candidates
        .into_iter()
        .max_by_key(|path| std::fs::metadata(path).map(|m| m.len()).unwrap_or(0))
        .expect("must contain at least one .gpn file")
}

#[test]
fn legacy_import_returns_ok() {
    let path = legacy_fixture_path();
    let result = import_legacy_gpn(&path);
    assert!(result.is_ok(), "legacy import should succeed");
}

#[test]
fn gpn_dump_runs_and_prints_summary() {
    let path = legacy_fixture_path();
    let bin = env!("CARGO_BIN_EXE_gpn_dump");

    let output = Command::new(bin)
        .arg(path)
        .arg("--strings")
        .output()
        .expect("failed to run gpn_dump");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Размер файла"));
}

#[test]
fn legacy_import_restores_coordinates_and_arcs() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    if imported.model.places.len() >= 18 && imported.model.transitions.len() >= 15 {
        assert_eq!(imported.model.arcs.len(), 32);
        assert_eq!(imported.model.places[0].pos, [21.0, 231.0]);
        assert_eq!(imported.model.places[1].pos, [86.0, 230.0]);
        assert_eq!(imported.model.tables.m0[0], 10);
        assert_eq!(imported.model.tables.mo[0], Some(10));
        assert!((imported.model.tables.mz[1] - 4.18).abs() < 1e-6);
        assert_eq!(imported.model.tables.m0[1], 0);
        assert_eq!(imported.model.tables.m0[2], 1);
        assert_eq!(imported.model.tables.m0[4], 1);
        assert_eq!(imported.model.tables.m0[7], 1);
        assert_eq!(imported.model.tables.m0[16], 1);
        assert_eq!(imported.model.transitions[0].angle_deg, 90);
        assert!(!imported.model.places[0].name.trim().is_empty());

        let place16_id = imported.model.places[15].id;
        let transition2_id = imported.model.transitions[1].id;
        let has_t2_to_p16 = imported.model.arcs.iter().any(|arc| {
            matches!(arc.from, NodeRef::Transition(id) if id == transition2_id)
                && matches!(arc.to, NodeRef::Place(id) if id == place16_id)
        });
        assert!(has_t2_to_p16);
    } else {
        assert!(!imported.model.places.is_empty());
        assert!(!imported.model.transitions.is_empty());
    }
}

#[test]
fn legacy_import_reads_cp1251_place_and_transition_names() {
    let path = Path::new("Сеть 3.gpn");
    if !path.exists() {
        return;
    }

    let imported = import_legacy_gpn(path).expect("legacy import must succeed");
    assert!(
        imported.model.places.iter().any(|p| p.name == "очередь"),
        "expected CP1251 place name to be imported"
    );
    assert!(
        imported
            .model
            .transitions
            .iter()
            .any(|t| t.note.contains("загрузка") || t.name.contains("загрузка")),
        "expected CP1251 transition label to be imported"
    );
}

#[test]
fn legacy_export_roundtrip_keeps_topology() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("roundtrip.gpn");

    export_legacy_gpn(&out, &imported.model).expect("legacy export must succeed");
    let loaded = import_legacy_gpn(&out).expect("legacy reimport must succeed");

    assert_eq!(loaded.model.places.len(), imported.model.places.len());
    assert_eq!(
        loaded.model.transitions.len(),
        imported.model.transitions.len()
    );
    assert_eq!(loaded.model.arcs.len(), imported.model.arcs.len());
}

#[test]
fn save_gpn_writes_legacy_for_gpn_extension() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("saved.gpn");

    save_gpn(&out, &imported.model).expect("save_gpn must succeed");
    let bytes = std::fs::read(&out).expect("saved file must exist");
    assert!(!bytes.starts_with(petri_net_legacy_editor::model::GPN2_MAGIC.as_bytes()));

    let loaded = import_legacy_gpn(&out).expect("saved legacy file must load");
    assert_eq!(loaded.model.places.len(), imported.model.places.len());
    assert_eq!(
        loaded.model.transitions.len(),
        imported.model.transitions.len()
    );
}

#[test]
fn legacy_simulation_has_enabled_transitions() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");

    let params = SimulationParams {
        use_pass_limit: true,
        pass_limit: 30,
        ..SimulationParams::default()
    };
    let result = run_simulation(&imported.model, &params, true, false);
    assert!(
        result.fired_count > 5,
        "simulation should keep firing transitions for the fixture, got {}",
        result.fired_count
    );
}

#[test]
fn legacy_simulation_runs_many_steps_for_set3() {
    let path = Path::new("Сеть 3.gpn");
    if !path.exists() {
        return;
    }

    let imported = import_legacy_gpn(path).expect("legacy import must succeed");
    let params = SimulationParams {
        use_pass_limit: true,
        pass_limit: 200,
        ..SimulationParams::default()
    };

    let result = run_simulation(&imported.model, &params, true, false);
    eprintln!(
        "places={} transitions={} arcs={} inhibitors={} mo={:?} mz={:?} fired_count={} logs={} final={:?}",
        imported.model.places.len(),
        imported.model.transitions.len(),
        imported.model.arcs.len(),
        imported.model.inhibitor_arcs.len(),
        imported.model.tables.mo,
        imported.model.tables.mz,
        result.fired_count,
        result.logs.len(),
        result.final_marking
    );
    if let Some(last) = result.logs.last() {
        eprintln!(
            "last_log: t={:.3} fired={:?} marking={:?}",
            last.time, last.fired_transition, last.marking
        );
    }
    assert_eq!(
        result.fired_count, 200,
        "expected to reach pass_limit=200, got {}",
        result.fired_count
    );
}

#[test]
fn legacy_save_and_reload_preserves_marking_profile() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("saved_again.gpn");

    save_gpn(&out, &imported.model).expect("save_gpn must succeed");
    let loaded = import_legacy_gpn(&out).expect("saved file should load");

    assert_eq!(loaded.model.tables.m0, imported.model.tables.m0);

    let params = SimulationParams {
        use_pass_limit: true,
        pass_limit: 30,
        ..SimulationParams::default()
    };
    let result = run_simulation(&loaded.model, &params, true, false);
    assert!(
        result.fired_count > 5,
        "reloaded model should remain simulatable with multiple firings, got {}",
        result.fired_count
    );
}

#[test]
fn legacy_import_removes_duplicate_unconnected_ghosts() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    let net = imported.model;

    let place_index = net.place_index_map();
    let transition_index = net.transition_index_map();
    let mut place_incident = vec![false; net.places.len()];
    let mut transition_incident = vec![false; net.transitions.len()];

    for arc in &net.arcs {
        match (arc.from, arc.to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid))
            | (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                if let Some(&pi) = place_index.get(&pid) {
                    place_incident[pi] = true;
                }
                if let Some(&ti) = transition_index.get(&tid) {
                    transition_incident[ti] = true;
                }
            }
            _ => {}
        }
    }

    for (idx, place) in net.places.iter().enumerate() {
        if place_incident[idx] {
            continue;
        }
        let has_connected_duplicate = net
            .places
            .iter()
            .enumerate()
            .any(|(other_idx, other)| {
                other_idx != idx
                    && place_incident[other_idx]
                    && (other.pos[0] - place.pos[0]).abs() < 0.5
                    && (other.pos[1] - place.pos[1]).abs() < 0.5
            });
        assert!(
            !has_connected_duplicate,
            "place ghost remained at {:?}",
            place.pos
        );
    }

    for (idx, tr) in net.transitions.iter().enumerate() {
        if transition_incident[idx] {
            continue;
        }
        let has_connected_duplicate = net
            .transitions
            .iter()
            .enumerate()
            .any(|(other_idx, other)| {
                other_idx != idx
                    && transition_incident[other_idx]
                    && (other.pos[0] - tr.pos[0]).abs() < 0.5
                    && (other.pos[1] - tr.pos[1]).abs() < 0.5
            });
        assert!(
            !has_connected_duplicate,
            "transition ghost remained at {:?}",
            tr.pos
        );
    }
}

#[test]
fn legacy_export_has_stable_arc_polyline_points() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("for_netstar.gpn");

    save_gpn(&out, &imported.model).expect("save_gpn must succeed");
    let bytes = std::fs::read(&out).expect("saved file must exist");

    let places = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let transitions = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    if transitions > 0 {
        let t0 = 16 + places * 231;
        assert_eq!(
            i32::from_le_bytes([bytes[t0 + 24], bytes[t0 + 25], bytes[t0 + 26], bytes[t0 + 27]]),
            196607
        );
        assert_eq!(
            i32::from_le_bytes([bytes[t0 + 28], bytes[t0 + 29], bytes[t0 + 30], bytes[t0 + 31]]),
            -655360
        );
        assert_eq!(
            i32::from_le_bytes([bytes[t0 + 32], bytes[t0 + 33], bytes[t0 + 34], bytes[t0 + 35]]),
            196607
        );
        assert_eq!(
            i32::from_le_bytes([bytes[t0 + 36], bytes[t0 + 37], bytes[t0 + 38], bytes[t0 + 39]]),
            655360
        );
        assert_eq!(
            i32::from_le_bytes([bytes[t0 + 40], bytes[t0 + 41], bytes[t0 + 42], bytes[t0 + 43]]),
            -131072
        );
        assert_eq!(
            i32::from_le_bytes([bytes[t0 + 44], bytes[t0 + 45], bytes[t0 + 46], bytes[t0 + 47]]),
            720895
        );
    }
    let arcs_offset = 16 + places * 231 + transitions * 105;
    let arc_max_index = i32::from_le_bytes([
        bytes[arcs_offset],
        bytes[arcs_offset + 1],
        bytes[arcs_offset + 2],
        bytes[arcs_offset + 3],
    ]);
    let arc_count = (arc_max_index + 1).max(0) as usize;
    let section_start = arcs_offset + 6;
    let section_end = section_start + arc_count * 46;
    assert!(section_end <= bytes.len(), "arc section must fit file");

    for idx in 0..arc_count {
        let off = section_start + idx * 46;
        let p1x = u16::from_le_bytes([bytes[off + 10], bytes[off + 11]]) as i32;
        let p1y = u16::from_le_bytes([bytes[off + 2], bytes[off + 3]]) as i32;
        let p2x = u16::from_le_bytes([bytes[off + 44], bytes[off + 45]]) as i32;
        let p2y = u16::from_le_bytes([bytes[off + 6], bytes[off + 7]]) as i32;
        let p3x = i32::from_le_bytes([
            bytes[off + 40],
            bytes[off + 41],
            bytes[off + 42],
            bytes[off + 43],
        ]);
        let p3y = u16::from_le_bytes([bytes[off + 14], bytes[off + 15]]) as i32;

        let mid_x = (p1x + p3x) / 2;
        let mid_y = (p1y + p3y) / 2;
        assert!(
            (p2x - mid_x).abs() <= 2 && (p2y - mid_y).abs() <= 2,
            "arc {} has unstable midpoint geometry",
            idx + 1
        );
    }

    assert_eq!(
        arc_max_index,
        (arc_count as i32) - 1,
        "legacy arc header must store max index (count - 1)"
    );

    let footer = &bytes[section_end..];
    assert_eq!(footer.len(), 52, "legacy footer must match NetStar-compatible size");
    assert_eq!(footer[0], 0x58);
    assert_eq!(footer[1], 0x81);
    assert_eq!(footer[2], 0x40);
}

fn arc_topology_fingerprint(model: &PetriNetModel) -> u64 {
    let place_idx: HashMap<u64, usize> = model
        .places
        .iter()
        .enumerate()
        .map(|(idx, place)| (place.id, idx + 1))
        .collect();
    let transition_idx: HashMap<u64, usize> = model
        .transitions
        .iter()
        .enumerate()
        .map(|(idx, transition)| (transition.id, idx + 1))
        .collect();

    let mut edges = Vec::<(u8, i8, usize, usize, u32)>::new();
    for arc in &model.arcs {
        match (arc.from, arc.to) {
            (NodeRef::Place(place_id), NodeRef::Transition(transition_id)) => {
                if let (Some(&p), Some(&t)) =
                    (place_idx.get(&place_id), transition_idx.get(&transition_id))
                {
                    edges.push((0, -1, p, t, arc.weight.max(1)));
                }
            }
            (NodeRef::Transition(transition_id), NodeRef::Place(place_id)) => {
                if let (Some(&t), Some(&p)) =
                    (transition_idx.get(&transition_id), place_idx.get(&place_id))
                {
                    edges.push((0, 1, t, p, arc.weight.max(1)));
                }
            }
            _ => {}
        }
    }
    for inh in &model.inhibitor_arcs {
        if let (Some(&p), Some(&t)) = (
            place_idx.get(&inh.place_id),
            transition_idx.get(&inh.transition_id),
        ) {
            edges.push((1, -1, p, t, inh.threshold.max(1)));
        }
    }
    edges.sort_unstable();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (model.places.len() as u64).hash(&mut hasher);
    (model.transitions.len() as u64).hash(&mut hasher);
    edges.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn legacy_export_with_hints_preserves_original_arc_blob_for_set3() {
    let path = Path::new("РЎРµС‚СЊ 3.gpn");
    if !path.exists() {
        return;
    }

    let src_bytes = std::fs::read(path).expect("read source set3");
    assert!(src_bytes.len() > 16, "set3 must not be empty");

    let imported = import_legacy_gpn(path).expect("legacy import must succeed");
    let places = imported.model.places.len();
    let transitions = imported.model.transitions.len();
    let arcs_off = 16usize + places * 231 + transitions * 105;
    assert!(arcs_off < src_bytes.len(), "arcs section must exist");
    let raw_arc_and_tail = src_bytes[arcs_off..].to_vec();

    let hints = LegacyExportHints {
        places_count: Some(places),
        transitions_count: Some(transitions),
        arc_topology_fingerprint: Some(arc_topology_fingerprint(&imported.model)),
        arc_header_extra: None,
        footer_bytes: None,
        raw_arc_and_tail: Some(raw_arc_and_tail.clone()),
    };

    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("set3_hinted.gpn");
    export_legacy_gpn_with_hints(&out, &imported.model, Some(&hints))
        .expect("legacy export with hints must succeed");
    let out_bytes = std::fs::read(&out).expect("read saved file");

    let out_arcs_off = 16usize + places * 231 + transitions * 105;
    assert!(out_arcs_off < out_bytes.len(), "output arcs section must exist");
    assert_eq!(
        &out_bytes[out_arcs_off..],
        raw_arc_and_tail.as_slice(),
        "arc+tail blob must be preserved verbatim"
    );
}
