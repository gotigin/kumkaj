#!/usr/bin/env bash
# Insert Kumkaj's shareText / readContentUri into cargo-makepad's activity.
# The Java that ships in the APK lives in MAKEPAD_ROOT, not in vendor/makepad
# (setup-makepad.sh does not copy tools/). Idempotent. Bash only (no Python).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=android-env.sh
source "$ROOT/scripts/android-env.sh"
resolve_makepad_root
JAVA="$MAKEPAD_ROOT/tools/cargo_makepad/src/android/java/dev/makepad/android/MakepadActivity.java"
SNIPPET="$ROOT/patches/makepad/MakepadActivity-share.java"
if [[ ! -f "$JAVA" ]]; then
  echo "MakepadActivity.java not found under $MAKEPAD_ROOT" >&2
  exit 1
fi
if grep -q 'public void shareText' "$JAVA" && grep -q 'public byte\[\] readContentUri' "$JAVA"; then
  echo "Android share/content bridge already present."
  exit 0
fi

NEEDLE='    public void copyToClipboard(String content) {'
if ! grep -qF "$NEEDLE" "$JAVA"; then
  echo "copyToClipboard anchor missing in MakepadActivity.java" >&2
  exit 1
fi

tmp="$(mktemp)"
cp -f "$JAVA" "$tmp"

# Add imports once (after Activity import).
ensure_import() {
  local line="$1"
  if ! grep -qF "$line" "$tmp"; then
    sed -i "s|import android.app.Activity;|import android.app.Activity;\n${line}|" "$tmp"
  fi
}
ensure_import 'import android.content.Intent;'
ensure_import 'import android.net.Uri;'
ensure_import 'import java.io.ByteArrayOutputStream;'
ensure_import 'import java.io.InputStream;'

# Insert snippet immediately before copyToClipboard.
{
  # Everything before the needle
  awk -v needle="$NEEDLE" '
    index($0, needle) == 1 { exit }
    { print }
  ' "$tmp"
  cat "$SNIPPET"
  # Needle and the rest
  awk -v needle="$NEEDLE" '
    found || index($0, needle) == 1 { found = 1; print }
  ' "$tmp"
} > "${tmp}.out"
mv -f "${tmp}.out" "$tmp"
mv -f "$tmp" "$JAVA"
echo "Inserted shareText and readContentUri into $JAVA"
