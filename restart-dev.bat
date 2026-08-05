@echo off
setlocal

echo === SmartCut Dev Restart ===

echo [1/3] Killing processes...
taskkill /F /IM "local-video-remix-workbench.exe" /T >nul 2>&1
taskkill /F /IM "smartcut-preview.exe" /T >nul 2>&1
taskkill /F /IM "cargo.exe" /T >nul 2>&1

echo [2/3] Starting tauri dev...
start "SmartCut Dev Test" /D "%~dp0" "%~dp0start-acceptance.bat"

echo [3/3] Done. Wait ~30-60s for Rust compile.
pause
