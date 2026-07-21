param(
  [string]$InstallerPath = "",
  [switch]$InstallAndLaunch
)

# 本脚本保存为带 BOM 的 UTF-8，确保 Windows PowerShell 5.1 能正确读取中文路径和提示。

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$resultRoot = "E:\Workspace\测试结果库\Project_03_本地短视频批量混剪工作台\2026-07-21_第四阶段用户测试包"
New-Item -ItemType Directory -Path $resultRoot -Force | Out-Null

function Get-PeSubsystem([string]$Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  if ($bytes.Length -lt 256) {
    throw "主程序不是有效的Windows PE文件：$Path"
  }
  $peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
  return [BitConverter]::ToUInt16($bytes, $peOffset + 24 + 68)
}

if (-not $InstallerPath) {
  $bundleDirectory = Join-Path $projectRoot "src-tauri\target\release\bundle\nsis"
  $installer = Get-ChildItem -LiteralPath $bundleDirectory -Filter "*.exe" -File |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
  if (-not $installer) {
    throw "没有找到NSIS安装包，请先执行 corepack pnpm tauri build。"
  }
  $InstallerPath = $installer.FullName
}

$installerItem = Get-Item -LiteralPath $InstallerPath
$maximumBytes = 1024MB
if ($installerItem.Length -gt $maximumBytes) {
  throw "安装包超过1GB限制：$($installerItem.Length) bytes"
}

$deliveryInstaller = Join-Path $resultRoot $installerItem.Name
Copy-Item -LiteralPath $installerItem.FullName -Destination $deliveryInstaller -Force

$ffmpeg = Join-Path $projectRoot "src-tauri\resources\ffmpeg\ffmpeg.exe"
$ffprobe = Join-Path $projectRoot "src-tauri\resources\ffmpeg\ffprobe.exe"
$privacy = Join-Path $projectRoot "src-tauri\resources\legal\PRIVACY_NOTICE.txt"
$license = Join-Path $projectRoot "src-tauri\resources\legal\FFMPEG_GPL-3.0.txt"
$notices = Join-Path $projectRoot "src-tauri\resources\legal\THIRD_PARTY_NOTICES.txt"

foreach ($path in @($ffmpeg, $ffprobe, $privacy, $license, $notices)) {
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    throw "用户测试包缺少资源：$path"
  }
}

$ffmpegVersion = (& $ffmpeg -version 2>&1 | Select-Object -First 1)
$ffprobeVersion = (& $ffprobe -version 2>&1 | Select-Object -First 1)
$installedExecutable = ""
$installedFfmpeg = ""
$installedPrivacy = ""
$installedSubsystem = ""

if ($InstallAndLaunch) {
  $installDirectory = Join-Path $resultRoot "installed-smartcut"
  if (Test-Path -LiteralPath $installDirectory) {
    $resolved = [System.IO.Path]::GetFullPath($installDirectory)
    $allowed = [System.IO.Path]::GetFullPath($resultRoot)
    if (-not $resolved.StartsWith($allowed, [System.StringComparison]::OrdinalIgnoreCase)) {
      throw "安装测试目录不在允许范围内。"
    }
    Remove-Item -LiteralPath $installDirectory -Recurse -Force
  }

  $process = Start-Process -FilePath $installerItem.FullName -ArgumentList @("/S", "/D=$installDirectory") -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    throw "静默安装失败，退出码：$($process.ExitCode)"
  }

  $app = Get-ChildItem -LiteralPath $installDirectory -Recurse -Filter "*.exe" -File |
    Where-Object { $_.Name -notin @("ffmpeg.exe", "ffprobe.exe", "uninstall.exe") } |
    Sort-Object Length -Descending |
    Select-Object -First 1
  if (-not $app) {
    throw "安装后没有找到智剪主程序。"
  }

  $installedExecutable = $app.FullName
  $installedSubsystem = Get-PeSubsystem $installedExecutable
  if ($installedSubsystem -ne 2) {
    throw "安装后主程序不是Windows桌面程序，PE Subsystem=$installedSubsystem"
  }
  $installedFfmpeg = Join-Path $app.Directory.FullName "ffmpeg\ffmpeg.exe"
  if (-not (Test-Path -LiteralPath $installedFfmpeg -PathType Leaf)) {
    throw "安装后没有找到内置FFmpeg：$installedFfmpeg"
  }

  $installedPrivacy = Join-Path $app.Directory.FullName "legal\PRIVACY_NOTICE.txt"
  if (-not (Test-Path -LiteralPath $installedPrivacy -PathType Leaf)) {
    throw "安装后没有找到隐私说明：$installedPrivacy"
  }

  Start-Process -FilePath $app.FullName | Out-Null
}

$report = @(
  "# 智剪 SmartCut 用户测试包自动检查"
  ""
  "- 检查时间：$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
  "- 安装包：$($installerItem.FullName)"
  "- 验收副本：$deliveryInstaller"
  "- 安装包大小：$([math]::Round($installerItem.Length / 1MB, 2)) MB"
  "- SHA256：$((Get-FileHash -LiteralPath $installerItem.FullName -Algorithm SHA256).Hash.ToLowerInvariant())"
  "- FFmpeg：$ffmpegVersion"
  "- FFprobe：$ffprobeVersion"
  "- 隐私说明：存在"
  "- FFmpeg许可证：存在"
  "- 第三方组件说明：存在"
  "- 安装测试：$(if ($InstallAndLaunch) { '通过并已启动' } else { '未执行' })"
  "- 安装后主程序：$installedExecutable"
  "- 主程序类型：Windows GUI (PE Subsystem $installedSubsystem)"
  "- 安装后内置FFmpeg：$installedFfmpeg"
  "- 安装后隐私说明：$installedPrivacy"
) -join "`r`n"

$reportPath = Join-Path $resultRoot "自动检查报告.md"
[System.IO.File]::WriteAllText($reportPath, $report, [System.Text.UTF8Encoding]::new($false))
Write-Host $report
Write-Host "报告已保存：$reportPath"
