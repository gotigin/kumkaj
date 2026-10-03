#!/usr/bin/env bash
# Build, install, and launch Kumkaj on a connected Android device/emulator.
# Usage: ./scripts/kumkaj-android-dev.sh
#    or: cargo kumkaj-android-dev
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=android-env.sh
source "$ROOT/scripts/android-env.sh"

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

echo "Makepad: $MAKEPAD_ROOT"
echo "ANDROID_HOME: $ANDROID_HOME"
echo "Building + installing + launching (release)…"
echo "Note: Android has no Splash --hot; each run rebuilds and relaunches."
echo "If adb says unauthorized: unlock the phone and accept the RSA prompt."

exec cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \
  android "${ANDROID_PKG_OPTS[@]}" run -p kumkaj --release "$@"
