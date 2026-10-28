#!/usr/bin/env bash
# Populate vendor/makepad from Makepad 2 and apply Kumkaj's patches (patches/makepad/).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENDOR="$ROOT/vendor/makepad"
# The patches below are written against this exact upstream commit. A newer
# Makepad would make them fail, so never fall back to "whatever is newest".
MAKEPAD_REV="915ce7c4e271593101b46dcff0aa96e4466864af"
CARGO_GIT="${CARGO_HOME:-$HOME/.cargo}/git"
CHECKOUT="$CARGO_GIT/checkouts/makepad-ec2f134f34cd9f98/${MAKEPAD_REV:0:7}"
CARGO_DB="$CARGO_GIT/db/makepad-ec2f134f34cd9f98"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if [[ -d "$CHECKOUT/widgets" ]]; then
  SRC="$CHECKOUT"
elif git -C "$CARGO_DB" cat-file -e "$MAKEPAD_REV^{commit}" 2>/dev/null; then
  SRC="$TMP/makepad"
  mkdir -p "$SRC"
  git -C "$CARGO_DB" archive "$MAKEPAD_REV" | tar -x -C "$SRC"
else
  echo "Fetching makepad $MAKEPAD_REV from GitHub…"
  SRC="$TMP/makepad"
  git init -q "$SRC"
  git -C "$SRC" fetch -q --depth 1 https://github.com/makepad/makepad.git "$MAKEPAD_REV"
  git -C "$SRC" checkout -q FETCH_HEAD
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

# Kumkaj's fixes on top of upstream Makepad, applied in this order. The
# vendor tree is rebuilt from scratch above, so every patch must apply
# cleanly; a failure stops the script instead of leaving a half-patched tree.
# vendor/makepad is gitignored: a change made there without a patch here is
# lost on the next setup.
PATCHES=(
  wayland_scroll_sign               # Wayland wheel direction
  dark_caption_buttons              # cream caption glyphs on the dark title bar
  heap_is_object_alive              # script heap liveness check (used by animator)
  animator_skip_dead_tracks         # Android: drop freed animator tracks, no panic
  android_image_force_sync          # Android: decode images synchronously
  android_ime_dismiss               # Android: one keyboard, stays hidden once dismissed
  android_asset_load_prefix         # Android: assets under the package root; as_mut_ptr read
  android_share_and_content_uri     # Android: share sheet + content:// reads (MakepadActivity-share.java)
  portal_list_stop_at_range         # content-sized PortalList
  portal_list_claim_wheel           # nested list scroll does not also scroll the page
  scroll_touch_over_buttons         # Android: drag-scroll starting on a button
  view_scroll_pos_getters           # ViewRef scroll position getters (keyboard scroll)
  text_flow_paragraph_align         # RTL prose alignment
  text_input_label_align_pub        # RTL input alignment
  text_flow_selection               # selectable text: opaque band, owns the drag
  text_input_caret_and_placeholder  # caret blinks and is visible; wrapped placeholder not clipped
  button_label_center               # fixed-width buttons centre their label; wrap CTAs keep Fit label (Fill+wrap freezes UI)
  screen_cap_picture_only_fallback  # Linux Shift+F12 recording without audio track
  wayland_maximize_toggle           # Wayland CSD maximize/restore + caption double-click
  mouse_back_button                 # mouse thumb button maps to Back
  d3d11_device_fallback             # Windows: create D3D11 without VIDEO_SUPPORT if refused
  text_input_float_caption          # filled fields keep a small label (their placeholder's short form)
  drag_scroll_touch_only            # vertical: touch-only drag-scroll; horizontal strips: mouse on whitespace
  android_keep_screen_on            # Android: Cx::keep_screen_on while serving (MakepadActivity-keep-awake.java)
  android_write_content_uri         # Android: Cx::write_content_uri for "Save a backup copy" (MakepadActivity-write.java)
)
for name in "${PATCHES[@]}"; do
  patch -p1 --forward --no-backup-if-mismatch --quiet -d "$VENDOR" \
    < "$ROOT/patches/makepad/$name.rs.patch"
  echo "Applied $name"
done

# Real API migrations for newer rustc (associated MAX/MIN, Atomic::try_update).
bash "$ROOT/scripts/fix-makepad-rust-deprecations.sh"

echo "Done. Makepad 2 at vendor/makepad (stock direction, no invert_scroll)."
