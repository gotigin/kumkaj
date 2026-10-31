#!/usr/bin/env bash
# Migrate vendored Makepad crates off deprecated std/core numeric APIs.
# Idempotent. Real API updates — not allow(deprecated).
#
# Called from:
#   - setup-makepad.sh after clone/patch (vendor/makepad)
#   - Android / package scripts for MAKEPAD_ROOT (often vendor/makepad-full)
#
# Usage:
#   scripts/fix-makepad-rust-deprecations.sh
#   scripts/fix-makepad-rust-deprecations.sh /path/to/makepad-tree […]
#   MAKEPAD_ROOT=… scripts/fix-makepad-rust-deprecations.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

VENDORS=()
if [[ $# -gt 0 ]]; then
  VENDORS+=("$@")
elif [[ -n "${MAKEPAD_ROOT:-}" ]]; then
  VENDORS+=("$MAKEPAD_ROOT")
  if [[ -d "$ROOT/vendor/makepad" && "$MAKEPAD_ROOT" != "$ROOT/vendor/makepad" ]]; then
    VENDORS+=("$ROOT/vendor/makepad")
  fi
else
  [[ -d "$ROOT/vendor/makepad" ]] && VENDORS+=("$ROOT/vendor/makepad")
  [[ -d "$ROOT/vendor/makepad-full" ]] && VENDORS+=("$ROOT/vendor/makepad-full")
fi

if [[ ${#VENDORS[@]} -eq 0 ]]; then
  echo "warn: no Makepad tree found — skip deprecation fixes" >&2
  exit 0
fi

changed=0

# Literal string replace in a file (idempotent). Args: FILE old new [old new ...]
# Uses VENDOR from the enclosing fix_one_vendor call for relative paths.
replace_in() {
  local file="$1"
  shift
  local rel="${file#"$VENDOR"/}"
  if [[ ! -f "$file" ]]; then
    echo "warn: missing $rel" >&2
    return 0
  fi
  local tmp
  tmp="$(mktemp)"
  cp -f "$file" "$tmp"
  local old new
  while [[ $# -ge 2 ]]; do
    old="$1"
    new="$2"
    shift 2
    OLD="$old" NEW="$new" awk '
      BEGIN { old = ENVIRON["OLD"]; new = ENVIRON["NEW"] }
      {
        s = $0
        out = ""
        while ((i = index(s, old)) > 0) {
          out = out substr(s, 1, i - 1) new
          s = substr(s, i + length(old))
        }
        print out s
      }
    ' "$tmp" > "${tmp}.out"
    mv -f "${tmp}.out" "$tmp"
  done
  if ! cmp -s "$file" "$tmp"; then
    mv -f "$tmp" "$file"
    echo "Updated $rel"
    changed=$((changed + 1))
  else
    rm -f "$tmp"
    echo "Already current: $rel"
  fi
}

fix_one_vendor() {
  local VENDOR="$1"
  echo "=== Makepad rust deprecations: $VENDOR ==="
  if [[ ! -d "$VENDOR" ]]; then
    echo "warn: $VENDOR missing — skip" >&2
    return 0
  fi

  replace_in "$VENDOR/libs/smallvec/src/lib.rs" \
    'core::usize::MAX' 'usize::MAX' \
    'core::isize::MAX' 'isize::MAX'

  replace_in "$VENDOR/libs/smallvec/src/tests.rs" \
    'std::usize::MAX' 'usize::MAX'

  replace_in "$VENDOR/libs/stitch/src/store.rs" \
    '.fetch_update(' '.try_update('

  replace_in "$VENDOR/libs/gif/weezl/src/encode.rs" \
    'Code::max_value()' 'Code::MAX'

  if [[ -f "$VENDOR/libs/unicode/unicode-script/src/lib.rs" ]]; then
    if grep -qx 'use core::u64;' "$VENDOR/libs/unicode/unicode-script/src/lib.rs"; then
      sed -i '/^use core::u64;$/d' "$VENDOR/libs/unicode/unicode-script/src/lib.rs"
      echo "Updated libs/unicode/unicode-script/src/lib.rs"
      changed=$((changed + 1))
    else
      echo "Already current: libs/unicode/unicode-script/src/lib.rs"
    fi
  fi

  replace_in "$VENDOR/libs/bytemuck/src/contiguous.rs" \
    'u8::max_value()' 'u8::MAX' \
    'u16::max_value()' 'u16::MAX' \
    'u32::max_value()' 'u32::MAX' \
    'u64::max_value()' 'u64::MAX' \
    'u128::max_value()' 'u128::MAX' \
    'usize::max_value()' 'usize::MAX' \
    'i8::max_value()' 'i8::MAX' \
    'i8::min_value()' 'i8::MIN' \
    'i16::max_value()' 'i16::MAX' \
    'i16::min_value()' 'i16::MIN' \
    'i32::max_value()' 'i32::MAX' \
    'i32::min_value()' 'i32::MIN' \
    'i64::max_value()' 'i64::MAX' \
    'i64::min_value()' 'i64::MIN' \
    'i128::max_value()' 'i128::MAX' \
    'i128::min_value()' 'i128::MIN' \
    'isize::max_value()' 'isize::MAX' \
    'isize::min_value()' 'isize::MIN'

  replace_in "$VENDOR/libs/windows/windows-core/src/imp/weak_ref_count.rs" \
    '.fetch_update(' '.try_update('

  replace_in "$VENDOR/libs/micro_serde/src/serde_json.rs" \
    'std::u64::MAX' 'u64::MAX' \
    'std::u128::MAX' 'u128::MAX' \
    'std::u32::MAX' 'u32::MAX' \
    'std::u16::MAX' 'u16::MAX' \
    'std::u8::MAX' 'u8::MAX' \
    'std::i64::MIN' 'i64::MIN' \
    'std::i64::MAX' 'i64::MAX' \
    'std::i128::MIN' 'i128::MIN' \
    'std::i128::MAX' 'i128::MAX' \
    'std::i8::MAX' 'i8::MAX'

  replace_in "$VENDOR/libs/micro_serde/src/serde_ron.rs" \
    'std::usize::MAX' 'usize::MAX' \
    'std::u64::MAX' 'u64::MAX' \
    'std::u32::MAX' 'u32::MAX' \
    'std::u16::MAX' 'u16::MAX' \
    'std::u8::MAX' 'u8::MAX' \
    'std::i64::MIN' 'i64::MIN' \
    'std::i64::MAX' 'i64::MAX' \
    'std::i8::MAX' 'i8::MAX'

  replace_in "$VENDOR/libs/toml_parser/src/toml.rs" \
    'std::f64::NAN' 'f64::NAN' \
    'std::f64::INFINITY' 'f64::INFINITY'

  replace_in "$VENDOR/libs/rustybuzz/src/hb/buffer.rs" \
    'core::u32::MAX' 'u32::MAX' \
    'u.is_default_ignorable()' 'CharExt::is_default_ignorable(u)'

  replace_in "$VENDOR/libs/rustybuzz/src/hb/common.rs" \
    'core::u32::MAX' 'u32::MAX'

  replace_in "$VENDOR/libs/rustybuzz/src/hb/face.rs" \
    'core::u16::MAX' 'u16::MAX'

  replace_in "$VENDOR/libs/rustybuzz/src/hb/ot_layout_gpos_table.rs" \
    'core::i32::MIN' 'i32::MIN' \
    'core::i32::MAX' 'i32::MAX'

  replace_in "$VENDOR/platform/src/audio.rs" \
    'std::i16::MIN' 'i16::MIN' \
    'std::i16::MAX' 'i16::MAX'

  replace_in "$VENDOR/platform/src/os/linux/pulse_audio.rs" \
    'std::u32::MAX' 'u32::MAX'

  replace_in "$VENDOR/platform/src/os/apple/macos/macos_delegates.rs" \
    'i64::max_value()' 'i64::MAX'

  replace_in "$VENDOR/platform/src/os/apple/macos/macos.rs" \
    '.fetch_update(' '.try_update('

  replace_in "$VENDOR/libs/byteorder/src/lib.rs" \
    '::core::u64::MAX' 'u64::MAX' \
    '::core::i64::MAX' 'i64::MAX' \
    '::core::u128::MAX' 'u128::MAX' \
    '::core::i128::MAX' 'i128::MAX' \
    '::core::u16::MAX' 'u16::MAX' \
    '::core::i16::MAX' 'i16::MAX' \
    '::core::u32::MAX' 'u32::MAX' \
    '::core::i32::MAX' 'i32::MAX' \
    '::std::u16::MAX' 'u16::MAX' \
    '::std::i16::MAX' 'i16::MAX' \
    '::std::u32::MAX' 'u32::MAX' \
    '::std::i32::MAX' 'i32::MAX' \
    '::std::u64::MAX' 'u64::MAX' \
    '::std::i64::MAX' 'i64::MAX'

  replace_in "$VENDOR/libs/byteorder-lite/src/lib.rs" \
    '::core::u64::MAX' 'u64::MAX' \
    '::core::i64::MAX' 'i64::MAX' \
    '::core::u128::MAX' 'u128::MAX' \
    '::core::i128::MAX' 'i128::MAX' \
    '::core::u16::MAX' 'u16::MAX' \
    '::core::i16::MAX' 'i16::MAX' \
    '::core::u32::MAX' 'u32::MAX' \
    '::core::i32::MAX' 'i32::MAX' \
    '::std::u16::MAX' 'u16::MAX' \
    '::std::i16::MAX' 'i16::MAX' \
    '::std::u32::MAX' 'u32::MAX' \
    '::std::i32::MAX' 'i32::MAX' \
    '::std::u64::MAX' 'u64::MAX' \
    '::std::i64::MAX' 'i64::MAX'

  replace_in "$VENDOR/libs/vulkan/hexf-parse/src/lib.rs" \
    'use std::{f32, f64, fmt, isize, str};' 'use std::{fmt, str};'

  replace_in "$VENDOR/platform/src/os/linux/libc_sys.rs" \
    'pub fn read(fd: c_int, buf: *mut c_void, count: size_t) -> c_int;' \
    'pub fn read(fd: c_int, buf: *mut c_void, count: size_t) -> isize;' \
    'pub fn write(fd: c_int, buf: *const c_void, count: size_t) -> c_int;' \
    'pub fn write(fd: c_int, buf: *const c_void, count: size_t) -> isize;'

  # num-traits: drop deprecated module imports that shadow primitives.
  local NT="$VENDOR/libs/vulkan/num-traits/src"
  if [[ -d "$NT" ]]; then
    while IFS= read -r -d '' f; do
      local tmp
      tmp="$(mktemp)"
      cp -f "$f" "$tmp"
      sed -i \
        -e '/^use core::{f32, f64};$/d' \
        -e '/^use core::{i128, i16, i32, i64, i8, isize};$/d' \
        -e '/^use core::{u128, u16, u32, u64, u8, usize};$/d' \
        -e '/^use core::f32;$/d' \
        -e '/^use core::f64;$/d' \
        -e '/^use core::i128;$/d' \
        -e '/^[[:space:]]*use core::{i16, i32, i64, i8};$/d' \
        -e '/^[[:space:]]*use core::{f32, f64};$/d' \
        -e '/^[[:space:]]*use core::\$t;$/d' \
        "$tmp"
      sed -i -E 's/core::\$t::(INFINITY|NEG_INFINITY|NAN|MIN|MAX)/\$t::\1/g' "$tmp"
      sed -i 's/\$constant() -> \$T::consts::\$constant;/\$constant() -> core::\$T::consts::\$constant;/g' "$tmp"
      local t c
      for t in f32 f64; do
        for c in INFINITY NEG_INFINITY NAN MIN MAX; do
          sed -i "s/core::${t}::${c}/${t}::${c}/g" "$tmp"
        done
      done
      for t in i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize; do
        for c in MIN MAX; do
          sed -i "s/core::${t}::${c}/${t}::${c}/g" "$tmp"
        done
      done
      local rel="${f#"$VENDOR"/}"
      if ! cmp -s "$f" "$tmp"; then
        mv -f "$tmp" "$f"
        echo "Updated $rel"
        changed=$((changed + 1))
      else
        rm -f "$tmp"
      fi
    done < <(find "$NT" -name '*.rs' -print0 | sort -z)
  fi

  # cfg_aliases: drop trailing `;` after recursive macro invocations (expression position).
  # Must write via temp + cmp — `sed -i` always bumps mtime even when nothing
  # changes, which forces a full Makepad rebuild on every android-dev run.
  local CFG="$VENDOR/libs/vulkan/cfg_aliases/src/lib.rs"
  if [[ -f "$CFG" ]]; then
    local cfg_tmp
    cfg_tmp="$(mktemp)"
    cp -f "$CFG" "$cfg_tmp"
    sed -i \
      -e 's/\] \$(\$current)\* \$tok);$/] $($current)* $tok)/' \
      -e 's/^\([[:space:]]*\]\));$/\1)/' \
      "$cfg_tmp"
    sed -i '/{\$(\$current)+}$/{ n; s/^\([[:space:]]*\));$/\1)/; }' "$cfg_tmp"
    if ! cmp -s "$CFG" "$cfg_tmp"; then
      mv -f "$cfg_tmp" "$CFG"
      echo "Updated libs/vulkan/cfg_aliases/src/lib.rs"
      changed=$((changed + 1))
    else
      rm -f "$cfg_tmp"
      echo "Already current: libs/vulkan/cfg_aliases/src/lib.rs"
    fi
  fi
}

for v in "${VENDORS[@]}"; do
  fix_one_vendor "$v"
done

echo "Makepad rust deprecation fixes: ${changed} file(s) changed."
