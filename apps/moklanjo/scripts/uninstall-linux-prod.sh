#!/usr/bin/env bash
# Uninstall Moklanjo (production) on Linux.
# Thin wrapper — same as: ./apps/moklanjo/scripts/uninstall-linux.sh --prod
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/uninstall-linux.sh" --prod "$@"
