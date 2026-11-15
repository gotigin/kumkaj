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

PRINT_MARK='    // Kumkaj: PrintManager + Bluetooth SPP + USB host ESC/POS.'
if grep -q 'kumkajEscposPieceEnds' "$JAVA"; then
  echo "Android print/Bluetooth/USB bridge already present."
elif grep -q 'public boolean printDocument' "$JAVA" && ! grep -qF "$PRINT_MARK" "$JAVA"; then
  # printDocument without our marker: not ours to rewrite. Leave it.
  echo "Android print bridge present without the Kumkaj marker — not replaced; check $JAVA" >&2
else
  if grep -qF "$PRINT_MARK" "$JAVA"; then
    # An older snippet wrote each ticket in one burst (the head kept the
    # first few lines). It was inserted last, right before the needle: drop
    # it, then insert the current one below.
    tmp="$(mktemp)"
    awk -v mark="$PRINT_MARK" -v needle="$NEEDLE" '
      $0 == mark { skip = 1 }
      skip && index($0, needle) == 1 { skip = 0 }
      !skip { print }
    ' "$JAVA" > "$tmp"
    mv -f "$tmp" "$JAVA"
    echo "Removed the older print/Bluetooth/USB bridge from $JAVA"
  fi
  ensure_import 'import android.bluetooth.BluetoothAdapter;'
  ensure_import 'import android.bluetooth.BluetoothDevice;'
  ensure_import 'import android.bluetooth.BluetoothSocket;'
  ensure_import 'import android.hardware.usb.UsbConstants;'
  ensure_import 'import android.hardware.usb.UsbDevice;'
  ensure_import 'import android.hardware.usb.UsbDeviceConnection;'
  ensure_import 'import android.hardware.usb.UsbEndpoint;'
  ensure_import 'import android.hardware.usb.UsbInterface;'
  ensure_import 'import android.hardware.usb.UsbManager;'
  ensure_import 'import android.print.PrintAttributes;'
  ensure_import 'import android.print.PrintDocumentAdapter;'
  ensure_import 'import android.print.PrintDocumentInfo;'
  ensure_import 'import android.print.PrintManager;'
  ensure_import 'import java.util.UUID;'
  insert_snippet "$ROOT/patches/makepad/MakepadActivity-print.java"
  echo "Inserted printDocument / Bluetooth / USB into $JAVA"
fi

# Soft keyboard retap (API 26 / OEM): SHOW_IMPLICIT (flag 0) is ignored after
# the user dismissed the IME — first tap works, later taps do not. Prefer
# WindowInsetsController.show(ime) on API 30+; SHOW_FORCED below that.
# Docs: DEV.md bug 6d. Idempotent.
if grep -qF 'SHOW_FORCED' "$JAVA" && grep -qF 'WindowInsets.Type.ime()' "$JAVA"; then
  echo "Android IME retap showSoftInput (SHOW_FORCED / InsetsController) already present."
elif grep -qF 'imm.showSoftInput(view, 0);' "$JAVA"; then
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = """                    view.requestFocus();
                    InputMethodManager imm = (InputMethodManager)getSystemService(Context.INPUT_METHOD_SERVICE);
                    imm.showSoftInput(view, 0);"""
new = """                    view.requestFocus();
                    InputMethodManager imm = (InputMethodManager)getSystemService(Context.INPUT_METHOD_SERVICE);
                    // API 26–29: SHOW_IMPLICIT (0) is ignored after the user
                    // dismissed the IME — retaps never brought the keyboard
                    // back on tablets (Huawei AGS2 / API 26). SHOW_FORCED
                    // re-opens it. API 30+ prefers WindowInsetsController.
                    // Docs: DEV.md bug 6d.
                    if (Build.VERSION.SDK_INT >= 30) {
                        android.view.WindowInsetsController controller = getWindow().getInsetsController();
                        if (controller != null) {
                            controller.show(WindowInsets.Type.ime());
                        } else if (imm != null) {
                            imm.showSoftInput(view, InputMethodManager.SHOW_FORCED);
                        }
                    } else if (imm != null) {
                        imm.showSoftInput(view, InputMethodManager.SHOW_FORCED);
                    }"""
if old not in text:
    if "SHOW_FORCED" in text:
        raise SystemExit(0)
    raise SystemExit(f"showSoftInput(view, 0) pattern not found in {path}")
