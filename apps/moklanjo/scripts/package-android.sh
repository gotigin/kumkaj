#!/usr/bin/env bash
# Android packaging via cargo-makepad (requires Android SDK/NDK toolchain):
# dist/android/kumkaj-moklanjo.apk with the production app id. Signed with
# the release key when KUMKAJ_ANDROID_KEYSTORE (+ _PASS) is set (see
# scripts/android-env.sh), which also builds the Play bundle
# dist/android/kumkaj-moklanjo.aab.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
export KUMKAJ_APP_DIR="$ROOT/apps/moklanjo"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=../../../scripts/android-env.sh
source "$ROOT/scripts/android-env.sh"

export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production

OUT="$ROOT/dist/android"
mkdir -p "$OUT"

resolve_makepad_root
export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"

android_pkg_opts "$ANDROID_HOME"
android_release_opts

cat > "$OUT/README.txt" <<EOF
Kumkaj Android packaging
========================
Application id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Website: https://kumkaj.com
Env file: apps/moklanjo/env/production.env

Build APK:
  MAKEPAD_ROOT=$MAKEPAD_ROOT ./apps/moklanjo/scripts/package-android.sh

Live install/run on device (development id com.moklanjo.debug):
  cargo moklanjo-android-dev

Release key (once; keep the file and password backed up, outside the repo —
a lost key means installed devices can never be updated):
  cargo run --manifest-path "\$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \\
    android --sdk-path="\$ANDROID_HOME" keystore-create ~/keys/kumkaj-moklanjo.keystore
Then every release build, signed APK + Play bundle (AAB):
  KUMKAJ_ANDROID_KEYSTORE=~/keys/kumkaj-moklanjo.keystore KUMKAJ_ANDROID_KEYSTORE_PASS=… \\
    ./apps/moklanjo/scripts/package-android.sh

An update installs over the last one and keeps the books — never uninstall
to update (that deletes the books on the device).
Any device can be the office's Server or a device paired to it: Windows,
Linux or Android. First start asks for the Owner PIN (Server Mode) — there
is no default PIN to change.
iOS / macOS App Store: deferred (see scripts/package-apple-scaffold.sh apps/moklanjo).
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
fix_makepad_android_deprecations

CARGO_MP=(cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release --)

LOG="$OUT/build.log"
set +e
"${CARGO_MP[@]}" android "${ANDROID_PKG_OPTS[@]}" build -p kumkaj-moklanjo --release 2>&1 | tee "$LOG"
STATUS=${PIPESTATUS[0]}
set -e

# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cd "$ROOT" && cargo_target_dir)"
# This build's APK: named after the production label (not the dev one, not
# another app's or ABI's leftover).
APK="$TARGET_DIR/android/makepad-android-apk/kumkaj_moklanjo/apk/$(echo "$KUMKAJ_APP_LABEL" | tr -d ' ' | tr '[:upper:]' '[:lower:]').apk"
rm -f "$OUT/kumkaj-moklanjo.apk" "$OUT/kumkaj-moklanjo.aab"
if [[ $STATUS -eq 0 && -f "$APK" ]]; then
  cp -f "$APK" "$OUT/kumkaj-moklanjo.apk"
  echo "APK → $OUT/kumkaj-moklanjo.apk"
  android_release_sign "$OUT/kumkaj-moklanjo.apk" "$ANDROID_HOME"
  if android_release_keystore_set; then
    android_aab_sign_opts
    set +e
    "${CARGO_MP[@]}" android "${ANDROID_PKG_OPTS[@]}" "${ANDROID_AAB_SIGN_OPTS[@]}" build-aab -p kumkaj-moklanjo --release 2>&1 | tee -a "$LOG"
    STATUS=${PIPESTATUS[0]}
    set -e
    AAB="$(find "$TARGET_DIR/android/makepad-android-aab/kumkaj_moklanjo" -maxdepth 1 -name '*.aab' 2>/dev/null | head -1 || true)"
    if [[ $STATUS -eq 0 && -n "$AAB" ]]; then
      cp -f "$AAB" "$OUT/kumkaj-moklanjo.aab"
      echo "Play bundle → $OUT/kumkaj-moklanjo.aab"
    fi
  fi
elif [[ $STATUS -eq 0 ]]; then
  echo "Built, but $APK is missing" >&2
  STATUS=1
fi
cp -f "$ROOT/docs/LAN_QUICK_START.md" "$OUT/Quick start.txt"
cp -f "$ROOT/web/play-store-listing.txt" "$OUT/PLAY_STORE_LISTING.txt" 2>/dev/null || true

if [[ $STATUS -eq 0 ]]; then
  echo "Android build succeeded — see $OUT/ and $LOG"
else
  echo "Android build did not complete (exit $STATUS). See $LOG and README.txt"
fi
echo "Android package notes: $OUT/README.txt"
exit "$STATUS"
