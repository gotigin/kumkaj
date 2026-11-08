#!/usr/bin/env bash
# Uninstall Kirano from a connected Android device/emulator.
# Package id: com.kirano
set -euo pipefail
APP_ID=com.kirano
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
