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
#
# Release signing. An Android update installs only over an app signed with
# the same key, and uninstalling to get past a key change deletes the books
# on that device — so packages for shops are signed with the app's own
# release key. cargo-makepad's `build` always signs with Makepad's bundled
# debug key; android_release_sign re-signs the APK in place.
#
# Defaults (no env required when these files exist) — same idea as Ghalbol
# ($HOME/ghalbol-release.jks):
#   $HOME/<app>-release.jks            production / Play upload keystore
#   $HOME/<app>-release.jks.password   store+key password (mode 600, one line)
#   $HOME/<app>-release.jks.makepad    alias sidecar (optional; else "upload")
# where <app> is the lowercased product label (khaynjo, moklanjo, …) from
# KUMKAJ_APP_LABEL, else the basename of KUMKAJ_APP_DIR.
#
# Debug fingerprint helper default (Android SDK):
#   $HOME/.android/debug.keystore  (alias androiddebugkey)
#
# Optional overrides:
#   KUMKAJ_ANDROID_KEYSTORE       keystore path
#   KUMKAJ_ANDROID_KEYSTORE_PASS  password (else read from <keystore>.password)
#   KUMKAJ_ANDROID_KEY_ALIAS      default: sibling <path>.makepad alias, else "upload"
#
# Keep the keystore and its password file outside the repo, backed up: a lost
# key means no more updates for installed devices.

# Resolved path for the production/upload keystore (file need not exist yet).
android_release_keystore_path() {
  if [[ -n "${KUMKAJ_ANDROID_KEYSTORE:-}" ]]; then
    printf '%s\n' "$KUMKAJ_ANDROID_KEYSTORE"
    return 0
  fi
  local slug=""
  if [[ -n "${KUMKAJ_APP_LABEL:-}" ]]; then
    slug="$(printf '%s' "$KUMKAJ_APP_LABEL" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
  elif [[ -n "${KUMKAJ_APP_DIR:-}" ]]; then
    slug="$(basename "$KUMKAJ_APP_DIR")"
  else
    slug="kumkaj"
  fi
  printf '%s\n' "${HOME}/${slug}-release.jks"
}

# True when a usable release keystore file is available; exports
# KUMKAJ_ANDROID_KEYSTORE to the resolved path for callers.
android_release_keystore_set() {
  local ks
  ks="$(android_release_keystore_path)"
  export KUMKAJ_ANDROID_KEYSTORE="$ks"
  [[ -f "$ks" ]]
}

# Password from env, else sibling `<keystore>.password` (trim trailing newlines).
android_release_keystore_pass() {
  if [[ -n "${KUMKAJ_ANDROID_KEYSTORE_PASS:-}" ]]; then
    printf '%s\n' "$KUMKAJ_ANDROID_KEYSTORE_PASS"
    return 0
  fi
  local ks passfile
  ks="${KUMKAJ_ANDROID_KEYSTORE:-$(android_release_keystore_path)}"
  passfile="${ks}.password"
  if [[ -f "$passfile" ]]; then
    # Single-line password file; strip a trailing newline only.
    tr -d '\r' <"$passfile" | sed -z 's/\n$//'
    return 0
  fi
  return 1
}

android_key_alias() {
  local alias="${KUMKAJ_ANDROID_KEY_ALIAS:-}"
  local ks="${KUMKAJ_ANDROID_KEYSTORE:-$(android_release_keystore_path)}"
  if [[ -z "$alias" && -f "${ks}.makepad" ]]; then
    alias="$(sed -n 's/^[[:space:]]*alias[[:space:]]*=[[:space:]]*"\{0,1\}\([^"]*\)"\{0,1\}[[:space:]]*$/\1/p' "${ks}.makepad" | head -1)"
  fi
  echo "${alias:-upload}"
}

android_release_sign() {
  local apk="$1" sdk="$2"
  if ! android_release_keystore_set; then
    echo "NOTE: $(basename "$apk") is signed with Makepad's debug key — fine to try out," >&2
    echo "      not for shops. Create \$HOME/<app>-release.jks and \$HOME/<app>-release.jks.password" >&2
    echo "      (e.g. khaynjo-release.jks) — or set KUMKAJ_ANDROID_KEYSTORE / _PASS." >&2
    return 0
  fi
  local pass
  if ! pass="$(android_release_keystore_pass)"; then
    echo "Release keystore found at $KUMKAJ_ANDROID_KEYSTORE but no password:" >&2
    echo "  put it in ${KUMKAJ_ANDROID_KEYSTORE}.password (mode 600) or set KUMKAJ_ANDROID_KEYSTORE_PASS" >&2
    return 1
  fi
  local java="$sdk/openjdk/bin/java"
  [[ -x "$java" ]] || java="$sdk/openjdk/jdk-17.0.2/bin/java"
  local signer="$sdk/build-tools/33.0.1/lib/apksigner.jar"
  # The password goes through the environment, never the command line.
  KUMKAJ_KS_PASS="$pass" "$java" -jar "$signer" sign \
    --ks "$KUMKAJ_ANDROID_KEYSTORE" --ks-key-alias "$(android_key_alias)" \
    --ks-pass env:KUMKAJ_KS_PASS --key-pass env:KUMKAJ_KS_PASS "$apk"
  "$java" -jar "$signer" verify --print-certs "$apk" | grep -m1 "certificate DN" || true
  echo "Signed with the release key: $apk ($KUMKAJ_ANDROID_KEYSTORE)"
}

# Signing options for build-aab: the release key when present (password through
# MAKEPAD_KEYSTORE_PASS), else nothing — cargo-makepad then uses its debug
# key, which Play rejects.
android_aab_sign_opts() {
  ANDROID_AAB_SIGN_OPTS=()
  if android_release_keystore_set; then
    local pass
    if ! pass="$(android_release_keystore_pass)"; then
      echo "Release keystore found at $KUMKAJ_ANDROID_KEYSTORE but no password:" >&2
      echo "  put it in ${KUMKAJ_ANDROID_KEYSTORE}.password (mode 600) or set KUMKAJ_ANDROID_KEYSTORE_PASS" >&2
      return 1
    fi
    export MAKEPAD_KEYSTORE_PASS="$pass"
    ANDROID_AAB_SIGN_OPTS=(--keystore="$KUMKAJ_ANDROID_KEYSTORE" --keystore-key-alias="$(android_key_alias)")
  fi
}
