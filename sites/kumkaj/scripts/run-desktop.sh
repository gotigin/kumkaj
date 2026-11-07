#!/usr/bin/env bash
# Run kumkaj.com as a native desktop window (fastest local preview).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
exec cargo run -p kumkaj-site "$@"
