#!/usr/bin/env bash
# Uninstall Kirano from a connected Android device/emulator.
# Package id: shop.kirano
set -euo pipefail
APP_ID=shop.kirano
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
