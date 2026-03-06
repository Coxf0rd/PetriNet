use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn import_matrix_csv(&mut self, target: MatrixCsvTarget) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file()
        else {
            return;
        };

        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                self.last_error = Some(format!("CSV read error: {e}"));
                return;
            }
        };

        let first_line = text.lines().next().unwrap_or_default();
        let semi = first_line.matches(';').count();
        let comma = first_line.matches(',').count();
        let delim = if semi >= comma { ';' } else { ',' };

        let mut lines = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty());
        let Some(header) = lines.next() else {
            self.last_error = Some("CSV parse error: empty file".to_string());
            return;
        };

        let header_cells: Vec<&str> = header.split(delim).map(|c| c.trim()).collect();
        if header_cells.len() < 2 {
            self.last_error = Some("CSV parse error: missing header columns".to_string());
            return;
        }

        let parse_ordinal = |s: &str, prefix: char| -> Option<usize> {
            let s = s.trim();
            let s = s.strip_prefix(prefix)?;
            let n: usize = s.parse().ok()?;
            n.checked_sub(1)
        };

        let mut col_map: Vec<usize> = Vec::new();
        for (col_idx, raw) in header_cells.iter().skip(1).enumerate() {
            col_map.push(parse_ordinal(raw, 'T').unwrap_or(col_idx));
        }

        let mut entries: Vec<(usize, usize, u32)> = Vec::new();
        let mut required_p = 0usize;
        let mut required_t = col_map.iter().copied().max().unwrap_or(0).saturating_add(1);

        for (row_idx, line) in lines.enumerate() {
            let cells: Vec<&str> = line.split(delim).map(|c| c.trim()).collect();
            if cells.len() < 2 {
                continue;
            }
            let p_idx = parse_ordinal(cells[0], 'P').unwrap_or(row_idx);
            required_p = required_p.max(p_idx + 1);

            for (ci, raw_val) in cells.iter().skip(1).enumerate() {
                let t_idx = *col_map.get(ci).unwrap_or(&ci);
                required_t = required_t.max(t_idx + 1);

                if raw_val.is_empty() {
                    continue;
                }

                let parsed: i64 = match raw_val.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.last_error =
                            Some(format!("CSV parse error: invalid number '{raw_val}'"));
                        return;
                    }
                };
                if parsed < 0 {
                    self.last_error = Some(format!("CSV parse error: negative value '{raw_val}'"));
                    return;
                }
                let val: u32 = match parsed.try_into() {
                    Ok(v) => v,
                    Err(_) => {
                        self.last_error =
                            Some(format!("CSV parse error: value too large '{raw_val}'"));
                        return;
                    }
                };
                entries.push((p_idx, t_idx, val));
            }
        }

        if required_p == 0 || required_t == 0 {
            self.last_error = Some("CSV parse error: empty matrix".to_string());
            return;
        }

        let cur_p = self.net.places.len();
        let cur_t = self.net.transitions.len();
        if required_p > cur_p || required_t > cur_t {
            self.net
                .set_counts(cur_p.max(required_p), cur_t.max(required_t));
        }

        match target {
            MatrixCsvTarget::Pre => {
                for (p, t, v) in entries {
                    if p < self.net.tables.pre.len() && t < self.net.tables.pre[p].len() {
                        self.net.tables.pre[p][t] = v;
                    }
                }
            }
            MatrixCsvTarget::Post => {
                for (p, t, v) in entries {
                    if p < self.net.tables.post.len() && t < self.net.tables.post[p].len() {
                        self.net.tables.post[p][t] = v;
                    }
                }
            }
            MatrixCsvTarget::Inhibitor => {
                for (p, t, v) in entries {
                    if p < self.net.tables.inhibitor.len() && t < self.net.tables.inhibitor[p].len()
                    {
                        self.net.tables.inhibitor[p][t] = v;
                    }
                }
            }
        }

        self.net.rebuild_arcs_from_matrices();
        self.last_error = None;
        let target_name = match target {
            MatrixCsvTarget::Pre => "Pre",
            MatrixCsvTarget::Post => "Post",
            MatrixCsvTarget::Inhibitor => "Inhibitor",
        };
        self.status_hint = Some(format!(
            "{}: {}x{} -> {}",
            self.tr("Импорт CSV", "CSV import"),
            required_p,
            required_t,
            target_name
        ));
    }
}
