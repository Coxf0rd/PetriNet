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

# Build versioned exe and also produce a stable name for release upload scripts.
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
$stableExe = Join-Path $ProjectDir "PetriNet.exe"
if (-not (Test-Path $versionedExe)) {
    throw "Versioned executable not found: $versionedExe"
}
Copy-Item $versionedExe $stableExe -Force
Write-Host "Executable ready: $stableExe"

