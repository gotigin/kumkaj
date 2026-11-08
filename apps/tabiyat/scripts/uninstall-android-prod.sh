#!/usr/bin/env bash
# Uninstall Tabiyat from a connected Android device/emulator.
# Package id: com.tabiyat
set -euo pipefail
APP_ID=com.tabiyat
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
