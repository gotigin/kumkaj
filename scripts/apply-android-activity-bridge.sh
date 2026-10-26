#!/usr/bin/env bash
# Insert Kumkaj's activity methods into cargo-makepad's activity: shareText /
# readContentUri (MakepadActivity-share.java), keepScreenOn
# (MakepadActivity-keep-awake.java) and writeContentUri
# (MakepadActivity-write.java). Each snippet goes in only when its method
# is missing, so a checkout bridged before a snippet existed still gets it.
# The Java that ships in the APK lives in MAKEPAD_ROOT, not in vendor/makepad
# (setup-makepad.sh does not copy tools/). Idempotent. Bash only (no Python).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=android-env.sh
source "$ROOT/scripts/android-env.sh"
resolve_makepad_root
JAVA="$MAKEPAD_ROOT/tools/cargo_makepad/src/android/java/dev/makepad/android/MakepadActivity.java"
if [[ ! -f "$JAVA" ]]; then
  echo "MakepadActivity.java not found under $MAKEPAD_ROOT" >&2
  exit 1
fi

NEEDLE='    public void copyToClipboard(String content) {'
if ! grep -qF "$NEEDLE" "$JAVA"; then
  echo "copyToClipboard anchor missing in MakepadActivity.java" >&2
  exit 1
fi

# Add an import once (after the Activity import).
ensure_import() {
  local line="$1"
  if ! grep -qF "$line" "$JAVA"; then
    sed -i "s|import android.app.Activity;|import android.app.Activity;\n${line}|" "$JAVA"
  fi
}

# Insert snippet file $1 immediately before copyToClipboard.
insert_snippet() {
  local snippet="$1" tmp
  tmp="$(mktemp)"
  {
    # Everything before the needle
    awk -v needle="$NEEDLE" '
      index($0, needle) == 1 { exit }
      { print }
    ' "$JAVA"
    cat "$snippet"
    # Needle and the rest
    awk -v needle="$NEEDLE" '
      found || index($0, needle) == 1 { found = 1; print }
    ' "$JAVA"
  } > "$tmp"
  mv -f "$tmp" "$JAVA"
}

if grep -q 'public void shareText' "$JAVA" && grep -q 'public byte\[\] readContentUri' "$JAVA"; then
  echo "Android share/content bridge already present."
else
  ensure_import 'import android.content.Intent;'
  ensure_import 'import android.net.Uri;'
  ensure_import 'import java.io.ByteArrayOutputStream;'
  ensure_import 'import java.io.InputStream;'
  insert_snippet "$ROOT/patches/makepad/MakepadActivity-share.java"
  echo "Inserted shareText and readContentUri into $JAVA"
fi

if grep -q 'public void keepScreenOn' "$JAVA"; then
  echo "Android keep-awake bridge already present."
else
  insert_snippet "$ROOT/patches/makepad/MakepadActivity-keep-awake.java"
  echo "Inserted keepScreenOn into $JAVA"
fi

if grep -q 'public boolean writeContentUri' "$JAVA"; then
  echo "Android write-to-picked-file bridge already present."
else
  insert_snippet "$ROOT/patches/makepad/MakepadActivity-write.java"
  echo "Inserted writeContentUri into $JAVA"
fi
