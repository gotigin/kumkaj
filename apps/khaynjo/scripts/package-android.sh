#!/usr/bin/env bash
# Khaynjo production APK for Android phones and tablets — the shop's Server
# or a device paired to one: dist/khaynjo-android/kumkaj-khaynjo.apk with the
# production app id. No device needed. For live install on a connected
# device use android-dev.sh (development app id).
#   apps/khaynjo/scripts/package-android.sh
# Signed with the release key when KUMKAJ_ANDROID_KEYSTORE (+ _PASS) is set
# (see scripts/android-env.sh), which also builds the Play bundle
# dist/khaynjo-android/khaynjo.aab.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
export KUMKAJ_APP_DIR="$ROOT/apps/khaynjo"
export KUMKAJ_ENV_DIR="$KUMKAJ_APP_DIR/env"
# shellcheck source=../../../scripts/load-kumkaj-env.sh
source "$ROOT/scripts/load-kumkaj-env.sh"
# shellcheck source=../../../scripts/android-env.sh
source "$ROOT/scripts/android-env.sh"
export KUMKAJ_ROOT="$ROOT"
load_kumkaj_env production
resolve_makepad_root
export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
if [[ ! -d "$ANDROID_HOME" ]]; then
  echo "Android SDK not found. Set ANDROID_HOME (tried: $ANDROID_HOME)" >&2
  exit 1
fi
ensure_android_toolchain "$ANDROID_HOME"
"$ROOT/scripts/apply-android-activity-bridge.sh"
android_pkg_opts "$ANDROID_HOME"
android_release_opts

OUT="$ROOT/dist/khaynjo-android"
mkdir -p "$OUT"
LOG="$OUT/build.log"
echo "Makepad: $MAKEPAD_ROOT"
echo "ANDROID_HOME: $ANDROID_HOME"
echo "Building ${KUMKAJ_APP_ID} (${KUMKAJ_APP_LABEL}) release APK…"
set +e
cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \
  android "${ANDROID_PKG_OPTS[@]}" build -p kumkaj-khaynjo --release 2>&1 | tee "$LOG"
STATUS=${PIPESTATUS[0]}
set -e

# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cargo_target_dir)"
# This build's APK: named after the production label (not the dev one,
# not another ABI's leftover).
APK="$TARGET_DIR/android/makepad-android-apk/kumkaj_khaynjo/apk/$(echo "$KUMKAJ_APP_LABEL" | tr -d ' ' | tr '[:upper:]' '[:lower:]').apk"
if [[ $STATUS -ne 0 || ! -f "$APK" ]]; then
  echo "Android build did not complete (exit $STATUS). See $LOG" >&2
  exit 1
fi
cp -f "$APK" "$OUT/kumkaj-khaynjo.apk"
android_release_sign "$OUT/kumkaj-khaynjo.apk" "$ANDROID_HOME"
SIGNED="Makepad's debug key — for trying out only; a shop's devices need the release key"
rm -f "$OUT/khaynjo.aab"
if android_release_keystore_set; then
  SIGNED="the Khaynjo release key"
  android_aab_sign_opts
  cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \
    android "${ANDROID_PKG_OPTS[@]}" "${ANDROID_AAB_SIGN_OPTS[@]}" build-aab -p kumkaj-khaynjo --release 2>&1 | tee -a "$LOG"
  AAB="$(find "$TARGET_DIR/android/makepad-android-aab/kumkaj_khaynjo" -maxdepth 1 -name '*.aab' 2>/dev/null | head -1 || true)"
  if [[ -z "$AAB" ]]; then
    echo "Play bundle build did not complete. See $LOG" >&2
    exit 1
  fi
  cp -f "$AAB" "$OUT/khaynjo.aab"
  echo "Play bundle → $OUT/khaynjo.aab"
fi
cat > "$OUT/README.txt" <<EOF
Khaynjo for Android phones and tablets
======================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Signed with: ${SIGNED}

Install: copy kumkaj-khaynjo.apk to the device and open it (allow installs
from this source), or: adb install -r kumkaj-khaynjo.apk
An update installs over the last one and keeps the books — never uninstall
to update (that deletes the books on the device).

At launch pick what this device is:
  Server Mode — it holds the shop's books (keep the app open, plugged in).
  Client Mode — Find the shop's server → type the pairing code shown on the
  Server (Settings → Extra tills → Allow extra tills) → sign in with your PIN.
The Server can be any device running Khaynjo: Windows, Linux or Android.
First start in Server Mode asks for the Owner PIN — there is no default PIN.
EOF
cp -f "$ROOT/docs/LAN_QUICK_START.md" "$OUT/Quick start.txt"
echo "APK → $OUT/kumkaj-khaynjo.apk"
