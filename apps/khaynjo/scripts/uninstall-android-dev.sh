#!/usr/bin/env bash
# Uninstall Khaynjo Dev from a connected Android device/emulator.
# Package id: com.khaynjo.debug
set -euo pipefail
APP_ID=com.khaynjo.debug
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
