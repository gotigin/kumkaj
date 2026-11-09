#!/usr/bin/env bash
# Kirano MSIX (Microsoft Store / sideload): dist/Kirano-<ver>.msix
# Identity Name is stable — never change after Store publish.
#   apps/kirano/scripts/msix.sh        # prod
#   apps/kirano/scripts/msix.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/kirano"
export KUMKAJ_BIN_NAME=kumkaj-kirano
export KUMKAJ_MSIX_NAME=Kirano
# Store Identity Name — Partner Center Package/Identity/Name (never rename after publish).
export KUMKAJ_MSIX_IDENTITY=MearajBhagad.Kirano
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/kirano-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/kirano-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/windows-prod.sh"
export KUMKAJ_MSI_PACKAGE_DEV="$HERE/windows-dev.sh"
exec "$ROOT/scripts/build-msix.sh" "${1:-prod}"
