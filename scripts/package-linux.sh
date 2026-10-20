#!/usr/bin/env bash
# Build production Linux package for Kumkaj Courier + install/uninstall helpers.
# Output: dist/linux/
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# shellcheck source=desktop-ids.sh
source "$ROOT/scripts/desktop-ids.sh"
# shellcheck source=stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production
makepad_enable_package_dir

cargo build -p kumkaj-courier --release
# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
BIN="$TARGET_DIR/release/kumkaj-courier"
if [[ ! -f "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi

OUT="$ROOT/dist/linux"
rm -rf "$OUT"
mkdir -p "$OUT/bin"

cp -f "$BIN" "$OUT/bin/kumkaj-courier"
chmod +x "$OUT/bin/kumkaj-courier"
# Makepad: resources at <package>/kumkaj_courier/resources/ (underscores; cwd = package root).
# Binary lives under bin/ (hyphenated name) so it never collides with the resources dir.
makepad_copy_desktop_resources "$OUT"

stage_desktop_package "$OUT" linux

cat >"$OUT/README.txt" <<EOF
Kumkaj Courier Linux (production)
=================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}

Install (recommended — adds app menu + Uninstall entry):
  ./install.sh

Portable run (no install):
  ./run.sh

Uninstall (after install, or re-run to delete leftover data):
  Uninstall Kumkaj Courier  (app menu)
  or: ~/.local/lib/kumkaj-courier/uninstall.sh
  or: ./uninstall.sh --prod

Uninstall asks whether to keep or remove books data:
  ${HOME}/.local/share/${KUMKAJ_APP_ID}/

Default Owner PIN: 0000 — change under More → Team after first unlock.
EOF

echo "Linux package: $OUT/"
echo "  Install with: $OUT/install.sh"
