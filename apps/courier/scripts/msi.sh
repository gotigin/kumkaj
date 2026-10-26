#!/usr/bin/env bash
# Kumkaj Courier MSI (per-user install, no admin): dist/KumkajCourier-<ver>.msi,
# or dist/KumkajCourierDev-<ver>.msi for the development app id. Builds the
# Windows package first. The upgrade codes are Courier's for good — installed
# copies upgrade by them; never change them.
#   apps/courier/scripts/msi.sh        # prod
#   apps/courier/scripts/msi.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/courier"
export KUMKAJ_BIN_NAME=kumkaj-courier
export KUMKAJ_MSI_NAME=KumkajCourier
export KUMKAJ_MSI_UPGRADE_CODE="{66C4AE1F-BD8E-4134-9A4F-8C67DCA42EA4}"
export KUMKAJ_MSI_DEV_UPGRADE_CODE="{B9D76BCC-DE80-4043-9B2E-47EA8972E82D}"
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/package-windows.sh"
exec "$ROOT/scripts/build-msi.sh" "${1:-prod}"
