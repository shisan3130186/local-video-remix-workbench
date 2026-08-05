@echo off
setlocal

cd /d "%~dp0"
title SmartCut - Dev Test

echo.
echo ========================================
echo SmartCut
echo Dev Test Environment
echo ========================================
echo.
echo Project folder:
echo %CD%
echo.

REM Resolve package manager: corepack pnpm > pnpm > npx tauri
set "PM_CMD="
where corepack >nul 2>nul
if not errorlevel 1 (
  set "PM_CMD=corepack pnpm tauri dev"
) else (
  where pnpm >nul 2>nul
  if not errorlevel 1 (
    set "PM_CMD=pnpm tauri dev"
  ) else (
    where npx >nul 2>nul
    if not errorlevel 1 (
      set "PM_CMD=npx tauri dev"
    )
  )
)

if not defined PM_CMD (
  echo [ERROR] No package manager found.
  echo Please install Node.js with corepack, or run: npm install -g pnpm
  echo.
  pause
  exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo [WARN] cargo was not found.
  echo If Tauri fails to start, please check the Rust environment.
  echo.
)

set CARGO_HTTP_CHECK_REVOKE=false

echo Starting dev environment...
echo Keep this window open while the app is running.
echo Close the app window to stop the development server.
echo.

%PM_CMD%
set EXIT_CODE=%ERRORLEVEL%

echo.
if not "%EXIT_CODE%"=="0" (
  echo [ERROR] Dev environment exited with code: %EXIT_CODE%
) else (
  echo Dev environment exited normally.
)
echo.
pause
exit /b %EXIT_CODE%