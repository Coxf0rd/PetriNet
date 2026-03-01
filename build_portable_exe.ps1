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
# This avoids accumulating multiple PetriNet-<version>.exe files over time.
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

if (-not $KeepTarget) {
    $targetDir = Join-Path $ProjectDir "target"
    if (Test-Path $targetDir) {
        Remove-Item $targetDir -Recurse -Force
        Write-Host "Target cleaned: $targetDir"
    }
}
