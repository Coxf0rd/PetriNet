# .github\workflows\release.yml
name: release-builds

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

permissions:
  contents: write

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            artifact: petri-net-linux
            bin: petri_net_legacy_editor
          - os: macos-latest
            artifact: petri-net-macos
            bin: petri_net_legacy_editor
          - os: windows-latest
            artifact: petri-net-windows
            bin: petri_net_legacy_editor.exe

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
      
        uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release

      - name: Package artifact (unix)
        if: runner.os != 'Windows'
        run: |
          mkdir -p dist
          cp target/release/${{ matrix.bin }} dist/
          tar -czf ${{ matrix.artifact }}.tar.gz -C dist ${{ matrix.bin }}

      - name: Package artifact (windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          New-Item -ItemType Directory -Force dist | Out-Null
          Copy-Item target/release/${{ matrix.bin }} dist/
          Compress-Archive -Path dist/${{ matrix.bin }} -DestinationPath ${{ matrix.artifact }}.zip

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: |
            *.tar.gz
            *.zip

  release:
    needs: build
    if: startsWith(github.ref, 'refs/tags/')
    runs-on: ubuntu-latest
    steps:
      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: out

      - name: Publish release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            out/**/*.tar.gz
            out/**/*.zip


# AGENTS.md
# AGENTS.md

## Обязательные правила работы Codex в этом проекте

1. После каждого фикса обязательно делать коммит с пояснением на русском, что именно поменялось.
2. После каждого фикса обязательно:
   - обновлять версию приложения (например, 0.6.3 -> 0.6.4),
   - пересобирать exe.
3. Данные для отправки в Git (использовать только когда пользователь прямо просит залить):
   - URL: `http://100.64.0.7:3000/Coxford/PetriNet`
   - token: `dcf0e97eec14daf50c69f02c20d924f1b87ad10e`
4. Если пользователь прямо не попросил залить на Git, заливку не выполнять.
## Кодировка русского языка

5. Формат русского языка во всех текстовых файлах проекта: только `UTF-8` (без BOM). Не использовать ANSI/CP1251/UTF-16.
6. После правок локализации обязательно проверять, что в исходниках и UI-строках нет битой кодировки (mojibake).

## Проверки перед коммитом

7. Перед каждым коммитом обязательно запускать и проверять результат:
   - `cargo check`
   - `cargo test`
   - `git diff --name-only -- '*.rs' | ForEach-Object { rustfmt --check $_ }` (проверка форматирования только изменённых Rust-файлов)
   - `rg -n "�|\?\?\?\?" src README.md` (вывод должен быть пустым)
8. Если хотя бы одна проверка не пройдена, коммит не делать до исправления.
## Структура раздела `src/ui/app/petri_app/` (функциональные модули)

- `src/ui/app/petri_app/mod.rs` — точка входа модуля `petri_app`; он теперь делится на вложенные подпапки по смыслу.
- `src/ui/app/petri_app/drawing/` — отрисовочные окна, статус и панель меню (`draw_*`).
- `src/ui/app/petri_app/file_ops/` — создание/загрузка/сохранение файла, импорт, синхронизация UI и подсказки (`new`, `open`, `save`, `ui_sidecar_path`, `sync_*`, `load_legacy_*`, `arc_topology_fingerprint` и т.п.).
- `src/ui/app/petri_app/netstar/` — экспорт/валидация Net* (функции `validate_netstar_export`, `export_netstar_file`, `netstar_non_exportable_items`, `start/clear/confirm` и вспомогательные проверки).
- `src/ui/app/petri_app/selection/` — логика выбора, копирования, отмены и собирания ID (`clear_selection`, `push_undo_snapshot`, `collect_selected_*`, `toggle_selected_id` и т.д.).
- `src/ui/app/petri_app/clipboard/` — код работы с буфером обмена и копированием объектов.
- `src/ui/app/petri_app/geometry/` — геометрические вычисления холста и фильтры (сетку, привязку, преобразования координат, работа с рамками/дугами).
- `src/ui/app/petri_app/indexing/` — поиск индексов/ID (места, переходы, дуги, уступы, тексты, рамки).
- `src/ui/app/petri_app/helpers/` — текстовые/цветовые/стилистические утилиты, вспомогательные метрики и форматирование (`tr`, `node_color_text`, `format_marking`, `approx_text_rect`, `sampled_indices` и т.д.).
- `src/ui/app/petri_app/markov/` — расчёт цепи Маркова и аннотаций.

Эта структура отражает выделенные функциональные блоки: интерфейс (drawing), управление файлом/синхронизация (file_ops), экспорт/валидация (netstar), выбор/отмена (selection), геометрия/хитрые расчёты (geometry), индексирование объектов (indexing), буфер обмена (clipboard), вспомогательные утилиты (helpers) и статистика/Markov (markov).


# build.rs
#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/petrinet.ico");
    let _ = res.compile();
}

#[cfg(not(windows))]
fn main() {}


# build_exe.ps1
param(
    [string]$ProjectDir = $PSScriptRoot,
    [switch]$KeepTarget
)

$ErrorActionPreference = "Stop"
Set-Location $ProjectDir

$buildPortable = Join-Path $ProjectDir "build_portable_exe.ps1"
if (-not (Test-Path $buildPortable)) {
    throw "build_portable_exe.ps1 not found: $buildPortable"
}

# Build only the versioned exe (PetriNet-<version>.exe). Do not create a stable PetriNet.exe,
# since we want to keep only versioned artifacts.
if ($KeepTarget) {
    & $buildPortable -ProjectDir $ProjectDir -KeepTarget
} else {
    & $buildPortable -ProjectDir $ProjectDir
}

$cargoTomlText = Get-Content -Path (Join-Path $ProjectDir "Cargo.toml") -Raw
$match = [regex]::Match($cargoTomlText, '(?m)^\s*version\s*=\s*"([^"]+)"')
if (-not $match.Success) {
    throw "Failed to read package version from Cargo.toml"
}
$version = $match.Groups[1].Value

$versionedExe = Join-Path $ProjectDir ("PetriNet-{0}.exe" -f $version)
if (-not (Test-Path $versionedExe)) {
    throw "Versioned executable not found: $versionedExe"
}
Write-Host "Executable ready: $versionedExe"


# build_portable_exe.ps1
param(
    [string]$ProjectDir = $PSScriptRoot,
    [string]$OutputExe = "",
    [switch]$KeepTarget
)

$ErrorActionPreference = "Stop"
Set-Location $ProjectDir

$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
$cargoPath = if ($cargoCmd) { $cargoCmd.Path } else { Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe" }
if (-not (Test-Path $cargoPath)) {
    throw "Cargo not found. Install Rust toolchain or add cargo to PATH."
}

$releaseExe = Join-Path $ProjectDir "target\release\petri_net_legacy_editor.exe"
if ([string]::IsNullOrWhiteSpace($OutputExe)) {
    $cargoTomlPath = Join-Path $ProjectDir "Cargo.toml"
    if (-not (Test-Path $cargoTomlPath)) {
        throw "Cargo.toml not found: $cargoTomlPath"
    }
    $cargoTomlText = Get-Content -Path $cargoTomlPath -Raw
    $match = [regex]::Match($cargoTomlText, '(?m)^\s*version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "Failed to read package version from Cargo.toml"
    }
    $version = $match.Groups[1].Value
    $OutputExe = "PetriNet-$version.exe"
}
$outputExePath = Join-Path $ProjectDir $OutputExe

Write-Host "Building release (static CRT)..."
$previousRustFlags = $env:RUSTFLAGS
$crtStaticFlag = "-C target-feature=+crt-static"
if ([string]::IsNullOrWhiteSpace($env:RUSTFLAGS)) {
    $env:RUSTFLAGS = $crtStaticFlag
} elseif ($env:RUSTFLAGS -notmatch [regex]::Escape($crtStaticFlag)) {
    $env:RUSTFLAGS = "$($env:RUSTFLAGS) $crtStaticFlag"
}
try {
    & $cargoPath build --release
} finally {
    $env:RUSTFLAGS = $previousRustFlags
}

if (-not (Test-Path $releaseExe)) {
    throw "Build finished, but executable not found: $releaseExe"
}

if (Test-Path $outputExePath) {
    Remove-Item $outputExePath -Force
}
Copy-Item $releaseExe $outputExePath -Force
Write-Host "Executable ready: $outputExePath"

# Keep only the newest versioned executable in the project dir.
$outputExeFull = $null
try {
    $outputExeFull = (Resolve-Path -LiteralPath $outputExePath).Path
} catch {
    $outputExeFull = $outputExePath
}
Get-ChildItem -Path $ProjectDir -Filter "PetriNet-*.exe" -File -ErrorAction SilentlyContinue | ForEach-Object {
    if ($_.FullName -ne $outputExeFull) {
        try {
            Remove-Item -LiteralPath $_.FullName -Force -ErrorAction Stop
        } catch {
            Write-Warning "Failed to remove old exe: $($_.FullName). Close any running old version and rebuild."
        }
    }
}

# If a stable PetriNet.exe exists from an older workflow, remove it.
$stableExe = Join-Path $ProjectDir "PetriNet.exe"
if (Test-Path $stableExe) {
    try {
        Remove-Item -LiteralPath $stableExe -Force -ErrorAction Stop
    } catch {
        Write-Warning "Failed to remove old exe: $stableExe. Close any running old version and rebuild."
    }
}

if (-not $KeepTarget) {
    $targetDir = Join-Path $ProjectDir "target"
    if (Test-Path $targetDir) {
        Remove-Item $targetDir -Recurse -Force
        Write-Host "Target cleaned: $targetDir"
    }
}


# Cargo.lock
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "ab_glyph"
version = "0.2.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "01c0457472c38ea5bd1c3b5ada5e368271cb550be7a4ca4a0b4634e9913f6cc2"
dependencies = [
 "ab_glyph_rasterizer",
 "owned_ttf_parser",
]

[[package]]
name = "ab_glyph_rasterizer"
version = "0.1.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "366ffbaa4442f4684d91e2cd7c5ea7c4ed8add41959a31447066e279e432b618"

[[package]]
name = "accesskit"
version = "0.12.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "74a4b14f3d99c1255dcba8f45621ab1a2e7540a0009652d33989005a4d0bfc6b"
dependencies = [
 "enumn",
 "serde",
]

[[package]]
name = "adler2"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa"

[[package]]
name = "ahash"
version = "0.8.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
dependencies = [
 "cfg-if",
 "getrandom 0.3.4",
 "once_cell",
 "serde",
 "version_check",
 "zerocopy",
]

[[package]]
name = "android-activity"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee91c0c2905bae44f84bfa4e044536541df26b7703fd0888deeb9060fcc44289"
dependencies = [
 "android-properties",
 "bitflags 2.11.0",
 "cc",
 "cesu8",
 "jni",
 "jni-sys",
 "libc",
 "log",
 "ndk",
 "ndk-context",
 "ndk-sys",
 "num_enum",
 "thiserror 1.0.69",
]

[[package]]
name = "android-properties"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc7eb209b1518d6bb87b283c20095f5228ecda460da70b44f0802523dea6da04"

[[package]]
name = "anyhow"
version = "1.0.102"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f202df86484c868dbad7eaa557ef785d5c66295e41b460ef922eca0723b842c"

[[package]]
name = "arboard"
version = "3.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0348a1c054491f4bfe6ab86a7b6ab1e44e45d899005de92f58b3df180b36ddaf"
dependencies = [
 "clipboard-win",
 "image",
 "log",
 "objc2 0.6.4",
 "objc2-app-kit 0.3.2",
 "objc2-core-foundation",
 "objc2-core-graphics",
 "objc2-foundation 0.3.2",
 "parking_lot",
 "percent-encoding",
 "windows-sys 0.60.2",
 "x11rb",
]

[[package]]
name = "as-raw-xcb-connection"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175571dd1d178ced59193a6fc02dde1b972eb0bc56c892cde9beeceac5bf0f6b"

[[package]]
name = "ashpd"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd884d7c72877a94102c3715f3b1cd09ff4fac28221add3e57cfbe25c236d093"
dependencies = [
 "async-fs",
 "async-net",
 "enumflags2",
 "futures-channel",
 "futures-util",
 "rand",
 "serde",
 "serde_repr",
 "url",
 "zbus",
]

[[package]]
name = "async-broadcast"
version = "0.7.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "435a87a52755b8f27fcf321ac4f04b2802e337c8c4872923137471ec39c37532"
dependencies = [
 "event-listener",
 "event-listener-strategy",
 "futures-core",
 "pin-project-lite",
]

[[package]]
name = "async-channel"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "924ed96dd52d1b75e9c1a3e6275715fd320f5f9439fb5a4a11fa51f4221158d2"
dependencies = [
 "concurrent-queue",
 "event-listener-strategy",
 "futures-core",
 "pin-project-lite",
]

[[package]]
name = "async-executor"
version = "1.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c96bf972d85afc50bf5ab8fe2d54d1586b4e0b46c97c50a0c9e71e2f7bcd812a"
dependencies = [
 "async-task",
 "concurrent-queue",
 "fastrand",
 "futures-lite",
 "pin-project-lite",
 "slab",
]

[[package]]
name = "async-fs"
version = "2.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8034a681df4aed8b8edbd7fbe472401ecf009251c8b40556b304567052e294c5"
dependencies = [
 "async-lock",
 "blocking",
 "futures-lite",
]

[[package]]
name = "async-io"
version = "2.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "456b8a8feb6f42d237746d4b3e9a178494627745c3c56c6ea55d92ba50d026fc"
dependencies = [
 "autocfg",
 "cfg-if",
 "concurrent-queue",
 "futures-io",
 "futures-lite",
 "parking",
 "polling",
 "rustix 1.1.4",
 "slab",
 "windows-sys 0.61.2",
]

[[package]]
name = "async-lock"
version = "3.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "290f7f2596bd5b78a9fec8088ccd89180d7f9f55b94b0576823bbbdc72ee8311"
dependencies = [
 "event-listener",
 "event-listener-strategy",
 "pin-project-lite",
]

[[package]]
name = "async-net"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b948000fad4873c1c9339d60f2623323a0cfd3816e5181033c6a5cb68b2accf7"
dependencies = [
 "async-io",
 "blocking",
 "futures-lite",
]

[[package]]
name = "async-process"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc50921ec0055cdd8a16de48773bfeec5c972598674347252c0399676be7da75"
dependencies = [
 "async-channel",
 "async-io",
 "async-lock",
 "async-signal",
 "async-task",
 "blocking",
 "cfg-if",
 "event-listener",
 "futures-lite",
 "rustix 1.1.4",
]

[[package]]
name = "async-recursion"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b43422f69d8ff38f95f1b2bb76517c91589a924d1559a0e935d7c8ce0274c11"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "async-signal"
version = "0.2.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "43c070bbf59cd3570b6b2dd54cd772527c7c3620fce8be898406dd3ed6adc64c"
dependencies = [
 "async-io",
 "async-lock",
 "atomic-waker",
 "cfg-if",
 "futures-core",
 "futures-io",
 "rustix 1.1.4",
 "signal-hook-registry",
 "slab",
 "windows-sys 0.61.2",
]

[[package]]
name = "async-task"
version = "4.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b75356056920673b02621b35afd0f7dda9306d03c79a30f5c56c44cf256e3de"

[[package]]
name = "async-trait"
version = "0.1.89"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9035ad2d096bed7955a320ee7e2230574d28fd3c3a0f186cbea1ff3c7eed5dbb"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "atomic-waker"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0"

[[package]]
name = "autocfg"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c08606f8c3cbf4ce6ec8e28fb0014a2c086708fe954eaa885384a6165172e7e8"

[[package]]
name = "base64"
version = "0.21.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9d297deb1925b89f2ccc13d7635fa0714f12c87adce1c75356b39ca9b7178567"

[[package]]
name = "bitflags"
version = "1.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"

[[package]]
name = "bitflags"
version = "2.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "843867be96c8daad0d758b57df9392b6d8d271134fce549de6ce169ff98a92af"
dependencies = [
 "serde_core",
]

[[package]]
name = "block"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a"

[[package]]
name = "block-buffer"
version = "0.10.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71"
dependencies = [
 "generic-array",
]

[[package]]
name = "block-sys"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae85a0696e7ea3b835a453750bf002770776609115e6d25c6d2ff28a8200f7e7"
dependencies = [
 "objc-sys",
]

[[package]]
name = "block2"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "15b55663a85f33501257357e6421bb33e769d5c9ffb5ba0921c975a123e35e68"
dependencies = [
 "block-sys",
 "objc2 0.4.1",
]

[[package]]
name = "block2"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c132eebf10f5cad5289222520a4a058514204aed6d791f1cf4fe8088b82d15f"
dependencies = [
 "objc2 0.5.2",
]

[[package]]
name = "blocking"
version = "1.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e83f8d02be6967315521be875afa792a316e28d57b5a2d401897e2a7921b7f21"
dependencies = [
 "async-channel",
 "async-task",
 "futures-io",
 "futures-lite",
 "piper",
]

[[package]]
name = "bumpalo"
version = "3.20.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5d20789868f4b01b2f2caec9f5c4e0213b41e3e5702a50157d699ae31ced2fcb"

[[package]]
name = "bytemuck"
version = "1.25.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c8efb64bd706a16a1bdde310ae86b351e4d21550d98d056f22f8a7f7a2183fec"
dependencies = [
 "bytemuck_derive",
]

[[package]]
name = "bytemuck_derive"
version = "1.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f9abbd1bc6865053c427f7198e6af43bfdedc55ab791faed4fbd361d789575ff"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "byteorder-lite"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f1fe948ff07f4bd06c30984e69f5b4899c516a3ef74f34df92a2df2ab535495"

[[package]]
name = "bytes"
version = "1.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33"

[[package]]
name = "calloop"
version = "0.12.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fba7adb4dd5aa98e5553510223000e7148f621165ec5f9acd7113f6ca4995298"
dependencies = [
 "bitflags 2.11.0",
 "log",
 "polling",
 "rustix 0.38.44",
 "slab",
 "thiserror 1.0.69",
]

[[package]]
name = "calloop"
version = "0.14.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4dbf9978365bac10f54d1d4b04f7ce4427e51f71d61f2fe15e3fed5166474df7"
dependencies = [
 "bitflags 2.11.0",
 "polling",
 "rustix 1.1.4",
 "slab",
 "tracing",
]

[[package]]
name = "calloop-wayland-source"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0f0ea9b9476c7fad82841a8dbb380e2eae480c21910feba80725b46931ed8f02"
dependencies = [
 "calloop 0.12.4",
 "rustix 0.38.44",
 "wayland-backend",
 "wayland-client",
]

[[package]]
name = "calloop-wayland-source"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "138efcf0940a02ebf0cc8d1eff41a1682a46b431630f4c52450d6265876021fa"
dependencies = [
 "calloop 0.14.4",
 "rustix 1.1.4",
 "wayland-backend",
 "wayland-client",
]

[[package]]
name = "cc"
version = "1.2.56"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aebf35691d1bfb0ac386a69bac2fde4dd276fb618cf8bf4f5318fe285e821bb2"
dependencies = [
 "find-msvc-tools",
 "jobserver",
 "libc",
 "shlex",
]

[[package]]
name = "cesu8"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6d43a04d8753f35258c91f8ec639f792891f748a1edbd759cf1dcea3382ad83c"

[[package]]
name = "cfg-if"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801"

[[package]]
name = "cfg_aliases"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fd16c4719339c4530435d38e511904438d07cce7950afa3718a84ac36c10e89e"

[[package]]
name = "cfg_aliases"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724"

[[package]]
name = "cgl"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ced0551234e87afee12411d535648dd89d2e7f34c78b753395567aff3d447ff"
dependencies = [
 "libc",
]

[[package]]
name = "clipboard-win"
version = "5.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bde03770d3df201d4fb868f2c9c59e66a3e4e2bd06692a0fe701e7103c7e84d4"
dependencies = [
 "error-code",
]

[[package]]
name = "combine"
version = "4.6.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ba5a308b75df32fe02788e748662718f03fde005016435c444eea572398219fd"
dependencies = [
 "bytes",
 "memchr",
]

[[package]]
name = "concurrent-queue"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ca0197aee26d1ae37445ee532fefce43251d24cc7c166799f4d46817f1d3973"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "core-foundation"
version = "0.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation-sys"
version = "0.8.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b"

[[package]]
name = "core-graphics"
version = "0.23.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c07782be35f9e1140080c6b96f0d44b739e2278479f64e02fdab4e32dfd8b081"
dependencies = [
 "bitflags 1.3.2",
 "core-foundation 0.9.4",
 "core-graphics-types",
 "foreign-types",
 "libc",
]

[[package]]
name = "core-graphics-types"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "45390e6114f68f718cc7a830514a96f903cccd70d02a8f6d9f643ac4ba45afaf"
dependencies = [
 "bitflags 1.3.2",
 "core-foundation 0.9.4",
 "libc",
]

[[package]]
name = "cpufeatures"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280"
dependencies = [
 "libc",
]

[[package]]
name = "crc32fast"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511"
dependencies = [
 "cfg-if",
]

[[package]]
name = "crossbeam-utils"
version = "0.8.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d0a5c400df2834b80a4c3327b3aad3a4c4cd4de0629063962b03235697506a28"

[[package]]
name = "crunchy"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5"

[[package]]
name = "crypto-common"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a"
dependencies = [
 "generic-array",
 "typenum",
]

[[package]]
name = "cursor-icon"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f27ae1dd37df86211c42e150270f82743308803d90a6f6e6651cd730d5e1732f"

[[package]]
name = "digest"
version = "0.10.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292"
dependencies = [
 "block-buffer",
 "crypto-common",
]

[[package]]
name = "directories"
version = "5.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a49173b84e034382284f27f1af4dcbbd231ffa358c0fe316541a7337f376a35"
dependencies = [
 "dirs-sys",
]

[[package]]
name = "dirs-sys"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "520f05a5cbd335fae5a99ff7a6ab8627577660ee5cfd6a94a6a929b52ff0321c"
dependencies = [
 "libc",
 "option-ext",
 "redox_users",
 "windows-sys 0.48.0",
]

[[package]]
name = "dispatch"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bd0c93bb4b0c6d9b77f4435b0ae98c24d17f1c45b2ff844c6151a07256ca923b"

[[package]]
name = "dispatch2"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e0e367e4e7da84520dedcac1901e4da967309406d1e51017ae1abfb97adbd38"
dependencies = [
 "bitflags 2.11.0",
 "objc2 0.6.4",
]

[[package]]
name = "displaydoc"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "97369cbbc041bc366949bc74d34658d6cda5621039731c6310521892a3a20ae0"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "dlib"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab8ecd87370524b461f8557c119c405552c396ed91fc0a8eec68679eab26f94a"
dependencies = [
 "libloading",
]

[[package]]
name = "document-features"
version = "0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d4b8a88685455ed29a21542a33abd9cb6510b6b129abadabdcef0f4c55bc8f61"
dependencies = [
 "litrs",
]

[[package]]
name = "downcast-rs"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75b325c5dbd37f80359721ad39aca5a29fb04c89279657cffdda8736d0c0b9d2"

[[package]]
name = "ecolor"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2e6b451ff1143f6de0f33fc7f1b68fecfd2c7de06e104de96c4514de3f5396f8"
dependencies = [
 "bytemuck",
 "emath",
 "serde",
]

[[package]]
name = "eframe"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6490ef800b2e41ee129b1f32f9ac15f713233fe3bc18e241a1afe1e4fb6811e0"
dependencies = [
 "ahash",
 "bytemuck",
 "directories",
 "document-features",
 "egui",
 "egui-winit",
 "egui_glow",
 "glow",
 "glutin",
 "glutin-winit",
 "image",
 "js-sys",
 "log",
 "objc2 0.5.2",
 "objc2-app-kit 0.2.2",
 "objc2-foundation 0.2.2",
 "parking_lot",
 "percent-encoding",
 "raw-window-handle 0.5.2",
 "raw-window-handle 0.6.2",
 "ron",
 "serde",
 "static_assertions",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-sys",
 "web-time",
 "winapi",
 "winit",
]

[[package]]
name = "egui"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "20c97e70a2768de630f161bb5392cbd3874fcf72868f14df0e002e82e06cb798"
dependencies = [
 "accesskit",
 "ahash",
 "emath",
 "epaint",
 "log",
 "nohash-hasher",
 "ron",
 "serde",
]

[[package]]
name = "egui-winit"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fac4e066af341bf92559f60dbdf2020b2a03c963415349af5f3f8d79ff7a4926"
dependencies = [
 "ahash",
 "arboard",
 "egui",
 "log",
 "raw-window-handle 0.6.2",
 "serde",
 "smithay-clipboard",
 "web-time",
 "webbrowser",
 "winit",
]

[[package]]
name = "egui_glow"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e2bdc8b38cfa17cc712c4ae079e30c71c00cd4c2763c9e16dc7860a02769103"
dependencies = [
 "ahash",
 "bytemuck",
 "egui",
 "glow",
 "log",
 "memoffset",
 "wasm-bindgen",
 "web-sys",
]

[[package]]
name = "emath"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0a6a21708405ea88f63d8309650b4d77431f4bc28fb9d8e6f77d3963b51249e6"
dependencies = [
 "bytemuck",
 "serde",
]

[[package]]
name = "encoding_rs"
version = "0.8.35"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3"
dependencies = [
 "cfg-if",
]

[[package]]
name = "endi"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "66b7e2430c6dff6a955451e2cfc438f09cea1965a9d6f87f7e3b90decc014099"

[[package]]
name = "enumflags2"
version = "0.7.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1027f7680c853e056ebcec683615fb6fbbc07dbaa13b4d5d9442b146ded4ecef"
dependencies = [
 "enumflags2_derive",
 "serde",
]

[[package]]
name = "enumflags2_derive"
version = "0.7.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67c78a4d8fdf9953a5c9d458f9efe940fd97a0cab0941c075a813ac594733827"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "enumn"
version = "0.1.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2f9ed6b3789237c8a0c1c505af1c7eb2c560df6186f01b098c3a1064ea532f38"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "epaint"
version = "0.28.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f0dcc0a0771e7500e94cd1cb797bd13c9f23b9409bdc3c824e2cbc562b7fa01"
dependencies = [
 "ab_glyph",
 "ahash",
 "bytemuck",
 "ecolor",
 "emath",
 "log",
 "nohash-hasher",
 "parking_lot",
 "serde",
]

[[package]]
name = "equivalent"
version = "1.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f"

[[package]]
name = "errno"
version = "0.3.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb"
dependencies = [
 "libc",
 "windows-sys 0.61.2",
]

[[package]]
name = "error-code"
version = "3.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dea2df4cf52843e0452895c455a1a2cfbb842a1e7329671acf418fdc53ed4c59"

[[package]]
name = "event-listener"
version = "5.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e13b66accf52311f30a0db42147dadea9850cb48cd070028831ae5f5d4b856ab"
dependencies = [
 "concurrent-queue",
 "parking",
 "pin-project-lite",
]

[[package]]
name = "event-listener-strategy"
version = "0.5.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8be9f3dfaaffdae2972880079a491a1a8bb7cbed0b8dd7a347f668b4150a3b93"
dependencies = [
 "event-listener",
 "pin-project-lite",
]

[[package]]
name = "fastrand"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "37909eebbb50d72f9059c3b6d82c0463f2ff062c9e95845c43a6c9c0355411be"

[[package]]
name = "fax"
version = "0.2.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f05de7d48f37cd6730705cbca900770cab77a89f413d23e100ad7fad7795a0ab"
dependencies = [
 "fax_derive",
]

[[package]]
name = "fax_derive"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0aca10fb742cb43f9e7bb8467c91aa9bcb8e3ffbc6a6f7389bb93ffc920577d"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "fdeflate"
version = "0.3.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e6853b52649d4ac5c0bd02320cddc5ba956bdb407c4b75a2c6b75bf51500f8c"
dependencies = [
 "simd-adler32",
]

[[package]]
name = "find-msvc-tools"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5baebc0774151f905a1a2cc41989300b1e6fbb29aff0ceffa1064fdd3088d582"

[[package]]
name = "flate2"
version = "1.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "843fba2746e448b37e26a819579957415c8cef339bf08564fe8b7ddbd959573c"
dependencies = [
 "crc32fast",
 "miniz_oxide",
]

[[package]]
name = "foldhash"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2"

[[package]]
name = "foreign-types"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d737d9aa519fb7b749cbc3b962edcf310a8dd1f4b67c91c4f83975dbdd17d965"
dependencies = [
 "foreign-types-macros",
 "foreign-types-shared",
]

[[package]]
name = "foreign-types-macros"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a5c6c585bc94aaf2c7b51dd4c2ba22680844aba4c687be581871a6f518c5742"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "foreign-types-shared"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aa9a19cbb55df58761df49b23516a86d432839add4af60fc256da840f66ed35b"

[[package]]
name = "form_urlencoded"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf"
dependencies = [
 "percent-encoding",
]

[[package]]
name = "futures-channel"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "07bbe89c50d7a535e539b8c17bc0b49bdb77747034daa8087407d655f3f7cc1d"
dependencies = [
 "futures-core",
]

[[package]]
name = "futures-core"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e3450815272ef58cec6d564423f6e755e25379b217b0bc688e295ba24df6b1d"

[[package]]
name = "futures-io"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cecba35d7ad927e23624b22ad55235f2239cfa44fd10428eecbeba6d6a717718"

[[package]]
name = "futures-lite"
version = "2.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f78e10609fe0e0b3f4157ffab1876319b5b0db102a2c60dc4626306dc46b44ad"
dependencies = [
 "fastrand",
 "futures-core",
 "futures-io",
 "parking",
 "pin-project-lite",
]

[[package]]
name = "futures-macro"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e835b70203e41293343137df5c0664546da5745f82ec9b84d40be8336958447b"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "futures-sink"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c39754e157331b013978ec91992bde1ac089843443c49cbc7f46150b0fad0893"

[[package]]
name = "futures-task"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "037711b3d59c33004d3856fbdc83b99d4ff37a24768fa1be9ce3538a1cde4393"

[[package]]
name = "futures-util"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "389ca41296e6190b48053de0321d02a77f32f8a5d2461dd38762c0593805c6d6"
dependencies = [
 "futures-core",
 "futures-io",
 "futures-macro",
 "futures-sink",
 "futures-task",
 "memchr",
 "pin-project-lite",
 "slab",
]

[[package]]
name = "generic-array"
version = "0.14.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a"
dependencies = [
 "typenum",
 "version_check",
]

[[package]]
name = "gethostname"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1bd49230192a3797a9a4d6abe9b3eed6f7fa4c8a8a4947977c6f80025f92cbd8"
dependencies = [
 "rustix 1.1.4",
 "windows-link",
]

[[package]]
name = "getrandom"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0"
dependencies = [
 "cfg-if",
 "libc",
 "wasi",
]

[[package]]
name = "getrandom"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
dependencies = [
 "cfg-if",
 "libc",
 "r-efi",
 "wasip2",
]

[[package]]
name = "getrandom"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "139ef39800118c7683f2fd3c98c1b23c09ae076556b435f8e9064ae108aaeeec"
dependencies = [
 "cfg-if",
 "libc",
 "r-efi",
 "wasip2",
 "wasip3",
]

[[package]]
name = "gl_generator"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a95dfc23a2b4a9a2f5ab41d194f8bfda3cabec42af4e39f08c339eb2a0c124d"
dependencies = [
 "khronos_api",
 "log",
 "xml-rs",
]

[[package]]
name = "glow"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bd348e04c43b32574f2de31c8bb397d96c9fcfa1371bd4ca6d8bdc464ab121b1"
dependencies = [
 "js-sys",
 "slotmap",
 "wasm-bindgen",
 "web-sys",
]

[[package]]
name = "glutin"
version = "0.31.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "18fcd4ae4e86d991ad1300b8f57166e5be0c95ef1f63f3f5b827f8a164548746"
dependencies = [
 "bitflags 2.11.0",
 "cfg_aliases 0.1.1",
 "cgl",
 "core-foundation 0.9.4",
 "dispatch",
 "glutin_egl_sys",
 "glutin_glx_sys",
 "glutin_wgl_sys",
 "icrate",
 "libloading",
 "objc2 0.4.1",
 "once_cell",
 "raw-window-handle 0.5.2",
 "wayland-sys",
 "windows-sys 0.48.0",
 "x11-dl",
]

[[package]]
name = "glutin-winit"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ebcdfba24f73b8412c5181e56f092b5eff16671c514ce896b258a0a64bd7735"
dependencies = [
 "cfg_aliases 0.1.1",
 "glutin",
 "raw-window-handle 0.5.2",
 "winit",
]

[[package]]
name = "glutin_egl_sys"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77cc5623f5309ef433c3dd4ca1223195347fe62c413da8e2fdd0eb76db2d9bcd"
dependencies = [
 "gl_generator",
 "windows-sys 0.48.0",
]

[[package]]
name = "glutin_glx_sys"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a165fd686c10dcc2d45380b35796e577eacfd43d4660ee741ec8ebe2201b3b4f"
dependencies = [
 "gl_generator",
 "x11-dl",
]

[[package]]
name = "glutin_wgl_sys"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6c8098adac955faa2d31079b65dc48841251f69efd3ac25477903fc424362ead"
dependencies = [
 "gl_generator",
]

[[package]]
name = "half"
version = "2.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ea2d84b969582b4b1864a92dc5d27cd2b77b622a8d79306834f1be5ba20d84b"
dependencies = [
 "cfg-if",
 "crunchy",
 "zerocopy",
]

[[package]]
name = "hashbrown"
version = "0.15.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1"
dependencies = [
 "foldhash",
]

[[package]]
name = "hashbrown"
version = "0.16.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "841d1cc9bed7f9236f321df977030373f4a4163ae1a7dbfe1a51a2c1a51d9100"

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "hermit-abi"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc0fef456e4baa96da950455cd02c081ca953b141298e41db3fc7e36b1da849c"

[[package]]
name = "hex"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70"

[[package]]
name = "icrate"
version = "0.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "99d3aaff8a54577104bafdf686ff18565c3b6903ca5782a2026ef06e2c7aa319"
dependencies = [
 "block2 0.3.0",
 "dispatch",
 "objc2 0.4.1",
]

[[package]]
name = "icu_collections"
version = "2.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4c6b649701667bbe825c3b7e6388cb521c23d88644678e83c0c4d0a621a34b43"
dependencies = [
 "displaydoc",
 "potential_utf",
 "yoke",
 "zerofrom",
 "zerovec",
]

[[package]]
name = "icu_locale_core"
version = "2.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "edba7861004dd3714265b4db54a3c390e880ab658fec5f7db895fae2046b5bb6"
dependencies = [
 "displaydoc",
 "litemap",
 "tinystr",
 "writeable",
 "zerovec",
]

[[package]]
name = "icu_normalizer"
version = "2.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5f6c8828b67bf8908d82127b2054ea1b4427ff0230ee9141c54251934ab1b599"
dependencies = [
 "icu_collections",
 "icu_normalizer_data",
 "icu_properties",
 "icu_provider",
 "smallvec",
 "zerovec",
]

[[package]]
name = "icu_normalizer_data"
version = "2.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7aedcccd01fc5fe81e6b489c15b247b8b0690feb23304303a9e560f37efc560a"

[[package]]
name = "icu_properties"
version = "2.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "020bfc02fe870ec3a66d93e677ccca0562506e5872c650f893269e08615d74ec"
dependencies = [
 "icu_collections",
 "icu_locale_core",
 "icu_properties_data",
 "icu_provider",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "icu_properties_data"
version = "2.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "616c294cf8d725c6afcd8f55abc17c56464ef6211f9ed59cccffe534129c77af"

[[package]]
name = "icu_provider"
version = "2.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85962cf0ce02e1e0a629cc34e7ca3e373ce20dda4c4d7294bbd0bf1fdb59e614"
dependencies = [
 "displaydoc",
 "icu_locale_core",
 "writeable",
 "yoke",
 "zerofrom",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "id-arena"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3d3067d79b975e8844ca9eb072e16b31c3c1c36928edf9c6789548c524d0d954"

[[package]]
name = "idna"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de"
dependencies = [
 "idna_adapter",
 "smallvec",
 "utf8_iter",
]

[[package]]
name = "idna_adapter"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3acae9609540aa318d1bc588455225fb2085b9ed0c4f6bd0d9d5bcd86f1a0344"
dependencies = [
 "icu_normalizer",
 "icu_properties",
]

[[package]]
name = "image"
version = "0.25.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6506c6c10786659413faa717ceebcb8f70731c0a60cbae39795fdf114519c1a"
dependencies = [
 "bytemuck",
 "byteorder-lite",
 "moxcms",
 "num-traits",
 "png",
 "tiff",
]

[[package]]
name = "indexmap"
version = "2.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7714e70437a7dc3ac8eb7e6f8df75fd8eb422675fc7678aff7364301092b1017"
dependencies = [
 "equivalent",
 "hashbrown 0.16.1",
 "serde",
 "serde_core",
]

[[package]]
name = "itoa"
version = "1.0.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92ecc6618181def0457392ccd0ee51198e065e016d1d527a7ac1b6dc7c1f09d2"

[[package]]
name = "jni"
version = "0.21.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a87aa2bb7d2af34197c04845522473242e1aa17c12f4935d5856491a7fb8c97"
dependencies = [
 "cesu8",
 "cfg-if",
 "combine",
 "jni-sys",
 "log",
 "thiserror 1.0.69",
 "walkdir",
 "windows-sys 0.45.0",
]

[[package]]
name = "jni-sys"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8eaf4bc02d17cbdd7ff4c7438cafcdf7fb9a4613313ad11b4f8fefe7d3fa0130"

[[package]]
name = "jobserver"
version = "0.1.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9afb3de4395d6b3e67a780b6de64b51c978ecf11cb9a462c66be7d4ca9039d33"
dependencies = [
 "getrandom 0.3.4",
 "libc",
]

[[package]]
name = "js-sys"
version = "0.3.91"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b49715b7073f385ba4bc528e5747d02e66cb39c6146efb66b781f131f0fb399c"
dependencies = [
 "once_cell",
 "wasm-bindgen",
]

[[package]]
name = "khronos_api"
version = "3.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e2db585e1d738fc771bf08a151420d3ed193d9d895a36df7f6f8a9456b911ddc"

[[package]]
name = "leb128fmt"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09edd9e8b54e49e587e4f6295a7d29c3ea94d469cb40ab8ca70b288248a81db2"

[[package]]
name = "libc"
version = "0.2.182"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6800badb6cb2082ffd7b6a67e6125bb39f18782f793520caee8cb8846be06112"

[[package]]
name = "libloading"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d7c4b02199fee7c5d21a5ae7d8cfa79a6ef5bb2fc834d6e9058e89c825efdc55"
dependencies = [
 "cfg-if",
 "windows-link",
]

[[package]]
name = "libm"
version = "0.2.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981"

[[package]]
name = "libredox"
version = "0.1.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1744e39d1d6a9948f4f388969627434e31128196de472883b39f148769bfe30a"
dependencies = [
 "bitflags 2.11.0",
 "libc",
 "plain",
 "redox_syscall 0.7.3",
]

[[package]]
name = "linux-raw-sys"
version = "0.4.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab"

[[package]]
name = "linux-raw-sys"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"

[[package]]
name = "litemap"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77"

[[package]]
name = "litrs"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11d3d7f243d5c5a8b9bb5d6dd2b1602c0cb0b9db1621bafc7ed66e35ff9fe092"

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "log"
version = "0.4.29"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e5032e24019045c762d3c0f28f5b6b8bbf38563a65908389bf7978758920897"

[[package]]
name = "malloc_buf"
version = "0.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "62bb907fe88d54d8d9ce32a3cceab4218ed2f6b7d35617cafe9adf84e43919cb"
dependencies = [
 "libc",
]

[[package]]
name = "memchr"
version = "2.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f8ca58f447f06ed17d5fc4043ce1b10dd205e060fb3ce5b979b8ed8e59ff3f79"

[[package]]
name = "memmap2"
version = "0.9.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "714098028fe011992e1c3962653c96b2d578c4b4bce9036e15ff220319b1e0e3"
dependencies = [
 "libc",
]

[[package]]
name = "memoffset"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "488016bfae457b036d996092f6cb448677611ce4449e970ceaf42695203f218a"
dependencies = [
 "autocfg",
]

[[package]]
name = "miniz_oxide"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316"
dependencies = [
 "adler2",
 "simd-adler32",
]

[[package]]
name = "moxcms"
version = "0.7.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ac9557c559cd6fc9867e122e20d2cbefc9ca29d80d027a8e39310920ed2f0a97"
dependencies = [
 "num-traits",
 "pxfm",
]

[[package]]
name = "ndk"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2076a31b7010b17a38c01907c45b945e8f11495ee4dd588309718901b1f7a5b7"
dependencies = [
 "bitflags 2.11.0",
 "jni-sys",
 "log",
 "ndk-sys",
 "num_enum",
 "raw-window-handle 0.5.2",
 "raw-window-handle 0.6.2",
 "thiserror 1.0.69",
]

[[package]]
name = "ndk-context"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "27b02d87554356db9e9a873add8782d4ea6e3e58ea071a9adb9a2e8ddb884a8b"

[[package]]
name = "ndk-sys"
version = "0.5.0+25.2.9519653"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8c196769dd60fd4f363e11d948139556a344e79d451aeb2fa2fd040738ef7691"
dependencies = [
 "jni-sys",
]

[[package]]
name = "nix"
version = "0.29.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "71e2746dc3a24dd78b3cfcb7be93368c6de9963d30f43a6a73998a9cf4b17b46"
dependencies = [
 "bitflags 2.11.0",
 "cfg-if",
 "cfg_aliases 0.2.1",
 "libc",
 "memoffset",
]

[[package]]
name = "nohash-hasher"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2bf50223579dc7cdcfb3bfcacf7069ff68243f8c363f62ffa99cf000a6b9c451"

[[package]]
name = "num-traits"
version = "0.2.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
dependencies = [
 "autocfg",
 "libm",
]

[[package]]
name = "num_enum"
version = "0.7.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b1207a7e20ad57b847bbddc6776b968420d38292bbfe2089accff5e19e82454c"
dependencies = [
 "num_enum_derive",
 "rustversion",
]

[[package]]
name = "num_enum_derive"
version = "0.7.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff32365de1b6743cb203b710788263c44a03de03802daf96092f2da4fe6ba4d7"
dependencies = [
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "objc"
version = "0.2.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "915b1b472bc21c53464d6c8461c9d3af805ba1ef837e1cac254428f4a77177b1"
dependencies = [
 "malloc_buf",
]

[[package]]
name = "objc-foundation"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1add1b659e36c9607c7aab864a76c7a4c2760cd0cd2e120f3fb8b952c7e22bf9"
dependencies = [
 "block",
 "objc",
 "objc_id",
]

[[package]]
name = "objc-sys"
version = "0.3.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdb91bdd390c7ce1a8607f35f3ca7151b65afc0ff5ff3b34fa350f7d7c7e4310"

[[package]]
name = "objc2"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "559c5a40fdd30eb5e344fbceacf7595a81e242529fb4e21cf5f43fb4f11ff98d"
dependencies = [
 "objc-sys",
 "objc2-encode 3.0.0",
]

[[package]]
name = "objc2"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "46a785d4eeff09c14c487497c162e92766fbb3e4059a71840cecc03d9a50b804"
dependencies = [
 "objc-sys",
 "objc2-encode 4.1.0",
]

[[package]]
name = "objc2"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3a12a8ed07aefc768292f076dc3ac8c48f3781c8f2d5851dd3d98950e8c5a89f"
dependencies = [
 "objc2-encode 4.1.0",
]

[[package]]
name = "objc2-app-kit"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e4e89ad9e3d7d297152b17d39ed92cd50ca8063a89a9fa569046d41568891eff"
dependencies = [
 "bitflags 2.11.0",
 "block2 0.5.1",
 "libc",
 "objc2 0.5.2",
 "objc2-core-data",
 "objc2-core-image",
 "objc2-foundation 0.2.2",
 "objc2-quartz-core",
]

[[package]]
name = "objc2-app-kit"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d49e936b501e5c5bf01fda3a9452ff86dc3ea98ad5f283e1455153142d97518c"
dependencies = [
 "bitflags 2.11.0",
 "objc2 0.6.4",
 "objc2-core-graphics",
 "objc2-foundation 0.3.2",
]

[[package]]
name = "objc2-core-data"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "617fbf49e071c178c0b24c080767db52958f716d9eabdf0890523aeae54773ef"
dependencies = [
 "bitflags 2.11.0",
 "block2 0.5.1",
 "objc2 0.5.2",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-core-foundation"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536"
dependencies = [
 "bitflags 2.11.0",
 "dispatch2",
 "objc2 0.6.4",
]

[[package]]
name = "objc2-core-graphics"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e022c9d066895efa1345f8e33e584b9f958da2fd4cd116792e15e07e4720a807"
dependencies = [
 "bitflags 2.11.0",
 "dispatch2",
 "objc2 0.6.4",
 "objc2-core-foundation",
 "objc2-io-surface",
]

[[package]]
name = "objc2-core-image"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "55260963a527c99f1819c4f8e3b47fe04f9650694ef348ffd2227e8196d34c80"
dependencies = [
 "block2 0.5.1",
 "objc2 0.5.2",
 "objc2-foundation 0.2.2",
 "objc2-metal",
]

[[package]]
name = "objc2-encode"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d079845b37af429bfe5dfa76e6d087d788031045b25cfc6fd898486fd9847666"

[[package]]
name = "objc2-encode"
version = "4.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ef25abbcd74fb2609453eb695bd2f860d389e457f67dc17cafc8b8cbc89d0c33"

[[package]]
name = "objc2-foundation"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ee638a5da3799329310ad4cfa62fbf045d5f56e3ef5ba4149e7452dcf89d5a8"
dependencies = [
 "bitflags 2.11.0",
 "block2 0.5.1",
 "libc",
 "objc2 0.5.2",
]

[[package]]
name = "objc2-foundation"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3e0adef53c21f888deb4fa59fc59f7eb17404926ee8a6f59f5df0fd7f9f3272"
dependencies = [
 "bitflags 2.11.0",
 "objc2 0.6.4",
 "objc2-core-foundation",
]

[[package]]
name = "objc2-io-surface"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "180788110936d59bab6bd83b6060ffdfffb3b922ba1396b312ae795e1de9d81d"
dependencies = [
 "bitflags 2.11.0",
 "objc2 0.6.4",
 "objc2-core-foundation",
]

[[package]]
name = "objc2-metal"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd0cba1276f6023976a406a14ffa85e1fdd19df6b0f737b063b95f6c8c7aadd6"
dependencies = [
 "bitflags 2.11.0",
 "block2 0.5.1",
 "objc2 0.5.2",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-quartz-core"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e42bee7bff906b14b167da2bac5efe6b6a07e6f7c0a21a7308d40c960242dc7a"
dependencies = [
 "bitflags 2.11.0",
 "block2 0.5.1",
 "objc2 0.5.2",
 "objc2-foundation 0.2.2",
 "objc2-metal",
]

[[package]]
name = "objc_id"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c92d4ddb4bd7b50d730c215ff871754d0da6b2178849f8a2a2ab69712d0c073b"
dependencies = [
 "objc",
]

[[package]]
name = "once_cell"
version = "1.21.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "42f5e15c9953c5e4ccceeb2e7382a716482c34515315f7b03532b8b4e8393d2d"

[[package]]
name = "option-ext"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "04744f49eae99ab78e0d5c0b603ab218f515ea8cfe5a456d7629ad883a3b6e7d"

[[package]]
name = "orbclient"
version = "0.3.50"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "52ad2c6bae700b7aa5d1cc30c59bdd3a1c180b09dbaea51e2ae2b8e1cf211fdd"
dependencies = [
 "libc",
 "libredox",
]

[[package]]
name = "ordered-stream"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9aa2b01e1d916879f73a53d01d1d6cee68adbb31d6d9177a8cfce093cced1d50"
dependencies = [
 "futures-core",
 "pin-project-lite",
]

[[package]]
name = "owned_ttf_parser"
version = "0.25.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "36820e9051aca1014ddc75770aab4d68bc1e9e632f0f5627c4086bc216fb583b"
dependencies = [
 "ttf-parser",
]

[[package]]
name = "parking"
version = "2.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f38d5652c16fde515bb1ecef450ab0f6a219d619a7274976324d5e377f7dceba"

[[package]]
name = "parking_lot"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
dependencies = [
 "lock_api",
 "parking_lot_core",
]

[[package]]
name = "parking_lot_core"
version = "0.9.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
dependencies = [
 "cfg-if",
 "libc",
 "redox_syscall 0.5.18",
 "smallvec",
 "windows-link",
]

[[package]]
name = "percent-encoding"
version = "2.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"

[[package]]
name = "petri_net_legacy_editor"
version = "0.7.79"
dependencies = [
 "anyhow",
 "arboard",
 "eframe",
 "egui",
 "encoding_rs",
 "image",
 "rand",
 "rand_distr",
 "rfd",
 "serde",
 "serde_json",
 "tempfile",
 "winres",
]

[[package]]
name = "pin-project-lite"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd"

[[package]]
name = "piper"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c835479a4443ded371d6c535cbfd8d31ad92c5d23ae9770a61bc155e4992a3c1"
dependencies = [
 "atomic-waker",
 "fastrand",
 "futures-io",
]

[[package]]
name = "pkg-config"
version = "0.3.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7edddbd0b52d732b21ad9a5fab5c704c14cd949e5e9a1ec5929a24fded1b904c"

[[package]]
name = "plain"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b4596b6d070b27117e987119b4dac604f3c58cfb0b191112e24771b2faeac1a6"

[[package]]
name = "png"
version = "0.18.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "60769b8b31b2a9f263dae2776c37b1b28ae246943cf719eb6946a1db05128a61"
dependencies = [
 "bitflags 2.11.0",
 "crc32fast",
 "fdeflate",
 "flate2",
 "miniz_oxide",
]

[[package]]
name = "polling"
version = "3.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5d0e4f59085d47d8241c88ead0f274e8a0cb551f3625263c05eb8dd897c34218"
dependencies = [
 "cfg-if",
 "concurrent-queue",
 "hermit-abi",
 "pin-project-lite",
 "rustix 1.1.4",
 "windows-sys 0.61.2",
]

[[package]]
name = "pollster"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "22686f4785f02a4fcc856d3b3bb19bf6c8160d103f7a99cc258bddd0251dc7f2"

[[package]]
name = "potential_utf"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b73949432f5e2a09657003c25bca5e19a0e9c84f8058ca374f49e0ebe605af77"
dependencies = [
 "zerovec",
]

[[package]]
name = "ppv-lite86"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9"
dependencies = [
 "zerocopy",
]

[[package]]
name = "prettyplease"
version = "0.2.37"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b"
dependencies = [
 "proc-macro2",
 "syn",
]

[[package]]
name = "proc-macro-crate"
version = "3.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "219cb19e96be00ab2e37d6e299658a0cfa83e52429179969b0f0121b4ac46983"
dependencies = [
 "toml_edit",
]

[[package]]
name = "proc-macro2"
version = "1.0.106"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "pxfm"
version = "0.1.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7186d3822593aa4393561d186d1393b3923e9d6163d3fbfd6e825e3e6cf3e6a8"
dependencies = [
 "num-traits",
]

[[package]]
name = "quick-error"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a993555f31e5a609f617c12db6250dedcac1b0a85076912c436e6fc9b2c8e6a3"

[[package]]
name = "quick-xml"
version = "0.38.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b66c2058c55a409d601666cffe35f04333cf1013010882cec174a7467cd4e21c"
dependencies = [
 "memchr",
]

[[package]]
name = "quote"
version = "1.0.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "21b2ebcf727b7760c461f091f9f0f539b77b8e87f2fd88131e7f1b433b3cece4"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "r-efi"
version = "5.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"

[[package]]
name = "rand"
version = "0.8.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "34af8d1a0e25924bc5b7c43c079c942339d8f0a8b57c39049bef581b46327404"
dependencies = [
 "libc",
 "rand_chacha",
 "rand_core",
]

[[package]]
name = "rand_chacha"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88"
dependencies = [
 "ppv-lite86",
 "rand_core",
]

[[package]]
name = "rand_core"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c"
dependencies = [
 "getrandom 0.2.17",
]

[[package]]
name = "rand_distr"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32cb0b9bc82b0a0876c2dd994a7e7a2683d3e7390ca40e6886785ef0c7e3ee31"
dependencies = [
 "num-traits",
 "rand",
]

[[package]]
name = "raw-window-handle"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2ff9a1f06a88b01621b7ae906ef0211290d1c8a168a15542486a8f61c0833b9"

[[package]]
name = "raw-window-handle"
version = "0.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "20675572f6f24e9e76ef639bc5552774ed45f1c30e2951e1e99c59888861c539"

[[package]]
name = "redox_syscall"
version = "0.3.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "567664f262709473930a4bf9e51bf2ebf3348f2e748ccc50dea20646858f8f29"
dependencies = [
 "bitflags 1.3.2",
]

[[package]]
name = "redox_syscall"
version = "0.5.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
dependencies = [
 "bitflags 2.11.0",
]

[[package]]
name = "redox_syscall"
version = "0.7.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce70a74e890531977d37e532c34d45e9055d2409ed08ddba14529471ed0be16"
dependencies = [
 "bitflags 2.11.0",
]

[[package]]
name = "redox_users"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ba009ff324d1fc1b900bd1fdb31564febe58a8ccc8a6fdbb93b543d33b13ca43"
dependencies = [
 "getrandom 0.2.17",
 "libredox",
 "thiserror 1.0.69",
]

[[package]]
name = "rfd"
version = "0.14.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "25a73a7337fc24366edfca76ec521f51877b114e42dab584008209cca6719251"
dependencies = [
 "ashpd",
 "block",
 "dispatch",
 "js-sys",
 "log",
 "objc",
 "objc-foundation",
 "objc_id",
 "pollster",
 "raw-window-handle 0.6.2",
 "urlencoding",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-sys",
 "windows-sys 0.48.0",
]

[[package]]
name = "ron"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b91f7eff05f748767f183df4320a63d6936e9c6107d97c9e6bdd9784f4289c94"
dependencies = [
 "base64",
 "bitflags 2.11.0",
 "serde",
 "serde_derive",
]

[[package]]
name = "rustix"
version = "0.38.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154"
dependencies = [
 "bitflags 2.11.0",
 "errno",
 "libc",
 "linux-raw-sys 0.4.15",
 "windows-sys 0.59.0",
]

[[package]]
name = "rustix"
version = "1.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190"
dependencies = [
 "bitflags 2.11.0",
 "errno",
 "libc",
 "linux-raw-sys 0.12.1",
 "windows-sys 0.61.2",
]

[[package]]
name = "rustversion"
version = "1.0.22"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d"

[[package]]
name = "same-file"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502"
dependencies = [
 "winapi-util",
]

[[package]]
name = "scoped-tls"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e1cf6437eb19a8f4a6cc0f7dca544973b0b78843adbfeb3683d1a94a0024a294"

[[package]]
name = "scopeguard"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"

[[package]]
name = "semver"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d767eb0aabc880b29956c35734170f26ed551a859dbd361d140cdbeca61ab1e2"

[[package]]
name = "serde"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e"
dependencies = [
 "serde_core",
 "serde_derive",
]

[[package]]
name = "serde_core"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "serde_json"
version = "1.0.149"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "83fc039473c5595ace860d8c4fafa220ff474b3fc6bfdb4293327f1a37e94d86"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "serde_repr"
version = "0.1.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175ee3e80ae9982737ca543e96133087cbd9a485eecc3bc4de9c1a37b47ea59c"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "sha1"
version = "0.10.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba"
dependencies = [
 "cfg-if",
 "cpufeatures",
 "digest",
]

[[package]]
name = "shlex"
version = "1.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0fda2ff0d084019ba4d7c6f371c95d8fd75ce3524c3cb8fb653a3023f6323e64"

[[package]]
name = "signal-hook-registry"
version = "1.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c4db69cba1110affc0e9f7bcd48bbf87b3f4fc7c61fc9155afd4c469eb3d6c1b"
dependencies = [
 "errno",
 "libc",
]

[[package]]
name = "simd-adler32"
version = "0.3.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e320a6c5ad31d271ad523dcf3ad13e2767ad8b1cb8f047f75a8aeaf8da139da2"

[[package]]
name = "slab"
version = "0.4.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5"

[[package]]
name = "slotmap"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bdd58c3c93c3d278ca835519292445cb4b0d4dc59ccfdf7ceadaab3f8aeb4038"
dependencies = [
 "version_check",
]

[[package]]
name = "smallvec"
version = "1.15.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67b1b7a3b5fe4f1376887184045fcf45c69e92af734b7aaddc05fb777b6fbd03"

[[package]]
name = "smithay-client-toolkit"
version = "0.18.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "922fd3eeab3bd820d76537ce8f582b1cf951eceb5475c28500c7457d9d17f53a"
dependencies = [
 "bitflags 2.11.0",
 "calloop 0.12.4",
 "calloop-wayland-source 0.2.0",
 "cursor-icon",
 "libc",
 "log",
 "memmap2",
 "rustix 0.38.44",
 "thiserror 1.0.69",
 "wayland-backend",
 "wayland-client",
 "wayland-csd-frame",
 "wayland-cursor",
 "wayland-protocols 0.31.2",
 "wayland-protocols-wlr 0.2.0",
 "wayland-scanner",
 "xkeysym",
]

[[package]]
name = "smithay-client-toolkit"
version = "0.20.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0512da38f5e2b31201a93524adb8d3136276fa4fe4aafab4e1f727a82b534cc0"
dependencies = [
 "bitflags 2.11.0",
 "calloop 0.14.4",
 "calloop-wayland-source 0.4.1",
 "cursor-icon",
 "libc",
 "log",
 "memmap2",
 "rustix 1.1.4",
 "thiserror 2.0.18",
 "wayland-backend",
 "wayland-client",
 "wayland-csd-frame",
 "wayland-cursor",
 "wayland-protocols 0.32.10",
 "wayland-protocols-experimental",
 "wayland-protocols-misc",
 "wayland-protocols-wlr 0.3.10",
 "wayland-scanner",
 "xkeysym",
]

[[package]]
name = "smithay-clipboard"
version = "0.7.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "71704c03f739f7745053bde45fa203a46c58d25bc5c4efba1d9a60e9dba81226"
dependencies = [
 "libc",
 "smithay-client-toolkit 0.20.0",
 "wayland-backend",
]

[[package]]
name = "smol_str"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd538fb6910ac1099850255cf94a94df6551fbdd602454387d0adb2d1ca6dead"
dependencies = [
 "serde",
]

[[package]]
name = "stable_deref_trait"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596"

[[package]]
name = "static_assertions"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2eb9349b6444b326872e140eb1cf5e7c522154d69e7a0ffb0fb81c06b37543f"

[[package]]
name = "syn"
version = "2.0.117"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e665b8803e7b1d2a727f4023456bbbbe74da67099c585258af0ad9c5013b9b99"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "synstructure"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "tempfile"
version = "3.26.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "82a72c767771b47409d2345987fda8628641887d5466101319899796367354a0"
dependencies = [
 "fastrand",
 "getrandom 0.4.1",
 "once_cell",
 "rustix 1.1.4",
 "windows-sys 0.61.2",
]

[[package]]
name = "thiserror"
version = "1.0.69"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52"
dependencies = [
 "thiserror-impl 1.0.69",
]

[[package]]
name = "thiserror"
version = "2.0.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4288b5bcbc7920c07a1149a35cf9590a2aa808e0bc1eafaade0b80947865fbc4"
dependencies = [
 "thiserror-impl 2.0.18",
]

[[package]]
name = "thiserror-impl"
version = "1.0.69"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "thiserror-impl"
version = "2.0.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ebc4ee7f67670e9b64d05fa4253e753e016c6c95ff35b89b7941d6b856dec1d5"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "tiff"
version = "0.10.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "af9605de7fee8d9551863fd692cce7637f548dbd9db9180fcc07ccc6d26c336f"
dependencies = [
 "fax",
 "flate2",
 "half",
 "quick-error",
 "weezl",
 "zune-jpeg",
]

[[package]]
name = "tinystr"
version = "0.8.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "42d3e9c45c09de15d06dd8acf5f4e0e399e85927b7f00711024eb7ae10fa4869"
dependencies = [
 "displaydoc",
 "zerovec",
]

[[package]]
name = "toml"
version = "0.5.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f4f7f0dd8d50a853a531c426359045b1998f04219d88799810762cd4ad314234"
dependencies = [
 "serde",
]

[[package]]
name = "toml_datetime"
version = "0.7.5+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92e1cfed4a3038bc5a127e35a2d360f145e1f4b971b551a2ba5fd7aedf7e1347"
dependencies = [
 "serde_core",
]

[[package]]
name = "toml_edit"
version = "0.23.10+spec-1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "84c8b9f757e028cee9fa244aea147aab2a9ec09d5325a9b01e0a49730c2b5269"
dependencies = [
 "indexmap",
 "toml_datetime",
 "toml_parser",
 "winnow",
]

[[package]]
name = "toml_parser"
version = "1.0.9+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "702d4415e08923e7e1ef96cd5727c0dfed80b4d2fa25db9647fe5eb6f7c5a4c4"
dependencies = [
 "winnow",
]

[[package]]
name = "tracing"
version = "0.1.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100"
dependencies = [
 "log",
 "pin-project-lite",
 "tracing-attributes",
 "tracing-core",
]

[[package]]
name = "tracing-attributes"
version = "0.1.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7490cfa5ec963746568740651ac6781f701c9c5ea257c58e057f3ba8cf69e8da"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "tracing-core"
version = "0.1.36"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "db97caf9d906fbde555dd62fa95ddba9eecfd14cb388e4f491a66d74cd5fb79a"
dependencies = [
 "once_cell",
]

[[package]]
name = "ttf-parser"
version = "0.25.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2df906b07856748fa3f6e0ad0cbaa047052d4a7dd609e231c4f72cee8c36f31"

[[package]]
name = "typenum"
version = "1.19.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "562d481066bde0658276a35467c4af00bdc6ee726305698a55b86e61d7ad82bb"

[[package]]
name = "uds_windows"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "89daebc3e6fd160ac4aa9fc8b3bf71e1f74fbf92367ae71fb83a037e8bf164b9"
dependencies = [
 "memoffset",
 "tempfile",
 "winapi",
]

[[package]]
name = "unicode-ident"
version = "1.0.24"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75"

[[package]]
name = "unicode-segmentation"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f6ccf251212114b54433ec949fd6a7841275f9ada20dddd2f29e9ceea4501493"

[[package]]
name = "unicode-xid"
version = "0.2.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ebc1c04c71510c7f702b52b7c350734c9ff1295c464a03335b00bb84fc54f853"

[[package]]
name = "url"
version = "2.5.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed"
dependencies = [
 "form_urlencoded",
 "idna",
 "percent-encoding",
 "serde",
 "serde_derive",
]

[[package]]
name = "urlencoding"
version = "2.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "daf8dba3b7eb870caf1ddeed7bc9d2a049f3cfdfae7cb521b087cc33ae4c49da"

[[package]]
name = "utf8_iter"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be"

[[package]]
name = "version_check"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"

[[package]]
name = "walkdir"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29790946404f91d9c5d06f9874efddea1dc06c5efe94541a7d6863108e3a5e4b"
dependencies = [
 "same-file",
 "winapi-util",
]

[[package]]
name = "wasi"
version = "0.11.1+wasi-snapshot-preview1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b"

[[package]]
name = "wasip2"
version = "1.0.2+wasi-0.2.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9517f9239f02c069db75e65f174b3da828fe5f5b945c4dd26bd25d89c03ebcf5"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasip3"
version = "0.4.0+wasi-0.3.0-rc-2026-01-06"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5428f8bf88ea5ddc08faddef2ac4a67e390b88186c703ce6dbd955e1c145aca5"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasm-bindgen"
version = "0.2.114"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6532f9a5c1ece3798cb1c2cfdba640b9b3ba884f5db45973a6f442510a87d38e"
dependencies = [
 "cfg-if",
 "once_cell",
 "rustversion",
 "wasm-bindgen-macro",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-futures"
version = "0.4.64"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e9c5522b3a28661442748e09d40924dfb9ca614b21c00d3fd135720e48b67db8"
dependencies = [
 "cfg-if",
 "futures-util",
 "js-sys",
 "once_cell",
 "wasm-bindgen",
 "web-sys",
]

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.114"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "18a2d50fcf105fb33bb15f00e7a77b772945a2ee45dcf454961fd843e74c18e6"
dependencies = [
 "quote",
 "wasm-bindgen-macro-support",
]

[[package]]
name = "wasm-bindgen-macro-support"
version = "0.2.114"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "03ce4caeaac547cdf713d280eda22a730824dd11e6b8c3ca9e42247b25c631e3"
dependencies = [
 "bumpalo",
 "proc-macro2",
 "quote",
 "syn",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-shared"
version = "0.2.114"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75a326b8c223ee17883a4251907455a2431acc2791c98c26279376490c378c16"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "wasm-encoder"
version = "0.244.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "990065f2fe63003fe337b932cfb5e3b80e0b4d0f5ff650e6985b1048f62c8319"
dependencies = [
 "leb128fmt",
 "wasmparser",
]

[[package]]
name = "wasm-metadata"
version = "0.244.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb0e353e6a2fbdc176932bbaab493762eb1255a7900fe0fea1a2f96c296cc909"
dependencies = [
 "anyhow",
 "indexmap",
 "wasm-encoder",
 "wasmparser",
]

[[package]]
name = "wasmparser"
version = "0.244.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47b807c72e1bac69382b3a6fb3dbe8ea4c0ed87ff5629b8685ae6b9a611028fe"
dependencies = [
 "bitflags 2.11.0",
 "hashbrown 0.15.5",
 "indexmap",
 "semver",
]

[[package]]
name = "wayland-backend"
version = "0.3.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fee64194ccd96bf648f42a65a7e589547096dfa702f7cadef84347b66ad164f9"
dependencies = [
 "cc",
 "downcast-rs",
 "rustix 1.1.4",
 "scoped-tls",
 "smallvec",
 "wayland-sys",
]

[[package]]
name = "wayland-client"
version = "0.31.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8e6faa537fbb6c186cb9f1d41f2f811a4120d1b57ec61f50da451a0c5122bec"
dependencies = [
 "bitflags 2.11.0",
 "rustix 1.1.4",
 "wayland-backend",
 "wayland-scanner",
]

[[package]]
name = "wayland-csd-frame"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "625c5029dbd43d25e6aa9615e88b829a5cad13b2819c4ae129fdbb7c31ab4c7e"
dependencies = [
 "bitflags 2.11.0",
 "cursor-icon",
 "wayland-backend",
]

[[package]]
name = "wayland-cursor"
version = "0.31.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5864c4b5b6064b06b1e8b74ead4a98a6c45a285fe7a0e784d24735f011fdb078"
dependencies = [
 "rustix 1.1.4",
 "wayland-client",
 "xcursor",
]

[[package]]
name = "wayland-protocols"
version = "0.31.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f81f365b8b4a97f422ac0e8737c438024b5951734506b0e1d775c73030561f4"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols"
version = "0.32.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "baeda9ffbcfc8cd6ddaade385eaf2393bd2115a69523c735f12242353c3df4f3"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-experimental"
version = "20250721.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "40a1f863128dcaaec790d7b4b396cc9b9a7a079e878e18c47e6c2d2c5a8dcbb1"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols 0.32.10",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-misc"
version = "0.3.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "791c58fdeec5406aa37169dd815327d1e47f334219b523444bc26d70ceb4c34e"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols 0.32.10",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-plasma"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23803551115ff9ea9bce586860c5c5a971e360825a0309264102a9495a5ff479"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols 0.31.2",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-wlr"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad1f61b76b6c2d8742e10f9ba5c3737f6530b4c243132c2a2ccc8aa96fe25cd6"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols 0.31.2",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-wlr"
version = "0.3.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e9597cdf02cf0c34cd5823786dce6b5ae8598f05c2daf5621b6e178d4f7345f3"
dependencies = [
 "bitflags 2.11.0",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols 0.32.10",
 "wayland-scanner",
]

[[package]]
name = "wayland-scanner"
version = "0.31.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5423e94b6a63e68e439803a3e153a9252d5ead12fd853334e2ad33997e3889e3"
dependencies = [
 "proc-macro2",
 "quick-xml",
 "quote",
]

[[package]]
name = "wayland-sys"
version = "0.31.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e6dbfc3ac5ef974c92a2235805cc0114033018ae1290a72e474aa8b28cbbdfd"
dependencies = [
 "dlib",
 "log",
 "once_cell",
 "pkg-config",
]

[[package]]
name = "web-sys"
version = "0.3.91"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "854ba17bb104abfb26ba36da9729addc7ce7f06f5c0f90f3c391f8461cca21f9"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "web-time"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aa30049b1c872b72c89866d458eae9f20380ab280ffd1b1e18df2d3e2d98cfe0"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "webbrowser"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f00bb839c1cf1e3036066614cbdcd035ecf215206691ea646aa3c60a24f68f2"
dependencies = [
 "core-foundation 0.10.1",
 "jni",
 "log",
 "ndk-context",
 "objc2 0.6.4",
 "objc2-foundation 0.3.2",
 "url",
 "web-sys",
]

[[package]]
name = "weezl"
version = "0.1.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a28ac98ddc8b9274cb41bb4d9d4d5c425b6020c50c46f25559911905610b4a88"

[[package]]
name = "winapi"
version = "0.3.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419"
dependencies = [
 "winapi-i686-pc-windows-gnu",
 "winapi-x86_64-pc-windows-gnu",
]

[[package]]
name = "winapi-i686-pc-windows-gnu"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6"

[[package]]
name = "winapi-util"
version = "0.1.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "winapi-x86_64-pc-windows-gnu"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f"

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "windows-sys"
version = "0.45.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75283be5efb2831d37ea142365f009c02ec203cd29a3ebecbc093d52315b66d0"
dependencies = [
 "windows-targets 0.42.2",
]

[[package]]
name = "windows-sys"
version = "0.48.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "677d2418bec65e3338edb076e806bc1ec15693c5d0104683f2efe857f61056a9"
dependencies = [
 "windows-targets 0.48.5",
]

[[package]]
name = "windows-sys"
version = "0.52.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d"
dependencies = [
 "windows-targets 0.52.6",
]

[[package]]
name = "windows-sys"
version = "0.59.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b"
dependencies = [
 "windows-targets 0.52.6",
]

[[package]]
name = "windows-sys"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2f500e4d28234f72040990ec9d39e3a6b950f9f22d3dba18416c35882612bcb"
dependencies = [
 "windows-targets 0.53.5",
]

[[package]]
name = "windows-sys"
version = "0.61.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-targets"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e5180c00cd44c9b1c88adb3693291f1cd93605ded80c250a75d472756b4d071"
dependencies = [
 "windows_aarch64_gnullvm 0.42.2",
 "windows_aarch64_msvc 0.42.2",
 "windows_i686_gnu 0.42.2",
 "windows_i686_msvc 0.42.2",
 "windows_x86_64_gnu 0.42.2",
 "windows_x86_64_gnullvm 0.42.2",
 "windows_x86_64_msvc 0.42.2",
]

[[package]]
name = "windows-targets"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a2fa6e2155d7247be68c096456083145c183cbbbc2764150dda45a87197940c"
dependencies = [
 "windows_aarch64_gnullvm 0.48.5",
 "windows_aarch64_msvc 0.48.5",
 "windows_i686_gnu 0.48.5",
 "windows_i686_msvc 0.48.5",
 "windows_x86_64_gnu 0.48.5",
 "windows_x86_64_gnullvm 0.48.5",
 "windows_x86_64_msvc 0.48.5",
]

[[package]]
name = "windows-targets"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973"
dependencies = [
 "windows_aarch64_gnullvm 0.52.6",
 "windows_aarch64_msvc 0.52.6",
 "windows_i686_gnu 0.52.6",
 "windows_i686_gnullvm 0.52.6",
 "windows_i686_msvc 0.52.6",
 "windows_x86_64_gnu 0.52.6",
 "windows_x86_64_gnullvm 0.52.6",
 "windows_x86_64_msvc 0.52.6",
]

[[package]]
name = "windows-targets"
version = "0.53.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4945f9f551b88e0d65f3db0bc25c33b8acea4d9e41163edf90dcd0b19f9069f3"
dependencies = [
 "windows-link",
 "windows_aarch64_gnullvm 0.53.1",
 "windows_aarch64_msvc 0.53.1",
 "windows_i686_gnu 0.53.1",
 "windows_i686_gnullvm 0.53.1",
 "windows_i686_msvc 0.53.1",
 "windows_x86_64_gnu 0.53.1",
 "windows_x86_64_gnullvm 0.53.1",
 "windows_x86_64_msvc 0.53.1",
]

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "597a5118570b68bc08d8d59125332c54f1ba9d9adeedeef5b99b02ba2b0698f8"

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2b38e32f0abccf9987a4e3079dfb67dcd799fb61361e53e2882c3cbaf0d905d8"

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3"

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a9d8416fa8b42f5c947f8482c43e7d89e73a173cead56d044f6a56104a6d1b53"

[[package]]
name = "windows_aarch64_msvc"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e08e8864a60f06ef0d0ff4ba04124db8b0fb3be5776a5cd47641e942e58c4d43"

[[package]]
name = "windows_aarch64_msvc"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc35310971f3b2dbbf3f0690a219f40e2d9afcf64f9ab7cc1be722937c26b4bc"

[[package]]
name = "windows_aarch64_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469"

[[package]]
name = "windows_aarch64_msvc"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9d782e804c2f632e395708e99a94275910eb9100b2114651e04744e9b125006"

[[package]]
name = "windows_i686_gnu"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c61d927d8da41da96a81f029489353e68739737d3beca43145c8afec9a31a84f"

[[package]]
name = "windows_i686_gnu"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a75915e7def60c94dcef72200b9a8e58e5091744960da64ec734a6c6e9b3743e"

[[package]]
name = "windows_i686_gnu"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b"

[[package]]
name = "windows_i686_gnu"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "960e6da069d81e09becb0ca57a65220ddff016ff2d6af6a223cf372a506593a3"

[[package]]
name = "windows_i686_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66"

[[package]]
name = "windows_i686_gnullvm"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fa7359d10048f68ab8b09fa71c3daccfb0e9b559aed648a8f95469c27057180c"

[[package]]
name = "windows_i686_msvc"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "44d840b6ec649f480a41c8d80f9c65108b92d89345dd94027bfe06ac444d1060"

[[package]]
name = "windows_i686_msvc"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f55c233f70c4b27f66c523580f78f1004e8b5a8b659e05a4eb49d4166cca406"

[[package]]
name = "windows_i686_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66"

[[package]]
name = "windows_i686_msvc"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e7ac75179f18232fe9c285163565a57ef8d3c89254a30685b57d83a38d326c2"

[[package]]
name = "windows_x86_64_gnu"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8de912b8b8feb55c064867cf047dda097f92d51efad5b491dfb98f6bbb70cb36"

[[package]]
name = "windows_x86_64_gnu"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "53d40abd2583d23e4718fddf1ebec84dbff8381c07cae67ff7768bbf19c6718e"

[[package]]
name = "windows_x86_64_gnu"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78"

[[package]]
name = "windows_x86_64_gnu"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9c3842cdd74a865a8066ab39c8a7a473c0778a3f29370b5fd6b4b9aa7df4a499"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "26d41b46a36d453748aedef1486d5c7a85db22e56aff34643984ea85514e94a3"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b7b52767868a23d5bab768e390dc5f5c55825b6d30b86c844ff2dc7414044cc"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ffa179e2d07eee8ad8f57493436566c7cc30ac536a3379fdf008f47f6bb7ae1"

[[package]]
name = "windows_x86_64_msvc"
version = "0.42.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9aec5da331524158c6d1a4ac0ab1541149c0b9505fde06423b02f5ef0106b9f0"

[[package]]
name = "windows_x86_64_msvc"
version = "0.48.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed94fce61571a4006852b7389a063ab983c02eb1bb37b47f8272ce92d06d9538"

[[package]]
name = "windows_x86_64_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec"

[[package]]
name = "windows_x86_64_msvc"
version = "0.53.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6bbff5f0aada427a1e5a6da5f1f98158182f26556f345ac9e04d36d0ebed650"

[[package]]
name = "winit"
version = "0.29.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0d59ad965a635657faf09c8f062badd885748428933dad8e8bdd64064d92e5ca"
dependencies = [
 "ahash",
 "android-activity",
 "atomic-waker",
 "bitflags 2.11.0",
 "bytemuck",
 "calloop 0.12.4",
 "cfg_aliases 0.1.1",
 "core-foundation 0.9.4",
 "core-graphics",
 "cursor-icon",
 "icrate",
 "js-sys",
 "libc",
 "log",
 "memmap2",
 "ndk",
 "ndk-sys",
 "objc2 0.4.1",
 "once_cell",
 "orbclient",
 "percent-encoding",
 "raw-window-handle 0.5.2",
 "raw-window-handle 0.6.2",
 "redox_syscall 0.3.5",
 "rustix 0.38.44",
 "smithay-client-toolkit 0.18.1",
 "smol_str",
 "unicode-segmentation",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols 0.31.2",
 "wayland-protocols-plasma",
 "web-sys",
 "web-time",
 "windows-sys 0.48.0",
 "x11-dl",
 "x11rb",
 "xkbcommon-dl",
]

[[package]]
name = "winnow"
version = "0.7.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a5364e9d77fcdeeaa6062ced926ee3381faa2ee02d3eb83a5c27a8825540829"
dependencies = [
 "memchr",
]

[[package]]
name = "winres"
version = "0.1.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b68db261ef59e9e52806f688020631e987592bd83619edccda9c47d42cde4f6c"
dependencies = [
 "toml",
]

[[package]]
name = "wit-bindgen"
version = "0.51.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d7249219f66ced02969388cf2bb044a09756a083d0fab1e566056b04d9fbcaa5"
dependencies = [
 "wit-bindgen-rust-macro",
]

[[package]]
name = "wit-bindgen-core"
version = "0.51.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ea61de684c3ea68cb082b7a88508a8b27fcc8b797d738bfc99a82facf1d752dc"
dependencies = [
 "anyhow",
 "heck",
 "wit-parser",
]

[[package]]
name = "wit-bindgen-rust"
version = "0.51.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b7c566e0f4b284dd6561c786d9cb0142da491f46a9fbed79ea69cdad5db17f21"
dependencies = [
 "anyhow",
 "heck",
 "indexmap",
 "prettyplease",
 "syn",
 "wasm-metadata",
 "wit-bindgen-core",
 "wit-component",
]

[[package]]
name = "wit-bindgen-rust-macro"
version = "0.51.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c0f9bfd77e6a48eccf51359e3ae77140a7f50b1e2ebfe62422d8afdaffab17a"
dependencies = [
 "anyhow",
 "prettyplease",
 "proc-macro2",
 "quote",
 "syn",
 "wit-bindgen-core",
 "wit-bindgen-rust",
]

[[package]]
name = "wit-component"
version = "0.244.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9d66ea20e9553b30172b5e831994e35fbde2d165325bec84fc43dbf6f4eb9cb2"
dependencies = [
 "anyhow",
 "bitflags 2.11.0",
 "indexmap",
 "log",
 "serde",
 "serde_derive",
 "serde_json",
 "wasm-encoder",
 "wasm-metadata",
 "wasmparser",
 "wit-parser",
]

[[package]]
name = "wit-parser"
version = "0.244.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ecc8ac4bc1dc3381b7f59c34f00b67e18f910c2c0f50015669dde7def656a736"
dependencies = [
 "anyhow",
 "id-arena",
 "indexmap",
 "log",
 "semver",
 "serde",
 "serde_derive",
 "serde_json",
 "unicode-xid",
 "wasmparser",
]

[[package]]
name = "writeable"
version = "0.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9edde0db4769d2dc68579893f2306b26c6ecfbe0ef499b013d731b7b9247e0b9"

[[package]]
name = "x11-dl"
version = "2.21.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "38735924fedd5314a6e548792904ed8c6de6636285cb9fec04d5b1db85c1516f"
dependencies = [
 "libc",
 "once_cell",
 "pkg-config",
]

[[package]]
name = "x11rb"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9993aa5be5a26815fe2c3eacfc1fde061fc1a1f094bf1ad2a18bf9c495dd7414"
dependencies = [
 "as-raw-xcb-connection",
 "gethostname",
 "libc",
 "libloading",
 "once_cell",
 "rustix 1.1.4",
 "x11rb-protocol",
]

[[package]]
name = "x11rb-protocol"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ea6fc2961e4ef194dcbfe56bb845534d0dc8098940c7e5c012a258bfec6701bd"

[[package]]
name = "xcursor"
version = "0.3.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bec9e4a500ca8864c5b47b8b482a73d62e4237670e5b5f1d6b9e3cae50f28f2b"

[[package]]
name = "xdg-home"
version = "1.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec1cdab258fb55c0da61328dc52c8764709b249011b2cad0454c72f0bf10a1f6"
dependencies = [
 "libc",
 "windows-sys 0.59.0",
]

[[package]]
name = "xkbcommon-dl"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d039de8032a9a8856a6be89cea3e5d12fdd82306ab7c94d74e6deab2460651c5"
dependencies = [
 "bitflags 2.11.0",
 "dlib",
 "log",
 "once_cell",
 "xkeysym",
]

[[package]]
name = "xkeysym"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9cc00251562a284751c9973bace760d86c0276c471b4be569fe6b068ee97a56"

[[package]]
name = "xml-rs"
version = "0.8.28"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ae8337f8a065cfc972643663ea4279e04e7256de865aa66fe25cec5fb912d3f"

[[package]]
name = "yoke"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72d6e5c6afb84d73944e5cedb052c4680d5657337201555f9f2a16b7406d4954"
dependencies = [
 "stable_deref_trait",
 "yoke-derive",
 "zerofrom",
]

[[package]]
name = "yoke-derive"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b659052874eb698efe5b9e8cf382204678a0086ebf46982b79d6ca3182927e5d"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
 "synstructure",
]

[[package]]
name = "zbus"
version = "4.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb97012beadd29e654708a0fdb4c84bc046f537aecfde2c3ee0a9e4b4d48c725"
dependencies = [
 "async-broadcast",
 "async-executor",
 "async-fs",
 "async-io",
 "async-lock",
 "async-process",
 "async-recursion",
 "async-task",
 "async-trait",
 "blocking",
 "enumflags2",
 "event-listener",
 "futures-core",
 "futures-sink",
 "futures-util",
 "hex",
 "nix",
 "ordered-stream",
 "rand",
 "serde",
 "serde_repr",
 "sha1",
 "static_assertions",
 "tracing",
 "uds_windows",
 "windows-sys 0.52.0",
 "xdg-home",
 "zbus_macros",
 "zbus_names",
 "zvariant",
]

[[package]]
name = "zbus_macros"
version = "4.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "267db9407081e90bbfa46d841d3cbc60f59c0351838c4bc65199ecd79ab1983e"
dependencies = [
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn",
 "zvariant_utils",
]

[[package]]
name = "zbus_names"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b9b1fef7d021261cc16cba64c351d291b715febe0fa10dc3a443ac5a5022e6c"
dependencies = [
 "serde",
 "static_assertions",
 "zvariant",
]

[[package]]
name = "zerocopy"
version = "0.8.40"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a789c6e490b576db9f7e6b6d661bcc9799f7c0ac8352f56ea20193b2681532e5"
dependencies = [
 "zerocopy-derive",
]

[[package]]
name = "zerocopy-derive"
version = "0.8.40"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f65c489a7071a749c849713807783f70672b28094011623e200cb86dcb835953"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "zerofrom"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "50cc42e0333e05660c3587f3bf9d0478688e15d870fab3346451ce7f8c9fbea5"
dependencies = [
 "zerofrom-derive",
]

[[package]]
name = "zerofrom-derive"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d71e5d6e06ab090c67b5e44993ec16b72dcbaabc526db883a360057678b48502"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
 "synstructure",
]

[[package]]
name = "zerotrie"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2a59c17a5562d507e4b54960e8569ebee33bee890c70aa3fe7b97e85a9fd7851"
dependencies = [
 "displaydoc",
 "yoke",
 "zerofrom",
]

[[package]]
name = "zerovec"
version = "0.11.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6c28719294829477f525be0186d13efa9a3c602f7ec202ca9e353d310fb9a002"
dependencies = [
 "yoke",
 "zerofrom",
 "zerovec-derive",
]

[[package]]
name = "zerovec-derive"
version = "0.11.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eadce39539ca5cb3985590102671f2567e659fca9666581ad3411d59207951f3"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "zmij"
version = "1.0.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8848ee67ecc8aedbaf3e4122217aff892639231befc6a1b58d29fff4c2cabaa"

[[package]]
name = "zune-core"
version = "0.4.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f423a2c17029964870cfaabb1f13dfab7d092a62a29a89264f4d36990ca414a"

[[package]]
name = "zune-jpeg"
version = "0.4.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29ce2c8a9384ad323cf564b67da86e21d3cfdff87908bc1223ed5c99bc792713"
dependencies = [
 "zune-core",
]

[[package]]
name = "zvariant"
version = "4.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2084290ab9a1c471c38fc524945837734fbf124487e105daec2bb57fd48c81fe"
dependencies = [
 "endi",
 "enumflags2",
 "serde",
 "static_assertions",
 "url",
 "zvariant_derive",
]

[[package]]
name = "zvariant_derive"
version = "4.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73e2ba546bda683a90652bac4a279bc146adad1386f25379cf73200d2002c449"
dependencies = [
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn",
 "zvariant_utils",
]

[[package]]
name = "zvariant_utils"
version = "2.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c51bcff7cc3dbb5055396bcf774748c3dab426b4b8659046963523cee4808340"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]


# Cargo.toml
[package]
name = "petri_net_legacy_editor"
version = "0.7.79"
edition = "2021"
license = "MIT"
build = "build.rs"

[dependencies]
anyhow = "1.0"
eframe = { version = "0.28", default-features = false, features = ["default_fonts", "glow", "persistence"] }
egui = "0.28"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rfd = "0.14"
rand = { version = "0.8", features = ["small_rng"] }
rand_distr = "0.4"
arboard = "3.6"
image = { version = "0.25", default-features = false, features = ["ico"] }
encoding_rs = "0.8"

[dev-dependencies]
tempfile = "3.10"

[target.'cfg(windows)'.build-dependencies]
winres = "0.1"

[profile.release]
strip = true
lto = "thin"
codegen-units = 1











# docs\copy.md


# docs\gpn_format.md
# Формат GPN

## GPN2 (нативный формат)

- Расширение файла: `.gpn2` (или `.gpn` с magic `GPN2\n`)
- Заголовок (обязателен): ASCII `GPN2\n`
- После заголовка: UTF-8 JSON, сериализованный через `serde_json`

### Обязательные поля JSON

- `format_version` (целое, текущая версия `2`)
- `meta`
- `places`
- `transitions`
- `arcs`
- `inhibitor_arcs`
- `tables`
- `ui`

### Базовая схема

```json
{
  "format_version": 2,
  "meta": {
    "name": "...",
    "author": "...",
    "description": "..."
  },
  "places": [{"id": 1, "name": "P1", "pos": [120.0, 80.0]}],
  "transitions": [{"id": 1, "name": "T1", "pos": [220.0, 80.0]}],
  "arcs": [{"id": 1, "from": {"type": "Place", "id": 1}, "to": {"type": "Transition", "id": 1}, "weight": 1}],
  "inhibitor_arcs": [],
  "tables": {
    "m0": [1],
    "mo": [null],
    "mz": [0.0],
    "mpr": [0],
    "pre": [[1]],
    "post": [[0]],
    "inhibitor": [[0]]
  },
  "ui": {
    "language": "Ru",
    "hide_grid": false,
    "snap_to_grid": true,
    "colored_petri_nets": false,
    "fix_time_step": true,
    "marker_count_stats": true,
    "light_theme": true
  }
}
```

### Валидация

При загрузке GPN2 выполняется `validate()`:

- `format_version == 2`
- уникальность `id` у мест и переходов
- дуги ссылаются только на существующие узлы
- веса/пороги дуг > 0
- размеры `tables` согласованы с числом мест/переходов
- значения `Mz` неотрицательные и конечные

### Миграции

`load_gpn2()` содержит каркас миграции `migrate_to_latest()`. Сейчас поддерживается версия `2`, заготовка под миграцию с версии `1` оставлена в коде.

## Legacy GPN (2003, бинарный)

Legacy `.gpn` импортируется и сохраняется модулем `io::legacy_gpn`:

- детектирование не-GPN2 файлов
- поиск кандидатов числа мест/переходов
- извлечение ASCII/UTF-16LE строк
- извлечение кандидатов координат как пар `float64` (little-endian)
- чтение фиксированных секций: `header` (16/247), места (231 байт), переходы (105 байт), дуги (count + записи по 46 байт)
- корректная обработка направления дуг (`dir=-1`: P->T, `dir=1`: T->P)
- задержки мест читаются из `f64` по смещению `+77` в записи места (с fallback на `+12`)

Импортер детерминированный и не должен падать на поврежденных данных.

## Утилита реверс-инжиниринга `gpn_dump`

Запуск:

```bash
gpn_dump <file.gpn> [--hexdump N] [--strings] [--floats] [--search "pattern"]
```

Что выводит:

- размер файла
- список строк ASCII/UTF-16LE с оффсетами
- кандидаты `float64` последовательностей с оффсетами
- кандидаты секций/счетчиков

Инструмент предназначен для постепенного восстановления структуры legacy бинарного формата.


# docs\markov_validation_plan.md
# :       

## 
1.        ( ,  /  .),            .
2.           (/)   ,         . Kolmogorov equations                   [Wikipedia: Kolmogorov equations].

## 
1.      `draw_sim_dialog`, `draw_place_props_window`, `draw_transition_props_window`   UI-:    (`time_limit_sec`, `pass_limit`, `place capacity`, `delay`,    ..).
2.   `clamp_input` ( `f64`, `u64`, `usize`)    NaN/Inf    ;     ,     (,  /).
3.     :  `DragValue`/`TextEdit`  ,  ,   `invalid`   . ,        .
4.    :
   -     `enabled_transitions`,          .
   -     (    )     `StateId -> Vec<(StateId, rate)>`.
5.        :    `Q`,   `??=1`,    (  ).      (.,  200 ).
6.   UI- (/),    Markov- (      ),     ,  , ,    .
7.   (`tests/markov.rs`)       (,     )  ,    .
8.  README/,   (0.7.13 > 0.7.14),  `PetriNet-0.7.14.exe`  `build_exe.ps1`,  `cargo check`, `cargo test`, `rustfmt --check`, `rg -n ...`.


# docs\task_plan_2026-03-08.md
﻿# План 08.03.2026

## Анализ
- Существующий код уже содержит модуль `markov`, но окно и расчёт цепи пока не вызованы из UI, а входные значения порой могут быть `NaN` или выходить за ожидаемые диапазоны.
- Чтобы не проигрывать непредвиденные ошибки, нужно централизованно санировать сетку (задержки, капас, веса), прежде чем запускать симуляцию или строить цепь.
- Для марковской подсистемы надо добавить элементы управления (кнопка/окно) и явно показать граф и решение уравнений Колмогорова (станционарные вероятности).

## План
1. Добавить метод `PetriNet::sanitize_values`, который выравнивает задержки/веса/ёмкости/ингибиторы, и вызывать его перед симуляцией и построением марковской модели.
2. В тул-палитре разместить кнопку по созданию марковской модели, чтобы запускать `calculate_markov_model`, включая вызов нового санитизатора, а окно `Markov model` расширить информативной статистикой и кнопкой «Пересчитать».
3. Обновить `table_view` перед симуляцией, чтобы данные очищались, а результаты не падали на некорректных вводах.
4. Добавить тесты: для `sanitize_values` (он не должен оставлять `NaN`/отрицательные задержки) и для `MarkovChain` (состояния/стационарное распределение суммируется в 1, граф ограничивается).
5. Фиксировать работу этими изменениями в `docs/Work.md`, а план — в новом `docs/task_plan_2026-03-08.md`.


# docs\task_plan_2026-03-10.md
# План 10.03.2026

## Новые требования
1. Показать кратность ингибиторных дуг напрямую в окне свойств дуги, чтобы пользователь сразу видел/редактировал вес или порог в одном месте.
2. Начинать отладку и симуляцию с текущего состояния сети (то есть с данных из таблиц, а не с шагов, пронумерованных от 1).
3. Марковский режим должен работать «локально» на каждой позиции: включается позиционно и выводит короткую метрику/вероятности непосредственно рядом с этой позицией.

## План
1. В таблице свойств дуги (draw_arc_props_window) добавить отображение строки с кратностью/порогом, сфокусированной на текущей дуге, и обновлять текст при переключении режима ингибитор.
2. Обновить логику старта симуляции/отладки: сохранить начальное состояние маркировки до запуска и использовать его для первого шага, убрав жесткую установку debug_step = 1.
3. Добавить поле markov_annotations: HashMap<u64, String> в PetriApp, переключатель в draw_place_props_window, и рисовать эту метку рядом с позицией через draw_graph_view, перерасчитывая цепи только для выбранных мест.
4. Обновить сопутствующую документацию (docs/work.md) и запустить сборку/тесты/экземпляр после завершения всех изменений.


# docs\task_plan_2026-03-13.md
# План 13.03.2026

## Задача
1. Сделать отображение кратности для ингибиторных дуг прямо на холсте по клику в свойствах, чтобы метка рисовалась только при включённой галочке и показывала актуальный порог.
2. Обеспечить, чтобы запуск отладки и симуляции сразу показывал текущее состояние сети (начальную маркировку из таблиц) в шаге 0 и отладочный журнал не начинался с шага 1.
3. Реализовать локальные марковские метки: отдельный переключатель для каждой позиции, подписание рядом с узлом и пересчёт меток только для включённых позиций после пересчёта цепи.

## План действий
1. В draw_arc_props_window показать блок с Порог/Кратность и чекбокс «Показывать кратность» для ингибиторов, а также отрисовать текст рядом с дугой только если show_weight включён.
2. Подтянуть run_simulation и debug_visible_log_indices так, чтобы первый элемент лога всегда был начальным состоянием, а debug_step и UI инициировались с 0.
3. Добавить поле markov_annotations в PetriApp, переключатель в draw_place_props_window и отрисовку метки в draw_graph_view, а calculate_markov_model и update_markov_annotations вызывать в необходимых местах.
4. Обновить документацию (docs/work.md) и зафиксировать выполнение всех тестов/сборок по инструкции после изменений.


# docs\work.md
# Общий список задач и требований (смерджено, без повторов)

> Важно: первые 14 пунктов — **в приоритете** над остальными добавлениями (если есть конфликт требований).

---

## Нововведение (обязательный процесс)

1. Перед реализацией любых изменений:
   - провести анализ требований;
   - записать общий план работ в единый `.md` (этот файл или отдельный, например `PLAN.md`);
   - далее выполнять по пунктам.
2. Это правило **жёстко зафиксировать в `AGENTS.md`**.

---

## A. Приоритет №1 — “первые 14 пунктов”

### 1) UI: заголовок окна, кнопки управления, вкладки/сворачивание
- Перенести кнопки управления (свернуть / полный экран / закрыть) в **заголовок окна**.
- Вкладки:
  - убрать текст “вкладка”;
  - кликабельна **вся ячейка вкладки**, не только текст;
  - если вкладка “есть/не нужна” → **элемент скрывать**, не показывать пустышку;
  - при открытии другого окна вкладки **не должны показываться**;
  - реализовать по логике браузера (ориентир: **Google Chrome**).
- Иконки элементов:
  - переработать так, чтобы не были одинаковыми и были однозначно понятны.
- Кнопки “свернуть/полный экран”:
  - **не менять иконку при нажатии**.
- Для всех раскрывающихся/плавающих элементов:
  - в заголовке: **свернуть**, **полный экран (в рамках окна программы)**, **закрыть**;
  - расположение: кнопка **свернуть справа**, чуть левее закрыть; **между** ними кнопка “полный экран”.

---

### 2) Дуги: точность прилипания и отсутствие задержек
- Исправить “прилипание” дуг к позициям/переходам (цепляется неточно).
- Убрать задержку после создания дуги: следующую дугу можно начинать сразу.

---

### 3) Графы состояний (Марковские модели): отображение и управление
- Реализовать построение **графов состояний** (переходы между состояниями).
- Отображать состояние/информацию **около позиции** текстом/метками (для конкретной позиции).
- В свойствах позиции дать выбор **что отображать** (UX-решение выбрать самостоятельно).

---

### 4) Цифровые показатели прямо на модель (оверлеи)
- Выводить прямо на модель:
  - размер очереди;
  - загрузку;
  - параметры позиций;
  - другие ключевые показатели.

---

### 5) Полный экран: исправить растяжение по высоте
- Сейчас full screen растягивает по ширине, но не по высоте — исправить.

---

### 6) Производительность: “Структура сети” и матрицы
- “Структура сети” подлагивает даже на малой сети.
- Особенно тормозят на больших сетях:
  - ингибиторная матрица;
  - матрица `post`;
  - матрица `pre`;
  - при этом “векторы” работают нормально → по возможности реализовать аналогичную оптимизацию.
- Если включить все “вкладки/панели” структуры сети — всё равно лагает:
  - прогружать/рендерить **только видимое** пользователю (виртуализация, ленивая отрисовка);
  - если не помогает — решить иначе, но без зависаний.
- Доп. требование: “Структуру сети сделать отдельно обрабатываемой” (чтобы UI не фризился).

---

### 7) График статистики: hover подсказки
- При наведении на синюю линию показывать у курсора параметры точки:
  - не требовать попадания в пиксель;
  - сделать небольшую область захвата.

---

### 8) Русский язык: локализация/кодировки/термины
- Местами ломается русский (буквы пропадают, слова кривые).
- Исправить язык в списке цветов на русский.
- Сделать так, чтобы русский “никогда не сбивался” (возможная причина: кодировка/сохранение через cmd).
- Пройтись по всему UI: корректные формулировки (пример: “наполный экран” → “полный экран”).

---

### 9) Память: перерасход/утечки
- После эмуляции приложение всё равно много потребляет памяти — найти и исправить.

---

### 10) Режим отладки: подписи, единицы, переносимость UI
- Исправить подпись: было “Скорость (с)”, нужно **“Скорость (мс)”**.
- Там, где показывается `t`, добавить слева пояснение (например “текущее время”).
- Все раскрывающиеся элементы:
  - должны перемещаться по области программы;
  - не должны растягиваться на весь экран по вертикали без возможности сдвига.
- Ограничение перемещения:
  - панели/выпадашки не должны уезжать за границы окна приложения.

---

### 11) Анимация движения объектов: точность, цвета, переходы, время
- Анимация должна быть “шариком” как в примере и **правильного цвета** (сейчас несоответствие).
- Анимация должна работать **и на переходы**:
  - сейчас поток пропадает в переходе и появляется позже — нужно наглядное непрерывное отображение.
- Анимация должна быть **связана с реальным временем** прохождения (с задержками и т.п.):
  - сейчас ощущение, что точка “зависает” в позиции.
- Отдельное требование:
  - анимация прохождения заявки по дугам должна включаться галочкой внутри **“режим отладки”**.

---

### 12) Распределения/вес дуг: чистка старого и синхронизация
- Убрать пункт распределения “заданное пользователем” (в текущем виде) и связанный функционал.
- Синхронизировать вес/кратность:
  - в параметрах дуги и в “свойства создаваемых элементов”.
- Переименовать в “свойства создаваемых элементов”:
  - “кратность дуги” → **“Вес (кратность)”**.

---

### 13) Панорамирование области
- Движение области (сейчас ПКМ по пустой области) заменить на **зажатие колёсика мыши** (MMB drag).

---

### 14) Тесты/разработка/процесс коммитов
- Оптимизировать запуск тестов:
  - не запускать каждый отдельно;
  - сделать общий скрипт/команду.
- Код: максимально модульный и читаемый.
- Тесты обязательны + в README описать, как их запускать.
- Каждая задача — отдельный коммит на русском (для отката).
- На сервер отправлять только по явному запросу.
- Перед началом работ — прочитать `AGENTS.md`.

---

## B. Дополнительные фиксы и требования (вне “14”, но уже запрошены)

### B1) Горячие клавиши и выделение
- `Ctrl+A` не выделяет рамки и текст — исправить.

### B2) Новый файл без элементов
- При создании нового файла по умолчанию не должно быть элементов (сейчас есть `p2` и `t2`) — убрать.

### B3) Баг с удалением пунктов/именами
- Если удаляется пункт — другой пункт становится именем текущего. Так быть не должно — исправить.

### B4) Терминология инструмента
- Переименовать инструмент “место” → **“позиция”**.

### B5) Вес дуг: UI + алгоритм выбора пути
- Добавить дугам вес (кратность) через UI:
  - выделить дугу → ПКМ → параметры:
    - ингибиторная ли дуга;
    - вес (кратность).
- Исправить алгоритм выбора пути по весам:
  - сейчас веса работают “коряво”.
- Добавить настройку **цвета дуги** в свойства дуги.

### B6) Файл сохранения: уменьшение размера/строк
- Сейчас сохранение идёт в слишком много строк (может >100k).
- Группировать данные (матрицы/блоки/структуры) в меньшее число строк/более компактный формат.
- Пример проблемного файла: `модель1.gpn2` — ориентир для оптимизации.

### B7) Недоделки и артефакты
- Упомянуты untracked-файлы: `_tmp_probe.rs` и `модель1.gpn2` (их не трогали).
- Доделать оптимизацию скролла/виртуализации больших матриц в “Структуре сети”.

---

## C. Новые фичи “вторым приоритетом” (если не конфликтуют с A)

### C1) Марковские модели (расширение)
- Автоматическое составление и решение уравнений Колмогорова.
- Расчёт предельных вероятностей состояний.
- “Должны считаться и рассчитываться марковцевые модели” (итоговая готовность фичи).

### C2) Вес/приоритеты для тележек/транспортеров
- Чем выше вес дуги → тем выше приоритет направления движения.
- Исправить расчёт времени перехода между станками/позициями.
- Проработать сценарий: “пришёл → взял → уехал в очередь → поехал к станку”.

### C3) Визуальные режимы
- Режим отладки: подробная информация, цифры, очереди.
- Режим презентации: “красивая” версия, скрытие технических деталей, чистая анимация.

### C4) Импорт/Экспорт
- Импорт матриц (резервный ввод данных).
- Экспорт результатов симуляции в таблицу (Excel/CSV).

### C5) Управление симуляцией
- Кнопки “Старт” и “Стоп”.

### C6) Совместимость версий
- Новая версия должна открывать модели/файлы старой версии (обратная совместимость).

### C7) Валидация ввода
- “Защита от дурака”: проверка значений, чтобы программа не падала от некорректного ввода.

### C8) Документация (руководство пользователя)
- Подробное описание интерфейса:
  - куда нажимать;
  - что делает каждая кнопка;
  - пошаговые сценарии работы.

### C9) Генерация случайных чисел
- Уточнить/реализовать “нормальный” RNG без заранее заданного списка (seed/генератор/распределения).

### C10) Формулы в “заданном пользователем” (если вернуть в новом виде)
- Если “заданное пользователем” возвращается как формулы:
  - пример должен быть формулой (не `base`);
  - в help добавить подробные примеры (умножение, степень и т.д.).

---

## D. Нефункциональные требования
- Скорость работы: приложение не должно лагать/зависать, особенно на больших моделях.
- Читаемость и модульность кода.
- Тестируемость (тесты + понятный запуск).## 08.03.2026 — Санитизация и марковские модели
- Работаю над автоматической проверкой ввода (затраты, веса, максимумы) и подключаю существующую цепь Маркова через новую кнопку/окно.

## 09.03.2026       
-           .
- Markov-    ,        .

## 10.03.2026 — Ингибиторные дуги и локальный Марков
- В свойствах дуг появилась галочка «Показывать кратность» для ингибиторов, теперь подпись рисуется только если галочка включена и употребляет порог.
- Симуляция/отладка теперь сразу добавляют начальное состояние в журнал, поэтому журналы стартуют с реального начального шага с индексом 0.
- Для каждой позиции можно включить локальную марковскую метку; после пересчета цепи π выводится рядом с позицией на графе.

## 13.03.2026 — Ингибиторные дуги, дебаг и локальные метки
- Свойства дуг дополнились чекбоксом «Показывать кратность» для ингибиторов, а метка с порогом появляется прямо около дуги, когда галочка активирована.
- Отладка и симуляция теперь начинают шаг 0 с фактической маркировкой из таблиц, так что журнал сразу показывает начальное состояние.
- Для каждой позиции можно включить локальную марковскую метку; после пересчёта цепи рядом с ней рисуется текст с ожидаемым числом маркеров.


# docs\баги.md
##1
Баг. "в окне отладки, без включенной анимации, нельзя визуально посмотреть состояние системы на графе для конкретного шага". Ожидаемый фикс: "в окне отладки можно выбрать конкретный шаг и состояние системы будет показано на графе для данного шага, при погашенной галочке анимации".

##1.1 fixed build 0.7.51

##2
Ошибка в тексте. Блок: "Help:Помощь по управлению. Основные кнопки и комбинации. Надо заменить текст (ПКМ + перетаскивание: двигать рабочую область) на (CКМ + перетаскивание: двигать рабочую область)"

#2.1 fixed build 0.7.53

##3
Добавить текст Блок: "Help:Помощь по управлению. Основные кнопки и комбинации. Надо добавить текст (Ctrl + колесо: изменить масштаб графа)".

#3.1 fixed build 0.7.53

##4
Баг. "в некоторых разветвленных системах строки в журнале окна результаты/статистика дублируются. Дополнительно возникают фантомные шаги в окне Proof". Ожидаемый фикс: " надо сделать, чтобы в журнале отображались только начальное состояние системы, а после шли строки по отработавшим переходам как в режиме отладки по шагам, аналогичное отображение должно быть реализовано и в окне Proof.".

#4.1 fixed build 0.7.53

##5
Баг. "в окне свойства перехода. настройка (положение метки), отвечает за расположение не только текста метки перехода, но и за расположение замещающего его текста названия перехода, когда специально отведенная настройка для данного названия (положение текста) не оказывает никакого влияния. Аналогичная проблема проявляется и в окне (свойства позиции), только наоборот, настройка (положение метки) никак не влияет на расположение текста метки, но настройка (положение текста) влияет на расположение и названия метки, и заданного текста". Ожидаемый фикс: "надо, чтобы можно было раздельно управлять расположением метки через настройку (положение метки), и расположением заданного названия через настройку (положение текста). дополнительно иметь чекбокс для возможности включить и отключить отображения текста метки, при добавлении названия чекбокс выключает отображение текста метки и при нужде его надо включать вручную".

#5.1 fixed build 

##6
Переход. "угол наклона" - непонятный параметр.

#6.1 fixed build 0.7.54 (удален)

##7
Ошибка в тексте. Блок: "панель инструменты. иконки для всех инструментов должны быть свои: сейчас корректно отображаются только (дуга), (текст) и (удалить), а для остальных отображается квадратик вместо иконки."

#7.1 fixed build 


# patch_markov.py
f r o m   p a t h l i b   i m p o r t   P a t h 
 
 p a t h   =   P a t h ( ' s r c / u i / a p p / p e t r i _ a p p / d r a w i n g / d r a w _ m a r k o v _ w i n d o w . r s ' ) 
 
 t e x t   =   p a t h . r e a d _ t e x t ( e n c o d i n g = ' u t f - 8 ' ) 
 
 s t a r t   =   t e x t . i n d e x ( '                                                 l e t   m a r k o v _ f o c u s _ p l a c e s   =   s e l f ' ) 
 
 e n d   =   t e x t . i n d e x ( '                                         }   e l s e   { ' ,   s t a r t ) 
 
 o l d _ b l o c k   =   t e x t [ s t a r t : e n d ] 
 
 i n d e n t e d _ b l o c k   =   ' ' . j o i n ( '         '   +   l i n e   f o r   l i n e   i n   o l d _ b l o c k . s p l i t l i n e s ( T r u e ) ) 
 
 r e p l a c e m e n t   =   '                                                 i f   s e l f . m a r k o v _ m o d e l _ e n a b l e d   { 
 
 '   +   i n d e n t e d _ b l o c k   +   ' 
 
                                                 }   e l s e   { 
 
                                                         u i . s e p a r a t o r ( ) ; 
 
                                                         u i . l a b e l ( s e l f . t r ( 
 
                                                                 
 
 t e x t   =   t e x t [ : s t a r t ]   +   r e p l a c e m e n t   +   t e x t [ e n d : ] 
 
 p a t h . w r i t e _ t e x t ( t e x t ,   e n c o d i n g = ' u t f - 8 ' ) 
 
 

# publish_release.ps1
param(
    [string]$ProjectDir = $PSScriptRoot,
    [string]$ServerUrl = "http://100.64.0.7:3000",
    [string]$Owner = "Coxford",
    [string]$Repo = "PetriNet",
    [string[]]$DeleteReleaseTags = @(),
    [string]$Tag = "",
    [string]$ReleaseName = "",
    [switch]$KeepTarget
)

$ErrorActionPreference = "Stop"

Set-Location $ProjectDir

$token = $env:GITEA_TOKEN
if ([string]::IsNullOrWhiteSpace($token)) {
    throw "GITEA_TOKEN is not set. Example: `$env:GITEA_TOKEN='your_token'"
}

if ([string]::IsNullOrWhiteSpace($Tag)) {
    $Tag = "v" + (Get-Date -Format "yyyy.MM.dd-HHmmss")
}

if ([string]::IsNullOrWhiteSpace($ReleaseName)) {
    $ReleaseName = "Release $Tag"
}

Write-Host "Building executable..."
if ($KeepTarget) {
    & (Join-Path $ProjectDir "build_portable_exe.ps1") -ProjectDir $ProjectDir -KeepTarget
} else {
    & (Join-Path $ProjectDir "build_portable_exe.ps1") -ProjectDir $ProjectDir
}

$cargoTomlText = Get-Content -Path (Join-Path $ProjectDir "Cargo.toml") -Raw
$match = [regex]::Match($cargoTomlText, '(?m)^\s*version\s*=\s*"([^"]+)"')
if (-not $match.Success) {
    throw "Failed to read package version from Cargo.toml"
}
$version = $match.Groups[1].Value

$exePath = Join-Path $ProjectDir ("PetriNet-{0}.exe" -f $version)
if (-not (Test-Path $exePath)) {
    throw "Executable not found: $exePath"
}

Write-Host "Preparing git tag $Tag..."
$tagExistsLocal = git tag --list $Tag
if ([string]::IsNullOrWhiteSpace($tagExistsLocal)) {
    git tag -a $Tag -m "Release $Tag"
}

Write-Host "Pushing tag to origin..."
$remoteUrl = "$ServerUrl/$Owner/$Repo.git"
git -c "http.extraHeader=Authorization: token $token" push $remoteUrl $Tag

$headers = @{
    Authorization = "token $token"
    Accept = "application/json"
}

$baseApi = "${ServerUrl}/api/v1/repos/${Owner}/${Repo}"

function Remove-ReleaseByTag([string]$TagToDelete) {
    if ([string]::IsNullOrWhiteSpace($TagToDelete)) { return }
    $url = "$baseApi/releases/tags/$TagToDelete"
    try {
        $rel = Invoke-RestMethod -Method Get -Headers $headers -Uri $url
        if ($rel -and $rel.id) {
            Write-Host "Deleting release for tag $TagToDelete (id=$($rel.id))..."
            Invoke-RestMethod -Method Delete -Headers $headers -Uri "$baseApi/releases/$($rel.id)"
        }
    } catch {
        $response = $_.Exception.Response
        if ($response -and $response.StatusCode.value__ -eq 404) {
            Write-Host "Release for tag $TagToDelete not found (skip)."
            return
        }
        throw
    }
}

foreach ($oldTag in $DeleteReleaseTags) {
    Remove-ReleaseByTag $oldTag
}
$releaseByTagUrl = "${baseApi}/releases/tags/${Tag}"

Write-Host "Finding/creating release..."
$release = $null
try {
    $release = Invoke-RestMethod -Method Get -Headers $headers -Uri $releaseByTagUrl
} catch {
    $response = $_.Exception.Response
    if (-not $response -or $response.StatusCode.value__ -ne 404) {
        throw
    }
}

if (-not $release) {
    $createBody = @{
        tag_name = $Tag
        name = $ReleaseName
        draft = $false
        prerelease = $false
    } | ConvertTo-Json

    $release = Invoke-RestMethod -Method Post -Headers $headers -Uri "${baseApi}/releases" -Body $createBody -ContentType "application/json"
}

$releaseId = $release.id
if (-not $releaseId) {
    throw "Failed to resolve release id for tag $Tag"
}

$assetsUrl = "${baseApi}/releases/${releaseId}/assets"
$assetName = Split-Path -Leaf $exePath

Write-Host "Removing old asset with same name (if exists)..."
$assets = Invoke-RestMethod -Method Get -Headers $headers -Uri $assetsUrl
foreach ($asset in $assets) {
    if ($asset.name -eq $assetName) {
        Invoke-RestMethod -Method Delete -Headers $headers -Uri "$assetsUrl/$($asset.id)"
    }
}

Write-Host "Uploading executable to release..."
$encodedName = [uri]::EscapeDataString($assetName)
$uploadUrl = "${assetsUrl}?name=${encodedName}"
Invoke-RestMethod -Method Post -Headers @{ Authorization = "token $token"; Accept = "application/json"; "Content-Type" = "application/octet-stream" } -Uri $uploadUrl -InFile $exePath

Write-Host "Done. Release asset uploaded:"
Write-Host "$ServerUrl/$Owner/$Repo/releases/tag/$Tag"


# README.md
﻿# PetriNet

Редактор и симулятор сетей Петри на Rust (eframe/egui) с совместимостью с NetStar.

## Кодировка

Файлы проекта и документация: UTF-8.

## Версия

Текущая версия: 0.7.60 (см. также `Cargo.toml`).

## Возможности

- Редактирование графа: места, переходы, дуги, текстовые блоки, декоративные рамки.
- Симуляция и режим отладки (пошаговое выполнение).
- Импорт legacy `.gpn` (NetStar) и экспорт в NetStar.
- Собственный формат `gpn2` для полного сохранения состояния редактора (UI-данные).
- Импорт матриц из CSV (Pre/Post/ингибиторные дуги).
- Буфер обмена: копировать/вставить выделенное (`Ctrl+C` / `Ctrl+V`).
- Выделение: `Ctrl+A` (выделить все), `Shift` (добавить к выделению), `Esc` (снять выделение/отменить текущий drag).
- Откат последнего действия: `Ctrl+Z`.
- Цвета дуг: влияют только на отображение и фильтры показа (на симуляцию не влияют).

## Форматы файлов

- `Сохранить` / `Сохранить как` -> `gpn2` (формат PetriNet, включает UI-данные).
- `Экспортировать -> Экспортировать в NetStar` -> legacy `.gpn`.

Примечание: элементы, которых нет в NetStar (например, декоративные рамки), сохраняются в `gpn2` и не экспортируются в legacy `.gpn`.

## Экспорт в NetStar

Перед экспортом выполняется проверка: ошибки блокируют экспорт, предупреждения можно пропустить.

## Управление (кратко)

- Режим `Редактировать`: ЛКМ по объекту, ЛКМ+drag по пустому месту для рамочного выделения.
- Инструмент `Дуга`: тянуть ЛКМ от узла к узлу.
- Инструмент `Рамка`: тянуть ЛКМ (прямоугольник), затем можно менять размер.
- Удаление: `Delete`.
- Панорама: ПКМ + drag.

## Сборка

```bash
cargo run
cargo test
cargo build --release
```

## Сборка EXE (Windows)

- `build_exe.ps1` - обычная release-сборка.
- `build_portable_exe.ps1` - portable-сборка (static CRT), создаёт `PetriNet-<version>.exe`.
- `publish_release.ps1` - сборка и публикация релиза в Gitea (нужен `GITEA_TOKEN`).

Скрипт portable-сборки автоматически удаляет старые `PetriNet-*.exe` и оставляет только текущую версию.

## Структура проекта

- `src/main.rs` - точка входа.
- `src/ui/` - интерфейс и инструменты редактора.
- `src/model.rs` - модель сети Петри.
- `src/sim/` - движок имитации.
- `src/io/` - импорт/экспорт (`legacy .gpn`, `gpn2`).
- `tests/` - интеграционные тесты.
- `assets/` - ресурсы приложения.





# scripts\generate_copy.py
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


# src\bin\gpn_dump.rs
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


# src\formats\atf.rs
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


# src\formats\mod.rs
pub mod atf;


# src\io\gpn2.rs
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::model::{PetriNetModel, GPN2_FORMAT_VERSION, GPN2_MAGIC};

pub fn save_gpn2(path: &Path, model: &PetriNetModel) -> Result<()> {
    let mut model = model.clone();
    model.format_version = GPN2_FORMAT_VERSION;
    model.validate()?;

    let json = serde_json::to_string(&model).context("Не удалось сериализовать GPN2")?;
    let mut bytes = Vec::with_capacity(GPN2_MAGIC.len() + json.len());
    bytes.extend_from_slice(GPN2_MAGIC.as_bytes());
    bytes.extend_from_slice(json.as_bytes());

    fs::write(path, bytes)
        .with_context(|| format!("Не удалось записать файл {}", path.display()))?;
    Ok(())
}

pub fn load_gpn2(path: &Path) -> Result<PetriNetModel> {
    let bytes =
        fs::read(path).with_context(|| format!("Не удалось прочитать файл {}", path.display()))?;
    load_gpn2_from_bytes(&bytes)
}

pub fn load_gpn2_from_bytes(bytes: &[u8]) -> Result<PetriNetModel> {
    if !bytes.starts_with(GPN2_MAGIC.as_bytes()) {
        return Err(anyhow!("Файл не содержит заголовок GPN2"));
    }

    let json_bytes = &bytes[GPN2_MAGIC.len()..];
    let value: serde_json::Value =
        serde_json::from_slice(json_bytes).context("Некорректный JSON в GPN2")?;

    let migrated = migrate_to_latest(value)?;
    let model: PetriNetModel =
        serde_json::from_value(migrated).context("JSON не соответствует схеме GPN2")?;
    model.validate()?;
    Ok(model)
}

fn migrate_to_latest(mut value: serde_json::Value) -> Result<serde_json::Value> {
    let Some(version) = value.get("format_version").and_then(|v| v.as_u64()) else {
        return Err(anyhow!("Отсутствует поле format_version"));
    };

    match version as u32 {
        GPN2_FORMAT_VERSION => Ok(value),
        1 => {
            // TODO: Миграция legacy JSON версии 1 -> GPN2 при необходимости.
            value["format_version"] = serde_json::Value::from(GPN2_FORMAT_VERSION);
            Ok(value)
        }
        other => Err(anyhow!("Неподдерживаемая версия формата: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeRef, PetriNetModel};

    #[test]
    fn roundtrip_save_load() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut model = PetriNetModel::new();
        model.add_place([1.0, 2.0]);
        model.add_transition([3.0, 4.0]);
        let p_id = model.places[0].id;
        let t_id = model.transitions[0].id;
        model.add_arc(NodeRef::Place(p_id), NodeRef::Transition(t_id), 2);

        save_gpn2(tmp.path(), &model).unwrap();
        let loaded = load_gpn2(tmp.path()).unwrap();

        assert_eq!(model, loaded);
    }

    #[test]
    fn header_detection() {
        let bytes = b"NOPE{}";
        assert!(load_gpn2_from_bytes(bytes).is_err());
    }

    #[test]
    fn validation_failure_for_duplicate_ids() {
        let mut model = PetriNetModel::new();
        model.add_place([0.0, 0.0]);
        model.add_place([1.0, 1.0]);
        model.places[1].id = model.places[0].id;

        let json = serde_json::to_string_pretty(&model).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(GPN2_MAGIC.as_bytes());
        bytes.extend_from_slice(json.as_bytes());

        assert!(load_gpn2_from_bytes(&bytes).is_err());
    }
}


# src\io\legacy_gpn.rs
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::model::{NodeColor, NodeRef, PetriNetModel, VisualSize};

const PLACE_RECORD_SIZE: usize = 231;
const PLACE_DELAY_OFFSET: usize = 77;
const PLACE_NAME_OFFSET: usize = 26;
const TRANSITION_RECORD_SIZE: usize = 105;
const TRANSITION_NAME_OFFSET: usize = 54;
const ARC_SECTION_HEADER_SIZE: usize = 6;
const ARC_RECORD_SIZE: usize = 46;

#[derive(Debug, Clone)]
pub struct LegacyDebugInfo {
    pub file_size: usize,
    pub candidate_counts: Vec<(usize, u32, u32)>,
    pub discovered_sections: Vec<String>,
    pub ascii_strings: Vec<(usize, String)>,
    pub utf16le_strings: Vec<(usize, String)>,
    pub candidate_float64_pairs: Vec<(usize, f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct LegacyImportResult {
    pub model: PetriNetModel,
    pub warnings: Vec<String>,
    pub debug: LegacyDebugInfo,
}

#[derive(Debug, Clone, Default)]
pub struct LegacyExportHints {
    pub places_count: Option<usize>,
    pub transitions_count: Option<usize>,
    pub arc_topology_fingerprint: Option<u64>,
    pub arc_header_extra: Option<u16>,
    pub footer_bytes: Option<Vec<u8>>,
    pub raw_arc_and_tail: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum LegacyImportError {
    Io(std::io::Error),
    Invalid(String),
}

impl fmt::Display for LegacyImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Ошибка ввода-вывода: {e}"),
            Self::Invalid(msg) => write!(f, "Некорректный legacy GPN: {msg}"),
        }
    }
}

impl std::error::Error for LegacyImportError {}

impl From<std::io::Error> for LegacyImportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, Copy)]
struct LegacyLayout {
    place_header_size: usize,
    places_offset: usize,
    transitions_offset: usize,
    arcs_offset: usize,
}

pub fn detect_legacy_gpn(bytes: &[u8]) -> bool {
    !bytes.starts_with(crate::model::GPN2_MAGIC.as_bytes())
}

pub fn import_legacy_gpn(path: &Path) -> Result<LegacyImportResult, LegacyImportError> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() {
        return Err(LegacyImportError::Invalid("Пустой файл".to_string()));
    }

    let candidate_counts = detect_counts(&bytes);
    let ascii_strings = extract_ascii_strings(&bytes, 4);
    let utf16le_strings = extract_utf16le_strings(&bytes, 4);
    let candidate_float64_pairs = extract_float64_pairs(&bytes, 256);

    let (places_count, transitions_count, mut warnings) =
        if let Some((p, t)) = header_counts_from_prefix(&bytes) {
            (p, t, Vec::new())
        } else if let Some((_, p, t)) = candidate_counts.first().copied() {
            (
                p.clamp(1, 2000) as usize,
                t.clamp(1, 2000) as usize,
                vec!["Использованы эвристические counts".to_string()],
            )
        } else {
            (
                1,
                1,
                vec![
                    "Не удалось надежно извлечь числа мест/переходов, применены значения 1/1"
                        .to_string(),
                ],
            )
        };

    let mut model = PetriNetModel::new();
    model.set_counts(places_count, transitions_count);

    let mut used_fallback = false;
    let layout = detect_legacy_layout(&bytes, places_count, transitions_count);
    if let Some(layout) = layout {
        let parsed_place_nodes = parse_place_nodes_from_layout(&bytes, places_count, layout);
        let parsed_transition_nodes =
            parse_transition_nodes_from_layout(&bytes, transitions_count, layout);
        let mut arcs_applied = false;

        for (idx, place) in model.places.iter_mut().enumerate() {
            if let Some(first) = parsed_place_nodes.get(idx).cloned() {
                if !first.valid {
                    continue;
                }
                place.pos = [first.x, first.y];
                place.size = VisualSize::Small;
                if !first.name.is_empty() {
                    place.name = first.name.clone();
                    if place.note.trim().is_empty() {
                        place.note = first.name;
                    }
                }
                model.tables.m0[idx] = first.markers.max(0) as u32;
                model.tables.mo[idx] = if first.capacity > 0 {
                    Some(first.capacity as u32)
                } else {
                    None
                };
                model.tables.mz[idx] = first.delay_sec.max(0.0);
                place.color = map_legacy_color(first.color_raw);
            }
        }

        for (idx, tr) in model.transitions.iter_mut().enumerate() {
            if let Some(first) = parsed_transition_nodes.get(idx).cloned() {
                if !first.valid {
                    continue;
                }
                tr.size = VisualSize::Medium;
                let (w, h) = legacy_transition_dims(tr.size);
                tr.pos = [first.x - w * 0.5, first.y - h * 0.5];
                if !first.name.is_empty() {
                    tr.name = first.name.clone();
                    if tr.note.trim().is_empty() {
                        tr.note = first.name;
                    }
                }
                model.tables.mpr[idx] = first.priority;
                tr.color = map_legacy_color(first.color_raw);
            }
        }

        if let Some(arcs) = parse_arcs_from_section(&bytes, places_count, transitions_count, layout)
        {
            apply_legacy_arcs(&mut model, &arcs);
            arcs_applied = true;
        } else if let Some(arcs) = parse_arcs_by_signature(
            &bytes,
            places_count,
            transitions_count,
            &model.places,
            &model.transitions,
        ) {
            used_fallback = true;
            warnings.push("Дуги восстановлены по сигнатурам".to_string());
            apply_legacy_arcs(&mut model, &arcs);
            arcs_applied = true;
        } else {
            used_fallback = true;
            warnings.push("Не удалось извлечь дуги".to_string());
        }
        if arcs_applied {
            prune_legacy_ghost_nodes(&mut model);
            // Read-arc heuristic is useful mostly for old files without explicit inhibitor arcs.
            // When inhibitor arcs are present, forcing extra Post edges may distort dynamics.
            if model.inhibitor_arcs.is_empty() {
                apply_legacy_read_arc_heuristics(&mut model);
            }
        }
    } else {
        used_fallback = true;
        warnings.push("Не удалось определить layout legacy секций".to_string());
    }

    if used_fallback {
        warnings.push("Импорт legacy GPN выполнен в режиме best-effort".to_string());
    }

    let mut discovered_sections = detect_section_boundaries(&bytes);
    if let Some(layout) = layout {
        discovered_sections.push(format!(
            "layout: header={}, places@0x{:X}, transitions@0x{:X}, arcs@0x{:X}",
            layout.place_header_size,
            layout.places_offset,
            layout.transitions_offset,
            layout.arcs_offset
        ));
    }

    let debug = LegacyDebugInfo {
        file_size: bytes.len(),
        candidate_counts,
        discovered_sections,
        ascii_strings,
        utf16le_strings,
        candidate_float64_pairs,
    };

    Ok(LegacyImportResult {
        model,
        warnings,
        debug,
    })
}

pub fn export_legacy_gpn(path: &Path, model: &PetriNetModel) -> std::io::Result<()> {
    export_legacy_gpn_with_hints(path, model, None)
}

pub fn export_legacy_gpn_with_hints(
    path: &Path,
    model: &PetriNetModel,
    _hints: Option<&LegacyExportHints>,
) -> std::io::Result<()> {
    let mut normalized = model.clone();
    normalized.rebuild_matrices_from_arcs();

    let places_count = normalized.places.len();
    let transitions_count = normalized.transitions.len();
    let place_legacy_idx: HashMap<u64, usize> = normalized
        .places
        .iter()
        .enumerate()
        .map(|(idx, place)| (place.id, idx + 1))
        .collect();
    let transition_legacy_idx: HashMap<u64, usize> = normalized
        .transitions
        .iter()
        .enumerate()
        .map(|(idx, transition)| (transition.id, idx + 1))
        .collect();

    let mut bytes = Vec::new();
    push_i32(&mut bytes, places_count as i32);
    push_i32(&mut bytes, transitions_count as i32);
    push_i32(&mut bytes, 0x20);
    push_i32(&mut bytes, 0);

    let looks_like_auto_place_name = |name: &str| -> bool {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !['P', 'p', 'Р', 'р'].contains(&first) {
            return false;
        }
        let rest: String = chars.collect();
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    };

    for idx in 0..places_count {
        let mut record = [0u8; PLACE_RECORD_SIZE];
        let place = &normalized.places[idx];
        let mo_raw = normalized
            .tables
            .mo
            .get(idx)
            .and_then(|value| *value)
            .unwrap_or(1)
            .clamp(1, 1_000_000);
        let markers = normalized
            .tables
            .m0
            .get(idx)
            .copied()
            .unwrap_or(0)
            .min(mo_raw)
            .min(1_000_000);
        write_i32(&mut record, 0, round_i32(place.pos[0]));
        write_i32(&mut record, 4, round_i32(place.pos[1]));
        write_i32(&mut record, 8, 10);
        write_i32(&mut record, 12, markers as i32);
        // Legacy variants disagree on the Mo field offset; write both to maximize NetStar compatibility.
        write_i32(&mut record, 16, mo_raw as i32);
        write_i32(&mut record, 112, mo_raw as i32);
        write_i32(&mut record, 20, map_color_to_legacy(place.color));
        let delay = normalized
            .tables
            .mz
            .get(idx)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, 86_400.0);
        write_f64(&mut record, PLACE_DELAY_OFFSET, delay);

        // NetStar displays the place label from this legacy field.
        // Prefer the explicit name; if it's an auto-name (P1, P2, ...) and note is filled,
        // export note instead so "Текст/Описание" is visible in NetStar.
        let place_label =
            if looks_like_auto_place_name(&place.name) && !place.note.trim().is_empty() {
                place.note.as_str()
            } else {
                place.name.as_str()
            };
        // Keep place name inside the safe legacy slot so it does not overwrite delay bytes.
        let place_name_max = PLACE_DELAY_OFFSET.saturating_sub(PLACE_NAME_OFFSET + 1);
        write_legacy_name_limited(&mut record, PLACE_NAME_OFFSET, place_label, place_name_max);
        bytes.extend_from_slice(&record);
    }

    for idx in 0..transitions_count {
        let mut record = [0u8; TRANSITION_RECORD_SIZE];
        let transition = &normalized.transitions[idx];
        let (w, h) = legacy_transition_dims(transition.size);
        write_i32(&mut record, 0, round_i32(transition.pos[0] + w * 0.5));
        write_i32(&mut record, 4, round_i32(transition.pos[1] + h * 0.5));
        write_i32(
            &mut record,
            8,
            normalized
                .tables
                .mpr
                .get(idx)
                .copied()
                .unwrap_or(1)
                .clamp(0, 1_000_000),
        );
        write_i32(&mut record, 16, -131072);
        write_i32(&mut record, 20, -589825);
        write_i32(&mut record, 24, 196607);
        write_i32(&mut record, 28, -655360);
        write_i32(&mut record, 32, 196607);
        write_i32(&mut record, 36, 655360);
        write_i32(&mut record, 40, -131072);
        write_i32(&mut record, 44, 720895);
        write_i32(&mut record, 52, map_color_to_legacy(transition.color));

        // NetStar displays the transition label from this legacy field.
        let tr_label = if transition.note.trim().is_empty() {
            transition.name.as_str()
        } else {
            transition.note.as_str()
        };
        // Reserve one byte at end to avoid touching undocumented tail fields.
        let transition_name_max = TRANSITION_RECORD_SIZE
            .saturating_sub(TRANSITION_NAME_OFFSET + 1)
            .saturating_sub(1);
        write_legacy_name_limited(
            &mut record,
            TRANSITION_NAME_OFFSET,
            tr_label,
            transition_name_max,
        );
        bytes.extend_from_slice(&record);
    }

    let mut arc_records = Vec::<(bool, i32, usize, usize, NodeRef, NodeRef)>::new();

    for arc in &normalized.arcs {
        match (arc.from, arc.to) {
            (NodeRef::Place(place_id), NodeRef::Transition(transition_id)) => {
                let Some(&place_idx) = place_legacy_idx.get(&place_id) else {
                    continue;
                };
                let Some(&transition_idx) = transition_legacy_idx.get(&transition_id) else {
                    continue;
                };
                for _ in 0..arc.weight.clamp(1, 1024) {
                    arc_records.push((
                        false,
                        -1,
                        place_idx,
                        transition_idx,
                        NodeRef::Place(place_id),
                        NodeRef::Transition(transition_id),
                    ));
                }
            }
            (NodeRef::Transition(transition_id), NodeRef::Place(place_id)) => {
                let Some(&place_idx) = place_legacy_idx.get(&place_id) else {
                    continue;
                };
                let Some(&transition_idx) = transition_legacy_idx.get(&transition_id) else {
                    continue;
                };
                for _ in 0..arc.weight.clamp(1, 1024) {
                    arc_records.push((
                        false,
                        1,
                        transition_idx,
                        place_idx,
                        NodeRef::Transition(transition_id),
                        NodeRef::Place(place_id),
                    ));
                }
            }
            _ => {}
        }
    }

    for inhibitor in &normalized.inhibitor_arcs {
        let Some(&place_idx) = place_legacy_idx.get(&inhibitor.place_id) else {
            continue;
        };
        let Some(&transition_idx) = transition_legacy_idx.get(&inhibitor.transition_id) else {
            continue;
        };
        for _ in 0..inhibitor.threshold.clamp(1, 1024) {
            arc_records.push((
                true,
                -1,
                place_idx,
                transition_idx,
                NodeRef::Place(inhibitor.place_id),
                NodeRef::Transition(inhibitor.transition_id),
            ));
        }
    }
    arc_records.sort_by_key(|(inhibitor, direction, a, b, _, _)| (*inhibitor, *direction, *a, *b));

    let mut encoded_arcs = Vec::new();
    for (inhibitor, direction, source_idx, target_idx, from_node, to_node) in arc_records {
        let points = legacy_arc_polyline_points(&normalized, from_node, to_node).unwrap_or((
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        ));
        encoded_arcs.push((inhibitor, direction, source_idx, target_idx, points));
    }

    let arc_max_index = encoded_arcs
        .len()
        .checked_sub(1)
        .and_then(|value| i32::try_from(value).ok())
        .unwrap_or(-1);
    push_i32(&mut bytes, arc_max_index);
    // NetStar legacy expects this header value to be 99 (0x63).
    let arc_extra = 99u16;
    bytes.extend_from_slice(&arc_extra.to_le_bytes());
    for (inhibitor, direction, source_idx, target_idx, (p1, p2, p3)) in encoded_arcs {
        let p1x = clamp_u16(p1[0]);
        let p1y = clamp_u16(p1[1]);
        let p2x = clamp_u16(p2[0]);
        let p2y = clamp_u16(p2[1]);
        let p3x = clamp_u16(p3[0]);
        let p3y = clamp_u16(p3[1]);
        let record = LegacyArcBinaryRecord {
            marker: if inhibitor { 0 } else { 1 },
            direction,
            source_raw: source_idx as i32,
            target_raw: target_idx as i32,
            p1x,
            p1y,
            p2x,
            p2y,
            p3x: p3x as i32,
            p3y,
        }
        .encode();
        bytes.extend_from_slice(&record);
    }
    bytes.extend_from_slice(legacy_footer_template());

    fs::write(path, bytes)
}

#[derive(Debug, Clone)]
struct LegacyTransitionNode {
    valid: bool,
    x: f32,
    y: f32,
    priority: i32,
    color_raw: i32,
    name: String,
}

fn header_counts_from_prefix(bytes: &[u8]) -> Option<(usize, usize)> {
    let places = read_i32(bytes, 0)?;
    let transitions = read_i32(bytes, 4)?;
    if !(1..=10_000).contains(&places) || !(1..=10_000).contains(&transitions) {
        return None;
    }
    Some((places as usize, transitions as usize))
}

fn detect_legacy_layout(
    bytes: &[u8],
    places_count: usize,
    transitions_count: usize,
) -> Option<LegacyLayout> {
    if places_count == 0 || transitions_count == 0 {
        return None;
    }

    let mut best: Option<(LegacyLayout, i32)> = None;
    for place_header_size in [16usize, 247usize] {
        let places_offset = place_header_size;
        let transitions_offset =
            places_offset.saturating_add(places_count.saturating_mul(PLACE_RECORD_SIZE));
        let arcs_offset = transitions_offset
            .saturating_add(transitions_count.saturating_mul(TRANSITION_RECORD_SIZE));
        if arcs_offset + 4 > bytes.len() {
            continue;
        }

        let mut score = 0i32;
        for idx in 0..places_count {
            let off = places_offset + idx * PLACE_RECORD_SIZE;
            let Some(x) = read_i32(bytes, off) else {
                break;
            };
            let Some(y) = read_i32(bytes, off + 4) else {
                break;
            };
            if (0..=50_000).contains(&x) && (0..=50_000).contains(&y) {
                score += 1;
            }
        }

        for idx in 0..transitions_count {
            let off = transitions_offset + idx * TRANSITION_RECORD_SIZE;
            let Some(x) = read_i32(bytes, off) else {
                break;
            };
            let Some(y) = read_i32(bytes, off + 4) else {
                break;
            };
            if (-50_000..=50_000).contains(&x) && (-50_000..=50_000).contains(&y) {
                score += 1;
            }
        }

        let layout = LegacyLayout {
            place_header_size,
            places_offset,
            transitions_offset,
            arcs_offset,
        };
        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((layout, score)),
        }
    }
    best.map(|(layout, _)| layout)
}

fn parse_transition_nodes_from_layout(
    bytes: &[u8],
    needed: usize,
    layout: LegacyLayout,
) -> Vec<LegacyTransitionNode> {
    let mut result = Vec::new();
    for idx in 0..needed {
        let off = layout.transitions_offset + idx * TRANSITION_RECORD_SIZE;
        let Some(x) = read_i32(bytes, off) else {
            break;
        };
        let Some(y) = read_i32(bytes, off + 4) else {
            break;
        };
        let priority = read_i32(bytes, off + 8).unwrap_or(1);
        let color_raw = read_i32(bytes, off + 52).unwrap_or(0);
        let name = read_legacy_name(bytes, off, TRANSITION_RECORD_SIZE, TRANSITION_NAME_OFFSET);
        let valid = (-50_000..=50_000).contains(&x) && (-50_000..=50_000).contains(&y);
        result.push(LegacyTransitionNode {
            valid,
            x: x as f32,
            y: y as f32,
            priority: priority.clamp(0, 1_000_000),
            color_raw,
            name,
        });
    }
    result
}

#[derive(Debug, Clone)]
struct LegacyPlaceNode {
    valid: bool,
    x: f32,
    y: f32,
    markers: i32,
    delay_sec: f64,
    capacity: i32,
    color_raw: i32,
    name: String,
}

fn detect_place_capacity_offset(bytes: &[u8], needed: usize, layout: LegacyLayout) -> usize {
    // Legacy variants exist: in some files the "capacity/Mo" field is not at +16.
    // We pick the most plausible offset by sampling first records and scoring candidates.
    const CANDIDATES: [usize; 3] = [16, 24, 28];
    let sample_n = needed.min(64);

    let mut best_off = 16usize;
    let mut best_score: i64 = i64::MIN;

    for cap_off in CANDIDATES {
        let mut ok = 0i64;
        let mut nonzero = 0i64;
        let mut invalid = 0i64;

        for idx in 0..sample_n {
            let off = layout.places_offset + idx * PLACE_RECORD_SIZE + cap_off;
            let Some(v) = read_i32(bytes, off) else {
                invalid += 1;
                continue;
            };
            if !(0..=1_000_000).contains(&v) {
                invalid += 1;
                continue;
            }
            ok += 1;
            if v != 0 {
                nonzero += 1;
            }
        }

        // Favor offsets that decode "reasonable" non-negative integers and aren't mostly invalid.
        let score = ok * 2 + nonzero * 3 - invalid * 5;
        if score > best_score || (score == best_score && cap_off == 16) {
            best_score = score;
            best_off = cap_off;
        }
    }

    best_off
}

fn parse_place_nodes_from_layout(
    bytes: &[u8],
    needed: usize,
    layout: LegacyLayout,
) -> Vec<LegacyPlaceNode> {
    #[derive(Debug, Clone)]
    struct RawPlaceNode {
        valid: bool,
        x: f32,
        y: f32,
        marker8: i32,
        marker12: i32,
        delay_sec: f64,
        capacity: i32,
        color_raw: i32,
        name: String,
    }

    let mut raw = Vec::<RawPlaceNode>::new();
    let capacity_off = detect_place_capacity_offset(bytes, needed, layout);
    for idx in 0..needed {
        let off = layout.places_offset + idx * PLACE_RECORD_SIZE;
        let Some(x) = read_i32(bytes, off) else {
            break;
        };
        let Some(y) = read_i32(bytes, off + 4) else {
            break;
        };
        let valid = (0..=20_000).contains(&x) && (0..=20_000).contains(&y);
        let marker8 = read_i32(bytes, off + 8).unwrap_or(0);
        let marker12 = read_i32(bytes, off + 12).unwrap_or(0);
        let delay_raw = read_f64(bytes, off + PLACE_DELAY_OFFSET);
        let delay_fallback = marker12 as f64;
        let capacity = read_i32(bytes, off + capacity_off).unwrap_or(0);
        let color_raw = read_i32(bytes, off + 20).unwrap_or(0);
        let name = read_legacy_name(bytes, off, PLACE_RECORD_SIZE, PLACE_NAME_OFFSET);
        raw.push(RawPlaceNode {
            valid,
            x: x as f32,
            y: y as f32,
            marker8,
            marker12,
            delay_sec: delay_raw
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(delay_fallback.max(0.0)),
            capacity,
            color_raw,
            name,
        });
    }

    let marker_pairs: Vec<(i32, i32)> = raw
        .iter()
        .filter(|node| node.valid)
        .map(|node| (node.marker8, node.marker12))
        .collect();
    let use_marker12 = should_use_marker12(&marker_pairs);
    raw.into_iter()
        .map(|node| LegacyPlaceNode {
            valid: node.valid,
            x: node.x,
            y: node.y,
            markers: if node.valid {
                if use_marker12 {
                    node.marker12
                } else {
                    node.marker8
                }
                .clamp(0, 1_000_000)
            } else {
                0
            },
            delay_sec: node.delay_sec,
            capacity: node.capacity,
            color_raw: node.color_raw,
            name: node.name,
        })
        .collect()
}

fn should_use_marker12(marker_pairs: &[(i32, i32)]) -> bool {
    if marker_pairs.is_empty() {
        return false;
    }
    let total = marker_pairs.len();
    let mut marker12_has_large_value = false;
    let mut marker8_is_legacy_sentinel = 0usize;
    let mut marker12_is_binary = 0usize;

    for (marker8, marker12) in marker_pairs.iter().copied() {
        if marker12 > 1 {
            marker12_has_large_value = true;
        }
        if marker8 == 10 {
            marker8_is_legacy_sentinel += 1;
        }
        if marker12 == 0 || marker12 == 1 {
            marker12_is_binary += 1;
        }
    }

    if marker12_has_large_value {
        return true;
    }

    marker8_is_legacy_sentinel * 10 >= total * 8 && marker12_is_binary * 10 >= total * 8
}

#[derive(Debug, Clone, Copy)]
struct LegacyArcRecord {
    place_idx: usize,
    transition_idx: usize,
    place_to_transition: bool,
    weight: u32,
    inhibitor: bool,
}

#[derive(Debug, Clone, Copy)]
struct LegacyArcBinaryRecord {
    marker: i32,
    direction: i32,
    source_raw: i32,
    target_raw: i32,
    p1x: u16,
    p1y: u16,
    p2x: u16,
    p2y: u16,
    p3x: i32,
    p3y: u16,
}

impl LegacyArcBinaryRecord {
    fn decode(bytes: &[u8], off: usize) -> Option<Self> {
        if off + ARC_RECORD_SIZE > bytes.len() {
            return None;
        }
        let p1y = u16::from_le_bytes([bytes[off + 2], bytes[off + 3]]);
        let p2y = u16::from_le_bytes([bytes[off + 6], bytes[off + 7]]);
        let p1x = u16::from_le_bytes([bytes[off + 10], bytes[off + 11]]);
        let p3y = u16::from_le_bytes([bytes[off + 14], bytes[off + 15]]);
        let marker = read_i32(bytes, off + 24)?;
        let direction = read_i32(bytes, off + 28)?;
        let source_raw = read_i32(bytes, off + 32)?;
        let target_raw = read_i32(bytes, off + 36)?;
        let p3x = read_i32(bytes, off + 40)?;
        let p2x = u16::from_le_bytes([bytes[off + 44], bytes[off + 45]]);
        Some(Self {
            marker,
            direction,
            source_raw,
            target_raw,
            p1x,
            p1y,
            p2x,
            p2y,
            p3x,
            p3y,
        })
    }

    fn encode(&self) -> [u8; ARC_RECORD_SIZE] {
        let mut record = [0u8; ARC_RECORD_SIZE];
        write_u16(&mut record, 2, self.p1y);
        write_u16(&mut record, 6, self.p2y);
        write_u16(&mut record, 10, self.p1x);
        write_u16(&mut record, 14, self.p3y);
        write_i32(&mut record, 24, self.marker);
        write_i32(&mut record, 28, self.direction);
        write_i32(&mut record, 32, self.source_raw);
        write_i32(&mut record, 36, self.target_raw);
        write_i32(&mut record, 40, self.p3x);
        write_u16(&mut record, 44, self.p2x);
        record
    }

    fn to_topology(self, places: usize, transitions: usize) -> Option<(usize, usize, bool, bool)> {
        if self.marker != 0 && self.marker != 1 {
            return None;
        }
        if self.direction != -1 && self.direction != 1 {
            return None;
        }
        if self.source_raw < 1 || self.target_raw < 1 {
            return None;
        }
        let (place_idx, transition_idx, place_to_transition) = if self.direction == -1 {
            if self.source_raw > places as i32 || self.target_raw > transitions as i32 {
                return None;
            }
            (
                (self.source_raw - 1) as usize,
                (self.target_raw - 1) as usize,
                true,
            )
        } else {
            if self.source_raw > transitions as i32 || self.target_raw > places as i32 {
                return None;
            }
            (
                (self.target_raw - 1) as usize,
                (self.source_raw - 1) as usize,
                false,
            )
        };
        Some((
            place_idx,
            transition_idx,
            place_to_transition,
            self.marker == 0,
        ))
    }
}
fn parse_arcs_from_section(
    bytes: &[u8],
    places: usize,
    transitions: usize,
    layout: LegacyLayout,
) -> Option<Vec<LegacyArcRecord>> {
    let arc_counter = read_i32(bytes, layout.arcs_offset)?;
    if arc_counter < -1 {
        return None;
    }
    let mut arc_count = (arc_counter + 1).max(0) as usize;
    let section_start = layout.arcs_offset + ARC_SECTION_HEADER_SIZE;
    if section_start > bytes.len() {
        return None;
    }
    let max_records = (bytes.len().saturating_sub(section_start)) / ARC_RECORD_SIZE;
    arc_count = arc_count.min(max_records);
    let mut counts = HashMap::<(usize, usize, bool, bool), u32>::new();

    let mut parsed_records = 0usize;
    for index in 0..arc_count {
        let off = section_start + index * ARC_RECORD_SIZE;
        let Some(bin) = LegacyArcBinaryRecord::decode(bytes, off) else {
            continue;
        };
        if let Some((place_idx, transition_idx, place_to_transition, inhibitor)) =
            bin.to_topology(places, transitions)
        {
            *counts
                .entry((place_idx, transition_idx, place_to_transition, inhibitor))
                .or_insert(0) += 1;
            parsed_records += 1;
        }
    }

    if parsed_records == 0 {
        return None;
    }

    let mut arcs = counts
        .into_iter()
        .map(
            |((place_idx, transition_idx, place_to_transition, inhibitor), weight)| {
                LegacyArcRecord {
                    place_idx,
                    transition_idx,
                    place_to_transition,
                    weight: weight.max(1),
                    inhibitor,
                }
            },
        )
        .collect::<Vec<_>>();
    arcs.sort_by_key(|arc| {
        (
            arc.place_idx,
            arc.transition_idx,
            arc.place_to_transition,
            arc.inhibitor,
        )
    });
    if arcs.is_empty() {
        None
    } else {
        Some(arcs)
    }
}

fn apply_legacy_arcs(model: &mut PetriNetModel, arcs: &[LegacyArcRecord]) {
    model.arcs.clear();
    model.inhibitor_arcs.clear();

    let mut next_arc_like_id = 1_u64;
    for arc in arcs {
        if arc.place_idx >= model.places.len() || arc.transition_idx >= model.transitions.len() {
            continue;
        }
        let place_id = model.places[arc.place_idx].id;
        let transition_id = model.transitions[arc.transition_idx].id;
        if arc.inhibitor {
            model.inhibitor_arcs.push(crate::model::InhibitorArc {
                id: next_arc_like_id,
                place_id,
                transition_id,
                threshold: arc.weight.max(1),
                color: crate::model::NodeColor::Red,
                visible: true,
                show_weight: false,
            });
            next_arc_like_id = next_arc_like_id.saturating_add(1);
        } else {
            let (from, to) = if arc.place_to_transition {
                (NodeRef::Place(place_id), NodeRef::Transition(transition_id))
            } else {
                (NodeRef::Transition(transition_id), NodeRef::Place(place_id))
            };
            model.arcs.push(crate::model::Arc {
                id: next_arc_like_id,
                from,
                to,
                weight: arc.weight.max(1),
                color: crate::model::NodeColor::Default,
                visible: true,
                show_weight: false,
            });
            next_arc_like_id = next_arc_like_id.saturating_add(1);
        }
    }
    model.rebuild_matrices_from_arcs();
}

fn prune_legacy_ghost_nodes(model: &mut PetriNetModel) {
    let place_count = model.places.len();
    let transition_count = model.transitions.len();
    if place_count == 0 || transition_count == 0 {
        return;
    }

    let mut place_incident = vec![false; place_count];
    let mut transition_incident = vec![false; transition_count];
    let place_index = model.place_index_map();
    let transition_index = model.transition_index_map();

    for arc in &model.arcs {
        match (arc.from, arc.to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid)) => {
                if let Some(&pi) = place_index.get(&pid) {
                    place_incident[pi] = true;
                }
                if let Some(&ti) = transition_index.get(&tid) {
                    transition_incident[ti] = true;
                }
            }
            (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
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
    for inh in &model.inhibitor_arcs {
        if let Some(&pi) = place_index.get(&inh.place_id) {
            place_incident[pi] = true;
        }
        if let Some(&ti) = transition_index.get(&inh.transition_id) {
            transition_incident[ti] = true;
        }
    }

    let mut keep_places = vec![true; place_count];
    for idx in 0..place_count {
        if place_incident[idx] {
            continue;
        }
        let node = &model.places[idx];
        let duplicate_connected_exists =
            model.places.iter().enumerate().any(|(other_idx, other)| {
                other_idx != idx
                    && place_incident[other_idx]
                    && near_point(other.pos, node.pos[0], node.pos[1], 0.5)
            });
        if duplicate_connected_exists {
            keep_places[idx] = false;
        }
    }

    let mut keep_transitions = vec![true; transition_count];
    for idx in 0..transition_count {
        if transition_incident[idx] {
            continue;
        }
        let node = &model.transitions[idx];
        let duplicate_connected_exists =
            model
                .transitions
                .iter()
                .enumerate()
                .any(|(other_idx, other)| {
                    other_idx != idx
                        && transition_incident[other_idx]
                        && near_point(other.pos, node.pos[0], node.pos[1], 0.5)
                });
        if duplicate_connected_exists {
            keep_transitions[idx] = false;
        }
    }

    if keep_places.iter().all(|keep| *keep) && keep_transitions.iter().all(|keep| *keep) {
        return;
    }

    let old_places = model.places.clone();
    let old_transitions = model.transitions.clone();
    let old_tables = model.tables.clone();

    let mut place_old_to_new = vec![None; old_places.len()];
    let mut transition_old_to_new = vec![None; old_transitions.len()];

    model.places.clear();
    for (old_idx, place) in old_places.into_iter().enumerate() {
        if keep_places[old_idx] {
            place_old_to_new[old_idx] = Some(model.places.len());
            model.places.push(place);
        }
    }

    model.transitions.clear();
    for (old_idx, tr) in old_transitions.into_iter().enumerate() {
        if keep_transitions[old_idx] {
            transition_old_to_new[old_idx] = Some(model.transitions.len());
            model.transitions.push(tr);
        }
    }

    let keep_place_ids = model.place_index_map();
    let keep_transition_ids = model.transition_index_map();
    model.arcs.retain(|arc| match (arc.from, arc.to) {
        (NodeRef::Place(pid), NodeRef::Transition(tid))
        | (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
            keep_place_ids.contains_key(&pid) && keep_transition_ids.contains_key(&tid)
        }
        _ => false,
    });
    model.inhibitor_arcs.retain(|arc| {
        keep_place_ids.contains_key(&arc.place_id)
            && keep_transition_ids.contains_key(&arc.transition_id)
    });

    model
        .tables
        .resize(model.places.len(), model.transitions.len());
    for (old_idx, maybe_new_idx) in place_old_to_new.into_iter().enumerate() {
        let Some(new_idx) = maybe_new_idx else {
            continue;
        };
        model.tables.m0[new_idx] = old_tables.m0.get(old_idx).copied().unwrap_or(0);
        model.tables.mo[new_idx] = old_tables.mo.get(old_idx).copied().unwrap_or(None);
        model.tables.mz[new_idx] = old_tables.mz.get(old_idx).copied().unwrap_or(0.0);
    }
    for (old_idx, maybe_new_idx) in transition_old_to_new.into_iter().enumerate() {
        let Some(new_idx) = maybe_new_idx else {
            continue;
        };
        model.tables.mpr[new_idx] = old_tables.mpr.get(old_idx).copied().unwrap_or(1);
    }

    model.rebuild_matrices_from_arcs();
}

fn apply_legacy_read_arc_heuristics(model: &mut PetriNetModel) {
    // NetStar legacy files can encode "resource" places as ordinary arcs, but semantically those arcs
    // may behave like read-arcs (test for token without consuming it). Without this, some imported
    // networks deadlock immediately.
    let places = model.places.len();
    let transitions = model.transitions.len();
    if places == 0 || transitions == 0 {
        return;
    }

    let mut changed = false;
    for p in 0..places {
        let name = model.places[p].name.to_lowercase();
        let looks_like_free_resource = name.contains("свобод") || name.contains("free");
        if !looks_like_free_resource {
            continue;
        }

        // Heuristic: 1 token resource used by many transitions, and at least one transition returns it.
        if model.tables.m0.get(p).copied().unwrap_or(0) != 1 {
            continue;
        }
        let outgoing = (0..transitions)
            .filter(|&t| model.tables.pre[p][t] > 0)
            .count();
        let incoming = (0..transitions)
            .filter(|&t| model.tables.post[p][t] > 0)
            .count();
        if outgoing < 3 || incoming == 0 {
            continue;
        }

        for t in 0..transitions {
            let pre = model.tables.pre[p][t];
            if pre > 0 && model.tables.post[p][t] == 0 {
                model.tables.post[p][t] = pre;
                changed = true;
            }
        }
    }

    if changed {
        model.rebuild_arcs_from_matrices();
    }
}

fn parse_arcs_by_signature(
    bytes: &[u8],
    places: usize,
    transitions: usize,
    place_nodes: &[crate::model::Place],
    transition_nodes: &[crate::model::Transition],
) -> Option<Vec<LegacyArcRecord>> {
    if bytes.len() < 64 || places == 0 || transitions == 0 {
        return None;
    }

    let read_i32 = |off: usize| -> Option<i32> {
        if off + 4 > bytes.len() {
            None
        } else {
            Some(i32::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
            ]))
        }
    };

    let mut counts_all = HashMap::<(usize, usize, bool, bool), u32>::new();
    let mut counts_filtered = HashMap::<(usize, usize, bool, bool), u32>::new();
    for off in 0..bytes.len().saturating_sub(64) {
        let Some(marker) = read_i32(off + 24) else {
            continue;
        };
        if marker != 1 && marker != 0 {
            continue;
        }
        let Some(direction_raw) = read_i32(off + 28) else {
            continue;
        };
        if direction_raw != -1 && direction_raw != 1 {
            continue;
        }
        let Some(source_raw) = read_i32(off + 32) else {
            continue;
        };
        let Some(target_raw) = read_i32(off + 36) else {
            continue;
        };
        if source_raw < 1 || target_raw < 1 {
            continue;
        }
        let x1 = read_i32(off + 40).unwrap_or(0);
        let y1 = read_i32(off + 44).unwrap_or(0);
        let x2 = read_i32(off + 56).unwrap_or(0);
        let y2 = read_i32(off + 60).unwrap_or(0);
        if !(-50_000..=50_000).contains(&x1)
            || !(-50_000..=50_000).contains(&y1)
            || !(-50_000..=50_000).contains(&x2)
            || !(-50_000..=50_000).contains(&y2)
        {
            continue;
        }
        let (place_idx, transition_idx, place_to_transition) = if direction_raw == -1 {
            if source_raw > places as i32 || target_raw > transitions as i32 {
                continue;
            }
            ((source_raw - 1) as usize, (target_raw - 1) as usize, true)
        } else {
            if source_raw > transitions as i32 || target_raw > places as i32 {
                continue;
            }
            ((target_raw - 1) as usize, (source_raw - 1) as usize, false)
        };
        let inhibitor = marker == 0;
        let dedup_key = (place_idx, transition_idx, place_to_transition, inhibitor);
        *counts_all.entry(dedup_key).or_insert(0) += 1;

        if place_idx < place_nodes.len() && transition_idx < transition_nodes.len() {
            let pp = place_nodes[place_idx].pos;
            let tp = transition_nodes[transition_idx].pos;
            let end_a_ok = near_point(pp, x1 as f32, y1 as f32, 220.0)
                && near_point(tp, x2 as f32, y2 as f32, 220.0);
            let end_b_ok = near_point(pp, x2 as f32, y2 as f32, 220.0)
                && near_point(tp, x1 as f32, y1 as f32, 220.0);
            if end_a_ok || end_b_ok {
                *counts_filtered.entry(dedup_key).or_insert(0) += 1;
            }
        }
    }

    let chosen = if counts_filtered.len() > counts_all.len() {
        counts_filtered
    } else {
        counts_all
    };

    let mut arcs = chosen
        .into_iter()
        .map(
            |((place_idx, transition_idx, place_to_transition, inhibitor), weight)| {
                LegacyArcRecord {
                    place_idx,
                    transition_idx,
                    place_to_transition,
                    weight: weight.max(1),
                    inhibitor,
                }
            },
        )
        .collect::<Vec<_>>();
    arcs.sort_by_key(|arc| {
        (
            arc.place_idx,
            arc.transition_idx,
            arc.place_to_transition,
            arc.inhibitor,
        )
    });

    if arcs.is_empty() {
        None
    } else {
        Some(arcs)
    }
}

fn detect_counts(bytes: &[u8]) -> Vec<(usize, u32, u32)> {
    let mut result = Vec::new();
    let scan_limit = bytes.len().min(4096);
    let mut offset = 0usize;

    while offset + 8 <= scan_limit {
        let p = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let t = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        if (1..=10_000).contains(&p) && (1..=10_000).contains(&t) {
            result.push((offset, p, t));
            if result.len() >= 10 {
                break;
            }
        }
        offset += 4;
    }

    result
}

fn legacy_arc_polyline_points(
    model: &PetriNetModel,
    from: NodeRef,
    to: NodeRef,
) -> Option<([f32; 2], [f32; 2], [f32; 2])> {
    let from_center = legacy_node_center(model, from)?;
    let to_center = legacy_node_center(model, to)?;
    let mut dir = [to_center[0] - from_center[0], to_center[1] - from_center[1]];
    let dir_len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
    if dir_len > f32::EPSILON {
        dir[0] /= dir_len;
        dir[1] /= dir_len;
    } else {
        dir = [1.0, 0.0];
    }

    let from_anchor = legacy_node_anchor(model, from, dir)?;
    let to_anchor = legacy_node_anchor(model, to, [-dir[0], -dir[1]])?;
    let middle = [
        (from_anchor[0] + to_anchor[0]) * 0.5,
        (from_anchor[1] + to_anchor[1]) * 0.5,
    ];
    Some((from_anchor, middle, to_anchor))
}

fn legacy_node_center(model: &PetriNetModel, node: NodeRef) -> Option<[f32; 2]> {
    match node {
        NodeRef::Place(id) => model
            .places
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.pos),
        NodeRef::Transition(id) => {
            model
                .transitions
                .iter()
                .find(|item| item.id == id)
                .map(|item| {
                    let (w, h) = legacy_transition_dims(item.size);
                    [item.pos[0] + w * 0.5, item.pos[1] + h * 0.5]
                })
        }
    }
}

fn legacy_node_anchor(model: &PetriNetModel, node: NodeRef, dir: [f32; 2]) -> Option<[f32; 2]> {
    match node {
        NodeRef::Place(id) => model.places.iter().find(|item| item.id == id).map(|item| {
            let r = legacy_place_radius(item.size);
            [item.pos[0] + dir[0] * r, item.pos[1] + dir[1] * r]
        }),
        NodeRef::Transition(id) => {
            model
                .transitions
                .iter()
                .find(|item| item.id == id)
                .map(|item| {
                    let (w, h) = legacy_transition_dims(item.size);
                    let center = [item.pos[0] + w * 0.5, item.pos[1] + h * 0.5];
                    let half_w = w * 0.5;
                    let half_h = h * 0.5;
                    let tx = if dir[0].abs() > f32::EPSILON {
                        half_w / dir[0].abs()
                    } else {
                        f32::INFINITY
                    };
                    let ty = if dir[1].abs() > f32::EPSILON {
                        half_h / dir[1].abs()
                    } else {
                        f32::INFINITY
                    };
                    let t = tx.min(ty);
                    if t.is_finite() {
                        [center[0] + dir[0] * t, center[1] + dir[1] * t]
                    } else {
                        center
                    }
                })
        }
    }
}

fn legacy_place_radius(size: VisualSize) -> f32 {
    match size {
        VisualSize::Small => 14.0,
        VisualSize::Medium => 20.0,
        VisualSize::Large => 28.0,
    }
}

fn legacy_transition_dims(size: VisualSize) -> (f32, f32) {
    match size {
        VisualSize::Small => (10.0, 18.0),
        VisualSize::Medium => (12.0, 28.0),
        VisualSize::Large => (16.0, 38.0),
    }
}

fn map_color_to_legacy(color: NodeColor) -> i32 {
    match color {
        NodeColor::Blue => 0x000000FF,
        NodeColor::Green => 0x0000FF00,
        NodeColor::Red => 0x00FF0000,
        NodeColor::Yellow => 0x00FF0100,
        NodeColor::Default => 0,
    }
}

fn legacy_footer_template() -> &'static [u8] {
    &[
        // NetStar simulation defaults: time limit = 1000, pass limit = 1000.
        0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xE8, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x63, 0x29, 0x20, 0x4D, 0x69, 0x6B, 0x68, 0x61,
        0x79, 0x6C, 0x69, 0x73, 0x68, 0x69, 0x6E,
    ]
}

fn read_legacy_name(
    bytes: &[u8],
    record_offset: usize,
    record_size: usize,
    field_offset: usize,
) -> String {
    if field_offset + 1 >= record_size {
        return String::new();
    }
    let len_off = record_offset.saturating_add(field_offset);
    if len_off >= bytes.len() {
        return String::new();
    }

    let len = bytes[len_off] as usize;
    if len == 0 {
        return String::new();
    }

    let value_off = len_off.saturating_add(1);
    let record_end = record_offset.saturating_add(record_size).min(bytes.len());
    if value_off >= record_end {
        return String::new();
    }
    let max_len = record_end.saturating_sub(value_off);
    let len = len.min(max_len);
    let raw = &bytes[value_off..value_off + len];
    decode_legacy_cp1251(raw)
        .trim_matches(|ch: char| ch.is_whitespace() || ch.is_control())
        .to_string()
}

fn decode_legacy_cp1251(raw: &[u8]) -> String {
    raw.iter()
        .copied()
        .map(|b| match b {
            0x00..=0x7F => b as char,
            0xA8 => '\u{0401}',
            0xB8 => '\u{0451}',
            0xC0..=0xFF => {
                let code = 0x0410 + (b - 0xC0) as u32;
                char::from_u32(code).unwrap_or('\u{FFFD}')
            }
            _ => '\u{FFFD}',
        })
        .collect()
}

fn encode_legacy_cp1251(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let b = match ch {
            '\u{0000}'..='\u{007F}' => ch as u8,
            '\u{0401}' => 0xA8, // Ё
            '\u{0451}' => 0xB8, // С‘
            '\u{0410}'..='\u{044F}' => (0xC0u32 + (ch as u32 - 0x0410)) as u8, // А..я
            _ => b'?',          // unsupported in our legacy subset
        };
        out.push(b);
    }
    out
}

fn write_legacy_name_limited(
    record: &mut [u8],
    field_offset: usize,
    value: &str,
    hard_max_len: usize,
) {
    if field_offset + 1 >= record.len() {
        return;
    }
    let trimmed = value.trim();
    if trimmed.is_empty() {
        record[field_offset] = 0;
        return;
    }

    let encoded = encode_legacy_cp1251(trimmed);
    let max_len = record
        .len()
        .saturating_sub(field_offset + 1)
        .min(hard_max_len);
    let len = encoded.len().min(max_len).min(255);
    record[field_offset] = len as u8;
    record[field_offset + 1..field_offset + 1 + len].copy_from_slice(&encoded[..len]);
}

fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    if offset + 4 > bytes.len() {
        return None;
    }
    Some(i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

fn read_f64(bytes: &[u8], offset: usize) -> Option<f64> {
    if offset + 8 > bytes.len() {
        return None;
    }
    Some(f64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ]))
}

fn write_i32(target: &mut [u8], offset: usize, value: i32) {
    if offset + 4 <= target.len() {
        target[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

fn write_u16(target: &mut [u8], offset: usize, value: u16) {
    if offset + 2 <= target.len() {
        target[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn write_f64(target: &mut [u8], offset: usize, value: f64) {
    if offset + 8 <= target.len() {
        target[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
}

fn push_i32(target: &mut Vec<u8>, value: i32) {
    target.extend_from_slice(&value.to_le_bytes());
}

fn round_i32(value: f32) -> i32 {
    if value.is_finite() {
        value.round().clamp(-2_000_000_000.0, 2_000_000_000.0) as i32
    } else {
        0
    }
}

fn clamp_u16(value: f32) -> u16 {
    if value.is_finite() {
        value.round().clamp(0.0, u16::MAX as f32) as u16
    } else {
        0
    }
}

fn map_legacy_color(raw: i32) -> NodeColor {
    let value = raw as u32;
    match value {
        0x000000FF => NodeColor::Blue,
        0x0000FF00 => NodeColor::Green,
        0x00FF0000 => NodeColor::Red,
        0x00FFFF00 | 0x00FF0100 => NodeColor::Yellow,
        _ => NodeColor::Default,
    }
}

fn near_point(center: [f32; 2], x: f32, y: f32, max_dist: f32) -> bool {
    let dx = center[0] - x;
    let dy = center[1] - y;
    dx * dx + dy * dy <= max_dist * max_dist
}

pub fn extract_ascii_strings(bytes: &[u8], min_len: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut start = None;

    for (i, b) in bytes.iter().copied().enumerate() {
        if b.is_ascii_graphic() || b == b' ' {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            if i - s >= min_len {
                out.push((s, String::from_utf8_lossy(&bytes[s..i]).to_string()));
            }
        }
    }

    if let Some(s) = start {
        if bytes.len() - s >= min_len {
            out.push((s, String::from_utf8_lossy(&bytes[s..]).to_string()));
        }
    }

    out
}

pub fn extract_utf16le_strings(bytes: &[u8], min_len: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;

    while i + 2 <= bytes.len() {
        let start = i;
        let mut data = Vec::new();

        while i + 2 <= bytes.len() {
            let lo = bytes[i];
            let hi = bytes[i + 1];
            if hi == 0 && (lo.is_ascii_graphic() || lo == b' ') {
                data.push(lo as u16);
                i += 2;
            } else {
                break;
            }
        }

        if data.len() >= min_len {
            if let Ok(s) = String::from_utf16(&data) {
                out.push((start, s));
            }
        }

        i = if i == start { i + 1 } else { i + 2 };
    }

    out
}

pub fn extract_float64_pairs(bytes: &[u8], max_items: usize) -> Vec<(usize, f64, f64)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 16 <= bytes.len() && out.len() < max_items {
        let a = f64::from_le_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        let b = f64::from_le_bytes([
            bytes[i + 8],
            bytes[i + 9],
            bytes[i + 10],
            bytes[i + 11],
            bytes[i + 12],
            bytes[i + 13],
            bytes[i + 14],
            bytes[i + 15],
        ]);
        if a.is_finite() && b.is_finite() && a.abs() <= 1.0e8 && b.abs() <= 1.0e8 {
            out.push((i, a, b));
        }
        i += 8;
    }
    out
}

fn detect_section_boundaries(bytes: &[u8]) -> Vec<String> {
    let mut sections = Vec::new();
    for (off, p, t) in detect_counts(bytes).into_iter().take(5) {
        sections.push(format!(
            "Кандидат секции counts @0x{off:08X}: places={p}, transitions={t}"
        ));
    }
    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn export_place_name_does_not_corrupt_delay_field() {
        let mut model = PetriNetModel::new();
        model.set_counts(1, 1);
        model.places[0].name =
            "THIS_IS_A_VERY_LONG_PLACE_NAME_THAT_USED_TO_OVERWRITE_DELAY_FIELD_IN_LEGACY_RECORD"
                .to_string();
        model.tables.mz[0] = 12.5;

        let tmp = NamedTempFile::new().expect("temp file");
        export_legacy_gpn(tmp.path(), &model).expect("legacy export");
        let imported = import_legacy_gpn(tmp.path()).expect("legacy import");
        let delay = imported
            .model
            .tables
            .mz
            .first()
            .copied()
            .unwrap_or_default();
        assert!(
            (delay - 12.5).abs() < 1e-9,
            "delay mismatch after roundtrip: {delay}"
        );
    }
}


# src\io\mod.rs
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::model::{PetriNetModel, GPN2_MAGIC};

pub mod gpn2;
pub mod legacy_gpn;

pub use legacy_gpn::{LegacyDebugInfo, LegacyExportHints, LegacyImportError, LegacyImportResult};

#[derive(Debug, Clone)]
pub struct LoadGpnResult {
    pub model: PetriNetModel,
    pub warnings: Vec<String>,
    pub legacy_debug: Option<LegacyDebugInfo>,
}

pub fn load_gpn(path: &Path) -> Result<LoadGpnResult> {
    let bytes =
        fs::read(path).with_context(|| format!("Не удалось прочитать файл {}", path.display()))?;

    if bytes.starts_with(GPN2_MAGIC.as_bytes()) {
        let model = gpn2::load_gpn2_from_bytes(&bytes)?;
        Ok(LoadGpnResult {
            model,
            warnings: Vec::new(),
            legacy_debug: None,
        })
    } else {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if value.is_object() {
                if let Ok(model) = serde_json::from_value::<PetriNetModel>(value.clone()) {
                    model.validate()?;
                    return Ok(LoadGpnResult {
                        model,
                        warnings: vec!["Файл JSON открыт без заголовка GPN2".to_string()],
                        legacy_debug: None,
                    });
                }
            }
        }

        let legacy = legacy_gpn::import_legacy_gpn(path)?;
        Ok(LoadGpnResult {
            model: legacy.model,
            warnings: legacy.warnings,
            legacy_debug: Some(legacy.debug),
        })
    }
}

pub fn save_gpn(path: &Path, model: &PetriNetModel) -> Result<()> {
    save_gpn_with_hints(path, model, None)
}

pub fn save_gpn_with_hints(
    path: &Path,
    model: &PetriNetModel,
    legacy_hints: Option<&LegacyExportHints>,
) -> Result<()> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("gpn2"))
        .unwrap_or(false)
    {
        gpn2::save_gpn2(path, model)
    } else {
        legacy_gpn::export_legacy_gpn_with_hints(path, model, legacy_hints)
            .with_context(|| format!("Не удалось сохранить legacy GPN в {}", path.display()))
    }
}


# src\lib.rs
pub mod formats;
pub mod io;
pub mod markov;
pub mod model;
pub mod sim;
pub mod ui;


# src\main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use image::ImageFormat;
use petri_net_legacy_editor::ui::app::PetriApp;

fn app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/petrinet.ico");
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Ico).ok()?;
    let rgba = image.to_rgba8();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width: image.width(),
        height: image.height(),
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]);
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "PetriNet",
        native_options,
        Box::new(|cc| Ok(Box::new(PetriApp::new(cc)))),
    )
}


# src\markov.rs
use std::collections::{HashMap, VecDeque};

use crate::model::PetriNet;

const DEFAULT_MAX_STATES: usize = 500;

/// Результат распределения цепи Маркова и её графа состояний.
pub struct MarkovChain {
    pub states: Vec<Vec<u32>>,
    pub transitions: Vec<Vec<(usize, f64)>>,
    pub stationary: Option<Vec<f64>>,
    pub limit_reached: bool,
}

impl MarkovChain {
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
}

/// Построить граф состояний и решить уравнение Кольмогорова для стационарного распределения.
/// Колмогоровы уравнения описывают эволюцию вероятностей дискретных цепей Маркова [Kolmogorov equations].
pub fn build_markov_chain(net: &PetriNet, max_states: Option<usize>) -> MarkovChain {
    let limit = max_states.unwrap_or(DEFAULT_MAX_STATES);
    let initial_marking = net.tables.m0.clone();
    let mut states = Vec::new();
    let mut transitions = Vec::new();
    let mut seen = HashMap::new();
    let mut queue = VecDeque::new();
    states.push(initial_marking.clone());
    transitions.push(Vec::new());
    seen.insert(initial_marking.clone(), 0);
    queue.push_back(0);

    let mut limit_reached = false;
    while let Some(idx) = queue.pop_front() {
        if states.len() >= limit {
            limit_reached = true;
            break;
        }
        let marking = states[idx].clone();
        let enabled = enabled_transitions_from_marking(net, &marking);
        let mut edges = Vec::new();
        for &t in &enabled {
            if let Some(next_marking) = apply_transition(net, &marking, t) {
                let state_id = if let Some(&id) = seen.get(&next_marking) {
                    id
                } else {
                    let id = states.len();
                    if id >= limit {
                        limit_reached = true;
                        break;
                    }
                    states.push(next_marking.clone());
                    transitions.push(Vec::new());
                    seen.insert(next_marking.clone(), id);
                    queue.push_back(id);
                    id
                };
                edges.push((state_id, 1.0));
            }
        }
        transitions[idx] = edges;
        if limit_reached {
            break;
        }
    }

    let generator = build_generator_matrix(&transitions);
    let stationary = compute_stationary(&generator);
    MarkovChain {
        states,
        transitions,
        stationary,
        limit_reached,
    }
}

fn enabled_transitions_from_marking(net: &PetriNet, marking: &[u32]) -> Vec<usize> {
    let places = net.places.len();
    let mut enabled = Vec::new();
    for t in 0..net.transitions.len() {
        let mut has_arc = false;
        for p in 0..places {
            if net.tables.pre[p][t] > 0
                || net.tables.post[p][t] > 0
                || net.tables.inhibitor[p][t] > 0
            {
                has_arc = true;
                break;
            }
        }
        if !has_arc {
            continue;
        }
        let mut ok = true;
        for p in 0..places {
            let need = net.tables.pre[p][t];
            if marking[p] < need {
                ok = false;
                break;
            }
            let inh = net.tables.inhibitor[p][t];
            if inh > 0 && marking[p] >= inh {
                ok = false;
                break;
            }
            if let Some(cap) = net.tables.mo[p] {
                let after = marking[p]
                    .saturating_sub(need)
                    .saturating_add(net.tables.post[p][t]);
                if after > cap {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            enabled.push(t);
        }
    }
    enabled
}

fn apply_transition(net: &PetriNet, marking: &[u32], t: usize) -> Option<Vec<u32>> {
    let mut next = marking.to_vec();
    for p in 0..net.places.len() {
        next[p] = next[p].saturating_sub(net.tables.pre[p][t]);
    }
    for p in 0..net.places.len() {
        next[p] = next[p].saturating_add(net.tables.post[p][t]);
    }
    Some(next)
}

fn build_generator_matrix(transitions: &[Vec<(usize, f64)>]) -> Vec<Vec<f64>> {
    let n = transitions.len();
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        let mut sum = 0.0;
        for &(dest, rate) in &transitions[i] {
            matrix[i][dest] += rate;
            sum += rate;
        }
        matrix[i][i] = -sum;
    }
    matrix
}

fn compute_stationary(generator: &[Vec<f64>]) -> Option<Vec<f64>> {
    let n = generator.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = generator[j][i];
        }
    }
    let mut rhs = vec![0.0; n];
    for row in 0..n - 1 {
        rhs[row] = 0.0;
    }
    for col in 0..n {
        matrix[n - 1][col] = 1.0;
    }
    rhs[n - 1] = 1.0;
    gaussian_elimination(&mut matrix, &mut rhs)
        .map(|mut solution| {
            let sum: f64 = solution.iter().sum();
            if sum > 0.0 {
                for v in solution.iter_mut() {
                    *v = (*v).max(0.0) / sum;
                }
            }
            solution
        })
        .or_else(|| uniform_stationary(n))
}

fn uniform_stationary(n: usize) -> Option<Vec<f64>> {
    if n == 0 {
        Some(Vec::new())
    } else {
        Some(vec![1.0 / (n as f64); n])
    }
}

fn gaussian_elimination(matrix: &mut [Vec<f64>], rhs: &mut [f64]) -> Option<Vec<f64>> {
    let n = matrix.len();
    for i in 0..n {
        let mut pivot = i;
        for row in (i + 1)..n {
            if matrix[row][i].abs() > matrix[pivot][i].abs() {
                pivot = row;
            }
        }
        if matrix[pivot][i].abs() < 1e-12 {
            return None;
        }
        if pivot != i {
            matrix.swap(pivot, i);
            rhs.swap(pivot, i);
        }
        let diag = matrix[i][i];
        for col in i..n {
            matrix[i][col] /= diag;
        }
        rhs[i] /= diag;
        for row in 0..n {
            if row == i {
                continue;
            }
            let factor = matrix[row][i];
            for col in i..n {
                matrix[row][col] -= factor * matrix[i][col];
            }
            rhs[row] -= factor * rhs[i];
        }
    }
    Some(rhs.to_vec())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PetriNet;

    #[test]
    fn chain_enumerates_states() {
        let mut net = PetriNet::new();
        net.set_counts(2, 1);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[1][0] = 1;
        net.tables.mo[1] = Some(2);
        let chain = build_markov_chain(&net, Some(20));

        assert!(chain.state_count() >= 2);
        assert!(chain.transitions.iter().any(|edges| !edges.is_empty()));
        assert!(chain
            .stationary
            .as_ref()
            .map_or(false, |v| (v.iter().sum::<f64>() - 1.0).abs() < 1e-6));
    }

    #[test]
    fn stationary_solver_handles_linear_system() {
        let generator = vec![vec![-0.5, 0.5], vec![0.25, -0.25]];
        let stationary = compute_stationary(&generator).expect("stationary computed");
        assert!((stationary.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!(stationary.iter().all(|v| *v >= 0.0));
    }
}


# src\model.rs
use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const GPN2_MAGIC: &str = "GPN2\n";
pub const GPN2_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Ru,
    En,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tool {
    Place,
    Transition,
    Arc,
    Text,
    Frame,
    Edit,
    Delete,
    Run,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum VisualSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LabelPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum NodeColor {
    #[default]
    Default,
    Blue,
    Red,
    Green,
    Yellow,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarkovPlacement {
    Bottom,
    Top,
}

impl Default for MarkovPlacement {
    fn default() -> Self {
        Self::Bottom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaInfo {
    pub name: String,
    pub author: String,
    pub description: String,
}

impl Default for MetaInfo {
    fn default() -> Self {
        Self {
            name: "Без названия".to_string(),
            author: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiTextBlock {
    pub id: u64,
    pub pos: [f32; 2],
    pub text: String,
    pub font_name: String,
    pub font_size: f32,
    pub color: NodeColor,
}

impl Default for UiTextBlock {
    fn default() -> Self {
        Self {
            id: 0,
            pos: [0.0, 0.0],
            text: String::new(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiDecorativeFrame {
    pub id: u64,
    pub pos: [f32; 2],
    pub width: f32,
    pub height: f32,
}

impl Default for UiDecorativeFrame {
    fn default() -> Self {
        Self {
            id: 0,
            pos: [0.0, 0.0],
            width: 120.0,
            height: 120.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiSettings {
    pub language: Language,
    pub hide_grid: bool,
    pub snap_to_grid: bool,
    pub colored_petri_nets: bool,
    pub fix_time_step: bool,
    pub marker_count_stats: bool,
    pub light_theme: bool,
    pub text_blocks: Vec<UiTextBlock>,
    pub decorative_frames: Vec<UiDecorativeFrame>,
    pub next_text_id: u64,
    pub next_frame_id: u64,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            hide_grid: false,
            snap_to_grid: true,
            colored_petri_nets: false,
            fix_time_step: true,
            marker_count_stats: true,
            light_theme: true,
            text_blocks: Vec::new(),
            decorative_frames: Vec::new(),
            next_text_id: 1,
            next_frame_id: 1,
        }
    }
}

fn default_visible_true() -> bool {
    true
}

fn default_inhibitor_color() -> NodeColor {
    NodeColor::Red
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Place {
    pub id: u64,
    pub name: String,
    pub pos: [f32; 2],
    pub note: String,
    pub color: NodeColor,
    pub marker_label_position: LabelPosition,
    pub text_position: LabelPosition,
    pub size: VisualSize,
    pub marker_color_on_pass: bool,
    pub input_module: bool,
    pub input_number: u32,
    pub input_description: String,
    pub stochastic: StochasticDistribution,
    pub stats: PlaceStatisticsSelection,
    pub markov_highlight: bool,
    pub markov_placement: MarkovPlacement,
    pub show_markov_model: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct PlaceStatisticsSelection {
    pub markers_total: bool,
    pub markers_input: bool,
    pub markers_output: bool,
    pub load_total: bool,
    pub load_input: bool,
    pub load_output: bool,
}

impl PlaceStatisticsSelection {
    pub fn any_enabled(&self) -> bool {
        self.markers_total
            || self.markers_input
            || self.markers_output
            || self.load_total
            || self.load_input
            || self.load_output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum StochasticDistribution {
    #[default]
    None,
    Uniform {
        min: f64,
        max: f64,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Exponential {
        lambda: f64,
    },
    Gamma {
        shape: f64,
        scale: f64,
    },
    Poisson {
        lambda: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Transition {
    pub id: u64,
    pub name: String,
    pub pos: [f32; 2],
    pub note: String,
    pub color: NodeColor,
    pub label_position: LabelPosition,
    pub text_position: LabelPosition,
    pub size: VisualSize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "id")]
pub enum NodeRef {
    Place(u64),
    Transition(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Arc {
    pub id: u64,
    pub from: NodeRef,
    pub to: NodeRef,
    pub weight: u32,
    pub color: NodeColor,
    #[serde(default = "default_visible_true")]
    pub visible: bool,
    #[serde(default)]
    pub show_weight: bool,
}

impl Default for Arc {
    fn default() -> Self {
        Self {
            id: 0,
            from: NodeRef::Place(0),
            to: NodeRef::Transition(0),
            weight: 1,
            color: NodeColor::Default,
            visible: true,
            show_weight: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct InhibitorArc {
    pub id: u64,
    pub place_id: u64,
    pub transition_id: u64,
    pub threshold: u32,
    pub show_weight: bool,
    #[serde(default = "default_inhibitor_color")]
    pub color: NodeColor,
    #[serde(default = "default_visible_true")]
    pub visible: bool,
}

impl Default for InhibitorArc {
    fn default() -> Self {
        Self {
            id: 0,
            place_id: 0,
            transition_id: 0,
            threshold: 1,
            color: NodeColor::Red,
            visible: true,
            show_weight: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Tables {
    pub m0: Vec<u32>,
    pub mo: Vec<Option<u32>>,
    pub mz: Vec<f64>,
    pub mpr: Vec<i32>,
    pub pre: Vec<Vec<u32>>,
    pub post: Vec<Vec<u32>>,
    pub inhibitor: Vec<Vec<u32>>,
}

impl Tables {
    pub fn resize(&mut self, places: usize, transitions: usize) {
        self.m0.resize(places, 0);
        // Default place capacity is 1 (Mo=1). Use None only when explicitly set to unlimited.
        self.mo.resize_with(places, || Some(1));
        self.mz.resize(places, 0.0);
        self.mpr.resize(transitions, 0);

        self.pre.resize_with(places, || vec![0; transitions]);
        self.post.resize_with(places, || vec![0; transitions]);
        self.inhibitor.resize_with(places, || vec![0; transitions]);

        for row in &mut self.pre {
            row.resize(transitions, 0);
        }
        for row in &mut self.post {
            row.resize(transitions, 0);
        }
        for row in &mut self.inhibitor {
            row.resize(transitions, 0);
        }
    }

    pub(crate) fn remove_place_row(&mut self, idx: usize) {
        if idx < self.m0.len() {
            self.m0.remove(idx);
        }
        if idx < self.mo.len() {
            self.mo.remove(idx);
        }
        if idx < self.mz.len() {
            self.mz.remove(idx);
        }
        if idx < self.pre.len() {
            self.pre.remove(idx);
        }
        if idx < self.post.len() {
            self.post.remove(idx);
        }
        if idx < self.inhibitor.len() {
            self.inhibitor.remove(idx);
        }
    }

    pub(crate) fn remove_transition_column(&mut self, idx: usize) {
        if idx < self.mpr.len() {
            self.mpr.remove(idx);
        }
        for row in &mut self.pre {
            if idx < row.len() {
                row.remove(idx);
            }
        }
        for row in &mut self.post {
            if idx < row.len() {
                row.remove(idx);
            }
        }
        for row in &mut self.inhibitor {
            if idx < row.len() {
                row.remove(idx);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PetriNetModel {
    pub format_version: u32,
    pub meta: MetaInfo,
    pub places: Vec<Place>,
    pub transitions: Vec<Transition>,
    pub arcs: Vec<Arc>,
    pub inhibitor_arcs: Vec<InhibitorArc>,
    pub tables: Tables,
    pub ui: UiSettings,
}

pub type PetriNet = PetriNetModel;

impl Default for PetriNetModel {
    fn default() -> Self {
        Self::new()
    }
}

impl PetriNetModel {
    fn is_auto_name(name: &str, prefixes: &[char]) -> bool {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !prefixes.contains(&first) {
            return false;
        }
        let digits: String = chars.collect();
        !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
    }

    pub fn new() -> Self {
        Self {
            format_version: GPN2_FORMAT_VERSION,
            meta: MetaInfo::default(),
            places: Vec::new(),
            transitions: Vec::new(),
            arcs: Vec::new(),
            inhibitor_arcs: Vec::new(),
            tables: Tables::default(),
            ui: UiSettings::default(),
        }
    }

    fn next_place_id(&self) -> u64 {
        self.places.iter().map(|p| p.id).max().unwrap_or(0) + 1
    }

    fn next_transition_id(&self) -> u64 {
        self.transitions.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    fn next_arc_id(&self) -> u64 {
        let max_arc = self.arcs.iter().map(|a| a.id).max().unwrap_or(0);
        let max_inh = self.inhibitor_arcs.iter().map(|a| a.id).max().unwrap_or(0);
        max_arc.max(max_inh) + 1
    }

    fn next_inhibitor_id(&self) -> u64 {
        let max_arc = self.arcs.iter().map(|a| a.id).max().unwrap_or(0);
        let max_inh = self.inhibitor_arcs.iter().map(|a| a.id).max().unwrap_or(0);
        max_arc.max(max_inh) + 1
    }

    pub fn normalize_arc_ids(&mut self) {
        let mut next_id = 1_u64;
        for arc in &mut self.arcs {
            arc.id = next_id;
            next_id = next_id.saturating_add(1);
        }
        for arc in &mut self.inhibitor_arcs {
            arc.id = next_id;
            next_id = next_id.saturating_add(1);
        }
    }

    fn default_place_pos(index: usize) -> [f32; 2] {
        let col = (index % 8) as f32;
        let row = (index / 8) as f32;
        [40.0 + col * 140.0, 40.0 + row * 140.0]
    }

    fn default_transition_pos(index: usize) -> [f32; 2] {
        let col = (index % 8) as f32;
        let row = (index / 8) as f32;
        [110.0 + col * 140.0, 40.0 + row * 140.0]
    }

    pub fn set_counts(&mut self, places: usize, transitions: usize) {
        let old_places = self.places.len();
        if places >= old_places {
            for index in old_places..places {
                self.places.push(Place {
                    id: 0,
                    name: String::new(),
                    pos: Self::default_place_pos(index),
                    ..Default::default()
                });
            }
        } else {
            self.places.truncate(places);
        }

        let old_transitions = self.transitions.len();
        if transitions >= old_transitions {
            for index in old_transitions..transitions {
                self.transitions.push(Transition {
                    id: 0,
                    name: String::new(),
                    pos: Self::default_transition_pos(index),
                    size: VisualSize::Medium,
                    ..Default::default()
                });
            }
        } else {
            self.transitions.truncate(transitions);
        }

        let mut used_place_ids = HashSet::new();
        for (i, place) in self.places.iter_mut().enumerate() {
            if place.id == 0 || !used_place_ids.insert(place.id) {
                place.id = (i + 1) as u64;
                used_place_ids.insert(place.id);
            }
        }
        for place in &mut self.places {
            if place.name.is_empty() {
                place.name = format!("P{}", place.id);
            }
        }

        let mut used_transition_ids = HashSet::new();
        for (i, tr) in self.transitions.iter_mut().enumerate() {
            if tr.id == 0 || !used_transition_ids.insert(tr.id) {
                tr.id = (i + 1) as u64;
                used_transition_ids.insert(tr.id);
            }
        }
        let mut sorted_transition_ids: Vec<u64> = self.transitions.iter().map(|t| t.id).collect();
        sorted_transition_ids.sort_unstable();
        let transition_rank: HashMap<u64, usize> = sorted_transition_ids
            .into_iter()
            .enumerate()
            .map(|(idx, id)| (id, idx + 1))
            .collect();
        for tr in &mut self.transitions {
            if tr.name.is_empty() || Self::is_auto_name(&tr.name, &['T', 't']) {
                if let Some(rank) = transition_rank.get(&tr.id) {
                    tr.name = format!("T{}", rank);
                }
            }
        }

        self.tables.resize(places, transitions);
        self.rebuild_matrices_from_arcs();
    }

    pub fn add_place(&mut self, pos: [f32; 2]) {
        let id = self.next_place_id();
        let idx = self.places.len();
        self.places.push(Place {
            id,
            name: format!("P{}", idx + 1),
            pos,
            ..Default::default()
        });
        self.set_counts(self.places.len(), self.transitions.len());
    }

    pub fn add_transition(&mut self, pos: [f32; 2]) {
        let id = self.next_transition_id();
        let idx = self.transitions.len();
        self.transitions.push(Transition {
            id,
            name: format!("T{}", idx + 1),
            pos,
            size: VisualSize::Medium,
            ..Default::default()
        });
        self.set_counts(self.places.len(), self.transitions.len());
    }

    pub fn add_arc(&mut self, from: NodeRef, to: NodeRef, weight: u32) {
        if matches!(
            (from, to),
            (NodeRef::Place(_), NodeRef::Transition(_))
                | (NodeRef::Transition(_), NodeRef::Place(_))
        ) {
            self.arcs.push(Arc {
                id: self.next_arc_id(),
                from,
                to,
                weight: weight.max(1),
                color: NodeColor::Default,
                visible: true,
                show_weight: false,
            });
            self.rebuild_matrices_from_arcs();
        }
    }

    pub fn add_inhibitor_arc(&mut self, place_id: u64, transition_id: u64, threshold: u32) {
        if self.places.iter().any(|p| p.id == place_id)
            && self.transitions.iter().any(|t| t.id == transition_id)
        {
            self.inhibitor_arcs.push(InhibitorArc {
                id: self.next_inhibitor_id(),
                place_id,
                transition_id,
                threshold: threshold.max(1),
                color: NodeColor::Red,
                visible: true,
                show_weight: false,
            });
            self.rebuild_matrices_from_arcs();
        }
    }

    pub fn place_index_map(&self) -> HashMap<u64, usize> {
        self.places
            .iter()
            .enumerate()
            .map(|(idx, p)| (p.id, idx))
            .collect()
    }

    pub fn transition_index_map(&self) -> HashMap<u64, usize> {
        self.transitions
            .iter()
            .enumerate()
            .map(|(idx, t)| (t.id, idx))
            .collect()
    }

    pub fn rebuild_matrices_from_arcs(&mut self) {
        self.tables
            .resize(self.places.len(), self.transitions.len());

        for p in 0..self.places.len() {
            for t in 0..self.transitions.len() {
                self.tables.pre[p][t] = 0;
                self.tables.post[p][t] = 0;
                self.tables.inhibitor[p][t] = 0;
            }
        }

        let pmap = self.place_index_map();
        let tmap = self.transition_index_map();

        self.arcs.retain(|arc| match (arc.from, arc.to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid))
            | (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                pmap.contains_key(&pid) && tmap.contains_key(&tid)
            }
            _ => false,
        });

        for arc in &self.arcs {
            match (arc.from, arc.to) {
                (NodeRef::Place(pid), NodeRef::Transition(tid)) => {
                    if let (Some(&p), Some(&t)) = (pmap.get(&pid), tmap.get(&tid)) {
                        self.tables.pre[p][t] =
                            self.tables.pre[p][t].saturating_add(arc.weight.max(1));
                    }
                }
                (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                    if let (Some(&p), Some(&t)) = (pmap.get(&pid), tmap.get(&tid)) {
                        self.tables.post[p][t] =
                            self.tables.post[p][t].saturating_add(arc.weight.max(1));
                    }
                }
                _ => {}
            }
        }

        self.inhibitor_arcs
            .retain(|a| pmap.contains_key(&a.place_id) && tmap.contains_key(&a.transition_id));
        for inh in &self.inhibitor_arcs {
            if let (Some(&p), Some(&t)) = (pmap.get(&inh.place_id), tmap.get(&inh.transition_id)) {
                self.tables.inhibitor[p][t] = inh.threshold.max(1);
            }
        }
    }

    pub fn rebuild_arcs_from_matrices(&mut self) {
        self.arcs.clear();
        self.inhibitor_arcs.clear();
        let mut next_id = 1_u64;

        let place_ids: Vec<u64> = self.places.iter().map(|p| p.id).collect();
        let transition_ids: Vec<u64> = self.transitions.iter().map(|t| t.id).collect();

        for (pi, place_id) in place_ids.iter().enumerate() {
            for (ti, tr_id) in transition_ids.iter().enumerate() {
                let pre = self.tables.pre[pi][ti];
                let post = self.tables.post[pi][ti];
                let inh = self.tables.inhibitor[pi][ti];

                if pre > 0 {
                    self.arcs.push(Arc {
                        id: next_id,
                        from: NodeRef::Place(*place_id),
                        to: NodeRef::Transition(*tr_id),
                        weight: pre,
                        color: NodeColor::Default,
                        visible: true,
                        show_weight: false,
                    });
                    next_id = next_id.saturating_add(1);
                }
                if post > 0 {
                    self.arcs.push(Arc {
                        id: next_id,
                        from: NodeRef::Transition(*tr_id),
                        to: NodeRef::Place(*place_id),
                        weight: post,
                        color: NodeColor::Default,
                        visible: true,
                        show_weight: false,
                    });
                    next_id = next_id.saturating_add(1);
                }
                if inh > 0 {
                    self.inhibitor_arcs.push(InhibitorArc {
                        id: next_id,
                        place_id: *place_id,
                        transition_id: *tr_id,
                        threshold: inh.max(1),
                        color: NodeColor::Red,
                        visible: true,
                        show_weight: false,
                    });
                    next_id = next_id.saturating_add(1);
                }
            }
        }
    }

    pub fn sanitize_values(&mut self) {
        for value in &mut self.tables.mz {
            if !value.is_finite() || *value < 0.0 {
                *value = 0.0;
            }
        }
        for cap in &mut self.tables.mo {
            if let Some(inner) = cap {
                if *inner == 0 {
                    *cap = None;
                }
            }
        }
        for arc in &mut self.arcs {
            arc.weight = arc.weight.max(1);
        }
        for inh in &mut self.inhibitor_arcs {
            inh.threshold = inh.threshold.max(1);
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.format_version != GPN2_FORMAT_VERSION {
            return Err(anyhow!(
                "Неподдерживаемая версия формата: {}",
                self.format_version
            ));
        }

        let mut place_ids = HashSet::new();
        for place in &self.places {
            if !place_ids.insert(place.id) {
                return Err(anyhow!("Дублирующийся id места: {}", place.id));
            }
            if !place.pos[0].is_finite() || !place.pos[1].is_finite() {
                return Err(anyhow!("Координаты места {} невалидны", place.id));
            }
        }

        let mut transition_ids = HashSet::new();
        for tr in &self.transitions {
            if !transition_ids.insert(tr.id) {
                return Err(anyhow!("Дублирующийся id перехода: {}", tr.id));
            }
            if !tr.pos[0].is_finite() || !tr.pos[1].is_finite() {
                return Err(anyhow!("Координаты перехода {} невалидны", tr.id));
            }
        }

        for (row_name, matrix) in [
            ("pre", &self.tables.pre),
            ("post", &self.tables.post),
            ("inhibitor", &self.tables.inhibitor),
        ] {
            if matrix.len() != self.places.len() {
                return Err(anyhow!(
                    "Матрица {} имеет некорректное число строк: {} вместо {}",
                    row_name,
                    matrix.len(),
                    self.places.len()
                ));
            }
            for row in matrix {
                if row.len() != self.transitions.len() {
                    return Err(anyhow!(
                        "Матрица {} имеет некорректное число столбцов",
                        row_name
                    ));
                }
            }
        }

        if self.tables.m0.len() != self.places.len()
            || self.tables.mo.len() != self.places.len()
            || self.tables.mz.len() != self.places.len()
            || self.tables.mpr.len() != self.transitions.len()
        {
            return Err(anyhow!(
                "Размеры таблиц не согласованы с числами мест/переходов"
            ));
        }

        for (idx, v) in self.tables.mz.iter().enumerate() {
            if !v.is_finite() || *v < 0.0 {
                return Err(anyhow!("Mz[{}] содержит недопустимое значение", idx));
            }
        }

        for arc in &self.arcs {
            if arc.weight == 0 {
                return Err(anyhow!("Вес дуги {} должен быть > 0", arc.id));
            }
            match (arc.from, arc.to) {
                (NodeRef::Place(p), NodeRef::Transition(t)) => {
                    if !place_ids.contains(&p) || !transition_ids.contains(&t) {
                        return Err(anyhow!(
                            "Дуга {} ссылается на отсутствующие вершины",
                            arc.id
                        ));
                    }
                }
                (NodeRef::Transition(t), NodeRef::Place(p)) => {
                    if !place_ids.contains(&p) || !transition_ids.contains(&t) {
                        return Err(anyhow!(
                            "Дуга {} ссылается на отсутствующие вершины",
                            arc.id
                        ));
                    }
                }
                _ => return Err(anyhow!("Дуга {} нарушает двудольность графа", arc.id)),
            }
        }

        for inh in &self.inhibitor_arcs {
            if inh.threshold == 0 {
                return Err(anyhow!(
                    "Порог ингибиторной дуги {} должен быть > 0",
                    inh.id
                ));
            }
            if !place_ids.contains(&inh.place_id) || !transition_ids.contains(&inh.transition_id) {
                return Err(anyhow!(
                    "Ингибиторная дуга {} ссылается на отсутствующие вершины",
                    inh.id
                ));
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_values_resets_invalid_inputs() {
        let mut net = PetriNet::new();
        net.set_counts(1, 1);
        net.tables.mz[0] = f64::NAN;
        net.tables.mo[0] = Some(0);
        let place_id = net.places[0].id;
        let transition_id = net.transitions[0].id;
        net.arcs.push(Arc {
            id: 1,
            from: NodeRef::Place(place_id),
            to: NodeRef::Transition(transition_id),
            weight: 0,
            color: NodeColor::Default,
            visible: true,
            show_weight: false,
        });
        net.inhibitor_arcs.push(InhibitorArc {
            id: 2,
            place_id,
            transition_id,
            threshold: 0,
            color: NodeColor::Red,
            visible: true,
            show_weight: false,
        });

        net.sanitize_values();

        assert_eq!(net.tables.mz[0], 0.0);
        assert_eq!(net.tables.mo[0], None);
        assert_eq!(net.arcs[0].weight, 1);
        assert_eq!(net.inhibitor_arcs[0].threshold, 1);
    }
}


# src\sim\engine.rs
use std::collections::HashMap;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Gamma};
use serde::{Deserialize, Serialize};

use crate::model::{PetriNet, StochasticDistribution};

const MAX_SIM_LOG_ENTRIES: usize = 20_000;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StopConditions {
    pub through_place: Option<(usize, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    pub use_pass_limit: bool,
    pub pass_limit: u64,
    pub use_time_limit: bool,
    pub time_limit: f64,
    pub dt: f64,
    pub stop: StopConditions,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            use_pass_limit: false,
            pass_limit: 1000,
            use_time_limit: false,
            time_limit: 60.0,
            dt: 0.1,
            stop: StopConditions::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: f64,
    pub fired_transition: Option<usize>,
    pub marking: Vec<u32>,
    pub touched_places: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceStats {
    pub min: u32,
    pub max: u32,
    pub avg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceFlowStats {
    pub in_tokens: u64,
    pub out_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceLoadStats {
    pub avg_over_capacity: Option<f64>,
    pub in_rate: Option<f64>,
    pub out_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub cycle_time: Option<f64>,
    pub logs: Vec<LogEntry>,
    pub log_entries_total: usize,
    pub log_sampling_stride: usize,
    pub place_stats: Option<Vec<PlaceStats>>,
    pub place_flow: Option<Vec<PlaceFlowStats>>,
    pub place_load: Option<Vec<PlaceLoadStats>>,
    pub sim_time: f64,
    pub fired_count: u64,
    pub final_marking: Vec<u32>,
}

#[derive(Debug, Clone)]
struct SimState {
    available: Vec<u32>,
    pending_release: Vec<Vec<f64>>,
    through_place_counter: Vec<u64>,
    in_tokens: Vec<u64>,
    out_tokens: Vec<u64>,
}

impl SimState {
    fn total_marking(&self) -> Vec<u32> {
        self.available
            .iter()
            .enumerate()
            .map(|(p, a)| *a + self.pending_release[p].len() as u32)
            .collect()
    }

    fn process_releases(&mut self, now: f64) {
        for p in 0..self.pending_release.len() {
            let mut still_pending = Vec::with_capacity(self.pending_release[p].len());
            for release_time in self.pending_release[p].drain(..) {
                if release_time <= now {
                    self.available[p] = self.available[p].saturating_add(1);
                } else {
                    still_pending.push(release_time);
                }
            }
            self.pending_release[p] = still_pending;
        }
    }

    fn next_release_time(&self) -> Option<f64> {
        self.pending_release
            .iter()
            .flat_map(|items| items.iter().copied())
            .reduce(f64::min)
    }
}

fn push_log_entry_sampled(
    logs: &mut Vec<LogEntry>,
    entry: LogEntry,
    raw_log_total: &mut usize,
    sample_stride: &mut usize,
) {
    if (*raw_log_total).is_multiple_of(*sample_stride) {
        logs.push(entry);
    }
    *raw_log_total = raw_log_total.saturating_add(1);

    while logs.len() > MAX_SIM_LOG_ENTRIES {
        let mut reduced = Vec::with_capacity(logs.len().div_ceil(2));
        for (idx, item) in logs.drain(..).enumerate() {
            if idx % 2 == 0 {
                reduced.push(item);
            }
        }
        *logs = reduced;
        *sample_stride = sample_stride.saturating_mul(2).max(1);
    }
}

pub fn run_simulation(
    net: &PetriNet,
    params: &SimulationParams,
    _fixed_step: bool,
    collect_stats: bool,
) -> SimulationResult {
    let places = net.places.len();
    let mut state = SimState {
        available: net.tables.m0.clone(),
        pending_release: vec![Vec::new(); places],
        through_place_counter: vec![0; places],
        in_tokens: vec![0; places],
        out_tokens: vec![0; places],
    };

    let mut now = 0.0;
    let mut passes = 0_u64;
    let mut logs = Vec::new();
    let mut raw_log_total = 0usize;
    let mut log_sampling_stride = 1usize;
    push_log_entry_sampled(
        &mut logs,
        LogEntry {
            time: now,
            fired_transition: None,
            marking: state.total_marking(),
            touched_places: Vec::new(),
        },
        &mut raw_log_total,
        &mut log_sampling_stride,
    );
    // Deterministic by default: makes tests and bug reports reproducible.
    let mut rng = SmallRng::seed_from_u64(0x5EED_5EED);
    let mut seen_markings: HashMap<Vec<u32>, f64> = HashMap::new();
    let mut cycle_time = None;

    let mut stats_acc = vec![0_f64; places];
    let mut stats_min = vec![u32::MAX; places];
    let mut stats_max = vec![0_u32; places];
    let mut stats_observations = 0usize;

    loop {
        state.process_releases(now);
        let marking = state.total_marking();

        if cycle_time.is_none() {
            if let Some(prev) = seen_markings.insert(marking.clone(), now) {
                cycle_time = Some((now - prev).max(0.0));
            }
        }

        if collect_stats {
            for p in 0..places {
                let m = marking[p];
                stats_min[p] = stats_min[p].min(m);
                stats_max[p] = stats_max[p].max(m);
                stats_acc[p] += m as f64;
            }
            stats_observations = stats_observations.saturating_add(1);
        }

        let enabled = enabled_transitions(net, &state);
        if enabled.is_empty() {
            push_log_entry_sampled(
                &mut logs,
                LogEntry {
                    time: now,
                    fired_transition: None,
                    marking,
                    touched_places: Vec::new(),
                },
                &mut raw_log_total,
                &mut log_sampling_stride,
            );
            if let Some(next_release) = state.next_release_time() {
                let next_time = next_release;
                if next_time > now {
                    now = next_time;
                    if should_stop(net, &state, params, now, passes) {
                        break;
                    }
                    continue;
                }
            }
            break;
        }

        let fired = pick_transition(net, &enabled, &mut rng);
        let touched_places = fire_transition(net, &mut state, fired, now, &mut rng);
        passes = passes.saturating_add(1);

        push_log_entry_sampled(
            &mut logs,
            LogEntry {
                time: now,
                fired_transition: Some(fired),
                marking: state.total_marking(),
                touched_places,
            },
            &mut raw_log_total,
            &mut log_sampling_stride,
        );

        if should_stop(net, &state, params, now, passes) {
            break;
        }
    }

    let final_marking = state.total_marking();
    let need_final_snapshot = logs
        .last()
        .map(|entry| {
            entry.time != now
                || entry.marking.as_slice() != final_marking.as_slice()
                || entry.fired_transition.is_some()
        })
        .unwrap_or(true);
    if need_final_snapshot {
        logs.push(LogEntry {
            time: now,
            fired_transition: None,
            marking: final_marking.clone(),
            touched_places: Vec::new(),
        });
        raw_log_total = raw_log_total.saturating_add(1);
        if logs.len() > MAX_SIM_LOG_ENTRIES {
            let overflow = logs.len() - MAX_SIM_LOG_ENTRIES;
            logs.drain(0..overflow);
        }
    }

    let place_stats = if collect_stats && stats_observations > 0 {
        let n = stats_observations as f64;
        Some(
            (0..places)
                .map(|p| PlaceStats {
                    min: if stats_min[p] == u32::MAX {
                        0
                    } else {
                        stats_min[p]
                    },
                    max: stats_max[p],
                    avg: stats_acc[p] / n,
                })
                .collect(),
        )
    } else {
        None
    };

    let place_flow = if collect_stats {
        Some(
            (0..places)
                .map(|p| PlaceFlowStats {
                    in_tokens: state.in_tokens[p],
                    out_tokens: state.out_tokens[p],
                })
                .collect(),
        )
    } else {
        None
    };

    let sim_time = now.max(0.0);
    let place_load = if collect_stats && stats_observations > 0 {
        let n = stats_observations as f64;
        Some(
            (0..places)
                .map(|p| {
                    let avg_marking = stats_acc[p] / n;
                    let avg_over_capacity = net.tables.mo.get(p).and_then(|cap| *cap).map(|cap| {
                        if cap == 0 {
                            0.0
                        } else {
                            (avg_marking / cap as f64).clamp(0.0, 1.0e9)
                        }
                    });
                    let (in_rate, out_rate) = if sim_time > 0.0 {
                        (
                            Some(state.in_tokens[p] as f64 / sim_time),
                            Some(state.out_tokens[p] as f64 / sim_time),
                        )
                    } else {
                        (None, None)
                    };
                    PlaceLoadStats {
                        avg_over_capacity,
                        in_rate,
                        out_rate,
                    }
                })
                .collect(),
        )
    } else {
        None
    };

    SimulationResult {
        cycle_time,
        logs,
        log_entries_total: raw_log_total,
        log_sampling_stride,
        place_stats,
        place_flow,
        place_load,
        sim_time,
        fired_count: passes,
        final_marking,
    }
}

fn enabled_transitions(net: &PetriNet, state: &SimState) -> Vec<usize> {
    let mut enabled = Vec::new();
    let places = net.places.len();

    for t in 0..net.transitions.len() {
        let mut has_incident_arc = false;
        for p in 0..places {
            if net.tables.pre[p][t] > 0
                || net.tables.post[p][t] > 0
                || net.tables.inhibitor[p][t] > 0
            {
                has_incident_arc = true;
                break;
            }
        }
        if !has_incident_arc {
            continue;
        }

        let mut ok = true;

        for p in 0..places {
            let need = net.tables.pre[p][t];
            if state.available[p] < need {
                ok = false;
                break;
            }

            let inh = net.tables.inhibitor[p][t];
            if inh > 0 {
                let marking_total = state.available[p] + state.pending_release[p].len() as u32;
                if marking_total >= inh {
                    ok = false;
                    break;
                }
            }
        }

        if !ok {
            continue;
        }

        for p in 0..places {
            if let Some(cap) = net.tables.mo[p] {
                let current_total = state.available[p] + state.pending_release[p].len() as u32;
                let after = current_total
                    .saturating_sub(net.tables.pre[p][t])
                    .saturating_add(net.tables.post[p][t]);
                if after > cap {
                    ok = false;
                    break;
                }
            }
        }

        if ok {
            enabled.push(t);
        }
    }

    enabled
}

fn pick_transition(net: &PetriNet, enabled: &[usize], rng: &mut SmallRng) -> usize {
    let mut best_priority = i32::MIN;
    let mut best_pre_weight = 0_u32;
    for &t in enabled {
        let priority = *net.tables.mpr.get(t).unwrap_or(&0);
        let pre_weight = transition_pre_weight(net, t);
        if priority > best_priority {
            best_priority = priority;
            best_pre_weight = pre_weight;
        } else if priority == best_priority {
            best_pre_weight = best_pre_weight.max(pre_weight);
        }
    }

    let mut candidates: Vec<usize> = enabled
        .iter()
        .copied()
        .filter(|&t| {
            *net.tables.mpr.get(t).unwrap_or(&0) == best_priority
                && transition_pre_weight(net, t) == best_pre_weight
        })
        .collect();
    candidates.sort_unstable();
    let idx = rng.gen_range(0..candidates.len());
    candidates[idx]
}

fn transition_pre_weight(net: &PetriNet, transition_idx: usize) -> u32 {
    net.tables
        .pre
        .iter()
        .filter_map(|row| row.get(transition_idx).copied())
        .sum()
}

fn fire_transition(
    net: &PetriNet,
    state: &mut SimState,
    t: usize,
    now: f64,
    rng: &mut SmallRng,
) -> Vec<usize> {
    let mut touched_places = Vec::new();
    let mut push_touched = |p: usize| {
        if !touched_places.contains(&p) {
            touched_places.push(p);
        }
    };

    for p in 0..net.places.len() {
        let pre = net.tables.pre[p][t];
        if pre > 0 {
            state.out_tokens[p] = state.out_tokens[p].saturating_add(pre as u64);
            push_touched(p);
        }
        state.available[p] = state.available[p].saturating_sub(pre);
    }

    for p in 0..net.places.len() {
        let post = net.tables.post[p][t];
        if post == 0 {
            continue;
        }

        state.in_tokens[p] = state.in_tokens[p].saturating_add(post as u64);

        push_touched(p);
        let delay = sample_place_delay(net, p, net.tables.mz[p].max(0.0), rng);
        for _ in 0..post {
            if delay > 0.0 {
                state.pending_release[p].push(now + delay);
            } else {
                state.available[p] = state.available[p].saturating_add(1);
            }
            state.through_place_counter[p] = state.through_place_counter[p].saturating_add(1);
        }
    }
    touched_places
}

fn sample_place_delay(
    net: &PetriNet,
    place_index: usize,
    base_delay: f64,
    rng: &mut SmallRng,
) -> f64 {
    let Some(place) = net.places.get(place_index) else {
        return base_delay.max(0.0);
    };
    let value = match place.stochastic {
        StochasticDistribution::None => base_delay,
        StochasticDistribution::Uniform { min, max } => {
            let lo = min.min(max);
            let hi = min.max(max);
            if (hi - lo).abs() < f64::EPSILON {
                lo
            } else {
                rng.gen_range(lo..=hi)
            }
        }
        StochasticDistribution::Normal { mean, std_dev } => {
            let sigma = std_dev.max(0.0);
            if sigma <= f64::EPSILON {
                mean
            } else {
                let u1 = (1.0 - rng.gen::<f64>()).clamp(1e-12, 1.0);
                let u2 = rng.gen::<f64>();
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                mean + sigma * z
            }
        }
        StochasticDistribution::Exponential { lambda } => {
            let l = lambda.max(1e-9);
            let u = (1.0 - rng.gen::<f64>()).clamp(1e-12, 1.0);
            -u.ln() / l
        }
        StochasticDistribution::Gamma { shape, scale } => {
            let k = shape.max(1e-9);
            let theta = scale.max(1e-9);
            if let Ok(dist) = Gamma::new(k, theta) {
                dist.sample(rng)
            } else {
                base_delay
            }
        }
        StochasticDistribution::Poisson { lambda } => {
            let l = lambda.max(0.0);
            if l <= f64::EPSILON {
                0.0
            } else {
                let limit = (-l).exp();
                let mut k = 0_u32;
                let mut p = 1.0_f64;
                loop {
                    k = k.saturating_add(1);
                    p *= rng.gen::<f64>();
                    if p <= limit {
                        break;
                    }
                }
                (k.saturating_sub(1)) as f64
            }
        }
    };
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn should_stop(
    net: &PetriNet,
    state: &SimState,
    params: &SimulationParams,
    now: f64,
    passes: u64,
) -> bool {
    if params.use_pass_limit && passes >= params.pass_limit {
        return true;
    }

    if params.use_time_limit && now >= params.time_limit {
        return true;
    }

    if let Some((pk, n)) = params.stop.through_place {
        if pk < net.places.len() && state.through_place_counter[pk] >= n {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeRef, PetriNet};

    #[test]
    fn firing_rules_with_priority() {
        let mut net = PetriNet::new();
        net.set_counts(1, 2);
        net.tables.m0[0] = 2;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.tables.pre[0][1] = 1;
        net.tables.post[0][1] = 0;
        net.tables.mpr[0] = 1;
        net.tables.mpr[1] = 5;
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 1,
            ..SimulationParams::default()
        };
        let res = run_simulation(&net, &p, true, false);
        assert!(res.logs.len() > 1);
        assert_eq!(res.logs[1].fired_transition, Some(1));
    }

    #[test]
    fn timed_tokens_become_available_after_delay() {
        let mut net = PetriNet::new();
        net.add_place([0.0, 0.0]);
        net.add_place([100.0, 0.0]);
        net.add_transition([50.0, 0.0]);
        net.tables.m0[0] = 1;
        net.tables.mz[1] = 1.0;
        let p1 = net.places[0].id;
        let p2 = net.places[1].id;
        let t1 = net.transitions[0].id;
        net.add_arc(NodeRef::Place(p1), NodeRef::Transition(t1), 1);
        net.add_arc(NodeRef::Transition(t1), NodeRef::Place(p2), 1);

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 1,
            dt: 0.5,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, true, false);
        assert_eq!(res.final_marking[1], 1);
        assert!(res.logs[0].marking[1] <= 1);
    }

    #[test]
    fn isolated_transition_is_ignored() {
        let mut net = PetriNet::new();
        net.set_counts(1, 2);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.tables.mpr[0] = 1;
        net.tables.mpr[1] = 100; // isolated but higher priority
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 1,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, true, false);
        assert!(res.logs.len() > 1);
        assert_eq!(res.logs[1].fired_transition, Some(0));
    }

    #[test]
    fn simulation_waits_for_delayed_tokens_instead_of_stopping() {
        let mut net = PetriNet::new();
        net.set_counts(3, 2);
        net.tables.m0[0] = 1;
        net.tables.mz[1] = 1.0;
        net.tables.pre[0][0] = 1; // P1 -> T1
        net.tables.post[1][0] = 1; // T1 -> P2 (delayed)
        net.tables.pre[1][1] = 1; // P2 -> T2
        net.tables.post[2][1] = 1; // T2 -> P3
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 2,
            dt: 0.1,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, true, false);
        assert_eq!(res.fired_count, 2);
        assert_eq!(res.final_marking[2], 1);
    }
    #[test]
    fn zero_delay_transitions_do_not_advance_time() {
        let mut net = PetriNet::new();
        net.set_counts(1, 1);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 3,
            dt: 0.1,
            ..SimulationParams::default()
        };

        let res = run_simulation(&net, &p, false, false);
        assert_eq!(res.fired_count, 3);
        assert!(res
            .logs
            .iter()
            .all(|entry| (entry.time - 0.0).abs() < f64::EPSILON));
    }

    #[test]
    fn long_run_log_is_sampled_and_bounded() {
        let mut net = PetriNet::new();
        net.set_counts(1, 1);
        net.tables.m0[0] = 1;
        net.tables.pre[0][0] = 1;
        net.tables.post[0][0] = 1;
        net.rebuild_arcs_from_matrices();

        let p = SimulationParams {
            use_pass_limit: true,
            pass_limit: 200_000,
            ..SimulationParams::default()
        };
        let res = run_simulation(&net, &p, false, false);

        assert!(res.log_entries_total > MAX_SIM_LOG_ENTRIES);
        assert!(res.logs.len() <= MAX_SIM_LOG_ENTRIES);
        assert!(res.log_sampling_stride >= 1);
    }

    #[test]
    fn pick_transition_prefers_higher_weight() {
        let mut net = PetriNet::new();
        net.set_counts(1, 2);
        net.tables.m0[0] = 2;
        net.add_arc(NodeRef::Place(1), NodeRef::Transition(1), 1);
        net.add_arc(NodeRef::Place(1), NodeRef::Transition(2), 2);
        let mut rng = SmallRng::seed_from_u64(0xDEAD_BEEF);
        let enabled = vec![0, 1];
        let chosen = pick_transition(&net, &enabled, &mut rng);
        assert_eq!(chosen, 1);
    }
}


# src\sim\mod.rs
pub mod engine;


# src\ui\app\graph_view.rs
use std::collections::HashSet;

use super::*;

use egui::epaint;

impl PetriApp {
    pub(super) fn draw_graph_view(&mut self, ui: &mut egui::Ui) {
        self.update_debug_animation_clock(ui.ctx());
        ui.heading("Граф");
        let desired = ui.available_size_before_wrap();
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > f32::EPSILON {
            self.canvas.zoom = (self.canvas.zoom * zoom_delta).clamp(0.2, 3.0);
        }

        if response.dragged_by(egui::PointerButton::Middle) {
            self.canvas.pan += response.drag_delta();
        }

        if !self.net.ui.hide_grid {
            // Draw grid aligned to world coordinates so snapped nodes land exactly on grid lines.
            let step_world = self.grid_step_world();
            let world_min = self.screen_to_world(rect, rect.left_top());
            let world_max = self.screen_to_world(rect, rect.right_bottom());
            let ppp = ui.ctx().pixels_per_point();
            let snap_to_pixel = |v: f32| (v * ppp).round() / ppp;

            let min_x = world_min[0].min(world_max[0]);
            let max_x = world_min[0].max(world_max[0]);
            let min_y = world_min[1].min(world_max[1]);
            let max_y = world_min[1].max(world_max[1]);

            // Start on the previous grid line so the first visible line is stable when panning.
            let mut xw = (min_x / step_world).floor() * step_world;
            while xw <= max_x + step_world {
                let xs = snap_to_pixel(self.world_to_screen(rect, [xw, 0.0]).x);
                painter.line_segment(
                    [Pos2::new(xs, rect.top()), Pos2::new(xs, rect.bottom())],
                    Stroke::new(1.0, Color32::from_gray(230)),
                );
                xw += step_world;
            }

            let mut yw = (min_y / step_world).floor() * step_world;
            while yw <= max_y + step_world {
                let ys = snap_to_pixel(self.world_to_screen(rect, [0.0, yw]).y);
                painter.line_segment(
                    [Pos2::new(rect.left(), ys), Pos2::new(rect.right(), ys)],
                    Stroke::new(1.0, Color32::from_gray(230)),
                );
                yw += step_world;
            }
        }

        if let Some(pos) = response.hover_pos() {
            self.canvas.cursor_world = self.screen_to_world(rect, pos);
            self.canvas.cursor_valid = true;
        }
        if response.hovered() {
            ui.output_mut(|o| {
                o.cursor_icon = match self.tool {
                    Tool::Place | Tool::Transition | Tool::Arc | Tool::Frame => {
                        egui::CursorIcon::Crosshair
                    }
                    Tool::Text => egui::CursorIcon::Text,
                    Tool::Delete => egui::CursorIcon::NotAllowed,
                    Tool::Edit | Tool::Run => egui::CursorIcon::PointingHand,
                }
            });
        }
        if response.double_clicked_by(egui::PointerButton::Primary) {
            if let Some(click) = response.interact_pointer_pos() {
                if let Some(node) = self.node_at(rect, click) {
                    self.tool = Tool::Edit;
                    self.clear_selection();
                    match node {
                        NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                        NodeRef::Transition(t) => self.canvas.selected_transition = Some(t),
                    }
                }
            }
        }

        if response.clicked() {
            if let Some(click) = response.interact_pointer_pos() {
                let world = self.screen_to_world(rect, click);
                let snapped = self.snapped_world(world);

                match self.tool {
                    Tool::Place => {
                        self.push_undo_snapshot();
                        self.net.add_place(snapped);
                        if let Some(new_id) = self.net.places.iter().map(|p| p.id).max() {
                            self.assign_auto_name_for_place(new_id);
                            if let Some(idx) = self.place_idx_by_id(new_id) {
                                self.net.places[idx].size = self.new_place_size;
                                self.net.places[idx].color = self.new_place_color;
                                if idx < self.net.tables.m0.len() {
                                    self.net.tables.m0[idx] = self.new_place_marking;
                                }
                                if idx < self.net.tables.mo.len() {
                                    self.net.tables.mo[idx] = self.new_place_capacity;
                                }
                                if idx < self.net.tables.mz.len() {
                                    self.net.tables.mz[idx] = self.new_place_delay.max(0.0);
                                }
                            }
                        }
                    }
                    Tool::Transition => {
                        // Store transition position as top-left.
                        // Snap the top-left to the grid (not the center) so the rectangle aligns with the grid.
                        self.push_undo_snapshot();
                        let dims = Self::transition_dimensions(self.new_transition_size);
                        let tl =
                            self.snapped_world([world[0] - dims.x * 0.5, world[1] - dims.y * 0.5]);
                        self.net.add_transition(tl);
                        if let Some(new_id) = self.net.transitions.iter().map(|t| t.id).max() {
                            if let Some(idx) = self.transition_idx_by_id(new_id) {
                                self.net.transitions[idx].size = self.new_transition_size;
                                self.net.transitions[idx].color = self.new_transition_color;
                                if idx < self.net.tables.mpr.len() {
                                    self.net.tables.mpr[idx] = self.new_transition_priority;
                                }
                            }
                        }
                    }
                    Tool::Arc => {}
                    Tool::Text => {
                        self.push_undo_snapshot();
                        let id = self.next_text_id;
                        self.next_text_id = self.next_text_id.saturating_add(1);
                        self.text_blocks.push(CanvasTextBlock {
                            id,
                            pos: snapped,
                            text: self.tr("Текст", "Text").to_string(),
                            font_name: "MS Sans Serif".to_string(),
                            font_size: 10.0,
                            color: NodeColor::Default,
                        });
                        self.clear_selection();
                        self.canvas.selected_text = Some(id);
                        self.text_props_id = Some(id);
                        self.show_text_props = true;
                        self.show_place_props = false;
                        self.show_transition_props = false;
                    }
                    Tool::Frame => {}
                    Tool::Delete => {
                        if let Some(node) = self.node_at(rect, click) {
                            self.push_undo_snapshot();
                            match node {
                                NodeRef::Place(p) => {
                                    if let Some(idx) = self.place_idx_by_id(p) {
                                        self.net.tables.remove_place_row(idx);
                                        self.net.places.remove(idx);
                                        self.net.set_counts(
                                            self.net.places.len(),
                                            self.net.transitions.len(),
                                        );
                                    }
                                }
                                NodeRef::Transition(t) => {
                                    if let Some(idx) = self.transition_idx_by_id(t) {
                                        self.net.tables.remove_transition_column(idx);
                                        self.net.transitions.remove(idx);
                                        self.net.set_counts(
                                            self.net.places.len(),
                                            self.net.transitions.len(),
                                        );
                                    }
                                }
                            }
                        } else if let Some(arc_id) = self.arc_at(rect, click) {
                            self.push_undo_snapshot();
                            self.net.arcs.retain(|a| a.id != arc_id);
                            self.net.inhibitor_arcs.retain(|a| a.id != arc_id);
                            self.net.rebuild_matrices_from_arcs();
                        } else if let Some(text_id) = self.text_at(rect, click) {
                            self.push_undo_snapshot();
                            self.text_blocks.retain(|item| item.id != text_id);
                        } else if let Some(frame_id) = self.frame_at(rect, click) {
                            self.push_undo_snapshot();
                            self.decorative_frames.retain(|item| item.id != frame_id);
                        }
                    }
                    Tool::Edit => {
                        let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
                        if shift_pressed {
                            self.promote_single_selection_to_multi();
                            if let Some(text_id) = self.text_at(rect, click) {
                                let added = Self::toggle_selected_id(
                                    &mut self.canvas.selected_texts,
                                    text_id,
                                );
                                self.canvas.selected_frames.clear();
                                self.canvas.selected_frame = None;
                                self.canvas.selected_text = if added {
                                    Some(text_id)
                                } else {
                                    self.canvas.selected_texts.last().copied()
                                };
                            } else if let Some(frame_id) = self.frame_at(rect, click) {
                                let added = Self::toggle_selected_id(
                                    &mut self.canvas.selected_frames,
                                    frame_id,
                                );
                                self.canvas.selected_texts.clear();
                                self.canvas.selected_text = None;
                                self.canvas.selected_frame = if added {
                                    Some(frame_id)
                                } else {
                                    self.canvas.selected_frames.last().copied()
                                };
                            } else if let Some(n) = self.node_at(rect, click) {
                                match n {
                                    NodeRef::Place(p) => {
                                        Self::toggle_selected_id(
                                            &mut self.canvas.selected_places,
                                            p,
                                        );
                                    }
                                    NodeRef::Transition(t) => {
                                        Self::toggle_selected_id(
                                            &mut self.canvas.selected_transitions,
                                            t,
                                        );
                                    }
                                }
                                self.canvas.selected_text = None;
                                self.canvas.selected_texts.clear();
                                self.canvas.selected_frame = None;
                                self.canvas.selected_frames.clear();
                            } else if let Some(arc_id) = self.arc_at(rect, click) {
                                Self::toggle_selected_id(&mut self.canvas.selected_arcs, arc_id);
                                self.canvas.selected_text = None;
                                self.canvas.selected_texts.clear();
                                self.canvas.selected_frame = None;
                                self.canvas.selected_frames.clear();
                            }
                            self.sync_primary_selection_from_multi();
                        } else {
                            self.clear_selection();
                            if let Some(text_id) = self.text_at(rect, click) {
                                self.canvas.selected_text = Some(text_id);
                            } else if let Some(frame_id) = self.frame_at(rect, click) {
                                self.canvas.selected_frame = Some(frame_id);
                            } else if let Some(n) = self.node_at(rect, click) {
                                match n {
                                    NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                                    NodeRef::Transition(t) => {
                                        self.canvas.selected_transition = Some(t)
                                    }
                                }
                            } else if let Some(arc_id) = self.arc_at(rect, click) {
                                self.canvas.selected_arc = Some(arc_id);
                                self.canvas.selected_arcs.clear();
                                self.canvas.selected_arcs.push(arc_id);
                            }
                        }
                    }
                    Tool::Run => {}
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) && self.tool == Tool::Arc {
            if let Some(pointer) = response.interact_pointer_pos() {
                self.canvas.arc_start = self.node_at(rect, pointer);
            }
        }
        if self.tool == Tool::Arc && response.drag_stopped() {
            if let Some(first) = self.canvas.arc_start.take() {
                if let Some(pointer) = response
                    .interact_pointer_pos()
                    .or_else(|| response.hover_pos())
                {
                    if let Some(last) = self.node_at(rect, pointer) {
                        if first != last {
                            self.push_undo_snapshot();
                            if self.new_arc_inhibitor {
                                let pair = match (first, last) {
                                    (NodeRef::Place(pid), NodeRef::Transition(tid)) => {
                                        Some((pid, tid))
                                    }
                                    (NodeRef::Transition(tid), NodeRef::Place(pid)) => {
                                        Some((pid, tid))
                                    }
                                    _ => None,
                                };
                                if let Some((place_id, transition_id)) = pair {
                                    self.net.add_inhibitor_arc(
                                        place_id,
                                        transition_id,
                                        self.new_arc_inhibitor_threshold.max(1),
                                    );
                                    if let Some(last_inh) = self.net.inhibitor_arcs.last_mut() {
                                        last_inh.color = self.new_arc_color;
                                        last_inh.visible = true;
                                    }
                                }
                            } else {
                                self.net.add_arc(first, last, self.new_arc_weight.max(1));
                                if let Some(last_arc) = self.net.arcs.last_mut() {
                                    last_arc.color = self.new_arc_color;
                                    last_arc.visible = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.tool == Tool::Arc && !ui.ctx().input(|i| i.pointer.any_down()) {
            self.canvas.arc_start = None;
        }

        if response.drag_started_by(egui::PointerButton::Primary) && self.tool == Tool::Frame {
            if let Some(pointer) = response.interact_pointer_pos() {
                self.clear_selection();
                let start = self.snapped_world(self.screen_to_world(rect, pointer));
                self.canvas.frame_draw_start_world = Some(start);
                self.canvas.frame_draw_current_world = Some(start);
            }
        }

        if self.tool == Tool::Frame && response.dragged_by(egui::PointerButton::Primary) {
            if let Some(pointer) = response.interact_pointer_pos() {
                self.canvas.frame_draw_current_world =
                    Some(self.snapped_world(self.screen_to_world(rect, pointer)));
            }
        }

        if self.tool == Tool::Frame && response.drag_stopped() {
            if let (Some(start), Some(current)) = (
                self.canvas.frame_draw_start_world.take(),
                self.canvas.frame_draw_current_world.take(),
            ) {
                let (mut pos, mut width, mut height) = Self::frame_from_drag(start, current);
                if width >= 1.0 || height >= 1.0 {
                    if self.net.ui.snap_to_grid {
                        pos = self.snap_point_to_grid(pos);
                        width = self.snap_scalar_to_grid(width);
                        height = self.snap_scalar_to_grid(height);
                    }
                    width = width.max(Self::FRAME_MIN_SIDE);
                    height = height.max(Self::FRAME_MIN_SIDE);
                    self.push_undo_snapshot();
                    let id = self.next_frame_id;
                    self.next_frame_id = self.next_frame_id.saturating_add(1);
                    self.decorative_frames.push(CanvasFrame {
                        id,
                        pos,
                        width,
                        height,
                    });
                    self.clear_selection();
                    self.canvas.selected_frame = Some(id);
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) && self.tool == Tool::Edit {
            if let Some(pointer) = response.interact_pointer_pos() {
                let mut handled_resize = false;
                if let Some(frame_id) = self.canvas.selected_frame {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        let handle =
                            self.frame_resize_handle_rect(rect, &self.decorative_frames[idx]);
                        if handle.expand(4.0).contains(pointer) {
                            self.push_undo_snapshot();
                            self.canvas.frame_resize_id = Some(frame_id);
                            self.canvas.drag_prev_world = None;
                            self.canvas.move_drag_active = false;
                            self.canvas.selection_start = None;
                            self.canvas.selection_rect = None;
                            handled_resize = true;
                        }
                    }
                }
                if !handled_resize {
                    let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
                    if shift_pressed {
                        self.promote_single_selection_to_multi();
                        self.canvas.selection_toggle_mode = true;
                        self.canvas.selection_start = Some(pointer);
                        self.canvas.selection_rect = Some(Rect::from_two_pos(pointer, pointer));
                        self.canvas.drag_prev_world = None;
                        self.canvas.move_drag_active = false;
                    } else if let Some(node) = self.node_at(rect, pointer) {
                        let is_selected = match node {
                            NodeRef::Place(p) => {
                                self.canvas.selected_place == Some(p)
                                    || self.canvas.selected_places.contains(&p)
                            }
                            NodeRef::Transition(t) => {
                                self.canvas.selected_transition == Some(t)
                                    || self.canvas.selected_transitions.contains(&t)
                            }
                        };

                        if is_selected {
                            self.push_undo_snapshot();
                            self.canvas.drag_prev_world = Some(self.screen_to_world(rect, pointer));
                            self.canvas.move_drag_active = true;
                        } else {
                            self.clear_selection();
                            match node {
                                NodeRef::Place(p) => self.canvas.selected_place = Some(p),
                                NodeRef::Transition(t) => self.canvas.selected_transition = Some(t),
                            }
                            self.canvas.drag_prev_world = None;
                            self.canvas.move_drag_active = false;
                        }
                    } else if let Some(text_id) = self.text_at(rect, pointer) {
                        if self.canvas.selected_text != Some(text_id) {
                            self.clear_selection();
                            self.canvas.selected_text = Some(text_id);
                        }
                        self.push_undo_snapshot();
                        self.canvas.drag_prev_world = Some(self.screen_to_world(rect, pointer));
                        self.canvas.move_drag_active = true;
                    } else if let Some(frame_id) = self.frame_at(rect, pointer) {
                        if self.canvas.selected_frame != Some(frame_id) {
                            self.clear_selection();
                            self.canvas.selected_frame = Some(frame_id);
                        }
                        self.push_undo_snapshot();
                        self.canvas.drag_prev_world = Some(self.screen_to_world(rect, pointer));
                        self.canvas.move_drag_active = true;
                    } else {
                        self.clear_selection();
                        self.canvas.selection_toggle_mode = false;
                        self.canvas.selection_start = Some(pointer);
                        self.canvas.selection_rect = Some(Rect::from_two_pos(pointer, pointer));
                        self.canvas.drag_prev_world = None;
                        self.canvas.move_drag_active = false;
                    }
                }
            }
        }

        if self.tool == Tool::Edit && response.dragged_by(egui::PointerButton::Primary) {
            if let Some(frame_id) = self.canvas.frame_resize_id {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        let frame_pos = self.decorative_frames[idx].pos;
                        let world = self.screen_to_world(rect, pointer);
                        let mut width = world[0] - frame_pos[0];
                        let mut height = world[1] - frame_pos[1];
                        if self.net.ui.snap_to_grid {
                            width = self.snap_scalar_to_grid(width);
                            height = self.snap_scalar_to_grid(height);
                        }
                        self.decorative_frames[idx].width = width.max(Self::FRAME_MIN_SIDE);
                        self.decorative_frames[idx].height = height.max(Self::FRAME_MIN_SIDE);
                    }
                }
            } else if let Some(start) = self.canvas.selection_start {
                if let Some(pointer) = response.interact_pointer_pos() {
                    self.canvas.selection_rect = Some(Rect::from_two_pos(start, pointer));
                }
            } else if self.canvas.move_drag_active {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let world = self.screen_to_world(rect, pointer);
                    if let Some(prev) = self.canvas.drag_prev_world {
                        let dx = world[0] - prev[0];
                        let dy = world[1] - prev[1];
                        if dx.abs() > f32::EPSILON || dy.abs() > f32::EPSILON {
                            let move_place_ids: Vec<u64> = if self.canvas.selected_places.is_empty()
                            {
                                self.canvas.selected_place.into_iter().collect()
                            } else {
                                self.canvas.selected_places.clone()
                            };
                            let move_transition_ids: Vec<u64> =
                                if self.canvas.selected_transitions.is_empty() {
                                    self.canvas.selected_transition.into_iter().collect()
                                } else {
                                    self.canvas.selected_transitions.clone()
                                };

                            for pid in move_place_ids {
                                if let Some(idx) = self.place_idx_by_id(pid) {
                                    self.net.places[idx].pos[0] += dx;
                                    self.net.places[idx].pos[1] += dy;
                                }
                            }
                            for tid in move_transition_ids {
                                if let Some(idx) = self.transition_idx_by_id(tid) {
                                    self.net.transitions[idx].pos[0] += dx;
                                    self.net.transitions[idx].pos[1] += dy;
                                }
                            }
                            for text_id in self.collect_selected_text_ids() {
                                if let Some(idx) = self.text_idx_by_id(text_id) {
                                    self.text_blocks[idx].pos[0] += dx;
                                    self.text_blocks[idx].pos[1] += dy;
                                }
                            }
                            for frame_id in self.collect_selected_frame_ids() {
                                if let Some(idx) = self.frame_idx_by_id(frame_id) {
                                    self.decorative_frames[idx].pos[0] += dx;
                                    self.decorative_frames[idx].pos[1] += dy;
                                }
                            }
                        }
                    }
                    self.canvas.drag_prev_world = Some(world);
                }
            }
        }

        if self.tool == Tool::Edit && response.drag_stopped() {
            if self.canvas.move_drag_active && self.net.ui.snap_to_grid {
                let step = self.grid_step_world();
                let snap = |value: f32| (value / step).round() * step;
                let move_place_ids: Vec<u64> = if self.canvas.selected_places.is_empty() {
                    self.canvas.selected_place.into_iter().collect()
                } else {
                    self.canvas.selected_places.clone()
                };
                let move_transition_ids: Vec<u64> = if self.canvas.selected_transitions.is_empty() {
                    self.canvas.selected_transition.into_iter().collect()
                } else {
                    self.canvas.selected_transitions.clone()
                };
                for pid in move_place_ids {
                    if let Some(idx) = self.place_idx_by_id(pid) {
                        self.net.places[idx].pos[0] = snap(self.net.places[idx].pos[0]);
                        self.net.places[idx].pos[1] = snap(self.net.places[idx].pos[1]);
                    }
                }
                for tid in move_transition_ids {
                    if let Some(idx) = self.transition_idx_by_id(tid) {
                        self.net.transitions[idx].pos[0] = snap(self.net.transitions[idx].pos[0]);
                        self.net.transitions[idx].pos[1] = snap(self.net.transitions[idx].pos[1]);
                    }
                }
                for text_id in self.collect_selected_text_ids() {
                    if let Some(idx) = self.text_idx_by_id(text_id) {
                        self.text_blocks[idx].pos[0] = snap(self.text_blocks[idx].pos[0]);
                        self.text_blocks[idx].pos[1] = snap(self.text_blocks[idx].pos[1]);
                    }
                }
                for frame_id in self.collect_selected_frame_ids() {
                    if let Some(idx) = self.frame_idx_by_id(frame_id) {
                        self.decorative_frames[idx].pos[0] =
                            snap(self.decorative_frames[idx].pos[0]);
                        self.decorative_frames[idx].pos[1] =
                            snap(self.decorative_frames[idx].pos[1]);
                    }
                }
            }
            if let Some(sel_rect) = self.canvas.selection_rect.take() {
                let norm = sel_rect.expand2(Vec2::ZERO);
                let hit_places: Vec<u64> = self
                    .net
                    .places
                    .iter()
                    .filter(|p| norm.contains(self.world_to_screen(rect, p.pos)))
                    .map(|p| p.id)
                    .collect();
                let hit_transitions: Vec<u64> = self
                    .net
                    .transitions
                    .iter()
                    .filter(|t| {
                        let pos = self.world_to_screen(rect, t.pos);
                        let tr_rect = Rect::from_min_size(
                            pos,
                            Self::transition_dimensions(t.size) * self.canvas.zoom,
                        );
                        norm.intersects(tr_rect)
                    })
                    .map(|t| t.id)
                    .collect();
                let mut hit_arcs: Vec<u64> = self
                    .net
                    .arcs
                    .iter()
                    .filter(|arc| {
                        if !self.arc_visible_by_mode(arc.color, arc.visible) {
                            return false;
                        }
                        let Some((from, to)) = self.arc_screen_endpoints(rect, arc) else {
                            return false;
                        };
                        Self::arc_fully_inside_rect(norm, from, to)
                    })
                    .map(|arc| arc.id)
                    .collect();
                let selected_inhibitor_ids: Vec<u64> = self
                    .net
                    .inhibitor_arcs
                    .iter()
                    .filter(|inh| {
                        if !self.arc_visible_by_mode(inh.color, inh.visible) {
                            return false;
                        }
                        let Some((from, to)) = self.inhibitor_screen_endpoints(rect, inh) else {
                            return false;
                        };
                        norm.contains(from) && norm.contains(to)
                    })
                    .map(|inh| inh.id)
                    .collect();
                hit_arcs.extend(selected_inhibitor_ids);
                let hit_text_ids: Vec<u64> = self
                    .text_blocks
                    .iter()
                    .filter(|text| norm.contains(self.world_to_screen(rect, text.pos)))
                    .map(|text| text.id)
                    .collect();
                let hit_frame_ids: Vec<u64> = self
                    .decorative_frames
                    .iter()
                    .filter(|frame| {
                        let min = self.world_to_screen(rect, frame.pos);
                        let size = Vec2::new(
                            frame.width.max(Self::FRAME_MIN_SIDE),
                            frame.height.max(Self::FRAME_MIN_SIDE),
                        ) * self.canvas.zoom;
                        let frame_rect = Rect::from_min_size(min, size);
                        norm.intersects(frame_rect)
                    })
                    .map(|frame| frame.id)
                    .collect();

                if self.canvas.selection_toggle_mode {
                    self.promote_single_selection_to_multi();
                    for place_id in hit_places {
                        Self::toggle_selected_id(&mut self.canvas.selected_places, place_id);
                    }
                    for transition_id in hit_transitions {
                        Self::toggle_selected_id(
                            &mut self.canvas.selected_transitions,
                            transition_id,
                        );
                    }
                    for arc_id in hit_arcs {
                        Self::toggle_selected_id(&mut self.canvas.selected_arcs, arc_id);
                    }
                    for text_id in hit_text_ids {
                        Self::toggle_selected_id(&mut self.canvas.selected_texts, text_id);
                    }
                    for frame_id in hit_frame_ids {
                        Self::toggle_selected_id(&mut self.canvas.selected_frames, frame_id);
                    }
                    self.sync_primary_selection_from_multi();
                } else {
                    self.canvas.selected_places = hit_places;
                    self.canvas.selected_transitions = hit_transitions;
                    self.canvas.selected_arcs = hit_arcs;
                    self.canvas.selected_texts = hit_text_ids;
                    self.canvas.selected_frames = hit_frame_ids;
                    self.canvas.selected_place = None;
                    self.canvas.selected_transition = None;
                    self.canvas.selected_arc = self.canvas.selected_arcs.first().copied();
                    self.canvas.selected_text = self.canvas.selected_texts.first().copied();
                    self.canvas.selected_frame = self.canvas.selected_frames.first().copied();
                }
                self.canvas.selection_toggle_mode = false;
            }
            self.canvas.selection_start = None;
            self.canvas.drag_prev_world = None;
            self.canvas.move_drag_active = false;
            self.canvas.frame_resize_id = None;
        }

        if response.clicked_by(egui::PointerButton::Secondary) {
            if let Some(click) = response.interact_pointer_pos() {
                if let Some(node) = self.node_at(rect, click) {
                    self.clear_selection();
                    match node {
                        NodeRef::Place(p) => {
                            self.canvas.selected_place = Some(p);
                            self.place_props_id = Some(p);
                            self.show_place_props = true;
                            self.show_transition_props = false;
                            self.show_text_props = false;
                        }
                        NodeRef::Transition(t) => {
                            self.canvas.selected_transition = Some(t);
                            self.transition_props_id = Some(t);
                            self.show_transition_props = true;
                            self.show_place_props = false;
                            self.show_text_props = false;
                        }
                    }
                } else if let Some(text_id) = self.text_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_text = Some(text_id);
                    self.text_props_id = Some(text_id);
                    self.show_text_props = true;
                    self.show_place_props = false;
                    self.show_transition_props = false;
                } else if let Some(arc_id) = self.arc_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_arc = Some(arc_id);
                    self.canvas.selected_arcs.clear();
                    self.canvas.selected_arcs.push(arc_id);
                    self.arc_props_id = Some(arc_id);
                    self.show_arc_props = true;
                    self.show_place_props = false;
                    self.show_transition_props = false;
                    self.show_text_props = false;
                } else if let Some(frame_id) = self.frame_at(rect, click) {
                    self.clear_selection();
                    self.canvas.selected_frame = Some(frame_id);
                    self.show_text_props = false;
                }
            }
        }

        if let Some(sel) = self.canvas.selection_rect {
            painter.rect_stroke(sel, 0.0, Stroke::new(1.0, Color32::from_rgb(70, 120, 210)));
            painter.rect_filled(sel, 0.0, Color32::from_rgba_premultiplied(70, 120, 210, 25));
        }

        for frame in &self.decorative_frames {
            let min = self.world_to_screen(rect, frame.pos);
            let size = Vec2::new(
                frame.width.max(Self::FRAME_MIN_SIDE),
                frame.height.max(Self::FRAME_MIN_SIDE),
            ) * self.canvas.zoom;
            let r = Rect::from_min_size(min, size);
            let is_selected = self.canvas.selected_frame == Some(frame.id);
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(
                    if is_selected { 3.0 } else { 1.5 },
                    if is_selected {
                        Color32::from_rgb(255, 140, 0)
                    } else {
                        Color32::from_gray(90)
                    },
                ),
            );
            if is_selected {
                let handle = self.frame_resize_handle_rect(rect, frame);
                painter.rect_filled(handle, 0.0, Color32::from_rgb(255, 140, 0));
                painter.rect_stroke(handle, 0.0, Stroke::new(1.0, Color32::from_rgb(80, 40, 0)));
            }
        }
        let active_event = if self.show_debug && self.debug_animation_enabled {
            self.debug_animation_active_event
                .and_then(|idx| self.debug_animation_events.get(idx))
        } else {
            None
        };
        let (active_pre_arc_ids, active_post_arc_ids) = if let Some(event) = active_event {
            (
                event
                    .pre_arcs
                    .iter()
                    .map(|arc| arc.arc_id)
                    .collect::<HashSet<_>>(),
                event
                    .post_arcs
                    .iter()
                    .map(|arc| arc.arc_id)
                    .collect::<HashSet<_>>(),
            )
        } else {
            (HashSet::new(), HashSet::new())
        };

        for arc in &self.net.arcs {
            if !self.arc_visible_by_mode(arc.color, arc.visible) {
                continue;
            }
            let (from_center, from_radius, from_rect, to_center, to_radius, to_rect) =
                match (arc.from, arc.to) {
                    (NodeRef::Place(p), NodeRef::Transition(t)) => {
                        if let (Some(pi), Some(ti)) =
                            (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                        {
                            let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                            let p_radius =
                                Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                            let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                            let t_rect = Rect::from_min_size(
                                t_min,
                                Self::transition_dimensions(self.net.transitions[ti].size)
                                    * self.canvas.zoom,
                            );
                            (
                                p_center,
                                Some(p_radius),
                                None,
                                t_rect.center(),
                                None,
                                Some(t_rect),
                            )
                        } else {
                            continue;
                        }
                    }
                    (NodeRef::Transition(t), NodeRef::Place(p)) => {
                        if let (Some(pi), Some(ti)) =
                            (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                        {
                            let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                            let t_rect = Rect::from_min_size(
                                t_min,
                                Self::transition_dimensions(self.net.transitions[ti].size)
                                    * self.canvas.zoom,
                            );
                            let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                            let p_radius =
                                Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                            (
                                t_rect.center(),
                                None,
                                Some(t_rect),
                                p_center,
                                Some(p_radius),
                                None,
                            )
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

            let mut from = from_center;
            let mut to = to_center;
            let delta = to_center - from_center;
            let dir = if delta.length_sq() > 0.0 {
                delta.normalized()
            } else {
                Vec2::X
            };

            if let Some(radius) = from_radius {
                from += dir * radius;
            } else if let Some(r) = from_rect {
                from = Self::rect_border_point(r, dir);
            }

            if let Some(radius) = to_radius {
                to -= dir * radius;
            } else if let Some(r) = to_rect {
                to = Self::rect_border_point(r, -dir);
            }

            let arc_color = Self::color_to_egui(arc.color, Color32::DARK_GRAY);
            let mut arc_stroke = if self.canvas.selected_arc == Some(arc.id)
                || self.canvas.selected_arcs.contains(&arc.id)
            {
                Stroke::new(3.0, Color32::from_rgb(255, 140, 0))
            } else {
                Stroke::new(2.0, arc_color)
            };
            if self.debug_arc_animation
                && self.debug_animation_enabled
                && self.canvas.selected_arc != Some(arc.id)
                && !self.canvas.selected_arcs.contains(&arc.id)
            {
                let is_pre_arc = active_pre_arc_ids.contains(&arc.id);
                let is_post_arc = active_post_arc_ids.contains(&arc.id);
                if is_pre_arc || is_post_arc {
                    if let Some(event) = active_event {
                        let highlight_color = if is_pre_arc {
                            event
                                .pre_arcs
                                .iter()
                                .find(|a| a.arc_id == arc.id)
                                .and_then(|a| a.token_colors.first().copied())
                                .unwrap_or(event.entry_color)
                        } else {
                            event
                                .post_arcs
                                .iter()
                                .find(|a| a.arc_id == arc.id)
                                .and_then(|a| a.token_colors.first().copied())
                                .unwrap_or(event.exit_color)
                        };
                        arc_stroke = Stroke::new(3.0, highlight_color);
                    }
                }
            }
            painter.line_segment([from, to], arc_stroke);
            let arrow = to - from;
            if arrow.length_sq() <= f32::EPSILON {
                continue;
            }
            if arc.show_weight {
                let label = arc.weight.to_string();
                Self::draw_arc_weight_label(
                    ui,
                    &painter,
                    from,
                    to,
                    &label,
                    arc_color,
                    self.canvas.zoom,
                );
            }
            let dir = arrow.normalized();
            let tip = to;
            let left = tip - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
            let right = tip - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
            painter.line_segment([tip, left], arc_stroke);
            painter.line_segment([tip, right], arc_stroke);
        }

        for inh in &self.net.inhibitor_arcs {
            if !self.arc_visible_by_mode(inh.color, inh.visible) {
                continue;
            }
            if let (Some(pi), Some(ti)) = (
                self.place_idx_by_id(inh.place_id),
                self.transition_idx_by_id(inh.transition_id),
            ) {
                let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                let t_rect = Rect::from_min_size(
                    t_min,
                    Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom,
                );
                let t_center = t_rect.center();
                let delta = t_center - p_center;
                let dir = if delta.length_sq() > 0.0 {
                    delta.normalized()
                } else {
                    Vec2::X
                };
                let from = p_center + dir * p_radius;
                let to = Self::rect_border_point(t_rect, -dir);
                let inh_color = Self::color_to_egui(inh.color, Color32::RED);
                let inh_stroke = if self.canvas.selected_arc == Some(inh.id)
                    || self.canvas.selected_arcs.contains(&inh.id)
                {
                    Stroke::new(3.0, Color32::from_rgb(255, 140, 0))
                } else {
                    Stroke::new(1.5, inh_color)
                };
                painter.line_segment([from, to], inh_stroke);
                if inh.show_weight {
                    let label = inh.threshold.to_string();
                    Self::draw_arc_weight_label(
                        ui,
                        &painter,
                        from,
                        to,
                        &label,
                        inh_color,
                        self.canvas.zoom,
                    );
                }
            }
        }

        self.draw_markov_place_arcs(rect, &painter, ui);

        let use_debug_colors = self.debug_animation_enabled;
        let debug_state_active =
            self.sim_result.is_some() && (self.show_debug || self.debug_animation_enabled);
        let debug_marking = if debug_state_active {
            self.sim_result
                .as_ref()
                .and_then(|res| {
                    let visible = Self::debug_visible_log_indices(res);
                    if visible.is_empty() {
                        return None;
                    }
                    let step = self.debug_step.min(visible.len() - 1);
                    visible
                        .get(step)
                        .and_then(|&log_idx| res.logs.get(log_idx))
                        .map(|entry| entry.marking.clone())
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let debug_place_colors = if use_debug_colors {
            self.debug_place_colors
                .get(self.debug_step)
                .cloned()
                .unwrap_or_else(|| Vec::new())
        } else {
            Vec::new()
        };

        for (place_idx, place) in self.net.places.iter().enumerate() {
            let center = self.world_to_screen(rect, place.pos);
            let radius = Self::place_radius(place.size) * self.canvas.zoom;
            let place_color = Self::color_to_egui(place.color, Color32::BLACK);
            let is_selected = self.canvas.selected_place == Some(place.id)
                || self.canvas.selected_places.contains(&place.id);
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(
                    if is_selected { 3.0 } else { 2.0 },
                    if is_selected {
                        Color32::from_rgb(255, 140, 0)
                    } else {
                        place_color
                    },
                ),
            );
            let name_offset = Self::keep_label_inside(
                rect,
                center,
                Self::place_label_offset(place.text_position, radius, self.canvas.zoom),
            );
            painter.text(
                center + name_offset,
                egui::Align2::CENTER_CENTER,
                &place.name,
                egui::TextStyle::Small.resolve(ui.style()),
                if self.net.ui.colored_petri_nets {
                    Color32::from_rgb(0, 100, 180)
                } else {
                    place_color
                },
            );

            let (tokens, token_colors) = if use_debug_colors {
                (
                    debug_place_colors
                        .get(place_idx)
                        .map(|colors| colors.len() as u32)
                        .unwrap_or_else(|| {
                            debug_marking.get(place_idx).copied().unwrap_or_else(|| {
                                self.net.tables.m0.get(place_idx).copied().unwrap_or(0)
                            })
                        }),
                    debug_place_colors
                        .get(place_idx)
                        .cloned()
                        .unwrap_or_else(|| Vec::new()),
                )
            } else if debug_state_active {
                (
                    debug_marking
                        .get(place_idx)
                        .copied()
                        .unwrap_or_else(|| self.net.tables.m0.get(place_idx).copied().unwrap_or(0)),
                    Vec::new(),
                )
            } else {
                (
                    self.net.tables.m0.get(place_idx).copied().unwrap_or(0),
                    Vec::new(),
                )
            };
            if tokens > 0 {
                if use_debug_colors {
                    if tokens > 5 {
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            format!("{tokens}"),
                            egui::TextStyle::Body.resolve(ui.style()),
                            token_colors
                                .last()
                                .copied()
                                .unwrap_or(Color32::from_rgb(200, 0, 0)),
                        );
                    } else {
                        let draw_tokens = tokens as usize;
                        for i in 0..draw_tokens {
                            let angle =
                                (i as f32) * std::f32::consts::TAU / (draw_tokens.max(1) as f32);
                            let dot_pos =
                                center + Vec2::new(angle.cos(), angle.sin()) * (radius * 0.55);
                            let color = token_colors
                                .iter()
                                .rev()
                                .nth(i)
                                .copied()
                                .unwrap_or(Color32::from_rgb(200, 0, 0));
                            painter.circle_filled(
                                dot_pos,
                                3.0 * self.canvas.zoom.clamp(0.7, 1.2),
                                color,
                            );
                        }
                    }
                } else if tokens <= 4 {
                    let draw_tokens = tokens;
                    for i in 0..draw_tokens {
                        let angle =
                            (i as f32) * std::f32::consts::TAU / (draw_tokens.max(1) as f32);
                        let dot_pos =
                            center + Vec2::new(angle.cos(), angle.sin()) * (radius * 0.55);
                        painter.circle_filled(
                            dot_pos,
                            3.0 * self.canvas.zoom.clamp(0.7, 1.2),
                            Color32::from_rgb(200, 0, 0),
                        );
                    }
                } else {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("{tokens}"),
                        egui::TextStyle::Body.resolve(ui.style()),
                        Color32::from_rgb(200, 0, 0),
                    );
                }
            }
            if let Some(annotation) = self.markov_annotations.get(&place.id) {
                let (annotation_offset, align) = match place.markov_placement {
                    MarkovPlacement::Bottom => {
                        (Vec2::new(0.0, radius + 8.0), egui::Align2::CENTER_TOP)
                    }
                    MarkovPlacement::Top => {
                        (Vec2::new(0.0, -(radius + 8.0)), egui::Align2::CENTER_BOTTOM)
                    }
                };
                let font_size = (12.0 * self.canvas.zoom).clamp(12.0, 22.0);
                let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);
                painter.text(
                    center + annotation_offset,
                    align,
                    annotation,
                    font_id,
                    Color32::from_rgb(24, 24, 24),
                );
            }
        }

        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let dims = Self::transition_dimensions(tr.size) * self.canvas.zoom;
            let r = Rect::from_min_size(p, dims);
            let tr_color = Self::color_to_egui(tr.color, Color32::BLACK);
            let is_selected = self.canvas.selected_transition == Some(tr.id)
                || self.canvas.selected_transitions.contains(&tr.id);
            painter.rect_stroke(
                r,
                0.0,
                Stroke::new(
                    if is_selected { 3.0 } else { 2.0 },
                    if is_selected {
                        Color32::from_rgb(255, 140, 0)
                    } else {
                        tr_color
                    },
                ),
            );
            painter.text(
                r.center() + Self::label_offset(tr.label_position, self.canvas.zoom),
                egui::Align2::CENTER_CENTER,
                &tr.name,
                egui::TextStyle::Small.resolve(ui.style()),
                tr_color,
            );
        }

        for text in &self.text_blocks {
            let center = self.world_to_screen(rect, text.pos);
            let draw_color = if self.canvas.selected_text == Some(text.id) {
                Color32::from_rgb(255, 140, 0)
            } else {
                Self::color_to_egui(text.color, Color32::from_rgb(40, 40, 40))
            };
            let family = Self::text_family_from_name(&text.font_name);
            let font_id = egui::FontId::new(text.font_size.max(6.0) * self.canvas.zoom, family);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                &text.text,
                font_id,
                draw_color,
            );
        }

        let preview_pos = response.hover_pos().map(|pointer| {
            let world = self.screen_to_world(rect, pointer);
            self.world_to_screen(rect, self.snapped_world(world))
        });
        if let Some(preview) = preview_pos {
            match self.tool {
                Tool::Place => {
                    painter.circle_stroke(
                        preview,
                        Self::place_radius(VisualSize::Medium) * self.canvas.zoom,
                        Stroke::new(2.0, Color32::from_rgb(60, 120, 220)),
                    );
                }
                Tool::Transition => {
                    let dims = Self::transition_dimensions(VisualSize::Medium) * self.canvas.zoom;
                    let r = Rect::from_center_size(preview, dims);
                    painter.rect_stroke(r, 0.0, Stroke::new(2.0, Color32::from_rgb(60, 120, 220)));
                }
                Tool::Text => {
                    painter.text(
                        preview,
                        egui::Align2::CENTER_CENTER,
                        self.tr("Текст", "Text"),
                        egui::TextStyle::Body.resolve(ui.style()),
                        Color32::from_rgb(60, 120, 220),
                    );
                }
                Tool::Frame => {
                    if let (Some(start), Some(current)) = (
                        self.canvas.frame_draw_start_world,
                        self.canvas.frame_draw_current_world,
                    ) {
                        let (pos, width, height) = Self::frame_from_drag(start, current);
                        if width >= 1.0 || height >= 1.0 {
                            let min = self.world_to_screen(rect, pos);
                            let r = Rect::from_min_size(
                                min,
                                Vec2::new(
                                    width.max(Self::FRAME_MIN_SIDE),
                                    height.max(Self::FRAME_MIN_SIDE),
                                ) * self.canvas.zoom,
                            );
                            painter.rect_stroke(
                                r,
                                0.0,
                                Stroke::new(2.0, Color32::from_rgb(60, 120, 220)),
                            );
                        }
                    }
                }
                Tool::Delete => {
                    let s = 8.0 * self.canvas.zoom;
                    let a = preview + Vec2::new(-s, -s);
                    let b = preview + Vec2::new(s, s);
                    let c = preview + Vec2::new(-s, s);
                    let d = preview + Vec2::new(s, -s);
                    let stroke = Stroke::new(2.0, Color32::from_rgb(220, 60, 60));
                    painter.line_segment([a, b], stroke);
                    painter.line_segment([c, d], stroke);
                }
                _ => {}
            }
        }
        if use_debug_colors {
            if let Some(pointer) = response.hover_pos() {
                if let Some(NodeRef::Place(place_id)) = self.node_at(rect, pointer) {
                    if let Some(place_idx) = self.place_idx_by_id(place_id) {
                        if let Some(place_colors) = self
                            .debug_place_colors
                            .get(self.debug_step)
                            .and_then(|places| places.get(place_idx))
                        {
                            if !place_colors.is_empty() {
                                let counts = Self::aggregate_token_counts(place_colors);
                                if !counts.is_empty() {
                                    let tooltip_id = egui::Id::new("debug_token_counts_tooltip");
                                    let tooltip_layer =
                                        egui::LayerId::new(egui::Order::Tooltip, tooltip_id);
                                    egui::show_tooltip(ui.ctx(), tooltip_layer, tooltip_id, |ui| {
                                        ui.label(self.tr("Состав маркеров", "Token breakdown"));
                                        for (color, count) in counts.iter() {
                                            ui.horizontal(|ui| {
                                                ui.colored_label(*color, "●");
                                                ui.label(count.to_string());
                                            });
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.tool == Tool::Arc {
            if let (Some(first), Some(pointer)) = (self.canvas.arc_start, response.hover_pos()) {
                let start = match first {
                    NodeRef::Place(pid) => {
                        if let Some(pi) = self.place_idx_by_id(pid) {
                            self.world_to_screen(rect, self.net.places[pi].pos)
                        } else {
                            pointer
                        }
                    }
                    NodeRef::Transition(tid) => {
                        if let Some(ti) = self.transition_idx_by_id(tid) {
                            let min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                            Rect::from_min_size(
                                min,
                                Self::transition_dimensions(self.net.transitions[ti].size)
                                    * self.canvas.zoom,
                            )
                            .center()
                        } else {
                            pointer
                        }
                    }
                };
                let stroke = Stroke::new(2.0, Color32::from_rgb(80, 130, 230));
                painter.line_segment([start, pointer], stroke);
                let dir_vec = pointer - start;
                if dir_vec.length_sq() > 1.0 {
                    let dir = dir_vec.normalized();
                    let left = pointer - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
                    let right = pointer - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
                    painter.line_segment([pointer, left], stroke);
                    painter.line_segment([pointer, right], stroke);
                }
            }
        }

        self.draw_debug_animation_overlay(rect, &painter);

        if let Some(p) = self.canvas.selected_place {
            if let Some(idx) = self.place_idx_by_id(p) {
                let place = &mut self.net.places[idx];
                ui.separator();
                ui.label("Выбранная позиция");
                ui.text_edit_singleline(&mut place.name);
            }
        }
        if let Some(t) = self.canvas.selected_transition {
            if let Some(idx) = self.transition_idx_by_id(t) {
                let tr = &mut self.net.transitions[idx];
                ui.separator();
                ui.label("Выбранный переход");
                ui.text_edit_singleline(&mut tr.name);
            }
        }
        if let Some(text_id) = self.canvas.selected_text {
            if let Some(idx) = self.text_idx_by_id(text_id) {
                ui.separator();
                ui.label("Выбранный текст");
                ui.text_edit_singleline(&mut self.text_blocks[idx].text);
            }
        }
        if let Some(frame_id) = self.canvas.selected_frame {
            if let Some(idx) = self.frame_idx_by_id(frame_id) {
                ui.separator();
                ui.label("Выбранная рамка");
                ui.horizontal(|ui| {
                    ui.label("Ширина");
                    ui.add(
                        egui::DragValue::new(&mut self.decorative_frames[idx].width)
                            .speed(1.0)
                            .range(10.0..=5000.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Высота");
                    ui.add(
                        egui::DragValue::new(&mut self.decorative_frames[idx].height)
                            .speed(1.0)
                            .range(10.0..=5000.0),
                    );
                });
            }
        }
    }

    fn update_debug_animation_clock(&mut self, ctx: &egui::Context) {
        if !self.show_debug || !self.debug_animation_enabled {
            self.debug_animation_last_update = None;
            return;
        }
        if self.debug_animation_events.is_empty() {
            self.debug_animation_last_update = None;
            return;
        }
        if !self.debug_animation_step_active && !self.debug_playing {
            self.debug_animation_last_update = None;
            return;
        }
        let now = Instant::now();
        let delta = if let Some(last) = self.debug_animation_last_update {
            now.duration_since(last).as_secs_f64()
        } else {
            0.0
        };
        self.debug_animation_last_update = Some(now);
        let speed = self.debug_animation_playback_speed();
        self.debug_animation_local_clock += delta * speed;
        let duration = self
            .debug_animation_current_duration
            .max(Self::DEBUG_ANIMATION_MIN_DURATION);
        if self.debug_animation_local_clock >= duration {
            self.debug_animation_local_clock = duration;
            if self.debug_playing {
                let visible_len = self
                    .sim_result
                    .as_ref()
                    .map(|result| Self::debug_visible_log_indices(result).len())
                    .unwrap_or(0);
                if self.debug_step + 1 < visible_len {
                    self.debug_step += 1;
                    self.sync_debug_animation_for_step();
                    return;
                }
                self.debug_playing = false;
            }
            self.debug_animation_step_active = false;
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn draw_arc_weight_label(
        ui: &egui::Ui,
        painter: &egui::Painter,
        from: Pos2,
        to: Pos2,
        text: &str,
        color: Color32,
        zoom: f32,
    ) {
        let delta = to - from;
        if delta.length_sq() <= f32::EPSILON {
            return;
        }
        let dir = delta.normalized();
        let perp = Vec2::new(-dir.y, dir.x);
        let label_pos = from + delta * 0.5 + perp * (8.0 * zoom);
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            text,
            egui::TextStyle::Small.resolve(ui.style()),
            color,
        );
    }

    fn draw_debug_animation_overlay(&self, rect: Rect, painter: &egui::Painter) {
        if !self.show_debug || !self.debug_animation_enabled {
            return;
        }
        let event_idx = match self.debug_animation_active_event {
            Some(idx) => idx,
            None => return,
        };
        let event = match self.debug_animation_events.get(event_idx) {
            Some(event) => event,
            None => return,
        };
        let relative = self.debug_animation_relative(event);
        self.draw_debug_animation_event(event, relative, rect, painter);
    }

    fn debug_animation_relative(&self, _event: &DebugAnimationEvent) -> f32 {
        let duration = self
            .debug_animation_current_duration
            .max(Self::DEBUG_ANIMATION_MIN_DURATION);
        if duration <= 0.0 {
            return 0.0;
        }
        (self.debug_animation_local_clock / duration).clamp(0.0, 1.0) as f32
    }

    fn draw_debug_animation_event(
        &self,
        event: &DebugAnimationEvent,
        relative: f32,
        rect: Rect,
        painter: &egui::Painter,
    ) {
        const PRE_FRACTION: f32 = 0.35;
        const TRANSITION_FRACTION: f32 = 0.2;
        const POST_FRACTION: f32 = 1.0 - PRE_FRACTION - TRANSITION_FRACTION;
        let transition = match self.net.transitions.get(event.transition_idx) {
            Some(tr) => tr,
            None => return,
        };
        let tr_pos = self.world_to_screen(rect, transition.pos);
        let tr_dims = Self::transition_dimensions(transition.size) * self.canvas.zoom;
        let tr_rect = Rect::from_min_size(tr_pos, tr_dims);
        let tr_center = tr_rect.center();
        let entry_color = event.entry_color;
        let exit_color = event.exit_color;
        let transition_token_color = entry_color;
        let token_radius = 4.0 * self.canvas.zoom;
        let token_spacing = token_radius * 2.2;

        if relative < PRE_FRACTION {
            if self.debug_arc_animation {
                let progress = (relative / PRE_FRACTION).clamp(0.0, 1.0);
                self.draw_debug_animation_tokens_along_arcs(
                    &event.pre_arcs,
                    rect,
                    painter,
                    tr_rect,
                    tr_center,
                    progress,
                    token_radius,
                    token_spacing,
                    true,
                    entry_color,
                );
            }
            return;
        }
        if relative < PRE_FRACTION + TRANSITION_FRACTION {
            let progress = ((relative - PRE_FRACTION) / TRANSITION_FRACTION).clamp(0.0, 1.0);
            let count = event
                .pre_arcs
                .iter()
                .map(|arc| arc.weight as usize)
                .sum::<usize>()
                .max(1)
                .min(4);
            let angle_offset = progress * std::f32::consts::TAU;
            let radius = token_spacing * 0.35;
            for i in 0..count {
                let angle = (i as f32) * (std::f32::consts::TAU / count as f32) + angle_offset;
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                painter.circle_filled(tr_center + offset, token_radius, transition_token_color);
            }
            return;
        }
        let post_progress =
            ((relative - PRE_FRACTION - TRANSITION_FRACTION) / POST_FRACTION).clamp(0.0, 1.0);
        if self.debug_arc_animation {
            self.draw_debug_animation_tokens_along_arcs(
                &event.post_arcs,
                rect,
                painter,
                tr_rect,
                tr_center,
                post_progress,
                token_radius,
                token_spacing,
                false,
                exit_color,
            );
        }
    }

    fn draw_debug_animation_tokens_along_arcs(
        &self,
        arcs: &[DebugAnimationArc],
        rect: Rect,
        painter: &egui::Painter,
        tr_rect: Rect,
        tr_center: Pos2,
        progress: f32,
        token_radius: f32,
        token_spacing: f32,
        toward_transition: bool,
        token_color: Color32,
    ) {
        for arc in arcs {
            if arc.weight == 0 {
                continue;
            }
            let place = match self.net.places.get(arc.place_idx) {
                Some(place) => place,
                None => continue,
            };
            let place_center = self.world_to_screen(rect, place.pos);
            let place_radius = Self::place_radius(place.size) * self.canvas.zoom;
            let dir = if toward_transition {
                Self::normalized_direction(tr_center - place_center)
            } else {
                Self::normalized_direction(place_center - tr_center)
            };
            let (start, end) = if toward_transition {
                (
                    place_center + dir * place_radius,
                    Self::rect_border_point(tr_rect, -dir),
                )
            } else {
                (
                    Self::rect_border_point(tr_rect, dir),
                    place_center - dir * place_radius,
                )
            };
            let perp = Vec2::new(-dir.y, dir.x);
            let count = (arc.weight as usize).min(3).max(1);
            let offset_base = (count as f32 - 1.0) * 0.5;
            let travel = start + (end - start) * progress;
            for i in 0..count {
                let offset = perp * token_spacing * (i as f32 - offset_base);
                let color = arc.token_colors.get(i).copied().unwrap_or(token_color);
                painter.circle_filled(travel + offset, token_radius, color);
            }
        }
    }

    fn normalized_direction(delta: Vec2) -> Vec2 {
        if delta.length_sq() < f32::EPSILON {
            Vec2::X
        } else {
            delta.normalized()
        }
    }

    fn draw_markov_place_arcs(&self, rect: Rect, painter: &egui::Painter, ui: &egui::Ui) {
        if self.markov_place_arcs.is_empty() {
            return;
        }
        let color = Color32::from_rgb(50, 130, 200);
        let stroke_width = 1.5 * self.canvas.zoom.clamp(0.6, 1.4);
        for arc in &self.markov_place_arcs {
            let from_idx = match self.place_idx_by_id(arc.from_place_id) {
                Some(idx) => idx,
                None => continue,
            };
            let from_center = self.world_to_screen(rect, self.net.places[from_idx].pos);
            let from_radius = Self::place_radius(self.net.places[from_idx].size) * self.canvas.zoom;
            let arrow = if let Some(to_id) = arc.to_place_id {
                let to_idx = match self.place_idx_by_id(to_id) {
                    Some(idx) => idx,
                    None => continue,
                };
                let to_center = self.world_to_screen(rect, self.net.places[to_idx].pos);
                let to_radius = Self::place_radius(self.net.places[to_idx].size) * self.canvas.zoom;
                let delta = to_center - from_center;
                if delta.length_sq() <= f32::EPSILON {
                    continue;
                }
                let dir = delta.normalized();
                let start = from_center + dir * from_radius;
                let end = to_center - dir * to_radius;
                let control =
                    Self::markov_curve_control(start, end, from_idx, to_idx, self.canvas.zoom);
                painter.add(epaint::QuadraticBezierShape {
                    points: [start, control, end],
                    stroke: Stroke::new(stroke_width, color).into(),
                    fill: Color32::TRANSPARENT.into(),
                    closed: false,
                });
                let tangent = Self::normalized_or_zero(end - control);
                self.draw_markov_arrow_head(painter, end, tangent, stroke_width, color);
                Some((start, end, tangent))
            } else {
                let dir = Vec2::Y;
                let start = from_center + dir * from_radius;
                let end = start + dir * (28.0 * self.canvas.zoom);
                painter.line_segment([start, end], Stroke::new(stroke_width, color));
                self.draw_markov_arrow_head(painter, end, dir, stroke_width, color);
                Some((start, end, dir))
            };
            if let Some((start, end, dir)) = arrow {
                let mid = Pos2::new((start.x + end.x) * 0.5, (start.y + end.y) * 0.5);
                let label_offset = Vec2::new(-dir.y, dir.x) * (6.0 * self.canvas.zoom);
                painter.text(
                    mid + label_offset,
                    egui::Align2::CENTER_CENTER,
                    format!("{:.1}%", (arc.probability * 100.0).clamp(0.0, 999.9)),
                    egui::TextStyle::Small.resolve(ui.style()),
                    Color32::from_rgb(30, 30, 30),
                );
            }
        }
    }

    fn draw_markov_arrow_head(
        &self,
        painter: &egui::Painter,
        tip: Pos2,
        dir: Vec2,
        stroke_width: f32,
        color: Color32,
    ) {
        let arrow_size = stroke_width * 3.0;
        let perp = Vec2::new(-dir.y, dir.x);
        let left = tip - dir * arrow_size + perp * (arrow_size * 0.4);
        let right = tip - dir * arrow_size - perp * (arrow_size * 0.4);
        painter.line_segment([tip, left], Stroke::new(stroke_width, color));
        painter.line_segment([tip, right], Stroke::new(stroke_width, color));
    }

    fn markov_curve_control(
        start: Pos2,
        end: Pos2,
        from_idx: usize,
        to_idx: usize,
        zoom: f32,
    ) -> Pos2 {
        let mid = Pos2::new((start.x + end.x) * 0.5, (start.y + end.y) * 0.5);
        let delta = end - start;
        let perp = Self::normalized_or_zero(Vec2::new(-delta.y, delta.x));
        let sign = if from_idx <= to_idx { 1.0 } else { -1.0 };
        let magnitude = 12.0 * zoom + (delta.length() * 0.15);
        mid + perp * (magnitude * sign)
    }

    fn normalized_or_zero(delta: Vec2) -> Vec2 {
        if delta.length_sq() < f32::EPSILON {
            Vec2::X
        } else {
            delta.normalized()
        }
    }
}


# src\ui\app\petri_app\clipboard\copy_selected_objects.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn copy_selected_objects(&mut self) {
        let mut place_ids = self.collect_selected_place_ids();
        let mut transition_ids = self.collect_selected_transition_ids();
        let text_ids = self.collect_selected_text_ids();

        // Fallback: if nothing is selected on canvas, allow copying currently opened properties target.
        if place_ids.is_empty() && transition_ids.is_empty() && text_ids.is_empty() {
            if self.show_place_props {
                if let Some(pid) = self.place_props_id {
                    place_ids.push(pid);
                }
            } else if self.show_transition_props {
                if let Some(tid) = self.transition_props_id {
                    transition_ids.push(tid);
                }
            }
        }

        if place_ids.is_empty() && transition_ids.is_empty() && text_ids.is_empty() {
            self.status_hint = Some("Нечего копировать: нет выделения".to_string());
            return;
        }

        let place_set: HashSet<u64> = place_ids.iter().copied().collect();
        let transition_set: HashSet<u64> = transition_ids.iter().copied().collect();
        let pmap = self.net.place_index_map();
        let tmap = self.net.transition_index_map();

        let mut copied_places = Vec::new();
        for pid in &place_ids {
            let Some(&idx) = pmap.get(pid) else {
                continue;
            };
            copied_places.push(CopiedPlace {
                place: self.net.places[idx].clone(),
                m0: self.net.tables.m0.get(idx).copied().unwrap_or(0),
                mo: self.net.tables.mo.get(idx).copied().unwrap_or(None),
                mz: self.net.tables.mz.get(idx).copied().unwrap_or(0.0),
            });
        }

        let mut copied_transitions = Vec::new();
        for tid in &transition_ids {
            let Some(&idx) = tmap.get(tid) else {
                continue;
            };
            copied_transitions.push(CopiedTransition {
                transition: self.net.transitions[idx].clone(),
                mpr: self.net.tables.mpr.get(idx).copied().unwrap_or(0),
            });
        }

        let mut copied_texts = Vec::new();
        for text_id in &text_ids {
            if let Some(idx) = self.text_idx_by_id(*text_id) {
                copied_texts.push(CopiedTextBlock {
                    pos: self.text_blocks[idx].pos,
                    text: self.text_blocks[idx].text.clone(),
                    font_name: self.text_blocks[idx].font_name.clone(),
                    font_size: self.text_blocks[idx].font_size,
                    color: self.text_blocks[idx].color,
                });
            }
        }

        let mut copied_arcs = Vec::new();
        let in_sel = |n: NodeRef| match n {
            NodeRef::Place(id) => place_set.contains(&id),
            NodeRef::Transition(id) => transition_set.contains(&id),
        };

        for arc in &self.net.arcs {
            if in_sel(arc.from) && in_sel(arc.to) {
                copied_arcs.push(CopiedArc {
                    from: arc.from,
                    to: arc.to,
                    weight: arc.weight,
                    color: arc.color,
                    visible: arc.visible,
                    show_weight: arc.show_weight,
                });
            }
        }

        let mut copied_inhibitors = Vec::new();
        for inh in &self.net.inhibitor_arcs {
            if place_set.contains(&inh.place_id) && transition_set.contains(&inh.transition_id) {
                copied_inhibitors.push(CopiedInhibitorArc {
                    place_id: inh.place_id,
                    transition_id: inh.transition_id,
                    threshold: inh.threshold,
                    color: inh.color,
                    visible: inh.visible,
                    show_weight: inh.show_weight,
                });
            }
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        for p in &copied_places {
            min_x = min_x.min(p.place.pos[0]);
            min_y = min_y.min(p.place.pos[1]);
        }
        for t in &copied_transitions {
            min_x = min_x.min(t.transition.pos[0]);
            min_y = min_y.min(t.transition.pos[1]);
        }
        for t in &copied_texts {
            min_x = min_x.min(t.pos[0]);
            min_y = min_y.min(t.pos[1]);
        }
        if !min_x.is_finite() || !min_y.is_finite() {
            min_x = self.canvas.cursor_world[0];
            min_y = self.canvas.cursor_world[1];
        }

        let copied_count = place_ids.len()
            + transition_ids.len()
            + text_ids.len()
            + copied_arcs.len()
            + copied_inhibitors.len();
        let clip = CopyBuffer {
            origin: [min_x, min_y],
            places: copied_places,
            transitions: copied_transitions,
            arcs: copied_arcs,
            inhibitors: copied_inhibitors,
            texts: copied_texts,
        };
        self.write_copy_buffer_to_system_clipboard(&clip);
        self.clipboard = Some(clip);
        // Keep first paste visibly offset from original selection.
        self.paste_serial = 1;
        self.status_hint = Some(format!("Скопировано объектов: {copied_count}"));
    }
}


# src\ui\app\petri_app\clipboard\mod.rs
﻿use super::*;

mod copy_selected_objects;
mod paste_copied_objects;
mod read_copy_buffer_from_system_clipboard;
mod write_copy_buffer_to_system_clipboard;


# src\ui\app\petri_app\clipboard\paste_copied_objects.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn paste_copied_objects(&mut self) {
        if let Some(ext) = self.read_copy_buffer_from_system_clipboard() {
            self.clipboard = Some(ext);
        }
        let Some(buf) = self.clipboard.clone() else {
            self.status_hint = Some("Буфер пуст".to_string());
            return;
        };
        if buf.places.is_empty() && buf.transitions.is_empty() && buf.texts.is_empty() {
            self.status_hint = Some("Буфер пуст".to_string());
            return;
        }
        self.push_undo_snapshot();

        let base = if self.canvas.cursor_valid {
            self.snapped_world(self.canvas.cursor_world)
        } else {
            self.snapped_world(buf.origin)
        };
        let step = self.grid_step_world();
        let delta = self.paste_serial as f32 * step;
        let offset = [delta, delta];

        let mut place_map = HashMap::<u64, u64>::new();
        let mut transition_map = HashMap::<u64, u64>::new();

        for cp in &buf.places {
            let rel = [
                cp.place.pos[0] - buf.origin[0],
                cp.place.pos[1] - buf.origin[1],
            ];
            let pos =
                self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let before_max = self.net.places.iter().map(|p| p.id).max().unwrap_or(0);
            self.net.add_place(pos);
            let new_id = self.net.places.iter().map(|p| p.id).max().unwrap_or(0);
            if new_id <= before_max {
                continue;
            }
            place_map.insert(cp.place.id, new_id);

            if let Some(idx) = self.place_idx_by_id(new_id) {
                let mut place = cp.place.clone();
                place.id = new_id;
                place.pos = pos;
                self.net.places[idx] = place;

                self.net.tables.m0[idx] = cp.m0;
                self.net.tables.mo[idx] = cp.mo;
                self.net.tables.mz[idx] = cp.mz;

                if Self::parse_place_auto_index(&cp.place.name).is_some()
                    || cp.place.name.trim().is_empty()
                {
                    self.net.places[idx].name.clear();
                    self.assign_auto_name_for_place(new_id);
                } else {
                    let desired = self.net.places[idx].name.clone();
                    self.net.places[idx].name = self.ensure_unique_place_name(&desired, new_id);
                }
            }
        }

        for ct in &buf.transitions {
            let rel = [
                ct.transition.pos[0] - buf.origin[0],
                ct.transition.pos[1] - buf.origin[1],
            ];
            let pos =
                self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let before_max = self.net.transitions.iter().map(|t| t.id).max().unwrap_or(0);
            self.net.add_transition(pos);
            let new_id = self.net.transitions.iter().map(|t| t.id).max().unwrap_or(0);
            if new_id <= before_max {
                continue;
            }
            transition_map.insert(ct.transition.id, new_id);

            if let Some(idx) = self.transition_idx_by_id(new_id) {
                let mut tr = ct.transition.clone();
                tr.id = new_id;
                tr.pos = pos;
                self.net.transitions[idx] = tr;
                self.net.tables.mpr[idx] = ct.mpr;

                if Self::parse_transition_auto_index(&ct.transition.name).is_some()
                    || ct.transition.name.trim().is_empty()
                {
                    self.net.transitions[idx].name.clear();
                    self.assign_auto_name_for_transition(new_id);
                } else {
                    let desired = self.net.transitions[idx].name.clone();
                    self.net.transitions[idx].name =
                        self.ensure_unique_transition_name(&desired, new_id);
                }
            }
        }

        let mut new_text_ids = Vec::new();
        for tt in &buf.texts {
            let rel = [tt.pos[0] - buf.origin[0], tt.pos[1] - buf.origin[1]];
            let pos =
                self.snapped_world([base[0] + rel[0] + offset[0], base[1] + rel[1] + offset[1]]);

            let id = self.next_text_id;
            self.next_text_id = self.next_text_id.saturating_add(1);
            self.text_blocks.push(CanvasTextBlock {
                id,
                pos,
                text: tt.text.clone(),
                font_name: tt.font_name.clone(),
                font_size: tt.font_size,
                color: tt.color,
            });
            new_text_ids.push(id);
        }

        for arc in &buf.arcs {
            let remap = |n: NodeRef| -> Option<NodeRef> {
                match n {
                    NodeRef::Place(id) => place_map.get(&id).copied().map(NodeRef::Place),
                    NodeRef::Transition(id) => {
                        transition_map.get(&id).copied().map(NodeRef::Transition)
                    }
                }
            };
            let (Some(from), Some(to)) = (remap(arc.from), remap(arc.to)) else {
                continue;
            };
            self.net.add_arc(from, to, arc.weight);
            if let Some(last) = self.net.arcs.last_mut() {
                last.color = arc.color;
                last.visible = arc.visible;
                last.show_weight = arc.show_weight;
            }
        }
        for inh in &buf.inhibitors {
            let (Some(&pid), Some(&tid)) = (
                place_map.get(&inh.place_id),
                transition_map.get(&inh.transition_id),
            ) else {
                continue;
            };
            self.net.add_inhibitor_arc(pid, tid, inh.threshold);
            if let Some(last) = self.net.inhibitor_arcs.last_mut() {
                last.color = inh.color;
                last.visible = inh.visible;
                last.show_weight = inh.show_weight;
            }
        }

        self.clear_selection();
        self.canvas.selected_places = place_map.values().copied().collect();
        self.canvas.selected_transitions = transition_map.values().copied().collect();
        self.canvas.selected_text = new_text_ids.last().copied();

        self.paste_serial = self.paste_serial.saturating_add(1);
        let pasted_count = place_map.len() + transition_map.len() + new_text_ids.len();
        self.status_hint = Some(format!("Вставлено объектов: {pasted_count}"));
    }
}


# src\ui\app\petri_app\clipboard\read_copy_buffer_from_system_clipboard.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn read_copy_buffer_from_system_clipboard(&self) -> Option<CopyBuffer> {
        let mut clipboard = arboard::Clipboard::new().ok()?;
        let text = clipboard.get_text().ok()?;
        // Guard against accidental huge clipboard payloads that can freeze UI on parse.
        if text.len() > 4 * 1024 * 1024 {
            return None;
        }
        let payload = text.strip_prefix(Self::CLIPBOARD_PREFIX)?;
        let parsed: ClipboardPayload = serde_json::from_str(payload).ok()?;
        Some(parsed.buffer)
    }
}


# src\ui\app\petri_app\clipboard\write_copy_buffer_to_system_clipboard.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn write_copy_buffer_to_system_clipboard(&mut self, buf: &CopyBuffer) {
        let payload = ClipboardPayload {
            version: 1,
            buffer: buf.clone(),
        };
        let Ok(json) = serde_json::to_string(&payload) else {
            return;
        };
        let text = format!("{}{}", Self::CLIPBOARD_PREFIX, json);
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }
}


# src\ui\app\petri_app\drawing\draw_arc_properties.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_arc_properties(&mut self, ctx: &egui::Context) {
        if !self.show_arc_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_arc
            .or_else(|| self.canvas.selected_arcs.last().copied())
        {
            self.arc_props_id = Some(id);
        }
        if let Some(arc_id) = self.arc_props_id {
            let title = self.tr("Свойства дуги", "Arc Properties").to_string();
            self.show_arc_props = self.draw_arc_props_window(ctx, arc_id, title);
        } else {
            self.show_arc_props = false;
        }
    }
}


# src\ui\app\petri_app\drawing\draw_arc_props_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_arc_props_window(
        &mut self,
        ctx: &egui::Context,
        arc_id: u64,
        title: String,
    ) -> bool {
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        #[derive(Clone, Copy)]
        enum SelectedArc {
            Regular(usize),
            Inhibitor(usize),
        }

        let variant = if let Some(idx) = self.arc_idx_by_id(arc_id) {
            SelectedArc::Regular(idx)
        } else if let Some(idx) = self.inhibitor_arc_idx_by_id(arc_id) {
            SelectedArc::Inhibitor(idx)
        } else {
            return false;
        };

        let mut weight = match variant {
            SelectedArc::Regular(idx) => self.net.arcs[idx].weight,
            SelectedArc::Inhibitor(_) => 1,
        };
        let mut threshold = match variant {
            SelectedArc::Inhibitor(idx) => self.net.inhibitor_arcs[idx].threshold,
            SelectedArc::Regular(_) => 1,
        };
        let mut color = match variant {
            SelectedArc::Regular(idx) => self.net.arcs[idx].color,
            SelectedArc::Inhibitor(idx) => self.net.inhibitor_arcs[idx].color,
        };
        let mut show_weight = match variant {
            SelectedArc::Regular(idx) => self.net.arcs[idx].show_weight,
            SelectedArc::Inhibitor(idx) => self.net.inhibitor_arcs[idx].show_weight,
        };
        let mut is_inhibitor = matches!(variant, SelectedArc::Inhibitor(_));
        let can_be_inhibitor = match variant {
            SelectedArc::Regular(idx) => {
                Self::arc_place_transition_pair(self.net.arcs[idx].from, self.net.arcs[idx].to)
                    .is_some()
            }
            SelectedArc::Inhibitor(_) => true,
        };
        if !can_be_inhibitor && is_inhibitor {
            is_inhibitor = false;
        }

        let color_combo = |ui: &mut egui::Ui, value: &mut NodeColor| {
            egui::ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(Self::node_color_text(*value, is_ru))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        value,
                        NodeColor::Default,
                        Self::node_color_text(NodeColor::Default, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Blue,
                        Self::node_color_text(NodeColor::Blue, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Red,
                        Self::node_color_text(NodeColor::Red, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Green,
                        Self::node_color_text(NodeColor::Green, is_ru),
                    );
                    ui.selectable_value(
                        value,
                        NodeColor::Yellow,
                        Self::node_color_text(NodeColor::Yellow, is_ru),
                    );
                });
        };

        let mut open = true;
        egui::Window::new(title)
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("arc_props_window"))
            .resizable(true)
            .default_size(egui::vec2(420.0, 440.0))
            .min_size(egui::vec2(320.0, 320.0))
            .open(&mut open)
            .show(ctx, |ui| {
                let mut corrected_inputs = false;
                ui.label(format!("ID: A{}", arc_id));
                ui.separator();
                ui.add_enabled_ui(can_be_inhibitor, |ui| {
                    ui.checkbox(&mut is_inhibitor, t("Ингибиторная дуга", "Inhibitor arc"));
                });
                if matches!(variant, SelectedArc::Regular(_)) && !can_be_inhibitor {
                    ui.label(t(
                        "Ингибиторная дуга должна начинаться с позиции и заканчиваться на переходе",
                        "Inhibitor arcs must start at a position and end at a transition",
                    ));
                }
                corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                corrected_inputs |= sanitize_u32(&mut weight, 1, u32::MAX);
                let weight_label = t("Кратность (вес)", "Weight");
                let show_weight_label =
                    t("Показывать кратность (вес)", "Show multiplicity (weight)");
                if is_inhibitor {
                    ui.horizontal(|ui| {
                        ui.label(t("Порог", "Threshold"));
                        if ui
                            .add(egui::DragValue::new(&mut threshold).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(weight_label);
                        if ui
                            .add(egui::DragValue::new(&mut weight).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut weight, 1, u32::MAX);
                        }
                    });
                }
                if ui.checkbox(&mut show_weight, show_weight_label).changed() {
                    // flag only
                }
                ui.horizontal(|ui| {
                    ui.label(t("Цвет", "Color"));
                    color_combo(ui, &mut color);
                });
                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
            });

        let new_weight = weight.max(1);
        let new_threshold = threshold.max(1);
        let mut should_rebuild = false;
        match variant {
            SelectedArc::Regular(idx) => {
                if is_inhibitor {
                    if let Some((place_id, transition_id)) = Self::arc_place_transition_pair(
                        self.net.arcs[idx].from,
                        self.net.arcs[idx].to,
                    ) {
                        let arc = self.net.arcs.remove(idx);
                        self.net.inhibitor_arcs.push(crate::model::InhibitorArc {
                            id: arc.id,
                            place_id,
                            transition_id,
                            threshold: new_threshold,
                            color,
                            visible: arc.visible,
                            show_weight,
                        });
                        self.canvas.selected_arc = Some(arc.id);
                        if !self.canvas.selected_arcs.contains(&arc.id) {
                            self.canvas.selected_arcs.push(arc.id);
                        }
                        should_rebuild = true;
                    }
                } else {
                    let arc = &mut self.net.arcs[idx];
                    if arc.weight != new_weight {
                        should_rebuild = true;
                    }
                    arc.weight = new_weight;
                    arc.color = color;
                    arc.show_weight = show_weight;
                }
            }
            SelectedArc::Inhibitor(idx) => {
                if !is_inhibitor {
                    let inh = self.net.inhibitor_arcs.remove(idx);
                    self.net.arcs.push(crate::model::Arc {
                        id: inh.id,
                        from: NodeRef::Place(inh.place_id),
                        to: NodeRef::Transition(inh.transition_id),
                        weight: new_weight,
                        color,
                        visible: inh.visible,
                        show_weight,
                    });
                    self.canvas.selected_arc = Some(inh.id);
                    if !self.canvas.selected_arcs.contains(&inh.id) {
                        self.canvas.selected_arcs.push(inh.id);
                    }
                    should_rebuild = true;
                } else {
                    let inh = &mut self.net.inhibitor_arcs[idx];
                    if inh.threshold != new_threshold {
                        should_rebuild = true;
                    }
                    inh.threshold = new_threshold;
                    inh.color = color;
                    inh.show_weight = show_weight;
                }
            }
        }

        if should_rebuild {
            self.net.rebuild_matrices_from_arcs();
        }

        open
    }
}


# src\ui\app\petri_app\drawing\draw_atf_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_atf_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_atf;
        egui::Window::new("ATF")
            .constrained_to_viewport(ctx)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Левая область");
                        ui.horizontal(|ui| {
                            ui.label("P:");
                            ui.add(
                                egui::DragValue::new(&mut self.atf_selected_place).range(0..=10000),
                            );
                            if ui.button("OK").clicked() {
                                self.atf_text = generate_atf(
                                    &self.net,
                                    self.atf_selected_place
                                        .min(self.net.places.len().saturating_sub(1)),
                                );
                            }
                        });
                        if ui.button("Сгенерировать ATF").clicked() {
                            self.atf_text = generate_atf(
                                &self.net,
                                self.atf_selected_place
                                    .min(self.net.places.len().saturating_sub(1)),
                            );
                        }
                        if ui.button("Открыть ATF файл").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("ATF", &["atf", "txt"])
                                .pick_file()
                            {
                                match fs::read_to_string(&path) {
                                    Ok(text) => self.atf_text = text,
                                    Err(e) => self.last_error = Some(e.to_string()),
                                }
                            }
                        }
                    });
                    ui.separator();
                    ui.add(
                        egui::TextEdit::multiline(&mut self.atf_text)
                            .desired_rows(30)
                            .desired_width(700.0),
                    );
                });
            });
        self.show_atf = open;
    }
}


# src\ui\app\petri_app\drawing\draw_debug_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_debug_window(&mut self, ctx: &egui::Context) {
        if !self.show_debug {
            return;
        }
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = self.show_debug;
        egui::Window::new(t("Режим отладки", "Debug Mode"))
            .constrained_to_viewport(ctx)
            .open(&mut open)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.clone() else {
                    ui.label(t("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };
                let visible_steps = Self::debug_visible_log_indices(&result);
                let steps = visible_steps.len();
                if steps == 0 {
                    ui.label(t("Пустой журнал.", "Empty log."));
                    return;
                }
                if self.debug_step >= steps {
                    self.debug_step = steps - 1;
                }

                ui.horizontal(|ui| {
                    if ui.button("<<").clicked() {
                        self.debug_playing = false;
                        self.debug_animation_last_update = None;
                        self.debug_step = self.debug_step.saturating_sub(1);
                        self.sync_debug_animation_for_step();
                    }
                    if ui
                        .button(if self.debug_playing {
                            t("Пауза", "Pause")
                        } else {
                            t("Пуск", "Play")
                        })
                        .clicked()
                    {
                        if self.debug_playing {
                            self.debug_playing = false;
                        } else {
                            self.debug_playing = true;
                        }
                        self.debug_animation_last_update = None;
                    }
                    if ui.button(">>").clicked() {
                        self.debug_playing = false;
                        self.debug_animation_last_update = None;
                        self.debug_step = (self.debug_step + 1).min(steps - 1);
                        self.sync_debug_animation_for_step();
                    }
                    ui.label(t("Скорость (мс сим.сек):", "Speed (ms per sim sec):"));
                    ui.add(egui::DragValue::new(&mut self.debug_interval_ms).range(50..=5_000));
                });

                let slider_response = ui.add(
                    egui::Slider::new(&mut self.debug_step, 0..=steps - 1).text(t("Шаг", "Step")),
                );
                if slider_response.changed() {
                    self.debug_playing = false;
                    self.debug_animation_last_update = None;
                    self.sync_debug_animation_for_step();
                }
                if self.debug_playing && steps > 1 {
                    let interval = Duration::from_millis(self.debug_interval_ms.max(1));
                    let now = Instant::now();
                    match self.debug_animation_last_update {
                        Some(last) => {
                            if now.duration_since(last) >= interval {
                                if self.debug_step < steps - 1 {
                                    self.debug_step += 1;
                                    self.sync_debug_animation_for_step();
                                } else {
                                    self.debug_playing = false;
                                }
                                self.debug_animation_last_update = Some(now);
                            }
                        }
                        None => {
                            self.debug_animation_last_update = Some(now);
                        }
                    }
                }
                if self.debug_playing {
                    ctx.request_repaint_after(Duration::from_millis(16));
                }
                let animation_response = ui.checkbox(
                    &mut self.debug_animation_enabled,
                    t("Включить анимацию", "Enable animation"),
                );
                if animation_response.changed() {
                    self.debug_arc_animation = self.debug_animation_enabled;
                    self.debug_animation_last_update = None;
                    if self.debug_animation_enabled {
                        self.refresh_debug_animation_state();
                    } else {
                        self.debug_playing = false;
                        self.clear_debug_animation_state();
                    }
                }
                if self.debug_animation_enabled {
                    if self.debug_animation_events.is_empty() {
                        ui.label(t(
                            "Сначала запустите симуляцию, чтобы увидеть анимацию.",
                            "Run a simulation first to see the animation.",
                        ));
                    }
                }
                if let Some(&log_idx) = visible_steps.get(self.debug_step) {
                    if let Some(entry) = result.logs.get(log_idx) {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(t("Текущее время", "Current time"));
                            ui.label("t");
                            ui.label(format!("= {:.3}", entry.time));
                        });
                        ui.label(format!(
                            "{}: {}",
                            t("Переход", "Transition"),
                            entry
                                .fired_transition
                                .map(|i| format!("T{}", i + 1))
                                .unwrap_or_else(|| "-".to_string())
                        ));
                        egui::Grid::new("debug_marking_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                for (idx, marking) in entry.marking.iter().enumerate() {
                                    ui.label(format!("P{}", idx + 1));
                                    ui.label(marking.to_string());
                                    ui.end_row();
                                }
                            });
                    }
                }
            });
        self.show_debug = open;
    }
}


# src\ui\app\petri_app\drawing\draw_help_controls.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_help_controls(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_controls;
        egui::Window::new("Help: Помощь по управлению")
            .constrained_to_viewport(ctx)
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading("Основные кнопки и комбинации");
                ui.separator();
                ui.label("ЛКМ: создать/выбрать элемент (в зависимости от активного инструмента)");
                ui.label("СКМ + перетаскивание: двигать рабочую область");
                ui.label("Delete: удалить выделенное");
                ui.separator();
                ui.label("Ctrl+N: новый файл");
                ui.label("Ctrl+O: открыть файл");
                ui.label("Ctrl+S: сохранить файл");
                ui.label("Ctrl+C: копировать выделенное");
                ui.label("Ctrl+V: вставить");
                ui.label("Ctrl+Z: отменить последнее действие");
                ui.label("Ctrl+Q: выход");
                ui.label("Ctrl + колесо: изменить масштаб графа");
            });
        self.show_help_controls = open;
    }
}


# src\ui\app\petri_app\drawing\draw_help_development.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_help_development(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help_development;
        egui::Window::new("Help: Разработка")
            .constrained_to_viewport(ctx)
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Информация о приложении");
                ui.separator();
                ui.label(egui::RichText::new(format!("Версия: {}", env!("CARGO_PKG_VERSION"))).size(20.0));
                ui.label(egui::RichText::new("Разработчик: Вайбкод + вылеты NetStar").size(18.0));
                ui.separator();
                ui.label("Редактор сетей Петри с совместимостью с форматом NetStar и инструментами имитации.");
            });
        self.show_help_development = open;
    }
}


# src\ui\app\petri_app\drawing\draw_layout.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_layout(&mut self, ctx: &egui::Context) {
        if self.show_table_view && self.table_fullscreen {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.draw_table_workspace(ui);
            });
            return;
        }

        if self.layout_mode == LayoutMode::Minimized {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Все окна свернуты");
            });
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.layout_mode {
            LayoutMode::Cascade => {
                if self.show_graph_view {
                    self.draw_graph_view(ui);
                }
                if self.show_table_view {
                    self.draw_table_workspace(ui);
                }
            }
            LayoutMode::TileHorizontal => {
                if !self.show_table_view {
                    if self.show_graph_view {
                        self.draw_graph_view(ui);
                    }
                    return;
                }
                ui.vertical(|ui| {
                    if self.show_graph_view {
                        ui.allocate_ui_with_layout(
                            Vec2::new(ui.available_width(), ui.available_height() * 0.55),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| self.draw_graph_view(ui),
                        );
                    }
                    ui.separator();
                    self.draw_table_workspace(ui);
                });
            }
            LayoutMode::TileVertical => {
                if !self.show_table_view {
                    if self.show_graph_view {
                        self.draw_graph_view(ui);
                    }
                    return;
                }
                ui.columns(2, |columns| {
                    if self.show_graph_view {
                        self.draw_graph_view(&mut columns[0]);
                    }
                    self.draw_table_workspace(&mut columns[1]);
                });
            }
            LayoutMode::Minimized => {}
        });
    }
}


# src\ui\app\petri_app\drawing\draw_markov_window.rs
use super::*;
use egui::{scroll_area, Color32, RichText, Vec2};

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_markov_window;
        let viewport = ctx.available_rect();
        let max_height = (viewport.height() - 120.0).max(360.0);
        let max_width = (viewport.width() - 120.0).max(360.0);

        egui::Window::new(self.tr("Марковская модель", "Markov model"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("markov_window"))
            .default_size(Vec2::new(520.0, 520.0))
            .min_size(Vec2::new(360.0, 360.0))
            .max_size(Vec2::new(max_width, max_height))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let simulation_ready = self.sim_result.is_some();
                    let mut toggle_changed = false;
                    let markov_checkbox_label =
                        self.tr("включить марковскую модель", "Enable Markov model");
                    let simulation_hint = self.tr(
                        "Сначала запустите симуляцию, чтобы включить марковскую модель",
                        "Run a simulation first to enable the model",
                    );

                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(simulation_ready, |ui| {
                            if ui
                                .checkbox(
                                    &mut self.markov_model_enabled,
                                    markov_checkbox_label.as_ref(),
                                )
                                .changed()
                            {
                                toggle_changed = true;
                            }
                        });

                        if !simulation_ready {
                            ui.colored_label(
                                Color32::from_rgb(190, 40, 40),
                                simulation_hint.as_ref(),
                            );
                        }
                    });

                    if toggle_changed {
                        for place in &mut self.net.places {
                            place.show_markov_model = self.markov_model_enabled;
                        }

                        if self.markov_model_enabled {
                            self.calculate_markov_model();
                        } else {
                            self.markov_place_arcs.clear();
                        }
                    }

                    ui.separator();
                    ui.add_space(6.0);

                    if let Some(chain) = &self.markov_model {
                        self.draw_markov_chain_summary(ui, chain);
                    } else {
                        ui.label(self.tr("Постройте модель", "Build the model"));
                    }

                    if !self.markov_model_enabled {
                        ui.separator();
                        ui.label(self.tr(
                            "Включите флажок выше, чтобы увидеть марковскую модель",
                            "Toggle the checkbox above to display the Markov model",
                        ));
                    }
                });
            });

        self.show_markov_window = open;
    }

    fn draw_markov_chain_summary(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        let stationary = chain.stationary.as_ref().map(|values| values.as_slice());

        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}{}",
                self.tr("Состояний", "States"),
                chain.state_count(),
                if chain.limit_reached {
                    format!(" ({})", self.tr("лимит", "limit reached"))
                } else {
                    String::new()
                }
            ));

            ui.label(format!(
                "{}: {}",
                self.tr("Переходов", "Transitions"),
                chain.transitions.iter().map(|edges| edges.len()).sum::<usize>()
            ));
        });

        ui.separator();
        ui.label(self.tr("Стационарное распределение", "Stationary distribution"));

        if let Some(stationary) = stationary {
            self.draw_markov_stationary_grid(ui, chain, stationary);
        } else {
            ui.label(self.tr(
                "Стационарное распределение не вычислено",
                "Unable to compute stationary",
            ));
        }

        ui.separator();
        self.draw_markov_state_graph(ui, chain);
        self.draw_markov_highlight(ui, chain, stationary);
    }

    fn draw_markov_stationary_grid(
        &self,
        ui: &mut egui::Ui,
        chain: &MarkovChain,
        stationary: &[f64],
    ) {
        if chain.state_count() == 0 {
            ui.label(self.tr("Состояний не найдено", "No states found"));
            return;
        }

        let available = ui.available_width();
        let marking_width = Self::markov_marking_column_width(available);

        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("Состояние", "State")).strong());
            ui.allocate_ui(Vec2::new(marking_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Маркировка", "Marking")).strong());
            });
            ui.label(RichText::new("π").strong());
        });

        egui::ScrollArea::vertical()
            .id_source("markov_stationary_distribution")
            .max_height(360.0)
            .auto_shrink([false, false])
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                for (idx, value) in stationary.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("S{}", idx + 1));
                        ui.allocate_ui(Vec2::new(marking_width, 0.0), |ui| {
                            self.draw_state_marking_table(ui, &chain.states[idx], idx);
                        });
                        ui.label(format!("{:.6}", value));
                    });
                    ui.add_space(6.0);
                }
            });
    }

    fn draw_markov_state_graph(&self, ui: &mut egui::Ui, chain: &MarkovChain) {
        ui.label(self.tr("Граф состояний", "State graph"));

        let available = ui.available_width();
        let transitions_width = Self::markov_transitions_column_width(available);

        ui.horizontal(|ui| {
            ui.label(RichText::new(self.tr("Состояние", "State")).strong());
            ui.allocate_ui(Vec2::new(transitions_width, 0.0), |ui| {
                ui.label(RichText::new(self.tr("Переходы", "Transitions")).strong());
            });
        });

        egui::ScrollArea::vertical()
            .id_source("markov_state_graph")
            .max_height(320.0)
            .auto_shrink([false, false])
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                if chain.transitions.is_empty() {
                    ui.label(self.tr("Переходов не найдено", "No transitions detected"));
                    return;
                }

                for (idx, edges) in chain.transitions.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("S{}", idx + 1));
                        ui.allocate_ui(Vec2::new(transitions_width, 0.0), |ui| {
                            if edges.is_empty() {
                                ui.label(self.tr("Переходов нет", "No transitions"));
                            } else {
                                let total_rate: f64 = edges.iter().map(|(_, rate)| *rate).sum();

                                ui.vertical(|ui| {
                                    for (dest, rate) in edges {
                                        let prob = if total_rate > 0.0 {
                                            (rate / total_rate).clamp(0.0, 1.0)
                                        } else {
                                            0.0
                                        };

                                        ui.add_sized(
                                            [transitions_width, 0.0],
                                            egui::Label::new(format!(
                                                "→ S{} ({:.2})",
                                                dest + 1,
                                                prob
                                            ))
                                            .wrap(),
                                        );
                                    }
                                });
                            }
                        });
                    });
                    ui.add_space(6.0);
                }
            });
    }

    fn draw_markov_highlight(
        &self,
        ui: &mut egui::Ui,
        chain: &MarkovChain,
        stationary: Option<&[f64]>,
    ) {
        let markov_highlight_places = self
            .net
            .places
            .iter()
            .enumerate()
            .filter(|(_, place)| place.markov_highlight)
            .collect::<Vec<_>>();

        if markov_highlight_places.is_empty() {
            ui.separator();
            ui.label(self.tr(
                "Отметьте марковскую метку в свойствах позиции, чтобы увидеть её отображение",
                "Enable the Markov highlight on a place to view its display",
            ));
            return;
        }

        ui.separator();
        ui.label(self.tr("Отображение марковской метки", "Markov highlight display"));

        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());

        egui::ScrollArea::vertical()
            .id_source("markov_place_distribution")
            .max_height(320.0)
            .scroll_bar_visibility(scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                for (place_idx, place) in &markov_highlight_places {
                    ui.group(|ui| {
                        let place_label = if place.name.is_empty() {
                            format!("P{}", place.id)
                        } else {
                            place.name.clone()
                        };

                        ui.label(format!(
                            "{}: {} (P{})",
                            self.tr("Позиция", "Place"),
                            place_label,
                            place.id
                        ));

                        if let Some(expected) = expectation
                            .as_ref()
                            .and_then(|values| values.get(*place_idx))
                        {
                            ui.label(format!(
                                "{}: {:.3}",
                                self.tr("Ожидаемое число маркеров", "Expected tokens"),
                                expected
                            ));
                        }

                        let distribution = Self::markov_tokens_distribution(chain, *place_idx);

                        if !distribution.is_empty() {
                            for (count, prob) in distribution.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{} {}",
                                        count,
                                        self.tr("маркеров", "tokens")
                                    ));
                                    ui.label(format!("{:.2}%", prob * 100.0));
                                });
                            }
                        } else if stationary.is_some() {
                            ui.label(self.tr(
                                "Для этой позиции состояния не найдены",
                                "No states found for this place",
                            ));
                        } else {
                            ui.label(self.tr(
                                "Стационарное распределение недоступно",
                                "Stationary distribution unavailable",
                            ));
                        }
                    });

                    ui.add_space(4.0);
                }
            });
    }

    fn draw_state_marking_table(&self, ui: &mut egui::Ui, marking: &[u32], state_idx: usize) {
        const COLUMNS: usize = 2;

        if marking.is_empty() {
            ui.label("—");
            return;
        }

        let rows = (marking.len() + COLUMNS - 1) / COLUMNS;

        egui::Grid::new(format!(
            "state_marking_summary_{}_{}",
            state_idx,
            marking.len()
        ))
        .striped(true)
        .spacing([6.0, 2.0])
        .show(ui, |ui| {
            for row in 0..rows {
                for col in 0..COLUMNS {
                    let idx = row + col * rows;
                    if idx < marking.len() {
                        ui.label(format!("P{}", idx + 1));
                        ui.label(marking[idx].to_string());
                    } else {
                        ui.label(" ");
                        ui.label(" ");
                    }
                }
                ui.end_row();
            }
        });
    }

    fn markov_marking_column_width(available: f32) -> f32 {
        const MIN_WIDTH: f32 = 120.0;
        let max_width = (available * 0.7).max(MIN_WIDTH);
        let width = (available * 0.55).clamp(MIN_WIDTH, max_width);
        width.min(available)
    }

    fn markov_transitions_column_width(available: f32) -> f32 {
        const MIN_WIDTH: f32 = 180.0;
        let max_width = (available * 0.65).max(MIN_WIDTH);
        let width = (available * 0.6).clamp(MIN_WIDTH, max_width);
        width.min(available)
    }
}

# src\ui\app\petri_app\drawing\draw_menu.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(self.tr("Файл", "File"), |ui| {
                    if ui.button("Новый (Ctrl+N)").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Открыть (Ctrl+O)").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.menu_button("Импорт", |ui| {
                        ui.label("Импорт PeSim: TODO");
                    });
                    ui.menu_button("Экспорт", |ui| {
                        if ui.button("Экспорт в NetStar (gpn)").clicked() {
                            self.export_netstar_file();
                            ui.close_menu();
                        }
                    });
                    if ui.button("Сохранить (gpn2) (Ctrl+S)").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Сохранить как (gpn2)").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    if ui.button("Выход").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Опции", |ui| {
                    ui.menu_button("Язык", |ui| {
                        ui.radio_value(&mut self.net.ui.language, Language::Ru, "RU");
                        ui.radio_value(&mut self.net.ui.language, Language::En, "EN");
                    });
                    ui.checkbox(&mut self.net.ui.hide_grid, "Скрыть сетку");
                    ui.checkbox(&mut self.net.ui.snap_to_grid, "Привязка к сетке");
                    ui.checkbox(&mut self.net.ui.colored_petri_nets, "Цветные сети Петри");
                    ui.menu_button("Сбор статистики", |ui| {
                        ui.checkbox(&mut self.net.ui.marker_count_stats, "Статистика маркеров");
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Разработка").clicked() {
                            self.show_help_development = true;
                            ui.close_menu();
                        }
                        if ui.button("Помощь по управлению").clicked() {
                            self.show_help_controls = true;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Окно", |ui| {
                    let options = [
                        (
                            LayoutMode::TileHorizontal,
                            self.tr("Плитка по горизонтали", "Tile horizontal"),
                        ),
                        (
                            LayoutMode::TileVertical,
                            self.tr("Плитка по вертикали", "Tile vertical"),
                        ),
                        (
                            LayoutMode::Minimized,
                            self.tr("Свернуть все", "Minimize all"),
                        ),
                    ];
                    for (mode, label) in options {
                        let selected = self.layout_mode == mode;
                        if ui
                            .add(egui::SelectableLabel::new(selected, label.as_ref()))
                            .clicked()
                        {
                            self.layout_mode = mode;
                        }
                    }
                });

                let markov_available = self.sim_result.is_some();
                ui.add_enabled_ui(markov_available, |ui| {
                    let response = ui
                        .button(self.tr("Марковская модель", "Markov model"))
                        .on_hover_text(self.tr(
                            "Требуется активная симуляция",
                            "Requires an active simulation",
                        ));
                    if response.clicked() {
                        self.calculate_markov_model();
                        self.show_markov_window = true;
                    }
                });

                if ui.button("Структура сети").clicked() {
                    self.show_table_view = !self.show_table_view;
                    if !self.show_table_view {
                        self.table_fullscreen = false;
                    }
                }
                if ui
                    .button(self.tr("Результаты имитации", "Simulation Results"))
                    .clicked()
                {
                    if self.sim_result.is_some() {
                        self.show_results = !self.show_results;
                    } else {
                        self.show_results = false;
                    }
                }
                if ui.button("Proof").clicked() && self.sim_result.is_some() {
                    self.show_proof = true;
                }
                if ui.button(self.tr("Режим отладки", "Debug Mode")).clicked()
                    && self.sim_result.is_some()
                {
                    self.show_debug = true;
                }
                if ui.button("ATF").clicked() {
                    self.show_atf = true;
                }
            });
        });
    }
}


# src\ui\app\petri_app\drawing\draw_netstar_export_validation.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_netstar_export_validation(&mut self, ctx: &egui::Context) {
        if !self.show_netstar_export_validation {
            return;
        }

        let Some(report) = self.netstar_export_validation.clone() else {
            self.clear_netstar_export_validation();
            return;
        };

        let mut open = self.show_netstar_export_validation;
        let target_path = self.pending_netstar_export_path.clone();
        let errors = report.error_count();
        let warnings = report.warning_count();
        let mut do_export = false;
        let mut do_cancel = false;

        egui::Window::new(self.tr("Проверка экспорта", "Export validation"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("netstar_export_validation_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(620.0)
            .show(ctx, |ui| {
                if let Some(path) = &target_path {
                    ui.label(format!("{} {}", self.tr("Файл:", "File:"), path.display()));
                }
                ui.separator();
                ui.label(format!(
                    "{}: {}    {}: {}",
                    self.tr("Ошибки", "Errors"),
                    errors,
                    self.tr("Предупреждения", "Warnings"),
                    warnings
                ));

                if report.is_clean() {
                    ui.colored_label(
                        Color32::from_rgb(0, 128, 0),
                        self.tr("Проблем не найдено.", "No issues found."),
                    );
                } else {
                    ui.label(self.tr(
                        "Нажмите на строку ошибки/предупреждения, чтобы выделить объект в графе.",
                        "Click an issue row to select the related object on the graph.",
                    ));
                    egui::ScrollArea::vertical()
                        .max_height(260.0)
                        .show(ui, |ui| {
                            for issue in &report.errors {
                                let line = format!("[{}] {}", self.tr("Ошибка", "Error"), issue);
                                let response = ui.add(
                                    egui::Label::new(egui::RichText::new(line).color(Color32::RED))
                                        .sense(Sense::click()),
                                );
                                if response.clicked() && !self.select_export_issue_target(issue) {
                                    self.status_hint = Some(
                                        self.tr(
                                            "Не удалось определить объект по строке отчёта.",
                                            "Could not resolve target object from issue row.",
                                        )
                                        .to_string(),
                                    );
                                }
                            }
                            for issue in &report.warnings {
                                let line =
                                    format!("[{}] {}", self.tr("Предупреждение", "Warning"), issue);
                                let response = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(line)
                                            .color(Color32::from_rgb(160, 110, 0)),
                                    )
                                    .sense(Sense::click()),
                                );
                                if response.clicked() {
                                    let _ = self.select_export_issue_target(issue);
                                }
                            }
                        });
                }

                if errors > 0 {
                    ui.separator();
                    ui.colored_label(
                        Color32::RED,
                        self.tr(
                            "Экспорт заблокирован: исправьте ошибки в модели.",
                            "Export blocked: fix model errors first.",
                        ),
                    );
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(self.tr("Отмена", "Cancel")).clicked() {
                        do_cancel = true;
                    }
                    let export_label = if warnings > 0 {
                        self.tr(
                            "Экспортировать с предупреждениями",
                            "Export despite warnings",
                        )
                    } else {
                        self.tr("Экспортировать", "Export")
                    };
                    if ui
                        .add_enabled(errors == 0, egui::Button::new(export_label))
                        .clicked()
                    {
                        do_export = true;
                    }
                });
            });

        if !open {
            do_cancel = true;
        }
        if do_cancel {
            self.clear_netstar_export_validation();
        }
        if do_export {
            self.confirm_netstar_export_from_validation();
        }
    }
}


# src\ui\app\petri_app\drawing\draw_place_properties.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_place_properties(&mut self, ctx: &egui::Context) {
        if !self.show_place_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_place
            .or_else(|| self.canvas.selected_places.last().copied())
        {
            self.place_props_id = Some(id);
        }
        if let Some(place_id) = self.place_props_id {
            let title = self
                .tr("Свойства позиции", "Position Properties")
                .to_string();
            self.show_place_props = self.draw_place_props_window(ctx, place_id, title);
        } else {
            self.show_place_props = false;
        }
    }
}


# src\ui\app\petri_app\drawing\draw_place_props_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_place_props_window(
        &mut self,
        ctx: &egui::Context,
        place_id: u64,
        title: String,
    ) -> bool {
        let Some(place_idx) = self.place_idx_by_id(place_id) else {
            return false;
        };
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };
        let mut open = true;
        egui::Window::new(title)
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("place_props_window"))
            .resizable(true)
            .default_size(egui::vec2(420.0, 520.0))
            .min_size(egui::vec2(320.0, 360.0))
            .open(&mut open)
            .show(ctx, |ui| {
                let mut corrected_inputs = false;
                ui.label(format!("ID: P{}", place_id));
                ui.separator();
                let mut markers = self.net.tables.m0[place_idx];
                corrected_inputs |= sanitize_u32(&mut markers, 0, u32::MAX);
                ui.horizontal(|ui| {
                    ui.label(t("Число маркеров", "Markers"));
                    if ui
                        .add(egui::DragValue::new(&mut markers).range(0..=u32::MAX))
                        .changed()
                    {
                        corrected_inputs |= sanitize_u32(&mut markers, 0, u32::MAX);
                    }
                });
                self.net.tables.m0[place_idx] = markers;

                let mut cap = self.net.tables.mo[place_idx].unwrap_or(0);
                corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                ui.horizontal(|ui| {
                    ui.label(t(
                        "Макс. емкость (0 = без ограничений)",
                        "Capacity (0 = unlimited)",
                    ));
                    if ui
                        .add(egui::DragValue::new(&mut cap).range(0..=u32::MAX))
                        .changed()
                    {
                        corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                    }
                });
                self.net.tables.mo[place_idx] = if cap == 0 { None } else { Some(cap) };

                let mut delay = self.net.tables.mz[place_idx];
                corrected_inputs |= sanitize_f64(&mut delay, 0.0, 10_000.0);
                ui.horizontal(|ui| {
                    ui.label(t("Время задержки (сек)", "Delay (sec)"));
                    if ui
                        .add(
                            egui::DragValue::new(&mut delay)
                                .speed(0.1)
                                .range(0.0..=10_000.0),
                        )
                        .changed()
                    {
                        corrected_inputs |= sanitize_f64(&mut delay, 0.0, 10_000.0);
                    }
                });
                self.net.tables.mz[place_idx] = delay;

                ui.separator();
                ui.label(t("Размер позиции", "Place size"));
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.net.places[place_idx].size,
                        VisualSize::Small,
                        t("Малый", "Small"),
                    );
                    ui.radio_value(
                        &mut self.net.places[place_idx].size,
                        VisualSize::Medium,
                        t("Средний", "Medium"),
                    );
                    ui.radio_value(
                        &mut self.net.places[place_idx].size,
                        VisualSize::Large,
                        t("Большой", "Large"),
                    );
                });

                egui::ComboBox::from_label(t("Положение метки", "Marker label position"))
                    .selected_text(Self::label_pos_text(
                        self.net.places[place_idx].marker_label_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].marker_label_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Положение текста", "Text position"))
                    .selected_text(Self::label_pos_text(
                        self.net.places[place_idx].text_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].text_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Цвет", "Color"))
                    .selected_text(Self::node_color_text(
                        self.net.places[place_idx].color,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Default,
                            t("По умолчанию", "Default"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Blue,
                            t("Синий", "Blue"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Red,
                            t("Красный", "Red"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Green,
                            t("Зеленый", "Green"),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].color,
                            NodeColor::Yellow,
                            t("Желтый", "Yellow"),
                        );
                    });

                ui.separator();
                ui.checkbox(
                    &mut self.net.places[place_idx].marker_color_on_pass,
                    t(
                        "Изменять цвет маркера при прохождении через позицию",
                        "Change marker color when token passes this place",
                    ),
                );
                ui.checkbox(
                    &mut self.net.places[place_idx].input_module,
                    t(
                        "Определить позицию как вход модуля",
                        "Define place as module input",
                    ),
                );
                if self.net.places[place_idx].input_module {
                    ui.horizontal(|ui| {
                        ui.label(t("Номер входа", "Input number"));
                        let mut input_number = self.net.places[place_idx].input_number;
                        corrected_inputs |= sanitize_u32(&mut input_number, 1, u32::MAX);
                        if ui
                            .add(egui::DragValue::new(&mut input_number).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut input_number, 1, u32::MAX);
                        }
                        self.net.places[place_idx].input_number = input_number;
                    });
                    ui.label(t("Описание входа", "Input description"));
                    ui.text_edit_singleline(&mut self.net.places[place_idx].input_description);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(t("Стохастичестие процессы", "Stochastic processes"));
                    let stats_enabled = self.net.ui.marker_count_stats;
                    if ui
                        .add_enabled(
                            stats_enabled,
                            egui::Button::new(t("Сбор статистики", "Collect statistics")),
                        )
                        .clicked()
                    {
                        self.place_stats_dialog_place_id = Some(place_id);
                        self.place_stats_dialog_backup =
                            Some((place_id, self.net.places[place_idx].stats));
                    }
                });
                egui::ComboBox::from_label(t("Распределение", "Distribution"))
                    .selected_text(Self::stochastic_text(
                        &self.net.places[place_idx].stochastic,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::None,
                            Self::stochastic_text(&StochasticDistribution::None, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Uniform { min: 0.0, max: 1.0 },
                            Self::stochastic_text(
                                &StochasticDistribution::Uniform { min: 0.0, max: 1.0 },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Normal {
                                mean: 1.0,
                                std_dev: 0.2,
                            },
                            Self::stochastic_text(
                                &StochasticDistribution::Normal {
                                    mean: 1.0,
                                    std_dev: 0.2,
                                },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Gamma {
                                shape: 2.0,
                                scale: 1.0,
                            },
                            Self::stochastic_text(
                                &StochasticDistribution::Gamma {
                                    shape: 2.0,
                                    scale: 1.0,
                                },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Exponential { lambda: 1.0 },
                            Self::stochastic_text(
                                &StochasticDistribution::Exponential { lambda: 1.0 },
                                is_ru,
                            ),
                        );
                        ui.selectable_value(
                            &mut self.net.places[place_idx].stochastic,
                            StochasticDistribution::Poisson { lambda: 1.0 },
                            Self::stochastic_text(
                                &StochasticDistribution::Poisson { lambda: 1.0 },
                                is_ru,
                            ),
                        );
                    });

                match &mut self.net.places[place_idx].stochastic {
                    StochasticDistribution::None => {}
                    StochasticDistribution::Uniform { min, max } => {
                        ui.horizontal(|ui| {
                            ui.label(t("min", "min"));
                            ui.add(egui::DragValue::new(min).speed(0.1).range(0.0..=10_000.0));
                            ui.label(t("max", "max"));
                            ui.add(egui::DragValue::new(max).speed(0.1).range(0.0..=10_000.0));
                        });
                        corrected_inputs |= sanitize_f64(min, 0.0, 10_000.0);
                        corrected_inputs |= sanitize_f64(max, 0.0, 10_000.0);
                        if *max < *min {
                            *max = *min;
                            corrected_inputs = true;
                        }
                    }
                    StochasticDistribution::Normal { mean, std_dev } => {
                        ui.horizontal(|ui| {
                            ui.label(t("mean", "mean"));
                            ui.add(egui::DragValue::new(mean).speed(0.1).range(0.0..=10_000.0));
                            ui.label(t("std", "std"));
                            ui.add(
                                egui::DragValue::new(std_dev)
                                    .speed(0.1)
                                    .range(0.0..=10_000.0),
                            );
                        });
                        corrected_inputs |= sanitize_f64(mean, 0.0, 10_000.0);
                        corrected_inputs |= sanitize_f64(std_dev, 0.0, 10_000.0);
                    }
                    StochasticDistribution::Gamma { shape, scale } => {
                        ui.horizontal(|ui| {
                            ui.label(t("shape", "shape"));
                            ui.add(
                                egui::DragValue::new(shape)
                                    .speed(0.1)
                                    .range(0.0001..=10_000.0),
                            );
                            ui.label(t("scale", "scale"));
                            ui.add(
                                egui::DragValue::new(scale)
                                    .speed(0.1)
                                    .range(0.0001..=10_000.0),
                            );
                        });
                        corrected_inputs |= sanitize_f64(shape, 0.0001, 10_000.0);
                        corrected_inputs |= sanitize_f64(scale, 0.0001, 10_000.0);
                    }
                    StochasticDistribution::Exponential { lambda }
                    | StochasticDistribution::Poisson { lambda } => {
                        ui.horizontal(|ui| {
                            ui.label(t("lambda", "lambda"));
                            ui.add(
                                egui::DragValue::new(lambda)
                                    .speed(0.1)
                                    .range(0.0001..=10_000.0),
                            );
                        });
                        corrected_inputs |= sanitize_f64(lambda, 0.0001, 10_000.0);
                    }
                }

                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
                let mut markov_enabled = self.net.places[place_idx].markov_highlight;
                if ui
                    .checkbox(
                        &mut markov_enabled,
                        t("Марковская метка", "Markov annotation"),
                    )
                    .changed()
                {
                    self.net.places[place_idx].markov_highlight = markov_enabled;
                    self.update_markov_annotations();
                }
                let mut markov_placement = self.net.places[place_idx].markov_placement;
                egui::ComboBox::from_label(t(
                    "Положение марковской метки",
                    "Markov highlight placement",
                ))
                .selected_text(Self::markov_placement_text(markov_placement, is_ru))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut markov_placement,
                        MarkovPlacement::Bottom,
                        Self::markov_placement_text(MarkovPlacement::Bottom, is_ru),
                    );
                    ui.selectable_value(
                        &mut markov_placement,
                        MarkovPlacement::Top,
                        Self::markov_placement_text(MarkovPlacement::Top, is_ru),
                    );
                });
                self.net.places[place_idx].markov_placement = markov_placement;
                ui.separator();
                ui.label(t("Название", "Name"));
                ui.text_edit_singleline(&mut self.net.places[place_idx].name);
            });
        open
    }
}


# src\ui\app\petri_app\drawing\draw_place_stats_dialog.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_place_stats_dialog(&mut self, ctx: &egui::Context) {
        let Some(place_id) = self.place_stats_dialog_place_id else {
            self.place_stats_dialog_backup = None;
            return;
        };
        if !self.net.ui.marker_count_stats {
            self.place_stats_dialog_place_id = None;
            self.place_stats_dialog_backup = None;
            return;
        }
        let Some(place_idx) = self.place_idx_by_id(place_id) else {
            self.place_stats_dialog_place_id = None;
            self.place_stats_dialog_backup = None;
            return;
        };

        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(t("Статистика", "Statistics"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new(("place_stats_dialog", place_id)))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("ID: P{}", place_id));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            if let Some((backup_id, backup)) = self.place_stats_dialog_backup.take()
                            {
                                if backup_id == place_id {
                                    self.net.places[place_idx].stats = backup;
                                }
                            }
                            self.place_stats_dialog_place_id = None;
                        }
                        if ui.button("Ok").clicked() {
                            self.place_stats_dialog_backup = None;
                            self.place_stats_dialog_place_id = None;
                        }
                    });
                });
                ui.separator();

                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
                        ui.label(t("Число маркеров", "Tokens"));
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.markers_total,
                            t("Общая", "Total"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.markers_input,
                            t("На входе", "On input"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.markers_output,
                            t("На выходе", "On output"),
                        );
                    });
                    cols[1].group(|ui| {
                        ui.label(t("Загруженность", "Load"));
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.load_total,
                            t("Общая", "Total"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.load_input,
                            t("Вход", "Input"),
                        );
                        ui.checkbox(
                            &mut self.net.places[place_idx].stats.load_output,
                            t("Выход", "Output"),
                        );
                    });
                });
            });

        if !open {
            // Treat closing via X as cancel.
            if let Some((backup_id, backup)) = self.place_stats_dialog_backup.take() {
                if backup_id == place_id {
                    self.net.places[place_idx].stats = backup;
                }
            }
            self.place_stats_dialog_place_id = None;
        }
    }
}


# src\ui\app\petri_app\drawing\draw_proof_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_proof_window(&mut self, ctx: &egui::Context) {
        if !self.show_proof {
            return;
        }
        let mut open = self.show_proof;
        egui::Window::new("Proof")
            .constrained_to_viewport(ctx)
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                let Some(result) = self.sim_result.as_ref() else {
                    ui.label(self.tr("Сначала запустите имитацию.", "Run simulation first."));
                    return;
                };
                ui.label(self.tr(
                    "Доказательство построено по журналу состояний (trace).",
                    "Proof is generated from simulation trace.",
                ));
                ui.separator();
                let visible_steps = Self::debug_visible_log_indices(result);
                if visible_steps.is_empty() {
                    ui.label(self.tr(
                        "Р’СЃС‚СЂР°С‚ РµС‰Рµ РЅРµСЂРµР°Р» Р·Р°РїРёСЁ.",
                        "Trace is empty.",
                    ));
                    return;
                }
                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                egui::Grid::new("proof_grid_header")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(self.tr("Шаг", "Step"));
                        ui.label(self.tr("Время", "Time"));
                        ui.label(self.tr("Сработал переход", "Fired transition"));
                        ui.label(self.tr("Маркировка", "Marking"));
                        ui.end_row();
                    });
                egui::ScrollArea::vertical().max_height(420.0).show_rows(
                    ui,
                    row_h,
                    visible_steps.len(),
                    |ui, range| {
                        egui::Grid::new("proof_grid_rows")
                            .striped(true)
                            .show(ui, |ui| {
                                for row_idx in range {
                                    let entry = &result.logs[visible_steps[row_idx]];
                                    ui.label(row_idx.to_string());
                                    ui.label(format!("{:.3}", entry.time));
                                    ui.label(
                                        entry
                                            .fired_transition
                                            .map(|i| format!("T{}", i + 1))
                                            .unwrap_or_else(|| "-".to_string()),
                                    );
                                    ui.label(format!("{:?}", entry.marking));
                                    ui.end_row();
                                }
                            });
                    },
                );
            });
        self.show_proof = open;
    }
}


# src\ui\app\petri_app\drawing\draw_status.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_status(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Курсор: x={:.2}, y={:.2}",
                    self.canvas.cursor_world[0], self.canvas.cursor_world[1]
                ));
                if let Some(path) = &self.file_path {
                    ui.separator();
                    ui.label(format!("File: {}", path.display()));
                }
                if let Some(err) = &self.last_error {
                    ui.separator();
                    ui.colored_label(Color32::RED, format!("Error: {err}"));
                }
                if let Some(hint) = &self.status_hint {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(0, 90, 170), hint);
                }
            });
        });
    }
}


# src\ui\app\petri_app\drawing\draw_table_workspace.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let desired = ui.available_size_before_wrap();
        let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
        let painter = ui.painter_at(rect);

        let step = self.grid_step_world();
        let mut x = rect.left();
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(225)),
            );
            x += step;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(225)),
            );
            y += step;
        }

        ui.allocate_ui_at_rect(rect.shrink(6.0), |ui| {
            if self.show_table_view {
                self.draw_table_view(ui);
            }
        });
    }
}


# src\ui\app\petri_app\drawing\draw_text_properties.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_text_properties(&mut self, ctx: &egui::Context) {
        if !self.show_text_props {
            return;
        }
        if let Some(id) = self.canvas.selected_text {
            self.text_props_id = Some(id);
        }
        if let Some(text_id) = self.text_props_id {
            let title = self.tr("Редактирование текста", "Text Editing").to_string();
            self.show_text_props = self.draw_text_props_window(ctx, text_id, title);
        } else {
            self.show_text_props = false;
        }
    }
}


# src\ui\app\petri_app\drawing\draw_text_props_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_text_props_window(
        &mut self,
        ctx: &egui::Context,
        text_id: u64,
        title: String,
    ) -> bool {
        let Some(text_idx) = self.text_idx_by_id(text_id) else {
            return false;
        };
        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(title)
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("text_props_window"))
            .open(&mut open)
            .resizable(true)
            .default_size(egui::vec2(460.0, 360.0))
            .min_size(egui::vec2(360.0, 260.0))
            .show(ctx, |ui| {
                let text = &mut self.text_blocks[text_idx];
                ui.horizontal(|ui| {
                    ui.label(t("Шрифт", "Font"));
                    egui::ComboBox::from_id_source("text_font_combo")
                        .selected_text(text.font_name.clone())
                        .show_ui(ui, |ui| {
                            for name in Self::text_font_candidates() {
                                ui.selectable_value(
                                    &mut text.font_name,
                                    (*name).to_string(),
                                    *name,
                                );
                            }
                        });

                    ui.label(t("Размер", "Size"));
                    ui.add(egui::DragValue::new(&mut text.font_size).range(6.0..=72.0));

                    ui.label(t("Цвет", "Color"));
                    egui::ComboBox::from_id_source("text_color_combo")
                        .selected_text(Self::text_color_text(text.color, is_ru))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Default,
                                Self::text_color_text(NodeColor::Default, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Blue,
                                Self::text_color_text(NodeColor::Blue, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Red,
                                Self::text_color_text(NodeColor::Red, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Green,
                                Self::text_color_text(NodeColor::Green, is_ru),
                            );
                            ui.selectable_value(
                                &mut text.color,
                                NodeColor::Yellow,
                                Self::text_color_text(NodeColor::Yellow, is_ru),
                            );
                        });
                });

                ui.separator();
                ui.add(
                    egui::TextEdit::multiline(&mut text.text)
                        .desired_rows(6)
                        .desired_width(380.0),
                );
            });
        open
    }
}


# src\ui\app\petri_app\drawing\draw_transition_properties.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_transition_properties(&mut self, ctx: &egui::Context) {
        if !self.show_transition_props {
            return;
        }
        if let Some(id) = self
            .canvas
            .selected_transition
            .or_else(|| self.canvas.selected_transitions.last().copied())
        {
            self.transition_props_id = Some(id);
        }
        if let Some(transition_id) = self.transition_props_id {
            let title = self
                .tr("Свойства перехода", "Transition Properties")
                .to_string();
            self.show_transition_props =
                self.draw_transition_props_window(ctx, transition_id, title);
        } else {
            self.show_transition_props = false;
        }
    }
}


# src\ui\app\petri_app\drawing\draw_transition_props_window.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_transition_props_window(
        &mut self,
        ctx: &egui::Context,
        transition_id: u64,
        title: String,
    ) -> bool {
        let Some(transition_idx) = self.transition_idx_by_id(transition_id) else {
            return false;
        };

        let is_ru = matches!(self.net.ui.language, Language::Ru);
        let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };

        let mut open = true;
        egui::Window::new(title)
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("transition_props_window"))
            .resizable(true)
            .default_size(egui::vec2(420.0, 520.0))
            .min_size(egui::vec2(320.0, 360.0))
            .open(&mut open)
            .show(ctx, |ui| {
                let mut corrected_inputs = false;
                ui.label(format!("ID: T{}", transition_id));
                ui.separator();
                let mut priority = self.net.tables.mpr[transition_idx];
                corrected_inputs |= sanitize_i32(&mut priority, -1_000_000, 1_000_000);
                ui.horizontal(|ui| {
                    ui.label(t("Приоритет", "Priority"));
                    if ui.add(egui::DragValue::new(&mut priority)).changed() {
                        corrected_inputs |= sanitize_i32(&mut priority, -1_000_000, 1_000_000);
                    }
                });
                self.net.tables.mpr[transition_idx] = priority;
                ui.label(t("Размер перехода", "Transition size"));
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.net.transitions[transition_idx].size,
                        VisualSize::Small,
                        t("Малый", "Small"),
                    );
                    ui.radio_value(
                        &mut self.net.transitions[transition_idx].size,
                        VisualSize::Medium,
                        t("Средний", "Medium"),
                    );
                    ui.radio_value(
                        &mut self.net.transitions[transition_idx].size,
                        VisualSize::Large,
                        t("Большой", "Large"),
                    );
                });

                egui::ComboBox::from_label(t("Положение метки", "Label position"))
                    .selected_text(Self::label_pos_text(
                        self.net.transitions[transition_idx].label_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].label_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Положение текста", "Text position"))
                    .selected_text(Self::label_pos_text(
                        self.net.transitions[transition_idx].text_position,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Top,
                            t("Вверху", "Top"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Bottom,
                            t("Внизу", "Bottom"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Left,
                            t("Слева", "Left"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Right,
                            t("Справа", "Right"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].text_position,
                            LabelPosition::Center,
                            t("По центру", "Center"),
                        );
                    });

                egui::ComboBox::from_label(t("Цвет", "Color"))
                    .selected_text(Self::node_color_text(
                        self.net.transitions[transition_idx].color,
                        is_ru,
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Default,
                            t("По умолчанию", "Default"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Blue,
                            t("Синий", "Blue"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Red,
                            t("Красный", "Red"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Green,
                            t("Зеленый", "Green"),
                        );
                        ui.selectable_value(
                            &mut self.net.transitions[transition_idx].color,
                            NodeColor::Yellow,
                            t("Желтый", "Yellow"),
                        );
                    });

                ui.separator();
                ui.label(t("Название", "Name"));
                ui.text_edit_singleline(&mut self.net.transitions[transition_idx].name);
                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
            });
        open
    }
}


# src\ui\app\petri_app\drawing\mod.rs
﻿use super::*;

mod draw_arc_properties;
mod draw_arc_props_window;
mod draw_atf_window;
mod draw_debug_window;
mod draw_help_controls;
mod draw_help_development;
mod draw_layout;
mod draw_markov_window;
mod draw_menu;
mod draw_netstar_export_validation;
mod draw_place_properties;
mod draw_place_props_window;
mod draw_place_stats_dialog;
mod draw_proof_window;
mod draw_status;
mod draw_table_workspace;
mod draw_text_properties;
mod draw_text_props_window;
mod draw_transition_properties;
mod draw_transition_props_window;
mod window_constraints;

pub use window_constraints::WindowExt;


# src\ui\app\petri_app\drawing\window_constraints.rs
use egui::{Context, Rect, Window};

/// Экстеншн для `egui::Window`, ограничивающий размер окна видимой областью контекста.
pub trait WindowExt {
    fn constrained_to_viewport(self, ctx: &Context) -> Self;
}

impl<'a> WindowExt for Window<'a> {
    fn constrained_to_viewport(self, ctx: &Context) -> Self {
        let screen_rect = ctx.input(|input| input.screen_rect());
        let viewport = if screen_rect == Rect::EVERYTHING {
            ctx.available_rect()
        } else {
            screen_rect
        };
        self.constrain_to(viewport)
    }
}


# src\ui\app\petri_app\file_ops\arc_topology_fingerprint.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_topology_fingerprint(net: &PetriNet) -> u64 {
        let mut place_idx = HashMap::<u64, usize>::new();
        for (idx, place) in net.places.iter().enumerate() {
            place_idx.insert(place.id, idx + 1);
        }
        let mut transition_idx = HashMap::<u64, usize>::new();
        for (idx, transition) in net.transitions.iter().enumerate() {
            transition_idx.insert(transition.id, idx + 1);
        }

        let mut edges = Vec::<(u8, i8, usize, usize, u32)>::new();
        for arc in &net.arcs {
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
        for inh in &net.inhibitor_arcs {
            if let (Some(&p), Some(&t)) = (
                place_idx.get(&inh.place_id),
                transition_idx.get(&inh.transition_id),
            ) {
                edges.push((1, -1, p, t, inh.threshold.max(1)));
            }
        }
        edges.sort_unstable();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        net.places.len().hash(&mut hasher);
        net.transitions.len().hash(&mut hasher);
        edges.hash(&mut hasher);
        hasher.finish()
    }
}


# src\ui\app\petri_app\file_ops\assign_auto_name_for_place.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn assign_auto_name_for_place(&mut self, place_id: u64) {
        let mut ids: Vec<u64> = self.net.places.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        let rank = ids
            .iter()
            .position(|&id| id == place_id)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| self.net.places.len().max(1));
        let new_name = format!("P{rank}");
        if let Some(index) = self.place_idx_by_id(place_id) {
            self.net.places[index].name = new_name;
        }
    }
}


# src\ui\app\petri_app\file_ops\assign_auto_name_for_transition.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn assign_auto_name_for_transition(&mut self, transition_id: u64) {
        let mut ids: Vec<u64> = self.net.transitions.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let rank = ids
            .iter()
            .position(|&id| id == transition_id)
            .map(|idx| idx + 1)
            .unwrap_or_else(|| self.net.transitions.len().max(1));
        let new_name = format!("T{rank}");
        if let Some(index) = self.transition_idx_by_id(transition_id) {
            self.net.transitions[index].name = new_name;
        }
    }
}


# src\ui\app\petri_app\file_ops\cleanup_legacy_sidecar.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn cleanup_legacy_sidecar(path: &std::path::Path) {
        let sidecar_path = Self::ui_sidecar_path(path);
        if sidecar_path.exists() {
            let _ = fs::remove_file(sidecar_path);
        }
    }
}


# src\ui\app\petri_app\file_ops\ensure_unique_place_name.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn ensure_unique_place_name(
        &self,
        desired: &str,
        exclude_id: u64,
    ) -> String {
        let base = desired.trim();
        if base.is_empty() {
            return String::new();
        }
        let mut candidate = base.to_string();
        let mut n = 2u32;
        while self
            .net
            .places
            .iter()
            .any(|p| p.id != exclude_id && p.name.trim() == candidate.as_str())
        {
            candidate = format!("{base} ({n})");
            n = n.saturating_add(1);
        }
        candidate
    }
}


# src\ui\app\petri_app\file_ops\ensure_unique_transition_name.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn ensure_unique_transition_name(
        &self,
        desired: &str,
        exclude_id: u64,
    ) -> String {
        let base = desired.trim();
        if base.is_empty() {
            return String::new();
        }
        let mut candidate = base.to_string();
        let mut n = 2u32;
        while self
            .net
            .transitions
            .iter()
            .any(|t| t.id != exclude_id && t.name.trim() == candidate.as_str())
        {
            candidate = format!("{base} ({n})");
            n = n.saturating_add(1);
        }
        candidate
    }
}


# src\ui\app\petri_app\file_ops\extract_legacy_export_hints.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn extract_legacy_export_hints(
        path: &std::path::Path,
    ) -> Option<LegacyExportHints> {
        const PLACE_RECORD_SIZE: usize = 231;
        const TRANSITION_RECORD_SIZE: usize = 105;
        let bytes = fs::read(path).ok()?;
        if bytes.starts_with(crate::model::GPN2_MAGIC.as_bytes()) || bytes.len() < 16 {
            return None;
        }
        let read_i32 = |off: usize| -> Option<i32> {
            if off + 4 > bytes.len() {
                return None;
            }
            Some(i32::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
            ]))
        };
        let p = read_i32(0)?.max(0) as usize;
        let t = read_i32(4)?.max(0) as usize;
        let arcs_off = 16usize
            .saturating_add(p.saturating_mul(PLACE_RECORD_SIZE))
            .saturating_add(t.saturating_mul(TRANSITION_RECORD_SIZE));
        if arcs_off + 6 > bytes.len() {
            return None;
        }
        let footer_bytes = None;
        let arc_header_extra = Some(u16::from_le_bytes([
            bytes[arcs_off + 4],
            bytes[arcs_off + 5],
        ]));
        Some(LegacyExportHints {
            places_count: Some(p),
            transitions_count: Some(t),
            arc_topology_fingerprint: None,
            arc_header_extra,
            footer_bytes,
            raw_arc_and_tail: Some(bytes[arcs_off..].to_vec()),
        })
    }
}


# src\ui\app\petri_app\file_ops\import_matrix_csv.rs
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


# src\ui\app\petri_app\file_ops\load_legacy_sidecar_for_migration.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn load_legacy_sidecar_for_migration(&mut self, path: &std::path::Path) {
        if !self.text_blocks.is_empty() || !self.decorative_frames.is_empty() {
            return;
        }

        let sidecar_path = Self::ui_sidecar_path(path);
        let Ok(raw) = fs::read_to_string(&sidecar_path) else {
            return;
        };
        let Ok(sidecar) = serde_json::from_str::<LegacyUiSidecar>(&raw) else {
            return;
        };

        self.text_blocks = sidecar.text_blocks;
        self.decorative_frames = sidecar
            .decorative_frames
            .into_iter()
            .map(|frame| CanvasFrame {
                id: frame.id,
                pos: frame.pos,
                width: frame.side.max(Self::FRAME_MIN_SIDE),
                height: frame.side.max(Self::FRAME_MIN_SIDE),
            })
            .collect();
        self.next_text_id = sidecar.next_text_id.max(
            self.text_blocks
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
        self.next_frame_id = sidecar.next_frame_id.max(
            self.decorative_frames
                .iter()
                .map(|f| f.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );

        // Persist migrated overlays to GPN2 on next save.
        self.sync_model_overlays_from_canvas();
    }
}


# src\ui\app\petri_app\file_ops\mod.rs
﻿use super::*;

mod arc_topology_fingerprint;
mod assign_auto_name_for_place;
mod assign_auto_name_for_transition;
mod cleanup_legacy_sidecar;
mod ensure_unique_place_name;
mod ensure_unique_transition_name;
mod extract_legacy_export_hints;
mod import_matrix_csv;
mod load_legacy_sidecar_for_migration;
mod new;
mod new_file;
mod new_for_tests;
mod open_file;
mod parse_place_auto_index;
mod parse_transition_auto_index;
mod reset_sim_stop_controls;
mod save_file;
mod save_file_as;
mod sync_canvas_overlays_from_model;
mod sync_model_overlays_from_canvas;
mod ui_sidecar_path;


# src\ui\app\petri_app\file_ops\new.rs
use super::*;

impl PetriApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(test)]
        {
            Self::new_for_tests()
        }
        #[cfg(not(test))]
        {
            let net = PetriNet::new();
            Self {
                net,
                tool: Tool::Edit,
                canvas: CanvasState::default(),
                sim_params: SimulationParams::default(),
                sim_result: None,
                show_sim_params: false,
                show_results: false,
                show_atf: false,
                atf_selected_place: 0,
                atf_text: String::new(),
                file_path: None,
                last_error: None,
                layout_mode: LayoutMode::TileVertical,
                show_graph_view: true,
                show_table_view: false,
                table_fullscreen: false,
                show_struct_vectors: true,
                show_struct_pre: true,
                show_struct_post: true,
                show_struct_inhibitor: true,
                place_props_id: None,
                transition_props_id: None,
                show_place_props: false,
                show_transition_props: false,
                text_props_id: None,
                show_text_props: false,
                arc_props_id: None,
                show_arc_props: false,
                show_debug: false,
                debug_step: 0,
                debug_playing: false,
                debug_interval_ms: 1000,
                debug_arc_animation: true,
                debug_animation_enabled: false,
                debug_animation_local_clock: 0.0,
                debug_animation_current_duration: 0.0,
                debug_animation_last_update: None,
                debug_animation_events: Vec::new(),
                debug_animation_active_event: None,
                debug_animation_step_active: false,
                debug_place_colors: Vec::new(),
                show_proof: false,
                text_blocks: Vec::new(),
                next_text_id: 1,
                decorative_frames: Vec::new(),
                next_frame_id: 1,
                clipboard: None,
                paste_serial: 0,
                undo_stack: Vec::new(),
                legacy_export_hints: None,
                status_hint: None,
                show_help_development: false,
                show_help_controls: false,
                place_stats_dialog_place_id: None,
                place_stats_dialog_backup: None,
                show_place_stats_window: false,
                place_stats_view_place: 0,
                place_stats_series: PlaceStatsSeries::Total,
                place_stats_zoom_x: 1.0,
                place_stats_pan_x: 1.0,
                place_stats_show_grid: true,
                arc_display_mode: ArcDisplayMode::All,
                arc_display_color: NodeColor::Default,
                show_netstar_export_validation: false,
                pending_netstar_export_path: None,
                netstar_export_validation: None,
                show_new_element_props: false,
                show_markov_window: false,
                markov_model_enabled: false,
                markov_model: None,
                markov_limit_reached: false,
                markov_annotations: HashMap::new(),
                markov_place_arcs: Vec::new(),
                new_place_size: VisualSize::Medium,
                new_place_color: NodeColor::Default,
                new_place_marking: 0,
                new_place_capacity: Some(1),
                new_place_delay: 0.0,
                new_transition_size: VisualSize::Medium,
                new_transition_color: NodeColor::Default,
                new_transition_priority: 1,
                new_arc_weight: 1,
                new_arc_color: NodeColor::Default,
                new_arc_inhibitor: false,
                new_arc_inhibitor_threshold: 1,
                new_element_props_window_size: Vec2::new(360.0, 520.0),
                new_element_props_window_was_open: false,
            }
        }
    }
}


# src\ui\app\petri_app\file_ops\new_file.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn new_file(&mut self) {
        self.net = PetriNet::new();
        self.net.set_counts(0, 0);
        self.file_path = None;
        self.text_blocks.clear();
        self.next_text_id = 1;
        self.decorative_frames.clear();
        self.next_frame_id = 1;
        self.undo_stack.clear();
        self.legacy_export_hints = None;
        self.status_hint = None;
        self.show_netstar_export_validation = false;
        self.pending_netstar_export_path = None;
        self.netstar_export_validation = None;
        self.markov_model = None;
        self.markov_limit_reached = false;
        self.markov_annotations.clear();
        self.show_markov_window = false;
        self.markov_model_enabled = false;
        self.canvas.cursor_valid = false;
    }
}


# src\ui\app\petri_app\file_ops\new_for_tests.rs
use super::*;

impl PetriApp {
    #[cfg(test)]
    pub(in crate::ui::app) fn new_for_tests() -> Self {
        let mut net = PetriNet::new();
        net.set_counts(2, 1);
        net.places[0].pos = [120.0, 150.0];
        net.places[1].pos = [340.0, 150.0];
        net.transitions[0].pos = [240.0, 145.0];

        Self {
            net,
            tool: Tool::Edit,
            canvas: CanvasState::default(),
            sim_params: SimulationParams::default(),
            sim_result: None,
            show_sim_params: false,
            show_results: false,
            show_atf: false,
            atf_selected_place: 0,
            atf_text: String::new(),
            file_path: None,
            last_error: None,
            layout_mode: LayoutMode::TileVertical,
            show_graph_view: true,
            show_table_view: false,
            table_fullscreen: false,
            show_struct_vectors: true,
            show_struct_pre: true,
            show_struct_post: true,
            show_struct_inhibitor: true,
            place_props_id: None,
            transition_props_id: None,
            show_place_props: false,
            show_transition_props: false,
            text_props_id: None,
            show_text_props: false,
            arc_props_id: None,
            show_arc_props: false,
            show_debug: false,
            debug_step: 0,
            debug_playing: false,
            debug_interval_ms: 1000,
            debug_arc_animation: true,
            debug_animation_enabled: false,
            debug_animation_local_clock: 0.0,
            debug_animation_current_duration: 0.0,
            debug_animation_last_update: None,
            debug_animation_events: Vec::new(),
            debug_animation_active_event: None,
            debug_animation_step_active: false,
            debug_place_colors: Vec::new(),
            show_proof: false,
            text_blocks: Vec::new(),
            next_text_id: 1,
            decorative_frames: Vec::new(),
            next_frame_id: 1,
            clipboard: None,
            paste_serial: 0,
            undo_stack: Vec::new(),
            legacy_export_hints: None,
            status_hint: None,
            show_help_development: false,
            show_help_controls: false,
            place_stats_dialog_place_id: None,
            place_stats_dialog_backup: None,
            show_place_stats_window: false,
            place_stats_view_place: 0,
            place_stats_series: PlaceStatsSeries::Total,
            place_stats_zoom_x: 1.0,
            place_stats_pan_x: 1.0,
            place_stats_show_grid: true,
            arc_display_mode: ArcDisplayMode::All,
            arc_display_color: NodeColor::Default,
            show_netstar_export_validation: false,
            pending_netstar_export_path: None,
            netstar_export_validation: None,
            show_new_element_props: false,
            show_markov_window: false,
            markov_model_enabled: false,
            markov_model: None,
            markov_limit_reached: false,
            markov_annotations: HashMap::new(),
            markov_place_arcs: Vec::new(),
            new_place_size: VisualSize::Medium,
            new_place_color: NodeColor::Default,
            new_place_marking: 0,
            new_place_capacity: Some(1),
            new_place_delay: 0.0,
            new_transition_size: VisualSize::Medium,
            new_transition_color: NodeColor::Default,
            new_transition_priority: 1,
            new_arc_weight: 1,
            new_arc_color: NodeColor::Default,
            new_arc_inhibitor: false,
            new_arc_inhibitor_threshold: 1,
            new_element_props_window_size: Vec2::new(360.0, 520.0),
            new_element_props_window_was_open: false,
        }
    }
}


# src\ui\app\petri_app\file_ops\open_file.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Файлы PetriNet", &["gpn2", "pn", "gpn"])
            .pick_file()
        {
            match load_gpn(&path) {
                Ok(result) => {
                    let legacy_hints = if result.legacy_debug.is_some() {
                        let mut hints = Self::extract_legacy_export_hints(&path);
                        if let Some(h) = hints.as_mut() {
                            h.arc_topology_fingerprint =
                                Some(Self::arc_topology_fingerprint(&result.model));
                        }
                        hints
                    } else {
                        None
                    };
                    self.net = result.model;
                    self.net.normalize_arc_ids();
                    self.net
                        .set_counts(self.net.places.len(), self.net.transitions.len());
                    self.file_path = Some(path.clone());
                    self.undo_stack.clear();
                    self.sync_canvas_overlays_from_model();
                    self.load_legacy_sidecar_for_migration(&path);
                    self.legacy_export_hints = legacy_hints;
                    self.status_hint = None;
                    self.canvas.cursor_valid = false;
                    let filtered: Vec<String> = result
                        .warnings
                        .iter()
                        .filter(|w| {
                            !w.contains("best-effort")
                                && !w.contains("signature heuristic")
                                && !w.contains("восстановлены по сигнатурам")
                        })
                        .cloned()
                        .collect();
                    if filtered.is_empty() {
                        self.last_error = None;
                    } else {
                        self.last_error = Some(format!(
                            "Импорт с предупреждениями: {}",
                            filtered.join("; ")
                        ));
                    }
                }
                Err(e) => self.last_error = Some(e.to_string()),
            }
        }
    }
}


# src\ui\app\petri_app\file_ops\parse_place_auto_index.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn parse_place_auto_index(name: &str) -> Option<usize> {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let first = chars.next()?;
        if !['P', 'p'].contains(&first) {
            return None;
        }
        let rest: String = chars.collect();
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        rest.parse::<usize>().ok()
    }
}


# src\ui\app\petri_app\file_ops\parse_transition_auto_index.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn parse_transition_auto_index(name: &str) -> Option<usize> {
        let trimmed = name.trim();
        let mut chars = trimmed.chars();
        let first = chars.next()?;
        if !['T', 't'].contains(&first) {
            return None;
        }
        let rest: String = chars.collect();
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        rest.parse::<usize>().ok()
    }
}


# src\ui\app\petri_app\file_ops\reset_sim_stop_controls.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn reset_sim_stop_controls(&mut self) {
        self.sim_params.use_pass_limit = false;
        self.sim_params.stop.through_place = None;
    }
}


# src\ui\app\petri_app\file_ops\save_file.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn save_file(&mut self) {
        if let Some(path) = self.file_path.clone() {
            let is_gpn2 = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("gpn2"))
                .unwrap_or(false);
            if !is_gpn2 {
                self.save_file_as();
                return;
            }
            self.sync_model_overlays_from_canvas();
            if let Err(e) = crate::io::gpn2::save_gpn2(&path, &self.net) {
                self.last_error = Some(e.to_string());
            } else {
                Self::cleanup_legacy_sidecar(&path);
            }
        } else {
            self.save_file_as();
        }
    }
}


# src\ui\app\petri_app\file_ops\save_file_as.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Файлы PetriNet (gpn2)", &["gpn2"])
            .set_file_name("модель.gpn2")
            .save_file()
        {
            self.file_path = Some(path.clone());
            self.sync_model_overlays_from_canvas();
            if let Err(e) = crate::io::gpn2::save_gpn2(&path, &self.net) {
                self.last_error = Some(e.to_string());
            } else {
                Self::cleanup_legacy_sidecar(&path);
            }
        }
    }
}


# src\ui\app\petri_app\file_ops\sync_canvas_overlays_from_model.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sync_canvas_overlays_from_model(&mut self) {
        self.text_blocks = self
            .net
            .ui
            .text_blocks
            .iter()
            .map(|item| CanvasTextBlock {
                id: item.id,
                pos: item.pos,
                text: item.text.clone(),
                font_name: item.font_name.clone(),
                font_size: item.font_size,
                color: item.color,
            })
            .collect();
        self.decorative_frames = self
            .net
            .ui
            .decorative_frames
            .iter()
            .map(|frame| CanvasFrame {
                id: frame.id,
                pos: frame.pos,
                width: frame.width.max(Self::FRAME_MIN_SIDE),
                height: frame.height.max(Self::FRAME_MIN_SIDE),
            })
            .collect();

        self.next_text_id = self.net.ui.next_text_id.max(
            self.text_blocks
                .iter()
                .map(|t| t.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
        self.next_frame_id = self.net.ui.next_frame_id.max(
            self.decorative_frames
                .iter()
                .map(|f| f.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );
    }
}


# src\ui\app\petri_app\file_ops\sync_model_overlays_from_canvas.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sync_model_overlays_from_canvas(&mut self) {
        self.net.ui.text_blocks = self
            .text_blocks
            .iter()
            .map(|item| UiTextBlock {
                id: item.id,
                pos: item.pos,
                text: item.text.clone(),
                font_name: item.font_name.clone(),
                font_size: item.font_size,
                color: item.color,
            })
            .collect();
        self.net.ui.decorative_frames = self
            .decorative_frames
            .iter()
            .map(|frame| UiDecorativeFrame {
                id: frame.id,
                pos: frame.pos,
                width: frame.width.max(Self::FRAME_MIN_SIDE),
                height: frame.height.max(Self::FRAME_MIN_SIDE),
            })
            .collect();
        self.net.ui.next_text_id = self.next_text_id;
        self.net.ui.next_frame_id = self.next_frame_id;
    }
}


# src\ui\app\petri_app\file_ops\ui_sidecar_path.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn ui_sidecar_path(path: &std::path::Path) -> PathBuf {
        let mut os = path.as_os_str().to_os_string();
        os.push(".petriui.json");
        PathBuf::from(os)
    }
}


# src\ui\app\petri_app\geometry\arc_at.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        let mut best_id = None;
        // Keep arc hit-test tighter so node clicks near edges still select the node.
        let mut best_dist = 12.0_f32;

        for arc in &self.net.arcs {
            if !self.arc_visible_by_mode(arc.color, arc.visible) {
                continue;
            }
            let Some((a, b)) = self.arc_screen_endpoints(rect, arc) else {
                continue;
            };
            let dist = Self::segment_distance_to_point(pos, a, b);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(arc.id);
            }
        }

        for inh in &self.net.inhibitor_arcs {
            if !self.arc_visible_by_mode(inh.color, inh.visible) {
                continue;
            }
            let Some((a, b)) = self.inhibitor_screen_endpoints(rect, inh) else {
                continue;
            };
            let dist = Self::segment_distance_to_point(pos, a, b);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(inh.id);
            }
        }

        best_id
    }
}


# src\ui\app\petri_app\geometry\arc_fully_inside_rect.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_fully_inside_rect(sel: Rect, from: Pos2, to: Pos2) -> bool {
        if !sel.contains(from) || !sel.contains(to) {
            return false;
        }

        let arrow = to - from;
        if arrow.length_sq() <= f32::EPSILON {
            return true;
        }

        let dir = arrow.normalized();
        let left = to - dir * 10.0 + Vec2::new(-dir.y, dir.x) * 5.0;
        let right = to - dir * 10.0 + Vec2::new(dir.y, -dir.x) * 5.0;
        sel.contains(left) && sel.contains(right)
    }
}


# src\ui\app\petri_app\geometry\arc_place_transition_pair.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_place_transition_pair(
        from: NodeRef,
        to: NodeRef,
    ) -> Option<(u64, u64)> {
        match (from, to) {
            (NodeRef::Place(pid), NodeRef::Transition(tid)) => Some((pid, tid)),
            _ => None,
        }
    }
}


# src\ui\app\petri_app\geometry\arc_screen_endpoints.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_screen_endpoints(
        &self,
        rect: Rect,
        arc: &crate::model::Arc,
    ) -> Option<(Pos2, Pos2)> {
        let (from_center, from_radius, from_rect, to_center, to_radius, to_rect) =
            match (arc.from, arc.to) {
                (NodeRef::Place(p), NodeRef::Transition(t)) => {
                    let (Some(pi), Some(ti)) =
                        (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                    else {
                        return None;
                    };
                    let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                    let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                    let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                    let t_rect = Rect::from_min_size(
                        t_min,
                        Self::transition_dimensions(self.net.transitions[ti].size)
                            * self.canvas.zoom,
                    );
                    (
                        p_center,
                        Some(p_radius),
                        None,
                        t_rect.center(),
                        None,
                        Some(t_rect),
                    )
                }
                (NodeRef::Transition(t), NodeRef::Place(p)) => {
                    let (Some(pi), Some(ti)) =
                        (self.place_idx_by_id(p), self.transition_idx_by_id(t))
                    else {
                        return None;
                    };
                    let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
                    let t_rect = Rect::from_min_size(
                        t_min,
                        Self::transition_dimensions(self.net.transitions[ti].size)
                            * self.canvas.zoom,
                    );
                    let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
                    let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
                    (
                        t_rect.center(),
                        None,
                        Some(t_rect),
                        p_center,
                        Some(p_radius),
                        None,
                    )
                }
                _ => return None,
            };

        let mut from = from_center;
        let mut to = to_center;
        let delta = to_center - from_center;
        let dir = if delta.length_sq() > 0.0 {
            delta.normalized()
        } else {
            Vec2::X
        };

        if let Some(radius) = from_radius {
            from += dir * radius;
        } else if let Some(r) = from_rect {
            from = Self::rect_border_point(r, dir);
        }

        if let Some(radius) = to_radius {
            to -= dir * radius;
        } else if let Some(r) = to_rect {
            to = Self::rect_border_point(r, -dir);
        }

        Some((from, to))
    }
}


# src\ui\app\petri_app\geometry\frame_at.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        self.decorative_frames
            .iter()
            .rev()
            .find(|frame| {
                let min = self.world_to_screen(rect, frame.pos);
                let size = Vec2::new(
                    frame.width.max(Self::FRAME_MIN_SIDE),
                    frame.height.max(Self::FRAME_MIN_SIDE),
                ) * self.canvas.zoom;
                let r = Rect::from_min_size(min, size);
                let tolerance = (6.0 * self.canvas.zoom).max(4.0);
                r.expand(tolerance).contains(pos) && !r.shrink(tolerance).contains(pos)
            })
            .map(|frame| frame.id)
    }
}


# src\ui\app\petri_app\geometry\frame_from_drag.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_from_drag(
        start: [f32; 2],
        current: [f32; 2],
    ) -> ([f32; 2], f32, f32) {
        let min_x = start[0].min(current[0]);
        let min_y = start[1].min(current[1]);
        let width = (current[0] - start[0]).abs();
        let height = (current[1] - start[1]).abs();
        ([min_x, min_y], width, height)
    }
}


# src\ui\app\petri_app\geometry\frame_idx_by_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_idx_by_id(&self, id: u64) -> Option<usize> {
        self.decorative_frames.iter().position(|item| item.id == id)
    }
}


# src\ui\app\petri_app\geometry\frame_resize_handle_rect.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn frame_resize_handle_rect(
        &self,
        rect: Rect,
        frame: &CanvasFrame,
    ) -> Rect {
        let min = self.world_to_screen(rect, frame.pos);
        let width = frame.width.max(Self::FRAME_MIN_SIDE) * self.canvas.zoom;
        let height = frame.height.max(Self::FRAME_MIN_SIDE) * self.canvas.zoom;
        let handle = Self::FRAME_RESIZE_HANDLE_PX;
        let center = Pos2::new(min.x + width, min.y + height);
        Rect::from_center_size(center, Vec2::splat(handle))
    }
}


# src\ui\app\petri_app\geometry\grid_step_world.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn grid_step_world(&self) -> f32 {
        if self.net.ui.snap_to_grid {
            Self::GRID_STEP_SNAP
        } else {
            Self::GRID_STEP_FREE
        }
    }
}


# src\ui\app\petri_app\geometry\inhibitor_screen_endpoints.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn inhibitor_screen_endpoints(
        &self,
        rect: Rect,
        inh: &crate::model::InhibitorArc,
    ) -> Option<(Pos2, Pos2)> {
        let (Some(pi), Some(ti)) = (
            self.place_idx_by_id(inh.place_id),
            self.transition_idx_by_id(inh.transition_id),
        ) else {
            return None;
        };

        let p_center = self.world_to_screen(rect, self.net.places[pi].pos);
        let p_radius = Self::place_radius(self.net.places[pi].size) * self.canvas.zoom;
        let t_min = self.world_to_screen(rect, self.net.transitions[ti].pos);
        let t_rect = Rect::from_min_size(
            t_min,
            Self::transition_dimensions(self.net.transitions[ti].size) * self.canvas.zoom,
        );
        let t_center = t_rect.center();
        let delta = t_center - p_center;
        let dir = if delta.length_sq() > 0.0 {
            delta.normalized()
        } else {
            Vec2::X
        };
        let from = p_center + dir * p_radius;
        let to = Self::rect_border_point(t_rect, -dir);

        Some((from, to))
    }
}


# src\ui\app\petri_app\geometry\keep_label_inside.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn keep_label_inside(
        rect: Rect,
        center: Pos2,
        mut offset: Vec2,
    ) -> Vec2 {
        let candidate = center + offset;
        let margin = 8.0;
        if candidate.y > rect.bottom() - margin {
            offset.y = -offset.y.abs();
        } else if candidate.y < rect.top() + margin {
            offset.y = offset.y.abs();
        }
        if candidate.x > rect.right() - margin {
            offset.x = -offset.x.abs();
        } else if candidate.x < rect.left() + margin {
            offset.x = offset.x.abs();
        }
        offset
    }
}


# src\ui\app\petri_app\geometry\label_offset.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn label_offset(pos: LabelPosition, scale: f32) -> Vec2 {
        match pos {
            LabelPosition::Top => Vec2::new(0.0, -24.0 * scale),
            LabelPosition::Bottom => Vec2::new(0.0, 24.0 * scale),
            LabelPosition::Left => Vec2::new(-28.0 * scale, 0.0),
            LabelPosition::Right => Vec2::new(28.0 * scale, 0.0),
            LabelPosition::Center => Vec2::ZERO,
        }
    }
}


# src\ui\app\petri_app\geometry\mod.rs
use super::*;

mod arc_at;
mod arc_fully_inside_rect;
mod arc_place_transition_pair;
mod arc_screen_endpoints;
mod frame_at;
mod frame_from_drag;
mod frame_idx_by_id;
mod frame_resize_handle_rect;
mod grid_step_world;
mod inhibitor_screen_endpoints;
mod keep_label_inside;
mod label_offset;
mod node_at;
mod place_label_offset;
mod rect_border_point;
mod screen_to_world;
mod segment_distance_to_point;
mod snap_point_to_grid;
mod snap_scalar_to_grid;
mod snapped_world;
mod text_at;
mod world_to_screen;


# src\ui\app\petri_app\geometry\node_at.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn node_at(&self, rect: Rect, pos: Pos2) -> Option<NodeRef> {
        for place in &self.net.places {
            let center = self.world_to_screen(rect, place.pos);
            if center.distance(pos) <= Self::place_radius(place.size) * self.canvas.zoom {
                return Some(NodeRef::Place(place.id));
            }
        }
        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let r = Rect::from_min_size(p, Self::transition_dimensions(tr.size) * self.canvas.zoom);
            if r.contains(pos) {
                return Some(NodeRef::Transition(tr.id));
            }
        }
        for place in &self.net.places {
            let center = self.world_to_screen(rect, place.pos);
            let radius = Self::place_radius(place.size) * self.canvas.zoom;
            let name_offset = Self::keep_label_inside(
                rect,
                center,
                Self::place_label_offset(place.text_position, radius, self.canvas.zoom),
            );
            let label_center = center + name_offset;
            let label_rect = Self::approx_text_rect(label_center, &place.name, self.canvas.zoom);
            if label_rect.contains(pos) {
                return Some(NodeRef::Place(place.id));
            }
        }
        for tr in &self.net.transitions {
            let p = self.world_to_screen(rect, tr.pos);
            let dims = Self::transition_dimensions(tr.size) * self.canvas.zoom;
            let r = Rect::from_min_size(p, dims);
            let label_center = r.center() + Self::label_offset(tr.label_position, self.canvas.zoom);
            let label_rect = Self::approx_text_rect(label_center, &tr.name, self.canvas.zoom);
            if label_rect.contains(pos) {
                return Some(NodeRef::Transition(tr.id));
            }
        }
        None
    }
}


# src\ui\app\petri_app\geometry\place_label_offset.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn place_label_offset(
        pos: LabelPosition,
        radius: f32,
        scale: f32,
    ) -> Vec2 {
        if pos == LabelPosition::Center {
            return Vec2::ZERO;
        }
        let distance = radius + 10.0 * scale;
        match pos {
            LabelPosition::Top => Vec2::new(0.0, -distance),
            LabelPosition::Bottom => Vec2::new(0.0, distance),
            LabelPosition::Left => Vec2::new(-distance, 0.0),
            LabelPosition::Right => Vec2::new(distance, 0.0),
            LabelPosition::Center => Vec2::ZERO,
        }
    }
}


# src\ui\app\petri_app\geometry\rect_border_point.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn rect_border_point(rect: Rect, dir: Vec2) -> Pos2 {
        let center = rect.center();
        let nx = if dir.x.abs() < f32::EPSILON {
            0.0
        } else {
            dir.x
        };
        let ny = if dir.y.abs() < f32::EPSILON {
            0.0
        } else {
            dir.y
        };
        let half_w = rect.width() * 0.5;
        let half_h = rect.height() * 0.5;
        let tx = if nx.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            half_w / nx.abs()
        };
        let ty = if ny.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            half_h / ny.abs()
        };
        let t = tx.min(ty);
        center + Vec2::new(nx * t, ny * t)
    }
}


# src\ui\app\petri_app\geometry\screen_to_world.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn screen_to_world(&self, rect: Rect, p: Pos2) -> [f32; 2] {
        [
            (p.x - rect.left() - self.canvas.pan.x) / self.canvas.zoom,
            (p.y - rect.top() - self.canvas.pan.y) / self.canvas.zoom,
        ]
    }
}


# src\ui\app\petri_app\geometry\segment_distance_to_point.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn segment_distance_to_point(pos: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        if ab.length_sq() <= f32::EPSILON {
            return pos.distance(a);
        }
        let t = ((pos - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        let proj = a + ab * t;
        proj.distance(pos)
    }
}


# src\ui\app\petri_app\geometry\snap_point_to_grid.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn snap_point_to_grid(&self, p: [f32; 2]) -> [f32; 2] {
        [
            self.snap_scalar_to_grid(p[0]),
            self.snap_scalar_to_grid(p[1]),
        ]
    }
}


# src\ui\app\petri_app\geometry\snap_scalar_to_grid.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn snap_scalar_to_grid(&self, v: f32) -> f32 {
        let step = self.grid_step_world();
        (v / step).round() * step
    }
}


# src\ui\app\petri_app\geometry\snapped_world.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn snapped_world(&self, world: [f32; 2]) -> [f32; 2] {
        if self.net.ui.snap_to_grid {
            self.snap_point_to_grid(world)
        } else {
            world
        }
    }
}


# src\ui\app\petri_app\geometry\text_at.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_at(&self, rect: Rect, pos: Pos2) -> Option<u64> {
        self.text_blocks
            .iter()
            .rev()
            .find(|item| {
                let center = self.world_to_screen(rect, item.pos);
                Self::approx_text_rect(center, &item.text, self.canvas.zoom).contains(pos)
            })
            .map(|item| item.id)
    }
}


# src\ui\app\petri_app\geometry\world_to_screen.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn world_to_screen(&self, rect: Rect, p: [f32; 2]) -> Pos2 {
        Pos2::new(
            rect.left() + self.canvas.pan.x + p[0] * self.canvas.zoom,
            rect.top() + self.canvas.pan.y + p[1] * self.canvas.zoom,
        )
    }
}


# src\ui\app\petri_app\helpers\approx_text_rect.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn approx_text_rect(center: Pos2, text: &str, zoom: f32) -> Rect {
        let scale = zoom.clamp(0.75, 2.0);
        let width = (text.chars().count().max(1) as f32 * 7.0 * scale).max(24.0);
        let height = 16.0 * scale;
        Rect::from_center_size(center, Vec2::new(width, height))
    }
}


# src\ui\app\petri_app\helpers\arc_display_mode_text.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_display_mode_text(
        mode: ArcDisplayMode,
        is_ru: bool,
    ) -> &'static str {
        match (mode, is_ru) {
            (ArcDisplayMode::All, true) => "Все",
            (ArcDisplayMode::OnlyColor, true) => "Только выбранный цвет",
            (ArcDisplayMode::Hidden, true) => "Скрыть все",
            (ArcDisplayMode::All, false) => "All",
            (ArcDisplayMode::OnlyColor, false) => "Only selected color",
            (ArcDisplayMode::Hidden, false) => "Hide all",
        }
    }
}


# src\ui\app\petri_app\helpers\arc_visible_by_mode.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_visible_by_mode(
        &self,
        color: NodeColor,
        per_arc_visible: bool,
    ) -> bool {
        if !per_arc_visible {
            return false;
        }
        match self.arc_display_mode {
            ArcDisplayMode::All => true,
            ArcDisplayMode::OnlyColor => color == self.arc_display_color,
            ArcDisplayMode::Hidden => false,
        }
    }
}


# src\ui\app\petri_app\helpers\color_to_egui.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn color_to_egui(color: NodeColor, fallback: Color32) -> Color32 {
        match color {
            NodeColor::Default => fallback,
            NodeColor::Blue => Color32::from_rgb(25, 90, 220),
            NodeColor::Red => Color32::from_rgb(200, 40, 40),
            NodeColor::Green => Color32::from_rgb(40, 150, 60),
            NodeColor::Yellow => Color32::from_rgb(200, 160, 20),
        }
    }
}


# src\ui\app\petri_app\helpers\debug_visible_log_indices.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn debug_visible_log_indices(result: &SimulationResult) -> Vec<usize> {
        if result.logs.is_empty() {
            return Vec::new();
        }

        // Step 0 in debug must always point to the initial state.
        let mut indices = vec![0usize];
        let mut previous_marking = result.logs[0].marking.as_slice();
        for (idx, entry) in result.logs.iter().enumerate().skip(1) {
            let marking_changed = previous_marking != entry.marking.as_slice();
            if entry.fired_transition.is_some() || marking_changed {
                indices.push(idx);
            }
            previous_marking = entry.marking.as_slice();
        }
        indices
    }
}


# src\ui\app\petri_app\helpers\format_marking.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn format_marking(marking: &[u32]) -> String {
        marking
            .iter()
            .enumerate()
            .map(|(idx, value)| format!("P{}={}", idx + 1, value))
            .collect::<Vec<_>>()
            .join(" ")
    }
}


# src\ui\app\petri_app\helpers\label_pos_text.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn label_pos_text(pos: LabelPosition, is_ru: bool) -> &'static str {
        match (pos, is_ru) {
            (LabelPosition::Top, true) => "Вверху",
            (LabelPosition::Bottom, true) => "Внизу",
            (LabelPosition::Left, true) => "Слева",
            (LabelPosition::Right, true) => "Справа",
            (LabelPosition::Center, true) => "По центру",
            (LabelPosition::Top, false) => "Top",
            (LabelPosition::Bottom, false) => "Bottom",
            (LabelPosition::Left, false) => "Left",
            (LabelPosition::Right, false) => "Right",
            (LabelPosition::Center, false) => "Center",
        }
    }
}


# src\ui\app\petri_app\helpers\markov_placement_text.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn markov_placement_text(
        placement: MarkovPlacement,
        is_ru: bool,
    ) -> &'static str {
        match (placement, is_ru) {
            (MarkovPlacement::Bottom, true) => "Вверху",
            (MarkovPlacement::Top, true) => "Внизу",
            (MarkovPlacement::Bottom, false) => "Bottom",
            (MarkovPlacement::Top, false) => "Top",
        }
    }
}


# src\ui\app\petri_app\helpers\mod.rs
﻿use super::*;

mod approx_text_rect;
mod arc_display_mode_text;
mod arc_visible_by_mode;
mod color_to_egui;
mod debug_visible_log_indices;
mod format_marking;
mod label_pos_text;
mod markov_placement_text;
mod node_color_text;
mod place_radius;
mod sampled_indices;
mod stochastic_text;
mod text_color_text;
mod text_family_from_name;
mod text_font_candidates;
mod tr;
mod transition_dimensions;


# src\ui\app\petri_app\helpers\node_color_text.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn node_color_text(color: NodeColor, is_ru: bool) -> &'static str {
        match (color, is_ru) {
            (NodeColor::Default, true) => "По умолчанию",
            (NodeColor::Blue, true) => "Синий",
            (NodeColor::Red, true) => "Красный",
            (NodeColor::Green, true) => "Зеленый",
            (NodeColor::Yellow, true) => "Желтый",
            (NodeColor::Default, false) => "Default",
            (NodeColor::Blue, false) => "Blue",
            (NodeColor::Red, false) => "Red",
            (NodeColor::Green, false) => "Green",
            (NodeColor::Yellow, false) => "Yellow",
        }
    }
}


# src\ui\app\petri_app\helpers\place_radius.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn place_radius(size: VisualSize) -> f32 {
        match size {
            VisualSize::Small => 14.0,
            VisualSize::Medium => 20.0,
            VisualSize::Large => 28.0,
        }
    }
}


# src\ui\app\petri_app\helpers\sampled_indices.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sampled_indices(total: usize, max_points: usize) -> Vec<usize> {
        if total == 0 {
            return Vec::new();
        }
        if max_points <= 1 || total <= max_points {
            return (0..total).collect();
        }

        let mut out = Vec::with_capacity(max_points);
        let last_idx = total - 1;
        let step = last_idx as f64 / (max_points - 1) as f64;
        for i in 0..max_points {
            let mut idx = (i as f64 * step).round() as usize;
            if idx > last_idx {
                idx = last_idx;
            }
            if out.last().copied() != Some(idx) {
                out.push(idx);
            }
        }
        if out.last().copied() != Some(last_idx) {
            out.push(last_idx);
        }
        out
    }
}


# src\ui\app\petri_app\helpers\stochastic_text.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn stochastic_text(
        dist: &StochasticDistribution,
        is_ru: bool,
    ) -> &'static str {
        match (dist, is_ru) {
            (StochasticDistribution::None, true) => "Нет",
            (StochasticDistribution::Uniform { .. }, true) => "Равномерное",
            (StochasticDistribution::Normal { .. }, true) => "Нормальное (Гаусса)",
            (StochasticDistribution::Exponential { .. }, true) => "Экспоненциальное",
            (StochasticDistribution::Gamma { .. }, true) => "Гамма",
            (StochasticDistribution::Poisson { .. }, true) => "Пуассона",
            (StochasticDistribution::None, false) => "None",
            (StochasticDistribution::Uniform { .. }, false) => "Uniform",
            (StochasticDistribution::Normal { .. }, false) => "Normal (Gaussian)",
            (StochasticDistribution::Exponential { .. }, false) => "Exponential",
            (StochasticDistribution::Gamma { .. }, false) => "Gamma",
            (StochasticDistribution::Poisson { .. }, false) => "Poisson",
        }
    }
}


# src\ui\app\petri_app\helpers\text_color_text.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_color_text(color: NodeColor, is_ru: bool) -> &'static str {
        match (color, is_ru) {
            (NodeColor::Default, true) => "Черный",
            (NodeColor::Blue, true) => "Синий",
            (NodeColor::Red, true) => "Красный",
            (NodeColor::Green, true) => "Зеленый",
            (NodeColor::Yellow, true) => "Желтый",
            (NodeColor::Default, false) => "Black",
            (NodeColor::Blue, false) => "Blue",
            (NodeColor::Red, false) => "Red",
            (NodeColor::Green, false) => "Green",
            (NodeColor::Yellow, false) => "Yellow",
        }
    }
}


# src\ui\app\petri_app\helpers\text_family_from_name.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_family_from_name(name: &str) -> egui::FontFamily {
        let lower = name.to_ascii_lowercase();
        if lower.contains("courier") || lower.contains("mono") {
            egui::FontFamily::Monospace
        } else {
            egui::FontFamily::Proportional
        }
    }
}


# src\ui\app\petri_app\helpers\text_font_candidates.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_font_candidates() -> &'static [&'static str] {
        &["MS Sans Serif", "Arial", "Courier New"]
    }
}


# src\ui\app\petri_app\helpers\tr.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn tr<'a>(&self, ru: &'a str, en: &'a str) -> Cow<'a, str> {
        match self.net.ui.language {
            Language::Ru => Cow::Borrowed(ru),
            Language::En => Cow::Borrowed(en),
        }
    }
}


# src\ui\app\petri_app\helpers\transition_dimensions.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn transition_dimensions(size: VisualSize) -> Vec2 {
        match size {
            VisualSize::Small => Vec2::new(10.0, 18.0),
            VisualSize::Medium => Vec2::new(12.0, 28.0),
            VisualSize::Large => Vec2::new(16.0, 38.0),
        }
    }
}


# src\ui\app\petri_app\indexing\arc_idx_by_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn arc_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.arcs.iter().position(|arc| arc.id == id)
    }
}


# src\ui\app\petri_app\indexing\inhibitor_arc_idx_by_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn inhibitor_arc_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.inhibitor_arcs.iter().position(|arc| arc.id == id)
    }
}


# src\ui\app\petri_app\indexing\mod.rs
﻿use super::*;

mod arc_idx_by_id;
mod inhibitor_arc_idx_by_id;
mod place_idx_by_id;
mod text_idx_by_id;
mod transition_idx_by_id;


# src\ui\app\petri_app\indexing\place_idx_by_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn place_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.places.iter().position(|p| p.id == id)
    }
}


# src\ui\app\petri_app\indexing\text_idx_by_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn text_idx_by_id(&self, id: u64) -> Option<usize> {
        self.text_blocks.iter().position(|item| item.id == id)
    }
}


# src\ui\app\petri_app\indexing\transition_idx_by_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn transition_idx_by_id(&self, id: u64) -> Option<usize> {
        self.net.transitions.iter().position(|t| t.id == id)
    }
}


# src\ui\app\petri_app\markov\calculate_markov_model.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn calculate_markov_model(&mut self) {
        self.net.sanitize_values();
        let chain = build_markov_chain(&self.net, Some(500));
        self.markov_limit_reached = chain.limit_reached;
        self.markov_model = Some(chain);
        self.update_markov_annotations();
        self.refresh_markov_place_arcs();
    }
}


# src\ui\app\petri_app\markov\helpers.rs
use crate::ui::app::MarkovPlaceArc;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn markov_expected_tokens(
        chain: &MarkovChain,
        place_count: usize,
    ) -> Option<Vec<f64>> {
        let weights = Self::chain_state_weights(chain);
        if weights.is_empty() {
            return None;
        }
        let mut expected = vec![0.0; place_count];
        for (state, prob) in chain.states.iter().zip(weights.iter()) {
            for (idx, &tokens) in state.iter().enumerate().take(place_count) {
                expected[idx] += *prob * tokens as f64;
            }
        }
        Some(expected)
    }

    pub(in crate::ui::app) fn markov_tokens_distribution(
        chain: &MarkovChain,
        place_idx: usize,
    ) -> Vec<(u32, f64)> {
        let weights = Self::chain_state_weights(chain);
        if weights.is_empty() {
            return Vec::new();
        }
        let mut distribution = HashMap::new();
        for (state, prob) in chain.states.iter().zip(weights.iter()) {
            let count = *state.get(place_idx).unwrap_or(&0);
            *distribution.entry(count).or_insert(0.0) += *prob;
        }
        let mut vec = distribution.into_iter().collect::<Vec<_>>();
        vec.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        vec
    }

    pub(in crate::ui::app) fn refresh_markov_place_arcs(&mut self) {
        if let Some(chain) = self.markov_model.as_ref() {
            self.markov_place_arcs = self.build_markov_place_arcs(chain);
        } else {
            self.markov_place_arcs.clear();
        }
    }

    fn build_markov_place_arcs(&self, chain: &MarkovChain) -> Vec<MarkovPlaceArc> {
        let mut arcs = HashMap::new();
        let state_weights = Self::chain_state_weights(chain);
        for (state_idx, edges) in chain.transitions.iter().enumerate() {
            let state_prob = *state_weights.get(state_idx).unwrap_or(&0.0);
            if state_prob <= 0.0 {
                continue;
            }
            let src_marking = &chain.states[state_idx];
            for &(dest_idx, rate) in edges {
                if rate <= 0.0 {
                    continue;
                }
                let dest_marking = &chain.states[dest_idx];
                let weight = state_prob * rate;
                let (consumed, produced) = Self::markov_places_delta(src_marking, dest_marking);
                if consumed.is_empty() {
                    continue;
                }
                let from_places = consumed
                    .into_iter()
                    .filter(|&idx| self.net.places[idx].show_markov_model)
                    .collect::<Vec<_>>();
                if from_places.is_empty() {
                    continue;
                }
                let pair_count = from_places.len() * produced.len().max(1);
                let contribution = weight / pair_count as f64;
                for from_idx in from_places {
                    if produced.is_empty() {
                        let key = (self.net.places[from_idx].id, None);
                        *arcs.entry(key).or_insert(0.0) += contribution;
                    } else {
                        for &to_idx in &produced {
                            let key = (
                                self.net.places[from_idx].id,
                                Some(self.net.places[to_idx].id),
                            );
                            *arcs.entry(key).or_insert(0.0) += contribution;
                        }
                    }
                }
            }
        }
        let mut result = arcs
            .into_iter()
            .map(|((from, to), probability)| MarkovPlaceArc {
                from_place_id: from,
                to_place_id: to,
                probability,
            })
            .collect::<Vec<_>>();
        let total: f64 = result.iter().map(|arc| arc.probability).sum();
        if total > 0.0 {
            for arc in &mut result {
                arc.probability /= total;
            }
        }
        result.sort_unstable_by(|a, b| {
            b.probability
                .partial_cmp(&a.probability)
                .unwrap_or(Ordering::Equal)
        });
        if result.is_empty() {
            result = self.fallback_markov_place_arcs();
        }
        result
    }

    fn fallback_markov_place_arcs(&self) -> Vec<MarkovPlaceArc> {
        let mut arcs = HashMap::new();
        let transition_count = self.net.transitions.len();
        for tr_idx in 0..transition_count {
            let mut total_pre = 0.0;
            for (place_idx, place) in self.net.places.iter().enumerate() {
                if !place.show_markov_model {
                    continue;
                }
                total_pre += self.net.tables.pre[place_idx][tr_idx] as f64;
            }
            if total_pre <= 0.0 {
                continue;
            }
            let mut output_places = Vec::new();
            for (place_idx, _) in self.net.places.iter().enumerate() {
                if self.net.tables.post[place_idx][tr_idx] > 0 {
                    output_places.push(place_idx);
                }
            }
            for (place_idx, place) in self.net.places.iter().enumerate() {
                if !place.show_markov_model {
                    continue;
                }
                let consumed = self.net.tables.pre[place_idx][tr_idx] as f64;
                if consumed <= 0.0 {
                    continue;
                }
                let share = consumed / total_pre;
                if output_places.is_empty() {
                    let key = (place.id, None);
                    *arcs.entry(key).or_insert(0.0) += share;
                } else {
                    let per_output = share / output_places.len() as f64;
                    for &to_idx in &output_places {
                        let key = (place.id, Some(self.net.places[to_idx].id));
                        *arcs.entry(key).or_insert(0.0) += per_output;
                    }
                }
            }
        }
        let mut result = arcs
            .into_iter()
            .map(|((from, to), probability)| MarkovPlaceArc {
                from_place_id: from,
                to_place_id: to,
                probability,
            })
            .collect::<Vec<_>>();
        let total: f64 = result.iter().map(|arc| arc.probability).sum();
        if total > 0.0 {
            for arc in &mut result {
                arc.probability /= total;
            }
        }
        result
    }

    fn chain_state_weights(chain: &MarkovChain) -> Vec<f64> {
        let mut weights = chain
            .stationary
            .as_ref()
            .cloned()
            .unwrap_or_else(|| vec![1.0; chain.states.len()]);
        let total: f64 = weights.iter().sum();
        if total > 0.0 {
            for w in weights.iter_mut() {
                *w /= total;
            }
        }
        weights
    }

    fn markov_places_delta(src: &[u32], dest: &[u32]) -> (Vec<usize>, Vec<usize>) {
        let mut consumed = Vec::new();
        let mut produced = Vec::new();
        for (idx, (&before, &after)) in src.iter().zip(dest.iter()).enumerate() {
            if before > after {
                consumed.push(idx);
            } else if after > before {
                produced.push(idx);
            }
        }
        (consumed, produced)
    }
}


# src\ui\app\petri_app\markov\mod.rs
use super::*;

mod calculate_markov_model;
mod helpers;
mod update_markov_annotations;


# src\ui\app\petri_app\markov\update_markov_annotations.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn update_markov_annotations(&mut self) {
        self.markov_annotations.clear();
        let Some(chain) = &self.markov_model else {
            return;
        };
        let expectation = Self::markov_expected_tokens(chain, self.net.places.len());
        for (idx, place) in self.net.places.iter().enumerate() {
            if !place.markov_highlight {
                continue;
            }
            let label = if let Some(expected) = expectation.as_ref() {
                format!("{} ≈ {:.3}", self.tr("π", "π"), expected[idx])
            } else {
                self.tr("Нет распределения", "No distribution").to_string()
            };
            self.markov_annotations.insert(place.id, label);
        }
    }
}


# src\ui\app\petri_app\mod.rs
use super::*;

mod clipboard;
mod drawing;
mod file_ops;
mod geometry;
mod helpers;
mod indexing;
mod markov;
mod netstar;
mod selection;


# src\ui\app\petri_app\netstar\clear_netstar_export_validation.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn clear_netstar_export_validation(&mut self) {
        self.show_netstar_export_validation = false;
        self.pending_netstar_export_path = None;
        self.netstar_export_validation = None;
    }
}


# src\ui\app\petri_app\netstar\confirm_netstar_export_from_validation.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn confirm_netstar_export_from_validation(&mut self) {
        let Some(path) = self.pending_netstar_export_path.clone() else {
            self.clear_netstar_export_validation();
            return;
        };

        self.sync_model_overlays_from_canvas();
        if let Err(e) = save_gpn_with_hints(&path, &self.net, self.legacy_export_hints.as_ref()) {
            self.last_error = Some(e.to_string());
        } else {
            self.last_error = None;
            self.status_hint = Some(
                self.tr("Экспорт в NetStar завершен", "NetStar export completed")
                    .to_string(),
            );
        }
        self.clear_netstar_export_validation();
    }
}


# src\ui\app\petri_app\netstar\duplicate_ids.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn duplicate_ids<I>(ids: I) -> Vec<u64>
    where
        I: IntoIterator<Item = u64>,
    {
        let mut counts: HashMap<u64, usize> = HashMap::new();
        for id in ids {
            *counts.entry(id).or_insert(0) += 1;
        }
        let mut duplicates: Vec<u64> = counts
            .into_iter()
            .filter_map(|(id, count)| (count > 1).then_some(id))
            .collect();
        duplicates.sort_unstable();
        duplicates
    }
}


# src\ui\app\petri_app\netstar\export_netstar_file.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn export_netstar_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Файлы NetStar (gpn)", &["gpn"])
            .set_file_name("экспорт_netstar.gpn")
            .save_file()
        {
            self.start_netstar_export_validation(path);
        }
    }
}


# src\ui\app\petri_app\netstar\mod.rs
﻿use super::*;

mod clear_netstar_export_validation;
mod confirm_netstar_export_from_validation;
mod duplicate_ids;
mod export_netstar_file;
mod netstar_non_exportable_items;
mod select_export_issue_target;
mod start_netstar_export_validation;
mod validate_netstar_export;


# src\ui\app\petri_app\netstar\netstar_non_exportable_items.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn netstar_non_exportable_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        if !self.text_blocks.is_empty() {
            items.push(format!(
                "{}: {}",
                self.tr("Текстовые блоки", "Text blocks"),
                self.text_blocks.len()
            ));
        }
        if !self.decorative_frames.is_empty() {
            items.push(format!(
                "{}: {}",
                self.tr("Декоративные рамки", "Decorative frames"),
                self.decorative_frames.len()
            ));
        }
        let has_arc_style_data = self
            .net
            .arcs
            .iter()
            .any(|arc| arc.color != NodeColor::Default || !arc.visible)
            || self
                .net
                .inhibitor_arcs
                .iter()
                .any(|arc| arc.color != NodeColor::Red || !arc.visible);
        if has_arc_style_data {
            items.push(
                self.tr("Цвет/скрытие дуг", "Arc color/visibility")
                    .to_string(),
            );
        }
        items
    }
}


# src\ui\app\petri_app\netstar\select_export_issue_target.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn select_export_issue_target(&mut self, issue: &str) -> bool {
        let mut arc_candidate: Option<u64> = None;
        let mut place_candidate: Option<u64> = None;
        let mut transition_candidate: Option<u64> = None;

        for token in issue.split(|c: char| !c.is_ascii_alphanumeric()) {
            if token.len() < 2 {
                continue;
            }
            let (prefix, rest) = token.split_at(1);
            let Ok(id) = rest.parse::<u64>() else {
                continue;
            };
            match prefix {
                "A" | "a" => arc_candidate = Some(id),
                "P" | "p" => place_candidate = Some(id),
                "T" | "t" => transition_candidate = Some(id),
                _ => {}
            }
        }

        if let Some(arc_id) = arc_candidate {
            let arc_exists = self.net.arcs.iter().any(|a| a.id == arc_id)
                || self.net.inhibitor_arcs.iter().any(|a| a.id == arc_id);
            if arc_exists {
                self.clear_selection();
                self.canvas.selected_arc = Some(arc_id);
                self.canvas.selected_arcs.push(arc_id);
                return true;
            }
        }

        if let Some(place_ref) = place_candidate {
            let by_id = self.place_idx_by_id(place_ref);
            let by_ordinal = place_ref
                .checked_sub(1)
                .and_then(|idx| usize::try_from(idx).ok())
                .filter(|&idx| idx < self.net.places.len());
            if let Some(idx) = by_id.or(by_ordinal) {
                let place_id = self.net.places[idx].id;
                self.clear_selection();
                self.canvas.selected_place = Some(place_id);
                self.canvas.selected_places.push(place_id);
                self.place_props_id = Some(place_id);
                self.show_place_props = true;
                return true;
            }
        }

        if let Some(transition_ref) = transition_candidate {
            let by_id = self.transition_idx_by_id(transition_ref);
            let by_ordinal = transition_ref
                .checked_sub(1)
                .and_then(|idx| usize::try_from(idx).ok())
                .filter(|&idx| idx < self.net.transitions.len());
            if let Some(idx) = by_id.or(by_ordinal) {
                let transition_id = self.net.transitions[idx].id;
                self.clear_selection();
                self.canvas.selected_transition = Some(transition_id);
                self.canvas.selected_transitions.push(transition_id);
                self.transition_props_id = Some(transition_id);
                self.show_transition_props = true;
                return true;
            }
        }

        false
    }
}


# src\ui\app\petri_app\netstar\start_netstar_export_validation.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn start_netstar_export_validation(&mut self, path: PathBuf) {
        self.sync_model_overlays_from_canvas();
        self.pending_netstar_export_path = Some(path);
        self.netstar_export_validation = Some(self.validate_netstar_export());
        self.show_netstar_export_validation = true;
    }
}


# src\ui\app\petri_app\netstar\validate_netstar_export.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn validate_netstar_export(&self) -> NetstarExportValidationReport {
        let mut report = NetstarExportValidationReport::default();

        let place_ids: HashSet<u64> = self.net.places.iter().map(|p| p.id).collect();
        let transition_ids: HashSet<u64> = self.net.transitions.iter().map(|t| t.id).collect();

        if self.net.tables.m0.len() != self.net.places.len()
            || self.net.tables.mo.len() != self.net.places.len()
            || self.net.tables.mz.len() != self.net.places.len()
        {
            report.errors.push(
                self.tr(
                    "Таблицы M0/Mo/Mz имеют неверный размер относительно числа мест.",
                    "M0/Mo/Mz table sizes do not match the places count.",
                )
                .to_string(),
            );
        }
        if self.net.tables.mpr.len() != self.net.transitions.len() {
            report.errors.push(
                self.tr(
                    "Таблица приоритетов переходов (Mpr) имеет неверный размер.",
                    "Mpr table size does not match the transitions count.",
                )
                .to_string(),
            );
        }
        for (name, matrix) in [
            ("Pre", &self.net.tables.pre),
            ("Post", &self.net.tables.post),
            ("Inhibitor", &self.net.tables.inhibitor),
        ] {
            if matrix.len() != self.net.places.len() {
                report.errors.push(format!(
                    "{}: {}",
                    self.tr(
                        "Некорректное число строк в матрице",
                        "Invalid matrix row count"
                    ),
                    name
                ));
                continue;
            }
            if matrix
                .iter()
                .any(|row| row.len() != self.net.transitions.len())
            {
                report.errors.push(format!(
                    "{}: {}",
                    self.tr(
                        "Некорректное число столбцов в матрице",
                        "Invalid matrix column count"
                    ),
                    name
                ));
            }
        }

        for id in Self::duplicate_ids(self.net.places.iter().map(|p| p.id)) {
            report.errors.push(format!(
                "{} P{}",
                self.tr("Дубликат ID позиции:", "Duplicate position ID:"),
                id
            ));
        }
        for id in Self::duplicate_ids(self.net.transitions.iter().map(|t| t.id)) {
            report.errors.push(format!(
                "{} T{}",
                self.tr("Дубликат ID перехода:", "Duplicate transition ID:"),
                id
            ));
        }
        let mut arc_like_ids: Vec<u64> = self.net.arcs.iter().map(|a| a.id).collect();
        arc_like_ids.extend(self.net.inhibitor_arcs.iter().map(|a| a.id));
        for id in Self::duplicate_ids(arc_like_ids) {
            report.errors.push(format!(
                "{} A{}",
                self.tr("Дубликат ID дуги:", "Duplicate arc ID:"),
                id
            ));
        }

        for arc in &self.net.arcs {
            if arc.weight == 0 {
                report.errors.push(format!(
                    "{} A{}",
                    self.tr(
                        "Вес дуги должен быть больше 0:",
                        "Arc weight must be greater than 0:"
                    ),
                    arc.id
                ));
            }
            if arc.weight > 1024 {
                report.warnings.push(format!(
                    "{} A{} ({} -> 1024)",
                    self.tr(
                        "Вес дуги будет ограничен при экспорте:",
                        "Arc weight will be clamped during export:"
                    ),
                    arc.id,
                    arc.weight
                ));
            }
            match (arc.from, arc.to) {
                (NodeRef::Place(place_id), NodeRef::Transition(transition_id))
                | (NodeRef::Transition(transition_id), NodeRef::Place(place_id)) => {
                    if !place_ids.contains(&place_id) || !transition_ids.contains(&transition_id) {
                        report.errors.push(format!(
                            "{} A{}",
                            self.tr(
                                "Дуга ссылается на несуществующую позицию/переход:",
                                "Arc references a missing position/transition:"
                            ),
                            arc.id
                        ));
                    }
                }
                _ => {
                    report.errors.push(format!(
                        "{} A{}",
                        self.tr(
                            "Дуга нарушает двудольность графа:",
                            "Arc breaks graph bipartiteness:"
                        ),
                        arc.id
                    ));
                }
            }
        }

        for inh in &self.net.inhibitor_arcs {
            if inh.threshold == 0 {
                report.errors.push(format!(
                    "{} A{}",
                    self.tr(
                        "Порог ингибиторной дуги должен быть больше 0:",
                        "Inhibitor threshold must be greater than 0:"
                    ),
                    inh.id
                ));
            }
            if inh.threshold > 1024 {
                report.warnings.push(format!(
                    "{} A{} ({} -> 1024)",
                    self.tr(
                        "Порог ингибиторной дуги будет ограничен при экспорте:",
                        "Inhibitor threshold will be clamped during export:"
                    ),
                    inh.id,
                    inh.threshold
                ));
            }
            if !place_ids.contains(&inh.place_id) || !transition_ids.contains(&inh.transition_id) {
                report.errors.push(format!(
                    "{} A{}",
                    self.tr(
                        "Ингибиторная дуга ссылается на несуществующую позицию/переход:",
                        "Inhibitor arc references a missing position/transition:"
                    ),
                    inh.id
                ));
            }
        }

        for (idx, place) in self.net.places.iter().enumerate() {
            let m0 = self.net.tables.m0.get(idx).copied().unwrap_or(0);
            let mo = self.net.tables.mo.get(idx).copied().flatten();
            let mz = self.net.tables.mz.get(idx).copied().unwrap_or(0.0);

            if !place.pos[0].is_finite() || !place.pos[1].is_finite() {
                report.errors.push(format!(
                    "{} P{}",
                    self.tr(
                        "Некорректные координаты позиции:",
                        "Invalid position coordinates:"
                    ),
                    idx + 1
                ));
            } else if place.pos[0] < 0.0
                || place.pos[1] < 0.0
                || place.pos[0] > 65535.0
                || place.pos[1] > 65535.0
            {
                report.warnings.push(format!(
                    "{} P{}",
                    self.tr(
                        "Координаты места могут выйти за диапазон legacy-формата:",
                        "Place coordinates may exceed legacy format limits:"
                    ),
                    idx + 1
                ));
            }

            if let Some(cap) = mo {
                if cap > 1_000_000 {
                    report.warnings.push(format!(
                        "{} P{} ({} -> 1000000)",
                        self.tr(
                            "Максимальная емкость позиции будет ограничена при экспорте:",
                            "Place capacity will be clamped during export:"
                        ),
                        idx + 1,
                        cap
                    ));
                }
            } else {
                report.warnings.push(format!(
                    "{} P{}",
                    self.tr(
                        "Безлимитная емкость позиции не поддерживается, будет заменена на 1:",
                        "Unlimited position capacity is not supported and will be replaced with 1:"
                    ),
                    idx + 1
                ));
            }

            let cap_for_export = mo.unwrap_or(1).clamp(1, 1_000_000);
            if m0 > cap_for_export || m0 > 1_000_000 {
                report.warnings.push(format!(
                    "{} P{}",
                    self.tr(
                        "Число маркеров места будет ограничено при экспорте:",
                        "Place markers count will be clamped during export:"
                    ),
                    idx + 1
                ));
            }

            if !mz.is_finite() {
                report.errors.push(format!(
                    "{} P{}",
                    self.tr(
                        "Задержка места имеет нечисловое значение:",
                        "Place delay has a non-finite value:"
                    ),
                    idx + 1
                ));
            } else if !(0.0..=86_400.0).contains(&mz) {
                report.warnings.push(format!(
                    "{} P{} ({:.3})",
                    self.tr(
                        "Задержка места будет ограничена диапазоном [0; 86400]:",
                        "Place delay will be clamped to [0; 86400]:"
                    ),
                    idx + 1,
                    mz
                ));
            }
        }

        for (idx, transition) in self.net.transitions.iter().enumerate() {
            let mpr = self.net.tables.mpr.get(idx).copied().unwrap_or(1);

            if !transition.pos[0].is_finite() || !transition.pos[1].is_finite() {
                report.errors.push(format!(
                    "{} T{}",
                    self.tr(
                        "Некорректные координаты перехода:",
                        "Invalid transition coordinates:"
                    ),
                    idx + 1
                ));
            } else if transition.pos[0] < 0.0
                || transition.pos[1] < 0.0
                || transition.pos[0] > 65535.0
                || transition.pos[1] > 65535.0
            {
                report.warnings.push(format!(
                    "{} T{}",
                    self.tr(
                        "Координаты перехода могут выйти за диапазон legacy-формата:",
                        "Transition coordinates may exceed legacy format limits:"
                    ),
                    idx + 1
                ));
            }

            if !(0..=1_000_000).contains(&mpr) {
                report.warnings.push(format!(
                    "{} T{} ({} -> диапазон 0..1000000)",
                    self.tr(
                        "Приоритет перехода будет ограничен при экспорте:",
                        "Transition priority will be clamped during export:"
                    ),
                    idx + 1,
                    mpr
                ));
            }
        }

        let non_exportable_items = self.netstar_non_exportable_items();
        if !non_exportable_items.is_empty() {
            report.warnings.push(
                self.tr(
                    "Есть элементы, которые не экспортируются в NetStar.",
                    "There are elements that are not exported to NetStar.",
                )
                .to_string(),
            );
            for item in non_exportable_items {
                report.warnings.push(format!("- {}", item));
            }
        }

        report
    }
}


# src\ui\app\petri_app\selection\clear_selection.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn clear_selection(&mut self) {
        self.canvas.selected_place = None;
        self.canvas.selected_transition = None;
        self.canvas.selected_places.clear();
        self.canvas.selected_transitions.clear();
        self.canvas.selected_arc = None;
        self.canvas.selected_arcs.clear();
        self.canvas.selected_text = None;
        self.canvas.selected_texts.clear();
        self.canvas.selected_frame = None;
        self.canvas.selected_frames.clear();
        self.canvas.frame_draw_start_world = None;
        self.canvas.frame_draw_current_world = None;
        self.canvas.frame_resize_id = None;
        self.canvas.selection_toggle_mode = false;
    }
}


# src\ui\app\petri_app\selection\collect_selected_arc_ids.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_arc_ids(&self) -> Vec<u64> {
        let mut arc_ids = self.canvas.selected_arcs.clone();
        if let Some(id) = self.canvas.selected_arc {
            arc_ids.push(id);
        }
        arc_ids.sort_unstable();
        arc_ids.dedup();
        arc_ids
    }
}


# src\ui\app\petri_app\selection\collect_selected_frame_ids.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_frame_ids(&self) -> Vec<u64> {
        let mut frame_ids = self.canvas.selected_frames.clone();
        if let Some(id) = self.canvas.selected_frame {
            frame_ids.push(id);
        }
        frame_ids.sort_unstable();
        frame_ids.dedup();
        frame_ids
    }
}


# src\ui\app\petri_app\selection\collect_selected_place_ids.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_place_ids(&self) -> Vec<u64> {
        let mut place_ids = self.canvas.selected_places.clone();
        if let Some(id) = self.canvas.selected_place {
            place_ids.push(id);
        }
        place_ids.sort_unstable();
        place_ids.dedup();
        place_ids
    }
}


# src\ui\app\petri_app\selection\collect_selected_text_ids.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_text_ids(&self) -> Vec<u64> {
        let mut text_ids = self.canvas.selected_texts.clone();
        if let Some(id) = self.canvas.selected_text {
            text_ids.push(id);
        }
        text_ids.sort_unstable();
        text_ids.dedup();
        text_ids
    }
}


# src\ui\app\petri_app\selection\collect_selected_transition_ids.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn collect_selected_transition_ids(&self) -> Vec<u64> {
        let mut transition_ids = self.canvas.selected_transitions.clone();
        if let Some(id) = self.canvas.selected_transition {
            transition_ids.push(id);
        }
        transition_ids.sort_unstable();
        transition_ids.dedup();
        transition_ids
    }
}


# src\ui\app\petri_app\selection\delete_selected.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn delete_selected(&mut self) {
        let text_ids = self.collect_selected_text_ids();
        if !text_ids.is_empty() {
            self.push_undo_snapshot();
            let text_set: HashSet<u64> = text_ids.into_iter().collect();
            self.text_blocks.retain(|item| !text_set.contains(&item.id));
            self.canvas.selected_texts.clear();
            self.canvas.selected_text = None;
            return;
        }
        let frame_ids = self.collect_selected_frame_ids();
        if !frame_ids.is_empty() {
            self.push_undo_snapshot();
            let frame_set: HashSet<u64> = frame_ids.into_iter().collect();
            self.decorative_frames
                .retain(|item| !frame_set.contains(&item.id));
            self.canvas.selected_frames.clear();
            self.canvas.selected_frame = None;
            return;
        }
        let mut arc_ids = self.canvas.selected_arcs.clone();
        if let Some(arc_id) = self.canvas.selected_arc.take() {
            arc_ids.push(arc_id);
        }
        arc_ids.sort_unstable();
        arc_ids.dedup();
        if !arc_ids.is_empty() {
            self.canvas.selected_arcs.clear();
            self.push_undo_snapshot();
            self.net.arcs.retain(|a| !arc_ids.contains(&a.id));
            self.net.inhibitor_arcs.retain(|a| !arc_ids.contains(&a.id));
            self.net.rebuild_matrices_from_arcs();
            return;
        }

        let mut place_ids = self.canvas.selected_places.clone();
        let mut transition_ids = self.canvas.selected_transitions.clone();
        if let Some(id) = self.canvas.selected_place {
            place_ids.push(id);
        }
        if let Some(id) = self.canvas.selected_transition {
            transition_ids.push(id);
        }
        place_ids.sort_unstable();
        place_ids.dedup();
        transition_ids.sort_unstable();
        transition_ids.dedup();

        if !place_ids.is_empty() || !transition_ids.is_empty() {
            self.push_undo_snapshot();
            let mut place_idxs: Vec<usize> = place_ids
                .iter()
                .filter_map(|id| self.place_idx_by_id(*id))
                .collect();
            place_idxs.sort_unstable();
            place_idxs.dedup();
            for idx in place_idxs.iter().rev() {
                self.net.tables.remove_place_row(*idx);
            }
            let mut transition_idxs: Vec<usize> = transition_ids
                .iter()
                .filter_map(|id| self.transition_idx_by_id(*id))
                .collect();
            transition_idxs.sort_unstable();
            transition_idxs.dedup();
            for idx in transition_idxs.iter().rev() {
                self.net.tables.remove_transition_column(*idx);
            }
            self.net.places.retain(|p| !place_ids.contains(&p.id));
            self.net
                .transitions
                .retain(|t| !transition_ids.contains(&t.id));
            self.net
                .set_counts(self.net.places.len(), self.net.transitions.len());
            self.clear_selection();
        }
    }
}


# src\ui\app\petri_app\selection\mod.rs
﻿use super::*;

mod clear_selection;
mod collect_selected_arc_ids;
mod collect_selected_frame_ids;
mod collect_selected_place_ids;
mod collect_selected_text_ids;
mod collect_selected_transition_ids;
mod delete_selected;
mod promote_single_selection_to_multi;
mod push_undo_snapshot;
mod select_all_objects;
mod sync_primary_selection_from_multi;
mod toggle_selected_id;
mod undo_last_action;


# src\ui\app\petri_app\selection\promote_single_selection_to_multi.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn promote_single_selection_to_multi(&mut self) {
        if let Some(place_id) = self.canvas.selected_place.take() {
            if !self.canvas.selected_places.contains(&place_id) {
                self.canvas.selected_places.push(place_id);
            }
        }
        if let Some(transition_id) = self.canvas.selected_transition.take() {
            if !self.canvas.selected_transitions.contains(&transition_id) {
                self.canvas.selected_transitions.push(transition_id);
            }
        }
        if let Some(arc_id) = self.canvas.selected_arc.take() {
            if !self.canvas.selected_arcs.contains(&arc_id) {
                self.canvas.selected_arcs.push(arc_id);
            }
        }
        if let Some(text_id) = self.canvas.selected_text.take() {
            if !self.canvas.selected_texts.contains(&text_id) {
                self.canvas.selected_texts.push(text_id);
            }
        }
        if let Some(frame_id) = self.canvas.selected_frame.take() {
            if !self.canvas.selected_frames.contains(&frame_id) {
                self.canvas.selected_frames.push(frame_id);
            }
        }
    }
}


# src\ui\app\petri_app\selection\push_undo_snapshot.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn push_undo_snapshot(&mut self) {
        self.undo_stack.push(UndoSnapshot {
            net: self.net.clone(),
            text_blocks: self.text_blocks.clone(),
            next_text_id: self.next_text_id,
            decorative_frames: self.decorative_frames.clone(),
            next_frame_id: self.next_frame_id,
        });
        // Keep memory bounded.
        if self.undo_stack.len() > 64 {
            self.undo_stack.remove(0);
        }
    }
}


# src\ui\app\petri_app\selection\select_all_objects.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn select_all_objects(&mut self) {
        self.canvas.selected_place = None;
        self.canvas.selected_transition = None;
        self.canvas.selected_places = self.net.places.iter().map(|place| place.id).collect();
        self.canvas.selected_transitions = self.net.transitions.iter().map(|tr| tr.id).collect();
        self.canvas.selected_arcs = self.net.arcs.iter().map(|arc| arc.id).collect();
        self.canvas
            .selected_arcs
            .extend(self.net.inhibitor_arcs.iter().map(|arc| arc.id));
        self.canvas.selected_arc = self.canvas.selected_arcs.first().copied();
        self.canvas.selected_texts = self.text_blocks.iter().map(|text| text.id).collect();
        self.canvas.selected_text = self.canvas.selected_texts.first().copied();
        self.canvas.selected_frames = self
            .decorative_frames
            .iter()
            .map(|frame| frame.id)
            .collect();
        self.canvas.selected_frame = self.canvas.selected_frames.first().copied();
    }
}


# src\ui\app\petri_app\selection\sync_primary_selection_from_multi.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn sync_primary_selection_from_multi(&mut self) {
        self.canvas.selected_place = self.canvas.selected_places.last().copied();
        self.canvas.selected_transition = self.canvas.selected_transitions.last().copied();
        self.canvas.selected_arc = self.canvas.selected_arcs.last().copied();
        self.canvas.selected_text = self.canvas.selected_texts.last().copied();
        self.canvas.selected_frame = self.canvas.selected_frames.last().copied();
    }
}


# src\ui\app\petri_app\selection\toggle_selected_id.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn toggle_selected_id(ids: &mut Vec<u64>, id: u64) -> bool {
        if let Some(idx) = ids.iter().position(|&value| value == id) {
            ids.remove(idx);
            false
        } else {
            ids.push(id);
            true
        }
    }
}


# src\ui\app\petri_app\selection\undo_last_action.rs
use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn undo_last_action(&mut self) {
        let Some(state) = self.undo_stack.pop() else {
            return;
        };
        self.net = state.net;
        self.text_blocks = state.text_blocks;
        self.next_text_id = state.next_text_id;
        self.decorative_frames = state.decorative_frames;
        self.next_frame_id = state.next_frame_id;
        self.clear_selection();
    }
}


# src\ui\app\shortcuts.rs
use super::*;

impl PetriApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let mut do_new = false;
        let mut do_open = false;
        let mut do_save = false;
        let mut do_exit = false;
        let mut do_delete = false;
        let mut do_copy = false;
        let mut do_paste = false;
        let mut do_undo = false;
        let mut do_select_all = false;
        let mut do_clear_selection = false;

        ctx.input(|i| {
            do_new = i.modifiers.command && i.key_pressed(egui::Key::N);
            do_open = i.modifiers.command && i.key_pressed(egui::Key::O);
            do_save = i.modifiers.command && i.key_pressed(egui::Key::S);
            do_exit = i.modifiers.command && i.key_pressed(egui::Key::Q);
            do_delete = i.key_pressed(egui::Key::Delete);
            // Strict shortcuts: only Ctrl+key where Ctrl is already held.
            do_copy = i.modifiers.ctrl && i.key_pressed(egui::Key::C);
            do_paste = i.modifiers.ctrl && i.key_pressed(egui::Key::V);
            do_undo = i.modifiers.ctrl && i.key_pressed(egui::Key::Z);
            do_select_all = i.modifiers.ctrl && i.key_pressed(egui::Key::A);
            do_clear_selection = i.key_pressed(egui::Key::Escape);

            // Layout fallback (RU keyboard), still requiring Ctrl held.
            for e in &i.events {
                match e {
                    egui::Event::Copy => do_copy = true,
                    egui::Event::Paste(_) => do_paste = true,
                    _ => {}
                }
                if let egui::Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    modifiers,
                    ..
                } = e
                {
                    if modifiers.ctrl
                        && (*key == egui::Key::C || *physical_key == Some(egui::Key::C))
                    {
                        do_copy = true;
                    }
                    if modifiers.ctrl
                        && (*key == egui::Key::V || *physical_key == Some(egui::Key::V))
                    {
                        do_paste = true;
                    }
                    if modifiers.ctrl
                        && (*key == egui::Key::Z || *physical_key == Some(egui::Key::Z))
                    {
                        do_undo = true;
                    }
                    if modifiers.ctrl
                        && (*key == egui::Key::A || *physical_key == Some(egui::Key::A))
                    {
                        do_select_all = true;
                    }
                    if *key == egui::Key::Escape || *physical_key == Some(egui::Key::Escape) {
                        do_clear_selection = true;
                    }
                }
                if let egui::Event::Text(text) = e {
                    if i.modifiers.ctrl {
                        if text.eq_ignore_ascii_case("c") || matches!(text.as_str(), "с" | "С") {
                            do_copy = true;
                        }
                        if text.eq_ignore_ascii_case("v") || matches!(text.as_str(), "м" | "М") {
                            do_paste = true;
                        }
                        if text.eq_ignore_ascii_case("z") || matches!(text.as_str(), "я" | "Я") {
                            do_undo = true;
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            {
                do_exit = do_exit || (i.modifiers.command && i.key_pressed(egui::Key::X));
            }
        });

        // Additional low-level key consumption to survive integrations where key_pressed/modifiers are flaky.
        ctx.input_mut(|i| {
            do_copy = do_copy || i.consume_key(egui::Modifiers::CTRL, egui::Key::C);
            do_paste = do_paste || i.consume_key(egui::Modifiers::CTRL, egui::Key::V);
            do_undo = do_undo || i.consume_key(egui::Modifiers::CTRL, egui::Key::Z);
            do_select_all = do_select_all || i.consume_key(egui::Modifiers::CTRL, egui::Key::A);
            do_clear_selection =
                do_clear_selection || i.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
        });

        if do_new {
            self.new_file();
        }
        if do_open {
            self.open_file();
        }
        if do_save {
            self.save_file();
        }
        if do_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if do_delete {
            self.delete_selected();
        }
        if do_copy {
            self.copy_selected_objects();
        }
        if do_paste {
            self.paste_copied_objects();
        }
        if do_undo {
            self.undo_last_action();
        }
        if do_select_all {
            self.select_all_objects();
        }
        if do_clear_selection {
            self.clear_selection();
            self.canvas.arc_start = None;
        }
    }
}


# src\ui\app\table_view.rs
use super::*;

impl PetriApp {
    pub(super) fn draw_table_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Структура сети");
        ui.horizontal(|ui| {
            if ui.button("Скрыть структуру").clicked() {
                self.show_table_view = false;
                self.table_fullscreen = false;
            }
            if ui
                .button(if self.table_fullscreen {
                    "Обычный режим"
                } else {
                    "Полный экран"
                })
                .clicked()
            {
                self.table_fullscreen = !self.table_fullscreen;
            }
        });
        ui.separator();
        if !self.show_table_view {
            return;
        }
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        let vector_scroll_height = 220.0;
        let matrix_scroll_height = 320.0;

        ui.horizontal(|ui| {
            ui.label("Показывать:");
            let vectors_label = self.tr("Векторы", "Vectors");
            let pre_label = self.tr("Матрица Pre", "Pre matrix");
            let post_label = self.tr("Матрица Post", "Post matrix");
            let inhibitor_label = self.tr("Ингибиторные дуги", "Inhibitor matrix");
            ui.checkbox(&mut self.show_struct_vectors, vectors_label);
            ui.checkbox(&mut self.show_struct_pre, pre_label);
            ui.checkbox(&mut self.show_struct_post, post_label);
            ui.checkbox(&mut self.show_struct_inhibitor, inhibitor_label);
        });

        let mut p_count = self.net.places.len() as i32;
        let mut t_count = self.net.transitions.len() as i32;
        ui.horizontal(|ui| {
            ui.label("Места:");
            ui.add(egui::DragValue::new(&mut p_count).range(0..=200));
            ui.label("Переходы:");
            ui.add(egui::DragValue::new(&mut t_count).range(0..=200));
            if ui.button("Применить количество").clicked() {
                self.net
                    .set_counts(p_count.max(0) as usize, t_count.max(0) as usize);
            }
        });

        let row_label_w = 46.0;
        let cell_w = 42.0;
        egui::ScrollArea::both().show(ui, |ui| {
            if self.show_struct_vectors {
                ui.separator();
                ui.label("Вектор начальной маркировки (M0)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("m0_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("m0_grid").striped(true).show(ui, |ui| {
                            for i in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", i + 1)),
                                );
                                ui.add_sized(
                                    [cell_w * 1.4, 0.0],
                                    egui::DragValue::new(&mut self.net.tables.m0[i])
                                        .range(0..=u32::MAX),
                                );
                                ui.end_row();
                            }
                        });
                    },
                );

                ui.separator();
                ui.label("Вектор максимальных емкостей (Mo)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("mo_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("mo_grid").striped(true).show(ui, |ui| {
                            for i in rows {
                                let mut cap = self.net.tables.mo[i].unwrap_or(0);
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", i + 1)),
                                );
                                if ui
                                    .add_sized(
                                        [cell_w * 1.4, 0.0],
                                        egui::DragValue::new(&mut cap).range(0..=u32::MAX),
                                    )
                                    .changed()
                                {
                                    self.net.tables.mo[i] = if cap == 0 { None } else { Some(cap) };
                                }
                                ui.end_row();
                            }
                        });
                    },
                );

                ui.separator();
                ui.label("Вектор временных задержек в позициях (Mz)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("mz_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("mz_grid").striped(true).show(ui, |ui| {
                            for i in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", i + 1)),
                                );
                                ui.add_sized(
                                    [cell_w * 1.8, 0.0],
                                    egui::DragValue::new(&mut self.net.tables.mz[i])
                                        .speed(0.1)
                                        .range(0.0..=10_000.0),
                                );
                                ui.end_row();
                            }
                        });
                    },
                );

                ui.separator();
                ui.label("Вектор приоритетов переходов (Mpr)");
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("mpr_grid_scroll"),
                    self.net.transitions.len(),
                    row_h,
                    vector_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("mpr_grid").striped(true).show(ui, |ui| {
                            for t in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                                ui.add_sized(
                                    [cell_w * 1.8, 0.0],
                                    egui::DragValue::new(&mut self.net.tables.mpr[t]).speed(1),
                                );
                                ui.end_row();
                            }
                        });
                    },
                );
            }
            let mut matrices_changed = false;
            if self.show_struct_pre {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Матрица инцидентности Pre");
                    if ui
                        .small_button(self.tr("Импорт CSV", "Import CSV"))
                        .clicked()
                    {
                        self.import_matrix_csv(MatrixCsvTarget::Pre);
                    }
                });
                let mut pre_changed = false;
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("pre_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("pre_grid").striped(true).show(ui, |ui| {
                            ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                            for t in 0..self.net.transitions.len() {
                                ui.add_sized(
                                    [cell_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                            }
                            ui.end_row();
                            for p in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", p + 1)),
                                );
                                for t in 0..self.net.transitions.len() {
                                    pre_changed |= ui
                                        .add_sized(
                                            [cell_w, 0.0],
                                            egui::DragValue::new(&mut self.net.tables.pre[p][t])
                                                .range(0..=u32::MAX)
                                                .speed(1),
                                        )
                                        .changed();
                                }
                                ui.end_row();
                            }
                        });
                    },
                );
                matrices_changed |= pre_changed;
            }
            if self.show_struct_post {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Матрица инцидентности Post");
                    if ui
                        .small_button(self.tr("Импорт CSV", "Import CSV"))
                        .clicked()
                    {
                        self.import_matrix_csv(MatrixCsvTarget::Post);
                    }
                });
                let mut post_changed = false;
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("post_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("post_grid").striped(true).show(ui, |ui| {
                            ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                            for t in 0..self.net.transitions.len() {
                                ui.add_sized(
                                    [cell_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                            }
                            ui.end_row();
                            for p in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", p + 1)),
                                );
                                for t in 0..self.net.transitions.len() {
                                    post_changed |= ui
                                        .add_sized(
                                            [cell_w, 0.0],
                                            egui::DragValue::new(&mut self.net.tables.post[p][t])
                                                .range(0..=u32::MAX)
                                                .speed(1),
                                        )
                                        .changed();
                                }
                                ui.end_row();
                            }
                        });
                    },
                );
                matrices_changed |= post_changed;
            }
            if self.show_struct_inhibitor {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Матрица ингибиторных дуг");
                    if ui
                        .small_button(self.tr("Импорт CSV", "Import CSV"))
                        .clicked()
                    {
                        self.import_matrix_csv(MatrixCsvTarget::Inhibitor);
                    }
                });
                let mut inhibitor_changed = false;
                Self::scroll_area_rows(
                    ui,
                    egui::Id::new("inh_grid_scroll"),
                    self.net.places.len(),
                    row_h,
                    matrix_scroll_height,
                    |ui, rows| {
                        egui::Grid::new("inh_grid").striped(true).show(ui, |ui| {
                            ui.add_sized([row_label_w, 0.0], egui::Label::new(""));
                            for t in 0..self.net.transitions.len() {
                                ui.add_sized(
                                    [cell_w, 0.0],
                                    egui::Label::new(format!("T{}", t + 1)),
                                );
                            }
                            ui.end_row();
                            for p in rows {
                                ui.add_sized(
                                    [row_label_w, 0.0],
                                    egui::Label::new(format!("P{}", p + 1)),
                                );
                                for t in 0..self.net.transitions.len() {
                                    inhibitor_changed |= ui
                                        .add_sized(
                                            [cell_w, 0.0],
                                            egui::DragValue::new(
                                                &mut self.net.tables.inhibitor[p][t],
                                            )
                                            .range(0..=u32::MAX)
                                            .speed(1),
                                        )
                                        .changed();
                                }
                                ui.end_row();
                            }
                        });
                    },
                );
                matrices_changed |= inhibitor_changed;
            }
            if matrices_changed {
                self.net.rebuild_arcs_from_matrices();
            }
        });
    }

    pub(super) fn draw_sim_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_sim_params;
        let mut close_now = false;
        egui::Window::new(self.tr("Параметры симуляции", "Simulation Parameters"))
            .open(&mut open)
            .resizable(true)
            .default_size(egui::vec2(420.0, 520.0))
            .min_size(egui::vec2(360.0, 320.0))
            .show(ctx, |ui| {
                let mut corrected_inputs = false;

                let pass_limit_label = self.tr("Лимит срабатываний", "Fire count limit");
                ui.checkbox(&mut self.sim_params.use_pass_limit, pass_limit_label);
                ui.add_enabled(
                    self.sim_params.use_pass_limit,
                    egui::DragValue::new(&mut self.sim_params.pass_limit).range(0..=u64::MAX),
                );
                corrected_inputs |= sanitize_u64(&mut self.sim_params.pass_limit, 0, 1_000_000);

                let time_limit_label = self.tr("Лимит времени (сек)", "Time limit (sec)");
                ui.checkbox(&mut self.sim_params.use_time_limit, time_limit_label);
                ui.add_enabled(
                    self.sim_params.use_time_limit,
                    egui::DragValue::new(&mut self.sim_params.time_limit)
                        .range(0.0..=1_000_000.0)
                        .speed(1.0),
                );
                corrected_inputs |= sanitize_f64(&mut self.sim_params.time_limit, 0.0, 1_000_000.0);

                ui.separator();
                ui.label(self.tr("Условия остановки", "Stop conditions"));
                let mut stop_place_enabled = self.sim_params.stop.through_place.is_some();
                let stop_place_label = self.tr(
                    "Через место Pk прошло N маркеров",
                    "N tokens passed through place Pk",
                );
                ui.checkbox(&mut stop_place_enabled, stop_place_label);
                if stop_place_enabled {
                    let (mut p, mut n) = self.sim_params.stop.through_place.unwrap_or((0, 1));
                    let max_place_idx = self.net.places.len().saturating_sub(1);
                    ui.horizontal(|ui| {
                        ui.label(self.tr("Pk (k-1)", "Pk (k-1)"));
                        ui.add(egui::DragValue::new(&mut p).range(0..=max_place_idx));
                        ui.label("N");
                        ui.add(egui::DragValue::new(&mut n).range(1..=u64::MAX));
                    });
                    corrected_inputs |= sanitize_usize(&mut p, 0, max_place_idx);
                    corrected_inputs |= sanitize_u64(&mut n, 1, 1_000_000);
                    p = p.min(max_place_idx);
                    self.sim_params.stop.through_place = Some((p, n));
                } else {
                    self.sim_params.stop.through_place = None;
                }

                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
                if ui.button(self.tr("СТАРТ", "START")).clicked() {
                    self.net.sanitize_values();
                    self.net.rebuild_matrices_from_arcs();
                    self.sim_result = Some(std::sync::Arc::new(run_simulation(
                        &self.net,
                        &self.sim_params,
                        false,
                        self.net.ui.marker_count_stats,
                    )));
                    self.calculate_markov_model();
                    self.refresh_debug_animation_state();
                    self.debug_step = 0;
                    self.sync_debug_animation_for_step();
                    self.debug_playing = false;
                    self.show_results = true;
                    self.show_place_stats_window = false;
                    self.show_sim_params = false;
                    close_now = true;
                }
            });
        if close_now {
            open = false;
        }
        self.show_sim_params = open;
    }

    pub(super) fn draw_results(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.sim_result.clone() {
            let mut open = self.show_results;
            egui::Window::new(self.tr("Результаты/Статистика", "Results/Statistics"))
                .open(&mut open)
                .resizable(true)
                .default_size(egui::vec2(1120.0, 760.0))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("results_window_scroll")
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.label(match result.cycle_time {
                                Some(t) => format!(
                                    "{}: {:.6} {}",
                                    self.tr("Время цикла", "Cycle time"),
                                    t,
                                    self.tr("сек", "sec")
                                ),
                                None => format!("{}: N/A", self.tr("Время цикла", "Cycle time")),
                            });
                            let total_minutes = result.sim_time / 60.0;
                            ui.label(format!(
                                "{}: {:.4} {} / {:.4} {}",
                                self.tr("Итоговое время эмуляции", "Total simulation time"),
                                result.sim_time,
                                self.tr("сек", "sec"),
                                total_minutes,
                                self.tr("мин", "min")
                            ));
                            ui.label(format!(
                                "{}: {}",
                                self.tr("Сработало переходов", "Fired transitions"),
                                result.fired_count
                            ));
                            if result.log_entries_total > result.logs.len() {
                                ui.label(format!(
                                    "{}: {} / {} ({})",
                                    self.tr("Журнал сэмплирован", "Log sampled"),
                                    result.logs.len(),
                                    result.log_entries_total,
                                    self.tr("шаг сэмплирования", "sampling stride"),
                                ));
                            }

                            let stats_places: Vec<usize> = self
                                .net
                                .places
                                .iter()
                                .enumerate()
                                .filter_map(|(idx, place)| {
                                    if place.stats.any_enabled() {
                                        Some(idx)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !stats_places.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(self.tr(
                                        "Детальная статистика по позициям доступна",
                                        "Detailed per-place statistics available",
                                    ));
                                    if ui.button(self.tr("Статистика", "Statistics")).clicked()
                                    {
                                        let selected = stats_places
                                            .iter()
                                            .position(|&p| p == self.place_stats_view_place)
                                            .unwrap_or(0);
                                        self.place_stats_view_place = stats_places[selected];
                                        self.show_place_stats_window = true;
                                    }
                                });
                            }

                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(self.tr("Журнал (таблица)", "Log (table)"));
                                if ui.button(self.tr("Экспорт CSV", "Export CSV")).clicked()
                                {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("CSV", &["csv"])
                                        .set_file_name("simulation_log.csv")
                                        .save_file()
                                    {
                                        let mut csv = String::new();
                                        csv.push_str("time");
                                        for (p, _) in self.net.places.iter().enumerate() {
                                            csv.push(',');
                                            csv.push_str(&format!("P{}", p + 1));
                                        }
                                        csv.push('\n');
                                        for entry in &result.logs {
                                            csv.push_str(&format!("{:.6}", entry.time));
                                            for token in &entry.marking {
                                                csv.push(',');
                                                csv.push_str(&token.to_string());
                                            }
                                            csv.push('\n');
                                        }
                                        match std::fs::write(&path, csv) {
                                            Ok(_) => {
                                                self.status_hint = Some(format!(
                                                    "{}: {}",
                                                    self.tr("Журнал экспортирован", "Log exported"),
                                                    path.display()
                                                ));
                                                self.last_error = None;
                                            }
                                            Err(e) => {
                                                self.last_error = Some(format!(
                                                    "{}: {}",
                                                    self.tr(
                                                        "Ошибка экспорта CSV",
                                                        "CSV export error"
                                                    ),
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                            });
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                egui::Grid::new("sim_log_grid_header").striped(true).show(
                                    ui,
                                    |ui| {
                                        ui.label(self.tr("Время", "Time"));
                                        for (p, _) in self.net.places.iter().enumerate() {
                                            ui.label(format!("P{}", p + 1));
                                        }
                                        ui.end_row();
                                    },
                                );

                                let visible_log_indices = Self::debug_visible_log_indices(&result);
                                egui::ScrollArea::vertical().max_height(320.0).show_rows(
                                    ui,
                                    row_h,
                                    visible_log_indices.len(),
                                    |ui, range| {
                                        egui::Grid::new("sim_log_grid_rows").striped(true).show(
                                            ui,
                                            |ui| {
                                                for row_idx in range {
                                                    let entry =
                                                        &result.logs[visible_log_indices[row_idx]];
                                                    ui.label(format!("{:.3}", entry.time));
                                                    for token in &entry.marking {
                                                        ui.label(token.to_string());
                                                    }
                                                    ui.end_row();
                                                }
                                            },
                                        );
                                    },
                                );
                            });

                            let any_place_stats_selected =
                                self.net.places.iter().any(|p| p.stats.any_enabled());
                            let show_all_places_in_stats = !any_place_stats_selected;

                            if let Some(stats) = &result.place_stats {
                                ui.separator();
                                ui.label(self.tr(
                                    "Статистика маркеров (min/max/avg)",
                                    "Token statistics (min/max/avg)",
                                ));
                                let rows: Vec<usize> = stats
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(p, _)| {
                                        let selected = self
                                            .net
                                            .places
                                            .get(p)
                                            .map(|pl| pl.stats.markers_total)
                                            .unwrap_or(false);
                                        if show_all_places_in_stats || selected {
                                            Some(p)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                egui::Grid::new("stats_grid_header")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(self.tr("Позиция", "Place"));
                                        ui.label("Min");
                                        ui.label("Max");
                                        ui.label("Avg");
                                        ui.end_row();
                                    });
                                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                egui::ScrollArea::vertical()
                                    .id_source("stats_grid_scroll")
                                    .max_height(180.0)
                                    .show_rows(ui, row_h, rows.len(), |ui, range| {
                                        egui::Grid::new("stats_grid_rows").striped(true).show(
                                            ui,
                                            |ui| {
                                                for row_idx in range {
                                                    let p = rows[row_idx];
                                                    let st = &stats[p];
                                                    ui.label(format!("P{}", p + 1));
                                                    ui.label(st.min.to_string());
                                                    ui.label(st.max.to_string());
                                                    ui.label(format!("{:.3}", st.avg));
                                                    ui.end_row();
                                                }
                                            },
                                        );
                                    });
                            }

                            if let Some(flow) = &result.place_flow {
                                let want_flow =
                                    show_all_places_in_stats
                                        || self.net.places.iter().any(|p| {
                                            p.stats.markers_input || p.stats.markers_output
                                        });
                                if want_flow {
                                    ui.separator();
                                    ui.label(self.tr("Потоки (вход/выход)", "Flows (in/out)"));
                                    let rows: Vec<usize> = flow
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(p, _)| {
                                            let selected = self
                                                .net
                                                .places
                                                .get(p)
                                                .map(|pl| {
                                                    pl.stats.markers_input
                                                        || pl.stats.markers_output
                                                })
                                                .unwrap_or(false);
                                            if show_all_places_in_stats || selected {
                                                Some(p)
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    egui::Grid::new("flow_grid_header").striped(true).show(
                                        ui,
                                        |ui| {
                                            ui.label(self.tr("Позиция", "Place"));
                                            ui.label(self.tr("Вход", "In"));
                                            ui.label(self.tr("Выход", "Out"));
                                            ui.end_row();
                                        },
                                    );
                                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                    egui::ScrollArea::vertical()
                                        .id_source("flow_grid_scroll")
                                        .max_height(180.0)
                                        .show_rows(ui, row_h, rows.len(), |ui, range| {
                                            egui::Grid::new("flow_grid_rows").striped(true).show(
                                                ui,
                                                |ui| {
                                                    for row_idx in range {
                                                        let p = rows[row_idx];
                                                        let st = &flow[p];
                                                        ui.label(format!("P{}", p + 1));
                                                        ui.label(st.in_tokens.to_string());
                                                        ui.label(st.out_tokens.to_string());
                                                        ui.end_row();
                                                    }
                                                },
                                            );
                                        });
                                }
                            }

                            if let Some(load) = &result.place_load {
                                let want_load = show_all_places_in_stats
                                    || self.net.places.iter().any(|p| {
                                        p.stats.load_total
                                            || p.stats.load_input
                                            || p.stats.load_output
                                    });
                                if want_load {
                                    ui.separator();
                                    ui.label(self.tr("Загруженность", "Load"));
                                    let rows: Vec<usize> = load
                                        .iter()
                                        .enumerate()
                                        .filter_map(|(p, _)| {
                                            let selected = self
                                                .net
                                                .places
                                                .get(p)
                                                .map(|pl| {
                                                    pl.stats.load_total
                                                        || pl.stats.load_input
                                                        || pl.stats.load_output
                                                })
                                                .unwrap_or(false);
                                            if show_all_places_in_stats || selected {
                                                Some(p)
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    egui::Grid::new("load_grid_header").striped(true).show(
                                        ui,
                                        |ui| {
                                            ui.label(self.tr("Позиция", "Place"));
                                            ui.label(self.tr("Общая", "Total"));
                                            ui.label(self.tr("Вход", "Input"));
                                            ui.label(self.tr("Выход", "Output"));
                                            ui.end_row();
                                        },
                                    );
                                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                                    egui::ScrollArea::vertical()
                                        .id_source("load_grid_scroll")
                                        .max_height(180.0)
                                        .show_rows(ui, row_h, rows.len(), |ui, range| {
                                            egui::Grid::new("load_grid_rows").striped(true).show(
                                                ui,
                                                |ui| {
                                                    for row_idx in range {
                                                        let p = rows[row_idx];
                                                        let st = &load[p];
                                                        ui.label(format!("P{}", p + 1));
                                                        ui.label(match st.avg_over_capacity {
                                                            Some(v) => format!("{:.3}", v),
                                                            None => "N/A".to_string(),
                                                        });
                                                        ui.label(match st.in_rate {
                                                            Some(v) => format!("{:.3}", v),
                                                            None => "N/A".to_string(),
                                                        });
                                                        ui.label(match st.out_rate {
                                                            Some(v) => format!("{:.3}", v),
                                                            None => "N/A".to_string(),
                                                        });
                                                        ui.end_row();
                                                    }
                                                },
                                            );
                                        });
                                }
                            }
                        });
                });
            self.show_results = open;
        }
    }

    pub(super) fn draw_place_statistics_window(&mut self, ctx: &egui::Context) {
        if !self.show_place_stats_window {
            return;
        }
        let Some(result) = self.sim_result.clone() else {
            self.show_place_stats_window = false;
            return;
        };

        let available_places: Vec<usize> = self
            .net
            .places
            .iter()
            .enumerate()
            .filter_map(|(idx, place)| place.stats.any_enabled().then_some(idx))
            .collect();
        if available_places.is_empty() {
            self.show_place_stats_window = false;
            return;
        }
        if !available_places.contains(&self.place_stats_view_place) {
            self.place_stats_view_place = available_places[0];
        }

        let mut open = self.show_place_stats_window;
        egui::Window::new(self.tr("Статистика", "Statistics"))
            .id(egui::Id::new("results_place_stats_window"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(self.tr("Позиция", "Place"));
                    let selected_place_text = self
                        .net
                        .places
                        .get(self.place_stats_view_place)
                        .map(|p| {
                            format!(
                                "P{} | {}",
                                self.place_stats_view_place + 1,
                                if p.name.is_empty() {
                                    format!("P{}", self.place_stats_view_place + 1)
                                } else {
                                    p.name.clone()
                                }
                            )
                        })
                        .unwrap_or_else(|| format!("P{}", self.place_stats_view_place + 1));
                    egui::ComboBox::from_id_source("results_stats_place_combo")
                        .selected_text(selected_place_text)
                        .width(420.0)
                        .show_ui(ui, |ui| {
                            for idx in &available_places {
                                let label = self
                                    .net
                                    .places
                                    .get(*idx)
                                    .map(|p| {
                                        format!(
                                            "P{} | {}",
                                            *idx + 1,
                                            if p.name.is_empty() {
                                                format!("P{}", *idx + 1)
                                            } else {
                                                p.name.clone()
                                            }
                                        )
                                    })
                                    .unwrap_or_else(|| format!("P{}", *idx + 1));
                                ui.selectable_value(&mut self.place_stats_view_place, *idx, label);
                            }
                        });
                    ui.label(format!("P{}", self.place_stats_view_place + 1));
                    ui.separator();
                    let selected_name = self
                        .net
                        .places
                        .get(self.place_stats_view_place)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| format!("P{}", self.place_stats_view_place + 1));
                    ui.label(selected_name);
                });

                let place_idx = self.place_stats_view_place;
                let place_stats = self
                    .net
                    .places
                    .get(place_idx)
                    .map(|p| p.stats)
                    .unwrap_or_default();
                let mut available_series = Vec::new();
                if place_stats.markers_total {
                    available_series.push(PlaceStatsSeries::Total);
                }
                if place_stats.markers_input {
                    available_series.push(PlaceStatsSeries::Input);
                }
                if place_stats.markers_output {
                    available_series.push(PlaceStatsSeries::Output);
                }
                if available_series.is_empty() {
                    available_series.push(PlaceStatsSeries::Total);
                }
                if !available_series.contains(&self.place_stats_series) {
                    self.place_stats_series = available_series[0];
                }
                ui.horizontal(|ui| {
                    ui.label(self.tr("Показатель", "Metric"));
                    for series in available_series {
                        let label = match series {
                            PlaceStatsSeries::Total => self.tr("Общая", "Total"),
                            PlaceStatsSeries::Input => self.tr("На входе", "On input"),
                            PlaceStatsSeries::Output => self.tr("На выходе", "On output"),
                        };
                        ui.selectable_value(&mut self.place_stats_series, series, label);
                    }
                });

                let sampled = Self::sampled_indices(result.logs.len(), Self::MAX_PLOT_POINTS);
                let mut values = Vec::<f64>::with_capacity(sampled.len());
                let mut times = Vec::<f64>::with_capacity(sampled.len());

                let mut cumulative_in = vec![0_u64; result.logs.len()];
                let mut cumulative_out = vec![0_u64; result.logs.len()];
                let mut in_sum = 0_u64;
                let mut out_sum = 0_u64;
                for (idx, entry) in result.logs.iter().enumerate() {
                    if let Some(t_idx) = entry.fired_transition {
                        in_sum = in_sum.saturating_add(
                            *self
                                .net
                                .tables
                                .post
                                .get(place_idx)
                                .and_then(|row| row.get(t_idx))
                                .unwrap_or(&0) as u64,
                        );
                        out_sum = out_sum.saturating_add(
                            *self
                                .net
                                .tables
                                .pre
                                .get(place_idx)
                                .and_then(|row| row.get(t_idx))
                                .unwrap_or(&0) as u64,
                        );
                    }
                    cumulative_in[idx] = in_sum;
                    cumulative_out[idx] = out_sum;
                }

                for idx in sampled {
                    let entry = &result.logs[idx];
                    let value = match self.place_stats_series {
                        PlaceStatsSeries::Total => {
                            entry.marking.get(place_idx).copied().unwrap_or_default() as f64
                        }
                        PlaceStatsSeries::Input => {
                            cumulative_in.get(idx).copied().unwrap_or_default() as f64
                        }
                        PlaceStatsSeries::Output => {
                            cumulative_out.get(idx).copied().unwrap_or_default() as f64
                        }
                    };
                    values.push(value);
                    times.push(if entry.time.is_finite() {
                        entry.time
                    } else {
                        idx as f64
                    });
                }

                if values.len() >= 2 {
                    let mut has_increasing_x = false;
                    for i in 1..times.len() {
                        if times[i] > times[i - 1] {
                            has_increasing_x = true;
                            break;
                        }
                    }
                    if !has_increasing_x {
                        for (i, t) in times.iter_mut().enumerate() {
                            *t = i as f64;
                        }
                    }
                }
                if values.is_empty() {
                    ui.label(self.tr("Нет данных для отображения", "No data to display"));
                    return;
                }
                if result.logs.len() > values.len() {
                    ui.label(format!(
                        "{}: {} / {}",
                        self.tr("График сэмплирован", "Plot sampled"),
                        values.len(),
                        result.logs.len()
                    ));
                }

                let mut max_v = values[0];
                let mut min_v = values[0];
                let mut max_t = times[0];
                let mut min_t = times[0];
                let mut sum = 0.0;
                for (v, t) in values.iter().zip(times.iter()) {
                    sum += *v;
                    if *v > max_v {
                        max_v = *v;
                        max_t = *t;
                    }
                    if *v < min_v {
                        min_v = *v;
                        min_t = *t;
                    }
                }
                let avg = sum / values.len() as f64;
                let place_load = result
                    .place_load
                    .as_ref()
                    .and_then(|load| load.get(place_idx));
                let summary_tail = match self.place_stats_series {
                    PlaceStatsSeries::Total => format!(
                        "{} {:.3}%",
                        self.tr("Утилизация", "Utilization"),
                        place_load
                            .and_then(|l| l.avg_over_capacity)
                            .map(|v| v * 100.0)
                            .unwrap_or(0.0)
                    ),
                    PlaceStatsSeries::Input => format!(
                        "{} {:.3}",
                        self.tr("Ср. вход/сек", "Avg in/sec"),
                        place_load.and_then(|l| l.in_rate).unwrap_or(0.0)
                    ),
                    PlaceStatsSeries::Output => format!(
                        "{} {:.3}",
                        self.tr("Ср. выход/сек", "Avg out/sec"),
                        place_load.and_then(|l| l.out_rate).unwrap_or(0.0)
                    ),
                };

                ui.horizontal(|ui| {
                    ui.label(format!("{} {:.3}", self.tr("Максимум", "Maximum"), max_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), max_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Минимум", "Minimum"), min_v));
                    ui.label(format!("{} {:.3}", self.tr("Время", "Time"), min_t));
                    ui.separator();
                    ui.label(format!("{} {:.3}", self.tr("Среднее", "Average"), avg));
                    ui.label(summary_tail);
                });
                ui.horizontal(|ui| {
                    ui.label(self.tr("Масштаб X", "X zoom"));
                    ui.add(
                        egui::Slider::new(&mut self.place_stats_zoom_x, 1.0..=20.0)
                            .logarithmic(true),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.place_stats_zoom_x)
                            .range(1.0..=20.0)
                            .speed(0.01)
                            .fixed_decimals(3),
                    );
                    ui.label(self.tr("Сдвиг X", "X pan"));
                    ui.add(egui::Slider::new(&mut self.place_stats_pan_x, 0.0..=1.0));
                    ui.add(
                        egui::DragValue::new(&mut self.place_stats_pan_x)
                            .range(0.0..=1.0)
                            .speed(0.001)
                            .fixed_decimals(3),
                    );
                    ui.separator();
                    let grid_label = self.tr("Показать сетку", "Show grid");
                    ui.checkbox(&mut self.place_stats_show_grid, grid_label);
                });

                let total = values.len();
                let visible = (((total as f32) / self.place_stats_zoom_x).round() as usize)
                    .clamp(2, total.max(2));
                let max_start = total.saturating_sub(visible);
                let start = ((max_start as f32) * self.place_stats_pan_x)
                    .round()
                    .clamp(0.0, max_start as f32) as usize;
                let end = (start + visible).min(total);
                let values_window = &values[start..end];
                let times_window = &times[start..end];

                let desired_size = egui::Vec2::new(ui.available_width(), 360.0);
                let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));
                let left_pad = 50.0;
                let right_pad = 14.0;
                let top_pad = 14.0;
                let bottom_pad = 36.0;
                let plot_rect = Rect::from_min_max(
                    Pos2::new(rect.left() + left_pad, rect.top() + top_pad),
                    Pos2::new(rect.right() - right_pad, rect.bottom() - bottom_pad),
                );
                painter.rect_stroke(plot_rect, 0.0, Stroke::new(1.0, Color32::GRAY));

                let x_min = times_window.first().copied().unwrap_or(0.0);
                let mut x_max = times_window.last().copied().unwrap_or(1.0);
                if x_max <= x_min {
                    x_max = x_min + (times_window.len().max(1) as f64);
                }
                let y_min = 0.0;
                let mut y_max = values_window
                    .iter()
                    .copied()
                    .fold(0.0_f64, |acc, v| if v > acc { v } else { acc })
                    .max(1.0);
                if y_max <= y_min {
                    y_max = y_min + 1.0;
                }

                ui.label(format!(
                    "{}: [{:.3} .. {:.3}] | {}: {} / {}",
                    self.tr("Диапазон X", "X range"),
                    x_min,
                    x_max,
                    self.tr("Точки", "Points"),
                    values_window.len(),
                    values.len()
                ));

                let x_step = ((x_max - x_min) / 10.0).max(0.000_001);
                let y_step = ((y_max - y_min) / 10.0).max(0.000_001);

                if self.place_stats_show_grid {
                    ui.label(format!(
                        "{}: {:.3} | {}: {:.3}",
                        self.tr("Шаг сетки X", "Grid step X"),
                        x_step,
                        self.tr("Шаг сетки Y", "Grid step Y"),
                        y_step
                    ));
                    for i in 1..10 {
                        let x = plot_rect.left() + plot_rect.width() * (i as f32 / 10.0);
                        painter.line_segment(
                            [
                                Pos2::new(x, plot_rect.top()),
                                Pos2::new(x, plot_rect.bottom()),
                            ],
                            Stroke::new(0.5, Color32::LIGHT_GRAY),
                        );
                    }
                    for i in 1..10 {
                        let y = plot_rect.bottom() - plot_rect.height() * (i as f32 / 10.0);
                        painter.line_segment(
                            [
                                Pos2::new(plot_rect.left(), y),
                                Pos2::new(plot_rect.right(), y),
                            ],
                            Stroke::new(0.5, Color32::LIGHT_GRAY),
                        );
                    }

                    for i in 0..=10 {
                        let t = i as f32 / 10.0;
                        let x = plot_rect.left() + plot_rect.width() * t;
                        let xv = x_min + x_step * i as f64;
                        painter.text(
                            Pos2::new(x, plot_rect.bottom() + 6.0),
                            egui::Align2::CENTER_TOP,
                            format!("{:.1}", xv),
                            egui::FontId::default(),
                            Color32::DARK_GRAY,
                        );
                    }

                    for i in 0..=10 {
                        let t = i as f32 / 10.0;
                        let y = plot_rect.bottom() - plot_rect.height() * t;
                        let yv = y_min + y_step * i as f64;
                        painter.text(
                            Pos2::new(rect.left() + 4.0, y),
                            egui::Align2::LEFT_CENTER,
                            format!("{:.1}", yv),
                            egui::FontId::default(),
                            Color32::DARK_GRAY,
                        );
                    }
                }

                let to_screen = |x: f64, y: f64| -> Pos2 {
                    let xr = ((x - x_min) / (x_max - x_min)).clamp(0.0, 1.0) as f32;
                    let yr = ((y - y_min) / (y_max - y_min)).clamp(0.0, 1.0) as f32;
                    Pos2::new(
                        plot_rect.left() + xr * plot_rect.width(),
                        plot_rect.bottom() - yr * plot_rect.height(),
                    )
                };

                let mut points_data = Vec::with_capacity(values_window.len());
                let mut line_points = Vec::with_capacity(values_window.len());
                for (x, y) in times_window.iter().zip(values_window.iter()) {
                    let pt = to_screen(*x, *y);
                    points_data.push((pt, *x, *y));
                    line_points.push(pt);
                }
                if line_points.len() >= 2 {
                    painter.add(egui::Shape::line(
                        line_points,
                        Stroke::new(1.6, Color32::BLUE),
                    ));
                }

                if let Some(mouse_pos) = response.hover_pos() {
                    if plot_rect.contains(mouse_pos) {
                        let x_tolerance = 32.0;
                        let y_tolerance = 80.0;
                        if let Some((pos, x, y)) = points_data
                            .iter()
                            .filter_map(|(pos, x, y)| {
                                let dx = (mouse_pos.x - pos.x).abs();
                                let dy = (mouse_pos.y - pos.y).abs();
                                let normalized =
                                    (dx / x_tolerance).powi(2) + (dy / y_tolerance).powi(2);
                                if normalized <= 1.0 {
                                    Some((normalized, *pos, *x, *y))
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| {
                                a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .map(|(_, pos, x, y)| (pos, x, y))
                        {
                            painter.circle_filled(pos, 4.0, Color32::WHITE);
                            painter.circle_stroke(pos, 4.0, Stroke::new(2.0, Color32::BLUE));
                            let point_label = format!(
                                "{}: {:.3}, {}: {:.3}",
                                self.tr("X", "X"),
                                x,
                                self.tr("Y", "Y"),
                                y
                            );
                            painter.text(
                                pos + Vec2::new(6.0, 12.0),
                                egui::Align2::LEFT_TOP,
                                point_label,
                                egui::FontId::default(),
                                Color32::BLACK,
                            );
                        }
                    }
                }

                if !self.place_stats_show_grid {
                    painter.text(
                        Pos2::new(rect.left() + 4.0, plot_rect.top()),
                        egui::Align2::LEFT_TOP,
                        format!("{:.3}", y_max),
                        egui::FontId::default(),
                        Color32::DARK_GRAY,
                    );
                    painter.text(
                        Pos2::new(rect.left() + 4.0, plot_rect.bottom()),
                        egui::Align2::LEFT_BOTTOM,
                        "0",
                        egui::FontId::default(),
                        Color32::DARK_GRAY,
                    );
                    painter.text(
                        Pos2::new(plot_rect.left(), plot_rect.bottom() + 6.0),
                        egui::Align2::LEFT_TOP,
                        format!("{:.3}", x_min),
                        egui::FontId::default(),
                        Color32::DARK_GRAY,
                    );
                    painter.text(
                        Pos2::new(plot_rect.right(), plot_rect.bottom() + 6.0),
                        egui::Align2::RIGHT_TOP,
                        format!("{:.3}", x_max),
                        egui::FontId::default(),
                        Color32::DARK_GRAY,
                    );
                }
                painter.text(
                    Pos2::new(plot_rect.center().x, rect.bottom() - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    self.tr("Ось X: время/шаги", "X axis: time/steps"),
                    egui::FontId::default(),
                    Color32::DARK_GRAY,
                );
            });

        self.show_place_stats_window = open;
    }

    fn scroll_area_rows<F>(
        ui: &mut egui::Ui,
        id: egui::Id,
        row_len: usize,
        row_h: f32,
        max_height: f32,
        body: F,
    ) where
        F: FnOnce(&mut egui::Ui, std::ops::Range<usize>),
    {
        if row_len == 0 {
            return;
        }
        let available = ui.available_height();
        let height = if available.is_finite() {
            max_height.min(available.max(row_h))
        } else {
            max_height.max(row_h)
        };
        egui::ScrollArea::vertical()
            .id_source(id)
            .max_height(height)
            .show_rows(ui, row_h, row_len, body);
    }
}


# src\ui\app\tool_palette.rs
use super::*;

impl PetriApp {
    pub(super) fn draw_tool_palette(&mut self, ctx: &egui::Context) {
        if self.tool == Tool::Run {
            self.tool = Tool::Edit;
        }

        let panel = egui::SidePanel::left("tools")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Инструменты");
                ui.separator();

                for (tool_variant, icon, label) in [
                    (Tool::Place, "O", "Позиция"),
                    (Tool::Transition, "II", "Переход"),
                    (Tool::Arc, "↗", "Дуга"),
                    (Tool::Text, "A", "Текст"),
                    (Tool::Frame, "[]", "Рамка"),
                    (Tool::Edit, "✥", "Редактировать"),
                    (Tool::Delete, "✖", "Удалить"),
                ] {
                    let selected = self.tool == tool_variant;
                    let text = format!("{} {}", icon, label);
                    if ui.add(egui::SelectableLabel::new(selected, text)).clicked() {
                        self.tool = tool_variant;
                    }
                }

                if ui.button("СТАРТ").clicked() {
                    self.reset_sim_stop_controls();
                    self.show_sim_params = true;
                }

                ui.separator();
                ui.label(self.tr("Отображение связей", "Link visibility"));
                let is_ru = matches!(self.net.ui.language, Language::Ru);
                egui::ComboBox::from_label(self.tr("Режим", "Mode"))
                    .selected_text(Self::arc_display_mode_text(self.arc_display_mode, is_ru))
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(
                            &mut self.arc_display_mode,
                            ArcDisplayMode::All,
                            Self::arc_display_mode_text(ArcDisplayMode::All, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.arc_display_mode,
                            ArcDisplayMode::OnlyColor,
                            Self::arc_display_mode_text(ArcDisplayMode::OnlyColor, is_ru),
                        );
                        ui.selectable_value(
                            &mut self.arc_display_mode,
                            ArcDisplayMode::Hidden,
                            Self::arc_display_mode_text(ArcDisplayMode::Hidden, is_ru),
                        );
                    });

                if self.arc_display_mode == ArcDisplayMode::OnlyColor {
                    let color_label = if is_ru { "Цвет" } else { "Color" };
                    let c_default = if is_ru {
                        "По умолчанию"
                    } else {
                        "Default"
                    };
                    let c_blue = if is_ru { "Синий" } else { "Blue" };
                    let c_red = if is_ru { "Красный" } else { "Red" };
                    let c_green = if is_ru { "Зеленый" } else { "Green" };
                    let c_yellow = if is_ru { "Желтый" } else { "Yellow" };

                    egui::ComboBox::from_label(color_label)
                        .selected_text(Self::node_color_text(self.arc_display_color, is_ru))
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Default,
                                c_default,
                            );
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Blue,
                                c_blue,
                            );
                            ui.selectable_value(&mut self.arc_display_color, NodeColor::Red, c_red);
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Green,
                                c_green,
                            );
                            ui.selectable_value(
                                &mut self.arc_display_color,
                                NodeColor::Yellow,
                                c_yellow,
                            );
                        });
                }

                let selected_arc_ids = self.collect_selected_arc_ids();
                if !selected_arc_ids.is_empty() {
                    ui.separator();
                    let color_label = self.tr("Цвет", "Color");

                    if selected_arc_ids.len() == 1 {
                        let arc_id = selected_arc_ids[0];
                        ui.label(self.tr("Выбранная связь", "Selected link"));

                        if let Some(arc) = self.net.arcs.iter_mut().find(|a| a.id == arc_id) {
                            egui::ComboBox::from_label(color_label)
                                .selected_text(Self::node_color_text(arc.color, is_ru))
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Default,
                                        Self::node_color_text(NodeColor::Default, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Blue,
                                        Self::node_color_text(NodeColor::Blue, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Red,
                                        Self::node_color_text(NodeColor::Red, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Green,
                                        Self::node_color_text(NodeColor::Green, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut arc.color,
                                        NodeColor::Yellow,
                                        Self::node_color_text(NodeColor::Yellow, is_ru),
                                    );
                                });
                        } else if let Some(inh) =
                            self.net.inhibitor_arcs.iter_mut().find(|a| a.id == arc_id)
                        {
                            egui::ComboBox::from_label(color_label)
                                .selected_text(Self::node_color_text(inh.color, is_ru))
                                .show_ui(ui, |ui: &mut egui::Ui| {
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Default,
                                        Self::node_color_text(NodeColor::Default, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Blue,
                                        Self::node_color_text(NodeColor::Blue, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Red,
                                        Self::node_color_text(NodeColor::Red, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Green,
                                        Self::node_color_text(NodeColor::Green, is_ru),
                                    );
                                    ui.selectable_value(
                                        &mut inh.color,
                                        NodeColor::Yellow,
                                        Self::node_color_text(NodeColor::Yellow, is_ru),
                                    );
                                });
                        }
                    } else {
                        let selected_label = if is_ru {
                            format!("Выбрано связей: {}", selected_arc_ids.len())
                        } else {
                            format!("Selected links: {}", selected_arc_ids.len())
                        };
                        ui.label(selected_label);

                        let mut bulk_color = selected_arc_ids
                            .iter()
                            .find_map(|id| {
                                self.net
                                    .arcs
                                    .iter()
                                    .find(|a| a.id == *id)
                                    .map(|a| a.color)
                                    .or_else(|| {
                                        self.net
                                            .inhibitor_arcs
                                            .iter()
                                            .find(|a| a.id == *id)
                                            .map(|a| a.color)
                                    })
                            })
                            .unwrap_or(NodeColor::Default);
                        let previous_color = bulk_color;

                        egui::ComboBox::from_label(color_label)
                            .selected_text(Self::node_color_text(bulk_color, is_ru))
                            .show_ui(ui, |ui: &mut egui::Ui| {
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Default,
                                    Self::node_color_text(NodeColor::Default, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Blue,
                                    Self::node_color_text(NodeColor::Blue, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Red,
                                    Self::node_color_text(NodeColor::Red, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Green,
                                    Self::node_color_text(NodeColor::Green, is_ru),
                                );
                                ui.selectable_value(
                                    &mut bulk_color,
                                    NodeColor::Yellow,
                                    Self::node_color_text(NodeColor::Yellow, is_ru),
                                );
                            });

                        if bulk_color != previous_color {
                            self.push_undo_snapshot();
                            let ids: HashSet<u64> = selected_arc_ids.iter().copied().collect();
                            for arc in &mut self.net.arcs {
                                if ids.contains(&arc.id) {
                                    arc.color = bulk_color;
                                }
                            }
                            for inh in &mut self.net.inhibitor_arcs {
                                if ids.contains(&inh.id) {
                                    inh.color = bulk_color;
                                }
                            }
                        }
                    }
                }
            });

        let open_props_by_rclick = ctx.input(|i| {
            if !i.pointer.button_clicked(egui::PointerButton::Secondary) {
                return false;
            }
            let Some(pos) = i.pointer.interact_pos() else {
                return false;
            };
            panel.response.rect.contains(pos)
        });
        if open_props_by_rclick {
            self.show_new_element_props = true;
        }
        if self.show_new_element_props {
            let is_ru = matches!(self.net.ui.language, Language::Ru);
            let t = |ru: &'static str, en: &'static str| if is_ru { ru } else { en };
            let mut open = self.show_new_element_props;
            let was_open = self.new_element_props_window_was_open;
            let apply_default_size = !was_open && open;
            let mut window = egui::Window::new(t(
                "Свойства создаваемых элементов",
                "New Element Properties",
            ))
            .open(&mut open)
            .resizable(true);
            if apply_default_size {
                window = window.default_size(self.new_element_props_window_size);
            }
            let response = window.show(ctx, |ui| {
                let mut corrected_inputs = false;
                let size_text = |size: VisualSize| -> &'static str {
                    if is_ru {
                        match size {
                            VisualSize::Small => "Малый",
                            VisualSize::Medium => "Средний",
                            VisualSize::Large => "Большой",
                        }
                    } else {
                        match size {
                            VisualSize::Small => "Small",
                            VisualSize::Medium => "Medium",
                            VisualSize::Large => "Large",
                        }
                    }
                };

                let color_combo = |ui: &mut egui::Ui, value: &mut NodeColor, is_ru: bool| {
                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(Self::node_color_text(*value, is_ru))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                value,
                                NodeColor::Default,
                                Self::node_color_text(NodeColor::Default, is_ru),
                            );
                            ui.selectable_value(
                                value,
                                NodeColor::Blue,
                                Self::node_color_text(NodeColor::Blue, is_ru),
                            );
                            ui.selectable_value(
                                value,
                                NodeColor::Red,
                                Self::node_color_text(NodeColor::Red, is_ru),
                            );
                            ui.selectable_value(
                                value,
                                NodeColor::Green,
                                Self::node_color_text(NodeColor::Green, is_ru),
                            );
                            ui.selectable_value(
                                value,
                                NodeColor::Yellow,
                                Self::node_color_text(NodeColor::Yellow, is_ru),
                            );
                        });
                };

                ui.group(|ui| {
                    ui.label(t("Новые позиции", "New positions"));
                    egui::ComboBox::from_label(t("Размер позиции", "Position size"))
                        .selected_text(size_text(self.new_place_size))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_place_size,
                                VisualSize::Small,
                                size_text(VisualSize::Small),
                            );
                            ui.selectable_value(
                                &mut self.new_place_size,
                                VisualSize::Medium,
                                size_text(VisualSize::Medium),
                            );
                            ui.selectable_value(
                                &mut self.new_place_size,
                                VisualSize::Large,
                                size_text(VisualSize::Large),
                            );
                        });
                    ui.horizontal(|ui| {
                        ui.label(t("Цвет", "Color"));
                        color_combo(ui, &mut self.new_place_color, is_ru);
                    });
                    let mut marking = self.new_place_marking;
                    corrected_inputs |= sanitize_u32(&mut marking, 0, u32::MAX);
                    ui.horizontal(|ui| {
                        ui.label(t("Маркеры", "Tokens"));
                        if ui
                            .add(egui::DragValue::new(&mut marking).range(0..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut marking, 0, u32::MAX);
                        }
                    });
                    self.new_place_marking = marking;
                    let mut cap = self.new_place_capacity.unwrap_or(0);
                    corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                    ui.horizontal(|ui| {
                        ui.label(t(
                            "Макс. емкость (0 = без ограничений)",
                            "Capacity (0 = unlimited)",
                        ));
                        if ui
                            .add(egui::DragValue::new(&mut cap).range(0..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut cap, 0, u32::MAX);
                        }
                    });
                    self.new_place_capacity = if cap == 0 { None } else { Some(cap) };
                });

                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.label(t("Новые переходы", "New transitions"));
                    egui::ComboBox::from_label(t("Размер перехода", "Transition size"))
                        .selected_text(size_text(self.new_transition_size))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_transition_size,
                                VisualSize::Small,
                                size_text(VisualSize::Small),
                            );
                            ui.selectable_value(
                                &mut self.new_transition_size,
                                VisualSize::Medium,
                                size_text(VisualSize::Medium),
                            );
                            ui.selectable_value(
                                &mut self.new_transition_size,
                                VisualSize::Large,
                                size_text(VisualSize::Large),
                            );
                        });
                    ui.horizontal(|ui| {
                        ui.label(t("Цвет", "Color"));
                        color_combo(ui, &mut self.new_transition_color, is_ru);
                    });
                    let mut transition_priority = self.new_transition_priority;
                    corrected_inputs |=
                        sanitize_i32(&mut transition_priority, -1_000_000, 1_000_000);
                    ui.horizontal(|ui| {
                        ui.label(t("Приоритет", "Priority"));
                        if ui
                            .add(
                                egui::DragValue::new(&mut transition_priority)
                                    .range(-1_000_000..=1_000_000),
                            )
                            .changed()
                        {
                            corrected_inputs |=
                                sanitize_i32(&mut transition_priority, -1_000_000, 1_000_000);
                        }
                    });
                    self.new_transition_priority = transition_priority;
                });

                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.label(t("Новые дуги", "New arcs"));
                    let mut arc_weight = self.new_arc_weight;
                    corrected_inputs |= sanitize_u32(&mut arc_weight, 1, u32::MAX);
                    ui.horizontal(|ui| {
                        ui.label(t("Кратность (вес)", "Weight"));
                        if ui
                            .add(egui::DragValue::new(&mut arc_weight).range(1..=u32::MAX))
                            .changed()
                        {
                            corrected_inputs |= sanitize_u32(&mut arc_weight, 1, u32::MAX);
                        }
                    });
                    self.new_arc_weight = arc_weight;
                    ui.horizontal(|ui| {
                        ui.label(t("Цвет", "Color"));
                        color_combo(ui, &mut self.new_arc_color, is_ru);
                    });
                    let inhibitor_label = t("Ингибиторная дуга", "Inhibitor arc");
                    ui.checkbox(&mut self.new_arc_inhibitor, inhibitor_label);
                    if self.new_arc_inhibitor {
                        ui.horizontal(|ui| {
                            ui.label(t("Порог", "Threshold"));
                            let mut threshold = self.new_arc_inhibitor_threshold;
                            corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                            if ui
                                .add(egui::DragValue::new(&mut threshold).range(1..=u32::MAX))
                                .changed()
                            {
                                corrected_inputs |= sanitize_u32(&mut threshold, 1, u32::MAX);
                            }
                            self.new_arc_inhibitor_threshold = threshold;
                        });
                    }
                });
                validation_hint(
                    ui,
                    corrected_inputs,
                    &self.tr(
                        "Некорректные значения были скорректированы",
                        "Invalid inputs were adjusted",
                    ),
                );
            });
            if open {
                if let Some(response) = response {
                    let size = response.response.rect.size();
                    if size.x > 0.0 && size.y > 0.0 {
                        self.new_element_props_window_size = size;
                    }
                }
            }
            self.show_new_element_props = open;
            self.new_element_props_window_was_open = open;
        }
    }
}


# src\ui\app.rs
use std::borrow::Cow;
use std::fs;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};

use crate::formats::atf::generate_atf;
use crate::io::{load_gpn, save_gpn_with_hints, LegacyExportHints};
use crate::markov::{build_markov_chain, MarkovChain};
use crate::model::{
    LabelPosition, Language, MarkovPlacement, NodeColor, NodeRef, PetriNet, Place,
    PlaceStatisticsSelection, StochasticDistribution, Tool, Transition, UiDecorativeFrame,
    UiTextBlock, VisualSize,
};
use crate::sim::engine::{run_simulation, SimulationParams, SimulationResult};

mod graph_view;
mod petri_app;
mod shortcuts;
mod table_view;
mod tool_palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Cascade,
    TileHorizontal,
    TileVertical,
    Minimized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArcDisplayMode {
    All,
    OnlyColor,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlaceStatsSeries {
    Total,
    Input,
    Output,
}

#[derive(Debug, Clone, Default)]
struct NetstarExportValidationReport {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl NetstarExportValidationReport {
    fn error_count(&self) -> usize {
        self.errors.len()
    }

    fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

#[derive(Debug, Clone)]
struct CanvasState {
    zoom: f32,
    pan: Vec2,
    selected_place: Option<u64>,
    selected_transition: Option<u64>,
    selected_places: Vec<u64>,
    selected_transitions: Vec<u64>,
    selected_arc: Option<u64>,
    selected_arcs: Vec<u64>,
    selected_text: Option<u64>,
    selected_texts: Vec<u64>,
    selected_frame: Option<u64>,
    selected_frames: Vec<u64>,
    arc_start: Option<NodeRef>,
    cursor_world: [f32; 2],
    selection_start: Option<Pos2>,
    selection_rect: Option<Rect>,
    selection_toggle_mode: bool,
    drag_prev_world: Option<[f32; 2]>,
    move_drag_active: bool,
    frame_draw_start_world: Option<[f32; 2]>,
    frame_draw_current_world: Option<[f32; 2]>,
    frame_resize_id: Option<u64>,
    cursor_valid: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::new(0.0, 0.0),
            selected_place: None,
            selected_transition: None,
            selected_places: Vec::new(),
            selected_transitions: Vec::new(),
            selected_arc: None,
            selected_arcs: Vec::new(),
            selected_text: None,
            selected_texts: Vec::new(),
            selected_frame: None,
            selected_frames: Vec::new(),
            arc_start: None,
            cursor_world: [0.0, 0.0],
            selection_start: None,
            selection_rect: None,
            selection_toggle_mode: false,
            drag_prev_world: None,
            move_drag_active: false,
            frame_draw_start_world: None,
            frame_draw_current_world: None,
            frame_resize_id: None,
            cursor_valid: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct MarkovPlaceArc {
    pub from_place_id: u64,
    pub to_place_id: Option<u64>,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct CanvasTextBlock {
    id: u64,
    pos: [f32; 2],
    text: String,
    font_name: String,
    font_size: f32,
    color: NodeColor,
}

impl Default for CanvasTextBlock {
    fn default() -> Self {
        Self {
            id: 0,
            pos: [0.0, 0.0],
            text: String::new(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanvasFrame {
    id: u64,
    pos: [f32; 2],
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyCanvasFrame {
    id: u64,
    pos: [f32; 2],
    side: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyUiSidecar {
    version: u32,
    #[serde(default)]
    text_blocks: Vec<CanvasTextBlock>,
    #[serde(default)]
    decorative_frames: Vec<LegacyCanvasFrame>,
    #[serde(default)]
    next_text_id: u64,
    #[serde(default)]
    next_frame_id: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedPlace {
    place: Place,
    m0: u32,
    mo: Option<u32>,
    mz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedTransition {
    transition: Transition,
    mpr: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedArc {
    from: NodeRef,
    to: NodeRef,
    weight: u32,
    color: NodeColor,
    visible: bool,
    show_weight: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopiedInhibitorArc {
    place_id: u64,
    transition_id: u64,
    threshold: u32,
    color: NodeColor,
    visible: bool,
    show_weight: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct CopiedTextBlock {
    pos: [f32; 2],
    text: String,
    font_name: String,
    font_size: f32,
    color: NodeColor,
}

impl Default for CopiedTextBlock {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0],
            text: String::new(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopyBuffer {
    origin: [f32; 2],
    places: Vec<CopiedPlace>,
    transitions: Vec<CopiedTransition>,
    arcs: Vec<CopiedArc>,
    inhibitors: Vec<CopiedInhibitorArc>,
    texts: Vec<CopiedTextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardPayload {
    version: u32,
    buffer: CopyBuffer,
}

#[derive(Debug, Clone)]
struct UndoSnapshot {
    net: PetriNet,
    text_blocks: Vec<CanvasTextBlock>,
    next_text_id: u64,
    decorative_frames: Vec<CanvasFrame>,
    next_frame_id: u64,
}

#[derive(Debug, Clone)]
struct DebugAnimationArc {
    arc_id: u64,
    weight: u32,
    place_idx: usize,
    token_colors: Vec<Color32>,
}

#[derive(Debug, Clone)]
struct DebugAnimationEvent {
    transition_idx: usize,
    step_idx: usize,
    duration: f64,
    entry_color: Color32,
    exit_color: Color32,
    pre_arcs: Vec<DebugAnimationArc>,
    post_arcs: Vec<DebugAnimationArc>,
}

impl DebugAnimationEvent {
    fn duration(&self) -> f64 {
        self.duration
    }
}

fn sanitize_f64(value: &mut f64, min: f64, max: f64) -> bool {
    if !value.is_finite() {
        *value = min;
        return true;
    }
    let clamped = value.clamp(min, max);
    let changed = (clamped - *value).abs() > f64::EPSILON;
    if changed {
        *value = clamped;
    }
    changed
}

fn sanitize_bounded<T: PartialOrd + Copy>(value: &mut T, min: T, max: T) -> bool {
    let mut changed = false;
    if *value < min {
        *value = min;
        changed = true;
    }
    if *value > max {
        *value = max;
        changed = true;
    }
    changed
}

fn sanitize_u64(value: &mut u64, min: u64, max: u64) -> bool {
    sanitize_bounded(value, min, max)
}

fn sanitize_usize(value: &mut usize, min: usize, max: usize) -> bool {
    sanitize_bounded(value, min, max)
}

fn sanitize_u32(value: &mut u32, min: u32, max: u32) -> bool {
    sanitize_bounded(value, min, max)
}

fn sanitize_i32(value: &mut i32, min: i32, max: i32) -> bool {
    sanitize_bounded(value, min, max)
}

fn validation_hint(ui: &mut egui::Ui, corrected: bool, msg: &str) {
    if corrected {
        ui.colored_label(Color32::from_rgb(190, 40, 40), msg);
    }
}

pub struct PetriApp {
    net: PetriNet,
    tool: Tool,
    canvas: CanvasState,
    sim_params: SimulationParams,
    sim_result: Option<Arc<SimulationResult>>,
    show_sim_params: bool,
    show_results: bool,
    show_atf: bool,
    atf_selected_place: usize,
    atf_text: String,
    file_path: Option<PathBuf>,
    last_error: Option<String>,
    layout_mode: LayoutMode,
    show_graph_view: bool,
    show_table_view: bool,
    table_fullscreen: bool,
    show_struct_vectors: bool,
    show_struct_pre: bool,
    show_struct_post: bool,
    show_struct_inhibitor: bool,
    place_props_id: Option<u64>,
    transition_props_id: Option<u64>,
    show_place_props: bool,
    show_transition_props: bool,
    text_props_id: Option<u64>,
    show_text_props: bool,
    arc_props_id: Option<u64>,
    show_arc_props: bool,
    show_debug: bool,
    debug_step: usize,
    debug_playing: bool,
    debug_interval_ms: u64,
    debug_arc_animation: bool,
    debug_animation_enabled: bool,
    debug_animation_local_clock: f64,
    debug_animation_current_duration: f64,
    debug_animation_last_update: Option<Instant>,
    debug_animation_events: Vec<DebugAnimationEvent>,
    debug_animation_active_event: Option<usize>,
    debug_animation_step_active: bool,
    debug_place_colors: Vec<Vec<Vec<Color32>>>,
    show_proof: bool,
    text_blocks: Vec<CanvasTextBlock>,
    next_text_id: u64,
    decorative_frames: Vec<CanvasFrame>,
    next_frame_id: u64,
    clipboard: Option<CopyBuffer>,
    paste_serial: u32,
    undo_stack: Vec<UndoSnapshot>,
    legacy_export_hints: Option<LegacyExportHints>,
    status_hint: Option<String>,
    show_help_development: bool,
    show_help_controls: bool,
    place_stats_dialog_place_id: Option<u64>,
    place_stats_dialog_backup: Option<(u64, PlaceStatisticsSelection)>,
    show_place_stats_window: bool,
    place_stats_view_place: usize,
    place_stats_series: PlaceStatsSeries,
    place_stats_zoom_x: f32,
    place_stats_pan_x: f32,
    place_stats_show_grid: bool,
    arc_display_mode: ArcDisplayMode,
    arc_display_color: NodeColor,
    show_netstar_export_validation: bool,
    pending_netstar_export_path: Option<PathBuf>,
    netstar_export_validation: Option<NetstarExportValidationReport>,
    show_new_element_props: bool,
    show_markov_window: bool,
    markov_model_enabled: bool,
    markov_model: Option<MarkovChain>,
    markov_limit_reached: bool,
    markov_annotations: HashMap<u64, String>,
    markov_place_arcs: Vec<MarkovPlaceArc>,
    new_place_size: VisualSize,
    new_place_color: NodeColor,
    new_place_marking: u32,
    new_place_capacity: Option<u32>,
    new_place_delay: f64,
    new_transition_size: VisualSize,
    new_transition_color: NodeColor,
    new_transition_priority: i32,
    new_arc_weight: u32,
    new_arc_color: NodeColor,
    new_arc_inhibitor: bool,
    new_arc_inhibitor_threshold: u32,
    new_element_props_window_size: Vec2,
    new_element_props_window_was_open: bool,
}

#[derive(Clone, Copy, Debug)]
enum MatrixCsvTarget {
    Pre,
    Post,
    Inhibitor,
}

impl PetriApp {
    const GRID_STEP_SNAP: f32 = 10.0;
    const GRID_STEP_FREE: f32 = 25.0;
    const CLIPBOARD_PREFIX: &'static str = "PETRINET_COPY_V1:";
    const FRAME_MIN_SIDE: f32 = 10.0;
    const FRAME_RESIZE_HANDLE_PX: f32 = 10.0;
    const MAX_PLOT_POINTS: usize = 2_000;
    const DEBUG_ANIMATION_MIN_DURATION: f64 = 0.1;
    const DEBUG_ANIMATION_MAX_DURATION: f64 = 1.5;

    pub(in crate::ui::app) fn refresh_debug_animation_state(&mut self) {
        if let Some(result) = self.sim_result.as_ref() {
            let (events, place_colors) = Self::build_debug_animation_events(&self.net, result);
            self.debug_animation_events = events;
            self.debug_place_colors = place_colors;
        } else {
            self.debug_animation_events.clear();
            self.debug_place_colors.clear();
        }
        self.sync_debug_animation_for_step();
    }

    fn sync_debug_animation_for_step(&mut self) {
        self.debug_animation_last_update = None;
        if !self.debug_animation_enabled || self.debug_animation_events.is_empty() {
            self.clear_debug_animation_state();
            return;
        }
        let Some(result) = self.sim_result.as_ref() else {
            self.clear_debug_animation_state();
            return;
        };
        let visible_steps = Self::debug_visible_log_indices(result);
        if visible_steps.is_empty() {
            self.clear_debug_animation_state();
            return;
        }
        if self.debug_step >= visible_steps.len() {
            self.debug_step = visible_steps.len() - 1;
        }
        let event_idx = self
            .debug_animation_events
            .iter()
            .position(|event| event.step_idx == self.debug_step);
        self.set_active_debug_animation_event(event_idx, visible_steps.len());
    }
    fn set_active_debug_animation_event(&mut self, event_idx: Option<usize>, visible_len: usize) {
        self.debug_animation_active_event = event_idx;
        if let Some(idx) = event_idx {
            if visible_len > 0 && self.debug_step >= visible_len {
                self.debug_step = visible_len - 1;
            }
            let duration = self.debug_animation_events[idx]
                .duration()
                .max(Self::DEBUG_ANIMATION_MIN_DURATION);
            self.debug_animation_current_duration = duration;
            self.debug_animation_local_clock = 0.0;
            self.debug_animation_step_active = self.debug_playing && duration > 0.0;
            self.debug_animation_last_update = None;
        } else {
            self.debug_animation_local_clock = 0.0;
            self.debug_animation_current_duration = 0.0;
            self.debug_animation_step_active = false;
        }
    }

    fn clear_debug_animation_state(&mut self) {
        self.debug_animation_active_event = None;
        self.debug_animation_local_clock = 0.0;
        self.debug_animation_last_update = None;
        self.debug_playing = false;
        self.debug_animation_current_duration = 0.0;
        self.debug_animation_step_active = false;
        self.debug_place_colors.clear();
    }

    fn debug_animation_playback_speed(&self) -> f64 {
        let interval = self.debug_interval_ms.max(1);
        1000.0 / interval as f64
    }

    fn build_debug_animation_events(
        net: &PetriNet,
        result: &SimulationResult,
    ) -> (Vec<DebugAnimationEvent>, Vec<Vec<Vec<Color32>>>) {
        let mut events = Vec::new();
        let visible_steps = Self::debug_visible_log_indices(result);
        if visible_steps.is_empty() {
            return (events, Vec::new());
        }
        let default_marker_color = Color32::from_rgb(200, 0, 0);
        let initial_marking = result
            .logs
            .get(*visible_steps.first().unwrap_or(&0))
            .map(|entry| entry.marking.clone())
            .unwrap_or_else(|| net.tables.m0.clone());
        let mut place_token_colors: Vec<Vec<Color32>> = net
            .places
            .iter()
            .enumerate()
            .map(|(place_idx, place)| {
                let count = initial_marking.get(place_idx).copied().unwrap_or(0);
                let token_color = if place.marker_color_on_pass {
                    Self::color_to_egui(place.color, default_marker_color)
                } else {
                    default_marker_color
                };
                vec![token_color; count as usize]
            })
            .collect();
        let mut place_color_timeline = vec![Vec::new(); visible_steps.len()];
        place_color_timeline[0] = place_token_colors.iter().cloned().collect::<Vec<_>>();

        for step_idx in 0..visible_steps.len().saturating_sub(1) {
            let next_log_idx = visible_steps[step_idx + 1];
            let entry = match result.logs.get(next_log_idx) {
                Some(entry) => entry,
                None => {
                    place_color_timeline[step_idx + 1] =
                        place_token_colors.iter().cloned().collect::<Vec<_>>();
                    continue;
                }
            };
            let transition_idx = match entry.fired_transition {
                Some(idx) => idx,
                None => {
                    place_color_timeline[step_idx + 1] =
                        place_token_colors.iter().cloned().collect::<Vec<_>>();
                    continue;
                }
            };
            let mut next_time = entry.time;
            for subsequent in result.logs.iter().skip(next_log_idx + 1) {
                if subsequent.time > entry.time {
                    next_time = subsequent.time;
                    break;
                }
            }
            if next_time <= entry.time {
                next_time = entry.time + Self::DEBUG_ANIMATION_MIN_DURATION;
            }
            let mut duration = next_time - entry.time;
            duration = duration
                .max(Self::DEBUG_ANIMATION_MIN_DURATION)
                .min(Self::DEBUG_ANIMATION_MAX_DURATION);
            let mut pre_arcs = Self::transition_arcs(net, transition_idx, true);
            let mut post_arcs = Self::transition_arcs(net, transition_idx, false);
            let mut moving_colors = VecDeque::new();
            let mut entry_color = default_marker_color;
            for arc in pre_arcs.iter_mut() {
                for _ in 0..arc.weight {
                    let token_color = place_token_colors[arc.place_idx]
                        .pop()
                        .unwrap_or(default_marker_color);
                    arc.token_colors.push(token_color);
                    moving_colors.push_back(token_color);
                }
            }
            if let Some(color) = moving_colors.front().copied() {
                entry_color = color;
            } else if let Some((color, _)) = Self::marker_color_from_places(
                net,
                entry.touched_places.as_slice(),
                default_marker_color,
            ) {
                entry_color = color;
            }
            for arc in post_arcs.iter_mut() {
                let mut assigned = Vec::new();
                for _ in 0..arc.weight {
                    let token_color = moving_colors.pop_front().unwrap_or(entry_color);
                    assigned.push(token_color);
                    if let Some(slot) = place_token_colors.get_mut(arc.place_idx) {
                        let placed_color = if let Some(place) = net.places.get(arc.place_idx) {
                            if place.marker_color_on_pass {
                                Self::color_to_egui(place.color, token_color)
                            } else {
                                token_color
                            }
                        } else {
                            token_color
                        };
                        slot.push(placed_color);
                    }
                }
                arc.token_colors = assigned;
            }
            let exit_color = post_arcs
                .iter()
                .flat_map(|arc| arc.token_colors.iter())
                .copied()
                .next()
                .unwrap_or(entry_color);
            place_color_timeline[step_idx + 1] =
                place_token_colors.iter().cloned().collect::<Vec<_>>();
            events.push(DebugAnimationEvent {
                transition_idx,
                step_idx,
                duration,
                entry_color,
                exit_color,
                pre_arcs,
                post_arcs,
            });
        }

        (events, place_color_timeline)
    }

    fn transition_arcs(
        net: &PetriNet,
        transition_idx: usize,
        incoming: bool,
    ) -> Vec<DebugAnimationArc> {
        let Some(transition) = net.transitions.get(transition_idx) else {
            return Vec::new();
        };
        let transition_id = transition.id;
        let mut arcs: Vec<(u64, u32, u64)> = net
            .arcs
            .iter()
            .filter(|arc| arc.weight > 0)
            .filter_map(|arc| {
                if incoming {
                    match (&arc.from, &arc.to) {
                        (NodeRef::Place(place_id), NodeRef::Transition(id))
                            if *id == transition_id =>
                        {
                            Some((arc.id, arc.weight, *place_id))
                        }
                        _ => None,
                    }
                } else {
                    match (&arc.from, &arc.to) {
                        (NodeRef::Transition(id), NodeRef::Place(place_id))
                            if *id == transition_id =>
                        {
                            Some((arc.id, arc.weight, *place_id))
                        }
                        _ => None,
                    }
                }
            })
            .collect();
        arcs.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        arcs.into_iter()
            .filter_map(|(arc_id, weight, place_id)| {
                Self::place_index_by_id(net, place_id).map(|place_idx| DebugAnimationArc {
                    arc_id,
                    weight,
                    place_idx,
                    token_colors: Vec::new(),
                })
            })
            .collect()
    }

    fn marker_color_from_places(
        net: &PetriNet,
        touched_places: &[usize],
        fallback: Color32,
    ) -> Option<(Color32, usize)> {
        for &place_idx in touched_places.iter().rev() {
            if let Some(place) = net.places.get(place_idx) {
                if place.marker_color_on_pass {
                    return Some((Self::color_to_egui(place.color, fallback), place_idx));
                }
            }
        }
        None
    }

    fn aggregate_token_counts(colors: &[Color32]) -> Vec<(Color32, u32)> {
        let mut map = HashMap::new();
        for &color in colors {
            *map.entry(color).or_insert(0) += 1;
        }
        let mut counts = map.into_iter().collect::<Vec<_>>();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        counts
    }

    fn place_index_by_id(net: &PetriNet, place_id: u64) -> Option<usize> {
        net.places.iter().position(|place| place.id == place_id)
    }
}

impl eframe::App for PetriApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        self.draw_menu(ctx);
        self.draw_tool_palette(ctx);
        self.draw_layout(ctx);
        self.draw_status(ctx);

        if self.show_sim_params {
            self.draw_sim_dialog(ctx);
        }
        if self.show_results {
            self.draw_results(ctx);
        }
        self.draw_place_statistics_window(ctx);
        if self.show_place_props {
            self.draw_place_properties(ctx);
        }
        self.draw_place_stats_dialog(ctx);
        if self.show_transition_props {
            self.draw_transition_properties(ctx);
        }
        if self.show_arc_props {
            self.draw_arc_properties(ctx);
        }
        if self.show_text_props {
            self.draw_text_properties(ctx);
        }
        if self.show_debug {
            self.draw_debug_window(ctx);
        }
        if self.show_proof {
            self.draw_proof_window(ctx);
        }
        if self.show_atf {
            self.draw_atf_window(ctx);
        }
        if self.show_help_development {
            self.draw_help_development(ctx);
        }
        if self.show_help_controls {
            self.draw_help_controls(ctx);
        }
        if self.show_markov_window {
            self.draw_markov_window(ctx);
        }
        self.draw_netstar_export_validation(ctx);
        self.handle_shortcuts(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_shortcut_copies_selected_place() {
        let mut app = PetriApp::new_for_tests();
        let selected = app.net.places[0].id;
        app.canvas.selected_place = Some(selected);

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: egui::Key::C,
            physical_key: Some(egui::Key::C),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        });

        ctx.begin_frame(raw);
        app.handle_shortcuts(&ctx);
        let _ = ctx.end_frame();

        assert!(
            app.clipboard.is_some(),
            "clipboard should be populated by Ctrl+C"
        );
        let copied = app.clipboard.as_ref().unwrap();
        assert_eq!(copied.places.len(), 1);
    }

    #[test]
    fn ctrl_c_ru_layout_text_event_copies_selected_place() {
        let mut app = PetriApp::new_for_tests();
        let selected = app.net.places[0].id;
        app.canvas.selected_place = Some(selected);

        let ctx = egui::Context::default();
        let mut raw = egui::RawInput {
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
            ..Default::default()
        };
        raw.events.push(egui::Event::Text("с".to_string()));

        ctx.begin_frame(raw);
        app.handle_shortcuts(&ctx);
        let _ = ctx.end_frame();

        assert!(
            app.clipboard.is_some(),
            "clipboard should be populated by Ctrl+С (RU layout fallback)"
        );
        let copied = app.clipboard.as_ref().unwrap();
        assert_eq!(copied.places.len(), 1);
    }

    #[test]
    fn netstar_export_validation_has_error_for_broken_arc_link() {
        let mut app = PetriApp::new_for_tests();
        app.net.arcs.push(crate::model::Arc {
            id: 999,
            from: NodeRef::Place(999_999),
            to: NodeRef::Transition(app.net.transitions[0].id),
            weight: 1,
            color: NodeColor::Default,
            visible: true,
            show_weight: false,
        });

        let report = app.validate_netstar_export();
        assert!(
            report.error_count() > 0,
            "broken arc link must produce a blocking export error"
        );
    }

    #[test]
    fn netstar_export_validation_warns_for_non_exportable_ui_elements() {
        let mut app = PetriApp::new_for_tests();
        app.text_blocks.push(CanvasTextBlock {
            id: 1,
            pos: [10.0, 10.0],
            text: "x".to_string(),
            font_name: "MS Sans Serif".to_string(),
            font_size: 10.0,
            color: NodeColor::Default,
        });
        app.decorative_frames.push(CanvasFrame {
            id: 1,
            pos: [20.0, 20.0],
            width: 120.0,
            height: 80.0,
        });

        let report = app.validate_netstar_export();
        assert!(
            report.warning_count() >= 2,
            "text blocks and frames should be reported as export warnings"
        );
    }
}


# src\ui\mod.rs
pub mod app;


# target\release\.fingerprint\ab_glyph-d84d047e2bcfce2c\lib-ab_glyph.json
{"rustc":8323788817864214825,"features":"[\"default\", \"gvar-alloc\", \"std\", \"variable-fonts\"]","declared_features":"[\"default\", \"gvar-alloc\", \"libm\", \"std\", \"variable-fonts\"]","target":11794240345726188307,"profile":16864349624179186615,"path":7179593176715281944,"deps":[[4945662571602681759,"ab_glyph_rasterizer",false,15825971059094471955],[5327495677235252177,"owned_ttf_parser",false,4116700028386188858]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ab_glyph-d84d047e2bcfce2c\\dep-lib-ab_glyph","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ab_glyph_rasterizer-43bd997728ea9081\lib-ab_glyph_rasterizer.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"default\", \"libm\", \"std\"]","target":4335109392423587462,"profile":16864349624179186615,"path":800481252786037124,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ab_glyph_rasterizer-43bd997728ea9081\\dep-lib-ab_glyph_rasterizer","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\adler2-1c0b0db37f26ca08\lib-adler2.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"core\", \"default\", \"rustc-dep-of-std\", \"std\"]","target":6569825234462323107,"profile":16864349624179186615,"path":986793354157808043,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\adler2-1c0b0db37f26ca08\\dep-lib-adler2","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ahash-28edca4096eb6f6c\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[966925859616469517,"build_script_build",false,685498266905956622]],"local":[{"RerunIfChanged":{"output":"release\\build\\ahash-28edca4096eb6f6c\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\ahash-6e8e4a8b6d023a04\lib-ahash.json
{"rustc":8323788817864214825,"features":"[\"no-rng\", \"serde\", \"std\"]","declared_features":"[\"atomic-polyfill\", \"compile-time-rng\", \"const-random\", \"default\", \"getrandom\", \"nightly-arm-aes\", \"no-rng\", \"runtime-rng\", \"serde\", \"std\"]","target":8470944000320059508,"profile":16864349624179186615,"path":983777462922855635,"deps":[[966925859616469517,"build_script_build",false,17494839546448250307],[3722963349756955755,"once_cell",false,7236280117719017647],[7667230146095136825,"cfg_if",false,4717990148927456231],[13548984313718623784,"serde",false,11479283716925170977],[17375358419629610217,"zerocopy",false,16411237750151551133]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ahash-6e8e4a8b6d023a04\\dep-lib-ahash","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ahash-75eff3481747d409\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"no-rng\", \"serde\", \"std\"]","declared_features":"[\"atomic-polyfill\", \"compile-time-rng\", \"const-random\", \"default\", \"getrandom\", \"nightly-arm-aes\", \"no-rng\", \"runtime-rng\", \"serde\", \"std\"]","target":17883862002600103897,"profile":9773466895796779991,"path":15934660706167982550,"deps":[[5398981501050481332,"version_check",false,7050309554974208742]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ahash-75eff3481747d409\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\anyhow-abc53d0aa36c20cc\lib-anyhow.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"backtrace\", \"default\", \"std\"]","target":1563897884725121975,"profile":16864349624179186615,"path":3237890435411738583,"deps":[[12478428894219133322,"build_script_build",false,15463427574697173435]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\anyhow-abc53d0aa36c20cc\\dep-lib-anyhow","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\anyhow-cd9c1cee6e24f436\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"backtrace\", \"default\", \"std\"]","target":5408242616063297496,"profile":9773466895796779991,"path":2746074406294380484,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\anyhow-cd9c1cee6e24f436\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\anyhow-de7d0baf8e8c4ee0\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[12478428894219133322,"build_script_build",false,2139702613510946215]],"local":[{"RerunIfChanged":{"output":"release\\build\\anyhow-de7d0baf8e8c4ee0\\output","paths":["src/nightly.rs"]}},{"RerunIfEnvChanged":{"var":"RUSTC_BOOTSTRAP","val":null}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\arboard-2ecef1daa289e0e9\lib-arboard.json
{"rustc":8323788817864214825,"features":"[\"core-graphics\", \"default\", \"image\", \"image-data\", \"windows-sys\"]","declared_features":"[\"core-graphics\", \"default\", \"image\", \"image-data\", \"wayland-data-control\", \"windows-sys\", \"wl-clipboard-rs\"]","target":1337616771932055151,"profile":16864349624179186615,"path":7813826341190821568,"deps":[[6536293665624942953,"clipboard_win",false,578532857206431034],[7263319592666514104,"windows_sys",false,13511790242110056361],[10630857666389190470,"log",false,7448553794738313875],[10681258086952200236,"image",false,7482427709356447215]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\arboard-2ecef1daa289e0e9\\dep-lib-arboard","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\autocfg-422be718d8cf2eff\lib-autocfg.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":6962977057026645649,"profile":9773466895796779991,"path":9943415613804787058,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\autocfg-422be718d8cf2eff\\dep-lib-autocfg","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\base64-2204999a8ff51f91\lib-base64.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"default\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"std\"]","target":13060062996227388079,"profile":16864349624179186615,"path":3400280385532412797,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\base64-2204999a8ff51f91\\dep-lib-base64","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\bitflags-2d049527b5e3760e\lib-bitflags.json
{"rustc":8323788817864214825,"features":"[\"serde\", \"serde_core\"]","declared_features":"[\"arbitrary\", \"bytemuck\", \"example_generated\", \"serde\", \"serde_core\", \"std\"]","target":7691312148208718491,"profile":16864349624179186615,"path":9729112754999151703,"deps":[[11899261697793765154,"serde_core",false,10322155688346445210]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\bitflags-2d049527b5e3760e\\dep-lib-bitflags","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\bytemuck-615c5663e28e66af\lib-bytemuck.json
{"rustc":8323788817864214825,"features":"[\"bytemuck_derive\", \"derive\", \"extern_crate_alloc\"]","declared_features":"[\"aarch64_simd\", \"align_offset\", \"alloc_uninit\", \"avx512_simd\", \"bytemuck_derive\", \"const_zeroed\", \"derive\", \"extern_crate_alloc\", \"extern_crate_std\", \"impl_core_error\", \"latest_stable_rust\", \"min_const_generics\", \"must_cast\", \"must_cast_extra\", \"nightly_docs\", \"nightly_float\", \"nightly_portable_simd\", \"nightly_stdsimd\", \"pod_saturating\", \"rustversion\", \"track_caller\", \"transparentwrapper_extra\", \"unsound_ptr_pod_impl\", \"wasm_simd\", \"zeroable_atomics\", \"zeroable_maybe_uninit\", \"zeroable_unwind_fn\"]","target":5195934831136530909,"profile":4001805701485671226,"path":13620760797920373188,"deps":[[15783091771682552589,"bytemuck_derive",false,1641220750578387397]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\bytemuck-615c5663e28e66af\\dep-lib-bytemuck","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\bytemuck_derive-f036f3240d51736f\lib-bytemuck_derive.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":11496395835559002815,"profile":9773466895796779991,"path":15190940017536711197,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\bytemuck_derive-f036f3240d51736f\\dep-lib-bytemuck_derive","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\byteorder-lite-facc52a555781917\lib-byteorder_lite.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"default\", \"std\"]","target":13691508551864173732,"profile":16864349624179186615,"path":1293681626576330363,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\byteorder-lite-facc52a555781917\\dep-lib-byteorder_lite","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\cfg-if-a8a24894282a3751\lib-cfg_if.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"core\", \"rustc-dep-of-std\"]","target":13840298032947503755,"profile":16864349624179186615,"path":12221618967600105978,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\cfg-if-a8a24894282a3751\\dep-lib-cfg_if","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\cfg_aliases-77ee418f8072dd7a\lib-cfg_aliases.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":14022534369768855544,"profile":9773466895796779991,"path":4953113534110305164,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\cfg_aliases-77ee418f8072dd7a\\dep-lib-cfg_aliases","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\clipboard-win-b3ea1e698eeffc15\lib-clipboard_win.json
{"rustc":8323788817864214825,"features":"[\"std\"]","declared_features":"[\"monitor\", \"std\", \"windows-win\"]","target":1945234718698444063,"profile":16864349624179186615,"path":17822084458939970624,"deps":[[8705426877712808690,"error_code",false,17144067004960470354]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\clipboard-win-b3ea1e698eeffc15\\dep-lib-clipboard_win","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\crc32fast-b7711e917bd75d35\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[7312356825837975969,"build_script_build",false,9466697773252183616]],"local":[{"Precalculated":"1.5.0"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\crc32fast-bc82a9b2dc3543bb\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"default\", \"nightly\", \"std\"]","target":5408242616063297496,"profile":9773466895796779991,"path":5093667990044438104,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\crc32fast-bc82a9b2dc3543bb\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\crc32fast-f3d76b99c3985de9\lib-crc32fast.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"default\", \"nightly\", \"std\"]","target":10823605331999153028,"profile":16864349624179186615,"path":5315010754507471253,"deps":[[7312356825837975969,"build_script_build",false,17525141040561718295],[7667230146095136825,"cfg_if",false,4717990148927456231]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\crc32fast-f3d76b99c3985de9\\dep-lib-crc32fast","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\cursor-icon-143f610c54c2403c\lib-cursor_icon.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"default\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"serde\", \"std\"]","target":2922482735460660294,"profile":16864349624179186615,"path":5530085215595226403,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\cursor-icon-143f610c54c2403c\\dep-lib-cursor_icon","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\directories-efab46220a5d20a0\lib-directories.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":17249629911599636029,"profile":16864349624179186615,"path":4254631972582960262,"deps":[[11795441179928084356,"dirs_sys",false,1050032006665209942]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\directories-efab46220a5d20a0\\dep-lib-directories","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\dirs-sys-7c8e945ba0210ff1\lib-dirs_sys.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":1716570026465204918,"profile":16864349624179186615,"path":10375191434477556258,"deps":[[1999565553139417705,"windows_sys",false,9819654464114153760],[9760035060063614848,"option_ext",false,17884234329363708995]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\dirs-sys-7c8e945ba0210ff1\\dep-lib-dirs_sys","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\displaydoc-b9edcf6334f40501\lib-displaydoc.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"default\", \"std\"]","target":9331843185013996172,"profile":9773466895796779991,"path":7525076079844704157,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\displaydoc-b9edcf6334f40501\\dep-lib-displaydoc","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\document-features-c0a5325f8f7a6ebe\lib-document_features.json
{"rustc":8323788817864214825,"features":"[\"default\"]","declared_features":"[\"default\", \"self-test\"]","target":4282619336790389174,"profile":9773466895796779991,"path":4781972703408459568,"deps":[[12609936415420532601,"litrs",false,7821704977415686949]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\document-features-c0a5325f8f7a6ebe\\dep-lib-document_features","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ecolor-b79105eb7d98c142\lib-ecolor.json
{"rustc":8323788817864214825,"features":"[\"bytemuck\", \"serde\"]","declared_features":"[\"bytemuck\", \"cint\", \"color-hex\", \"default\", \"document-features\", \"serde\"]","target":5564790870329063819,"profile":11841608122064889856,"path":14593973795475579638,"deps":[[5334405045287021829,"emath",false,5716571162882726735],[13548984313718623784,"serde",false,11479283716925170977],[14589292995769234176,"bytemuck",false,8681264735004750289]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ecolor-b79105eb7d98c142\\dep-lib-ecolor","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\egui-winit-412980226c2aa773\lib-egui_winit.json
{"rustc":8323788817864214825,"features":"[\"arboard\", \"clipboard\", \"links\", \"serde\", \"smithay-clipboard\", \"webbrowser\"]","declared_features":"[\"accesskit\", \"accesskit_winit\", \"android-game-activity\", \"android-native-activity\", \"arboard\", \"bytemuck\", \"clipboard\", \"default\", \"document-features\", \"links\", \"puffin\", \"serde\", \"smithay-clipboard\", \"wayland\", \"webbrowser\", \"x11\"]","target":15155777706629005642,"profile":11841608122064889856,"path":13662130705408683886,"deps":[[86246135597337767,"arboard",false,11318292946226331850],[966925859616469517,"ahash",false,7396982772510991919],[1202109798122345414,"webbrowser",false,15164366624576380537],[2901339412823178527,"winit",false,10908655721099948944],[4143744114649553716,"raw_window_handle",false,16672366439269497394],[4310028563857582016,"web_time",false,16491806654593307286],[9821838137176528293,"egui",false,10682107544508587686],[10630857666389190470,"log",false,7448553794738313875],[13548984313718623784,"serde",false,11479283716925170977]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\egui-winit-412980226c2aa773\\dep-lib-egui_winit","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\egui_glow-bc175781a7d3350a\lib-egui_glow.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"clipboard\", \"default\", \"document-features\", \"egui-winit\", \"links\", \"puffin\", \"wayland\", \"winit\", \"x11\"]","target":15671185835846101178,"profile":11841608122064889856,"path":4633515310962507577,"deps":[[966925859616469517,"ahash",false,7396982772510991919],[2579673976484116293,"glow",false,1109909655750287572],[9821838137176528293,"egui",false,10682107544508587686],[10630857666389190470,"log",false,7448553794738313875],[14589292995769234176,"bytemuck",false,8681264735004750289],[14643204177830147187,"memoffset",false,15870767511149099667]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\egui_glow-bc175781a7d3350a\\dep-lib-egui_glow","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\emath-8ee69df58b10abd0\lib-emath.json
{"rustc":8323788817864214825,"features":"[\"bytemuck\", \"serde\"]","declared_features":"[\"bytemuck\", \"default\", \"document-features\", \"mint\", \"serde\"]","target":14620128083324269871,"profile":11841608122064889856,"path":3735211761848449054,"deps":[[13548984313718623784,"serde",false,11479283716925170977],[14589292995769234176,"bytemuck",false,8681264735004750289]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\emath-8ee69df58b10abd0\\dep-lib-emath","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\encoding_rs-0457adf4f584933f\lib-encoding_rs.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"default\"]","declared_features":"[\"alloc\", \"any_all_workaround\", \"default\", \"fast-big5-hanzi-encode\", \"fast-gb-hanzi-encode\", \"fast-hangul-encode\", \"fast-hanja-encode\", \"fast-kanji-encode\", \"fast-legacy-encode\", \"less-slow-big5-hanzi-encode\", \"less-slow-gb-hanzi-encode\", \"less-slow-kanji-encode\", \"serde\", \"simd-accel\"]","target":17616512236202378241,"profile":16864349624179186615,"path":12359341941117679782,"deps":[[7667230146095136825,"cfg_if",false,4717990148927456231]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\encoding_rs-0457adf4f584933f\\dep-lib-encoding_rs","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\epaint-73d58611f912fc42\lib-epaint.json
{"rustc":8323788817864214825,"features":"[\"bytemuck\", \"default_fonts\", \"log\", \"serde\"]","declared_features":"[\"bytemuck\", \"cint\", \"color-hex\", \"deadlock_detection\", \"default\", \"default_fonts\", \"document-features\", \"log\", \"mint\", \"puffin\", \"rayon\", \"serde\", \"unity\"]","target":10495837225410426609,"profile":11841608122064889856,"path":17858884944574382735,"deps":[[966925859616469517,"ahash",false,7396982772510991919],[5334405045287021829,"emath",false,5716571162882726735],[5931649091606299019,"nohash_hasher",false,11022125352988666442],[7459327328022629880,"ecolor",false,11641954395905521181],[10630857666389190470,"log",false,7448553794738313875],[12459942763388630573,"parking_lot",false,16763488655450507642],[13548984313718623784,"serde",false,11479283716925170977],[13755666026417058023,"ab_glyph",false,14157702803808807364],[14589292995769234176,"bytemuck",false,8681264735004750289]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\epaint-73d58611f912fc42\\dep-lib-epaint","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\error-code-e3c8adae6a482a81\lib-error_code.json
{"rustc":8323788817864214825,"features":"[\"std\"]","declared_features":"[\"std\"]","target":13660428293521089546,"profile":16864349624179186615,"path":8359619705654265594,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\error-code-e3c8adae6a482a81\\dep-lib-error_code","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\fdeflate-5a7ed540bccd8b07\lib-fdeflate.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":4671662198888697476,"profile":18347699107735970791,"path":7206393585965917762,"deps":[[5982862185909702272,"simd_adler32",false,788125168118963293]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\fdeflate-5a7ed540bccd8b07\\dep-lib-fdeflate","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\flate2-bcf85ad8580b55e8\lib-flate2.json
{"rustc":8323788817864214825,"features":"[\"any_impl\", \"default\", \"miniz_oxide\", \"rust_backend\"]","declared_features":"[\"any_c_zlib\", \"any_impl\", \"any_zlib\", \"cloudflare-zlib-sys\", \"cloudflare_zlib\", \"default\", \"document-features\", \"libz-ng-sys\", \"libz-sys\", \"miniz-sys\", \"miniz_oxide\", \"rust_backend\", \"zlib\", \"zlib-default\", \"zlib-ng\", \"zlib-ng-compat\", \"zlib-rs\"]","target":6173716359330453699,"profile":16864349624179186615,"path":16440822950421130340,"deps":[[7312356825837975969,"crc32fast",false,13648362039306283598],[7636735136738807108,"miniz_oxide",false,7159646662007226201]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\flate2-bcf85ad8580b55e8\\dep-lib-flate2","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\form_urlencoded-465e7f25b62442c9\lib-form_urlencoded.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"std\"]","target":6496257856677244489,"profile":16864349624179186615,"path":2583407701755440059,"deps":[[6803352382179706244,"percent_encoding",false,15708728636639529212]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\form_urlencoded-465e7f25b62442c9\\dep-lib-form_urlencoded","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\getrandom-a046ad1f5209699b\lib-getrandom.json
{"rustc":8323788817864214825,"features":"[\"std\"]","declared_features":"[\"compiler_builtins\", \"core\", \"custom\", \"js\", \"js-sys\", \"linux_disable_fallback\", \"rdrand\", \"rustc-dep-of-std\", \"std\", \"test-in-browser\", \"wasm-bindgen\"]","target":16244099637825074703,"profile":16864349624179186615,"path":9800400122011586126,"deps":[[7667230146095136825,"cfg_if",false,4717990148927456231]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\getrandom-a046ad1f5209699b\\dep-lib-getrandom","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\gl_generator-f8ae1d92b4ba5d34\lib-gl_generator.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"unstable_generator_utils\"]","target":15798113755487949458,"profile":9773466895796779991,"path":7320720918507590433,"deps":[[4891955779658748086,"khronos_api",false,7141171948998162409],[10630857666389190470,"log",false,244567944830839462],[13254818194777074109,"xml",false,11374346422283819099]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\gl_generator-f8ae1d92b4ba5d34\\dep-lib-gl_generator","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glow-82e1b6cd21c1a967\lib-glow.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"debug_automatic_glGetError\", \"debug_trace_calls\", \"log\"]","target":17705349501093277854,"profile":16864349624179186615,"path":12497291677853735864,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glow-82e1b6cd21c1a967\\dep-lib-glow","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin-afc6b752f2709735\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"egl\", \"glutin_egl_sys\", \"glutin_glx_sys\", \"glutin_wgl_sys\", \"glx\", \"libloading\", \"wayland\", \"wayland-sys\", \"wgl\", \"windows-sys\", \"x11\", \"x11-dl\"]","declared_features":"[\"default\", \"egl\", \"glutin_egl_sys\", \"glutin_glx_sys\", \"glutin_wgl_sys\", \"glx\", \"libloading\", \"wayland\", \"wayland-sys\", \"wgl\", \"windows-sys\", \"x11\", \"x11-dl\"]","target":5408242616063297496,"profile":9773466895796779991,"path":11883478890684295379,"deps":[[13650835054453599687,"cfg_aliases",false,17995880469106153942]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin-afc6b752f2709735\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin-e96c5deb0cc87d6b\lib-glutin.json
{"rustc":8323788817864214825,"features":"[\"default\", \"egl\", \"glutin_egl_sys\", \"glutin_glx_sys\", \"glutin_wgl_sys\", \"glx\", \"libloading\", \"wayland\", \"wayland-sys\", \"wgl\", \"windows-sys\", \"x11\", \"x11-dl\"]","declared_features":"[\"default\", \"egl\", \"glutin_egl_sys\", \"glutin_glx_sys\", \"glutin_wgl_sys\", \"glx\", \"libloading\", \"wayland\", \"wayland-sys\", \"wgl\", \"windows-sys\", \"x11\", \"x11-dl\"]","target":1390373939866593923,"profile":16864349624179186615,"path":17483125954578757850,"deps":[[1999565553139417705,"windows_sys",false,9819654464114153760],[3309154526855700477,"glutin_egl_sys",false,13102679079619902794],[3722963349756955755,"once_cell",false,7236280117719017647],[7628053700111581507,"build_script_build",false,6166866377214704331],[7883780462905440460,"libloading",false,6576397874779550291],[11693073011723388840,"raw_window_handle",false,10692714251261719210],[16909888598953886583,"bitflags",false,3782616987721616857],[17088206048040325894,"glutin_wgl_sys",false,2056604618129243530]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin-e96c5deb0cc87d6b\\dep-lib-glutin","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin-f19f65c71286f5f5\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[7628053700111581507,"build_script_build",false,921656954206217321]],"local":[{"Precalculated":"0.31.3"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\glutin-winit-45a8b699f6a70125\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"egl\", \"glx\", \"wayland\", \"wgl\", \"x11\"]","declared_features":"[\"default\", \"egl\", \"glx\", \"wayland\", \"wgl\", \"x11\"]","target":5408242616063297496,"profile":9773466895796779991,"path":10833131239012105264,"deps":[[13650835054453599687,"cfg_aliases",false,17995880469106153942]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin-winit-45a8b699f6a70125\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin-winit-4f62a28089f6b9a3\lib-glutin_winit.json
{"rustc":8323788817864214825,"features":"[\"default\", \"egl\", \"glx\", \"wayland\", \"wgl\", \"x11\"]","declared_features":"[\"default\", \"egl\", \"glx\", \"wayland\", \"wgl\", \"x11\"]","target":5190775271701184214,"profile":16864349624179186615,"path":17592769584771431838,"deps":[[2901339412823178527,"winit",false,10908655721099948944],[7628053700111581507,"glutin",false,2938916037388422495],[11693073011723388840,"raw_window_handle",false,10692714251261719210],[16504075427286224702,"build_script_build",false,18190697296290361956]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin-winit-4f62a28089f6b9a3\\dep-lib-glutin_winit","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin-winit-afc1d7f491784bf9\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[16504075427286224702,"build_script_build",false,3716704012993249986]],"local":[{"Precalculated":"0.4.2"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\glutin_egl_sys-13809c4341d8347b\lib-glutin_egl_sys.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":18082394282365913375,"profile":16864349624179186615,"path":18003425468490688397,"deps":[[1999565553139417705,"windows_sys",false,9819654464114153760],[3309154526855700477,"build_script_build",false,5157481309727246345]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin_egl_sys-13809c4341d8347b\\dep-lib-glutin_egl_sys","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin_egl_sys-6cb1ed25dca8d7c8\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[3309154526855700477,"build_script_build",false,1025524543757058026]],"local":[{"RerunIfChanged":{"output":"release\\build\\glutin_egl_sys-6cb1ed25dca8d7c8\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\glutin_egl_sys-8878a6790bf21866\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":5408242616063297496,"profile":9773466895796779991,"path":4958038802411297965,"deps":[[8440717196623885952,"gl_generator",false,13322189829144030276]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin_egl_sys-8878a6790bf21866\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin_wgl_sys-15c6d085b74742af\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":5408242616063297496,"profile":9773466895796779991,"path":11757594537712821484,"deps":[[8440717196623885952,"gl_generator",false,13322189829144030276]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin_wgl_sys-15c6d085b74742af\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin_wgl_sys-21dcd08c4b50a2cb\lib-glutin_wgl_sys.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":3307031728572018049,"profile":16864349624179186615,"path":16438494626774739217,"deps":[[17088206048040325894,"build_script_build",false,16187677458223066661]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\glutin_wgl_sys-21dcd08c4b50a2cb\\dep-lib-glutin_wgl_sys","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\glutin_wgl_sys-d01013064eeb78ec\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[17088206048040325894,"build_script_build",false,745920170089284327]],"local":[{"RerunIfChanged":{"output":"release\\build\\glutin_wgl_sys-d01013064eeb78ec\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\icu_collections-ceab2632acb70bcb\lib-icu_collections.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"alloc\", \"databake\", \"serde\"]","target":8741949119514994751,"profile":16864349624179186615,"path":14251036214295350254,"deps":[[697207654067905947,"yoke",false,9548063385926952084],[1847693542725807353,"potential_utf",false,1927420664097066388],[5298260564258778412,"displaydoc",false,5465584798189317805],[14563910249377136032,"zerovec",false,15069352263028099581],[17046516144589451410,"zerofrom",false,5381117248792976011]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_collections-ceab2632acb70bcb\\dep-lib-icu_collections","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_locale_core-66b1d8e3c34e2b9f\lib-icu_locale_core.json
{"rustc":8323788817864214825,"features":"[\"zerovec\"]","declared_features":"[\"alloc\", \"databake\", \"serde\", \"zerovec\"]","target":7234736894702847895,"profile":16864349624179186615,"path":14141670287884852948,"deps":[[5298260564258778412,"displaydoc",false,5465584798189317805],[11782995109291648529,"tinystr",false,2195307972023489574],[13225456964504773423,"writeable",false,6416058706092124476],[13749468390089984218,"litemap",false,14514472358414764074],[14563910249377136032,"zerovec",false,15069352263028099581]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_locale_core-66b1d8e3c34e2b9f\\dep-lib-icu_locale_core","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_normalizer-9d774dcee0f90cf1\lib-icu_normalizer.json
{"rustc":8323788817864214825,"features":"[\"compiled_data\"]","declared_features":"[\"compiled_data\", \"datagen\", \"default\", \"experimental\", \"icu_properties\", \"serde\", \"utf16_iter\", \"utf8_iter\", \"write16\"]","target":4082895731217690114,"profile":14646408770732888442,"path":5215694900145187301,"deps":[[3666196340704888985,"smallvec",false,17957084861844350500],[5251024081607271245,"icu_provider",false,660349396449765154],[8584278803131124045,"icu_normalizer_data",false,2687356093890256296],[14324911895384364736,"icu_collections",false,13760917742807498579],[14563910249377136032,"zerovec",false,15069352263028099581]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_normalizer-9d774dcee0f90cf1\\dep-lib-icu_normalizer","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_normalizer_data-53a6e447f5087243\lib-icu_normalizer_data.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":17980939898269686983,"profile":156233555137424450,"path":3629138786901935833,"deps":[[8584278803131124045,"build_script_build",false,332350615581842870]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_normalizer_data-53a6e447f5087243\\dep-lib-icu_normalizer_data","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_normalizer_data-c70f9d4753c4a3a8\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":5408242616063297496,"profile":1988152399573204263,"path":9795605497527306449,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_normalizer_data-c70f9d4753c4a3a8\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_normalizer_data-f6915b896bfdb738\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[8584278803131124045,"build_script_build",false,8104480097930576234]],"local":[{"RerunIfEnvChanged":{"var":"ICU4X_DATA_DIR","val":null}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\icu_properties-a3bf22fabb450cdb\lib-icu_properties.json
{"rustc":8323788817864214825,"features":"[\"compiled_data\"]","declared_features":"[\"alloc\", \"compiled_data\", \"datagen\", \"default\", \"serde\", \"unicode_bidi\"]","target":12882061015678277883,"profile":16864349624179186615,"path":10931247858411528024,"deps":[[3966877249195716185,"icu_locale_core",false,11082704154816482982],[5251024081607271245,"icu_provider",false,660349396449765154],[5858954507332936698,"icu_properties_data",false,7936234680257131899],[6160379875186348458,"zerotrie",false,6104110141781557187],[14324911895384364736,"icu_collections",false,13760917742807498579],[14563910249377136032,"zerovec",false,15069352263028099581]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_properties-a3bf22fabb450cdb\\dep-lib-icu_properties","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_properties_data-1b23e56110603d5e\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[5858954507332936698,"build_script_build",false,4288608400627980624]],"local":[{"RerunIfEnvChanged":{"var":"ICU4X_DATA_DIR","val":null}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\icu_properties_data-465b2a619ebe440d\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":5408242616063297496,"profile":1988152399573204263,"path":2771322250483140788,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_properties_data-465b2a619ebe440d\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_properties_data-b2f660685868a8ca\lib-icu_properties_data.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":9037757742335137726,"profile":156233555137424450,"path":4897704906425235487,"deps":[[5858954507332936698,"build_script_build",false,7065810103454927998]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_properties_data-b2f660685868a8ca\\dep-lib-icu_properties_data","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\icu_provider-68700eb1e50e022c\lib-icu_provider.json
{"rustc":8323788817864214825,"features":"[\"baked\"]","declared_features":"[\"alloc\", \"baked\", \"deserialize_bincode_1\", \"deserialize_json\", \"deserialize_postcard_1\", \"export\", \"logging\", \"serde\", \"std\", \"sync\", \"zerotrie\"]","target":8134314816311233441,"profile":16864349624179186615,"path":2013764237647945937,"deps":[[697207654067905947,"yoke",false,9548063385926952084],[3966877249195716185,"icu_locale_core",false,11082704154816482982],[5298260564258778412,"displaydoc",false,5465584798189317805],[6160379875186348458,"zerotrie",false,6104110141781557187],[13225456964504773423,"writeable",false,6416058706092124476],[14563910249377136032,"zerovec",false,15069352263028099581],[17046516144589451410,"zerofrom",false,5381117248792976011]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\icu_provider-68700eb1e50e022c\\dep-lib-icu_provider","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\idna-8d420256761b1347\lib-idna.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"compiled_data\", \"std\"]","declared_features":"[\"alloc\", \"compiled_data\", \"default\", \"std\"]","target":2602963282308965300,"profile":16864349624179186615,"path":12509413171224934471,"deps":[[3666196340704888985,"smallvec",false,17957084861844350500],[5078124415930854154,"utf8_iter",false,14212460695488746763],[15512052560677395824,"idna_adapter",false,1248574409482738497]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\idna-8d420256761b1347\\dep-lib-idna","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\idna_adapter-fb1763611004a12f\lib-idna_adapter.json
{"rustc":8323788817864214825,"features":"[\"compiled_data\"]","declared_features":"[\"compiled_data\"]","target":9682399050268992880,"profile":16864349624179186615,"path":18358264759338219617,"deps":[[13090240085421024152,"icu_normalizer",false,16223033346057105794],[18157230703293167834,"icu_properties",false,195573779475180847]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\idna_adapter-fb1763611004a12f\\dep-lib-idna_adapter","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\itoa-50b8708faf18516d\lib-itoa.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"no-panic\"]","target":18426369533666673425,"profile":16864349624179186615,"path":5341792533824183923,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\itoa-50b8708faf18516d\\dep-lib-itoa","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\khronos_api-3a7343b73f8d80b7\lib-khronos_api.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":8622573395090798477,"profile":9773466895796779991,"path":9177875259043350080,"deps":[[4891955779658748086,"build_script_build",false,970912001252602819]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\khronos_api-3a7343b73f8d80b7\\dep-lib-khronos_api","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\khronos_api-bb92728082603f6c\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[4891955779658748086,"build_script_build",false,17472426889732446561]],"local":[{"Precalculated":"3.1.0"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\khronos_api-cb0174d8da736b28\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":12318548087768197662,"profile":9773466895796779991,"path":17353263095423481414,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\khronos_api-cb0174d8da736b28\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\libloading-a18d76c1d110638a\lib-libloading.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":9378127968640496523,"profile":1198722984184668747,"path":15736612445962285787,"deps":[[6959378045035346538,"windows_link",false,6073905924303963518]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\libloading-a18d76c1d110638a\\dep-lib-libloading","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\libm-740c044675b36399\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"arch\", \"default\"]","declared_features":"[\"arch\", \"default\", \"force-soft-floats\", \"unstable\", \"unstable-float\", \"unstable-intrinsics\", \"unstable-public-internals\"]","target":5408242616063297496,"profile":11489253199352426896,"path":6451294429733614808,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\libm-740c044675b36399\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\libm-a0cf315ed1e66c6b\lib-libm.json
{"rustc":8323788817864214825,"features":"[\"arch\", \"default\"]","declared_features":"[\"arch\", \"default\", \"force-soft-floats\", \"unstable\", \"unstable-float\", \"unstable-intrinsics\", \"unstable-public-internals\"]","target":9164340821866854471,"profile":5729184360613421012,"path":7698401729962408879,"deps":[[8471564120405487369,"build_script_build",false,11949408820233758490]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\libm-a0cf315ed1e66c6b\\dep-lib-libm","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\libm-a1f1767a77da5113\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[8471564120405487369,"build_script_build",false,2194077897885067018]],"local":[{"RerunIfChanged":{"output":"release\\build\\libm-a1f1767a77da5113\\output","paths":["build.rs","configure.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\litemap-1179f2896f6a3172\lib-litemap.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"alloc\", \"databake\", \"default\", \"serde\", \"testing\", \"yoke\"]","target":6548088149557820361,"profile":16864349624179186615,"path":14891946583551509081,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\litemap-1179f2896f6a3172\\dep-lib-litemap","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\litrs-3f7b008529df19c3\lib-litrs.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"check_suffix\", \"proc-macro2\", \"unicode-xid\"]","target":16562482054466051373,"profile":9773466895796779991,"path":4827653120389698011,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\litrs-3f7b008529df19c3\\dep-lib-litrs","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\lock_api-927d06e07a5c9550\lib-lock_api.json
{"rustc":8323788817864214825,"features":"[\"atomic_usize\", \"default\"]","declared_features":"[\"arc_lock\", \"atomic_usize\", \"default\", \"nightly\", \"owning_ref\", \"serde\"]","target":16157403318809843794,"profile":16864349624179186615,"path":14788712627124669947,"deps":[[15358414700195712381,"scopeguard",false,3817905062041558784]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\lock_api-927d06e07a5c9550\\dep-lib-lock_api","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\log-066be53a42f95af2\lib-log.json
{"rustc":8323788817864214825,"features":"[\"std\"]","declared_features":"[\"kv\", \"kv_serde\", \"kv_std\", \"kv_sval\", \"kv_unstable\", \"kv_unstable_serde\", \"kv_unstable_std\", \"kv_unstable_sval\", \"max_level_debug\", \"max_level_error\", \"max_level_info\", \"max_level_off\", \"max_level_trace\", \"max_level_warn\", \"release_max_level_debug\", \"release_max_level_error\", \"release_max_level_info\", \"release_max_level_off\", \"release_max_level_trace\", \"release_max_level_warn\", \"serde\", \"serde_core\", \"std\", \"sval\", \"sval_ref\", \"value-bag\"]","target":6550155848337067049,"profile":16864349624179186615,"path":16437490559642347756,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\log-066be53a42f95af2\\dep-lib-log","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\log-cf894c5d82e8ae90\lib-log.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"kv\", \"kv_serde\", \"kv_std\", \"kv_sval\", \"kv_unstable\", \"kv_unstable_serde\", \"kv_unstable_std\", \"kv_unstable_sval\", \"max_level_debug\", \"max_level_error\", \"max_level_info\", \"max_level_off\", \"max_level_trace\", \"max_level_warn\", \"release_max_level_debug\", \"release_max_level_error\", \"release_max_level_info\", \"release_max_level_off\", \"release_max_level_trace\", \"release_max_level_warn\", \"serde\", \"serde_core\", \"std\", \"sval\", \"sval_ref\", \"value-bag\"]","target":6550155848337067049,"profile":9773466895796779991,"path":16437490559642347756,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\log-cf894c5d82e8ae90\\dep-lib-log","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\memchr-5f3459df887419bc\lib-memchr.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"std\"]","declared_features":"[\"alloc\", \"core\", \"default\", \"libc\", \"logging\", \"rustc-dep-of-std\", \"std\", \"use_std\"]","target":11745930252914242013,"profile":16864349624179186615,"path":6926474930567721152,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\memchr-5f3459df887419bc\\dep-lib-memchr","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\memoffset-3a8f687162852509\lib-memoffset.json
{"rustc":8323788817864214825,"features":"[\"default\"]","declared_features":"[\"default\", \"unstable_const\", \"unstable_offset_of\"]","target":5262764120681397832,"profile":16864349624179186615,"path":18399432575805712822,"deps":[[14643204177830147187,"build_script_build",false,3298071217710510608]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\memoffset-3a8f687162852509\\dep-lib-memoffset","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\memoffset-59ff63b74699e8ca\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\"]","declared_features":"[\"default\", \"unstable_const\", \"unstable_offset_of\"]","target":12318548087768197662,"profile":9773466895796779991,"path":944429169442052613,"deps":[[13927012481677012980,"autocfg",false,3332798563120748878]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\memoffset-59ff63b74699e8ca\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\memoffset-ef9483bf60e5953a\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[14643204177830147187,"build_script_build",false,11082219411625729130]],"local":[{"Precalculated":"0.9.1"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\miniz_oxide-82ed6a0b3c33a411\lib-miniz_oxide.json
{"rustc":8323788817864214825,"features":"[\"default\", \"simd\", \"simd-adler32\", \"with-alloc\"]","declared_features":"[\"alloc\", \"block-boundary\", \"core\", \"default\", \"rustc-dep-of-std\", \"serde\", \"simd\", \"simd-adler32\", \"std\", \"with-alloc\"]","target":8661567070972402511,"profile":9072388833977215748,"path":3643154516155220809,"deps":[[5982862185909702272,"simd_adler32",false,788125168118963293],[7911289239703230891,"adler2",false,2693871804324890753]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\miniz_oxide-82ed6a0b3c33a411\\dep-lib-miniz_oxide","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\nohash-hasher-c0d77e8de078b984\lib-nohash_hasher.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"default\", \"std\"]","target":17363221687715233408,"profile":16864349624179186615,"path":8641592410866858028,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\nohash-hasher-c0d77e8de078b984\\dep-lib-nohash_hasher","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\num-traits-73253f3a27aad766\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[5157631553186200874,"build_script_build",false,10239727151453150783]],"local":[{"RerunIfChanged":{"output":"release\\build\\num-traits-73253f3a27aad766\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\num-traits-bf5177c15cdcbce0\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"libm\", \"std\"]","declared_features":"[\"default\", \"i128\", \"libm\", \"std\"]","target":5408242616063297496,"profile":9773466895796779991,"path":2520350670549826469,"deps":[[13927012481677012980,"autocfg",false,3332798563120748878]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\num-traits-bf5177c15cdcbce0\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\num-traits-f752e4a3cbff7b61\lib-num_traits.json
{"rustc":8323788817864214825,"features":"[\"default\", \"libm\", \"std\"]","declared_features":"[\"default\", \"i128\", \"libm\", \"std\"]","target":4278088450330190724,"profile":16864349624179186615,"path":3047373628894212259,"deps":[[5157631553186200874,"build_script_build",false,14228564838609615676],[8471564120405487369,"libm",false,6050913994048154793]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\num-traits-f752e4a3cbff7b61\\dep-lib-num_traits","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\once_cell-9aba8567ae582e07\lib-once_cell.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"default\", \"race\", \"std\"]","declared_features":"[\"alloc\", \"atomic-polyfill\", \"critical-section\", \"default\", \"parking_lot\", \"portable-atomic\", \"race\", \"std\", \"unstable\"]","target":17524666916136250164,"profile":16864349624179186615,"path":4533274643142125062,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\once_cell-9aba8567ae582e07\\dep-lib-once_cell","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\option-ext-0bba4387ec036a71\lib-option_ext.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":17153617223804709240,"profile":16864349624179186615,"path":4933823910875076553,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\option-ext-0bba4387ec036a71\\dep-lib-option_ext","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\owned_ttf_parser-62cdae7bdcaaa79c\lib-owned_ttf_parser.json
{"rustc":8323788817864214825,"features":"[\"apple-layout\", \"default\", \"glyph-names\", \"gvar-alloc\", \"opentype-layout\", \"std\", \"variable-fonts\"]","declared_features":"[\"apple-layout\", \"default\", \"glyph-names\", \"gvar-alloc\", \"no-std-float\", \"opentype-layout\", \"std\", \"variable-fonts\"]","target":840748602129315102,"profile":16864349624179186615,"path":4636412497958433209,"deps":[[10434485102629434171,"ttf_parser",false,14937150787974634765]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\owned_ttf_parser-62cdae7bdcaaa79c\\dep-lib-owned_ttf_parser","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\parking_lot-5c0e7a54f03b543c\lib-parking_lot.json
{"rustc":8323788817864214825,"features":"[\"default\"]","declared_features":"[\"arc_lock\", \"deadlock_detection\", \"default\", \"hardware-lock-elision\", \"nightly\", \"owning_ref\", \"send_guard\", \"serde\"]","target":9887373948397848517,"profile":16864349624179186615,"path":12806448026601882622,"deps":[[2555121257709722468,"lock_api",false,10129745693711813857],[6545091685033313457,"parking_lot_core",false,3584103432580800462]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\parking_lot-5c0e7a54f03b543c\\dep-lib-parking_lot","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\parking_lot_core-6f6afa330edde148\lib-parking_lot_core.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"backtrace\", \"deadlock_detection\", \"nightly\", \"petgraph\"]","target":12558056885032795287,"profile":16864349624179186615,"path":274436311314251487,"deps":[[3666196340704888985,"smallvec",false,17957084861844350500],[6545091685033313457,"build_script_build",false,1027297847655144196],[6959378045035346538,"windows_link",false,6073905924303963518],[7667230146095136825,"cfg_if",false,4717990148927456231]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\parking_lot_core-6f6afa330edde148\\dep-lib-parking_lot_core","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\parking_lot_core-7b61d94b95fdc25c\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[6545091685033313457,"build_script_build",false,13583818402827938689]],"local":[{"RerunIfChanged":{"output":"release\\build\\parking_lot_core-7b61d94b95fdc25c\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\parking_lot_core-dbbdd6d85f38d174\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"backtrace\", \"deadlock_detection\", \"nightly\", \"petgraph\"]","target":5408242616063297496,"profile":9773466895796779991,"path":16725460468075377589,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\parking_lot_core-dbbdd6d85f38d174\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\percent-encoding-95a6504c2e591761\lib-percent_encoding.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"std\"]","target":6219969305134610909,"profile":16864349624179186615,"path":8441358758161468945,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\percent-encoding-95a6504c2e591761\\dep-lib-percent_encoding","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\petri_net_legacy_editor-4067af87aa6d73de\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":5408242616063297496,"profile":9773466895796779991,"path":13767053534773805487,"deps":[[13352475244373065756,"winres",false,4816063823154059705]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\petri_net_legacy_editor-4067af87aa6d73de\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\petri_net_legacy_editor-6a1e5eb0a864f73c\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[2108366079699393392,"build_script_build",false,16564102638543510419]],"local":[{"Precalculated":"13417432777.865753800s (src/ui/app/petri_app/drawing/draw_markov_window.rs)"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\png-aa688ca5d9a92051\lib-png.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"benchmarks\", \"unstable\", \"zlib-rs\"]","target":12046889002252286887,"profile":9072388833977215748,"path":3072297445707537201,"deps":[[3389776682256874761,"fdeflate",false,12785977435424952129],[7312356825837975969,"crc32fast",false,13648362039306283598],[7636735136738807108,"miniz_oxide",false,7159646662007226201],[10456045882549826531,"flate2",false,10179230080643451635],[16909888598953886583,"bitflags",false,3782616987721616857]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\png-aa688ca5d9a92051\\dep-lib-png","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\potential_utf-23c42780d1f8d2f4\lib-potential_utf.json
{"rustc":8323788817864214825,"features":"[\"zerovec\"]","declared_features":"[\"alloc\", \"databake\", \"default\", \"serde\", \"writeable\", \"zerovec\"]","target":16089386906944150126,"profile":16864349624179186615,"path":6504289948148367869,"deps":[[14563910249377136032,"zerovec",false,15069352263028099581]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\potential_utf-23c42780d1f8d2f4\\dep-lib-potential_utf","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ppv-lite86-80fd966bf85e7574\lib-ppv_lite86.json
{"rustc":8323788817864214825,"features":"[\"simd\", \"std\"]","declared_features":"[\"default\", \"no_simd\", \"simd\", \"std\"]","target":2607852365283500179,"profile":16864349624179186615,"path":14409795955615815395,"deps":[[17375358419629610217,"zerocopy",false,16411237750151551133]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ppv-lite86-80fd966bf85e7574\\dep-lib-ppv_lite86","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\proc-macro2-08affea3864d4b31\lib-proc_macro2.json
{"rustc":8323788817864214825,"features":"[\"default\", \"proc-macro\"]","declared_features":"[\"default\", \"nightly\", \"proc-macro\", \"span-locations\"]","target":369203346396300798,"profile":9773466895796779991,"path":12350216112086701673,"deps":[[4289358735036141001,"build_script_build",false,5546776675369409158],[8901712065508858692,"unicode_ident",false,7972736069442729413]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\proc-macro2-08affea3864d4b31\\dep-lib-proc_macro2","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\proc-macro2-a9517693a4ae8373\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"proc-macro\"]","declared_features":"[\"default\", \"nightly\", \"proc-macro\", \"span-locations\"]","target":5408242616063297496,"profile":9773466895796779991,"path":3996037322062779620,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\proc-macro2-a9517693a4ae8373\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\proc-macro2-b7a8bcaf0f78822d\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[4289358735036141001,"build_script_build",false,8982067860505403081]],"local":[{"RerunIfChanged":{"output":"release\\build\\proc-macro2-b7a8bcaf0f78822d\\output","paths":["src/probe/proc_macro_span.rs","src/probe/proc_macro_span_location.rs","src/probe/proc_macro_span_file.rs"]}},{"RerunIfEnvChanged":{"var":"RUSTC_BOOTSTRAP","val":null}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\pxfm-3d5679c030897850\lib-pxfm.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":3969741579377267933,"profile":16864349624179186615,"path":11743327659242715501,"deps":[[5157631553186200874,"num_traits",false,6083056013559614080]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\pxfm-3d5679c030897850\\dep-lib-pxfm","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\quote-009c724eb5d86848\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"proc-macro\"]","declared_features":"[\"default\", \"proc-macro\"]","target":5408242616063297496,"profile":9773466895796779991,"path":12802358187047263209,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\quote-009c724eb5d86848\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\quote-04d3b2406c4f5e36\lib-quote.json
{"rustc":8323788817864214825,"features":"[\"default\", \"proc-macro\"]","declared_features":"[\"default\", \"proc-macro\"]","target":8313845041260779044,"profile":9773466895796779991,"path":16052895031434757796,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"build_script_build",false,1783675294759606292]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\quote-04d3b2406c4f5e36\\dep-lib-quote","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\quote-1cfb0a9b28d8db83\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[6100504282945712449,"build_script_build",false,4214726380700668304]],"local":[{"RerunIfChanged":{"output":"release\\build\\quote-1cfb0a9b28d8db83\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\rand-8f85e8cf97492ad0\lib-rand.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"default\", \"getrandom\", \"libc\", \"rand_chacha\", \"small_rng\", \"std\", \"std_rng\"]","declared_features":"[\"alloc\", \"default\", \"getrandom\", \"libc\", \"log\", \"min_const_gen\", \"nightly\", \"packed_simd\", \"rand_chacha\", \"serde\", \"serde1\", \"simd_support\", \"small_rng\", \"std\", \"std_rng\"]","target":8827111241893198906,"profile":16864349624179186615,"path":4995703119769088475,"deps":[[1573238666360410412,"rand_chacha",false,5148185614498264397],[18130209639506977569,"rand_core",false,3296839946430016362]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\rand-8f85e8cf97492ad0\\dep-lib-rand","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\rand_chacha-cb590eedb4f7b17e\lib-rand_chacha.json
{"rustc":8323788817864214825,"features":"[\"std\"]","declared_features":"[\"default\", \"serde\", \"serde1\", \"simd\", \"std\"]","target":15766068575093147603,"profile":16864349624179186615,"path":15421302647587782209,"deps":[[12919011715531272606,"ppv_lite86",false,4627305985588530516],[18130209639506977569,"rand_core",false,3296839946430016362]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\rand_chacha-cb590eedb4f7b17e\\dep-lib-rand_chacha","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\rand_core-cca3f65145f94b28\lib-rand_core.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"getrandom\", \"std\"]","declared_features":"[\"alloc\", \"getrandom\", \"serde\", \"serde1\", \"std\"]","target":13770603672348587087,"profile":16864349624179186615,"path":10136785050900844359,"deps":[[11023519408959114924,"getrandom",false,12268623208105785830]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\rand_core-cca3f65145f94b28\\dep-lib-rand_core","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\rand_distr-3db0c5dafcd850bb\lib-rand_distr.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"default\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"serde\", \"serde1\", \"std\", \"std_math\"]","target":7560948345641947107,"profile":16864349624179186615,"path":704936813791742640,"deps":[[5157631553186200874,"num_traits",false,6083056013559614080],[13208667028893622512,"rand",false,7933888343074170714]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\rand_distr-3db0c5dafcd850bb\\dep-lib-rand_distr","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\raw-window-handle-8f991af4412377da\lib-raw_window_handle.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"std\"]","declared_features":"[\"alloc\", \"std\", \"wasm-bindgen\", \"wasm-bindgen-0-2\"]","target":10454692504300247140,"profile":16864349624179186615,"path":14505300611058510377,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\raw-window-handle-8f991af4412377da\\dep-lib-raw_window_handle","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\raw-window-handle-b5fd7484c1e2f206\lib-raw_window_handle.json
{"rustc":8323788817864214825,"features":"[\"alloc\", \"std\"]","declared_features":"[\"alloc\", \"std\"]","target":6155386952425211338,"profile":16864349624179186615,"path":15619569140450540165,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\raw-window-handle-b5fd7484c1e2f206\\dep-lib-raw_window_handle","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\rfd-256f5d802098a4cb\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[1072176636087918192,"build_script_build",false,12205050176657176877]],"local":[{"Precalculated":"0.14.1"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\rfd-364256b63b803620\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"ashpd\", \"async-std\", \"default\", \"pollster\", \"urlencoding\", \"xdg-portal\"]","declared_features":"[\"ashpd\", \"async-std\", \"common-controls-v6\", \"default\", \"file-handle-inner\", \"glib-sys\", \"gobject-sys\", \"gtk-sys\", \"gtk3\", \"pollster\", \"tokio\", \"urlencoding\", \"xdg-portal\"]","target":5408242616063297496,"profile":9773466895796779991,"path":16503879343798881450,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\rfd-364256b63b803620\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\rfd-99ef4e8bd6af22c5\lib-rfd.json
{"rustc":8323788817864214825,"features":"[\"ashpd\", \"async-std\", \"default\", \"pollster\", \"urlencoding\", \"xdg-portal\"]","declared_features":"[\"ashpd\", \"async-std\", \"common-controls-v6\", \"default\", \"file-handle-inner\", \"glib-sys\", \"gobject-sys\", \"gtk-sys\", \"gtk3\", \"pollster\", \"tokio\", \"urlencoding\", \"xdg-portal\"]","target":2038336923818351611,"profile":16864349624179186615,"path":2780524962001122584,"deps":[[1072176636087918192,"build_script_build",false,18120099260742083801],[1999565553139417705,"windows_sys",false,9819654464114153760],[4143744114649553716,"raw_window_handle",false,16672366439269497394],[10630857666389190470,"log",false,7448553794738313875]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\rfd-99ef4e8bd6af22c5\\dep-lib-rfd","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ron-b70b22f84888ef4d\lib-ron.json
{"rustc":8323788817864214825,"features":"[\"default\", \"integer128\"]","declared_features":"[\"default\", \"indexmap\", \"integer128\"]","target":402237813285985954,"profile":16864349624179186615,"path":15033706373460822060,"deps":[[3051629642231505422,"serde_derive",false,16749135748641676341],[13548984313718623784,"serde",false,11479283716925170977],[16909888598953886583,"bitflags",false,3782616987721616857],[18066890886671768183,"base64",false,6812074503322056895]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ron-b70b22f84888ef4d\\dep-lib-ron","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\scopeguard-5fdd3f9bdac96f5c\lib-scopeguard.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"default\", \"use_std\"]","target":3556356971060988614,"profile":16864349624179186615,"path":6004337641500757253,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\scopeguard-5fdd3f9bdac96f5c\\dep-lib-scopeguard","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde-36c946b900836c6f\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"derive\", \"rc\", \"serde_derive\", \"std\", \"unstable\"]","target":5408242616063297496,"profile":9773466895796779991,"path":14695494493993979351,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde-36c946b900836c6f\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde-6e0ad93b0f20f25d\lib-serde.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"derive\", \"rc\", \"serde_derive\", \"std\", \"unstable\"]","target":11327258112168116673,"profile":9773466895796779991,"path":16403180203438082193,"deps":[[11899261697793765154,"serde_core",false,12407005396840873565],[13548984313718623784,"build_script_build",false,14477864872754589718]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde-6e0ad93b0f20f25d\\dep-lib-serde","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde-833c3163c273f0ef\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"derive\", \"rc\", \"serde_derive\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"derive\", \"rc\", \"serde_derive\", \"std\", \"unstable\"]","target":5408242616063297496,"profile":9773466895796779991,"path":14695494493993979351,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde-833c3163c273f0ef\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde-84773184d0f76abd\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[13548984313718623784,"build_script_build",false,2111537209605784258]],"local":[{"RerunIfChanged":{"output":"release\\build\\serde-84773184d0f76abd\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\serde-c3d456d08aad94d8\lib-serde.json
{"rustc":8323788817864214825,"features":"[\"default\", \"derive\", \"rc\", \"serde_derive\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"derive\", \"rc\", \"serde_derive\", \"std\", \"unstable\"]","target":11327258112168116673,"profile":16864349624179186615,"path":16403180203438082193,"deps":[[3051629642231505422,"serde_derive",false,16749135748641676341],[11899261697793765154,"serde_core",false,10322155688346445210],[13548984313718623784,"build_script_build",false,9044261033171381611]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde-c3d456d08aad94d8\\dep-lib-serde","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde-d1ac855126029dc0\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[13548984313718623784,"build_script_build",false,3870308233963521808]],"local":[{"RerunIfChanged":{"output":"release\\build\\serde-d1ac855126029dc0\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\serde_core-272fed9d6b498810\lib-serde_core.json
{"rustc":8323788817864214825,"features":"[\"rc\", \"result\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"rc\", \"result\", \"std\", \"unstable\"]","target":6810695588070812737,"profile":16864349624179186615,"path":14160570826146116882,"deps":[[11899261697793765154,"build_script_build",false,9935277321128362943]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_core-272fed9d6b498810\\dep-lib-serde_core","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde_core-64e2650110492774\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"rc\", \"result\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"rc\", \"result\", \"std\", \"unstable\"]","target":5408242616063297496,"profile":9773466895796779991,"path":17268716603447190986,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_core-64e2650110492774\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde_core-6f8ab0b2c52a593e\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"result\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"rc\", \"result\", \"std\", \"unstable\"]","target":5408242616063297496,"profile":9773466895796779991,"path":17268716603447190986,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_core-6f8ab0b2c52a593e\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde_core-7c2c42ecdbbff776\lib-serde_core.json
{"rustc":8323788817864214825,"features":"[\"result\", \"std\"]","declared_features":"[\"alloc\", \"default\", \"rc\", \"result\", \"std\", \"unstable\"]","target":6810695588070812737,"profile":9773466895796779991,"path":14160570826146116882,"deps":[[11899261697793765154,"build_script_build",false,2076286460626305241]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_core-7c2c42ecdbbff776\\dep-lib-serde_core","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde_core-a4b11a02c3568caa\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[11899261697793765154,"build_script_build",false,17259894610978143080]],"local":[{"RerunIfChanged":{"output":"release\\build\\serde_core-a4b11a02c3568caa\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\serde_core-dfdcedfdba429326\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[11899261697793765154,"build_script_build",false,10140819535855792800]],"local":[{"RerunIfChanged":{"output":"release\\build\\serde_core-dfdcedfdba429326\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\serde_derive-4f675820930c2394\lib-serde_derive.json
{"rustc":8323788817864214825,"features":"[\"default\"]","declared_features":"[\"default\", \"deserialize_in_place\"]","target":13076129734743110817,"profile":9773466895796779991,"path":4696545472894649449,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_derive-4f675820930c2394\\dep-lib-serde_derive","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde_json-79670dca326e0ceb\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"alloc\", \"arbitrary_precision\", \"default\", \"float_roundtrip\", \"indexmap\", \"preserve_order\", \"raw_value\", \"std\", \"unbounded_depth\"]","target":5408242616063297496,"profile":9773466895796779991,"path":12466340515086179490,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_json-79670dca326e0ceb\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\serde_json-d1f68a904108d94e\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[13795362694956882968,"build_script_build",false,10291381181999520954]],"local":[{"RerunIfChanged":{"output":"release\\build\\serde_json-d1f68a904108d94e\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\serde_json-de5679b0f8655fe6\lib-serde_json.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"alloc\", \"arbitrary_precision\", \"default\", \"float_roundtrip\", \"indexmap\", \"preserve_order\", \"raw_value\", \"std\", \"unbounded_depth\"]","target":9592559880233824070,"profile":16864349624179186615,"path":4574484665991783078,"deps":[[1363051979936526615,"memchr",false,10133561002890529370],[9938278000850417404,"itoa",false,10332839751084834900],[11899261697793765154,"serde_core",false,10322155688346445210],[12347024475581975995,"zmij",false,2490440423076979162],[13795362694956882968,"build_script_build",false,4747783072165235387]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\serde_json-de5679b0f8655fe6\\dep-lib-serde_json","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\simd-adler32-f139a51ce1a77f07\lib-simd_adler32.json
{"rustc":8323788817864214825,"features":"[\"const-generics\", \"default\", \"std\"]","declared_features":"[\"const-generics\", \"default\", \"nightly\", \"std\"]","target":13480744403352105069,"profile":16864349624179186615,"path":1072845308816100197,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\simd-adler32-f139a51ce1a77f07\\dep-lib-simd_adler32","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\smallvec-e5e79fe546a1b347\lib-smallvec.json
{"rustc":8323788817864214825,"features":"[\"const_generics\"]","declared_features":"[\"arbitrary\", \"bincode\", \"const_generics\", \"const_new\", \"debugger_visualizer\", \"drain_filter\", \"drain_keep_rest\", \"impl_bincode\", \"malloc_size_of\", \"may_dangle\", \"serde\", \"specialization\", \"union\", \"unty\", \"write\"]","target":9091769176333489034,"profile":16864349624179186615,"path":8920971961923350866,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\smallvec-e5e79fe546a1b347\\dep-lib-smallvec","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\smol_str-9b712fae041be91e\lib-smol_str.json
{"rustc":8323788817864214825,"features":"[\"default\", \"std\"]","declared_features":"[\"arbitrary\", \"default\", \"serde\", \"std\"]","target":7538947361851984637,"profile":16864349624179186615,"path":516375135770538239,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\smol_str-9b712fae041be91e\\dep-lib-smol_str","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\stable_deref_trait-389e85142004dee2\lib-stable_deref_trait.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"alloc\", \"default\", \"std\"]","target":5616890217583455155,"profile":16864349624179186615,"path":2557361157607591368,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\stable_deref_trait-389e85142004dee2\\dep-lib-stable_deref_trait","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\static_assertions-a914f4f2c3014a26\lib-static_assertions.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"nightly\"]","target":4712552111018528150,"profile":16864349624179186615,"path":13391077405841107062,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\static_assertions-a914f4f2c3014a26\\dep-lib-static_assertions","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\syn-699aa7993f024f70\lib-syn.json
{"rustc":8323788817864214825,"features":"[\"clone-impls\", \"default\", \"derive\", \"extra-traits\", \"fold\", \"parsing\", \"printing\", \"proc-macro\", \"visit\"]","declared_features":"[\"clone-impls\", \"default\", \"derive\", \"extra-traits\", \"fold\", \"full\", \"parsing\", \"printing\", \"proc-macro\", \"test\", \"visit\", \"visit-mut\"]","target":9442126953582868550,"profile":9773466895796779991,"path":18435949173312476477,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"quote",false,2222159866716857781],[8901712065508858692,"unicode_ident",false,7972736069442729413]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\syn-699aa7993f024f70\\dep-lib-syn","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\synstructure-6c53836d70ef0e38\lib-synstructure.json
{"rustc":8323788817864214825,"features":"[\"default\", \"proc-macro\"]","declared_features":"[\"default\", \"proc-macro\"]","target":14291004384071580589,"profile":9773466895796779991,"path":16276694391605560080,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\synstructure-6c53836d70ef0e38\\dep-lib-synstructure","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\tinystr-3756195c4665e6f2\lib-tinystr.json
{"rustc":8323788817864214825,"features":"[\"zerovec\"]","declared_features":"[\"alloc\", \"databake\", \"default\", \"serde\", \"std\", \"zerovec\"]","target":161691779326313357,"profile":16864349624179186615,"path":10220267106813712821,"deps":[[5298260564258778412,"displaydoc",false,5465584798189317805],[14563910249377136032,"zerovec",false,15069352263028099581]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\tinystr-3756195c4665e6f2\\dep-lib-tinystr","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\toml-8e139182813f9fe9\lib-toml.json
{"rustc":8323788817864214825,"features":"[\"default\"]","declared_features":"[\"default\", \"indexmap\", \"preserve_order\"]","target":18137309532358137380,"profile":9773466895796779991,"path":8171312386278530114,"deps":[[13548984313718623784,"serde",false,237024249810002870]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\toml-8e139182813f9fe9\\dep-lib-toml","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\ttf-parser-7a11e7ee550992ab\lib-ttf_parser.json
{"rustc":8323788817864214825,"features":"[\"apple-layout\", \"glyph-names\", \"gvar-alloc\", \"opentype-layout\", \"std\", \"variable-fonts\"]","declared_features":"[\"apple-layout\", \"core_maths\", \"default\", \"glyph-names\", \"gvar-alloc\", \"no-std-float\", \"opentype-layout\", \"std\", \"variable-fonts\"]","target":1684398895170894906,"profile":16864349624179186615,"path":15581378163787307147,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\ttf-parser-7a11e7ee550992ab\\dep-lib-ttf_parser","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\unicode-ident-5e800edad97e06e4\lib-unicode_ident.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":14045917370260632744,"profile":9773466895796779991,"path":8954379932651619167,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\unicode-ident-5e800edad97e06e4\\dep-lib-unicode_ident","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\unicode-segmentation-90b54f7ab1c6324b\lib-unicode_segmentation.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"no_std\"]","target":14369684853076716314,"profile":16864349624179186615,"path":13045525056348522248,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\unicode-segmentation-90b54f7ab1c6324b\\dep-lib-unicode_segmentation","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\url-fdc24526e0f6e7d9\lib-url.json
{"rustc":8323788817864214825,"features":"[\"std\"]","declared_features":"[\"debugger_visualizer\", \"default\", \"expose_internals\", \"serde\", \"std\"]","target":7686100221094031937,"profile":16864349624179186615,"path":3798426921989068056,"deps":[[1074175012458081222,"form_urlencoded",false,7293499410931989347],[6159443412421938570,"idna",false,990422232171725366],[6803352382179706244,"percent_encoding",false,15708728636639529212]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\url-fdc24526e0f6e7d9\\dep-lib-url","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\utf8_iter-6c5ce9322ccbb797\lib-utf8_iter.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":6216520282702351879,"profile":16864349624179186615,"path":3219035399717388099,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\utf8_iter-6c5ce9322ccbb797\\dep-lib-utf8_iter","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\version_check-76d419e08f784b67\lib-version_check.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":18099224280402537651,"profile":9773466895796779991,"path":13059201552806593124,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\version_check-76d419e08f784b67\\dep-lib-version_check","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\web-time-5029b0b56cae4d10\lib-web_time.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":12164945070175213125,"profile":13915834312088742273,"path":8760172217265483495,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\web-time-5029b0b56cae4d10\\dep-lib-web_time","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\webbrowser-529ab300f47ca54a\lib-webbrowser.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"disable-wsl\", \"hardened\", \"wasm-console\"]","target":1679616043190486840,"profile":16864349624179186615,"path":7012163891164811561,"deps":[[1528297757488249563,"url",false,18109759763277958451],[10630857666389190470,"log",false,7448553794738313875]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\webbrowser-529ab300f47ca54a\\dep-lib-webbrowser","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\winapi-08851291d8a6c1ac\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"winuser\"]","declared_features":"[\"accctrl\", \"aclapi\", \"activation\", \"adhoc\", \"appmgmt\", \"audioclient\", \"audiosessiontypes\", \"avrt\", \"basetsd\", \"bcrypt\", \"bits\", \"bits10_1\", \"bits1_5\", \"bits2_0\", \"bits2_5\", \"bits3_0\", \"bits4_0\", \"bits5_0\", \"bitscfg\", \"bitsmsg\", \"bluetoothapis\", \"bluetoothleapis\", \"bthdef\", \"bthioctl\", \"bthledef\", \"bthsdpdef\", \"bugcodes\", \"cderr\", \"cfg\", \"cfgmgr32\", \"cguid\", \"combaseapi\", \"coml2api\", \"commapi\", \"commctrl\", \"commdlg\", \"commoncontrols\", \"consoleapi\", \"corecrt\", \"corsym\", \"d2d1\", \"d2d1_1\", \"d2d1_2\", \"d2d1_3\", \"d2d1effectauthor\", \"d2d1effects\", \"d2d1effects_1\", \"d2d1effects_2\", \"d2d1svg\", \"d2dbasetypes\", \"d3d\", \"d3d10\", \"d3d10_1\", \"d3d10_1shader\", \"d3d10effect\", \"d3d10misc\", \"d3d10sdklayers\", \"d3d10shader\", \"d3d11\", \"d3d11_1\", \"d3d11_2\", \"d3d11_3\", \"d3d11_4\", \"d3d11on12\", \"d3d11sdklayers\", \"d3d11shader\", \"d3d11tokenizedprogramformat\", \"d3d12\", \"d3d12sdklayers\", \"d3d12shader\", \"d3d9\", \"d3d9caps\", \"d3d9types\", \"d3dcommon\", \"d3dcompiler\", \"d3dcsx\", \"d3dkmdt\", \"d3dkmthk\", \"d3dukmdt\", \"d3dx10core\", \"d3dx10math\", \"d3dx10mesh\", \"datetimeapi\", \"davclnt\", \"dbghelp\", \"dbt\", \"dcommon\", \"dcomp\", \"dcompanimation\", \"dcomptypes\", \"dde\", \"ddraw\", \"ddrawi\", \"ddrawint\", \"debug\", \"debugapi\", \"devguid\", \"devicetopology\", \"devpkey\", \"devpropdef\", \"dinput\", \"dinputd\", \"dispex\", \"dmksctl\", \"dmusicc\", \"docobj\", \"documenttarget\", \"dot1x\", \"dpa_dsa\", \"dpapi\", \"dsgetdc\", \"dsound\", \"dsrole\", \"dvp\", \"dwmapi\", \"dwrite\", \"dwrite_1\", \"dwrite_2\", \"dwrite_3\", \"dxdiag\", \"dxfile\", \"dxgi\", \"dxgi1_2\", \"dxgi1_3\", \"dxgi1_4\", \"dxgi1_5\", \"dxgi1_6\", \"dxgidebug\", \"dxgiformat\", \"dxgitype\", \"dxva2api\", \"dxvahd\", \"eaptypes\", \"enclaveapi\", \"endpointvolume\", \"errhandlingapi\", \"everything\", \"evntcons\", \"evntprov\", \"evntrace\", \"excpt\", \"exdisp\", \"fibersapi\", \"fileapi\", \"functiondiscoverykeys_devpkey\", \"gl-gl\", \"guiddef\", \"handleapi\", \"heapapi\", \"hidclass\", \"hidpi\", \"hidsdi\", \"hidusage\", \"highlevelmonitorconfigurationapi\", \"hstring\", \"http\", \"ifdef\", \"ifmib\", \"imm\", \"impl-debug\", \"impl-default\", \"in6addr\", \"inaddr\", \"inspectable\", \"interlockedapi\", \"intsafe\", \"ioapiset\", \"ipexport\", \"iphlpapi\", \"ipifcons\", \"ipmib\", \"iprtrmib\", \"iptypes\", \"jobapi\", \"jobapi2\", \"knownfolders\", \"ks\", \"ksmedia\", \"ktmtypes\", \"ktmw32\", \"l2cmn\", \"libloaderapi\", \"limits\", \"lmaccess\", \"lmalert\", \"lmapibuf\", \"lmat\", \"lmcons\", \"lmdfs\", \"lmerrlog\", \"lmjoin\", \"lmmsg\", \"lmremutl\", \"lmrepl\", \"lmserver\", \"lmshare\", \"lmstats\", \"lmsvc\", \"lmuse\", \"lmwksta\", \"lowlevelmonitorconfigurationapi\", \"lsalookup\", \"memoryapi\", \"minschannel\", \"minwinbase\", \"minwindef\", \"mmdeviceapi\", \"mmeapi\", \"mmreg\", \"mmsystem\", \"mprapidef\", \"msaatext\", \"mscat\", \"mschapp\", \"mssip\", \"mstcpip\", \"mswsock\", \"mswsockdef\", \"namedpipeapi\", \"namespaceapi\", \"nb30\", \"ncrypt\", \"netioapi\", \"nldef\", \"ntddndis\", \"ntddscsi\", \"ntddser\", \"ntdef\", \"ntlsa\", \"ntsecapi\", \"ntstatus\", \"oaidl\", \"objbase\", \"objidl\", \"objidlbase\", \"ocidl\", \"ole2\", \"oleauto\", \"olectl\", \"oleidl\", \"opmapi\", \"pdh\", \"perflib\", \"physicalmonitorenumerationapi\", \"playsoundapi\", \"portabledevice\", \"portabledeviceapi\", \"portabledevicetypes\", \"powerbase\", \"powersetting\", \"powrprof\", \"processenv\", \"processsnapshot\", \"processthreadsapi\", \"processtopologyapi\", \"profileapi\", \"propidl\", \"propkey\", \"propkeydef\", \"propsys\", \"prsht\", \"psapi\", \"qos\", \"realtimeapiset\", \"reason\", \"restartmanager\", \"restrictederrorinfo\", \"rmxfguid\", \"roapi\", \"robuffer\", \"roerrorapi\", \"rpc\", \"rpcdce\", \"rpcndr\", \"rtinfo\", \"sapi\", \"sapi51\", \"sapi53\", \"sapiddk\", \"sapiddk51\", \"schannel\", \"sddl\", \"securityappcontainer\", \"securitybaseapi\", \"servprov\", \"setupapi\", \"shellapi\", \"shellscalingapi\", \"shlobj\", \"shobjidl\", \"shobjidl_core\", \"shtypes\", \"softpub\", \"spapidef\", \"spellcheck\", \"sporder\", \"sql\", \"sqlext\", \"sqltypes\", \"sqlucode\", \"sspi\", \"std\", \"stralign\", \"stringapiset\", \"strmif\", \"subauth\", \"synchapi\", \"sysinfoapi\", \"systemtopologyapi\", \"taskschd\", \"tcpestats\", \"tcpmib\", \"textstor\", \"threadpoolapiset\", \"threadpoollegacyapiset\", \"timeapi\", \"timezoneapi\", \"tlhelp32\", \"transportsettingcommon\", \"tvout\", \"udpmib\", \"unknwnbase\", \"urlhist\", \"urlmon\", \"usb\", \"usbioctl\", \"usbiodef\", \"usbscan\", \"usbspec\", \"userenv\", \"usp10\", \"utilapiset\", \"uxtheme\", \"vadefs\", \"vcruntime\", \"vsbackup\", \"vss\", \"vsserror\", \"vswriter\", \"wbemads\", \"wbemcli\", \"wbemdisp\", \"wbemprov\", \"wbemtran\", \"wct\", \"werapi\", \"winbase\", \"wincodec\", \"wincodecsdk\", \"wincon\", \"wincontypes\", \"wincred\", \"wincrypt\", \"windef\", \"windot11\", \"windowsceip\", \"windowsx\", \"winefs\", \"winerror\", \"winevt\", \"wingdi\", \"winhttp\", \"wininet\", \"winineti\", \"winioctl\", \"winnetwk\", \"winnls\", \"winnt\", \"winreg\", \"winsafer\", \"winscard\", \"winsmcrd\", \"winsock2\", \"winspool\", \"winstring\", \"winsvc\", \"wintrust\", \"winusb\", \"winusbio\", \"winuser\", \"winver\", \"wlanapi\", \"wlanihv\", \"wlanihvtypes\", \"wlantypes\", \"wlclient\", \"wmistr\", \"wnnc\", \"wow64apiset\", \"wpdmtpextensions\", \"ws2bth\", \"ws2def\", \"ws2ipdef\", \"ws2spi\", \"ws2tcpip\", \"wtsapi32\", \"wtypes\", \"wtypesbase\", \"xinput\"]","target":12318548087768197662,"profile":9773466895796779991,"path":18129784766136936092,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\winapi-08851291d8a6c1ac\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\winapi-370cb2d3ca32eca6\lib-winapi.json
{"rustc":8323788817864214825,"features":"[\"winuser\"]","declared_features":"[\"accctrl\", \"aclapi\", \"activation\", \"adhoc\", \"appmgmt\", \"audioclient\", \"audiosessiontypes\", \"avrt\", \"basetsd\", \"bcrypt\", \"bits\", \"bits10_1\", \"bits1_5\", \"bits2_0\", \"bits2_5\", \"bits3_0\", \"bits4_0\", \"bits5_0\", \"bitscfg\", \"bitsmsg\", \"bluetoothapis\", \"bluetoothleapis\", \"bthdef\", \"bthioctl\", \"bthledef\", \"bthsdpdef\", \"bugcodes\", \"cderr\", \"cfg\", \"cfgmgr32\", \"cguid\", \"combaseapi\", \"coml2api\", \"commapi\", \"commctrl\", \"commdlg\", \"commoncontrols\", \"consoleapi\", \"corecrt\", \"corsym\", \"d2d1\", \"d2d1_1\", \"d2d1_2\", \"d2d1_3\", \"d2d1effectauthor\", \"d2d1effects\", \"d2d1effects_1\", \"d2d1effects_2\", \"d2d1svg\", \"d2dbasetypes\", \"d3d\", \"d3d10\", \"d3d10_1\", \"d3d10_1shader\", \"d3d10effect\", \"d3d10misc\", \"d3d10sdklayers\", \"d3d10shader\", \"d3d11\", \"d3d11_1\", \"d3d11_2\", \"d3d11_3\", \"d3d11_4\", \"d3d11on12\", \"d3d11sdklayers\", \"d3d11shader\", \"d3d11tokenizedprogramformat\", \"d3d12\", \"d3d12sdklayers\", \"d3d12shader\", \"d3d9\", \"d3d9caps\", \"d3d9types\", \"d3dcommon\", \"d3dcompiler\", \"d3dcsx\", \"d3dkmdt\", \"d3dkmthk\", \"d3dukmdt\", \"d3dx10core\", \"d3dx10math\", \"d3dx10mesh\", \"datetimeapi\", \"davclnt\", \"dbghelp\", \"dbt\", \"dcommon\", \"dcomp\", \"dcompanimation\", \"dcomptypes\", \"dde\", \"ddraw\", \"ddrawi\", \"ddrawint\", \"debug\", \"debugapi\", \"devguid\", \"devicetopology\", \"devpkey\", \"devpropdef\", \"dinput\", \"dinputd\", \"dispex\", \"dmksctl\", \"dmusicc\", \"docobj\", \"documenttarget\", \"dot1x\", \"dpa_dsa\", \"dpapi\", \"dsgetdc\", \"dsound\", \"dsrole\", \"dvp\", \"dwmapi\", \"dwrite\", \"dwrite_1\", \"dwrite_2\", \"dwrite_3\", \"dxdiag\", \"dxfile\", \"dxgi\", \"dxgi1_2\", \"dxgi1_3\", \"dxgi1_4\", \"dxgi1_5\", \"dxgi1_6\", \"dxgidebug\", \"dxgiformat\", \"dxgitype\", \"dxva2api\", \"dxvahd\", \"eaptypes\", \"enclaveapi\", \"endpointvolume\", \"errhandlingapi\", \"everything\", \"evntcons\", \"evntprov\", \"evntrace\", \"excpt\", \"exdisp\", \"fibersapi\", \"fileapi\", \"functiondiscoverykeys_devpkey\", \"gl-gl\", \"guiddef\", \"handleapi\", \"heapapi\", \"hidclass\", \"hidpi\", \"hidsdi\", \"hidusage\", \"highlevelmonitorconfigurationapi\", \"hstring\", \"http\", \"ifdef\", \"ifmib\", \"imm\", \"impl-debug\", \"impl-default\", \"in6addr\", \"inaddr\", \"inspectable\", \"interlockedapi\", \"intsafe\", \"ioapiset\", \"ipexport\", \"iphlpapi\", \"ipifcons\", \"ipmib\", \"iprtrmib\", \"iptypes\", \"jobapi\", \"jobapi2\", \"knownfolders\", \"ks\", \"ksmedia\", \"ktmtypes\", \"ktmw32\", \"l2cmn\", \"libloaderapi\", \"limits\", \"lmaccess\", \"lmalert\", \"lmapibuf\", \"lmat\", \"lmcons\", \"lmdfs\", \"lmerrlog\", \"lmjoin\", \"lmmsg\", \"lmremutl\", \"lmrepl\", \"lmserver\", \"lmshare\", \"lmstats\", \"lmsvc\", \"lmuse\", \"lmwksta\", \"lowlevelmonitorconfigurationapi\", \"lsalookup\", \"memoryapi\", \"minschannel\", \"minwinbase\", \"minwindef\", \"mmdeviceapi\", \"mmeapi\", \"mmreg\", \"mmsystem\", \"mprapidef\", \"msaatext\", \"mscat\", \"mschapp\", \"mssip\", \"mstcpip\", \"mswsock\", \"mswsockdef\", \"namedpipeapi\", \"namespaceapi\", \"nb30\", \"ncrypt\", \"netioapi\", \"nldef\", \"ntddndis\", \"ntddscsi\", \"ntddser\", \"ntdef\", \"ntlsa\", \"ntsecapi\", \"ntstatus\", \"oaidl\", \"objbase\", \"objidl\", \"objidlbase\", \"ocidl\", \"ole2\", \"oleauto\", \"olectl\", \"oleidl\", \"opmapi\", \"pdh\", \"perflib\", \"physicalmonitorenumerationapi\", \"playsoundapi\", \"portabledevice\", \"portabledeviceapi\", \"portabledevicetypes\", \"powerbase\", \"powersetting\", \"powrprof\", \"processenv\", \"processsnapshot\", \"processthreadsapi\", \"processtopologyapi\", \"profileapi\", \"propidl\", \"propkey\", \"propkeydef\", \"propsys\", \"prsht\", \"psapi\", \"qos\", \"realtimeapiset\", \"reason\", \"restartmanager\", \"restrictederrorinfo\", \"rmxfguid\", \"roapi\", \"robuffer\", \"roerrorapi\", \"rpc\", \"rpcdce\", \"rpcndr\", \"rtinfo\", \"sapi\", \"sapi51\", \"sapi53\", \"sapiddk\", \"sapiddk51\", \"schannel\", \"sddl\", \"securityappcontainer\", \"securitybaseapi\", \"servprov\", \"setupapi\", \"shellapi\", \"shellscalingapi\", \"shlobj\", \"shobjidl\", \"shobjidl_core\", \"shtypes\", \"softpub\", \"spapidef\", \"spellcheck\", \"sporder\", \"sql\", \"sqlext\", \"sqltypes\", \"sqlucode\", \"sspi\", \"std\", \"stralign\", \"stringapiset\", \"strmif\", \"subauth\", \"synchapi\", \"sysinfoapi\", \"systemtopologyapi\", \"taskschd\", \"tcpestats\", \"tcpmib\", \"textstor\", \"threadpoolapiset\", \"threadpoollegacyapiset\", \"timeapi\", \"timezoneapi\", \"tlhelp32\", \"transportsettingcommon\", \"tvout\", \"udpmib\", \"unknwnbase\", \"urlhist\", \"urlmon\", \"usb\", \"usbioctl\", \"usbiodef\", \"usbscan\", \"usbspec\", \"userenv\", \"usp10\", \"utilapiset\", \"uxtheme\", \"vadefs\", \"vcruntime\", \"vsbackup\", \"vss\", \"vsserror\", \"vswriter\", \"wbemads\", \"wbemcli\", \"wbemdisp\", \"wbemprov\", \"wbemtran\", \"wct\", \"werapi\", \"winbase\", \"wincodec\", \"wincodecsdk\", \"wincon\", \"wincontypes\", \"wincred\", \"wincrypt\", \"windef\", \"windot11\", \"windowsceip\", \"windowsx\", \"winefs\", \"winerror\", \"winevt\", \"wingdi\", \"winhttp\", \"wininet\", \"winineti\", \"winioctl\", \"winnetwk\", \"winnls\", \"winnt\", \"winreg\", \"winsafer\", \"winscard\", \"winsmcrd\", \"winsock2\", \"winspool\", \"winstring\", \"winsvc\", \"wintrust\", \"winusb\", \"winusbio\", \"winuser\", \"winver\", \"wlanapi\", \"wlanihv\", \"wlanihvtypes\", \"wlantypes\", \"wlclient\", \"wmistr\", \"wnnc\", \"wow64apiset\", \"wpdmtpextensions\", \"ws2bth\", \"ws2def\", \"ws2ipdef\", \"ws2spi\", \"ws2tcpip\", \"wtsapi32\", \"wtypes\", \"wtypesbase\", \"xinput\"]","target":10040225253703075495,"profile":16864349624179186615,"path":7792988503677897761,"deps":[[10020888071089587331,"build_script_build",false,11885591821008538797]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\winapi-370cb2d3ca32eca6\\dep-lib-winapi","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\winapi-3f14200d95eaa2cb\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[10020888071089587331,"build_script_build",false,15241861915121636911]],"local":[{"RerunIfChanged":{"output":"release\\build\\winapi-3f14200d95eaa2cb\\output","paths":["build.rs"]}},{"RerunIfEnvChanged":{"var":"WINAPI_NO_BUNDLED_LIBRARIES","val":null}},{"RerunIfEnvChanged":{"var":"WINAPI_STATIC_NOBUNDLE","val":null}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\windows-link-287acb0d9b05b3b8\lib-windows_link.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":2558631941022679061,"profile":10833886183550092482,"path":2971064302890974324,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows-link-287acb0d9b05b3b8\\dep-lib-windows_link","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows-sys-0bda650039902571\lib-windows_sys.json
{"rustc":8323788817864214825,"features":"[\"Win32\", \"Win32_Devices\", \"Win32_Devices_HumanInterfaceDevice\", \"Win32_Foundation\", \"Win32_Globalization\", \"Win32_Graphics\", \"Win32_Graphics_Dwm\", \"Win32_Graphics_Gdi\", \"Win32_Graphics_OpenGL\", \"Win32_Media\", \"Win32_System\", \"Win32_System_Com\", \"Win32_System_Com_StructuredStorage\", \"Win32_System_LibraryLoader\", \"Win32_System_Ole\", \"Win32_System_SystemInformation\", \"Win32_System_SystemServices\", \"Win32_System_Threading\", \"Win32_System_WindowsProgramming\", \"Win32_UI\", \"Win32_UI_Accessibility\", \"Win32_UI_Controls\", \"Win32_UI_HiDpi\", \"Win32_UI_Input\", \"Win32_UI_Input_Ime\", \"Win32_UI_Input_KeyboardAndMouse\", \"Win32_UI_Input_Pointer\", \"Win32_UI_Input_Touch\", \"Win32_UI_Shell\", \"Win32_UI_Shell_Common\", \"Win32_UI_TextServices\", \"Win32_UI_WindowsAndMessaging\", \"default\"]","declared_features":"[\"Wdk\", \"Wdk_System\", \"Wdk_System_OfflineRegistry\", \"Win32\", \"Win32_Data\", \"Win32_Data_HtmlHelp\", \"Win32_Data_RightsManagement\", \"Win32_Data_Xml\", \"Win32_Data_Xml_MsXml\", \"Win32_Data_Xml_XmlLite\", \"Win32_Devices\", \"Win32_Devices_AllJoyn\", \"Win32_Devices_BiometricFramework\", \"Win32_Devices_Bluetooth\", \"Win32_Devices_Communication\", \"Win32_Devices_DeviceAccess\", \"Win32_Devices_DeviceAndDriverInstallation\", \"Win32_Devices_DeviceQuery\", \"Win32_Devices_Display\", \"Win32_Devices_Enumeration\", \"Win32_Devices_Enumeration_Pnp\", \"Win32_Devices_Fax\", \"Win32_Devices_FunctionDiscovery\", \"Win32_Devices_Geolocation\", \"Win32_Devices_HumanInterfaceDevice\", \"Win32_Devices_ImageAcquisition\", \"Win32_Devices_PortableDevices\", \"Win32_Devices_Properties\", \"Win32_Devices_Pwm\", \"Win32_Devices_Sensors\", \"Win32_Devices_SerialCommunication\", \"Win32_Devices_Tapi\", \"Win32_Devices_Usb\", \"Win32_Devices_WebServicesOnDevices\", \"Win32_Foundation\", \"Win32_Gaming\", \"Win32_Globalization\", \"Win32_Graphics\", \"Win32_Graphics_Dwm\", \"Win32_Graphics_Gdi\", \"Win32_Graphics_Hlsl\", \"Win32_Graphics_OpenGL\", \"Win32_Graphics_Printing\", \"Win32_Graphics_Printing_PrintTicket\", \"Win32_Management\", \"Win32_Management_MobileDeviceManagementRegistration\", \"Win32_Media\", \"Win32_Media_Audio\", \"Win32_Media_Audio_Apo\", \"Win32_Media_Audio_DirectMusic\", \"Win32_Media_Audio_Endpoints\", \"Win32_Media_Audio_XAudio2\", \"Win32_Media_DeviceManager\", \"Win32_Media_DxMediaObjects\", \"Win32_Media_KernelStreaming\", \"Win32_Media_LibrarySharingServices\", \"Win32_Media_MediaPlayer\", \"Win32_Media_Multimedia\", \"Win32_Media_Speech\", \"Win32_Media_Streaming\", \"Win32_Media_WindowsMediaFormat\", \"Win32_NetworkManagement\", \"Win32_NetworkManagement_Dhcp\", \"Win32_NetworkManagement_Dns\", \"Win32_NetworkManagement_InternetConnectionWizard\", \"Win32_NetworkManagement_IpHelper\", \"Win32_NetworkManagement_MobileBroadband\", \"Win32_NetworkManagement_Multicast\", \"Win32_NetworkManagement_Ndis\", \"Win32_NetworkManagement_NetBios\", \"Win32_NetworkManagement_NetManagement\", \"Win32_NetworkManagement_NetShell\", \"Win32_NetworkManagement_NetworkDiagnosticsFramework\", \"Win32_NetworkManagement_NetworkPolicyServer\", \"Win32_NetworkManagement_P2P\", \"Win32_NetworkManagement_QoS\", \"Win32_NetworkManagement_Rras\", \"Win32_NetworkManagement_Snmp\", \"Win32_NetworkManagement_WNet\", \"Win32_NetworkManagement_WebDav\", \"Win32_NetworkManagement_WiFi\", \"Win32_NetworkManagement_WindowsConnectNow\", \"Win32_NetworkManagement_WindowsConnectionManager\", \"Win32_NetworkManagement_WindowsFilteringPlatform\", \"Win32_NetworkManagement_WindowsFirewall\", \"Win32_NetworkManagement_WindowsNetworkVirtualization\", \"Win32_Networking\", \"Win32_Networking_ActiveDirectory\", \"Win32_Networking_BackgroundIntelligentTransferService\", \"Win32_Networking_Clustering\", \"Win32_Networking_HttpServer\", \"Win32_Networking_Ldap\", \"Win32_Networking_NetworkListManager\", \"Win32_Networking_RemoteDifferentialCompression\", \"Win32_Networking_WebSocket\", \"Win32_Networking_WinHttp\", \"Win32_Networking_WinInet\", \"Win32_Networking_WinSock\", \"Win32_Networking_WindowsWebServices\", \"Win32_Security\", \"Win32_Security_AppLocker\", \"Win32_Security_Authentication\", \"Win32_Security_Authentication_Identity\", \"Win32_Security_Authentication_Identity_Provider\", \"Win32_Security_Authorization\", \"Win32_Security_Authorization_UI\", \"Win32_Security_ConfigurationSnapin\", \"Win32_Security_Credentials\", \"Win32_Security_Cryptography\", \"Win32_Security_Cryptography_Catalog\", \"Win32_Security_Cryptography_Certificates\", \"Win32_Security_Cryptography_Sip\", \"Win32_Security_Cryptography_UI\", \"Win32_Security_DiagnosticDataQuery\", \"Win32_Security_DirectoryServices\", \"Win32_Security_EnterpriseData\", \"Win32_Security_ExtensibleAuthenticationProtocol\", \"Win32_Security_Isolation\", \"Win32_Security_LicenseProtection\", \"Win32_Security_NetworkAccessProtection\", \"Win32_Security_Tpm\", \"Win32_Security_WinTrust\", \"Win32_Security_WinWlx\", \"Win32_Storage\", \"Win32_Storage_Cabinets\", \"Win32_Storage_CloudFilters\", \"Win32_Storage_Compression\", \"Win32_Storage_DataDeduplication\", \"Win32_Storage_DistributedFileSystem\", \"Win32_Storage_EnhancedStorage\", \"Win32_Storage_FileHistory\", \"Win32_Storage_FileServerResourceManager\", \"Win32_Storage_FileSystem\", \"Win32_Storage_Imapi\", \"Win32_Storage_IndexServer\", \"Win32_Storage_InstallableFileSystems\", \"Win32_Storage_IscsiDisc\", \"Win32_Storage_Jet\", \"Win32_Storage_OfflineFiles\", \"Win32_Storage_OperationRecorder\", \"Win32_Storage_Packaging\", \"Win32_Storage_Packaging_Appx\", \"Win32_Storage_Packaging_Opc\", \"Win32_Storage_ProjectedFileSystem\", \"Win32_Storage_StructuredStorage\", \"Win32_Storage_Vhd\", \"Win32_Storage_VirtualDiskService\", \"Win32_Storage_Vss\", \"Win32_Storage_Xps\", \"Win32_Storage_Xps_Printing\", \"Win32_System\", \"Win32_System_AddressBook\", \"Win32_System_Antimalware\", \"Win32_System_ApplicationInstallationAndServicing\", \"Win32_System_ApplicationVerifier\", \"Win32_System_AssessmentTool\", \"Win32_System_ClrHosting\", \"Win32_System_Com\", \"Win32_System_Com_CallObj\", \"Win32_System_Com_ChannelCredentials\", \"Win32_System_Com_Events\", \"Win32_System_Com_Marshal\", \"Win32_System_Com_StructuredStorage\", \"Win32_System_Com_UI\", \"Win32_System_Com_Urlmon\", \"Win32_System_ComponentServices\", \"Win32_System_Console\", \"Win32_System_Contacts\", \"Win32_System_CorrelationVector\", \"Win32_System_DataExchange\", \"Win32_System_DeploymentServices\", \"Win32_System_DesktopSharing\", \"Win32_System_DeveloperLicensing\", \"Win32_System_Diagnostics\", \"Win32_System_Diagnostics_Ceip\", \"Win32_System_Diagnostics_ClrProfiling\", \"Win32_System_Diagnostics_Debug\", \"Win32_System_Diagnostics_Debug_ActiveScript\", \"Win32_System_Diagnostics_Debug_Extensions\", \"Win32_System_Diagnostics_Etw\", \"Win32_System_Diagnostics_ProcessSnapshotting\", \"Win32_System_Diagnostics_ToolHelp\", \"Win32_System_DistributedTransactionCoordinator\", \"Win32_System_Environment\", \"Win32_System_ErrorReporting\", \"Win32_System_EventCollector\", \"Win32_System_EventLog\", \"Win32_System_EventNotificationService\", \"Win32_System_GroupPolicy\", \"Win32_System_HostCompute\", \"Win32_System_HostComputeNetwork\", \"Win32_System_HostComputeSystem\", \"Win32_System_Hypervisor\", \"Win32_System_IO\", \"Win32_System_Iis\", \"Win32_System_Ioctl\", \"Win32_System_JobObjects\", \"Win32_System_Js\", \"Win32_System_Kernel\", \"Win32_System_LibraryLoader\", \"Win32_System_Mailslots\", \"Win32_System_Mapi\", \"Win32_System_Memory\", \"Win32_System_Memory_NonVolatile\", \"Win32_System_MessageQueuing\", \"Win32_System_MixedReality\", \"Win32_System_Mmc\", \"Win32_System_Ole\", \"Win32_System_ParentalControls\", \"Win32_System_PasswordManagement\", \"Win32_System_Performance\", \"Win32_System_Performance_HardwareCounterProfiling\", \"Win32_System_Pipes\", \"Win32_System_Power\", \"Win32_System_ProcessStatus\", \"Win32_System_RealTimeCommunications\", \"Win32_System_Recovery\", \"Win32_System_Registry\", \"Win32_System_RemoteAssistance\", \"Win32_System_RemoteDesktop\", \"Win32_System_RemoteManagement\", \"Win32_System_RestartManager\", \"Win32_System_Restore\", \"Win32_System_Rpc\", \"Win32_System_Search\", \"Win32_System_Search_Common\", \"Win32_System_SecurityCenter\", \"Win32_System_ServerBackup\", \"Win32_System_Services\", \"Win32_System_SettingsManagementInfrastructure\", \"Win32_System_SetupAndMigration\", \"Win32_System_Shutdown\", \"Win32_System_StationsAndDesktops\", \"Win32_System_SubsystemForLinux\", \"Win32_System_SystemInformation\", \"Win32_System_SystemServices\", \"Win32_System_TaskScheduler\", \"Win32_System_Threading\", \"Win32_System_Time\", \"Win32_System_TpmBaseServices\", \"Win32_System_UpdateAgent\", \"Win32_System_UpdateAssessment\", \"Win32_System_UserAccessLogging\", \"Win32_System_VirtualDosMachines\", \"Win32_System_WindowsProgramming\", \"Win32_System_WindowsSync\", \"Win32_System_Wmi\", \"Win32_UI\", \"Win32_UI_Accessibility\", \"Win32_UI_Animation\", \"Win32_UI_ColorSystem\", \"Win32_UI_Controls\", \"Win32_UI_Controls_Dialogs\", \"Win32_UI_Controls_RichEdit\", \"Win32_UI_HiDpi\", \"Win32_UI_Input\", \"Win32_UI_Input_Ime\", \"Win32_UI_Input_Ink\", \"Win32_UI_Input_KeyboardAndMouse\", \"Win32_UI_Input_Pointer\", \"Win32_UI_Input_Radial\", \"Win32_UI_Input_Touch\", \"Win32_UI_Input_XboxController\", \"Win32_UI_InteractionContext\", \"Win32_UI_LegacyWindowsEnvironmentFeatures\", \"Win32_UI_Magnification\", \"Win32_UI_Notifications\", \"Win32_UI_Ribbon\", \"Win32_UI_Shell\", \"Win32_UI_Shell_Common\", \"Win32_UI_Shell_PropertiesSystem\", \"Win32_UI_TabletPC\", \"Win32_UI_TextServices\", \"Win32_UI_WindowsAndMessaging\", \"Win32_UI_Wpf\", \"Win32_Web\", \"Win32_Web_InternetExplorer\", \"default\"]","target":8763985620648092641,"profile":16864349624179186615,"path":2457738085561210771,"deps":[[7977276776398531731,"windows_targets",false,4755823303349071629]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows-sys-0bda650039902571\\dep-lib-windows_sys","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows-sys-8e472fd9124f9202\lib-windows_sys.json
{"rustc":8323788817864214825,"features":"[\"Win32\", \"Win32_Foundation\", \"Win32_Graphics\", \"Win32_Graphics_Gdi\", \"Win32_Storage\", \"Win32_Storage_FileSystem\", \"Win32_System\", \"Win32_System_DataExchange\", \"Win32_System_Memory\", \"Win32_System_Ole\", \"Win32_UI\", \"Win32_UI_Shell\", \"default\"]","declared_features":"[\"Wdk\", \"Wdk_Devices\", \"Wdk_Devices_Bluetooth\", \"Wdk_Devices_HumanInterfaceDevice\", \"Wdk_Foundation\", \"Wdk_Graphics\", \"Wdk_Graphics_Direct3D\", \"Wdk_NetworkManagement\", \"Wdk_NetworkManagement_Ndis\", \"Wdk_NetworkManagement_WindowsFilteringPlatform\", \"Wdk_Storage\", \"Wdk_Storage_FileSystem\", \"Wdk_Storage_FileSystem_Minifilters\", \"Wdk_System\", \"Wdk_System_IO\", \"Wdk_System_Memory\", \"Wdk_System_OfflineRegistry\", \"Wdk_System_Registry\", \"Wdk_System_SystemInformation\", \"Wdk_System_SystemServices\", \"Wdk_System_Threading\", \"Win32\", \"Win32_Data\", \"Win32_Data_HtmlHelp\", \"Win32_Data_RightsManagement\", \"Win32_Devices\", \"Win32_Devices_AllJoyn\", \"Win32_Devices_Beep\", \"Win32_Devices_BiometricFramework\", \"Win32_Devices_Bluetooth\", \"Win32_Devices_Cdrom\", \"Win32_Devices_Communication\", \"Win32_Devices_DeviceAndDriverInstallation\", \"Win32_Devices_DeviceQuery\", \"Win32_Devices_Display\", \"Win32_Devices_Dvd\", \"Win32_Devices_Enumeration\", \"Win32_Devices_Enumeration_Pnp\", \"Win32_Devices_Fax\", \"Win32_Devices_HumanInterfaceDevice\", \"Win32_Devices_Nfc\", \"Win32_Devices_Nfp\", \"Win32_Devices_PortableDevices\", \"Win32_Devices_Properties\", \"Win32_Devices_Pwm\", \"Win32_Devices_Sensors\", \"Win32_Devices_SerialCommunication\", \"Win32_Devices_Tapi\", \"Win32_Devices_Usb\", \"Win32_Devices_WebServicesOnDevices\", \"Win32_Foundation\", \"Win32_Gaming\", \"Win32_Globalization\", \"Win32_Graphics\", \"Win32_Graphics_Dwm\", \"Win32_Graphics_Gdi\", \"Win32_Graphics_GdiPlus\", \"Win32_Graphics_Hlsl\", \"Win32_Graphics_OpenGL\", \"Win32_Graphics_Printing\", \"Win32_Graphics_Printing_PrintTicket\", \"Win32_Management\", \"Win32_Management_MobileDeviceManagementRegistration\", \"Win32_Media\", \"Win32_Media_Audio\", \"Win32_Media_DxMediaObjects\", \"Win32_Media_KernelStreaming\", \"Win32_Media_Multimedia\", \"Win32_Media_Streaming\", \"Win32_Media_WindowsMediaFormat\", \"Win32_NetworkManagement\", \"Win32_NetworkManagement_Dhcp\", \"Win32_NetworkManagement_Dns\", \"Win32_NetworkManagement_InternetConnectionWizard\", \"Win32_NetworkManagement_IpHelper\", \"Win32_NetworkManagement_Multicast\", \"Win32_NetworkManagement_Ndis\", \"Win32_NetworkManagement_NetBios\", \"Win32_NetworkManagement_NetManagement\", \"Win32_NetworkManagement_NetShell\", \"Win32_NetworkManagement_NetworkDiagnosticsFramework\", \"Win32_NetworkManagement_P2P\", \"Win32_NetworkManagement_QoS\", \"Win32_NetworkManagement_Rras\", \"Win32_NetworkManagement_Snmp\", \"Win32_NetworkManagement_WNet\", \"Win32_NetworkManagement_WebDav\", \"Win32_NetworkManagement_WiFi\", \"Win32_NetworkManagement_WindowsConnectionManager\", \"Win32_NetworkManagement_WindowsFilteringPlatform\", \"Win32_NetworkManagement_WindowsFirewall\", \"Win32_NetworkManagement_WindowsNetworkVirtualization\", \"Win32_Networking\", \"Win32_Networking_ActiveDirectory\", \"Win32_Networking_Clustering\", \"Win32_Networking_HttpServer\", \"Win32_Networking_Ldap\", \"Win32_Networking_WebSocket\", \"Win32_Networking_WinHttp\", \"Win32_Networking_WinInet\", \"Win32_Networking_WinSock\", \"Win32_Networking_WindowsWebServices\", \"Win32_Security\", \"Win32_Security_AppLocker\", \"Win32_Security_Authentication\", \"Win32_Security_Authentication_Identity\", \"Win32_Security_Authorization\", \"Win32_Security_Credentials\", \"Win32_Security_Cryptography\", \"Win32_Security_Cryptography_Catalog\", \"Win32_Security_Cryptography_Certificates\", \"Win32_Security_Cryptography_Sip\", \"Win32_Security_Cryptography_UI\", \"Win32_Security_DiagnosticDataQuery\", \"Win32_Security_DirectoryServices\", \"Win32_Security_EnterpriseData\", \"Win32_Security_ExtensibleAuthenticationProtocol\", \"Win32_Security_Isolation\", \"Win32_Security_LicenseProtection\", \"Win32_Security_NetworkAccessProtection\", \"Win32_Security_WinTrust\", \"Win32_Security_WinWlx\", \"Win32_Storage\", \"Win32_Storage_Cabinets\", \"Win32_Storage_CloudFilters\", \"Win32_Storage_Compression\", \"Win32_Storage_DistributedFileSystem\", \"Win32_Storage_FileHistory\", \"Win32_Storage_FileSystem\", \"Win32_Storage_Imapi\", \"Win32_Storage_IndexServer\", \"Win32_Storage_InstallableFileSystems\", \"Win32_Storage_IscsiDisc\", \"Win32_Storage_Jet\", \"Win32_Storage_Nvme\", \"Win32_Storage_OfflineFiles\", \"Win32_Storage_OperationRecorder\", \"Win32_Storage_Packaging\", \"Win32_Storage_Packaging_Appx\", \"Win32_Storage_ProjectedFileSystem\", \"Win32_Storage_StructuredStorage\", \"Win32_Storage_Vhd\", \"Win32_Storage_Xps\", \"Win32_System\", \"Win32_System_AddressBook\", \"Win32_System_Antimalware\", \"Win32_System_ApplicationInstallationAndServicing\", \"Win32_System_ApplicationVerifier\", \"Win32_System_ClrHosting\", \"Win32_System_Com\", \"Win32_System_Com_Marshal\", \"Win32_System_Com_StructuredStorage\", \"Win32_System_Com_Urlmon\", \"Win32_System_ComponentServices\", \"Win32_System_Console\", \"Win32_System_CorrelationVector\", \"Win32_System_DataExchange\", \"Win32_System_DeploymentServices\", \"Win32_System_DeveloperLicensing\", \"Win32_System_Diagnostics\", \"Win32_System_Diagnostics_Ceip\", \"Win32_System_Diagnostics_Debug\", \"Win32_System_Diagnostics_Debug_Extensions\", \"Win32_System_Diagnostics_Etw\", \"Win32_System_Diagnostics_ProcessSnapshotting\", \"Win32_System_Diagnostics_ToolHelp\", \"Win32_System_Diagnostics_TraceLogging\", \"Win32_System_DistributedTransactionCoordinator\", \"Win32_System_Environment\", \"Win32_System_ErrorReporting\", \"Win32_System_EventCollector\", \"Win32_System_EventLog\", \"Win32_System_EventNotificationService\", \"Win32_System_GroupPolicy\", \"Win32_System_HostCompute\", \"Win32_System_HostComputeNetwork\", \"Win32_System_HostComputeSystem\", \"Win32_System_Hypervisor\", \"Win32_System_IO\", \"Win32_System_Iis\", \"Win32_System_Ioctl\", \"Win32_System_JobObjects\", \"Win32_System_Js\", \"Win32_System_Kernel\", \"Win32_System_LibraryLoader\", \"Win32_System_Mailslots\", \"Win32_System_Mapi\", \"Win32_System_Memory\", \"Win32_System_Memory_NonVolatile\", \"Win32_System_MessageQueuing\", \"Win32_System_MixedReality\", \"Win32_System_Ole\", \"Win32_System_PasswordManagement\", \"Win32_System_Performance\", \"Win32_System_Performance_HardwareCounterProfiling\", \"Win32_System_Pipes\", \"Win32_System_Power\", \"Win32_System_ProcessStatus\", \"Win32_System_Recovery\", \"Win32_System_Registry\", \"Win32_System_RemoteDesktop\", \"Win32_System_RemoteManagement\", \"Win32_System_RestartManager\", \"Win32_System_Restore\", \"Win32_System_Rpc\", \"Win32_System_Search\", \"Win32_System_Search_Common\", \"Win32_System_SecurityCenter\", \"Win32_System_Services\", \"Win32_System_SetupAndMigration\", \"Win32_System_Shutdown\", \"Win32_System_StationsAndDesktops\", \"Win32_System_SubsystemForLinux\", \"Win32_System_SystemInformation\", \"Win32_System_SystemServices\", \"Win32_System_Threading\", \"Win32_System_Time\", \"Win32_System_TpmBaseServices\", \"Win32_System_UserAccessLogging\", \"Win32_System_Variant\", \"Win32_System_VirtualDosMachines\", \"Win32_System_WindowsProgramming\", \"Win32_System_Wmi\", \"Win32_UI\", \"Win32_UI_Accessibility\", \"Win32_UI_ColorSystem\", \"Win32_UI_Controls\", \"Win32_UI_Controls_Dialogs\", \"Win32_UI_HiDpi\", \"Win32_UI_Input\", \"Win32_UI_Input_Ime\", \"Win32_UI_Input_KeyboardAndMouse\", \"Win32_UI_Input_Pointer\", \"Win32_UI_Input_Touch\", \"Win32_UI_Input_XboxController\", \"Win32_UI_InteractionContext\", \"Win32_UI_Magnification\", \"Win32_UI_Shell\", \"Win32_UI_Shell_Common\", \"Win32_UI_Shell_PropertiesSystem\", \"Win32_UI_TabletPC\", \"Win32_UI_TextServices\", \"Win32_UI_WindowsAndMessaging\", \"Win32_Web\", \"Win32_Web_InternetExplorer\", \"default\", \"docs\"]","target":7306158158326771440,"profile":12102312631911130436,"path":14386197104959296507,"deps":[[758057172878111074,"windows_targets",false,6611214630679475216]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows-sys-8e472fd9124f9202\\dep-lib-windows_sys","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows-targets-9ddbf3a285604d82\lib-windows_targets.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":1645428365803780117,"profile":16864349624179186615,"path":4444253806216022627,"deps":[[15665326712850635925,"windows_x86_64_msvc",false,17229574572889619847]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows-targets-9ddbf3a285604d82\\dep-lib-windows_targets","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows-targets-b1e5c234d826f5d5\lib-windows_targets.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":12110220207092481134,"profile":10833886183550092482,"path":12894088756930656724,"deps":[[4937765985372346599,"windows_x86_64_msvc",false,12942390183723260550]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows-targets-b1e5c234d826f5d5\\dep-lib-windows_targets","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows_x86_64_msvc-026d9bce8b3666b7\lib-windows_x86_64_msvc.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":16215071153133045705,"profile":16864349624179186615,"path":13180171610454829358,"deps":[[15665326712850635925,"build_script_build",false,10178000697631597220]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows_x86_64_msvc-026d9bce8b3666b7\\dep-lib-windows_x86_64_msvc","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows_x86_64_msvc-0850a12dcbb9bae9\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[4937765985372346599,"build_script_build",false,10322725266179729147]],"local":[{"Precalculated":"0.53.1"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\windows_x86_64_msvc-1d407f3093490359\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[15665326712850635925,"build_script_build",false,2888514797008160530]],"local":[{"Precalculated":"0.48.5"}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\windows_x86_64_msvc-412ba69872769a8b\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":5408242616063297496,"profile":16726572182692774933,"path":3705594871246992051,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows_x86_64_msvc-412ba69872769a8b\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows_x86_64_msvc-827a3f1b9f2faf62\lib-windows_x86_64_msvc.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":3306771437825829530,"profile":10833886183550092482,"path":11296531144160753832,"deps":[[4937765985372346599,"build_script_build",false,17561876365687655082]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows_x86_64_msvc-827a3f1b9f2faf62\\dep-lib-windows_x86_64_msvc","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\windows_x86_64_msvc-a73f139970daf69b\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":17883862002600103897,"profile":9773466895796779991,"path":15668703705524727963,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\windows_x86_64_msvc-a73f139970daf69b\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\winit-2c5bbc729f5ebc90\lib-winit.json
{"rustc":8323788817864214825,"features":"[\"ahash\", \"bytemuck\", \"memmap2\", \"percent-encoding\", \"rwh_05\", \"rwh_06\", \"sctk\", \"wayland\", \"wayland-backend\", \"wayland-client\", \"wayland-protocols\", \"wayland-protocols-plasma\", \"x11\", \"x11-dl\", \"x11rb\"]","declared_features":"[\"ahash\", \"android-game-activity\", \"android-native-activity\", \"bytemuck\", \"default\", \"memmap2\", \"mint\", \"percent-encoding\", \"rwh_04\", \"rwh_05\", \"rwh_06\", \"sctk\", \"sctk-adwaita\", \"serde\", \"wayland\", \"wayland-backend\", \"wayland-client\", \"wayland-csd-adwaita\", \"wayland-csd-adwaita-crossfont\", \"wayland-csd-adwaita-notitle\", \"wayland-dlopen\", \"wayland-protocols\", \"wayland-protocols-plasma\", \"x11\", \"x11-dl\", \"x11rb\"]","target":13997923951790487333,"profile":16864349624179186615,"path":593320131594838598,"deps":[[1232198224951696867,"unicode_segmentation",false,17871489863677602357],[1999565553139417705,"windows_sys",false,9819654464114153760],[2901339412823178527,"build_script_build",false,3904751811311742457],[3571374251074753029,"smol_str",false,1729590352382452136],[3722963349756955755,"once_cell",false,7236280117719017647],[4143744114649553716,"rwh_06",false,16672366439269497394],[5130283301485625812,"cursor_icon",false,8168708282271880812],[10630857666389190470,"log",false,7448553794738313875],[11693073011723388840,"rwh_05",false,10692714251261719210],[16909888598953886583,"bitflags",false,3782616987721616857]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\winit-2c5bbc729f5ebc90\\dep-lib-winit","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\winit-8e62b20b8f500a79\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[2901339412823178527,"build_script_build",false,6866395930562380118]],"local":[{"RerunIfChanged":{"output":"release\\build\\winit-8e62b20b8f500a79\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\winit-a0fbf8848fb20608\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"ahash\", \"bytemuck\", \"memmap2\", \"percent-encoding\", \"rwh_05\", \"rwh_06\", \"sctk\", \"wayland\", \"wayland-backend\", \"wayland-client\", \"wayland-protocols\", \"wayland-protocols-plasma\", \"x11\", \"x11-dl\", \"x11rb\"]","declared_features":"[\"ahash\", \"android-game-activity\", \"android-native-activity\", \"bytemuck\", \"default\", \"memmap2\", \"mint\", \"percent-encoding\", \"rwh_04\", \"rwh_05\", \"rwh_06\", \"sctk\", \"sctk-adwaita\", \"serde\", \"wayland\", \"wayland-backend\", \"wayland-client\", \"wayland-csd-adwaita\", \"wayland-csd-adwaita-crossfont\", \"wayland-csd-adwaita-notitle\", \"wayland-dlopen\", \"wayland-protocols\", \"wayland-protocols-plasma\", \"x11\", \"x11-dl\", \"x11rb\"]","target":5408242616063297496,"profile":9773466895796779991,"path":7717739158078623616,"deps":[[13650835054453599687,"cfg_aliases",false,17995880469106153942]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\winit-a0fbf8848fb20608\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\winres-c8948a1a7e8d5524\lib-winres.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":16390929351713129067,"profile":9773466895796779991,"path":7903551490924229269,"deps":[[9280368297895604912,"toml",false,8952421407415858179]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\winres-c8948a1a7e8d5524\\dep-lib-winres","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\writeable-c925e44f94a2cca5\lib-writeable.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"alloc\", \"default\", \"either\"]","target":6209224040855486982,"profile":16864349624179186615,"path":6854617203864338775,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\writeable-c925e44f94a2cca5\\dep-lib-writeable","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\xml-rs-009dbd7891d70b42\lib-xml.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":7845153393992308883,"profile":9773466895796779991,"path":18281132635804615860,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\xml-rs-009dbd7891d70b42\\dep-lib-xml","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\yoke-derive-723b65c0620b9661\lib-yoke_derive.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":1654536213780382264,"profile":9773466895796779991,"path":16395917050406669661,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[4621990586401870511,"synstructure",false,6140569752773814453],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\yoke-derive-723b65c0620b9661\\dep-lib-yoke_derive","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\yoke-e419794fc0c45dbf\lib-yoke.json
{"rustc":8323788817864214825,"features":"[\"derive\", \"zerofrom\"]","declared_features":"[\"alloc\", \"default\", \"derive\", \"serde\", \"zerofrom\"]","target":11250006364125496299,"profile":16864349624179186615,"path":4199813506559204587,"deps":[[4776946450414566059,"yoke_derive",false,2540062123743714824],[12669569555400633618,"stable_deref_trait",false,5809568263060965358],[17046516144589451410,"zerofrom",false,5381117248792976011]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\yoke-e419794fc0c45dbf\\dep-lib-yoke","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerocopy-0c9e1ede50a9649c\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[\"simd\"]","declared_features":"[\"__internal_use_only_features_that_work_on_stable\", \"alloc\", \"derive\", \"float-nightly\", \"simd\", \"simd-nightly\", \"std\", \"zerocopy-derive\"]","target":5408242616063297496,"profile":9773466895796779991,"path":192572836075330456,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerocopy-0c9e1ede50a9649c\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerocopy-b23da971639be921\lib-zerocopy.json
{"rustc":8323788817864214825,"features":"[\"simd\"]","declared_features":"[\"__internal_use_only_features_that_work_on_stable\", \"alloc\", \"derive\", \"float-nightly\", \"simd\", \"simd-nightly\", \"std\", \"zerocopy-derive\"]","target":3084901215544504908,"profile":16864349624179186615,"path":162879423290760418,"deps":[[17375358419629610217,"build_script_build",false,9751713152782944850]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerocopy-b23da971639be921\\dep-lib-zerocopy","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerocopy-c2929f2ac2ca9d24\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[17375358419629610217,"build_script_build",false,12202312677054795314]],"local":[{"RerunIfChanged":{"output":"release\\build\\zerocopy-c2929f2ac2ca9d24\\output","paths":["build.rs","Cargo.toml"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\zerofrom-528eef59890d0cf8\lib-zerofrom.json
{"rustc":8323788817864214825,"features":"[\"derive\"]","declared_features":"[\"alloc\", \"default\", \"derive\"]","target":723370850876025358,"profile":16864349624179186615,"path":9295066938864088053,"deps":[[4022439902832367970,"zerofrom_derive",false,7350437929792741219]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerofrom-528eef59890d0cf8\\dep-lib-zerofrom","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerofrom-derive-fdc8dfaa485fd629\lib-zerofrom_derive.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":1753304412232254384,"profile":9773466895796779991,"path":98091880743512252,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[4621990586401870511,"synstructure",false,6140569752773814453],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerofrom-derive-fdc8dfaa485fd629\\dep-lib-zerofrom_derive","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerotrie-7a483616a0710d31\lib-zerotrie.json
{"rustc":8323788817864214825,"features":"[\"yoke\", \"zerofrom\"]","declared_features":"[\"alloc\", \"databake\", \"default\", \"litemap\", \"serde\", \"yoke\", \"zerofrom\", \"zerovec\"]","target":12445875338185814621,"profile":16864349624179186615,"path":6028689309800856920,"deps":[[697207654067905947,"yoke",false,9548063385926952084],[5298260564258778412,"displaydoc",false,5465584798189317805],[17046516144589451410,"zerofrom",false,5381117248792976011]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerotrie-7a483616a0710d31\\dep-lib-zerotrie","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerovec-c7693e56f36285ba\lib-zerovec.json
{"rustc":8323788817864214825,"features":"[\"derive\", \"yoke\"]","declared_features":"[\"alloc\", \"databake\", \"derive\", \"hashmap\", \"serde\", \"std\", \"yoke\"]","target":1825474209729987087,"profile":16864349624179186615,"path":1169997344555780035,"deps":[[697207654067905947,"yoke",false,9548063385926952084],[6522303474648583265,"zerovec_derive",false,8884832794083396221],[17046516144589451410,"zerofrom",false,5381117248792976011]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerovec-c7693e56f36285ba\\dep-lib-zerovec","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zerovec-derive-6302b41b7e1885f8\lib-zerovec_derive.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[]","target":14030368369369144574,"profile":9773466895796779991,"path":3079617793441518355,"deps":[[4289358735036141001,"proc_macro2",false,5526647100583999725],[6100504282945712449,"quote",false,2222159866716857781],[10420560437213941093,"syn",false,4464326096249428732]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zerovec-derive-6302b41b7e1885f8\\dep-lib-zerovec_derive","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zmij-751ee2aebcc41845\lib-zmij.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"no-panic\"]","target":16603507647234574737,"profile":16864349624179186615,"path":8622243488073196097,"deps":[[12347024475581975995,"build_script_build",false,13941981752328913563]],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zmij-751ee2aebcc41845\\dep-lib-zmij","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\.fingerprint\zmij-d2a318105d311d98\run-build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"","declared_features":"","target":0,"profile":0,"path":0,"deps":[[12347024475581975995,"build_script_build",false,1014623640146628605]],"local":[{"RerunIfChanged":{"output":"release\\build\\zmij-d2a318105d311d98\\output","paths":["build.rs"]}}],"rustflags":["-C","target-feature=+crt-static"],"config":0,"compile_kind":0}

# target\release\.fingerprint\zmij-e6461aa53dac6b73\build-script-build-script-build.json
{"rustc":8323788817864214825,"features":"[]","declared_features":"[\"no-panic\"]","target":5408242616063297496,"profile":9773466895796779991,"path":2662432289397539400,"deps":[],"local":[{"CheckDepInfo":{"dep_info":"release\\.fingerprint\\zmij-e6461aa53dac6b73\\dep-build-script-build-script-build","checksum":false}}],"rustflags":["-C","target-feature=+crt-static"],"config":2069994364910194474,"compile_kind":0}

# target\release\build\glutin_egl_sys-6cb1ed25dca8d7c8\out\egl_bindings.rs

        mod __gl_imports {
            pub use std::mem;
            pub use std::marker::Send;
            pub use std::os::raw;
        }
    

        pub mod types {
            #![allow(non_camel_case_types, non_snake_case, dead_code, missing_copy_implementations)]
    
// platform-specific aliases are unknown
// IMPORTANT: these are alises to the same level of the bindings
// the values must be defined by the user
#[allow(dead_code)]
pub type khronos_utime_nanoseconds_t = super::khronos_utime_nanoseconds_t;
#[allow(dead_code)]
pub type khronos_uint64_t = super::khronos_uint64_t;
#[allow(dead_code)]
pub type khronos_ssize_t = super::khronos_ssize_t;
pub type EGLNativeDisplayType = super::EGLNativeDisplayType;
#[allow(dead_code)]
pub type EGLNativePixmapType = super::EGLNativePixmapType;
#[allow(dead_code)]
pub type EGLNativeWindowType = super::EGLNativeWindowType;
pub type EGLint = super::EGLint;
#[allow(dead_code)]
pub type NativeDisplayType = super::NativeDisplayType;
#[allow(dead_code)]
pub type NativePixmapType = super::NativePixmapType;
#[allow(dead_code)]
pub type NativeWindowType = super::NativeWindowType;

// EGL alises
pub type Bool = EGLBoolean; // TODO: not sure
pub type EGLBoolean = super::__gl_imports::raw::c_uint;
pub type EGLenum = super::__gl_imports::raw::c_uint;
pub type EGLAttribKHR = isize;
pub type EGLAttrib = isize;
pub type EGLConfig = *const super::__gl_imports::raw::c_void;
pub type EGLContext = *const super::__gl_imports::raw::c_void;
pub type EGLDeviceEXT = *const super::__gl_imports::raw::c_void;
pub type EGLDisplay = *const super::__gl_imports::raw::c_void;
pub type EGLSurface = *const super::__gl_imports::raw::c_void;
pub type EGLClientBuffer = *const super::__gl_imports::raw::c_void;
pub enum __eglMustCastToProperFunctionPointerType_fn {}
pub type __eglMustCastToProperFunctionPointerType =
    *mut __eglMustCastToProperFunctionPointerType_fn;
pub type EGLImageKHR = *const super::__gl_imports::raw::c_void;
pub type EGLImage = *const super::__gl_imports::raw::c_void;
pub type EGLOutputLayerEXT = *const super::__gl_imports::raw::c_void;
pub type EGLOutputPortEXT = *const super::__gl_imports::raw::c_void;
pub type EGLSyncKHR = *const super::__gl_imports::raw::c_void;
pub type EGLSync = *const super::__gl_imports::raw::c_void;
pub type EGLTimeKHR = khronos_utime_nanoseconds_t;
pub type EGLTime = khronos_utime_nanoseconds_t;
pub type EGLSyncNV = *const super::__gl_imports::raw::c_void;
pub type EGLTimeNV = khronos_utime_nanoseconds_t;
pub type EGLuint64NV = khronos_utime_nanoseconds_t;
pub type EGLStreamKHR = *const super::__gl_imports::raw::c_void;
pub type EGLuint64KHR = khronos_uint64_t;
pub type EGLNativeFileDescriptorKHR = super::__gl_imports::raw::c_int;
pub type EGLsizeiANDROID = khronos_ssize_t;
pub type EGLSetBlobFuncANDROID = extern "system" fn(*const super::__gl_imports::raw::c_void,
                                                    EGLsizeiANDROID,
                                                    *const super::__gl_imports::raw::c_void,
                                                    EGLsizeiANDROID)
                                                    -> ();
pub type EGLGetBlobFuncANDROID = extern "system" fn(*const super::__gl_imports::raw::c_void,
                                                    EGLsizeiANDROID,
                                                    *mut super::__gl_imports::raw::c_void,
                                                    EGLsizeiANDROID)
                                                    -> EGLsizeiANDROID;

#[repr(C)]
pub struct EGLClientPixmapHI {
    pData: *const super::__gl_imports::raw::c_void,
    iWidth: EGLint,
    iHeight: EGLint,
    iStride: EGLint,
}

}
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_FORMAT: types::EGLenum = 0x3088;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_FORMAT_NONPRE: types::EGLenum = 0x308B;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_FORMAT_PRE: types::EGLenum = 0x308C;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_MASK_SIZE: types::EGLenum = 0x303E;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_SIZE: types::EGLenum = 0x3021;
#[allow(dead_code, non_upper_case_globals)] pub const BACK_BUFFER: types::EGLenum = 0x3084;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_ACCESS: types::EGLenum = 0x3002;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_ALLOC: types::EGLenum = 0x3003;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_ATTRIBUTE: types::EGLenum = 0x3004;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_CONFIG: types::EGLenum = 0x3005;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_CONTEXT: types::EGLenum = 0x3006;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_CURRENT_SURFACE: types::EGLenum = 0x3007;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_DEVICE_EXT: types::EGLenum = 0x322B;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_DISPLAY: types::EGLenum = 0x3008;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_MATCH: types::EGLenum = 0x3009;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_NATIVE_PIXMAP: types::EGLenum = 0x300A;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_NATIVE_WINDOW: types::EGLenum = 0x300B;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_PARAMETER: types::EGLenum = 0x300C;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_SURFACE: types::EGLenum = 0x300D;
#[allow(dead_code, non_upper_case_globals)] pub const BIND_TO_TEXTURE_RGB: types::EGLenum = 0x3039;
#[allow(dead_code, non_upper_case_globals)] pub const BIND_TO_TEXTURE_RGBA: types::EGLenum = 0x303A;
#[allow(dead_code, non_upper_case_globals)] pub const BLUE_SIZE: types::EGLenum = 0x3022;
#[allow(dead_code, non_upper_case_globals)] pub const BUFFER_AGE_EXT: types::EGLenum = 0x313D;
#[allow(dead_code, non_upper_case_globals)] pub const BUFFER_DESTROYED: types::EGLenum = 0x3095;
#[allow(dead_code, non_upper_case_globals)] pub const BUFFER_PRESERVED: types::EGLenum = 0x3094;
#[allow(dead_code, non_upper_case_globals)] pub const BUFFER_SIZE: types::EGLenum = 0x3020;
#[allow(dead_code, non_upper_case_globals)] pub const CLIENT_APIS: types::EGLenum = 0x308D;
#[allow(dead_code, non_upper_case_globals)] pub const CL_EVENT_HANDLE: types::EGLenum = 0x309C;
#[allow(dead_code, non_upper_case_globals)] pub const COLORSPACE: types::EGLenum = 0x3087;
#[allow(dead_code, non_upper_case_globals)] pub const COLORSPACE_LINEAR: types::EGLenum = 0x308A;
#[allow(dead_code, non_upper_case_globals)] pub const COLORSPACE_sRGB: types::EGLenum = 0x3089;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_BUFFER_TYPE: types::EGLenum = 0x303F;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_COMPONENT_TYPE_EXT: types::EGLenum = 0x3339;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_COMPONENT_TYPE_FIXED_EXT: types::EGLenum = 0x333A;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_COMPONENT_TYPE_FLOAT_EXT: types::EGLenum = 0x333B;
#[allow(dead_code, non_upper_case_globals)] pub const CONDITION_SATISFIED: types::EGLenum = 0x30F6;
#[allow(dead_code, non_upper_case_globals)] pub const CONFIG_CAVEAT: types::EGLenum = 0x3027;
#[allow(dead_code, non_upper_case_globals)] pub const CONFIG_ID: types::EGLenum = 0x3028;
#[allow(dead_code, non_upper_case_globals)] pub const CONFORMANT: types::EGLenum = 0x3042;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_CLIENT_TYPE: types::EGLenum = 0x3097;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_CLIENT_VERSION: types::EGLenum = 0x3098;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_FLAGS_KHR: types::EGLenum = 0x30FC;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_LOST: types::EGLenum = 0x300E;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_MAJOR_VERSION: types::EGLenum = 0x3098;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_MAJOR_VERSION_KHR: types::EGLenum = 0x3098;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_MINOR_VERSION: types::EGLenum = 0x30FB;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_MINOR_VERSION_KHR: types::EGLenum = 0x30FB;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_COMPATIBILITY_PROFILE_BIT: types::EGLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_COMPATIBILITY_PROFILE_BIT_KHR: types::EGLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_CORE_PROFILE_BIT: types::EGLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_CORE_PROFILE_BIT_KHR: types::EGLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_DEBUG: types::EGLenum = 0x31B0;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_DEBUG_BIT_KHR: types::EGLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_FORWARD_COMPATIBLE: types::EGLenum = 0x31B1;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_FORWARD_COMPATIBLE_BIT_KHR: types::EGLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_NO_ERROR_KHR: types::EGLenum = 0x31B3;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_PROFILE_MASK: types::EGLenum = 0x30FD;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_PROFILE_MASK_KHR: types::EGLenum = 0x30FD;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY: types::EGLenum = 0x31BD;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY_EXT: types::EGLenum = 0x3138;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY_KHR: types::EGLenum = 0x31BD;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_ROBUST_ACCESS: types::EGLenum = 0x31B2;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_ROBUST_ACCESS_BIT_KHR: types::EGLenum = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_ROBUST_ACCESS_EXT: types::EGLenum = 0x30BF;
#[allow(dead_code, non_upper_case_globals)] pub const CORE_NATIVE_ENGINE: types::EGLenum = 0x305B;
#[allow(dead_code, non_upper_case_globals)] pub const DEFAULT_DISPLAY: types::EGLNativeDisplayType = 0 as types::EGLNativeDisplayType;
#[allow(dead_code, non_upper_case_globals)] pub const DEPTH_SIZE: types::EGLenum = 0x3025;
#[allow(dead_code, non_upper_case_globals)] pub const DEVICE_EXT: types::EGLenum = 0x322C;
#[allow(dead_code, non_upper_case_globals)] pub const DISPLAY_SCALING: types::EGLenum = 10000;
#[allow(dead_code, non_upper_case_globals)] pub const DONT_CARE: types::EGLint = -1 as types::EGLint;
#[allow(dead_code, non_upper_case_globals)] pub const DRAW: types::EGLenum = 0x3059;
#[allow(dead_code, non_upper_case_globals)] pub const DRM_DEVICE_FILE_EXT: types::EGLenum = 0x3233;
#[allow(dead_code, non_upper_case_globals)] pub const DRM_MASTER_FD_EXT: types::EGLenum = 0x333C;
#[allow(dead_code, non_upper_case_globals)] pub const EXTENSIONS: types::EGLenum = 0x3055;
#[allow(dead_code, non_upper_case_globals)] pub const FALSE: types::EGLBoolean = 0;
#[allow(dead_code, non_upper_case_globals)] pub const FOREVER: types::EGLuint64KHR = 0xFFFFFFFFFFFFFFFF;
#[allow(dead_code, non_upper_case_globals)] pub const GL_COLORSPACE: types::EGLenum = 0x309D;
#[allow(dead_code, non_upper_case_globals)] pub const GL_COLORSPACE_LINEAR: types::EGLenum = 0x308A;
#[allow(dead_code, non_upper_case_globals)] pub const GL_COLORSPACE_SRGB: types::EGLenum = 0x3089;
#[allow(dead_code, non_upper_case_globals)] pub const GL_RENDERBUFFER: types::EGLenum = 0x30B9;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_2D: types::EGLenum = 0x30B1;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_3D: types::EGLenum = 0x30B2;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_CUBE_MAP_NEGATIVE_X: types::EGLenum = 0x30B4;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_CUBE_MAP_NEGATIVE_Y: types::EGLenum = 0x30B6;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_CUBE_MAP_NEGATIVE_Z: types::EGLenum = 0x30B8;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_CUBE_MAP_POSITIVE_X: types::EGLenum = 0x30B3;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_CUBE_MAP_POSITIVE_Y: types::EGLenum = 0x30B5;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_CUBE_MAP_POSITIVE_Z: types::EGLenum = 0x30B7;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_LEVEL: types::EGLenum = 0x30BC;
#[allow(dead_code, non_upper_case_globals)] pub const GL_TEXTURE_ZOFFSET: types::EGLenum = 0x30BD;
#[allow(dead_code, non_upper_case_globals)] pub const GREEN_SIZE: types::EGLenum = 0x3023;
#[allow(dead_code, non_upper_case_globals)] pub const HEIGHT: types::EGLenum = 0x3056;
#[allow(dead_code, non_upper_case_globals)] pub const HORIZONTAL_RESOLUTION: types::EGLenum = 0x3090;
#[allow(dead_code, non_upper_case_globals)] pub const IMAGE_PRESERVED: types::EGLenum = 0x30D2;
#[allow(dead_code, non_upper_case_globals)] pub const LARGEST_PBUFFER: types::EGLenum = 0x3058;
#[allow(dead_code, non_upper_case_globals)] pub const LEVEL: types::EGLenum = 0x3029;
#[allow(dead_code, non_upper_case_globals)] pub const LOSE_CONTEXT_ON_RESET: types::EGLenum = 0x31BF;
#[allow(dead_code, non_upper_case_globals)] pub const LOSE_CONTEXT_ON_RESET_EXT: types::EGLenum = 0x31BF;
#[allow(dead_code, non_upper_case_globals)] pub const LOSE_CONTEXT_ON_RESET_KHR: types::EGLenum = 0x31BF;
#[allow(dead_code, non_upper_case_globals)] pub const LUMINANCE_BUFFER: types::EGLenum = 0x308F;
#[allow(dead_code, non_upper_case_globals)] pub const LUMINANCE_SIZE: types::EGLenum = 0x303D;
#[allow(dead_code, non_upper_case_globals)] pub const MATCH_NATIVE_PIXMAP: types::EGLenum = 0x3041;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_PBUFFER_HEIGHT: types::EGLenum = 0x302A;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_PBUFFER_PIXELS: types::EGLenum = 0x302B;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_PBUFFER_WIDTH: types::EGLenum = 0x302C;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_SWAP_INTERVAL: types::EGLenum = 0x303C;
#[allow(dead_code, non_upper_case_globals)] pub const MIN_SWAP_INTERVAL: types::EGLenum = 0x303B;
#[allow(dead_code, non_upper_case_globals)] pub const MIPMAP_LEVEL: types::EGLenum = 0x3083;
#[allow(dead_code, non_upper_case_globals)] pub const MIPMAP_TEXTURE: types::EGLenum = 0x3082;
#[allow(dead_code, non_upper_case_globals)] pub const MULTISAMPLE_RESOLVE: types::EGLenum = 0x3099;
#[allow(dead_code, non_upper_case_globals)] pub const MULTISAMPLE_RESOLVE_BOX: types::EGLenum = 0x309B;
#[allow(dead_code, non_upper_case_globals)] pub const MULTISAMPLE_RESOLVE_BOX_BIT: types::EGLenum = 0x0200;
#[allow(dead_code, non_upper_case_globals)] pub const MULTISAMPLE_RESOLVE_DEFAULT: types::EGLenum = 0x309A;
#[allow(dead_code, non_upper_case_globals)] pub const NATIVE_RENDERABLE: types::EGLenum = 0x302D;
#[allow(dead_code, non_upper_case_globals)] pub const NATIVE_VISUAL_ID: types::EGLenum = 0x302E;
#[allow(dead_code, non_upper_case_globals)] pub const NATIVE_VISUAL_TYPE: types::EGLenum = 0x302F;
#[allow(dead_code, non_upper_case_globals)] pub const NONE: types::EGLenum = 0x3038;
#[allow(dead_code, non_upper_case_globals)] pub const NON_CONFORMANT_CONFIG: types::EGLenum = 0x3051;
#[allow(dead_code, non_upper_case_globals)] pub const NOT_INITIALIZED: types::EGLenum = 0x3001;
#[allow(dead_code, non_upper_case_globals)] pub const NO_CONTEXT: types::EGLContext = 0 as types::EGLContext;
#[allow(dead_code, non_upper_case_globals)] pub const NO_DEVICE_EXT: types::EGLDeviceEXT = 0 as types::EGLDeviceEXT;
#[allow(dead_code, non_upper_case_globals)] pub const NO_DISPLAY: types::EGLDisplay = 0 as types::EGLDisplay;
#[allow(dead_code, non_upper_case_globals)] pub const NO_IMAGE: types::EGLImage = 0 as types::EGLImage;
#[allow(dead_code, non_upper_case_globals)] pub const NO_NATIVE_FENCE_FD_ANDROID: types::EGLint = -1;
#[allow(dead_code, non_upper_case_globals)] pub const NO_RESET_NOTIFICATION: types::EGLenum = 0x31BE;
#[allow(dead_code, non_upper_case_globals)] pub const NO_RESET_NOTIFICATION_EXT: types::EGLenum = 0x31BE;
#[allow(dead_code, non_upper_case_globals)] pub const NO_RESET_NOTIFICATION_KHR: types::EGLenum = 0x31BE;
#[allow(dead_code, non_upper_case_globals)] pub const NO_SURFACE: types::EGLSurface = 0 as types::EGLSurface;
#[allow(dead_code, non_upper_case_globals)] pub const NO_SYNC: types::EGLSync = 0 as types::EGLSync;
#[allow(dead_code, non_upper_case_globals)] pub const NO_TEXTURE: types::EGLenum = 0x305C;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_API: types::EGLenum = 0x30A2;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_BIT: types::EGLenum = 0x0008;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_ES2_BIT: types::EGLenum = 0x0004;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_ES3_BIT: types::EGLenum = 0x00000040;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_ES3_BIT_KHR: types::EGLenum = 0x00000040;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_ES_API: types::EGLenum = 0x30A0;
#[allow(dead_code, non_upper_case_globals)] pub const OPENGL_ES_BIT: types::EGLenum = 0x0001;
#[allow(dead_code, non_upper_case_globals)] pub const OPENVG_API: types::EGLenum = 0x30A1;
#[allow(dead_code, non_upper_case_globals)] pub const OPENVG_BIT: types::EGLenum = 0x0002;
#[allow(dead_code, non_upper_case_globals)] pub const OPENVG_IMAGE: types::EGLenum = 0x3096;
#[allow(dead_code, non_upper_case_globals)] pub const PBUFFER_BIT: types::EGLenum = 0x0001;
#[allow(dead_code, non_upper_case_globals)] pub const PIXEL_ASPECT_RATIO: types::EGLenum = 0x3092;
#[allow(dead_code, non_upper_case_globals)] pub const PIXMAP_BIT: types::EGLenum = 0x0002;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_ANDROID_KHR: types::EGLenum = 0x3141;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_DEVICE_EXT: types::EGLenum = 0x313F;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_GBM_KHR: types::EGLenum = 0x31D7;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_GBM_MESA: types::EGLenum = 0x31D7;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_WAYLAND_EXT: types::EGLenum = 0x31D8;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_WAYLAND_KHR: types::EGLenum = 0x31D8;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_X11_EXT: types::EGLenum = 0x31D5;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_X11_KHR: types::EGLenum = 0x31D5;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_X11_SCREEN_EXT: types::EGLenum = 0x31D6;
#[allow(dead_code, non_upper_case_globals)] pub const PLATFORM_X11_SCREEN_KHR: types::EGLenum = 0x31D6;
#[allow(dead_code, non_upper_case_globals)] pub const READ: types::EGLenum = 0x305A;
#[allow(dead_code, non_upper_case_globals)] pub const RED_SIZE: types::EGLenum = 0x3024;
#[allow(dead_code, non_upper_case_globals)] pub const RENDERABLE_TYPE: types::EGLenum = 0x3040;
#[allow(dead_code, non_upper_case_globals)] pub const RENDER_BUFFER: types::EGLenum = 0x3086;
#[allow(dead_code, non_upper_case_globals)] pub const RGB_BUFFER: types::EGLenum = 0x308E;
#[allow(dead_code, non_upper_case_globals)] pub const SAMPLES: types::EGLenum = 0x3031;
#[allow(dead_code, non_upper_case_globals)] pub const SAMPLE_BUFFERS: types::EGLenum = 0x3032;
#[allow(dead_code, non_upper_case_globals)] pub const SIGNALED: types::EGLenum = 0x30F2;
#[allow(dead_code, non_upper_case_globals)] pub const SINGLE_BUFFER: types::EGLenum = 0x3085;
#[allow(dead_code, non_upper_case_globals)] pub const SLOW_CONFIG: types::EGLenum = 0x3050;
#[allow(dead_code, non_upper_case_globals)] pub const STENCIL_SIZE: types::EGLenum = 0x3026;
#[allow(dead_code, non_upper_case_globals)] pub const SUCCESS: types::EGLenum = 0x3000;
#[allow(dead_code, non_upper_case_globals)] pub const SURFACE_TYPE: types::EGLenum = 0x3033;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_BEHAVIOR: types::EGLenum = 0x3093;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_BEHAVIOR_PRESERVED_BIT: types::EGLenum = 0x0400;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_CL_EVENT: types::EGLenum = 0x30FE;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_CL_EVENT_COMPLETE: types::EGLenum = 0x30FF;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_CONDITION: types::EGLenum = 0x30F8;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_CONDITION_KHR: types::EGLenum = 0x30F8;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_FENCE: types::EGLenum = 0x30F9;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_FENCE_KHR: types::EGLenum = 0x30F9;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_FLUSH_COMMANDS_BIT: types::EGLenum = 0x0001;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_NATIVE_FENCE_ANDROID: types::EGLenum = 0x3144;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_NATIVE_FENCE_FD_ANDROID: types::EGLenum = 0x3145;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_NATIVE_FENCE_SIGNALED_ANDROID: types::EGLenum = 0x3146;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_PRIOR_COMMANDS_COMPLETE: types::EGLenum = 0x30F0;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_PRIOR_COMMANDS_COMPLETE_KHR: types::EGLenum = 0x30F0;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_STATUS: types::EGLenum = 0x30F1;
#[allow(dead_code, non_upper_case_globals)] pub const SYNC_TYPE: types::EGLenum = 0x30F7;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_2D: types::EGLenum = 0x305F;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_FORMAT: types::EGLenum = 0x3080;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_RGB: types::EGLenum = 0x305D;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_RGBA: types::EGLenum = 0x305E;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_TARGET: types::EGLenum = 0x3081;
#[allow(dead_code, non_upper_case_globals)] pub const TIMEOUT_EXPIRED: types::EGLenum = 0x30F5;
#[allow(dead_code, non_upper_case_globals)] pub const TRACK_REFERENCES_KHR: types::EGLenum = 0x3352;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_BLUE_VALUE: types::EGLenum = 0x3035;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_GREEN_VALUE: types::EGLenum = 0x3036;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_RED_VALUE: types::EGLenum = 0x3037;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_RGB: types::EGLenum = 0x3052;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_TYPE: types::EGLenum = 0x3034;
#[allow(dead_code, non_upper_case_globals)] pub const TRUE: types::EGLBoolean = 1;
#[allow(dead_code, non_upper_case_globals)] pub const UNKNOWN: types::EGLint = -1 as types::EGLint;
#[allow(dead_code, non_upper_case_globals)] pub const UNSIGNALED: types::EGLenum = 0x30F3;
#[allow(dead_code, non_upper_case_globals)] pub const VENDOR: types::EGLenum = 0x3053;
#[allow(dead_code, non_upper_case_globals)] pub const VERSION: types::EGLenum = 0x3054;
#[allow(dead_code, non_upper_case_globals)] pub const VERTICAL_RESOLUTION: types::EGLenum = 0x3091;
#[allow(dead_code, non_upper_case_globals)] pub const VG_ALPHA_FORMAT: types::EGLenum = 0x3088;
#[allow(dead_code, non_upper_case_globals)] pub const VG_ALPHA_FORMAT_NONPRE: types::EGLenum = 0x308B;
#[allow(dead_code, non_upper_case_globals)] pub const VG_ALPHA_FORMAT_PRE: types::EGLenum = 0x308C;
#[allow(dead_code, non_upper_case_globals)] pub const VG_ALPHA_FORMAT_PRE_BIT: types::EGLenum = 0x0040;
#[allow(dead_code, non_upper_case_globals)] pub const VG_COLORSPACE: types::EGLenum = 0x3087;
#[allow(dead_code, non_upper_case_globals)] pub const VG_COLORSPACE_LINEAR: types::EGLenum = 0x308A;
#[allow(dead_code, non_upper_case_globals)] pub const VG_COLORSPACE_LINEAR_BIT: types::EGLenum = 0x0020;
#[allow(dead_code, non_upper_case_globals)] pub const VG_COLORSPACE_sRGB: types::EGLenum = 0x3089;
#[allow(dead_code, non_upper_case_globals)] pub const WIDTH: types::EGLenum = 0x3057;
#[allow(dead_code, non_upper_case_globals)] pub const WINDOW_BIT: types::EGLenum = 0x0004;

        #[allow(dead_code, missing_copy_implementations)]
        #[derive(Clone)]
        pub struct FnPtr {
            /// The function pointer that will be used when calling the function.
            f: *const __gl_imports::raw::c_void,
            /// True if the pointer points to a real function, false if points to a `panic!` fn.
            is_loaded: bool,
        }

        impl FnPtr {
            /// Creates a `FnPtr` from a load attempt.
            fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {
                if ptr.is_null() {
                    FnPtr {
                        f: missing_fn_panic as *const __gl_imports::raw::c_void,
                        is_loaded: false
                    }
                } else {
                    FnPtr { f: ptr, is_loaded: true }
                }
            }

            /// Returns `true` if the function has been successfully loaded.
            ///
            /// If it returns `false`, calling the corresponding function will fail.
            #[inline]
            #[allow(dead_code)]
            pub fn is_loaded(&self) -> bool {
                self.is_loaded
            }
        }
    
#[inline(never)]
        fn missing_fn_panic() -> ! {
            panic!("egl function was not loaded")
        }

        #[allow(non_camel_case_types, non_snake_case, dead_code)]
        #[derive(Clone)]
        pub struct Egl {
pub BindAPI: FnPtr,
pub BindTexImage: FnPtr,
pub ChooseConfig: FnPtr,
/// Fallbacks: ClientWaitSyncKHR
pub ClientWaitSync: FnPtr,
pub ClientWaitSyncKHR: FnPtr,
pub CopyBuffers: FnPtr,
pub CreateContext: FnPtr,
pub CreateImage: FnPtr,
pub CreatePbufferFromClientBuffer: FnPtr,
pub CreatePbufferSurface: FnPtr,
pub CreatePixmapSurface: FnPtr,
pub CreatePlatformPixmapSurface: FnPtr,
pub CreatePlatformPixmapSurfaceEXT: FnPtr,
pub CreatePlatformWindowSurface: FnPtr,
pub CreatePlatformWindowSurfaceEXT: FnPtr,
/// Fallbacks: CreateSync64KHR
pub CreateSync: FnPtr,
pub CreateSyncKHR: FnPtr,
pub CreateWindowSurface: FnPtr,
pub DestroyContext: FnPtr,
/// Fallbacks: DestroyImageKHR
pub DestroyImage: FnPtr,
pub DestroySurface: FnPtr,
/// Fallbacks: DestroySyncKHR
pub DestroySync: FnPtr,
pub DestroySyncKHR: FnPtr,
pub DupNativeFenceFDANDROID: FnPtr,
pub GetConfigAttrib: FnPtr,
pub GetConfigs: FnPtr,
pub GetCurrentContext: FnPtr,
pub GetCurrentDisplay: FnPtr,
pub GetCurrentSurface: FnPtr,
pub GetDisplay: FnPtr,
pub GetError: FnPtr,
pub GetPlatformDisplay: FnPtr,
pub GetPlatformDisplayEXT: FnPtr,
pub GetProcAddress: FnPtr,
pub GetSyncAttrib: FnPtr,
pub GetSyncAttribKHR: FnPtr,
pub Initialize: FnPtr,
pub MakeCurrent: FnPtr,
pub QueryAPI: FnPtr,
pub QueryContext: FnPtr,
pub QueryDeviceAttribEXT: FnPtr,
pub QueryDeviceStringEXT: FnPtr,
pub QueryDevicesEXT: FnPtr,
pub QueryDisplayAttribEXT: FnPtr,
/// Fallbacks: QueryDisplayAttribEXT, QueryDisplayAttribNV
pub QueryDisplayAttribKHR: FnPtr,
pub QueryString: FnPtr,
pub QuerySurface: FnPtr,
pub ReleaseTexImage: FnPtr,
pub ReleaseThread: FnPtr,
pub SurfaceAttrib: FnPtr,
pub SwapBuffers: FnPtr,
pub SwapBuffersWithDamageEXT: FnPtr,
pub SwapBuffersWithDamageKHR: FnPtr,
pub SwapInterval: FnPtr,
pub Terminate: FnPtr,
pub WaitClient: FnPtr,
pub WaitGL: FnPtr,
pub WaitNative: FnPtr,
pub WaitSync: FnPtr,
pub WaitSyncKHR: FnPtr,
_priv: ()
}
impl Egl {
            /// Load each OpenGL symbol using a custom load function. This allows for the
            /// use of functions like `glfwGetProcAddress` or `SDL_GL_GetProcAddress`.
            ///
            /// ~~~ignore
            /// let gl = Gl::load_with(|s| glfw.get_proc_address(s));
            /// ~~~
            #[allow(dead_code, unused_variables)]
            pub fn load_with<F>(mut loadfn: F) -> Egl where F: FnMut(&'static str) -> *const __gl_imports::raw::c_void {
                #[inline(never)]
                fn do_metaloadfn(loadfn: &mut dyn FnMut(&'static str) -> *const __gl_imports::raw::c_void,
                                 symbol: &'static str,
                                 symbols: &[&'static str])
                                 -> *const __gl_imports::raw::c_void {
                    let mut ptr = loadfn(symbol);
                    if ptr.is_null() {
                        for &sym in symbols {
                            ptr = loadfn(sym);
                            if !ptr.is_null() { break; }
                        }
                    }
                    ptr
                }
                let mut metaloadfn = |symbol: &'static str, symbols: &[&'static str]| {
                    do_metaloadfn(&mut loadfn, symbol, symbols)
                };
                Egl {
BindAPI: FnPtr::new(metaloadfn("eglBindAPI", &[])),
BindTexImage: FnPtr::new(metaloadfn("eglBindTexImage", &[])),
ChooseConfig: FnPtr::new(metaloadfn("eglChooseConfig", &[])),
ClientWaitSync: FnPtr::new(metaloadfn("eglClientWaitSync", &["eglClientWaitSyncKHR"])),
ClientWaitSyncKHR: FnPtr::new(metaloadfn("eglClientWaitSyncKHR", &[])),
CopyBuffers: FnPtr::new(metaloadfn("eglCopyBuffers", &[])),
CreateContext: FnPtr::new(metaloadfn("eglCreateContext", &[])),
CreateImage: FnPtr::new(metaloadfn("eglCreateImage", &[])),
CreatePbufferFromClientBuffer: FnPtr::new(metaloadfn("eglCreatePbufferFromClientBuffer", &[])),
CreatePbufferSurface: FnPtr::new(metaloadfn("eglCreatePbufferSurface", &[])),
CreatePixmapSurface: FnPtr::new(metaloadfn("eglCreatePixmapSurface", &[])),
CreatePlatformPixmapSurface: FnPtr::new(metaloadfn("eglCreatePlatformPixmapSurface", &[])),
CreatePlatformPixmapSurfaceEXT: FnPtr::new(metaloadfn("eglCreatePlatformPixmapSurfaceEXT", &[])),
CreatePlatformWindowSurface: FnPtr::new(metaloadfn("eglCreatePlatformWindowSurface", &[])),
CreatePlatformWindowSurfaceEXT: FnPtr::new(metaloadfn("eglCreatePlatformWindowSurfaceEXT", &[])),
CreateSync: FnPtr::new(metaloadfn("eglCreateSync", &["eglCreateSync64KHR"])),
CreateSyncKHR: FnPtr::new(metaloadfn("eglCreateSyncKHR", &[])),
CreateWindowSurface: FnPtr::new(metaloadfn("eglCreateWindowSurface", &[])),
DestroyContext: FnPtr::new(metaloadfn("eglDestroyContext", &[])),
DestroyImage: FnPtr::new(metaloadfn("eglDestroyImage", &["eglDestroyImageKHR"])),
DestroySurface: FnPtr::new(metaloadfn("eglDestroySurface", &[])),
DestroySync: FnPtr::new(metaloadfn("eglDestroySync", &["eglDestroySyncKHR"])),
DestroySyncKHR: FnPtr::new(metaloadfn("eglDestroySyncKHR", &[])),
DupNativeFenceFDANDROID: FnPtr::new(metaloadfn("eglDupNativeFenceFDANDROID", &[])),
GetConfigAttrib: FnPtr::new(metaloadfn("eglGetConfigAttrib", &[])),
GetConfigs: FnPtr::new(metaloadfn("eglGetConfigs", &[])),
GetCurrentContext: FnPtr::new(metaloadfn("eglGetCurrentContext", &[])),
GetCurrentDisplay: FnPtr::new(metaloadfn("eglGetCurrentDisplay", &[])),
GetCurrentSurface: FnPtr::new(metaloadfn("eglGetCurrentSurface", &[])),
GetDisplay: FnPtr::new(metaloadfn("eglGetDisplay", &[])),
GetError: FnPtr::new(metaloadfn("eglGetError", &[])),
GetPlatformDisplay: FnPtr::new(metaloadfn("eglGetPlatformDisplay", &[])),
GetPlatformDisplayEXT: FnPtr::new(metaloadfn("eglGetPlatformDisplayEXT", &[])),
GetProcAddress: FnPtr::new(metaloadfn("eglGetProcAddress", &[])),
GetSyncAttrib: FnPtr::new(metaloadfn("eglGetSyncAttrib", &[])),
GetSyncAttribKHR: FnPtr::new(metaloadfn("eglGetSyncAttribKHR", &[])),
Initialize: FnPtr::new(metaloadfn("eglInitialize", &[])),
MakeCurrent: FnPtr::new(metaloadfn("eglMakeCurrent", &[])),
QueryAPI: FnPtr::new(metaloadfn("eglQueryAPI", &[])),
QueryContext: FnPtr::new(metaloadfn("eglQueryContext", &[])),
QueryDeviceAttribEXT: FnPtr::new(metaloadfn("eglQueryDeviceAttribEXT", &[])),
QueryDeviceStringEXT: FnPtr::new(metaloadfn("eglQueryDeviceStringEXT", &[])),
QueryDevicesEXT: FnPtr::new(metaloadfn("eglQueryDevicesEXT", &[])),
QueryDisplayAttribEXT: FnPtr::new(metaloadfn("eglQueryDisplayAttribEXT", &[])),
QueryDisplayAttribKHR: FnPtr::new(metaloadfn("eglQueryDisplayAttribKHR", &["eglQueryDisplayAttribEXT", "eglQueryDisplayAttribNV"])),
QueryString: FnPtr::new(metaloadfn("eglQueryString", &[])),
QuerySurface: FnPtr::new(metaloadfn("eglQuerySurface", &[])),
ReleaseTexImage: FnPtr::new(metaloadfn("eglReleaseTexImage", &[])),
ReleaseThread: FnPtr::new(metaloadfn("eglReleaseThread", &[])),
SurfaceAttrib: FnPtr::new(metaloadfn("eglSurfaceAttrib", &[])),
SwapBuffers: FnPtr::new(metaloadfn("eglSwapBuffers", &[])),
SwapBuffersWithDamageEXT: FnPtr::new(metaloadfn("eglSwapBuffersWithDamageEXT", &[])),
SwapBuffersWithDamageKHR: FnPtr::new(metaloadfn("eglSwapBuffersWithDamageKHR", &[])),
SwapInterval: FnPtr::new(metaloadfn("eglSwapInterval", &[])),
Terminate: FnPtr::new(metaloadfn("eglTerminate", &[])),
WaitClient: FnPtr::new(metaloadfn("eglWaitClient", &[])),
WaitGL: FnPtr::new(metaloadfn("eglWaitGL", &[])),
WaitNative: FnPtr::new(metaloadfn("eglWaitNative", &[])),
WaitSync: FnPtr::new(metaloadfn("eglWaitSync", &[])),
WaitSyncKHR: FnPtr::new(metaloadfn("eglWaitSyncKHR", &[])),
_priv: ()
}
        }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn BindAPI(&self, api: types::EGLenum) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLenum) -> types::EGLBoolean>(self.BindAPI.f)(api) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn BindTexImage(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, buffer: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, types::EGLint) -> types::EGLBoolean>(self.BindTexImage.f)(dpy, surface, buffer) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ChooseConfig(&self, dpy: types::EGLDisplay, attrib_list: *const types::EGLint, configs: *mut types::EGLConfig, config_size: types::EGLint, num_config: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, *const types::EGLint, *mut types::EGLConfig, types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.ChooseConfig.f)(dpy, attrib_list, configs, config_size, num_config) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ClientWaitSync(&self, dpy: types::EGLDisplay, sync: types::EGLSync, flags: types::EGLint, timeout: types::EGLTime) -> types::EGLint { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSync, types::EGLint, types::EGLTime) -> types::EGLint>(self.ClientWaitSync.f)(dpy, sync, flags, timeout) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ClientWaitSyncKHR(&self, dpy: types::EGLDisplay, sync: types::EGLSyncKHR, flags: types::EGLint, timeout: types::EGLTimeKHR) -> types::EGLint { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSyncKHR, types::EGLint, types::EGLTimeKHR) -> types::EGLint>(self.ClientWaitSyncKHR.f)(dpy, sync, flags, timeout) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CopyBuffers(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, target: types::EGLNativePixmapType) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, types::EGLNativePixmapType) -> types::EGLBoolean>(self.CopyBuffers.f)(dpy, surface, target) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateContext(&self, dpy: types::EGLDisplay, config: types::EGLConfig, share_context: types::EGLContext, attrib_list: *const types::EGLint) -> types::EGLContext { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, types::EGLContext, *const types::EGLint) -> types::EGLContext>(self.CreateContext.f)(dpy, config, share_context, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateImage(&self, dpy: types::EGLDisplay, ctx: types::EGLContext, target: types::EGLenum, buffer: types::EGLClientBuffer, attrib_list: *const types::EGLAttrib) -> types::EGLImage { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLContext, types::EGLenum, types::EGLClientBuffer, *const types::EGLAttrib) -> types::EGLImage>(self.CreateImage.f)(dpy, ctx, target, buffer, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePbufferFromClientBuffer(&self, dpy: types::EGLDisplay, buftype: types::EGLenum, buffer: types::EGLClientBuffer, config: types::EGLConfig, attrib_list: *const types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLenum, types::EGLClientBuffer, types::EGLConfig, *const types::EGLint) -> types::EGLSurface>(self.CreatePbufferFromClientBuffer.f)(dpy, buftype, buffer, config, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePbufferSurface(&self, dpy: types::EGLDisplay, config: types::EGLConfig, attrib_list: *const types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, *const types::EGLint) -> types::EGLSurface>(self.CreatePbufferSurface.f)(dpy, config, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePixmapSurface(&self, dpy: types::EGLDisplay, config: types::EGLConfig, pixmap: types::EGLNativePixmapType, attrib_list: *const types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, types::EGLNativePixmapType, *const types::EGLint) -> types::EGLSurface>(self.CreatePixmapSurface.f)(dpy, config, pixmap, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePlatformPixmapSurface(&self, dpy: types::EGLDisplay, config: types::EGLConfig, native_pixmap: *mut __gl_imports::raw::c_void, attrib_list: *const types::EGLAttrib) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, *mut __gl_imports::raw::c_void, *const types::EGLAttrib) -> types::EGLSurface>(self.CreatePlatformPixmapSurface.f)(dpy, config, native_pixmap, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePlatformPixmapSurfaceEXT(&self, dpy: types::EGLDisplay, config: types::EGLConfig, native_pixmap: *mut __gl_imports::raw::c_void, attrib_list: *const types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, *mut __gl_imports::raw::c_void, *const types::EGLint) -> types::EGLSurface>(self.CreatePlatformPixmapSurfaceEXT.f)(dpy, config, native_pixmap, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePlatformWindowSurface(&self, dpy: types::EGLDisplay, config: types::EGLConfig, native_window: *mut __gl_imports::raw::c_void, attrib_list: *const types::EGLAttrib) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, *mut __gl_imports::raw::c_void, *const types::EGLAttrib) -> types::EGLSurface>(self.CreatePlatformWindowSurface.f)(dpy, config, native_window, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreatePlatformWindowSurfaceEXT(&self, dpy: types::EGLDisplay, config: types::EGLConfig, native_window: *mut __gl_imports::raw::c_void, attrib_list: *const types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, *mut __gl_imports::raw::c_void, *const types::EGLint) -> types::EGLSurface>(self.CreatePlatformWindowSurfaceEXT.f)(dpy, config, native_window, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateSync(&self, dpy: types::EGLDisplay, type_: types::EGLenum, attrib_list: *const types::EGLAttrib) -> types::EGLSync { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLenum, *const types::EGLAttrib) -> types::EGLSync>(self.CreateSync.f)(dpy, type_, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateSyncKHR(&self, dpy: types::EGLDisplay, type_: types::EGLenum, attrib_list: *const types::EGLint) -> types::EGLSyncKHR { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLenum, *const types::EGLint) -> types::EGLSyncKHR>(self.CreateSyncKHR.f)(dpy, type_, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateWindowSurface(&self, dpy: types::EGLDisplay, config: types::EGLConfig, win: types::EGLNativeWindowType, attrib_list: *const types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, types::EGLNativeWindowType, *const types::EGLint) -> types::EGLSurface>(self.CreateWindowSurface.f)(dpy, config, win, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DestroyContext(&self, dpy: types::EGLDisplay, ctx: types::EGLContext) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLContext) -> types::EGLBoolean>(self.DestroyContext.f)(dpy, ctx) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DestroyImage(&self, dpy: types::EGLDisplay, image: types::EGLImage) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLImage) -> types::EGLBoolean>(self.DestroyImage.f)(dpy, image) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DestroySurface(&self, dpy: types::EGLDisplay, surface: types::EGLSurface) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface) -> types::EGLBoolean>(self.DestroySurface.f)(dpy, surface) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DestroySync(&self, dpy: types::EGLDisplay, sync: types::EGLSync) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSync) -> types::EGLBoolean>(self.DestroySync.f)(dpy, sync) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DestroySyncKHR(&self, dpy: types::EGLDisplay, sync: types::EGLSyncKHR) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSyncKHR) -> types::EGLBoolean>(self.DestroySyncKHR.f)(dpy, sync) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DupNativeFenceFDANDROID(&self, dpy: types::EGLDisplay, sync: types::EGLSyncKHR) -> types::EGLint { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSyncKHR) -> types::EGLint>(self.DupNativeFenceFDANDROID.f)(dpy, sync) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetConfigAttrib(&self, dpy: types::EGLDisplay, config: types::EGLConfig, attribute: types::EGLint, value: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLConfig, types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.GetConfigAttrib.f)(dpy, config, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetConfigs(&self, dpy: types::EGLDisplay, configs: *mut types::EGLConfig, config_size: types::EGLint, num_config: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, *mut types::EGLConfig, types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.GetConfigs.f)(dpy, configs, config_size, num_config) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetCurrentContext(&self, ) -> types::EGLContext { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLContext>(self.GetCurrentContext.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetCurrentDisplay(&self, ) -> types::EGLDisplay { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLDisplay>(self.GetCurrentDisplay.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetCurrentSurface(&self, readdraw: types::EGLint) -> types::EGLSurface { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLint) -> types::EGLSurface>(self.GetCurrentSurface.f)(readdraw) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetDisplay(&self, display_id: types::EGLNativeDisplayType) -> types::EGLDisplay { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLNativeDisplayType) -> types::EGLDisplay>(self.GetDisplay.f)(display_id) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetError(&self, ) -> types::EGLint { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLint>(self.GetError.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetPlatformDisplay(&self, platform: types::EGLenum, native_display: *mut __gl_imports::raw::c_void, attrib_list: *const types::EGLAttrib) -> types::EGLDisplay { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLenum, *mut __gl_imports::raw::c_void, *const types::EGLAttrib) -> types::EGLDisplay>(self.GetPlatformDisplay.f)(platform, native_display, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetPlatformDisplayEXT(&self, platform: types::EGLenum, native_display: *mut __gl_imports::raw::c_void, attrib_list: *const types::EGLint) -> types::EGLDisplay { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLenum, *mut __gl_imports::raw::c_void, *const types::EGLint) -> types::EGLDisplay>(self.GetPlatformDisplayEXT.f)(platform, native_display, attrib_list) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetProcAddress(&self, procname: *const __gl_imports::raw::c_char) -> types::__eglMustCastToProperFunctionPointerType { __gl_imports::mem::transmute::<_, extern "system" fn(*const __gl_imports::raw::c_char) -> types::__eglMustCastToProperFunctionPointerType>(self.GetProcAddress.f)(procname) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetSyncAttrib(&self, dpy: types::EGLDisplay, sync: types::EGLSync, attribute: types::EGLint, value: *mut types::EGLAttrib) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSync, types::EGLint, *mut types::EGLAttrib) -> types::EGLBoolean>(self.GetSyncAttrib.f)(dpy, sync, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetSyncAttribKHR(&self, dpy: types::EGLDisplay, sync: types::EGLSyncKHR, attribute: types::EGLint, value: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSyncKHR, types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.GetSyncAttribKHR.f)(dpy, sync, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn Initialize(&self, dpy: types::EGLDisplay, major: *mut types::EGLint, minor: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, *mut types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.Initialize.f)(dpy, major, minor) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn MakeCurrent(&self, dpy: types::EGLDisplay, draw: types::EGLSurface, read: types::EGLSurface, ctx: types::EGLContext) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, types::EGLSurface, types::EGLContext) -> types::EGLBoolean>(self.MakeCurrent.f)(dpy, draw, read, ctx) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryAPI(&self, ) -> types::EGLenum { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLenum>(self.QueryAPI.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryContext(&self, dpy: types::EGLDisplay, ctx: types::EGLContext, attribute: types::EGLint, value: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLContext, types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.QueryContext.f)(dpy, ctx, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryDeviceAttribEXT(&self, device: types::EGLDeviceEXT, attribute: types::EGLint, value: *mut types::EGLAttrib) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDeviceEXT, types::EGLint, *mut types::EGLAttrib) -> types::EGLBoolean>(self.QueryDeviceAttribEXT.f)(device, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryDeviceStringEXT(&self, device: types::EGLDeviceEXT, name: types::EGLint) -> *const __gl_imports::raw::c_char { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDeviceEXT, types::EGLint) -> *const __gl_imports::raw::c_char>(self.QueryDeviceStringEXT.f)(device, name) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryDevicesEXT(&self, max_devices: types::EGLint, devices: *mut types::EGLDeviceEXT, num_devices: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLint, *mut types::EGLDeviceEXT, *mut types::EGLint) -> types::EGLBoolean>(self.QueryDevicesEXT.f)(max_devices, devices, num_devices) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryDisplayAttribEXT(&self, dpy: types::EGLDisplay, attribute: types::EGLint, value: *mut types::EGLAttrib) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLint, *mut types::EGLAttrib) -> types::EGLBoolean>(self.QueryDisplayAttribEXT.f)(dpy, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryDisplayAttribKHR(&self, dpy: types::EGLDisplay, name: types::EGLint, value: *mut types::EGLAttrib) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLint, *mut types::EGLAttrib) -> types::EGLBoolean>(self.QueryDisplayAttribKHR.f)(dpy, name, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QueryString(&self, dpy: types::EGLDisplay, name: types::EGLint) -> *const __gl_imports::raw::c_char { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLint) -> *const __gl_imports::raw::c_char>(self.QueryString.f)(dpy, name) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn QuerySurface(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, attribute: types::EGLint, value: *mut types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, types::EGLint, *mut types::EGLint) -> types::EGLBoolean>(self.QuerySurface.f)(dpy, surface, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ReleaseTexImage(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, buffer: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, types::EGLint) -> types::EGLBoolean>(self.ReleaseTexImage.f)(dpy, surface, buffer) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ReleaseThread(&self, ) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLBoolean>(self.ReleaseThread.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SurfaceAttrib(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, attribute: types::EGLint, value: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, types::EGLint, types::EGLint) -> types::EGLBoolean>(self.SurfaceAttrib.f)(dpy, surface, attribute, value) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SwapBuffers(&self, dpy: types::EGLDisplay, surface: types::EGLSurface) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface) -> types::EGLBoolean>(self.SwapBuffers.f)(dpy, surface) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SwapBuffersWithDamageEXT(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, rects: *mut types::EGLint, n_rects: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, *mut types::EGLint, types::EGLint) -> types::EGLBoolean>(self.SwapBuffersWithDamageEXT.f)(dpy, surface, rects, n_rects) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SwapBuffersWithDamageKHR(&self, dpy: types::EGLDisplay, surface: types::EGLSurface, rects: *mut types::EGLint, n_rects: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSurface, *mut types::EGLint, types::EGLint) -> types::EGLBoolean>(self.SwapBuffersWithDamageKHR.f)(dpy, surface, rects, n_rects) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SwapInterval(&self, dpy: types::EGLDisplay, interval: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLint) -> types::EGLBoolean>(self.SwapInterval.f)(dpy, interval) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn Terminate(&self, dpy: types::EGLDisplay) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay) -> types::EGLBoolean>(self.Terminate.f)(dpy) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn WaitClient(&self, ) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLBoolean>(self.WaitClient.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn WaitGL(&self, ) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::EGLBoolean>(self.WaitGL.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn WaitNative(&self, engine: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLint) -> types::EGLBoolean>(self.WaitNative.f)(engine) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn WaitSync(&self, dpy: types::EGLDisplay, sync: types::EGLSync, flags: types::EGLint) -> types::EGLBoolean { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSync, types::EGLint) -> types::EGLBoolean>(self.WaitSync.f)(dpy, sync, flags) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn WaitSyncKHR(&self, dpy: types::EGLDisplay, sync: types::EGLSyncKHR, flags: types::EGLint) -> types::EGLint { __gl_imports::mem::transmute::<_, extern "system" fn(types::EGLDisplay, types::EGLSyncKHR, types::EGLint) -> types::EGLint>(self.WaitSyncKHR.f)(dpy, sync, flags) }
}

        unsafe impl __gl_imports::Send for Egl {}


# target\release\build\glutin_wgl_sys-d01013064eeb78ec\out\wgl_bindings.rs

        mod __gl_imports {
            pub use std::mem;
            pub use std::os::raw;
        }
    

        pub mod types {
            #![allow(non_camel_case_types, non_snake_case, dead_code, missing_copy_implementations)]
    
// Common types from OpenGL 1.1
pub type GLenum = super::__gl_imports::raw::c_uint;
pub type GLboolean = super::__gl_imports::raw::c_uchar;
pub type GLbitfield = super::__gl_imports::raw::c_uint;
pub type GLvoid = super::__gl_imports::raw::c_void;
pub type GLbyte = super::__gl_imports::raw::c_char;
pub type GLshort = super::__gl_imports::raw::c_short;
pub type GLint = super::__gl_imports::raw::c_int;
pub type GLclampx = super::__gl_imports::raw::c_int;
pub type GLubyte = super::__gl_imports::raw::c_uchar;
pub type GLushort = super::__gl_imports::raw::c_ushort;
pub type GLuint = super::__gl_imports::raw::c_uint;
pub type GLsizei = super::__gl_imports::raw::c_int;
pub type GLfloat = super::__gl_imports::raw::c_float;
pub type GLclampf = super::__gl_imports::raw::c_float;
pub type GLdouble = super::__gl_imports::raw::c_double;
pub type GLclampd = super::__gl_imports::raw::c_double;
pub type GLeglImageOES = *const super::__gl_imports::raw::c_void;
pub type GLchar = super::__gl_imports::raw::c_char;
pub type GLcharARB = super::__gl_imports::raw::c_char;

#[cfg(target_os = "macos")]
pub type GLhandleARB = *const super::__gl_imports::raw::c_void;
#[cfg(not(target_os = "macos"))]
pub type GLhandleARB = super::__gl_imports::raw::c_uint;

pub type GLhalfARB = super::__gl_imports::raw::c_ushort;
pub type GLhalf = super::__gl_imports::raw::c_ushort;

// Must be 32 bits
pub type GLfixed = GLint;

pub type GLintptr = isize;
pub type GLsizeiptr = isize;
pub type GLint64 = i64;
pub type GLuint64 = u64;
pub type GLintptrARB = isize;
pub type GLsizeiptrARB = isize;
pub type GLint64EXT = i64;
pub type GLuint64EXT = u64;

pub enum __GLsync {}
pub type GLsync = *const __GLsync;

// compatible with OpenCL cl_context
pub enum _cl_context {}
pub enum _cl_event {}

pub type GLDEBUGPROC = Option<extern "system" fn(source: GLenum,
                                                 gltype: GLenum,
                                                 id: GLuint,
                                                 severity: GLenum,
                                                 length: GLsizei,
                                                 message: *const GLchar,
                                                 userParam: *mut super::__gl_imports::raw::c_void)>;
pub type GLDEBUGPROCARB = Option<extern "system" fn(source: GLenum,
                                                    gltype: GLenum,
                                                    id: GLuint,
                                                    severity: GLenum,
                                                    length: GLsizei,
                                                    message: *const GLchar,
                                                    userParam: *mut super::__gl_imports::raw::c_void)>;
pub type GLDEBUGPROCKHR = Option<extern "system" fn(source: GLenum,
                                                    gltype: GLenum,
                                                    id: GLuint,
                                                    severity: GLenum,
                                                    length: GLsizei,
                                                    message: *const GLchar,
                                                    userParam: *mut super::__gl_imports::raw::c_void)>;

// GLES 1 types
// "pub type GLclampx = i32;",

// GLES 1/2 types (tagged for GLES 1)
// "pub type GLbyte = i8;",
// "pub type GLubyte = u8;",
// "pub type GLfloat = GLfloat;",
// "pub type GLclampf = GLfloat;",
// "pub type GLfixed = i32;",
// "pub type GLint64 = i64;",
// "pub type GLuint64 = u64;",
// "pub type GLintptr = intptr_t;",
// "pub type GLsizeiptr = ssize_t;",

// GLES 1/2 types (tagged for GLES 2 - attribute syntax is limited)
// "pub type GLbyte = i8;",
// "pub type GLubyte = u8;",
// "pub type GLfloat = GLfloat;",
// "pub type GLclampf = GLfloat;",
// "pub type GLfixed = i32;",
// "pub type GLint64 = i64;",
// "pub type GLuint64 = u64;",
// "pub type GLint64EXT = i64;",
// "pub type GLuint64EXT = u64;",
// "pub type GLintptr = intptr_t;",
// "pub type GLsizeiptr = ssize_t;",

// GLES 2 types (none currently)

// Vendor extension types
pub type GLDEBUGPROCAMD = Option<extern "system" fn(id: GLuint,
                                                    category: GLenum,
                                                    severity: GLenum,
                                                    length: GLsizei,
                                                    message: *const GLchar,
                                                    userParam: *mut super::__gl_imports::raw::c_void)>;
pub type GLhalfNV = super::__gl_imports::raw::c_ushort;
pub type GLvdpauSurfaceNV = GLintptr;

// From WinNT.h

pub type CHAR = super::__gl_imports::raw::c_char;
pub type HANDLE = PVOID;
pub type LONG = super::__gl_imports::raw::c_long;
pub type LPCSTR = *const super::__gl_imports::raw::c_char;
pub type VOID = ();
// #define DECLARE_HANDLE(name) struct name##__{int unused;}; typedef struct name##__ *name
pub type HPBUFFERARB = *const super::__gl_imports::raw::c_void;
pub type HPBUFFEREXT = *const super::__gl_imports::raw::c_void;
pub type HVIDEOOUTPUTDEVICENV = *const super::__gl_imports::raw::c_void;
pub type HPVIDEODEV = *const super::__gl_imports::raw::c_void;
pub type HPGPUNV = *const super::__gl_imports::raw::c_void;
pub type HGPUNV = *const super::__gl_imports::raw::c_void;
pub type HVIDEOINPUTDEVICENV = *const super::__gl_imports::raw::c_void;

// From Windef.h

pub type BOOL = super::__gl_imports::raw::c_int;
pub type BYTE = super::__gl_imports::raw::c_uchar;
pub type COLORREF = DWORD;
pub type FLOAT = super::__gl_imports::raw::c_float;
pub type HDC = HANDLE;
pub type HENHMETAFILE = HANDLE;
pub type HGLRC = *const super::__gl_imports::raw::c_void;
pub type INT = super::__gl_imports::raw::c_int;
pub type PVOID = *const super::__gl_imports::raw::c_void;
pub type LPVOID = *const super::__gl_imports::raw::c_void;
pub enum __PROC_fn {}
pub type PROC = *mut __PROC_fn;

#[repr(C)]
pub struct RECT {
    left: LONG,
    top: LONG,
    right: LONG,
    bottom: LONG,
}

pub type UINT = super::__gl_imports::raw::c_uint;
pub type USHORT = super::__gl_imports::raw::c_ushort;
pub type WORD = super::__gl_imports::raw::c_ushort;

// From BaseTsd.h

pub type INT32 = i32;
pub type INT64 = i64;

// From IntSafe.h

pub type DWORD = super::__gl_imports::raw::c_ulong;

// From Wingdi.h

#[repr(C)]
pub struct POINTFLOAT {
    pub x: FLOAT,
    pub y: FLOAT,
}

#[repr(C)]
pub struct GLYPHMETRICSFLOAT {
    pub gmfBlackBoxX: FLOAT,
    pub gmfBlackBoxY: FLOAT,
    pub gmfptGlyphOrigin: POINTFLOAT,
    pub gmfCellIncX: FLOAT,
    pub gmfCellIncY: FLOAT,
}
pub type LPGLYPHMETRICSFLOAT = *const GLYPHMETRICSFLOAT;

#[repr(C)]
pub struct LAYERPLANEDESCRIPTOR {
    pub nSize: WORD,
    pub nVersion: WORD,
    pub dwFlags: DWORD,
    pub iPixelType: BYTE,
    pub cColorBits: BYTE,
    pub cRedBits: BYTE,
    pub cRedShift: BYTE,
    pub cGreenBits: BYTE,
    pub cGreenShift: BYTE,
    pub cBlueBits: BYTE,
    pub cBlueShift: BYTE,
    pub cAlphaBits: BYTE,
    pub cAlphaShift: BYTE,
    pub cAccumBits: BYTE,
    pub cAccumRedBits: BYTE,
    pub cAccumGreenBits: BYTE,
    pub cAccumBlueBits: BYTE,
    pub cAccumAlphaBits: BYTE,
    pub cDepthBits: BYTE,
    pub cStencilBits: BYTE,
    pub cAuxBuffers: BYTE,
    pub iLayerType: BYTE,
    pub bReserved: BYTE,
    pub crTransparent: COLORREF,
}

#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
    pub nSize: WORD,
    pub nVersion: WORD,
    pub dwFlags: DWORD,
    pub iPixelType: BYTE,
    pub cColorBits: BYTE,
    pub cRedBits: BYTE,
    pub cRedShift: BYTE,
    pub cGreenBits: BYTE,
    pub cGreenShift: BYTE,
    pub cBlueBits: BYTE,
    pub cBlueShift: BYTE,
    pub cAlphaBits: BYTE,
    pub cAlphaShift: BYTE,
    pub cAccumBits: BYTE,
    pub cAccumRedBits: BYTE,
    pub cAccumGreenBits: BYTE,
    pub cAccumBlueBits: BYTE,
    pub cAccumAlphaBits: BYTE,
    pub cDepthBits: BYTE,
    pub cStencilBits: BYTE,
    pub cAuxBuffers: BYTE,
    pub iLayerType: BYTE,
    pub bReserved: BYTE,
    pub dwLayerMask: DWORD,
    pub dwVisibleMask: DWORD,
    pub dwDamageMask: DWORD,
}

#[repr(C)]
pub struct _GPU_DEVICE {
    cb: DWORD,
    DeviceName: [CHAR; 32],
    DeviceString: [CHAR; 128],
    Flags: DWORD,
    rcVirtualScreen: RECT,
}

pub struct GPU_DEVICE(_GPU_DEVICE);
pub struct PGPU_DEVICE(*const _GPU_DEVICE);


        }
    
#[allow(dead_code, non_upper_case_globals)] pub const FONT_LINES: types::GLenum = 0;
#[allow(dead_code, non_upper_case_globals)] pub const FONT_POLYGONS: types::GLenum = 1;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_MAIN_PLANE: types::GLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY1: types::GLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY10: types::GLenum = 0x00000400;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY11: types::GLenum = 0x00000800;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY12: types::GLenum = 0x00001000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY13: types::GLenum = 0x00002000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY14: types::GLenum = 0x00004000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY15: types::GLenum = 0x00008000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY2: types::GLenum = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY3: types::GLenum = 0x00000008;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY4: types::GLenum = 0x00000010;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY5: types::GLenum = 0x00000020;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY6: types::GLenum = 0x00000040;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY7: types::GLenum = 0x00000080;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY8: types::GLenum = 0x00000100;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY9: types::GLenum = 0x00000200;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY1: types::GLenum = 0x00010000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY10: types::GLenum = 0x02000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY11: types::GLenum = 0x04000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY12: types::GLenum = 0x08000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY13: types::GLenum = 0x10000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY14: types::GLenum = 0x20000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY15: types::GLenum = 0x40000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY2: types::GLenum = 0x00020000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY3: types::GLenum = 0x00040000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY4: types::GLenum = 0x00080000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY5: types::GLenum = 0x00100000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY6: types::GLenum = 0x00200000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY7: types::GLenum = 0x00400000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY8: types::GLenum = 0x00800000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY9: types::GLenum = 0x01000000;

        #[allow(non_snake_case, unused_variables, dead_code)]
        extern "system" {
#[link_name="wglCopyContext"]
            pub fn CopyContext(hglrcSrc: types::HGLRC, hglrcDst: types::HGLRC, mask: types::UINT) -> types::BOOL;
#[link_name="wglCreateContext"]
            pub fn CreateContext(hDc: types::HDC) -> types::HGLRC;
#[link_name="wglCreateLayerContext"]
            pub fn CreateLayerContext(hDc: types::HDC, level: __gl_imports::raw::c_int) -> types::HGLRC;
#[link_name="wglDeleteContext"]
            pub fn DeleteContext(oldContext: types::HGLRC) -> types::BOOL;
#[link_name="wglDescribeLayerPlane"]
            pub fn DescribeLayerPlane(hDc: types::HDC, pixelFormat: __gl_imports::raw::c_int, layerPlane: __gl_imports::raw::c_int, nBytes: types::UINT, plpd: *const types::LAYERPLANEDESCRIPTOR) -> types::BOOL;
#[link_name="wglGetCurrentContext"]
            pub fn GetCurrentContext() -> types::HGLRC;
#[link_name="wglGetCurrentDC"]
            pub fn GetCurrentDC() -> types::HDC;
#[link_name="wglGetLayerPaletteEntries"]
            pub fn GetLayerPaletteEntries(hdc: types::HDC, iLayerPlane: __gl_imports::raw::c_int, iStart: __gl_imports::raw::c_int, cEntries: __gl_imports::raw::c_int, pcr: *const types::COLORREF) -> __gl_imports::raw::c_int;
#[link_name="wglGetProcAddress"]
            pub fn GetProcAddress(lpszProc: types::LPCSTR) -> types::PROC;
#[link_name="wglMakeCurrent"]
            pub fn MakeCurrent(hDc: types::HDC, newContext: types::HGLRC) -> types::BOOL;
#[link_name="wglRealizeLayerPalette"]
            pub fn RealizeLayerPalette(hdc: types::HDC, iLayerPlane: __gl_imports::raw::c_int, bRealize: types::BOOL) -> types::BOOL;
#[link_name="wglSetLayerPaletteEntries"]
            pub fn SetLayerPaletteEntries(hdc: types::HDC, iLayerPlane: __gl_imports::raw::c_int, iStart: __gl_imports::raw::c_int, cEntries: __gl_imports::raw::c_int, pcr: *const types::COLORREF) -> __gl_imports::raw::c_int;
#[link_name="wglShareLists"]
            pub fn ShareLists(hrcSrvShare: types::HGLRC, hrcSrvSource: types::HGLRC) -> types::BOOL;
#[link_name="wglSwapLayerBuffers"]
            pub fn SwapLayerBuffers(hdc: types::HDC, fuFlags: types::UINT) -> types::BOOL;
#[link_name="wglUseFontBitmaps"]
            pub fn UseFontBitmaps(hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD) -> types::BOOL;
#[link_name="wglUseFontBitmapsA"]
            pub fn UseFontBitmapsA(hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD) -> types::BOOL;
#[link_name="wglUseFontBitmapsW"]
            pub fn UseFontBitmapsW(hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD) -> types::BOOL;
#[link_name="wglUseFontOutlines"]
            pub fn UseFontOutlines(hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD, deviation: types::FLOAT, extrusion: types::FLOAT, format: __gl_imports::raw::c_int, lpgmf: types::LPGLYPHMETRICSFLOAT) -> types::BOOL;
#[link_name="wglUseFontOutlinesA"]
            pub fn UseFontOutlinesA(hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD, deviation: types::FLOAT, extrusion: types::FLOAT, format: __gl_imports::raw::c_int, lpgmf: types::LPGLYPHMETRICSFLOAT) -> types::BOOL;
#[link_name="wglUseFontOutlinesW"]
            pub fn UseFontOutlinesW(hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD, deviation: types::FLOAT, extrusion: types::FLOAT, format: __gl_imports::raw::c_int, lpgmf: types::LPGLYPHMETRICSFLOAT) -> types::BOOL;
}


# target\release\build\glutin_wgl_sys-d01013064eeb78ec\out\wgl_extra_bindings.rs

        mod __gl_imports {
            pub use std::mem;
            pub use std::marker::Send;
            pub use std::os::raw;
        }
    

        pub mod types {
            #![allow(non_camel_case_types, non_snake_case, dead_code, missing_copy_implementations)]
    
// Common types from OpenGL 1.1
pub type GLenum = super::__gl_imports::raw::c_uint;
pub type GLboolean = super::__gl_imports::raw::c_uchar;
pub type GLbitfield = super::__gl_imports::raw::c_uint;
pub type GLvoid = super::__gl_imports::raw::c_void;
pub type GLbyte = super::__gl_imports::raw::c_char;
pub type GLshort = super::__gl_imports::raw::c_short;
pub type GLint = super::__gl_imports::raw::c_int;
pub type GLclampx = super::__gl_imports::raw::c_int;
pub type GLubyte = super::__gl_imports::raw::c_uchar;
pub type GLushort = super::__gl_imports::raw::c_ushort;
pub type GLuint = super::__gl_imports::raw::c_uint;
pub type GLsizei = super::__gl_imports::raw::c_int;
pub type GLfloat = super::__gl_imports::raw::c_float;
pub type GLclampf = super::__gl_imports::raw::c_float;
pub type GLdouble = super::__gl_imports::raw::c_double;
pub type GLclampd = super::__gl_imports::raw::c_double;
pub type GLeglImageOES = *const super::__gl_imports::raw::c_void;
pub type GLchar = super::__gl_imports::raw::c_char;
pub type GLcharARB = super::__gl_imports::raw::c_char;

#[cfg(target_os = "macos")]
pub type GLhandleARB = *const super::__gl_imports::raw::c_void;
#[cfg(not(target_os = "macos"))]
pub type GLhandleARB = super::__gl_imports::raw::c_uint;

pub type GLhalfARB = super::__gl_imports::raw::c_ushort;
pub type GLhalf = super::__gl_imports::raw::c_ushort;

// Must be 32 bits
pub type GLfixed = GLint;

pub type GLintptr = isize;
pub type GLsizeiptr = isize;
pub type GLint64 = i64;
pub type GLuint64 = u64;
pub type GLintptrARB = isize;
pub type GLsizeiptrARB = isize;
pub type GLint64EXT = i64;
pub type GLuint64EXT = u64;

pub enum __GLsync {}
pub type GLsync = *const __GLsync;

// compatible with OpenCL cl_context
pub enum _cl_context {}
pub enum _cl_event {}

pub type GLDEBUGPROC = Option<extern "system" fn(source: GLenum,
                                                 gltype: GLenum,
                                                 id: GLuint,
                                                 severity: GLenum,
                                                 length: GLsizei,
                                                 message: *const GLchar,
                                                 userParam: *mut super::__gl_imports::raw::c_void)>;
pub type GLDEBUGPROCARB = Option<extern "system" fn(source: GLenum,
                                                    gltype: GLenum,
                                                    id: GLuint,
                                                    severity: GLenum,
                                                    length: GLsizei,
                                                    message: *const GLchar,
                                                    userParam: *mut super::__gl_imports::raw::c_void)>;
pub type GLDEBUGPROCKHR = Option<extern "system" fn(source: GLenum,
                                                    gltype: GLenum,
                                                    id: GLuint,
                                                    severity: GLenum,
                                                    length: GLsizei,
                                                    message: *const GLchar,
                                                    userParam: *mut super::__gl_imports::raw::c_void)>;

// GLES 1 types
// "pub type GLclampx = i32;",

// GLES 1/2 types (tagged for GLES 1)
// "pub type GLbyte = i8;",
// "pub type GLubyte = u8;",
// "pub type GLfloat = GLfloat;",
// "pub type GLclampf = GLfloat;",
// "pub type GLfixed = i32;",
// "pub type GLint64 = i64;",
// "pub type GLuint64 = u64;",
// "pub type GLintptr = intptr_t;",
// "pub type GLsizeiptr = ssize_t;",

// GLES 1/2 types (tagged for GLES 2 - attribute syntax is limited)
// "pub type GLbyte = i8;",
// "pub type GLubyte = u8;",
// "pub type GLfloat = GLfloat;",
// "pub type GLclampf = GLfloat;",
// "pub type GLfixed = i32;",
// "pub type GLint64 = i64;",
// "pub type GLuint64 = u64;",
// "pub type GLint64EXT = i64;",
// "pub type GLuint64EXT = u64;",
// "pub type GLintptr = intptr_t;",
// "pub type GLsizeiptr = ssize_t;",

// GLES 2 types (none currently)

// Vendor extension types
pub type GLDEBUGPROCAMD = Option<extern "system" fn(id: GLuint,
                                                    category: GLenum,
                                                    severity: GLenum,
                                                    length: GLsizei,
                                                    message: *const GLchar,
                                                    userParam: *mut super::__gl_imports::raw::c_void)>;
pub type GLhalfNV = super::__gl_imports::raw::c_ushort;
pub type GLvdpauSurfaceNV = GLintptr;

// From WinNT.h

pub type CHAR = super::__gl_imports::raw::c_char;
pub type HANDLE = PVOID;
pub type LONG = super::__gl_imports::raw::c_long;
pub type LPCSTR = *const super::__gl_imports::raw::c_char;
pub type VOID = ();
// #define DECLARE_HANDLE(name) struct name##__{int unused;}; typedef struct name##__ *name
pub type HPBUFFERARB = *const super::__gl_imports::raw::c_void;
pub type HPBUFFEREXT = *const super::__gl_imports::raw::c_void;
pub type HVIDEOOUTPUTDEVICENV = *const super::__gl_imports::raw::c_void;
pub type HPVIDEODEV = *const super::__gl_imports::raw::c_void;
pub type HPGPUNV = *const super::__gl_imports::raw::c_void;
pub type HGPUNV = *const super::__gl_imports::raw::c_void;
pub type HVIDEOINPUTDEVICENV = *const super::__gl_imports::raw::c_void;

// From Windef.h

pub type BOOL = super::__gl_imports::raw::c_int;
pub type BYTE = super::__gl_imports::raw::c_uchar;
pub type COLORREF = DWORD;
pub type FLOAT = super::__gl_imports::raw::c_float;
pub type HDC = HANDLE;
pub type HENHMETAFILE = HANDLE;
pub type HGLRC = *const super::__gl_imports::raw::c_void;
pub type INT = super::__gl_imports::raw::c_int;
pub type PVOID = *const super::__gl_imports::raw::c_void;
pub type LPVOID = *const super::__gl_imports::raw::c_void;
pub enum __PROC_fn {}
pub type PROC = *mut __PROC_fn;

#[repr(C)]
pub struct RECT {
    left: LONG,
    top: LONG,
    right: LONG,
    bottom: LONG,
}

pub type UINT = super::__gl_imports::raw::c_uint;
pub type USHORT = super::__gl_imports::raw::c_ushort;
pub type WORD = super::__gl_imports::raw::c_ushort;

// From BaseTsd.h

pub type INT32 = i32;
pub type INT64 = i64;

// From IntSafe.h

pub type DWORD = super::__gl_imports::raw::c_ulong;

// From Wingdi.h

#[repr(C)]
pub struct POINTFLOAT {
    pub x: FLOAT,
    pub y: FLOAT,
}

#[repr(C)]
pub struct GLYPHMETRICSFLOAT {
    pub gmfBlackBoxX: FLOAT,
    pub gmfBlackBoxY: FLOAT,
    pub gmfptGlyphOrigin: POINTFLOAT,
    pub gmfCellIncX: FLOAT,
    pub gmfCellIncY: FLOAT,
}
pub type LPGLYPHMETRICSFLOAT = *const GLYPHMETRICSFLOAT;

#[repr(C)]
pub struct LAYERPLANEDESCRIPTOR {
    pub nSize: WORD,
    pub nVersion: WORD,
    pub dwFlags: DWORD,
    pub iPixelType: BYTE,
    pub cColorBits: BYTE,
    pub cRedBits: BYTE,
    pub cRedShift: BYTE,
    pub cGreenBits: BYTE,
    pub cGreenShift: BYTE,
    pub cBlueBits: BYTE,
    pub cBlueShift: BYTE,
    pub cAlphaBits: BYTE,
    pub cAlphaShift: BYTE,
    pub cAccumBits: BYTE,
    pub cAccumRedBits: BYTE,
    pub cAccumGreenBits: BYTE,
    pub cAccumBlueBits: BYTE,
    pub cAccumAlphaBits: BYTE,
    pub cDepthBits: BYTE,
    pub cStencilBits: BYTE,
    pub cAuxBuffers: BYTE,
    pub iLayerType: BYTE,
    pub bReserved: BYTE,
    pub crTransparent: COLORREF,
}

#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
    pub nSize: WORD,
    pub nVersion: WORD,
    pub dwFlags: DWORD,
    pub iPixelType: BYTE,
    pub cColorBits: BYTE,
    pub cRedBits: BYTE,
    pub cRedShift: BYTE,
    pub cGreenBits: BYTE,
    pub cGreenShift: BYTE,
    pub cBlueBits: BYTE,
    pub cBlueShift: BYTE,
    pub cAlphaBits: BYTE,
    pub cAlphaShift: BYTE,
    pub cAccumBits: BYTE,
    pub cAccumRedBits: BYTE,
    pub cAccumGreenBits: BYTE,
    pub cAccumBlueBits: BYTE,
    pub cAccumAlphaBits: BYTE,
    pub cDepthBits: BYTE,
    pub cStencilBits: BYTE,
    pub cAuxBuffers: BYTE,
    pub iLayerType: BYTE,
    pub bReserved: BYTE,
    pub dwLayerMask: DWORD,
    pub dwVisibleMask: DWORD,
    pub dwDamageMask: DWORD,
}

#[repr(C)]
pub struct _GPU_DEVICE {
    cb: DWORD,
    DeviceName: [CHAR; 32],
    DeviceString: [CHAR; 128],
    Flags: DWORD,
    rcVirtualScreen: RECT,
}

pub struct GPU_DEVICE(_GPU_DEVICE);
pub struct PGPU_DEVICE(*const _GPU_DEVICE);

}
#[allow(dead_code, non_upper_case_globals)] pub const ACCELERATION_ARB: types::GLenum = 0x2003;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_ALPHA_BITS_ARB: types::GLenum = 0x2021;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_BITS_ARB: types::GLenum = 0x201D;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_BLUE_BITS_ARB: types::GLenum = 0x2020;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_GREEN_BITS_ARB: types::GLenum = 0x201F;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_RED_BITS_ARB: types::GLenum = 0x201E;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_BITS_ARB: types::GLenum = 0x201B;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_SHIFT_ARB: types::GLenum = 0x201C;
#[allow(dead_code, non_upper_case_globals)] pub const AUX_BUFFERS_ARB: types::GLenum = 0x2024;
#[allow(dead_code, non_upper_case_globals)] pub const BLUE_BITS_ARB: types::GLenum = 0x2019;
#[allow(dead_code, non_upper_case_globals)] pub const BLUE_SHIFT_ARB: types::GLenum = 0x201A;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_BITS_ARB: types::GLenum = 0x2014;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB: types::GLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_CORE_PROFILE_BIT_ARB: types::GLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_DEBUG_BIT_ARB: types::GLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_ES2_PROFILE_BIT_EXT: types::GLenum = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_FLAGS_ARB: types::GLenum = 0x2094;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: types::GLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_LAYER_PLANE_ARB: types::GLenum = 0x2093;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_MAJOR_VERSION_ARB: types::GLenum = 0x2091;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_MINOR_VERSION_ARB: types::GLenum = 0x2092;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_OPENGL_NO_ERROR_ARB: types::GLenum = 0x31B3;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_PROFILE_MASK_ARB: types::GLenum = 0x9126;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_RELEASE_BEHAVIOR_ARB: types::GLenum = 0x2097;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_RELEASE_BEHAVIOR_FLUSH_ARB: types::GLenum = 0x2098;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_RELEASE_BEHAVIOR_NONE_ARB: types::GLenum = 0;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_RESET_NOTIFICATION_STRATEGY_ARB: types::GLenum = 0x8256;
#[allow(dead_code, non_upper_case_globals)] pub const CONTEXT_ROBUST_ACCESS_BIT_ARB: types::GLenum = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const DEPTH_BITS_ARB: types::GLenum = 0x2022;
#[allow(dead_code, non_upper_case_globals)] pub const DOUBLE_BUFFER_ARB: types::GLenum = 0x2011;
#[allow(dead_code, non_upper_case_globals)] pub const DRAW_TO_BITMAP_ARB: types::GLenum = 0x2002;
#[allow(dead_code, non_upper_case_globals)] pub const DRAW_TO_WINDOW_ARB: types::GLenum = 0x2001;
#[allow(dead_code, non_upper_case_globals)] pub const FONT_LINES: types::GLenum = 0;
#[allow(dead_code, non_upper_case_globals)] pub const FONT_POLYGONS: types::GLenum = 1;
#[allow(dead_code, non_upper_case_globals)] pub const FRAMEBUFFER_SRGB_CAPABLE_ARB: types::GLenum = 0x20A9;
#[allow(dead_code, non_upper_case_globals)] pub const FRAMEBUFFER_SRGB_CAPABLE_EXT: types::GLenum = 0x20A9;
#[allow(dead_code, non_upper_case_globals)] pub const FULL_ACCELERATION_ARB: types::GLenum = 0x2027;
#[allow(dead_code, non_upper_case_globals)] pub const GENERIC_ACCELERATION_ARB: types::GLenum = 0x2026;
#[allow(dead_code, non_upper_case_globals)] pub const GREEN_BITS_ARB: types::GLenum = 0x2017;
#[allow(dead_code, non_upper_case_globals)] pub const GREEN_SHIFT_ARB: types::GLenum = 0x2018;
#[allow(dead_code, non_upper_case_globals)] pub const LOSE_CONTEXT_ON_RESET_ARB: types::GLenum = 0x8252;
#[allow(dead_code, non_upper_case_globals)] pub const NEED_PALETTE_ARB: types::GLenum = 0x2004;
#[allow(dead_code, non_upper_case_globals)] pub const NEED_SYSTEM_PALETTE_ARB: types::GLenum = 0x2005;
#[allow(dead_code, non_upper_case_globals)] pub const NO_ACCELERATION_ARB: types::GLenum = 0x2025;
#[allow(dead_code, non_upper_case_globals)] pub const NO_RESET_NOTIFICATION_ARB: types::GLenum = 0x8261;
#[allow(dead_code, non_upper_case_globals)] pub const NUMBER_OVERLAYS_ARB: types::GLenum = 0x2008;
#[allow(dead_code, non_upper_case_globals)] pub const NUMBER_PIXEL_FORMATS_ARB: types::GLenum = 0x2000;
#[allow(dead_code, non_upper_case_globals)] pub const NUMBER_UNDERLAYS_ARB: types::GLenum = 0x2009;
#[allow(dead_code, non_upper_case_globals)] pub const PIXEL_TYPE_ARB: types::GLenum = 0x2013;
#[allow(dead_code, non_upper_case_globals)] pub const RED_BITS_ARB: types::GLenum = 0x2015;
#[allow(dead_code, non_upper_case_globals)] pub const RED_SHIFT_ARB: types::GLenum = 0x2016;
#[allow(dead_code, non_upper_case_globals)] pub const SAMPLES_ARB: types::GLenum = 0x2042;
#[allow(dead_code, non_upper_case_globals)] pub const SAMPLE_BUFFERS_ARB: types::GLenum = 0x2041;
#[allow(dead_code, non_upper_case_globals)] pub const SHARE_ACCUM_ARB: types::GLenum = 0x200E;
#[allow(dead_code, non_upper_case_globals)] pub const SHARE_DEPTH_ARB: types::GLenum = 0x200C;
#[allow(dead_code, non_upper_case_globals)] pub const SHARE_STENCIL_ARB: types::GLenum = 0x200D;
#[allow(dead_code, non_upper_case_globals)] pub const STENCIL_BITS_ARB: types::GLenum = 0x2023;
#[allow(dead_code, non_upper_case_globals)] pub const STEREO_ARB: types::GLenum = 0x2012;
#[allow(dead_code, non_upper_case_globals)] pub const SUPPORT_GDI_ARB: types::GLenum = 0x200F;
#[allow(dead_code, non_upper_case_globals)] pub const SUPPORT_OPENGL_ARB: types::GLenum = 0x2010;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_COPY_ARB: types::GLenum = 0x2029;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_EXCHANGE_ARB: types::GLenum = 0x2028;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_LAYER_BUFFERS_ARB: types::GLenum = 0x2006;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_MAIN_PLANE: types::GLenum = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_METHOD_ARB: types::GLenum = 0x2007;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY1: types::GLenum = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY10: types::GLenum = 0x00000400;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY11: types::GLenum = 0x00000800;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY12: types::GLenum = 0x00001000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY13: types::GLenum = 0x00002000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY14: types::GLenum = 0x00004000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY15: types::GLenum = 0x00008000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY2: types::GLenum = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY3: types::GLenum = 0x00000008;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY4: types::GLenum = 0x00000010;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY5: types::GLenum = 0x00000020;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY6: types::GLenum = 0x00000040;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY7: types::GLenum = 0x00000080;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY8: types::GLenum = 0x00000100;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_OVERLAY9: types::GLenum = 0x00000200;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDEFINED_ARB: types::GLenum = 0x202A;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY1: types::GLenum = 0x00010000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY10: types::GLenum = 0x02000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY11: types::GLenum = 0x04000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY12: types::GLenum = 0x08000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY13: types::GLenum = 0x10000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY14: types::GLenum = 0x20000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY15: types::GLenum = 0x40000000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY2: types::GLenum = 0x00020000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY3: types::GLenum = 0x00040000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY4: types::GLenum = 0x00080000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY5: types::GLenum = 0x00100000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY6: types::GLenum = 0x00200000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY7: types::GLenum = 0x00400000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY8: types::GLenum = 0x00800000;
#[allow(dead_code, non_upper_case_globals)] pub const SWAP_UNDERLAY9: types::GLenum = 0x01000000;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_ALPHA_VALUE_ARB: types::GLenum = 0x203A;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_ARB: types::GLenum = 0x200A;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_BLUE_VALUE_ARB: types::GLenum = 0x2039;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_GREEN_VALUE_ARB: types::GLenum = 0x2038;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_INDEX_VALUE_ARB: types::GLenum = 0x203B;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_RED_VALUE_ARB: types::GLenum = 0x2037;
#[allow(dead_code, non_upper_case_globals)] pub const TYPE_COLORINDEX_ARB: types::GLenum = 0x202C;
#[allow(dead_code, non_upper_case_globals)] pub const TYPE_RGBA_ARB: types::GLenum = 0x202B;
#[allow(dead_code, non_upper_case_globals)] pub const TYPE_RGBA_FLOAT_ARB: types::GLenum = 0x21A0;

        #[allow(dead_code, missing_copy_implementations)]
        #[derive(Clone)]
        pub struct FnPtr {
            /// The function pointer that will be used when calling the function.
            f: *const __gl_imports::raw::c_void,
            /// True if the pointer points to a real function, false if points to a `panic!` fn.
            is_loaded: bool,
        }

        impl FnPtr {
            /// Creates a `FnPtr` from a load attempt.
            fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {
                if ptr.is_null() {
                    FnPtr {
                        f: missing_fn_panic as *const __gl_imports::raw::c_void,
                        is_loaded: false
                    }
                } else {
                    FnPtr { f: ptr, is_loaded: true }
                }
            }

            /// Returns `true` if the function has been successfully loaded.
            ///
            /// If it returns `false`, calling the corresponding function will fail.
            #[inline]
            #[allow(dead_code)]
            pub fn is_loaded(&self) -> bool {
                self.is_loaded
            }
        }
    
#[inline(never)]
        fn missing_fn_panic() -> ! {
            panic!("wgl function was not loaded")
        }

        #[allow(non_camel_case_types, non_snake_case, dead_code)]
        #[derive(Clone)]
        pub struct Wgl {
pub ChoosePixelFormatARB: FnPtr,
pub CopyContext: FnPtr,
pub CreateContext: FnPtr,
pub CreateContextAttribsARB: FnPtr,
pub CreateLayerContext: FnPtr,
pub DeleteContext: FnPtr,
pub DescribeLayerPlane: FnPtr,
pub GetCurrentContext: FnPtr,
pub GetCurrentDC: FnPtr,
pub GetExtensionsStringARB: FnPtr,
pub GetExtensionsStringEXT: FnPtr,
pub GetLayerPaletteEntries: FnPtr,
pub GetPixelFormatAttribfvARB: FnPtr,
pub GetPixelFormatAttribivARB: FnPtr,
pub GetProcAddress: FnPtr,
pub GetSwapIntervalEXT: FnPtr,
pub MakeCurrent: FnPtr,
pub RealizeLayerPalette: FnPtr,
pub SetLayerPaletteEntries: FnPtr,
pub ShareLists: FnPtr,
pub SwapIntervalEXT: FnPtr,
pub SwapLayerBuffers: FnPtr,
pub UseFontBitmaps: FnPtr,
pub UseFontBitmapsA: FnPtr,
pub UseFontBitmapsW: FnPtr,
pub UseFontOutlines: FnPtr,
pub UseFontOutlinesA: FnPtr,
pub UseFontOutlinesW: FnPtr,
_priv: ()
}
impl Wgl {
            /// Load each OpenGL symbol using a custom load function. This allows for the
            /// use of functions like `glfwGetProcAddress` or `SDL_GL_GetProcAddress`.
            ///
            /// ~~~ignore
            /// let gl = Gl::load_with(|s| glfw.get_proc_address(s));
            /// ~~~
            #[allow(dead_code, unused_variables)]
            pub fn load_with<F>(mut loadfn: F) -> Wgl where F: FnMut(&'static str) -> *const __gl_imports::raw::c_void {
                #[inline(never)]
                fn do_metaloadfn(loadfn: &mut dyn FnMut(&'static str) -> *const __gl_imports::raw::c_void,
                                 symbol: &'static str,
                                 symbols: &[&'static str])
                                 -> *const __gl_imports::raw::c_void {
                    let mut ptr = loadfn(symbol);
                    if ptr.is_null() {
                        for &sym in symbols {
                            ptr = loadfn(sym);
                            if !ptr.is_null() { break; }
                        }
                    }
                    ptr
                }
                let mut metaloadfn = |symbol: &'static str, symbols: &[&'static str]| {
                    do_metaloadfn(&mut loadfn, symbol, symbols)
                };
                Wgl {
ChoosePixelFormatARB: FnPtr::new(metaloadfn("wglChoosePixelFormatARB", &[])),
CopyContext: FnPtr::new(metaloadfn("wglCopyContext", &[])),
CreateContext: FnPtr::new(metaloadfn("wglCreateContext", &[])),
CreateContextAttribsARB: FnPtr::new(metaloadfn("wglCreateContextAttribsARB", &[])),
CreateLayerContext: FnPtr::new(metaloadfn("wglCreateLayerContext", &[])),
DeleteContext: FnPtr::new(metaloadfn("wglDeleteContext", &[])),
DescribeLayerPlane: FnPtr::new(metaloadfn("wglDescribeLayerPlane", &[])),
GetCurrentContext: FnPtr::new(metaloadfn("wglGetCurrentContext", &[])),
GetCurrentDC: FnPtr::new(metaloadfn("wglGetCurrentDC", &[])),
GetExtensionsStringARB: FnPtr::new(metaloadfn("wglGetExtensionsStringARB", &[])),
GetExtensionsStringEXT: FnPtr::new(metaloadfn("wglGetExtensionsStringEXT", &[])),
GetLayerPaletteEntries: FnPtr::new(metaloadfn("wglGetLayerPaletteEntries", &[])),
GetPixelFormatAttribfvARB: FnPtr::new(metaloadfn("wglGetPixelFormatAttribfvARB", &[])),
GetPixelFormatAttribivARB: FnPtr::new(metaloadfn("wglGetPixelFormatAttribivARB", &[])),
GetProcAddress: FnPtr::new(metaloadfn("wglGetProcAddress", &[])),
GetSwapIntervalEXT: FnPtr::new(metaloadfn("wglGetSwapIntervalEXT", &[])),
MakeCurrent: FnPtr::new(metaloadfn("wglMakeCurrent", &[])),
RealizeLayerPalette: FnPtr::new(metaloadfn("wglRealizeLayerPalette", &[])),
SetLayerPaletteEntries: FnPtr::new(metaloadfn("wglSetLayerPaletteEntries", &[])),
ShareLists: FnPtr::new(metaloadfn("wglShareLists", &[])),
SwapIntervalEXT: FnPtr::new(metaloadfn("wglSwapIntervalEXT", &[])),
SwapLayerBuffers: FnPtr::new(metaloadfn("wglSwapLayerBuffers", &[])),
UseFontBitmaps: FnPtr::new(metaloadfn("wglUseFontBitmaps", &[])),
UseFontBitmapsA: FnPtr::new(metaloadfn("wglUseFontBitmapsA", &[])),
UseFontBitmapsW: FnPtr::new(metaloadfn("wglUseFontBitmapsW", &[])),
UseFontOutlines: FnPtr::new(metaloadfn("wglUseFontOutlines", &[])),
UseFontOutlinesA: FnPtr::new(metaloadfn("wglUseFontOutlinesA", &[])),
UseFontOutlinesW: FnPtr::new(metaloadfn("wglUseFontOutlinesW", &[])),
_priv: ()
}
        }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ChoosePixelFormatARB(&self, hdc: types::HDC, piAttribIList: *const __gl_imports::raw::c_int, pfAttribFList: *const types::FLOAT, nMaxFormats: types::UINT, piFormats: *mut __gl_imports::raw::c_int, nNumFormats: *mut types::UINT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, *const __gl_imports::raw::c_int, *const types::FLOAT, types::UINT, *mut __gl_imports::raw::c_int, *mut types::UINT) -> types::BOOL>(self.ChoosePixelFormatARB.f)(hdc, piAttribIList, pfAttribFList, nMaxFormats, piFormats, nNumFormats) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CopyContext(&self, hglrcSrc: types::HGLRC, hglrcDst: types::HGLRC, mask: types::UINT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HGLRC, types::HGLRC, types::UINT) -> types::BOOL>(self.CopyContext.f)(hglrcSrc, hglrcDst, mask) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateContext(&self, hDc: types::HDC) -> types::HGLRC { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC) -> types::HGLRC>(self.CreateContext.f)(hDc) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateContextAttribsARB(&self, hDC: types::HDC, hShareContext: types::HGLRC, attribList: *const __gl_imports::raw::c_int) -> types::HGLRC { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::HGLRC, *const __gl_imports::raw::c_int) -> types::HGLRC>(self.CreateContextAttribsARB.f)(hDC, hShareContext, attribList) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn CreateLayerContext(&self, hDc: types::HDC, level: __gl_imports::raw::c_int) -> types::HGLRC { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int) -> types::HGLRC>(self.CreateLayerContext.f)(hDc, level) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DeleteContext(&self, oldContext: types::HGLRC) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HGLRC) -> types::BOOL>(self.DeleteContext.f)(oldContext) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn DescribeLayerPlane(&self, hDc: types::HDC, pixelFormat: __gl_imports::raw::c_int, layerPlane: __gl_imports::raw::c_int, nBytes: types::UINT, plpd: *const types::LAYERPLANEDESCRIPTOR) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int, __gl_imports::raw::c_int, types::UINT, *const types::LAYERPLANEDESCRIPTOR) -> types::BOOL>(self.DescribeLayerPlane.f)(hDc, pixelFormat, layerPlane, nBytes, plpd) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetCurrentContext(&self, ) -> types::HGLRC { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::HGLRC>(self.GetCurrentContext.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetCurrentDC(&self, ) -> types::HDC { __gl_imports::mem::transmute::<_, extern "system" fn() -> types::HDC>(self.GetCurrentDC.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetExtensionsStringARB(&self, hdc: types::HDC) -> *const __gl_imports::raw::c_char { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC) -> *const __gl_imports::raw::c_char>(self.GetExtensionsStringARB.f)(hdc) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetExtensionsStringEXT(&self, ) -> *const __gl_imports::raw::c_char { __gl_imports::mem::transmute::<_, extern "system" fn() -> *const __gl_imports::raw::c_char>(self.GetExtensionsStringEXT.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetLayerPaletteEntries(&self, hdc: types::HDC, iLayerPlane: __gl_imports::raw::c_int, iStart: __gl_imports::raw::c_int, cEntries: __gl_imports::raw::c_int, pcr: *const types::COLORREF) -> __gl_imports::raw::c_int { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int, __gl_imports::raw::c_int, __gl_imports::raw::c_int, *const types::COLORREF) -> __gl_imports::raw::c_int>(self.GetLayerPaletteEntries.f)(hdc, iLayerPlane, iStart, cEntries, pcr) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetPixelFormatAttribfvARB(&self, hdc: types::HDC, iPixelFormat: __gl_imports::raw::c_int, iLayerPlane: __gl_imports::raw::c_int, nAttributes: types::UINT, piAttributes: *const __gl_imports::raw::c_int, pfValues: *mut types::FLOAT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int, __gl_imports::raw::c_int, types::UINT, *const __gl_imports::raw::c_int, *mut types::FLOAT) -> types::BOOL>(self.GetPixelFormatAttribfvARB.f)(hdc, iPixelFormat, iLayerPlane, nAttributes, piAttributes, pfValues) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetPixelFormatAttribivARB(&self, hdc: types::HDC, iPixelFormat: __gl_imports::raw::c_int, iLayerPlane: __gl_imports::raw::c_int, nAttributes: types::UINT, piAttributes: *const __gl_imports::raw::c_int, piValues: *mut __gl_imports::raw::c_int) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int, __gl_imports::raw::c_int, types::UINT, *const __gl_imports::raw::c_int, *mut __gl_imports::raw::c_int) -> types::BOOL>(self.GetPixelFormatAttribivARB.f)(hdc, iPixelFormat, iLayerPlane, nAttributes, piAttributes, piValues) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetProcAddress(&self, lpszProc: types::LPCSTR) -> types::PROC { __gl_imports::mem::transmute::<_, extern "system" fn(types::LPCSTR) -> types::PROC>(self.GetProcAddress.f)(lpszProc) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn GetSwapIntervalEXT(&self, ) -> __gl_imports::raw::c_int { __gl_imports::mem::transmute::<_, extern "system" fn() -> __gl_imports::raw::c_int>(self.GetSwapIntervalEXT.f)() }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn MakeCurrent(&self, hDc: types::HDC, newContext: types::HGLRC) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::HGLRC) -> types::BOOL>(self.MakeCurrent.f)(hDc, newContext) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn RealizeLayerPalette(&self, hdc: types::HDC, iLayerPlane: __gl_imports::raw::c_int, bRealize: types::BOOL) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int, types::BOOL) -> types::BOOL>(self.RealizeLayerPalette.f)(hdc, iLayerPlane, bRealize) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SetLayerPaletteEntries(&self, hdc: types::HDC, iLayerPlane: __gl_imports::raw::c_int, iStart: __gl_imports::raw::c_int, cEntries: __gl_imports::raw::c_int, pcr: *const types::COLORREF) -> __gl_imports::raw::c_int { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, __gl_imports::raw::c_int, __gl_imports::raw::c_int, __gl_imports::raw::c_int, *const types::COLORREF) -> __gl_imports::raw::c_int>(self.SetLayerPaletteEntries.f)(hdc, iLayerPlane, iStart, cEntries, pcr) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn ShareLists(&self, hrcSrvShare: types::HGLRC, hrcSrvSource: types::HGLRC) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HGLRC, types::HGLRC) -> types::BOOL>(self.ShareLists.f)(hrcSrvShare, hrcSrvSource) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SwapIntervalEXT(&self, interval: __gl_imports::raw::c_int) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(__gl_imports::raw::c_int) -> types::BOOL>(self.SwapIntervalEXT.f)(interval) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn SwapLayerBuffers(&self, hdc: types::HDC, fuFlags: types::UINT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::UINT) -> types::BOOL>(self.SwapLayerBuffers.f)(hdc, fuFlags) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn UseFontBitmaps(&self, hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::DWORD, types::DWORD, types::DWORD) -> types::BOOL>(self.UseFontBitmaps.f)(hDC, first, count, listBase) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn UseFontBitmapsA(&self, hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::DWORD, types::DWORD, types::DWORD) -> types::BOOL>(self.UseFontBitmapsA.f)(hDC, first, count, listBase) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn UseFontBitmapsW(&self, hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::DWORD, types::DWORD, types::DWORD) -> types::BOOL>(self.UseFontBitmapsW.f)(hDC, first, count, listBase) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn UseFontOutlines(&self, hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD, deviation: types::FLOAT, extrusion: types::FLOAT, format: __gl_imports::raw::c_int, lpgmf: types::LPGLYPHMETRICSFLOAT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::DWORD, types::DWORD, types::DWORD, types::FLOAT, types::FLOAT, __gl_imports::raw::c_int, types::LPGLYPHMETRICSFLOAT) -> types::BOOL>(self.UseFontOutlines.f)(hDC, first, count, listBase, deviation, extrusion, format, lpgmf) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn UseFontOutlinesA(&self, hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD, deviation: types::FLOAT, extrusion: types::FLOAT, format: __gl_imports::raw::c_int, lpgmf: types::LPGLYPHMETRICSFLOAT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::DWORD, types::DWORD, types::DWORD, types::FLOAT, types::FLOAT, __gl_imports::raw::c_int, types::LPGLYPHMETRICSFLOAT) -> types::BOOL>(self.UseFontOutlinesA.f)(hDC, first, count, listBase, deviation, extrusion, format, lpgmf) }
#[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn UseFontOutlinesW(&self, hDC: types::HDC, first: types::DWORD, count: types::DWORD, listBase: types::DWORD, deviation: types::FLOAT, extrusion: types::FLOAT, format: __gl_imports::raw::c_int, lpgmf: types::LPGLYPHMETRICSFLOAT) -> types::BOOL { __gl_imports::mem::transmute::<_, extern "system" fn(types::HDC, types::DWORD, types::DWORD, types::DWORD, types::FLOAT, types::FLOAT, __gl_imports::raw::c_int, types::LPGLYPHMETRICSFLOAT) -> types::BOOL>(self.UseFontOutlinesW.f)(hDC, first, count, listBase, deviation, extrusion, format, lpgmf) }
}

        unsafe impl __gl_imports::Send for Wgl {}


# target\release\build\khronos_api-bb92728082603f6c\out\webgl_exts.rs
&[
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\ANGLE_instanced_arrays\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_blend_minmax\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_color_buffer_float\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_color_buffer_half_float\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_disjoint_timer_query\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_disjoint_timer_query_webgl2\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_float_blend\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_frag_depth\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_shader_texture_lod\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_sRGB\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_texture_compression_bptc\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_texture_compression_rgtc\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\EXT_texture_filter_anisotropic\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\KHR_parallel_shader_compile\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_element_index_uint\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_fbo_render_mipmap\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_standard_derivatives\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_texture_float\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_texture_float_linear\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_texture_half_float\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_texture_half_float_linear\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\OES_vertex_array_object\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_color_buffer_float\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_compressed_texture_astc\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_compressed_texture_etc\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_compressed_texture_etc1\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_compressed_texture_pvrtc\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_compressed_texture_s3tc\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_compressed_texture_s3tc_srgb\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_debug_renderer_info\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_debug_shaders\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_depth_texture\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_draw_buffers\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_lose_context\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_multiview\\extension.xml"),
&*include_bytes!("C:\\Users\\KDFX Modes\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\khronos_api-3.1.0\\api_webgl/extensions\\WEBGL_security_sensitive_resources\\extension.xml"),
]


# target\release\build\serde-84773184d0f76abd\out\private.rs
#[doc(hidden)]
pub mod __private228 {
    #[doc(hidden)]
    pub use crate::private::*;
}
use serde_core::__private228 as serde_core_private;


# target\release\build\serde-d1ac855126029dc0\out\private.rs
#[doc(hidden)]
pub mod __private228 {
    #[doc(hidden)]
    pub use crate::private::*;
}
use serde_core::__private228 as serde_core_private;


# target\release\build\serde_core-a4b11a02c3568caa\out\private.rs
#[doc(hidden)]
pub mod __private228 {
    #[doc(hidden)]
    pub use crate::private::*;
}


# target\release\build\serde_core-dfdcedfdba429326\out\private.rs
#[doc(hidden)]
pub mod __private228 {
    #[doc(hidden)]
    pub use crate::private::*;
}


# temp_excerpt.txt
        egui::ScrollArea::vertical()
            .id_source("markov_stationary_distribution")
            .max_height(260.0)
            .show(ui, |ui| {
                egui::ScrollArea::horizontal()
                    .id_source("markov_state_summary")
                    .show(ui, |ui| {
                        egui::Grid::new("markov_state_summary_grid")
                            .striped(true)
                            .min_col_width(360.0)
                            .show(ui, |ui| {
                                ui.label(self.tr("Состояние", "State"));
                                ui.label("π");
                                ui.end_row();
                                let rows = chain.state_count().min(32);
                                for idx in 0..rows {
                                    ui.add(
                                        egui::Label::new(Self::format_markov_state_summary(
                                            &chain.states[idx],
                                            5,
                                        ))
                                        .wrap(),
                                    );
                                    ui.label(format!("{:.6}", stationary[idx]));
                                    ui.end_row();
                                }
                                if chain.state_count() > rows {
                                    ui.label(format!("... {} ...", chain.state_count() - rows));
                                    ui.label("");
                                    ui.end_row();
                                }
                            });
                    });
            });
    }



# temp_insert14.py
﻿from pathlib import Path
path = Path('src/ui/app/table_view.rs')
text = path.read_text(encoding='utf-8')
needle = '                        });\n                        p = p.min(max_place_idx);'
if needle not in text:
    raise SystemExit('needle missing stop place block')
replacement = '                        });\n                        corrected_inputs |= sanitize_usize(&mut p, 0, max_place_idx);\n                        corrected_inputs |= sanitize_u64(&mut n, 1, 1_000_000);\n                        p = p.min(max_place_idx);'
text = text.replace(needle, replacement, 1)
path.write_text(text, encoding='utf-8')


# temp_insert5.py
﻿from pathlib import Path
path = Path('src/ui/app.rs')
text = path.read_text(encoding='utf-8')
needle = '                self.table_fullscreen = !self.table_fullscreen;\n            }\n        });'
if needle not in text:
    raise SystemExit('needle missing')
replacement = '                self.table_fullscreen = !self.table_fullscreen;\n            }\n            if ui.button("Марковская модель").clicked() {\n                self.calculate_markov_model();\n                self.show_markov_window = true;\n            }\n        });'
text = text.replace(needle, replacement, 1)
path.write_text(text, encoding='utf-8')


# temp_prev.txt
﻿use super::*;
use egui::{Color32, Vec2};

impl PetriApp {
    pub(in crate::ui::app) fn draw_markov_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_markov_window;
        let viewport = ctx.available_rect();
        let max_height = (viewport.height() - 120.0).max(360.0);
        let max_width = (viewport.width() - 120.0).max(360.0);
        egui::Window::new(self.tr("Марковская модель", "Markov model"))
            .constrained_to_viewport(ctx)
            .id(egui::Id::new("markov_window"))
            .default_size(Vec2::new(520.0, 520.0))
            .min_size(Vec2::new(360.0, 360.0))
            .max_size(Vec2::new(max_width, max_height))
            .open(&mut open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            let simulation_ready = self.sim_result.is_some();
                            let mut toggle_changed = false;
                            let markov_checkbox_label =
                                self.tr("включить марковскую модель", "Enable Markov model");
                            let simulation_hint = self.tr(
                                "Сначала запустите симуляцию, чтобы включить марковскую модель",
                                "Run a simulation first to enable the model",
                            );
                            ui.horizontal(|ui| {
                                ui.add_enabled_ui(simulation_ready, |ui| {
                                    if ui
                                        .checkbox(
                                            &mut self.markov_model_enabled,
                                            markov_checkbox_label.as_ref(),
                                        )
                                        .changed()
                                    {
                                        toggle_changed = true;
                                    }
                                });
                                if !simulation_ready {
                                    ui.colored_label(
                                        Color32::from_rgb(190, 40, 40),
                                        simulation_hint.as_ref(),
                                    );
                                }
                            });
                            if toggle_changed {
                                for place in &mut self.net.places {
                                    place.show_markov_model = self.markov_model_enabled;
                                }
                                if self.markov_model_enabled {
                                    self.calculate_markov_model();
                                } else {
                                    self.markov_place_arcs.clear();
                                }
                            }
                            ui.separator();
                            ui.add_space(6.0);
                            if self.markov_model_enabled {
                                if let Some(chain) = &self.markov_model {
                                    let stationary = chain.stationary.as_ref();
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{}: {}{}",
                                            self.tr("Состояний", "States"),
                                            chain.state_count(),
                                            if chain.limit_reached {
                                                format!(" ({})", self.tr("лимит", "limit reached"))
                                            } else {
                                                String::new()
                                            }
                                        ));
                                        ui.label(format!(
                                            "{}: {}",
                                            self.tr("Переходов", "Transitions"),
                                            chain
                                                .transitions
                                                .iter()
                                                .map(|edges| edges.len())
                                                .sum::<usize>()
                                        ));
                                    });
                                    ui.separator();
                                    ui.label(
                                        self.tr(
                                            "Стационарное распределение",
                                            "Stationary distribution",
                                        ),
                                    );
                                    egui::ScrollArea::vertical()
                                        .id_source("markov_stationary_distribution")
                                        .max_height(260.0)
                                        .show(ui, |ui| {
                                            if let Some(stationary) = stationary {
                                                egui::Grid::new("markov_states")
                                                    .striped(true)
                                                    .show(ui, |ui| {
                                                        ui.label(self.tr("Состояние", "State"));
                                                        ui.label("π");
                                                        ui.end_row();
                                                        let rows = chain.state_count().min(32);
                                                        for idx in 0..rows {
                                                            ui.label(Self::format_marking(
                                                                &chain.states[idx],
                                                            ));
                                                            ui.label(format!(
                                                                "{:.6}",
                                                                stationary[idx]
                                                            ));
                                                            ui.end_row();
                                                        }
                                                        if chain.state_count() > rows {
                                                            ui.label(format!(
                                                                "... {} ...",
                                                                chain.state_count() - rows
                                                            ));
                                                            ui.label("");
                                                            ui.end_row();
                                                        }
                                                    });
                                            } else {
                                                ui.label(self.tr(
                                                    "Стационарное распределение не вычислено",
                                                    "Unable to compute stationary",
                                                ));
                                            }
                                        });
                                    ui.separator();
                                    ui.label(self.tr("Граф состояний", "State graph"));
                                    egui::ScrollArea::vertical()
                                        .id_source("markov_state_graph")
                                        .max_height(320.0)
                                        .show(ui, |ui| {
                                            let graph_width = ui.available_width().min(520.0);
                                            let has_transitions =
                                                chain.transitions.iter().any(|edges| !edges.is_empty());
                                            if has_transitions {
                                                egui::Grid::new("markov_state_graph_grid")
                                                    .striped(true)
                                                    .min_col_width(graph_width)
                                                    .show(ui, |ui| {
                                                        ui.label(self.tr("Состояние", "State"));
                                                        ui.label(self.tr("Переходы", "Transitions"));
                                                        ui.end_row();
                                                        for (idx, edges) in
                                                            chain.transitions.iter().enumerate()
                                                        {
                                                            ui.label(format!("S{}", idx + 1));
                                                            if edges.is_empty() {
                                                                ui.label(
                                                                    self.tr(
                                                                        "Переходов нет",
                                                                        "No transitions",
                                                                    ),
                                                                );
                                                            } else {
                                                                let total_rate: f64 = edges
                                                                    .iter()
                                                                    .map(|(_, rate)| *rate)
                                                                    .sum();
                                                                ui.vertical(|ui| {
                                                                    for (dest, rate) in edges {
                                                                        let prob = if total_rate > 0.0
                                                                        {
                                                                            (rate / total_rate)
                                                                                .clamp(0.0, 1.0)
                                                                        } else {
                                                                            0.0
                                                                        };
                                                                        ui.label(format!(
                                                                            "→ S{} ({:.2})",
                                                                            dest + 1,
                                                                            prob
                                                                        ));
                                                                    }
                                                                });
                                                            }
                                                            ui.end_row();
                                                        }
                                                    });
                                            } else {
                                                ui.label(self.tr(
                                                    "Переходов не найдено",
                                                    "No transitions detected",
                                                ));
                                            }
                                        });
                                    let markov_highlight_places = self
                                        .net
                                        .places
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, place)| place.markov_highlight)
                                        .collect::<Vec<_>>();
                                    if markov_highlight_places.is_empty() {
                                        ui.separator();
                                        ui.label(self.tr(
                                            "Отметьте марковскую метку в свойствах позиции, чтобы увидеть её отображение",
                                            "Enable the Markov highlight on a place to view its display",
                                        ));
                                    } else {
                                        ui.separator();
                                        ui.label(
                                            self.tr(
                                                "Отображение марковской метки",
                                                "Markov highlight display",
                                            ),
                                        );
                                        let expectation =
                                            Self::markov_expected_tokens(chain, self.net.places.len());
                                        egui::ScrollArea::vertical()
                                            .id_source("markov_place_distribution")
                                            .max_height(320.0)
                                            .show(ui, |ui| {
                                                for (place_idx, place) in
                                                    &markov_highlight_places
                                                {
                                                    ui.group(|ui| {
                                                        let place_label = if place.name.is_empty() {
                                                            format!("P{}", place.id)
                                                        } else {
                                                            place.name.clone()
                                                        };
                                                        ui.label(format!(
                                                            "{}: {} (P{})",
                                                            self.tr("РџРѕР·РёС†РёСЏ", "Place"),
                                                            place_label,
                                                            place.id
                                                        ));
                                                        if let Some(expected) = expectation
                                                            .as_ref()
                                                            .and_then(|values| {
                                                                values.get(*place_idx)
                                                            })
                                                        {
                                                            ui.label(format!(
                                                                "{}: {:.3}",
                                                                self.tr(
                                                                    "Ожидаемое число маркеров",
                                                                    "Expected tokens"
                                                                ),
                                                                expected
                                                            ));
                                                        }
                                                        let distribution = Self::markov_tokens_distribution(
                                                            chain, *place_idx,
                                                        );
                                                        if !distribution.is_empty() {
                                                            for (count, prob) in distribution.iter() {
                                                                ui.horizontal(|ui| {
                                                                    ui.label(format!(
                                                                        "{} {}",
                                                                        count,
                                                                        self.tr("маркеров", "tokens")
                                                                    ));
                                                                    ui.label(format!(
                                                                        "{:.2}%",
                                                                        prob * 100.0
                                                                    ));
                                                                });
                                                            }
                                                        } else if stationary.is_some() {
                                                            ui.label(self.tr(
                                                                "Для этой позиции состояния не найдены",
                                                                "No states found for this place",
                                                            ));
                                                        } else {
                                                            ui.label(self.tr(
                                                                "Стационарное распределение недоступно",
                                                                "Stationary distribution unavailable",
                                                            ));
                                                        }
                                                    });
                                                    ui.add_space(4.0);
                                                }
                                            });
                                    }
                                } else {
                                    ui.label(self.tr("Постройте модель", "Build the model"));
                                }
                            } else {
                                ui.label(self.tr(
                                    "Включите флажок выше, чтобы увидеть марковскую модель",
                                    "Toggle the checkbox above to display the Markov model",
                                ));
                            }
                        });
                    });
            });
        self.show_markov_window = open;
    }
}


# tests\encoding_guard.rs
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


# tests\export_runtime_probe.rs
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


# tests\gpn2_probe.rs
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


# tests\legacy_import.rs
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
    let mut fixture_candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir("fixtures/legacy") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("gpn") {
                fixture_candidates.push(path);
            }
        }
    }
    if let Some(path) = fixture_candidates
        .into_iter()
        .max_by_key(|path| std::fs::metadata(path).map(|m| m.len()).unwrap_or(0))
    {
        return path;
    }

    let mut root_candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("gpn") {
                root_candidates.push(path);
            }
        }
    }
    root_candidates
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
    let set3 = Path::new("Сеть 3.gpn");
    if !set3.exists() {
        return;
    }
    let path = set3.to_path_buf();
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
    let set3 = Path::new("Сеть 3.gpn");
    if !set3.exists() {
        return;
    }
    let path = set3.to_path_buf();
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
        let has_connected_duplicate = net.places.iter().enumerate().any(|(other_idx, other)| {
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
        let has_connected_duplicate =
            net.transitions
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
            i32::from_le_bytes([
                bytes[t0 + 24],
                bytes[t0 + 25],
                bytes[t0 + 26],
                bytes[t0 + 27]
            ]),
            196607
        );
        assert_eq!(
            i32::from_le_bytes([
                bytes[t0 + 28],
                bytes[t0 + 29],
                bytes[t0 + 30],
                bytes[t0 + 31]
            ]),
            -655360
        );
        assert_eq!(
            i32::from_le_bytes([
                bytes[t0 + 32],
                bytes[t0 + 33],
                bytes[t0 + 34],
                bytes[t0 + 35]
            ]),
            196607
        );
        assert_eq!(
            i32::from_le_bytes([
                bytes[t0 + 36],
                bytes[t0 + 37],
                bytes[t0 + 38],
                bytes[t0 + 39]
            ]),
            655360
        );
        assert_eq!(
            i32::from_le_bytes([
                bytes[t0 + 40],
                bytes[t0 + 41],
                bytes[t0 + 42],
                bytes[t0 + 43]
            ]),
            -131072
        );
        assert_eq!(
            i32::from_le_bytes([
                bytes[t0 + 44],
                bytes[t0 + 45],
                bytes[t0 + 46],
                bytes[t0 + 47]
            ]),
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
    assert_eq!(
        footer.len(),
        52,
        "legacy footer must match NetStar-compatible size"
    );
    assert_eq!(&footer[0..4], &[0xE8, 0x03, 0x00, 0x00]);
    assert_eq!(&footer[16..20], &[0xE8, 0x03, 0x00, 0x00]);
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
fn legacy_export_with_hints_writes_native_arc_section_for_set3() {
    let path = legacy_fixture_path();
    let imported = import_legacy_gpn(Path::new(&path)).expect("legacy import must succeed");
    let places = imported.model.places.len();
    let transitions = imported.model.transitions.len();

    let fake_tail = vec![0xAAu8; 512];
    let hints = LegacyExportHints {
        places_count: Some(places),
        transitions_count: Some(transitions),
        arc_topology_fingerprint: Some(arc_topology_fingerprint(&imported.model)),
        arc_header_extra: Some(123),
        footer_bytes: Some(fake_tail.clone()),
        raw_arc_and_tail: Some(fake_tail),
    };

    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("set3_hinted_native.gpn");
    export_legacy_gpn_with_hints(&out, &imported.model, Some(&hints))
        .expect("legacy export with hints must succeed");
    let out_bytes = std::fs::read(&out).expect("read saved file");

    let out_arcs_off = 16usize + places * 231 + transitions * 105;
    assert!(
        out_arcs_off + 6 <= out_bytes.len(),
        "output arcs section must exist"
    );
    let out_arc_extra =
        u16::from_le_bytes([out_bytes[out_arcs_off + 4], out_bytes[out_arcs_off + 5]]);
    assert_eq!(
        out_arc_extra, 99,
        "native exporter uses canonical arc header extra (not hints)"
    );

    let out_arc_max = i32::from_le_bytes([
        out_bytes[out_arcs_off],
        out_bytes[out_arcs_off + 1],
        out_bytes[out_arcs_off + 2],
        out_bytes[out_arcs_off + 3],
    ]);
    let out_arc_count = (out_arc_max + 1).max(0) as usize;
    let out_section_end = out_arcs_off + 6 + out_arc_count * 46;
    assert!(
        out_section_end <= out_bytes.len(),
        "output arc section must fit file"
    );
    let footer = &out_bytes[out_section_end..];
    assert_eq!(
        footer.len(),
        52,
        "native exporter must write canonical footer"
    );
    assert_eq!(&footer[0..4], &[0xE8, 0x03, 0x00, 0x00]);
    assert_eq!(&footer[16..20], &[0xE8, 0x03, 0x00, 0x00]);

    let reloaded = import_legacy_gpn(&out).expect("reimport must succeed");
    assert_eq!(reloaded.model.arcs.len(), imported.model.arcs.len());
    assert_eq!(
        reloaded.model.inhibitor_arcs.len(),
        imported.model.inhibitor_arcs.len()
    );
}


# tmp_print.py
﻿# -*- coding: utf-8 -*-
text = 'Выбранная позиция'
print(text.encode('utf-8'))
try:
    print(text.encode('utf-8').decode('cp1251'))
except Exception as e:
    print('error', e)


