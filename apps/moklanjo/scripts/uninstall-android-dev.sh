#!/usr/bin/env bash
# Uninstall Moklanjo Dev from a connected Android device/emulator.
# Package id: com.moklanjo.debug
set -euo pipefail
APP_ID=com.moklanjo.debug
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
