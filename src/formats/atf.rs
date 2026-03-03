use crate::model::PetriNet;

pub fn generate_atf(net: &PetriNet, selected_place: usize) -> String {
    let mut out = String::new();
    out.push_str("ATF v1\n");
    out.push_str(&format!("selected_place=P{}\n", selected_place + 1));
    out.push_str(&format!(
        "places={} transitions={}\n",
        net.places.len(),
        net.transitions.len()
    ));
    out.push_str("[M0]\n");
    for (i, v) in net.tables.m0.iter().enumerate() {
        out.push_str(&format!("P{}={}\n", i + 1, v));
    }
    out.push_str("[Mo]\n");
    for (i, v) in net.tables.mo.iter().enumerate() {
        match v {
            Some(cap) => out.push_str(&format!("P{}={}\n", i + 1, cap)),
            None => out.push_str(&format!("P{}=inf\n", i + 1)),
        }
    }
    out.push_str("[Mz]\n");
    for (i, v) in net.tables.mz.iter().enumerate() {
        out.push_str(&format!("P{}={}\n", i + 1, v));
    }
    out.push_str("[Mpr]\n");
    for (i, v) in net.tables.mpr.iter().enumerate() {
        out.push_str(&format!("T{}={}\n", i + 1, v));
    }
    out
}
