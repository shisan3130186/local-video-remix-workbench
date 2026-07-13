$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$ffmpegPath = Join-Path $projectRoot "src-tauri\resources\ffmpeg\ffmpeg.exe"
$ffprobePath = Join-Path $projectRoot "src-tauri\resources\ffmpeg\ffprobe.exe"
$iconPath = Join-Path $projectRoot "src-tauri\icons\icon.ico"
$tauriConfigPath = Join-Path $projectRoot "src-tauri\tauri.conf.json"

$checks = @(
    [PSCustomObject]@{ Item = "Tauri config"; Ready = Test-Path -LiteralPath $tauriConfigPath; Required = $true },
    [PSCustomObject]@{ Item = "Windows icon"; Ready = Test-Path -LiteralPath $iconPath; Required = $true },
    [PSCustomObject]@{ Item = "Bundled ffmpeg.exe"; Ready = Test-Path -LiteralPath $ffmpegPath; Required = $true },
    [PSCustomObject]@{ Item = "Bundled ffprobe.exe"; Ready = Test-Path -LiteralPath $ffprobePath; Required = $true },
    [PSCustomObject]@{ Item = "AI_API_KEY in current shell"; Ready = [bool]$env:AI_API_KEY; Required = $false },
    [PSCustomObject]@{ Item = "AI_BASE_URL in current shell"; Ready = [bool]$env:AI_BASE_URL; Required = $false },
    [PSCustomObject]@{ Item = "AI_MODEL in current shell"; Ready = [bool]$env:AI_MODEL; Required = $false }
)

$checks | Format-Table Item, Ready, Required -AutoSize

$missingRequired = @($checks | Where-Object { $_.Required -and -not $_.Ready })

if ($missingRequired.Count -gt 0) {
    Write-Host ""
    Write-Host "Installer is not ready for a clean PC. Required items are missing." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Basic installer resources are ready. An AI credential delivery plan is still required." -ForegroundColor Green
