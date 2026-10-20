@echo off
REM Double-click uninstaller for Kumkaj Courier (Windows).
cd /d "%~dp0"
title Uninstall Kumkaj Courier
echo Uninstalling Kumkaj Courier...
echo.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1" -NoPause
set ERR=%ERRORLEVEL%
echo.
if %ERR% neq 0 (
  echo Uninstall failed with exit code %ERR%.
) else (
  echo Uninstall finished.
)
echo.
pause
exit /b %ERR%
