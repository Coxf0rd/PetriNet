from pathlib import Path

root = Path('.').resolve()
allowed = {
    '.rs', '.md', '.toml', '.ps1', '.py', '.txt', '.json', '.yaml', '.yml',
    '.csv', '.ini', '.sh', '.cfg', '.lock', '.gitignore', '.cargo'
}
out = Path('docs/copy.md')

with out.open('w', encoding='utf-8') as f:
    for path in sorted(root.rglob('*')):
        if not path.is_file():
            continue
        if path.name.startswith('.'):
            continue
        if path == out:
            continue
        if path.suffix.lower() not in allowed:
            continue
        rel = path.relative_to(root)
        f.write(f"# {rel}\n")
        try:
            f.write(path.read_text(encoding='utf-8'))
        except UnicodeDecodeError:
            f.write(path.read_text(encoding='utf-8', errors='ignore'))
        f.write('\n\n')
