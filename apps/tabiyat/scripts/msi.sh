#!/usr/bin/env bash
# Tabiyat MSI (per-user install, no admin): dist/Tabiyat-<ver>.msi, or
# dist/TabiyatDev-<ver>.msi for the development app id. Builds the Windows
# package first. Its own upgrade codes: installs and upgrades never touch
# Moklanjo's.
#   apps/tabiyat/scripts/msi.sh        # prod
#   apps/tabiyat/scripts/msi.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/tabiyat"
export KUMKAJ_BIN_NAME=kumkaj-tabiyat
export KUMKAJ_MSI_NAME=Tabiyat
export KUMKAJ_MSI_UPGRADE_CODE="{547E1FB9-D2AA-4BAC-AC1F-663243B0030C}"
export KUMKAJ_MSI_DEV_UPGRADE_CODE="{041F003D-26DA-45EE-B77F-388ADD9B9EC2}"
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/tabiyat-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/tabiyat-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/windows-prod.sh"
exec "$ROOT/scripts/build-msi.sh" "${1:-prod}"
