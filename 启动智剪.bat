@echo off
setlocal EnableExtensions

cd /d "%~dp0"
title SmartCut - Starting

if not exist "package.json" (
  echo [ERROR] package.json was not found.
  echo Please keep this file in the project root folder.
  pause
  exit /b 1
)

where corepack >nul 2>&1
if not errorlevel 1 (
  echo Starting SmartCut with Corepack and pnpm...
  set "CARGO_HTTP_CHECK_REVOKE=false"
  call corepack pnpm tauri dev
  goto :result
)

where pnpm >nul 2>&1
if not errorlevel 1 (
  echo Starting SmartCut with pnpm...
  set "CARGO_HTTP_CHECK_REVOKE=false"
  call pnpm tauri dev
  goto :result
)

where npx >nul 2>&1
if not errorlevel 1 (
  echo pnpm was not found. Starting SmartCut with npx...
  set "CARGO_HTTP_CHECK_REVOKE=false"
  call npx tauri dev
  goto :result
)

echo [ERROR] Node.js, Corepack, pnpm and npx were not found.
echo Please install Node.js and try again.
set "EXIT_CODE=1"
goto :show_result

:result
set "EXIT_CODE=%ERRORLEVEL%"

:show_result
echo.
if not "%EXIT_CODE%"=="0" (
  echo [ERROR] SmartCut failed to start. Exit code: %EXIT_CODE%
  echo Keep this window open if you need to report the error.
) else (
  echo SmartCut exited normally.
)
echo.
pause
exit /b %EXIT_CODE%
