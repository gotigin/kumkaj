#!/usr/bin/env bash
# Android packaging via cargo-makepad (requires Android SDK/NDK toolchain).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=android-env.sh
source "$ROOT/scripts/android-env.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production

OUT="$ROOT/dist/android"
mkdir -p "$OUT"

resolve_makepad_root
export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"

android_pkg_opts "$ANDROID_HOME"

cat > "$OUT/README.txt" <<EOF
Kumkaj Android packaging
========================
Application id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Website: https://kumkaj.com
Env file: apps/courier/env/production.env

Build APK:
  MAKEPAD_ROOT=$MAKEPAD_ROOT ./scripts/package-android.sh

Live install/run on device (development id com.kumkaj.courier.debug):
  cargo courier-android-dev

Release AAB for Play Store (after creating a keystore):
  cargo run --manifest-path "\$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \\
    android --package-name=${KUMKAJ_APP_ID} --app-label=${KUMKAJ_APP_LABEL} --abi=aarch64 --sdk-path="\$ANDROID_HOME" \\
    --keystore=play.keystore --keystore-pass="\$MAKEPAD_KEYSTORE_PASS" \\
    build-aab -p kumkaj-courier --release

Default Owner PIN after install: 0000 — change under More → Team.
iOS / macOS App Store: deferred (see package-apple-scaffold.sh).
EOF

echo "Makepad: $MAKEPAD_ROOT"
echo "ANDROID_HOME: $ANDROID_HOME"

if [[ ! -d "$ANDROID_HOME" ]]; then
  echo "Android SDK not found at ANDROID_HOME — scaffold only."
  echo "Android notes: $OUT/README.txt"
  exit 0
fi

ensure_android_toolchain "$ANDROID_HOME"
"$ROOT/scripts/apply-android-activity-bridge.sh"

CARGO_MP=(cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release --)

LOG="$OUT/build.log"
set +e
"${CARGO_MP[@]}" android "${ANDROID_PKG_OPTS[@]}" build -p kumkaj-courier --release 2>&1 | tee "$LOG"
STATUS=${PIPESTATUS[0]}
set -e

# shellcheck source=cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cd "$ROOT" && cargo_target_dir)"
# Prefer the final signed APK; avoid copying the same file onto itself.
mapfile -t APKS < <(find "$TARGET_DIR" -name 'kumkaj-courier.apk' 2>/dev/null | head -5 || true)
if [[ ${#APKS[@]} -eq 0 ]]; then
  mapfile -t APKS < <(find "$TARGET_DIR" -name '*.apk' 2>/dev/null | head -10 || true)
fi
declare -A SEEN=()
for art in "${APKS[@]:-}"; do
  [[ -n "$art" && -f "$art" ]] || continue
  base="$(basename "$art")"
  dest="$OUT/$base"
  [[ -n "${SEEN[$base]:-}" ]] && continue
  SEEN[$base]=1
  if [[ "$(realpath "$art")" == "$(realpath -m "$dest")" ]]; then
    echo "Already in place → $dest"
    continue
  fi
  cp -f "$art" "$dest"
  echo "Copied → $dest"
done

set +e
"${CARGO_MP[@]}" android "${ANDROID_PKG_OPTS[@]}" build-aab -p kumkaj-courier --release 2>&1 | tee -a "$LOG"
set -e
mapfile -t AABS < <(find "$TARGET_DIR" -name '*.aab' 2>/dev/null | head -10 || true)
for art in "${AABS[@]:-}"; do
  [[ -n "$art" && -f "$art" ]] || continue
  dest="$OUT/$(basename "$art")"
  if [[ "$(realpath "$art" 2>/dev/null || echo "$art")" == "$(realpath -m "$dest")" ]]; then
    continue
  fi
  cp -f "$art" "$dest"
  echo "Copied → $dest"
done
cp -f "$ROOT/web/play-store-listing.txt" "$OUT/PLAY_STORE_LISTING.txt" 2>/dev/null || true

if [[ $STATUS -eq 0 ]]; then
  echo "Android build succeeded — see $OUT/ and $LOG"
else
  echo "Android build did not complete (exit $STATUS). See $LOG and README.txt"
fi
echo "Android package notes: $OUT/README.txt"
exit "$STATUS"
