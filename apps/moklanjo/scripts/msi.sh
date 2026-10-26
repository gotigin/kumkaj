#!/usr/bin/env bash
# Moklanjo MSI (per-user install, no admin): dist/Moklanjo-<ver>.msi, or
# dist/MoklanjoDev-<ver>.msi for the development app id. Builds the Windows
# package first. Its own upgrade codes: installs and upgrades never touch
# other Kumkaj products' codes (and are distinct from the pre-rename
# Kumkaj Courier codes).
#   apps/moklanjo/scripts/msi.sh        # prod
#   apps/moklanjo/scripts/msi.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/moklanjo"
export KUMKAJ_BIN_NAME=kumkaj-moklanjo
export KUMKAJ_MSI_NAME=Moklanjo
export KUMKAJ_MSI_UPGRADE_CODE="{C183C0FA-3B73-45A8-8072-E5DE834A82BD}"
export KUMKAJ_MSI_DEV_UPGRADE_CODE="{3983F210-F93E-4B6B-A7EB-89B3A51BF11E}"
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/moklanjo-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/moklanjo-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/package-windows.sh"
exec "$ROOT/scripts/build-msi.sh" "${1:-prod}"
