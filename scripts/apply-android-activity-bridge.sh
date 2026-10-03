#!/usr/bin/env bash
# Insert Kumkaj's shareText / readContentUri into cargo-makepad's activity.
# The Java that ships in the APK lives in MAKEPAD_ROOT, not in vendor/makepad
# (setup-makepad.sh does not copy tools/). Idempotent.
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
python3 - "$JAVA" "$SNIPPET" <<'PY'
import pathlib, sys
java = pathlib.Path(sys.argv[1])
snippet = pathlib.Path(sys.argv[2]).read_text()
text = java.read_text()
needle = "    public void copyToClipboard(String content) {"
if needle not in text:
    sys.exit("copyToClipboard anchor missing in MakepadActivity.java")
# Imports the methods use. Add them once if this activity predates them.
for line in (
    "import android.content.Intent;\n",
    "import android.net.Uri;\n",
    "import java.io.ByteArrayOutputStream;\n",
    "import java.io.InputStream;\n",
):
    if line not in text:
        text = text.replace("import android.app.Activity;\n", "import android.app.Activity;\n" + line, 1)
java.write_text(text.replace(needle, snippet + needle, 1))
print("Inserted shareText and readContentUri into", java)
PY
