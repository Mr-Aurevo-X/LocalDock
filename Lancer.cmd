@echo off
setlocal
cd /d "%~dp0"

if exist "target\release\localdock.exe" (
  start "" "target\release\localdock.exe"
  exit /b 0
)
if exist "target\debug\localdock.exe" (
  start "" "target\debug\localdock.exe"
  exit /b 0
)

where cargo >nul 2>&1
if errorlevel 1 (
  echo cargo introuvable. Installe Rust, ou compile LocalDock d'abord.
  pause
  exit /b 1
)

cargo run -p localdock
exit /b %ERRORLEVEL%
