param(
    [string]$ProjectDir = $PSScriptRoot,
    [string]$OutputExe = "",
    [switch]$KeepTarget
)


$ErrorActionPreference = "Stop"
Set-Location $ProjectDir

$logDir = Join-Path $ProjectDir "build_logs"
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$logPath = Join-Path $logDir "build_portable_exe-$timestamp.log"
$cargoStdout = $null
$cargoStderr = $null
$cargoExitCode = $null
$buildFailed = $false

function Write-Utf8NoBomFile {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Text
    )
    $enc = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($Path, $Text, $enc)
}

try {
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

        $tempBase = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "petri_build_$timestamp")
        $stdoutPath = "$tempBase.stdout.txt"
        $stderrPath = "$tempBase.stderr.txt"

        $p = Start-Process -FilePath $cargoPath -ArgumentList @("build", "--release") -NoNewWindow -Wait -PassThru `
            -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath

        $cargoExitCode = $p.ExitCode
        if (Test-Path $stdoutPath) {
            $cargoStdout = Get-Content -LiteralPath $stdoutPath -Raw -ErrorAction SilentlyContinue
            if ($cargoStdout) { Write-Host $cargoStdout }
        }
        if (Test-Path $stderrPath) {
            $cargoStderr = Get-Content -LiteralPath $stderrPath -Raw -ErrorAction SilentlyContinue
            if ($cargoStderr) { Write-Host $cargoStderr }
        }

        if ($cargoExitCode -ne 0) {
            throw "Cargo build failed with exit code $cargoExitCode"
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
    } finally {
        $env:RUSTFLAGS = $previousRustFlags
    }
} catch {
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir | Out-Null
    }

    $errText = $_.Exception.ToString()
    $logText = @()
    $logText += "build_portable_exe.ps1 failed: $timestamp"
    $logText += "ProjectDir: $ProjectDir"
    if ($cargoPath) { $logText += "Cargo: $cargoPath" }
    if ($cargoExitCode -ne $null) { $logText += "CargoExitCode: $cargoExitCode" }
    $logText += ""
    $logText += "ERROR:"
    $logText += $errText
    $logText += ""
    if ($cargoStdout) {
        $logText += "CARGO STDOUT:"
        $logText += $cargoStdout.TrimEnd()
        $logText += ""
    }
    if ($cargoStderr) {
        $logText += "CARGO STDERR:"
        $logText += $cargoStderr.TrimEnd()
        $logText += ""
    }

    Write-Utf8NoBomFile -Path $logPath -Text ($logText -join "`n")
    Write-Warning "Build failed. Log saved: $logPath"
    $buildFailed = $true
} finally {
    if (-not $KeepTarget) {
        $targetDir = Join-Path $ProjectDir "target"
        if (Test-Path $targetDir) {
            try {
                Remove-Item $targetDir -Recurse -Force
                Write-Host "Target cleaned: $targetDir"
            } catch {
                Write-Warning "Failed to remove target dir: $targetDir"
            }
        }
    }
}

if ($buildFailed) {
    return
}
