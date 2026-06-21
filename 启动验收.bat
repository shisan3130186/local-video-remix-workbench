@echo off
chcp 65001 >nul
setlocal

cd /d "%~dp0"
title 本地短视频批量混剪工作台 - 验收启动

echo.
echo ========================================
echo 本地短视频批量混剪工作台 - 验收启动
echo ========================================
echo.
echo 当前目录：
echo %CD%
echo.

where corepack >nul 2>nul
if errorlevel 1 (
  echo [错误] 未检测到 corepack，请先确认 Node.js / corepack 环境是否可用。
  echo.
  pause
  exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo [提醒] 未检测到 cargo。如果 Tauri 启动失败，请先检查 Rust 环境。
  echo.
)

set CARGO_HTTP_CHECK_REVOKE=false

echo 正在启动验收环境...
echo 如果窗口停在这里，通常表示软件正在运行中。
echo 关闭软件后，本窗口会继续显示退出信息。
echo.

corepack pnpm tauri dev
set EXIT_CODE=%ERRORLEVEL%

echo.
if not "%EXIT_CODE%"=="0" (
  echo [错误] 验收环境启动或运行异常，错误码：%EXIT_CODE%
) else (
  echo 验收环境已正常退出。
)
echo.
pause
exit /b %EXIT_CODE%
