#!/usr/bin/env bash
# Build production Linux package for Kirano + install/uninstall helpers.
# Output: dist/kirano-linux/
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/kirano"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
export KUMKAJ_BIN_NAME=kumkaj-kirano
export KUMKAJ_INSTALL_SLUG=kumkaj-kirano
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=../../../scripts/windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# shellcheck source=desktop-ids.sh
source "$KUMKAJ_APP_DIR/scripts/desktop-ids.sh"
# shellcheck source=../../../scripts/stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production
makepad_enable_package_dir

# Window/taskbar icons are this app's (workspace .cargo/config.toml defaults
# to Moklanjo's). Cargo [env] does not override exported vars.
for size in 32 64 128 256 512 1024; do
  export "MAKEPAD_APP_ICON_$size=$KUMKAJ_APP_DIR/resources/icon_$size.png"
done
export MAKEPAD_APP_ICON_ICO="$KUMKAJ_APP_DIR/resources/icon.ico"
cargo build -p kumkaj-kirano --release
# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
BIN="$TARGET_DIR/release/kumkaj-kirano"
if [[ ! -f "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi

OUT="$ROOT/dist/kirano-linux"
rm -rf "$OUT"
mkdir -p "$OUT/bin"

cp -f "$BIN" "$OUT/bin/kumkaj-kirano"
chmod +x "$OUT/bin/kumkaj-kirano"
# Makepad: resources at <package>/kumkaj_kirano/resources/ (underscores; cwd = package root).
# Binary lives under bin/ (hyphenated name) so it never collides with the resources dir.
makepad_copy_desktop_resources "$OUT"
copy_app_menu_assets_if_present "$OUT"

stage_desktop_package "$OUT" linux

cat >"$OUT/README.txt" <<EOF
Kirano Linux (production)
=================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}

Install (recommended — adds app menu + Uninstall entry):
  ./install.sh

Portable run (no install):
  ./run.sh

Uninstall (after install, or re-run to delete leftover data):
  Uninstall Kirano  (app menu)
  or: ~/.local/lib/kumkaj-kirano/uninstall.sh
  or: ./uninstall.sh --prod

Uninstall asks which local data to remove (records / setup / preferences):
  ${HOME}/.local/share/${KUMKAJ_APP_ID}/

First start asks for the Owner PIN (Server Mode) — no default PIN to change.
EOF

echo "Linux package: $OUT/"
echo "  Install with: $OUT/install.sh"
