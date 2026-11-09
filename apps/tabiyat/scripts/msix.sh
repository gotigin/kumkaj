#!/usr/bin/env bash
# Tabiyat MSIX (Microsoft Store / sideload): dist/Tabiyat-<ver>.msix
# Identity Name is stable — never change after Store publish.
#   apps/tabiyat/scripts/msix.sh        # prod
#   apps/tabiyat/scripts/msix.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/tabiyat"
export KUMKAJ_BIN_NAME=kumkaj-tabiyat
export KUMKAJ_MSIX_NAME=Tabiyat
# Store Identity Name — Partner Center Package/Identity/Name (never rename after publish).
export KUMKAJ_MSIX_IDENTITY=MearajBhagad.Tabiyat
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/tabiyat-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/tabiyat-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/windows-prod.sh"
export KUMKAJ_MSI_PACKAGE_DEV="$HERE/windows-dev.sh"
exec "$ROOT/scripts/build-msix.sh" "${1:-prod}"