path.write_text(text.replace(old, new, 1))
print(f"Patched IME retap showSoftInput into {path}")
PY
else
  echo "MakepadActivity showKeyboard: no showSoftInput(view, 0) to replace (check manually)."
fi

# API 26 (Android 8.0) crash: InputDevice.isEnabled() is API 27+.
if grep -qF 'Our shops still run API 26' "$JAVA"; then
  echo "Android API-26 InputDevice.isEnabled guard already present."
else
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = """    private boolean isPhysicalTextKeyboard(InputDevice device) {
        return device != null
            && device.isEnabled()
            && !device.isVirtual()
            && device.supportsSource(InputDevice.SOURCE_KEYBOARD)
            && device.getKeyboardType() == InputDevice.KEYBOARD_TYPE_ALPHABETIC;
    }"""
new = """    private boolean isPhysicalTextKeyboard(InputDevice device) {
        // InputDevice.isEnabled() is API 27+. Our shops still run API 26
        // (Android 8.0); calling it there is a NoSuchMethodError crash at boot.
        if (device == null || device.isVirtual()) {
            return false;
        }
        if (android.os.Build.VERSION.SDK_INT >= 27 && !device.isEnabled()) {
            return false;
        }
        return device.supportsSource(InputDevice.SOURCE_KEYBOARD)
            && device.getKeyboardType() == InputDevice.KEYBOARD_TYPE_ALPHABETIC;
    }"""
if old not in text:
    # Already patched, or upstream changed — leave alone if guard present.
    if "Our shops still run API 26" in text:
        raise SystemExit(0)
    raise SystemExit(f"isPhysicalTextKeyboard pattern not found in {path}")
path.write_text(text.replace(old, new, 1))
print(f"Patched API-26 InputDevice.isEnabled guard into {path}")
PY
fi

# OS Copy bar opened without Copy: a second showClipboardActions in a row
# (text block + selection root after a finger hold) stored hasSelection,
# then finish() on the old bar ran onDestroyActionMode, which reset it.
if grep -qF 'KUMKAJ_CLIP_BAR_RACE' "$JAVA"; then
  echo "Android clipboard bar show/finish order already patched."
else
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old_show = """            public void run() {
                mHasSelection = hasSelection;
                mSelectionBounds[0] = left;
                mSelectionBounds[1] = top;
                mSelectionBounds[2] = right;
                mSelectionBounds[3] = bottom;
                mKeyboardShift = keyboardShift;

                // If ActionMode is already showing, finish it first
                if (mActionMode != null) {
                    mActionMode.finish();
                }
"""
new_show = """            public void run() {
                // KUMKAJ_CLIP_BAR_RACE: finish the bar already up BEFORE
                // storing the new state. finish() runs onDestroyActionMode;
                // it used to clear mHasSelection, so a second show in a row
                // (block + selection root after a finger hold) opened
                // Paste / Select all without Copy (AGS2, 2026-11-14).
                if (mActionMode != null) {
                    ActionMode old = mActionMode;
                    mActionMode = null;
                    old.finish();
                }
                mHasSelection = hasSelection;
                mSelectionBounds[0] = left;
                mSelectionBounds[1] = top;
                mSelectionBounds[2] = right;
                mSelectionBounds[3] = bottom;
                mKeyboardShift = keyboardShift;
"""
old_destroy = """    private void onDestroyActionModeInternal(ActionMode mode) {
        mActionMode = null;
        mHasSelection = false;
    }"""
new_destroy = """    private void onDestroyActionModeInternal(ActionMode mode) {
        // Only the bar on screen resets the state: one replaced by a newer
        // show must not clear the newer bar's selection.
        if (mode == mActionMode) {
            mActionMode = null;
            mHasSelection = false;
        }
    }"""
for old, label in ((old_show, "showClipboardActions"), (old_destroy, "onDestroyActionModeInternal")):
    if old not in text:
        raise SystemExit(f"{label} pattern not found in {path}")
text = text.replace(old_show, new_show, 1).replace(old_destroy, new_destroy, 1)
path.write_text(text)
print(f"Patched clipboard bar show/finish order into {path}")
PY
fi

# Read-only selections (labels, pages) get Copy / Select all only: Cut there
# only copied and Paste did nothing. Rust passes the flag as a 7th argument
# (android_jni.rs to_java_show_clipboard_actions, signature (ZIIIIIZ)V) —
# a checkout without this block crashes on the first touch selection.
if grep -qF 'KUMKAJ_CLIP_BAR_READ_ONLY' "$JAVA"; then
  echo "Android clipboard bar read-only flag already patched."
