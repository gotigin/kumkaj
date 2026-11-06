#!/usr/bin/env bash
# Install Khaynjo Dev (debug app id) on Linux.
# Rebuilds dist/khaynjo-linux-dev/ from the current tree, then installs it.
# Thin wrapper — same as: ./apps/khaynjo/scripts/install-linux.sh --dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/install-linux.sh" --dev "$@"
