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
        setBackgroundResource(R.drawable.makepad_launch_background);
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
