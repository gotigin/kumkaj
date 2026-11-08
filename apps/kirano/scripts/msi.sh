#!/usr/bin/env bash
# Kirano MSI (per-user install, no admin): dist/Kirano-<ver>.msi, or
# dist/KiranoDev-<ver>.msi for the development app id. Builds the Windows
# package first. Its own upgrade codes: installs and upgrades never touch
# Moklanjo's.
#   apps/kirano/scripts/msi.sh        # prod
#   apps/kirano/scripts/msi.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/kirano"
export KUMKAJ_BIN_NAME=kumkaj-kirano
export KUMKAJ_MSI_NAME=Kirano
export KUMKAJ_MSI_UPGRADE_CODE="{5F57629C-0F20-4F81-9ABE-EBBA1C27DC5E}"
export KUMKAJ_MSI_DEV_UPGRADE_CODE="{188CA577-79BE-4192-8383-B77DDFED0DA8}"
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/kirano-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/kirano-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/windows-prod.sh"
exec "$ROOT/scripts/build-msi.sh" "${1:-prod}"
