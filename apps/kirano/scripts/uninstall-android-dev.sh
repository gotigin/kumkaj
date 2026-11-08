#!/usr/bin/env bash
# Uninstall Kirano Dev from a connected Android device/emulator.
# Package id: com.kirano.debug
set -euo pipefail
APP_ID=com.kirano.debug
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
