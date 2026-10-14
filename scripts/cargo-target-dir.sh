#!/usr/bin/env bash
# Print Cargo target_directory for the current workspace (no Python).
# Usage: source this file, or: ./scripts/cargo-target-dir.sh
cargo_target_dir() {
  if command -v jq >/dev/null 2>&1; then
    cargo metadata --format-version=1 --no-deps | jq -r '.target_directory'
    return
  fi
  # Fallback: first "target_directory":"…" field (paths rarely need JSON escapes).
  cargo metadata --format-version=1 --no-deps \
    | tr ',' '\n' \
    | sed -n 's/.*"target_directory"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n1
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -euo pipefail
  cargo_target_dir
fi
