#!/usr/bin/env bash
# Shared Android env for package-android.sh / kumkaj-android-dev.sh.
# shellcheck disable=SC2034

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

resolve_makepad_root() {
  if [[ -n "${MAKEPAD_ROOT:-}" && -d "$MAKEPAD_ROOT/tools/cargo_makepad" ]]; then
    return 0
  fi
  MAKEPAD_ROOT=""
  local c
  for c in \
    "$ROOT/vendor/makepad-full" \
    "$ROOT/../makepad" \
    "$HOME/.cargo/git/checkouts/makepad-ec2f134f34cd9f98/915ce7c" \
    "$HOME/.cargo/git/checkouts/makepad-ec2f134f34cd9f98/a4ea253"; do
    if [[ -d "$c/tools/cargo_makepad" ]]; then
      MAKEPAD_ROOT="$c"
      break
    fi
  done
  if [[ -z "$MAKEPAD_ROOT" && -d "$HOME/.cargo/git/checkouts/makepad-ec2f134f34cd9f98" ]]; then
    while IFS= read -r c; do
      if [[ -d "$c/tools/cargo_makepad" ]]; then
        MAKEPAD_ROOT="$c"
        break
      fi
    done < <(ls -1dt "$HOME/.cargo/git/checkouts/makepad-ec2f134f34cd9f98"/* 2>/dev/null || true)
  fi
  if [[ -z "${MAKEPAD_ROOT:-}" || ! -d "$MAKEPAD_ROOT/tools/cargo_makepad" ]]; then
    echo "Cloning Makepad main into vendor/makepad-full for cargo-makepad…"
    mkdir -p "$ROOT/vendor"
    if [[ ! -d "$ROOT/vendor/makepad-full/.git" ]]; then
      git clone --depth 1 --branch main https://github.com/makepad/makepad.git "$ROOT/vendor/makepad-full"
    fi
    MAKEPAD_ROOT="$ROOT/vendor/makepad-full"
  fi
}

# cargo-makepad installs INTO --sdk-path (ANDROID_HOME), not under tools/cargo_makepad.
# These match ANDROID_SDK_URLS_33 in vendor/makepad-full/tools/cargo_makepad/src/android/sdk.rs
android_toolchain_ready() {
  local sdk="$1"
  [[ -x "$sdk/platform-tools/adb" ]] \
    && [[ -d "$sdk/build-tools/33.0.1" ]] \
    && [[ -d "$sdk/platforms/android-33-ext4" || -d "$sdk/platforms/android-33" ]] \
    && [[ -d "$sdk/ndk/28.2.13676358" ]] \
    && [[ -x "$sdk/openjdk/bin/java" || -x "$sdk/openjdk/jdk-17.0.2/bin/java" ]] \
    && [[ -f "$sdk/bundletool/bundletool.jar" ]]
}

ensure_rust_android_target() {
  if ! rustup target list --installed 2>/dev/null | grep -qx 'aarch64-linux-android'; then
    echo "Adding rustup target aarch64-linux-android…"
    rustup target add aarch64-linux-android
  fi
}

# Only call cargo-makepad install-toolchain when ANDROID_HOME is missing pieces.
# That command always re-downloads zips — never run it "just in case".
ensure_android_toolchain() {
  local sdk="$1"
  ensure_rust_android_target
  if android_toolchain_ready "$sdk"; then
    echo "Android toolchain already present under $sdk — skipping download."
    return 0
  fi
  echo "Android toolchain incomplete under $sdk — running install-toolchain once…"
  echo "  (missing one of: platform-tools/adb, build-tools/33.0.1, platforms/android-33*, ndk/28.2.13676358, openjdk, bundletool)"
  cargo run --manifest-path "$MAKEPAD_ROOT/tools/cargo_makepad/Cargo.toml" --release -- \
    android --sdk-path="$sdk" install-toolchain
}

android_pkg_opts() {
  local sdk="$1"
  # Prefer already-loaded KUMKAJ_APP_* from load_kumkaj_env; fall back to production ids.
  local pkg="${KUMKAJ_APP_ID:-com.kumkaj.app}"
  local label="${KUMKAJ_APP_LABEL:-Kumkaj}"
  ANDROID_PKG_OPTS=(
    --package-name="$pkg"
    --app-label="$label"
    --abi=aarch64
    --sdk-path="$sdk"
  )
  echo "Android package: $pkg ($label)"
}
