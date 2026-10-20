#!/usr/bin/env bash
# Uninstall Kumkaj Courier Dev (debug app id) on Linux.
# Thin wrapper — same as: ./scripts/uninstall-linux.sh --dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/uninstall-linux.sh" --dev "$@"
