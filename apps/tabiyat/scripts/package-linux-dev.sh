#!/usr/bin/env bash
# Build development Linux package (debug app id) + install/uninstall helpers.
# Output: dist/tabiyat-linux-dev/
# Uses debug Cargo profile + the app's env/development.env (→ shop.tabiyat.debug)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/tabiyat"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
export KUMKAJ_BIN_NAME=kumkaj-tabiyat
export KUMKAJ_INSTALL_SLUG=kumkaj-tabiyat
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=../../../scripts/windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# shellcheck source=desktop-ids.sh
source "$KUMKAJ_APP_DIR/scripts/desktop-ids.sh"
# shellcheck source=../../../scripts/stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env development
makepad_enable_package_dir

# Window/taskbar icons are this app's (workspace .cargo/config.toml defaults
# to Moklanjo's). Cargo [env] does not override exported vars.
for size in 32 64 128 256 512 1024; do
  export "MAKEPAD_APP_ICON_$size=$KUMKAJ_APP_DIR/resources/icon_$size.png"
done
export MAKEPAD_APP_ICON_ICO="$KUMKAJ_APP_DIR/resources/icon.ico"
cargo build -p kumkaj-tabiyat
# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
BIN="$TARGET_DIR/debug/kumkaj-tabiyat"
if [[ ! -f "$BIN" ]]; then
  echo "Missing binary: $BIN" >&2
  exit 1
fi

OUT="$ROOT/dist/tabiyat-linux-dev"
rm -rf "$OUT"
mkdir -p "$OUT/bin"

cp -f "$BIN" "$OUT/bin/kumkaj-tabiyat"
chmod +x "$OUT/bin/kumkaj-tabiyat"
makepad_copy_desktop_resources "$OUT"
copy_app_menu_assets_if_present "$OUT"

stage_desktop_package "$OUT" linux

cat >"$OUT/README.txt" <<EOF
Tabiyat Linux (development / debug)
==========================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}

Install:
  ./install.sh
→ ~/.local/lib/kumkaj-tabiyat-debug/
→ command: kumkaj-tabiyat-dev
→ data: ~/.local/share/${KUMKAJ_APP_ID}/

Uninstall:
  Uninstall Tabiyat Dev (app menu)
  or: ./uninstall.sh --dev

Prod package: ./apps/tabiyat/scripts/package-linux.sh → dist/tabiyat-linux/
Live from repo: cargo tabiyat-dev
EOF

echo "Linux Dev package: $OUT/"
