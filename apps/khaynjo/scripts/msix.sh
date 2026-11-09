#!/usr/bin/env bash
# Khaynjo MSIX (Microsoft Store / sideload): dist/Khaynjo-<ver>.msix
# Identity Name is stable — never change after Store publish.
#   apps/khaynjo/scripts/msix.sh        # prod
#   apps/khaynjo/scripts/msix.sh dev    # dev
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/khaynjo"
export KUMKAJ_BIN_NAME=kumkaj-khaynjo
export KUMKAJ_MSIX_NAME=Khaynjo
export KUMKAJ_MSIX_IDENTITY=Kumkaj.Khaynjo
export KUMKAJ_MSIX_IDENTITY_DEV=Kumkaj.Khaynjo.Dev
export KUMKAJ_MSI_PKG_PROD="$ROOT/dist/khaynjo-windows"
export KUMKAJ_MSI_PKG_DEV="$ROOT/dist/khaynjo-windows-dev"
export KUMKAJ_MSI_PACKAGE_PROD="$HERE/windows-prod.sh"
export KUMKAJ_MSI_PACKAGE_DEV="$HERE/windows-dev.sh"
exec "$ROOT/scripts/build-msix.sh" "${1:-prod}"
