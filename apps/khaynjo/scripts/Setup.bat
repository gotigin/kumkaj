@echo off
REM Double-click installer for Khaynjo (Windows).
REM Runs install.ps1 and always waits so errors are visible.
cd /d "%~dp0"
title Khaynjo Setup
echo Installing Khaynjo...
echo.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" -NoPause
set ERR=%ERRORLEVEL%
echo.
if %ERR% neq 0 (
  echo Setup failed with exit code %ERR%.
) else (
  echo Setup finished.
)
echo.
pause
exit /b %ERR%