else
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
swaps = [
    ("    private boolean mHasSelection = false;\n",
     "    private boolean mHasSelection = false;\n"
     "    // KUMKAJ_CLIP_BAR_READ_ONLY: selection in copy that cannot be edited.\n"
     "    private boolean mClipReadOnly = false;\n"),
    ("    public void showClipboardActions(final boolean hasSelection, final int left, final int top, final int right, final int bottom, final int keyboardShift) {",
     "    public void showClipboardActions(final boolean hasSelection, final int left, final int top, final int right, final int bottom, final int keyboardShift, final boolean readOnly) {"),
    ("                mKeyboardShift = keyboardShift;\n",
     "                mKeyboardShift = keyboardShift;\n"
     "                mClipReadOnly = readOnly;\n"),
    ("        if (cutItem != null) cutItem.setVisible(hasSelection);\n"
     "        if (pasteItem != null) pasteItem.setVisible(hasClipboard);",
     "        if (cutItem != null) cutItem.setVisible(hasSelection && !mClipReadOnly);\n"
     "        if (pasteItem != null) pasteItem.setVisible(hasClipboard && !mClipReadOnly);"),
]
for old, new in swaps:
    if text.count(old) != 1:
        raise SystemExit(f"clipboard read-only pattern not found once in {path}: {old.strip()[:60]}")
    text = text.replace(old, new, 1)
path.write_text(text)
print(f"Patched clipboard bar read-only flag into {path}")
PY
fi

JAVA_DIR="$(dirname "$JAVA")"
HELPER_DST="$JAVA_DIR/ImeInsetsAnimationHelper.java"
HELPER_SRC="$ROOT/patches/makepad/ImeInsetsAnimationHelper.java"
if [[ -f "$HELPER_SRC" ]]; then
  if [[ ! -f "$HELPER_DST" ]] || ! cmp -s "$HELPER_SRC" "$HELPER_DST"; then
    cp -f "$HELPER_SRC" "$HELPER_DST"
    echo "Installed ImeInsetsAnimationHelper.java into $JAVA_DIR"
  else
    echo "ImeInsetsAnimationHelper.java already current."
  fi
fi

COMPILE_RS="$MAKEPAD_ROOT/tools/cargo_makepad/src/android/compile.rs"
if [[ -f "$COMPILE_RS" ]] && ! grep -qF 'ImeInsetsAnimationHelper.java' "$COMPILE_RS"; then
  python3 - "$COMPILE_RS" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
needle = '        makepad_java_classes_dir.join("MakepadActivity.java"),\n'
insert = needle + '        // Separate from MakepadActivity so API 26–29 never load WindowInsetsAnimation.\n        makepad_java_classes_dir.join("ImeInsetsAnimationHelper.java"),\n'
if 'ImeInsetsAnimationHelper.java' in text:
    raise SystemExit(0)
if needle not in text:
    raise SystemExit(f"MakepadActivity.java compile.rs entry not found in {path}")
path.write_text(text.replace(needle, insert, 1))
print(f"Registered ImeInsetsAnimationHelper.java in {path}")
PY
else
  echo "cargo-makepad already compiles ImeInsetsAnimationHelper.java."
fi

# AAB aapt2 / bundletool remaps resource IDs vs legacy `aapt package` R.java.
# Baking R.drawable.makepad_launch_background into dex crashed onCreate on API 26
# (Play "keeps stopping") when that constant landed on id/menu_copy. Resolve by name.
if grep -qF 'class MakepadLaunchBg' "$JAVA" && ! grep -qF 'setBackgroundResource(R.drawable.makepad_launch_background)' "$JAVA"; then
  echo "MakepadLaunchBg runtime drawable lookup already present."
