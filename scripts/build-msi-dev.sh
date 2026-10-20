#!/usr/bin/env bash
# Development MSI -> dist/KumkajCourierDev-<ver>.msi
# (com.kumkaj.courier.debug; separate install + data from prod.)
# Package first: cargo courier-windows-dev
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/build-msi.sh" dev
