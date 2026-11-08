#!/usr/bin/env bash
# Install Tabiyat Dev (debug app id) on Linux.
# Rebuilds dist/tabiyat-linux-dev/ from the current tree, then installs it.
# Thin wrapper — same as: ./apps/tabiyat/scripts/install-linux.sh --dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/install-linux.sh" --dev "$@"