else
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
helper = '''
/**
 * Launch / surface backdrop. Look up the drawable by name at runtime.
 *
 * AAB packaging links resources with aapt2 --proto-format (and bundletool may
 * inject split XML), which assigns different type/entry IDs than the legacy
 * `aapt package` R.java used to compile classes.dex. Baking
 * {@code R.drawable.makepad_launch_background} into dex then crashes onCreate
 * on API 26+ when that constant lands on an {@code id/} entry (Play:
 * "keeps stopping"). Name lookup always matches the installed resource table.
 */
final class MakepadLaunchBg {
    static final int FALLBACK_COLOR = 0xFFF5F7FA;

    static void apply(View view) {
        try {
            Context ctx = view.getContext();
            int id = ctx.getResources().getIdentifier(
                "makepad_launch_background", "drawable", ctx.getPackageName());
            if (id != 0) {
                view.setBackgroundResource(id);
                return;
            }
        } catch (RuntimeException ignored) {
        }
        view.setBackgroundColor(FALLBACK_COLOR);
    }

    static int styleId(Context ctx, String name) {
        try {
            return ctx.getResources().getIdentifier(name, "style", ctx.getPackageName());
        } catch (RuntimeException ignored) {
            return 0;
        }
    }
}

'''
if 'class MakepadLaunchBg' not in text:
    anchor = 'class ResizingLayout'
    if anchor not in text:
        raise SystemExit(f"ResizingLayout class not found in {path}")
    text = text.replace(anchor, helper + anchor, 1)
# Exact statement replacements only — never strip a receiver into `.MakepadLaunchBg`.
for old, new in [
    (
        '        setBackgroundResource(R.drawable.makepad_launch_background);',
        '        MakepadLaunchBg.apply(this);',
    ),
    (
        '        surfaceContentLayout.setBackgroundResource(R.drawable.makepad_launch_background);',
        '        MakepadLaunchBg.apply(surfaceContentLayout);',
    ),
    (
        '        mSurfaceSnapshotBackdrop.setBackgroundResource(R.drawable.makepad_launch_background);',
        '        MakepadLaunchBg.apply(mSurfaceSnapshotBackdrop);',
    ),
    (
        '        mSurfaceCoverOverlay.setBackgroundResource(R.drawable.makepad_launch_background);',
        '        MakepadLaunchBg.apply(mSurfaceCoverOverlay);',
    ),
    (
        '        mSurfaceSnapshotOverlay.setBackgroundResource(R.drawable.makepad_launch_background);',
        '        MakepadLaunchBg.apply(mSurfaceSnapshotOverlay);',
    ),
]:
    text = text.replace(old, new)
old_theme = '            setTheme(R.style.MakepadAppTheme);'
new_theme = '''            int themeId = MakepadLaunchBg.styleId(this, "MakepadAppTheme");
            if (themeId != 0) {
                setTheme(themeId);
            }'''
if old_theme in text:
    text = text.replace(old_theme, new_theme, 1)
if 'setBackgroundResource(R.drawable.makepad_launch_background)' in text:
    raise SystemExit(f"leftover R.drawable.makepad_launch_background in {path}")
if 'R.style.MakepadAppTheme' in text:
    raise SystemExit(f"leftover R.style.MakepadAppTheme in {path}")
path.write_text(text)
print(f"Patched MakepadLaunchBg runtime resource lookup in {path}")
PY
fi

# Inline WindowInsetsAnimation.Callback in ResizingLayout ClassNotFound's on API 26–29.
if grep -qF 'ImeInsetsAnimationHelper' "$JAVA" && grep -qF 'Class.forName("dev.makepad.android.ImeInsetsAnimationHelper")' "$JAVA"; then
  echo "Android API-26 IME helper reflection attach already present."
elif grep -q 'setWindowInsetsAnimationCallback' "$JAVA"; then
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import re, sys
path = Path(sys.argv[1])
text = path.read_text()
# Replace ResizingLayout constructor body that inlines WindowInsetsAnimation.
pat = re.compile(
    r'(class ResizingLayout\s+extends\s+LinearLayout\s+implements\s+View\.OnApplyWindowInsetsListener\s*\{\s*'
    r'public ResizingLayout\(Context context\)\{\s*'
    r'super\(context\);\s*)'
    r'.*?(\n    \}\s*\n\s*@Override\s*\n\s*public WindowInsets onApplyWindowInsets)',
    re.S,
)
repl = r'''\1// Keep a stable non-black fallback behind the SurfaceView for task snapshots
        // and system transition frames that cannot capture the separate surface layer.
        MakepadLaunchBg.apply(this);
        setOnApplyWindowInsetsListener(this);

        // IME animation API is API 30+. Load the helper by name so API 26–29
        // never verify/load WindowInsetsAnimation$Callback (a direct class
        // reference still ClassNotFound's on old ART and left a black SurfaceView).
        if (android.os.Build.VERSION.SDK_INT >= 30) {
            try {
                Class.forName("dev.makepad.android.ImeInsetsAnimationHelper")
                    .getMethod("attachIfSupported", View.class)
                    .invoke(null, this);
            } catch (Throwable ignored) {
            }
        }
    \2'''
