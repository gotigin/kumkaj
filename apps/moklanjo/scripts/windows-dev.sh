#!/usr/bin/env bash
# Development Windows x86_64 package (debug *app id*, not Cargo debug).
# Uses --release so the .exe stays sane (~tens of MB). Cargo debug PE with
# Makepad was ~400MB — unusable to copy. Identity still comes from
# development.env → com.moklanjo.debug (separate install/data from prod).
# Makepad layout: kumkaj-moklanjo.exe + kumkaj_moklanjo/resources/ (underscores).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/moklanjo"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
export KUMKAJ_BIN_NAME=kumkaj-moklanjo
export KUMKAJ_INSTALL_SLUG=kumkaj-moklanjo
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

OUT="$ROOT/dist/moklanjo-windows-dev"
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
cargo build -p kumkaj-moklanjo --release --target "$TARGET"

# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/release/kumkaj-moklanjo.exe" \
  "$TARGET_DIR/$TARGET/release/kumkaj-moklanjo"
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

cp -f "$EXE" "$OUT/kumkaj-moklanjo.exe"
makepad_copy_desktop_resources "$OUT"
stage_desktop_package "$OUT" windows

SIZE="$(du -h "$OUT/kumkaj-moklanjo.exe" | awk '{print $1}')"

cat >"$OUT/README.txt" <<EOF
Moklanjo Windows (development)
====================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Cargo: --release (optimized). "Dev" means debug app id / data dir, not a 400MB debug PE.
Exe size: ${SIZE}
Target: $TARGET
Console: on (development.env) — prod hides it.

Install:
  Double-click Setup.bat
→ %LOCALAPPDATA%\\Programs\\Moklanjo Dev\\
→ data: %APPDATA%\\${KUMKAJ_APP_ID}\\

Uninstall:
  Start Menu → Uninstall Moklanjo Dev
  or double-click Uninstall.bat

Prod: ./apps/moklanjo/scripts/package-windows.sh → dist/moklanjo-windows/
MSI:  ./apps/moklanjo/scripts/msi.sh [prod|dev] → dist/Moklanjo[Dev]-<ver>.msi
EOF

echo "Windows Dev package: $OUT/  (exe ${SIZE})"
echo "Copy the whole folder to Windows (exe + kumkaj_moklanjo/resources + Setup.bat)."
