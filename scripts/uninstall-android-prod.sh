#!/usr/bin/env bash
# Uninstall Kumkaj Courier (production) from a connected Android device/emulator.
# Package id: com.kumkaj.courier
set -euo pipefail
APP_ID=com.kumkaj.courier
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
