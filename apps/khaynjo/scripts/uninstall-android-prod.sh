#!/usr/bin/env bash
# Uninstall Khaynjo from a connected Android device/emulator.
# Package id: com.khaynjo
set -euo pipefail
APP_ID=com.khaynjo
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
