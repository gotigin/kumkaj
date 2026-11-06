#!/usr/bin/env bash
# Install Moklanjo Dev (debug app id) on Linux.
# Rebuilds dist/linux-dev/ from the current tree, then installs it.
# Thin wrapper — same as: ./apps/moklanjo/scripts/install-linux.sh --dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/install-linux.sh" --dev "$@"
