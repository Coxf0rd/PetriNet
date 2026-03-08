from pathlib import Path
path = Path('src/ui/app/table_view.rs')
text = path.read_text(encoding='utf-8')
needle = '                        });\n                        p = p.min(max_place_idx);'
if needle not in text:
    raise SystemExit('needle missing stop place block')
replacement = '                        });\n                        corrected_inputs |= sanitize_usize(&mut p, 0, max_place_idx);\n                        corrected_inputs |= sanitize_u64(&mut n, 1, 1_000_000);\n                        p = p.min(max_place_idx);'
text = text.replace(needle, replacement, 1)
path.write_text(text, encoding='utf-8')
