param(
    [string]$ProjectDir = $PSScriptRoot,
    [string]$ServerUrl = "http://100.64.0.7:3000",
    [string]$Owner = "Coxford",
    [string]$Repo = "TestProject",
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

$buildScript = Join-Path $ProjectDir "build_exe.ps1"
if (-not (Test-Path $buildScript)) {
    throw "build_exe.ps1 not found: $buildScript"
}

Write-Host "Building executable..."
if ($KeepTarget) {
    & $buildScript -ProjectDir $ProjectDir -KeepTarget
} else {
    & $buildScript -ProjectDir $ProjectDir
}

$exePath = Join-Path $ProjectDir "PetriNet.exe"
if (-not (Test-Path $exePath)) {
    throw "Executable not found: $exePath"
}

Write-Host "Preparing git tag $Tag..."
$tagExistsLocal = git tag --list $Tag
if ([string]::IsNullOrWhiteSpace($tagExistsLocal)) {
    git tag -a $Tag -m "Release $Tag"
}

Write-Host "Pushing tag to origin..."
git push origin $Tag

$headers = @{
    Authorization = "token $token"
    Accept = "application/json"
}

$baseApi = "$ServerUrl/api/v1/repos/$Owner/$Repo"
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
$assetName = "PetriNet.exe"

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
