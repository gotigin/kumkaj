#!/usr/bin/env bash
# Khaynjo MSI (per-user install, no admin): dist/Khaynjo-<ver>.msi, or
# dist/KhaynjoDev-<ver>.msi for the development app id. Builds the Windows
# package first. Its own upgrade codes: installs and upgrades never touch
# Kumkaj Courier's.
#   apps/khaynjo/scripts/msi.sh        # prod
#   apps/khaynjo/scripts/msi.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/khaynjo"
export KUMKAJ_BIN_NAME=kumkaj-khaynjo
export KUMKAJ_MSI_NAME=Khaynjo
export KUMKAJ_MSI_UPGRADE_CODE="{6A2D5834-79D6-47A7-B4D2-ADB9A9ABF832}"
export KUMKAJ_MSI_DEV_UPGRADE_CODE="{9EBCFB61-47C3-4746-8025-C3E6885E79F8}"
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/khaynjo-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/khaynjo-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/windows-prod.sh"
exec "$ROOT/scripts/build-msi.sh" "${1:-prod}"