new, n = pat.subn(repl, text, count=1)
if n != 1:
    raise SystemExit(f"ResizingLayout constructor pattern not found in {path}")
path.write_text(new)
print(f"Patched ResizingLayout to reflectively attach ImeInsetsAnimationHelper in {path}")
PY
else
  echo "ResizingLayout has no WindowInsetsAnimation callback to relocate."
fi

# Soft-keyboard height under adjustNothing (API 30+, esp. landscape): keep the
# last measured overlap while the IME stays open when a later onEnd /
# onGlobalLayout sample reads 0. Never estimates a height.
if grep -qF 'lastNonZeroOverlapPx' "$JAVA"; then
  echo "MakepadImeInsets lastNonZeroOverlap already present."
else
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
if "lastNonZeroOverlapPx" in text:
    raise SystemExit(0)
marker = "    static boolean imeAnimationInProgress = false;\n"
insert = marker + """
    // Last non-zero bottom overlap while the IME was open. adjustNothing +
    // landscape OEMs often follow a good onProgress height with a final
    // onGlobalLayout/onEnd sample of 0; without this, native KeyboardView
    // never lifts the focused field (or collapses a shift it already applied).
    static int lastNonZeroOverlapPx = 0;
"""
if marker not in text:
    raise SystemExit(f"imeAnimationInProgress marker not found in {path}")
text = text.replace(marker, insert, 1)
old_report = """        int bottomOverlap = bottomOverlapPx(view, insets);
        boolean visible = isVisible(view, bottomOverlap);
        MakepadNative.surfaceOnResizeTextIME(bottomOverlap, visible);
    }
}
"""
new_report = """        int bottomOverlap = bottomOverlapPx(view, insets);
        boolean visible = isVisible(view, bottomOverlap);
        if (bottomOverlap > keyboardThresholdPx(view)) {
            lastNonZeroOverlapPx = bottomOverlap;
        } else if (visible && lastNonZeroOverlapPx > keyboardThresholdPx(view)) {
            // Keep the last good height while the IME is still open.
            bottomOverlap = lastNonZeroOverlapPx;
        }
        if (!visible) {
            lastNonZeroOverlapPx = 0;
        }
        MakepadNative.surfaceOnResizeTextIME(bottomOverlap, visible);
    }
}
"""
if old_report not in text:
    raise SystemExit(f"MakepadImeInsets.report tail not found in {path}")
path.write_text(text.replace(old_report, new_report, 1))
print(f"Patched MakepadImeInsets lastNonZeroOverlap into {path}")
PY
fi

# Never estimate an IME height. An earlier version of the step above guessed a
# docked keyboard of 2/5 of the window in landscape whenever no measurement
# arrived; KeyboardView then lifted fields with no keyboard on screen (beige
# gap). Strip that branch from activities bridged before it was removed.
if grep -qF 'h * 2 / 5' "$JAVA"; then
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = """        } else if (visible && bottomOverlap <= 0) {
            // Last resort: Type.ime() and visible-frame both report 0 under
            // adjustNothing, most often in landscape. Estimate a docked IME
            // so KeyboardView can still lift the focused field.
            int h = rootHeightPx(view);
            if (h > 0 && view.getWidth() > view.getHeight()) {
                bottomOverlap = clampToRootHeight(view, Math.max(keyboardThresholdPx(view), h * 2 / 5));
                lastNonZeroOverlapPx = bottomOverlap;
            }
        }
"""
if old not in text:
    raise SystemExit(f"IME landscape estimate block not found verbatim in {path}; remove it by hand")
path.write_text(text.replace(old, "        }\n", 1))
print(f"Removed the IME landscape height estimate from {path}")
PY
else
  echo "MakepadImeInsets has no IME height estimate."
fi

# Soft-keyboard height on API 26–29. Under adjustNothing WindowManager never
# folds the IME into this window's visible frame (vf = cf), so
# MakepadImeInsets measured 0 and native code never got DidShow. Measured on
# Huawei AGS2-W09 / API 26 / SwiftKey. MakepadImeProbe.java reads the real
# keyboard top from an adjustResize side panel; MakepadImeInsets uses that
# overlap when Type.ime() is unavailable. API 30+ never creates the probe.
PROBE_SRC="$ROOT/patches/makepad/MakepadImeProbe.java"
PROBE_DST="$JAVA_DIR/MakepadImeProbe.java"
if [[ ! -f "$PROBE_DST" ]] || ! cmp -s "$PROBE_SRC" "$PROBE_DST"; then
  cp -f "$PROBE_SRC" "$PROBE_DST"
  echo "Installed MakepadImeProbe.java into $JAVA_DIR"
