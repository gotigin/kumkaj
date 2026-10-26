@echo off
REM Double-click installer for Kumkaj Courier (Windows).
REM Runs install.ps1 and always waits so errors are visible.
cd /d "%~dp0"
title Kumkaj Courier Setup
echo Installing Kumkaj Courier...
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
