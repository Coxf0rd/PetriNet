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

$baseApi = "$ServerUrl/api/v1/repos/$Owner/$Repo"

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
$releaseByTagUrl = "$baseApi/releases/tags/$Tag"

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

    $release = Invoke-RestMethod -Method Post -Headers $headers -Uri "$baseApi/releases" -Body $createBody -ContentType "application/json"
}

$releaseId = $release.id
if (-not $releaseId) {
    throw "Failed to resolve release id for tag $Tag"
}

$assetsUrl = "$baseApi/releases/$releaseId/assets"
$assetName = Split-Path -Leaf $exePath

Write-Host "Removing old asset with same name (if exists)..."
$assets = Invoke-RestMethod -Method Get -Headers $headers -Uri $assetsUrl
foreach ($asset in $assets) {
    if ($asset.name -eq $assetName) {
        Invoke-RestMethod -Method Delete -Headers $headers -Uri "$assetsUrl/$($asset.id)"
    }
}

Write-Host "Uploading executable to release..."
$uploadUrl = "$assetsUrl?name=$assetName"
Invoke-RestMethod -Method Post -Headers @{ Authorization = "token $token"; Accept = "application/json"; "Content-Type" = "application/octet-stream" } -Uri $uploadUrl -InFile $exePath

Write-Host "Done. Release asset uploaded:"
Write-Host "$ServerUrl/$Owner/$Repo/releases/tag/$Tag"