else
  echo "MakepadImeProbe.java already current."
fi
if [[ -f "$COMPILE_RS" ]] && ! grep -qF 'MakepadImeProbe.java' "$COMPILE_RS"; then
  python3 - "$COMPILE_RS" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
needle = '        makepad_java_classes_dir.join("MakepadActivity.java"),\n'
insert = needle + '        // API 26–29 soft-keyboard height probe (adjustNothing hides the IME).\n        makepad_java_classes_dir.join("MakepadImeProbe.java"),\n'
if needle not in text:
    raise SystemExit(f"MakepadActivity.java compile.rs entry not found in {path}")
path.write_text(text.replace(needle, insert, 1))
print(f"Registered MakepadImeProbe.java in {path}")
PY
else
  echo "cargo-makepad already compiles MakepadImeProbe.java."
fi
if grep -qF 'probeOverlapPx' "$JAVA"; then
  echo "MakepadActivity already wires MakepadImeProbe."
else
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()

def swap(old, new, what):
    global text
    if old not in text:
        raise SystemExit(f"{what} not found in {path}")
    text = text.replace(old, new, 1)

swap(
    "    static int lastNonZeroOverlapPx = 0;\n",
    "    static int lastNonZeroOverlapPx = 0;\n"
    "\n"
    "    // API 26–29: keyboard overlap with the surface, measured by\n"
    "    // MakepadImeProbe. adjustNothing keeps the IME out of this window's\n"
    "    // own visible frame below API 30, so that frame always reads 0.\n"
    "    static int probeOverlapPx = 0;\n",
    "lastNonZeroOverlapPx field",
)
swap(
    "        int fallback = visibleFrameBottomOverlapPx(view);\n",
    "        int fallback = Math.max(visibleFrameBottomOverlapPx(view), probeOverlapPx);\n",
    "visible-frame fallback in bottomOverlapPx",
)
swap(
    "    private MakepadSurface view;\n",
    "    private MakepadSurface view;\n"
    "    // API 26–29 keyboard height probe; null on API 30+.\n"
    "    private MakepadImeProbe mImeProbe;\n",
    "MakepadSurface view field",
)
swap(
    "        MakepadNative.activityOnCreate(this);\n",
    "        MakepadNative.activityOnCreate(this);\n"
    "        if (Build.VERSION.SDK_INT < 30) {\n"
    "            mImeProbe = new MakepadImeProbe(this, view);\n"
    "            mImeProbe.attach();\n"
    "        }\n",
    "activityOnCreate call",
)
swap(
    "    protected void onDestroy() {\n",
    "    protected void onDestroy() {\n"
    "        if (mImeProbe != null) {\n"
    "            mImeProbe.detach();\n"
    "            mImeProbe = null;\n"
    "        }\n",
    "onDestroy",
)
path.write_text(text)
print(f"Wired MakepadImeProbe into {path}")
PY
fi

# LocationListener.onStatusChanged is deprecated (API 29+); default empty — drop override.
if grep -qF 'onStatusChanged(String provider, int status, Bundle extras)' "$JAVA"; then
  python3 - "$JAVA" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = '                    @Override public void onStatusChanged(String provider, int status, Bundle extras) {}\n'
if old not in text:
    raise SystemExit(0)
path.write_text(text.replace(old, '', 1))
print(f"Removed deprecated LocationListener.onStatusChanged override from {path}")
PY
else
  echo "LocationListener.onStatusChanged override already absent."
fi

# Skip desktop packaging images + unused widget script fonts from Android APK assets.
# resources/ still holds icon_*.png / .ico for MSI/Linux; Android only needs
# mipmaps (res/) + in-app WebP (kumkaj-mark.webp, nav/*.webp) + fonts.
if [[ -f "$COMPILE_RS" ]] && grep -qF 'fn prune_android_runtime_assets' "$COMPILE_RS"; then
  echo "Android APK asset prune already present."
elif [[ -f "$COMPILE_RS" ]]; then
  python3 - "$COMPILE_RS" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
if "fn prune_android_runtime_assets" in text:
    raise SystemExit(0)

