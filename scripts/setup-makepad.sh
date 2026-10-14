#!/usr/bin/env bash
# Populate vendor/makepad from Makepad 2 and apply the Wayland scroll-sign fix.
# (Older makepad main tips negated Wayland axis values, which inverted every page.)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENDOR="$ROOT/vendor/makepad"
CHECKOUT_ROOT="$HOME/.cargo/git/checkouts/makepad-ec2f134f34cd9f98"

SRC=""
# Prefer the 915ce7c checkout (known to build with this workspace toolchain).
for c in "$CHECKOUT_ROOT/915ce7c" "$CHECKOUT_ROOT/a4ea253"; do
  if [[ -d "$c/widgets" ]]; then
    SRC="$c"
    break
  fi
done
if [[ -z "$SRC" && -d "$CHECKOUT_ROOT" ]]; then
  while IFS= read -r c; do
    if [[ -d "$c/widgets" ]]; then SRC="$c"; break; fi
  done < <(ls -1dt "$CHECKOUT_ROOT"/* 2>/dev/null)
fi
if [[ -z "$SRC" ]]; then
  echo "No local makepad checkout — cloning github.com/makepad/makepad (main)…"
  TMP="$(mktemp -d)"
  git clone --depth 1 --branch main https://github.com/makepad/makepad.git "$TMP/makepad"
  SRC="$TMP/makepad"
fi

echo "Using Makepad 2 source: $SRC"
rm -rf "$VENDOR"
mkdir -p "$ROOT/vendor"
# Copy Makepad tree without build/docs/examples noise (bash + rsync, no Python).
rsync -a \
  --exclude '.git/' \
  --exclude 'target/' \
  --exclude 'node_modules/' \
  --exclude 'apps/' \
  --exclude 'examples/' \
  --exclude 'tools/' \
  --exclude 'code_editor/' \
  --exclude 'xr/' \
  --exclude 'docs/' \
  "$SRC/" "$VENDOR/"
BIDI="$VENDOR/libs/unicode/unicode-bidi/Cargo.toml"
if [[ -f "$BIDI" ]]; then
  sed -i 's|path = "../smallvec"|path = "../../smallvec"|g' "$BIDI"
fi
echo "vendor ready: $VENDOR"

PATCH="$ROOT/patches/makepad/wayland_scroll_sign.rs.patch"
if [[ -f "$PATCH" ]]; then
  # Idempotent: only apply if still using the old negation.
  if grep -q 'scroll_accumulator.y -= value' \
      "$VENDOR/platform/src/os/linux/wayland/wayland_state.rs"; then
    patch -p1 -d "$VENDOR" < "$PATCH"
    echo "Applied Wayland scroll-sign fix."
  else
    echo "Wayland scroll sign already correct — skip patch."
  fi
fi

CAPTION_PATCH="$ROOT/patches/makepad/dark_caption_buttons.rs.patch"
if [[ -f "$CAPTION_PATCH" ]]; then
  if grep -q 'color: #000, color_hover: #000, color_down: #000' \
      "$VENDOR/widgets/src/window.rs"; then
    patch -p1 -d "$VENDOR" < "$CAPTION_PATCH"
    echo "Applied dark caption button fix."
  else
    echo "Dark caption buttons already applied — skip patch."
  fi
fi

HEAP_PATCH="$ROOT/patches/makepad/heap_is_object_alive.rs.patch"
if [[ -f "$HEAP_PATCH" ]]; then
  if ! grep -q 'fn is_object_alive' "$VENDOR/platform/script/src/heap.rs"; then
    patch -p1 -d "$VENDOR" < "$HEAP_PATCH"
    echo "Applied heap is_object_alive helper."
  else
    echo "heap is_object_alive already present — skip patch."
  fi
fi

ANIM_PATCH="$ROOT/patches/makepad/animator_skip_dead_tracks.rs.patch"
if [[ -f "$ANIM_PATCH" ]]; then
  if ! grep -q 'Drop tracks whose script objects were freed' \
      "$VENDOR/widgets/src/animator.rs"; then
    patch -p1 -d "$VENDOR" < "$ANIM_PATCH"
    echo "Applied animator dead-track skip (Android UAF guard)."
  else
    echo "Animator dead-track skip already present — skip patch."
  fi
fi

IMG_SYNC_PATCH="$ROOT/patches/makepad/android_image_force_sync.rs.patch"
if [[ -f "$IMG_SYNC_PATCH" ]]; then
  if ! grep -q 'target_os = "android"' "$VENDOR/draw/src/image_cache.rs" \
      || ! grep -q 'crate_resource logos' "$VENDOR/draw/src/image_cache.rs"; then
    patch -p1 -d "$VENDOR" < "$IMG_SYNC_PATCH"
    echo "Applied Android image force_sync decode."
  else
    echo "Android image force_sync already present — skip patch."
  fi
fi

ASSET_PATCH="$ROOT/patches/makepad/android_asset_load_prefix.rs.patch"
if [[ -f "$ASSET_PATCH" ]]; then
  if ! grep -q 'as_mut_ptr() as *mut _' \
      "$VENDOR/platform/src/os/linux/android/android_jni.rs"; then
    patch -p1 -d "$VENDOR" < "$ASSET_PATCH"
    echo "Applied Android asset load prefix + mut ptr fix."
  else
    echo "Android asset load fix already present — skip patch."
  fi
fi

PORTAL_PATCH="$ROOT/patches/makepad/portal_list_stop_at_range.rs.patch"
if [[ -f "$PORTAL_PATCH" ]]; then
  if ! grep -q 'drawn_range_extent' "$VENDOR/widgets/src/portal_list.rs"; then
    patch -p1 -d "$VENDOR" < "$PORTAL_PATCH"
    echo "Applied PortalList stop-at-range (content-sized lists)."
  else
    echo "PortalList stop-at-range already present — skip patch."
  fi
fi

IME_PATCH="$ROOT/patches/makepad/android_ime_dismiss.rs.patch"
if [[ -f "$IME_PATCH" ]]; then
  if ! grep -q 'Same-area calls (caret repair' \
      "$VENDOR/platform/src/event/keyboard.rs"; then
    patch -p1 -d "$VENDOR" < "$IME_PATCH"
    echo "Applied Android IME dismiss latch (one keyboard, stay hidden)."
  else
    echo "Android IME dismiss latch already present — skip patch."
  fi
fi

SCROLL_TOUCH_PATCH="$ROOT/patches/makepad/scroll_touch_over_buttons.rs.patch"
if [[ -f "$SCROLL_TOUCH_PATCH" ]]; then
  if ! grep -q "hits_with_capture_overload(cx, scroll_area, true)" \
      "$VENDOR/widgets/src/scroll_bar.rs"; then
    patch -p1 -d "$VENDOR" < "$SCROLL_TOUCH_PATCH"
    echo "Applied ScrollYView touch-scroll over buttons (Android)."
  else
    echo "Scroll touch-over-buttons already present — skip patch."
  fi
fi

VIEW_SCROLL_GETTERS="$ROOT/patches/makepad/view_scroll_pos_getters.rs.patch"
if [[ -f "$VIEW_SCROLL_GETTERS" ]]; then
  if ! grep -q "pub fn get_scroll_pos(&self) -> Vec2d" \
      "$VENDOR/widgets/src/view.rs"; then
    patch -p1 -d "$VENDOR" < "$VIEW_SCROLL_GETTERS"
    echo "Applied ViewRef scroll position getters (keyboard scroll)."
  else
    echo "ViewRef scroll getters already present — skip patch."
  fi
fi

# Real API migrations for newer rustc (associated MAX/MIN, Atomic::try_update).
bash "$ROOT/scripts/fix-makepad-rust-deprecations.sh"

echo "Done. Makepad 2 at vendor/makepad (stock direction, no invert_scroll)."
