from pathlib import Path
path = Path('src/ui/app.rs')
text = path.read_text(encoding='utf-8')
needle = '                self.table_fullscreen = !self.table_fullscreen;\n            }\n        });'
if needle not in text:
    raise SystemExit('needle missing')
replacement = '                self.table_fullscreen = !self.table_fullscreen;\n            }\n            if ui.button("Марковская модель").clicked() {\n                self.calculate_markov_model();\n                self.show_markov_window = true;\n            }\n        });'
text = text.replace(needle, replacement, 1)
path.write_text(text, encoding='utf-8')
