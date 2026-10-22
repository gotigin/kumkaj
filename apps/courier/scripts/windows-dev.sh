#!/usr/bin/env bash
# Development Windows x86_64 package (debug *app id*, not Cargo debug).
# Uses --release so the .exe stays sane (~tens of MB). Cargo debug PE with
# Makepad was ~400MB — unusable to copy. Identity still comes from
# development.env → com.kumkaj.courier.debug (separate install/data from prod).
# Makepad layout: kumkaj-courier.exe + kumkaj_courier/resources/ (underscores).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/courier"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=windows-env.sh
source "$ROOT/scripts/windows-env.sh"
# shellcheck source=desktop-ids.sh
source "$ROOT/scripts/desktop-ids.sh"
# shellcheck source=stage-desktop-package.sh
source "$ROOT/scripts/stage-desktop-package.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env development
makepad_enable_package_dir

OUT="$ROOT/dist/windows-dev"
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
cargo build -p kumkaj-courier --release --target "$TARGET"

# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
EXE=""
for cand in \
  "$TARGET_DIR/$TARGET/release/kumkaj-courier.exe" \
  "$TARGET_DIR/$TARGET/release/kumkaj-courier"
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

cp -f "$EXE" "$OUT/kumkaj-courier.exe"
makepad_copy_desktop_resources "$OUT"
stage_desktop_package "$OUT" windows

SIZE="$(du -h "$OUT/kumkaj-courier.exe" | awk '{print $1}')"

cat >"$OUT/README.txt" <<EOF
Kumkaj Courier Windows (development)
====================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Cargo: --release (optimized). "Dev" means debug app id / data dir, not a 400MB debug PE.
Exe size: ${SIZE}
Target: $TARGET
Console: on (development.env) — prod hides it.

Install:
  Double-click Setup.bat
→ %LOCALAPPDATA%\\Programs\\Kumkaj Courier Dev\\
→ data: %APPDATA%\\${KUMKAJ_APP_ID}\\

Uninstall:
  Start Menu → Uninstall Kumkaj Courier Dev
  or double-click Uninstall.bat

Prod: ./scripts/package-windows.sh → dist/windows/
EOF

echo "Windows Dev package: $OUT/  (exe ${SIZE})"
echo "Copy the whole folder to Windows (exe + kumkaj_courier/resources + Setup.bat)."
