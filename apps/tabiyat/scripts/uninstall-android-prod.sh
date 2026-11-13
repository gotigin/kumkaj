#!/usr/bin/env bash
# Uninstall Tabiyat from a connected Android device/emulator.
# Package id: shop.tabiyat
set -euo pipefail
APP_ID=shop.tabiyat
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
