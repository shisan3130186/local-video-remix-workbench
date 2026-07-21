param(
  [string]$SourceDirectory = "E:\Workspace\外部工具库\FFmpeg\ffmpeg-8.1.1-full_build-runtime\bin"
)

# 本脚本保存为带 BOM 的 UTF-8，确保 Windows PowerShell 5.1 能正确读取中文路径和提示。

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$targetDirectory = Join-Path $projectRoot "src-tauri\resources\ffmpeg"
$ffmpegSource = Join-Path $SourceDirectory "ffmpeg.exe"
$ffprobeSource = Join-Path $SourceDirectory "ffprobe.exe"

foreach ($path in @($ffmpegSource, $ffprobeSource)) {
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    throw "缺少FFmpeg运行文件：$path"
  }
}

$version = & $ffmpegSource -version 2>&1 | Select-Object -First 1
$filters = (& $ffmpegSource -hide_banner -filters 2>&1) -join "`n"
$encoders = (& $ffmpegSource -hide_banner -encoders 2>&1) -join "`n"

foreach ($requiredFilter in @("ass", "subtitles", "amix", "overlay", "xfade")) {
  if ($filters -notmatch "(?m)\b$requiredFilter\b") {
    throw "当前FFmpeg缺少智剪需要的滤镜：$requiredFilter"
  }
}

if ($encoders -notmatch "(?m)\blibx264\b") {
  throw "当前FFmpeg缺少H.264软件编码器libx264。"
}

New-Item -ItemType Directory -Path $targetDirectory -Force | Out-Null
Copy-Item -LiteralPath $ffmpegSource, $ffprobeSource -Destination $targetDirectory -Force

Write-Host "FFmpeg运行文件已准备完成。"
Write-Host $version
Get-ChildItem -LiteralPath $targetDirectory -Filter "*.exe" | ForEach-Object {
  $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  Write-Host "$($_.Name)  $($_.Length) bytes  SHA256=$hash"
}
