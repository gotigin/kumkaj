#!/usr/bin/env bash
# Shared Android env for each app's package-android.sh / android-dev.sh.
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

# Real API migrations (associated MAX/MIN, Atomic::try_update) on the
# cargo-makepad tree and the app path-dep — not allow(deprecated).
fix_makepad_android_deprecations() {
  local app_makepad="$ROOT/vendor/makepad"
  if [[ -d "$app_makepad" && "$MAKEPAD_ROOT" != "$app_makepad" ]]; then
    "$ROOT/scripts/fix-makepad-rust-deprecations.sh" "$MAKEPAD_ROOT" "$app_makepad"
  else
    "$ROOT/scripts/fix-makepad-rust-deprecations.sh" "$MAKEPAD_ROOT"
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
  # The app's ids, from load_kumkaj_env (no default: never another app's id).
  local pkg="${KUMKAJ_APP_ID:?load_kumkaj_env first (KUMKAJ_APP_ID)}"
  local label="${KUMKAJ_APP_LABEL:?load_kumkaj_env first (KUMKAJ_APP_LABEL)}"
  ANDROID_PKG_OPTS=(
    --package-name="$pkg"
    --app-label="$label"
    --abi=aarch64
    --sdk-path="$sdk"
  )
  echo "Android package: $pkg ($label)"
}

# Release builds: a version code that always goes up (YYYYMMDDHH), so each
# package installs over the last one and Play accepts it.
android_release_opts() {
  ANDROID_PKG_OPTS+=(--version-code=auto)
}

# Release signing. An Android update installs only over an app signed with
# the same key, and uninstalling to get past a key change deletes the books
# on that device — so packages for shops are signed with the app's own
# release key. cargo-makepad's `build` always signs with Makepad's bundled
# debug key; android_release_sign re-signs the APK in place.
#   KUMKAJ_ANDROID_KEYSTORE       keystore file, made once per app with
#                                 cargo makepad android keystore-create <path>
#   KUMKAJ_ANDROID_KEYSTORE_PASS  its password
#   KUMKAJ_ANDROID_KEY_ALIAS      key alias (default: the keystore's
#                                 <path>.makepad sidecar, else "upload")
# Keep the keystore and its password outside the repo, backed up: a lost key
# means no more updates for installed devices.
android_release_keystore_set() {
  [[ -n "${KUMKAJ_ANDROID_KEYSTORE:-}" ]]
}

android_key_alias() {
  local alias="${KUMKAJ_ANDROID_KEY_ALIAS:-}"
  if [[ -z "$alias" && -f "$KUMKAJ_ANDROID_KEYSTORE.makepad" ]]; then
    alias="$(sed -n 's/^[[:space:]]*alias[[:space:]]*=[[:space:]]*"\{0,1\}\([^"]*\)"\{0,1\}[[:space:]]*$/\1/p' "$KUMKAJ_ANDROID_KEYSTORE.makepad" | head -1)"
  fi
  echo "${alias:-upload}"
}

android_release_sign() {
  local apk="$1" sdk="$2"
  if ! android_release_keystore_set; then
    echo "NOTE: $(basename "$apk") is signed with Makepad's debug key — fine to try out," >&2
    echo "      not for shops: set KUMKAJ_ANDROID_KEYSTORE and KUMKAJ_ANDROID_KEYSTORE_PASS" >&2
    echo "      so every later update installs over this one." >&2
    return 0
  fi
  if [[ ! -f "$KUMKAJ_ANDROID_KEYSTORE" ]]; then
    echo "KUMKAJ_ANDROID_KEYSTORE not found: $KUMKAJ_ANDROID_KEYSTORE" >&2
    return 1
  fi
  : "${KUMKAJ_ANDROID_KEYSTORE_PASS:?set KUMKAJ_ANDROID_KEYSTORE_PASS for $KUMKAJ_ANDROID_KEYSTORE}"
  local java="$sdk/openjdk/bin/java"
  [[ -x "$java" ]] || java="$sdk/openjdk/jdk-17.0.2/bin/java"
  local signer="$sdk/build-tools/33.0.1/lib/apksigner.jar"
  # The password goes through the environment, never the command line.
  KUMKAJ_KS_PASS="$KUMKAJ_ANDROID_KEYSTORE_PASS" "$java" -jar "$signer" sign \
    --ks "$KUMKAJ_ANDROID_KEYSTORE" --ks-key-alias "$(android_key_alias)" \
    --ks-pass env:KUMKAJ_KS_PASS --key-pass env:KUMKAJ_KS_PASS "$apk"
  "$java" -jar "$signer" verify --print-certs "$apk" | grep -m1 "certificate DN" || true
  echo "Signed with the release key: $apk"
}

# Signing options for build-aab: the release key when set (password through
# MAKEPAD_KEYSTORE_PASS), else nothing — cargo-makepad then uses its debug
# key, which Play rejects.
android_aab_sign_opts() {
  ANDROID_AAB_SIGN_OPTS=()
  if android_release_keystore_set; then
    : "${KUMKAJ_ANDROID_KEYSTORE_PASS:?set KUMKAJ_ANDROID_KEYSTORE_PASS for $KUMKAJ_ANDROID_KEYSTORE}"
    export MAKEPAD_KEYSTORE_PASS="$KUMKAJ_ANDROID_KEYSTORE_PASS"
    ANDROID_AAB_SIGN_OPTS=(--keystore="$KUMKAJ_ANDROID_KEYSTORE" --keystore-key-alias="$(android_key_alias)")
  fi
}
