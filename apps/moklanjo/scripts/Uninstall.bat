@echo off
REM Double-click uninstaller for Moklanjo (Windows).
cd /d "%~dp0"
title Uninstall Moklanjo
echo Uninstalling Moklanjo...
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
