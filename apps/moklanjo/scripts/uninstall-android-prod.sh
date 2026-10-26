#!/usr/bin/env bash
# Uninstall Moklanjo (production) from a connected Android device/emulator.
# Package id: com.moklanjo
set -euo pipefail
APP_ID=com.moklanjo
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
