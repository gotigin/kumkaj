#!/usr/bin/env bash
# Uninstall Tabiyat Dev from a connected Android device/emulator.
# Package id: shop.tabiyat.debug
set -euo pipefail
APP_ID=shop.tabiyat.debug
echo "adb uninstall $APP_ID"
adb uninstall "$APP_ID"
