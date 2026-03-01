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
