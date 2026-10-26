#!/usr/bin/env bash
# Uninstall Kumkaj Courier Dev from a connected Android device/emulator.
# Package id: com.kumkaj.courier.debug
set -euo pipefail
APP_ID=com.kumkaj.courier.debug
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
