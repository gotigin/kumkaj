#!/usr/bin/env bash
# Development Windows x86_64 package (debug *app id*, not Cargo debug).
# Uses --release so the .exe stays sane (~tens of MB). Cargo debug PE with
# Makepad was ~400MB — unusable to copy. Identity still comes from
# development.env → com.kirano.debug (separate install/data from prod).
# Makepad layout: kumkaj-kirano.exe + kumkaj_kirano/resources/ (underscores).
#
#   windows-dev.sh        development package  -> dist/kirano-windows-dev/
#   windows-dev.sh prod   production package   -> dist/kirano-windows/
#                         (the MSI: apps/kirano/scripts/msi.sh)
set -euo pipefail
FLAVOR="${1:-dev}"
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/kirano"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
# Desktop install identity for THIS app (exe, install folder slug).
export KUMKAJ_BIN_NAME=kumkaj-kirano
export KUMKAJ_INSTALL_SLUG=kumkaj-kirano
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=../../../scripts/windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# Kirano's own install helpers (Setup.bat / install.ps1 / uninstall name
# Kirano's exe and folders; Moklanjo's used to be staged here).
# shellcheck source=desktop-ids.sh
source "$KUMKAJ_APP_DIR/scripts/desktop-ids.sh"
# shellcheck source=../../../scripts/stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
case "$FLAVOR" in
  prod) load_kumkaj_env production; OUT_NAME=kirano-windows ;;
  dev) load_kumkaj_env development; OUT_NAME=kirano-windows-dev ;;
  *) echo "usage: $0 [dev|prod]" >&2; exit 1 ;;
esac
makepad_enable_package_dir

OUT="$ROOT/dist/$OUT_NAME"
rm -rf "$OUT"
mkdir -p "$OUT"
TARGET="$(windows_default_target)"

if [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]]; then
  if rustup target list --installed 2>/dev/null | grep -qx 'x86_64-pc-windows-msvc'; then
    TARGET=x86_64-pc-windows-msvc
  fi
else
  windows_ensure_toolchain
fi

# Same as Android: release binary + development.env (console stays — KUMKAJ_ENV).
# Window/taskbar icons are this app's (workspace .cargo/config.toml defaults
# to Moklanjo's). Cargo [env] does not override exported vars.
export MAKEPAD_APP_ICON_32="$KUMKAJ_APP_DIR/resources/icon_32.png"
export MAKEPAD_APP_ICON_64="$KUMKAJ_APP_DIR/resources/icon_64.png"
export MAKEPAD_APP_ICON_128="$KUMKAJ_APP_DIR/resources/icon_128.png"
export MAKEPAD_APP_ICON_256="$KUMKAJ_APP_DIR/resources/icon_256.png"
export MAKEPAD_APP_ICON_512="$KUMKAJ_APP_DIR/resources/icon_512.png"
export MAKEPAD_APP_ICON_1024="$KUMKAJ_APP_DIR/resources/icon_1024.png"
export MAKEPAD_APP_ICON_ICO="$KUMKAJ_APP_DIR/resources/icon.ico"
cargo build -p kumkaj-kirano --release --target "$TARGET"

# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/release/kumkaj-kirano.exe" \
  "$TARGET_DIR/$TARGET/release/kumkaj-kirano"
do
  if [[ -f "$cand" ]]; then
    EXE="$cand"
    break
  fi
done
if [[ -z "$EXE" ]]; then
  echo "Built binary not found under $TARGET_DIR/$TARGET/release/" >&2
  exit 1
fi

cp -f "$EXE" "$OUT/kumkaj-kirano.exe"
makepad_copy_desktop_resources "$OUT"
copy_app_menu_assets_if_present "$OUT"
stage_desktop_package "$OUT" windows

SIZE="$(du -h "$OUT/kumkaj-kirano.exe" | awk '{print $1}')"

if [ "$FLAVOR" = prod ]; then CONSOLE_NOTE="off (production.env)"; else CONSOLE_NOTE="on (development.env) — prod hides it."; fi
cat >"$OUT/README.txt" <<EOF
Kirano Windows ($FLAVOR)
====================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Cargo: --release (optimized). "Dev" means debug app id / data dir, not a 400MB debug PE.
Exe size: ${SIZE}
Target: $TARGET
Console: ${CONSOLE_NOTE}

Install:
  Double-click Setup.bat
→ %LOCALAPPDATA%\\Programs\\${KUMKAJ_APP_LABEL}\\
→ data: %APPDATA%\\${KUMKAJ_APP_ID}\\

Uninstall:
  Start Menu → Uninstall Kirano Dev
  or double-click Uninstall.bat

Prod: ./apps/kirano/scripts/windows-dev.sh prod → dist/kirano-windows/
MSI:  ./apps/kirano/scripts/msi.sh [prod|dev] → dist/Kirano-<ver>.msi
EOF

echo "Windows $FLAVOR package: $OUT/  (exe ${SIZE})"
echo "Copy the whole folder to Windows (exe + kumkaj_kirano/resources + assets/ + Setup.bat)."
