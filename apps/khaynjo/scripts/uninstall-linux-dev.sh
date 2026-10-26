#!/usr/bin/env bash
# Uninstall Khaynjo Dev (debug app id) on Linux.
# Thin wrapper — same as: ./apps/khaynjo/scripts/uninstall-linux.sh --dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/uninstall-linux.sh" --dev "$@"
