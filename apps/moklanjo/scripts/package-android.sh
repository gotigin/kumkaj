#!/usr/bin/env bash
# Moklanjo production APK for Android phones and tablets — the office's
# Server or a device paired to one: dist/moklanjo-android/kumkaj-moklanjo.apk
# with the production app id. No device needed. For live install on a
# connected device use android-dev.sh (development app id).
#   apps/moklanjo/scripts/package-android.sh
# Signed with the release key when $HOME/moklanjo-release.jks exists and the
# password is in $HOME/moklanjo-release.jks.password (or KUMKAJ_ANDROID_*
# overrides) — see scripts/android-env.sh / docs/PUBLISH_PLAY_STORE.md.
# Also builds the Play bundle dist/moklanjo-android/kumkaj-moklanjo.aab.
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

# The app's own folder, like the other three apps — it is where the Play guide
# and scripts/product-cert.sh look (this script wrote dist/android until
# 2026-11-21, and the cert went on testing an older hand-made copy).
OUT="$ROOT/dist/moklanjo-android"
# Stage then swap on success — like MSI overwriting dist/Moklanjo-*.msi
# (failed builds keep the previous good package).
STAGE="$ROOT/dist/.moklanjo-android-staging.$$"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cleanup_stage() { rm -rf "$STAGE"; }
trap cleanup_stage EXIT

resolve_makepad_root
export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"

android_pkg_opts "$ANDROID_HOME"
android_release_opts
SIGNED="Makepad's debug key — for trying out only; an office's devices need the release key"
if android_release_keystore_set; then
  SIGNED="the Moklanjo release key"
fi

cat > "$STAGE/README.txt" <<EOF
Moklanjo for Android phones and tablets
=======================================
App id: ${KUMKAJ_APP_ID}
Label: ${KUMKAJ_APP_LABEL}
Signed with: ${SIGNED}
Website: https://moklanjo.com
Env file: apps/moklanjo/env/production.env

Build APK:
  ./apps/moklanjo/scripts/package-android.sh

Live install/run on device (development id com.moklanjo.debug):
  cargo moklanjo-android-dev

Release key: \$HOME/moklanjo-release.jks, its password in
\$HOME/moklanjo-release.jks.password (one line, mode 600). The package script
finds both by itself, signs the APK with them and also builds the Play bundle
(kumkaj-moklanjo.aab). Keep both backed up outside the repo — a lost key means
installed devices can never be updated.

Play Console (with the .aab): also upload native-debug-symbols.zip and
mapping.txt from this folder — see apps/moklanjo/docs/PUBLISH_PLAY_STORE.md.

Install: copy kumkaj-moklanjo.apk to the device and open it (allow installs
from this source), or: adb install -r kumkaj-moklanjo.apk
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
  # Publish the notes so the folder exists — never over a built package.
  if [[ ! -d "$OUT" ]]; then
    mv "$STAGE" "$OUT"
    trap - EXIT
    echo "Android notes: $OUT/README.txt"
  else
    echo "Kept the package already in $OUT."
  fi
  exit 0
fi

ensure_android_toolchain "$ANDROID_HOME"
"$ROOT/scripts/apply-android-activity-bridge.sh"
fix_makepad_android_deprecations

CARGO_MP=(cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release --)

LOG="$STAGE/build.log"
set +e
"${CARGO_MP[@]}" android "${ANDROID_PKG_OPTS[@]}" build -p kumkaj-moklanjo --release 2>&1 | tee "$LOG"
STATUS=${PIPESTATUS[0]}
set -e

# shellcheck source=../../../scripts/cargo-target-dir.sh
source "$ROOT/scripts/cargo-target-dir.sh"
TARGET_DIR="$(cd "$ROOT" && cargo_target_dir)"
# This build's APK: named after the production label (not the dev one, not
# another app's or ABI's leftover).
APK_DIR="$TARGET_DIR/android/makepad-android-apk/kumkaj_moklanjo/apk"
APK_LABEL="$(echo "$KUMKAJ_APP_LABEL" | tr -d ' ' | tr '[:upper:]' '[:lower:]').apk"
APK="$APK_DIR/$APK_LABEL"
if [[ ! -f "$APK" ]]; then
  APK="$(find "$APK_DIR" -maxdepth 1 -name '*.apk' -printf '%T@ %p\n' 2>/dev/null | sort -nr | head -1 | cut -d' ' -f2- || true)"
fi
if [[ $STATUS -eq 0 && -n "${APK:-}" && -f "$APK" ]]; then
  cp -f "$APK" "$STAGE/kumkaj-moklanjo.apk"
  echo "APK → $OUT/kumkaj-moklanjo.apk"
  android_release_sign "$STAGE/kumkaj-moklanjo.apk" "$ANDROID_HOME"
  if android_release_keystore_set; then
    android_aab_sign_opts
    set +e
    "${CARGO_MP[@]}" android "${ANDROID_PKG_OPTS[@]}" "${ANDROID_AAB_SIGN_OPTS[@]}" build-aab -p kumkaj-moklanjo --release 2>&1 | tee -a "$LOG"
    STATUS=${PIPESTATUS[0]}
    set -e
    AAB="$(find "$TARGET_DIR/android/makepad-android-aab/kumkaj_moklanjo" -maxdepth 1 -name '*.aab' -printf '%T@ %p\n' 2>/dev/null | sort -nr | head -1 | cut -d' ' -f2- || true)"
    if [[ $STATUS -eq 0 && -n "$AAB" ]]; then
      cp -f "$AAB" "$STAGE/kumkaj-moklanjo.aab"
      echo "Play bundle → $OUT/kumkaj-moklanjo.aab"
      android_assert_aab_launch_bg "$STAGE/kumkaj-moklanjo.aab"
      AAB_LIBS="$TARGET_DIR/android/makepad-android-aab/kumkaj_moklanjo/lib"
      android_write_play_crash_files "$AAB_LIBS" "$STAGE/kumkaj-moklanjo.aab" "$STAGE"
    fi
  fi
elif [[ $STATUS -eq 0 ]]; then
  echo "Built, but no APK under $APK_DIR" >&2
  STATUS=1
fi
cp -f "$ROOT/docs/LAN_QUICK_START.md" "$STAGE/Quick start.txt"
cp -f "$KUMKAJ_APP_DIR/web/static/play-store-listing.txt" "$STAGE/PLAY_STORE_LISTING.txt" 2>/dev/null || true

if [[ $STATUS -eq 0 ]]; then
  rm -rf "$OUT"
  mv "$STAGE" "$OUT"
  trap - EXIT
  echo "Android build succeeded — see $OUT/ and $OUT/build.log"
  [[ -f "$OUT/native-debug-symbols.zip" ]] && echo "Native debug symbols → $OUT/native-debug-symbols.zip"
  [[ -f "$OUT/mapping.txt" ]] && echo "Deobfuscation mapping → $OUT/mapping.txt"
else
  echo "Android build did not complete (exit $STATUS). See $LOG — previous $OUT kept if present."
fi
echo "Android package notes: ${OUT}/README.txt (after a successful build)"
exit "$STATUS"
