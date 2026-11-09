#!/usr/bin/env bash
# Moklanjo MSIX (Microsoft Store / sideload): dist/Moklanjo-<ver>.msix
# Identity Name is stable — never change after Store publish.
#   apps/moklanjo/scripts/msix.sh        # prod
#   apps/moklanjo/scripts/msix.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/moklanjo"
export KUMKAJ_BIN_NAME=kumkaj-moklanjo
export KUMKAJ_MSIX_NAME=Moklanjo
export KUMKAJ_MSIX_IDENTITY=Kumkaj.Moklanjo
export KUMKAJ_MSIX_IDENTITY_DEV=Kumkaj.Moklanjo.Dev
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/moklanjo-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/moklanjo-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/package-windows.sh"
export KUMKAJ_MSI_PACKAGE_DEV="$HERE/windows-dev.sh"
exec "$ROOT/scripts/build-msix.sh" "${1:-prod}"