helper = r'''
/// Drop files that belong on disk for desktop packaging but are never loaded
/// from Android assets (launcher uses res/mipmap; in-app mark is WebP /
/// include_bytes). Also drop makepad_widgets NotoSans* script fonts the stock
/// theme does not reference — apps ship their own WorldFonts copy.
fn prune_android_runtime_assets(dst_dir: &Path, crate_name: &str) -> Result<(), String> {
    let packaging = [
        "icon.ico",
        "icon_32.png",
        "icon_64.png",
        "icon_128.png",
        "icon_256.png",
        "icon_512.png",
        "icon_1024.png",
        "kumkaj-mark.png",
        "kumkaj-mark-128.png",
        "kumkaj-mark-128.webp",
        "kumkaj-mark-512.webp",
        "kumkaj-mark.avif",
        "kumkaj-mark-128.avif",
        "kumkaj-mark-512.avif",
        "gotigin-mark.webp",
    ];
    for name in packaging {
        let p = dst_dir.join(name);
        if p.is_file() {
            rm(&p)?;
        }
    }
    // Any leftover AVIF companions under resources/
    if let Ok(entries) = ls(dst_dir) {
        for path in entries {
            let is_avif = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("avif"))
                .unwrap_or(false);
            if is_avif {
                rm(&dst_dir.join(&path))?;
            }
        }
    }
    if crate_name == "makepad_widgets" {
        if let Ok(entries) = ls(dst_dir) {
            for path in entries {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                // Theme uses IBMPlex / LXGW / emoji / fa-solid / LiberationMono.
                // App WorldFonts owns the NotoSans* script set under the app crate.
                // jetbrains_mono is studio/editor-only — not referenced by app UI.
                if (name.starts_with("NotoSans") && name.ends_with(".ttf"))
                    || name == "jetbrains_mono_variable.ttf"
                    || name == "NewCMMath-Regular.otf"
                {
                    rm(&dst_dir.join(&path))?;
                }
            }
        }
    }
    // latex_math only exists for MathView; apps do not use it. Drop its fonts
    // (NewCMMath is also under widgets if ever needed).
    if crate_name == "makepad_latex_math" {
        if let Ok(entries) = ls(dst_dir) {
            for path in entries {
                rm(&dst_dir.join(&path))?;
            }
        }
    }
    Ok(())
}

'''

# Insert helper before cp_android_runtime_assets
anchor = "// resources/ios is iOS-only; resources/android ships as runtime assets minus"
if anchor not in text:
    raise SystemExit(f"anchor for prune helper not found in {path}")
text = text.replace(anchor, helper + anchor, 1)

# Call prune after cp_android_runtime_assets in add_assets_dir_to_apk
old1 = """    cp_android_runtime_assets(source_dir, &dst_dir)?;
    if config.small_fonts && asset_subdir == \"resources\" {
        for (target_name, replacement_name) in SMALL_FONT_REPLACEMENTS {
            let replacement = source_dir.join(replacement_name);
            let target = dst_dir.join(target_name);
            if replacement.is_file() && target.is_file() {
                cp(&replacement, &target, false)?;
            }
        }
    }

    let assets = ls(&dst_dir)?;
"""
new1 = """    cp_android_runtime_assets(source_dir, &dst_dir)?;
    prune_android_runtime_assets(&dst_dir, &crate_name)?;
    if config.small_fonts && asset_subdir == \"resources\" {
        for (target_name, replacement_name) in SMALL_FONT_REPLACEMENTS {
            let replacement = source_dir.join(replacement_name);
            let target = dst_dir.join(target_name);
            if replacement.is_file() && target.is_file() {
                cp(&replacement, &target, false)?;
            }
        }
    }

    let assets = ls(&dst_dir)?;
"""
if old1 not in text:
    raise SystemExit("add_assets_dir_to_apk insert point not found")
text = text.replace(old1, new1, 1)

# Same for stage_makepad_assets_subdir (AAB path)
old2 = """    cp_android_runtime_assets(source_dir, &dst_dir)?;
    if config.small_fonts && asset_subdir == \"resources\" {
        for (target_name, replacement_name) in SMALL_FONT_REPLACEMENTS {
            let replacement = source_dir.join(replacement_name);
            let target = dst_dir.join(target_name);
            if replacement.is_file() && target.is_file() {
                cp(&replacement, &target, false)?;
            }
        }
    }
    Ok(())
}
"""
new2 = """    cp_android_runtime_assets(source_dir, &dst_dir)?;
    prune_android_runtime_assets(&dst_dir, &crate_name)?;
    if config.small_fonts && asset_subdir == \"resources\" {
        for (target_name, replacement_name) in SMALL_FONT_REPLACEMENTS {
            let replacement = source_dir.join(replacement_name);
            let target = dst_dir.join(target_name);
            if replacement.is_file() && target.is_file() {
                cp(&replacement, &target, false)?;
            }
        }
    }
    Ok(())
}
"""
if old2 not in text:
    raise SystemExit("stage_makepad_assets_subdir insert point not found")
