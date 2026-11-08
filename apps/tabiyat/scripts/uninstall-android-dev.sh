#!/usr/bin/env bash
# Uninstall Tabiyat Dev from a connected Android device/emulator.
# Package id: com.tabiyat.debug
set -euo pipefail
APP_ID=com.tabiyat.debug
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
