@echo off
setlocal EnableExtensions
chcp 65001 >nul

cd /d "%~dp0"
title 智剪 SmartCut - 启动中

echo.
echo ========================================
echo        智剪 SmartCut
echo        正在启动开发版
echo ========================================
echo.
echo 项目目录：%CD%
echo.

if not exist "package.json" (
  echo [错误] 没有找到 package.json。
  echo 请确认此脚本位于项目根目录。
  echo.
  pause
  exit /b 1
)

where corepack >nul 2>&1
if not errorlevel 1 (
  echo 正在使用 Corepack + pnpm 启动，请稍候...
  set "CARGO_HTTP_CHECK_REVOKE=false"
  call corepack pnpm tauri dev
  goto :finished
)

where pnpm >nul 2>&1
if not errorlevel 1 (
  echo 正在使用 pnpm 启动，请稍候...
  set "CARGO_HTTP_CHECK_REVOKE=false"
  call pnpm tauri dev
  goto :finished
)

where npx >nul 2>&1
if not errorlevel 1 (
  echo 未找到 pnpm，正在使用 npx 启动，请稍候...
  set "CARGO_HTTP_CHECK_REVOKE=false"
  call npx tauri dev
  goto :finished
)

echo [错误] 没有找到 Node.js、Corepack、pnpm 或 npx。
echo 请先安装 Node.js，再重新双击此脚本。
set "EXIT_CODE=1"
goto :show_result

:finished
set "EXIT_CODE=%ERRORLEVEL%"

:show_result
echo.
if not "%EXIT_CODE%"=="0" (
  echo [错误] 智剪启动失败，错误代码：%EXIT_CODE%
  echo 请保留此窗口中的错误信息，便于后续排查。
) else (
  echo 智剪已正常退出。
)
echo.
pause
exit /b %EXIT_CODE%