text = text.replace(old2, new2, 1)

path.write_text(text)
print(f"Patched Android APK asset prune into {path}")
PY
else
  echo "warn: cargo-makepad compile.rs missing — skip asset prune" >&2
fi

# Pack apps' menu shelf + named templates into the APK/AAB (same trees desktop
# stages beside the exe). Only assets/items and assets/templates — not
# Screenshots / store banners under assets/.
if [[ -f "$COMPILE_RS" ]] && grep -qF 'assets/items' "$COMPILE_RS" && grep -qF 'Menu shelf + named templates' "$COMPILE_RS"; then
  echo "Android menu assets packaging already present."
elif [[ -f "$COMPILE_RS" ]]; then
  python3 - "$COMPILE_RS" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
if "Menu shelf + named templates" in text:
    raise SystemExit(0)

apk_insert = '''    add_font_assets_dir_to_apk(
        &build_paths.out_dir,
        &mut assets_to_add,
        build_crate,
        &build_crate_dir.join("fonts"),
        &build_crate_dir.join("resources"),
        config,
    )?;

    // Menu shelf + named templates (parity with desktop package staging).
    // Skip when assets/CATALOGUE_REMOTE — remote catalogue (Khaynjo).
    if !build_crate_dir.join("assets/CATALOGUE_REMOTE").is_file() {
        add_assets_dir_to_apk(
            &build_paths.out_dir,
            &mut assets_to_add,
            build_crate,
            &build_crate_dir.join("assets/items"),
            "assets/items",
            config,
        )?;
        add_assets_dir_to_apk(
            &build_paths.out_dir,
            &mut assets_to_add,
            build_crate,
            &build_crate_dir.join("assets/templates"),
            "assets/templates",
            config,
        )?;
    }

    let deps = get_crate_dep_dirs(build_crate, &build_dir, &android_targets[0].toolchain());
'''

apk_old = '''    add_font_assets_dir_to_apk(
        &build_paths.out_dir,
        &mut assets_to_add,
        build_crate,
        &build_crate_dir.join("fonts"),
        &build_crate_dir.join("resources"),
        config,
    )?;

    let deps = get_crate_dep_dirs(build_crate, &build_dir, &android_targets[0].toolchain());
'''
if apk_old not in text:
    raise SystemExit("add_resources menu-assets insert point not found")
text = text.replace(apk_old, apk_insert, 1)

aab_insert = '''    stage_makepad_font_subdir(
        assets_root,
        build_crate,
        &build_crate_dir.join("fonts"),
        &build_crate_dir.join("resources"),
        config,
    )?;

    // Menu shelf + named templates (parity with desktop package staging).
    // Skip when assets/CATALOGUE_REMOTE — remote catalogue (Khaynjo).
    if !build_crate_dir.join("assets/CATALOGUE_REMOTE").is_file() {
        stage_makepad_assets_subdir(
            assets_root,
            build_crate,
            &build_crate_dir.join("assets/items"),
            "assets/items",
            config,
        )?;
        stage_makepad_assets_subdir(
            assets_root,
            build_crate,
            &build_crate_dir.join("assets/templates"),
            "assets/templates",
            config,
        )?;
    }

    let deps = get_crate_dep_dirs(build_crate, build_dir, &android_targets[0].toolchain());
'''

aab_old = '''    stage_makepad_font_subdir(
        assets_root,
        build_crate,
        &build_crate_dir.join("fonts"),
        &build_crate_dir.join("resources"),
        config,
    )?;

    let deps = get_crate_dep_dirs(build_crate, build_dir, &android_targets[0].toolchain());
'''
if aab_old not in text:
    raise SystemExit("stage_aab_assets menu-assets insert point not found")
text = text.replace(aab_old, aab_insert, 1)

path.write_text(text)
print(f"Patched Android menu assets packaging into {path}")
PY
else
  echo "warn: cargo-makepad compile.rs missing — skip menu assets packaging" >&2
fi
