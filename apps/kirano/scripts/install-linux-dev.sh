#!/usr/bin/env bash
# Install Kirano Dev (debug app id) on Linux.
# Rebuilds dist/kirano-linux-dev/ from the current tree, then installs it.
# Thin wrapper — same as: ./apps/kirano/scripts/install-linux.sh --dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec "$ROOT/install-linux.sh" --dev "$@"
