use std::env;
use std::fs;
use std::path::Path;

use petri_net_legacy_editor::io::legacy_gpn;

fn main() {
    if let Err(e) = run() {
        eprintln!("Ошибка: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Использование: gpn_dump <file.gpn> [--hexdump N] [--strings] [--floats] [--search \"pattern\"]".to_string());
    }

    let file = args[1].clone();
    let mut hexdump: Option<usize> = None;
    let mut show_strings = false;
    let mut show_floats = false;
    let mut search: Option<String> = None;

    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--hexdump" => {
                let n = args
                    .get(i + 1)
                    .ok_or_else(|| "Для --hexdump нужно число байт".to_string())?
                    .parse::<usize>()
                    .map_err(|_| "Некорректное значение --hexdump".to_string())?;
                hexdump = Some(n);
                i += 2;
            }
            "--strings" => {
                show_strings = true;
                i += 1;
            }
            "--floats" => {
                show_floats = true;
                i += 1;
            }
            "--search" => {
                search = Some(
                    args.get(i + 1)
                        .ok_or_else(|| "Для --search нужна строка".to_string())?
                        .to_string(),
                );
                i += 2;
            }
            other => {
                return Err(format!("Неизвестный флаг: {other}"));
            }
        }
    }

    let path = Path::new(&file);
    let bytes = fs::read(path).map_err(|e| format!("Не удалось прочитать файл: {e}"))?;

    println!("Файл: {}", path.display());
    println!("Размер файла: {} байт", bytes.len());
    println!("Legacy: {}", legacy_gpn::detect_legacy_gpn(&bytes));

    if let Some(n) = hexdump {
        println!("\n[Hexdump первых {} байт]", n.min(bytes.len()));
        print_hexdump(&bytes[..n.min(bytes.len())]);
    }

    let ascii = legacy_gpn::extract_ascii_strings(&bytes, 4);
    let utf16 = legacy_gpn::extract_utf16le_strings(&bytes, 4);
    let floats = legacy_gpn::extract_float64_pairs(&bytes, 64);

    if show_strings {
        println!("\n[ASCII строки]");
        for (off, s) in &ascii {
            println!("0x{off:08X}: {s}");
        }
        println!("\n[UTF-16LE строки]");
        for (off, s) in &utf16 {
            println!("0x{off:08X}: {s}");
        }
    }

    if show_floats {
        println!("\n[Кандидаты float64 пар]");
        for (off, a, b) in &floats {
            println!("0x{off:08X}: ({a:.6}, {b:.6})");
        }
    }

    if let Some(pattern) = search {
        println!("\n[Поиск ASCII паттерна: {pattern}]");
        for off in find_ascii_pattern(&bytes, pattern.as_bytes()) {
            println!("Найдено смещение: 0x{off:08X}");
        }
    }

    if let Ok(parsed) = legacy_gpn::import_legacy_gpn(path) {
        println!("\n[Импорт best-effort]");
        println!(
            "places={}, transitions={}, warnings={}",
            parsed.model.places.len(),
            parsed.model.transitions.len(),
            parsed.warnings.len()
        );
        for section in parsed.debug.discovered_sections {
            println!("section: {section}");
        }
    }

    Ok(())
}

fn print_hexdump(bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let off = i * 16;
        print!("{off:08X}  ");
        for b in chunk {
            print!("{b:02X} ");
        }
        println!();
    }
}

fn find_ascii_pattern(bytes: &[u8], pat: &[u8]) -> Vec<usize> {
    if pat.is_empty() || pat.len() > bytes.len() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for i in 0..=(bytes.len() - pat.len()) {
        if &bytes[i..i + pat.len()] == pat {
            out.push(i);
        }
    }
    out
}
