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
  android_ime_retap_show            # Android: retap field must re-show IME (do not dedupe showKeyboard on last_ime_config alone)
  android_ime_keep_height           # Android: ignore spurious DidShow(0) while IME open (adjustNothing/landscape)
  android_ime_field_switch          # Android: field-to-field focus keeps the IME up (no hide→show, no false dismiss)
  keyboard_view_focus_rect          # KeyboardView: stock field lift; every hide snaps the shift to 0 (no stalled ease)
  android_api26_render_loop         # Android API 26–28: guard bad refresh rates in simple frame pump
  android_post_layout_redraw        # Android: extra redraws after first drawable so layout paints
  android_asset_load_prefix         # Android: assets under the package root; as_mut_ptr read
  android_load_asset_pub            # Android: pub load_asset for menu shelf/templates extract
  android_share_and_content_uri     # Android: share sheet + content:// reads (MakepadActivity-share.java)
  portal_list_stop_at_range         # content-sized PortalList
  portal_list_claim_wheel           # nested list scroll does not also scroll the page
  scroll_touch_over_buttons         # Android: drag-scroll starting on a button
  view_scroll_pos_getters           # ViewRef scroll position getters (keyboard scroll)
  text_flow_paragraph_align         # RTL prose alignment
  text_input_label_align_pub        # RTL input alignment
  text_flow_selection               # selectable text: opaque band, owns the drag
  text_flow_touch_scroll            # Android/touch: swipe scrolls; long-press selects (WrapLabel)
  text_input_touch_scroll           # Android/touch: swipe over fields scrolls page; long-press selects (PIN / pair)
  text_input_caret_and_placeholder  # caret blinks and is visible; wrapped placeholder not clipped
  button_label_center               # fixed-width buttons centre their label; wrap CTAs keep Fit label (Fill+wrap freezes UI)
  turtle_wrap_fill_no_spam          # wrap+Fill: log once, never error!-spam every frame (that freezes Settings → Add printer)
  screen_cap_picture_only_fallback  # Linux Shift+F12 recording without audio track
  wayland_maximize_toggle           # Wayland CSD maximize/restore + caption double-click
  mouse_back_button                 # mouse thumb button maps to Back
  d3d11_device_fallback             # Windows: create D3D11 without VIDEO_SUPPORT if refused
  text_input_float_caption          # filled fields keep a small label (their placeholder's short form)
  drag_scroll_touch_only            # page mouse press-drag; lists touch-fling; capped Suggest/StationCheck opt in mouse_drag_scrolling
  portal_list_mouse_drag_promote    # capped list mouse press-drag: promote capture so page ScrollYView cannot steal it
  android_keep_screen_on            # Android: Cx::keep_screen_on while serving (MakepadActivity-keep-awake.java)
  android_write_content_uri         # Android: Cx::write_content_uri for "Save a backup copy" (MakepadActivity-write.java)
  android_print                     # Android: PrintManager + Bluetooth SPP + USB host (MakepadActivity-print.java)
  mouse_cursor_state                # Wayland Arrow=Default; edge resize keeps cursor; HoverOver re-asserts Text/Hand/View cursor
  scroll_bar_orphan_unblock         # wheel: clear stuck TextFlow Empty scroll-block (thumb drag still worked)
  scroll_touch_orphan_unblock       # Android: same Empty orphan clear on touch drag (no wheel)
  scroll_axis_lock                  # opt-in: drag direction lock for nested X/Y scroll views; Shift+wheel sideways
  # Hand edits that lived only in vendor/makepad (gitignored). Captured so a
  # fresh setup-makepad.sh no longer wipes touch / SSD / tooltip / button fixes.
  event_window_server_decorations   # Event::WindowServerDecorations (Wayland SSD)
  finger_touch_tap_distance         # touch tap slop 24px (finger jitter vs mouse)
  wayland_wl_touch                  # real wl_touch path (scroll / long-press / fling)
  wayland_compositor_imposed        # maximized/snapped: do not overwrite compositor size
  wayland_configure_and_touch       # configure commit + touch dispatch on Wayland
  xlib_touch_and_ssd                # X11 Touch + ServerDecorations event variants
  x11_touch_dispatch                # dispatch TouchUpdate on X11
  x11_drag_query_current_pos        # X11: caption hit-test at the motion position (jumped pointer clicked header buttons as window drag)
  win32_touch_event                 # Win32 Touch event variant
  win32_touch_mouse_filter          # drop mouse messages synthesized from touch/pen
  windows_touch_dispatch            # dispatch TouchUpdate on Windows
  window_hide_caption_under_ssd     # hide CSD caption when compositor draws the title bar
  button_honor_grab_key_focus       # grab_key_focus:false must not call set_key_focus
  callout_tooltip_wrap_cap          # wrap long tips; max bubble width 320
  no_warn_shared_bytes              # wasm: drop unused Read/map_file stub (no allow)
  no_warn_blocking_http_wasm        # wasm: gate native HTTP helpers; keep builder API live
  no_warn_gpu_texture_wasm          # wasm: TextureAlloc imports only where adopt paths compile
  slides_view_story_axis            # sites: SlidesView axis Vertical, tap/swipe/wheel flips, presenter keys (PgDn/PgUp/Space/Home/End), goal_slide/slide_count
  web_init_window_order             # sites (wasm): Startup creates window 0 before the init geom is applied (else IdPool panic, stuck on Loading)
  web_aux_buttons_stay              # wasm: mouse side buttons do not navigate the browser away from the app
  d3d11_warp_fallback               # Windows: no usable adapter 0 (VMs, broken DXGI) → CPU WARP device instead of a panic
  # Press / touch / UI-thread doctrine (docs/DEV.md § Press, touch and a responsive UI thread).
  press_capture_cancel              # a press a scroll view takes over gets Hit::FingerCancel (never a click); TOUCH_SCROLL_SLOP
  button_press_lifecycle            # pressed face follows the finger: 80 ms touch delay, drag-off, cancel, heal when orphaned
  text_flow_touch_focus             # copy takes focus on tap / long-press, not on press (swipe keeps the soft keyboard)
  touch_scroll_slop                 # scroll views + PortalList take a finger only past 8 px (2 px / 5 px cancelled taps)
  widget_tree_name_index            # cold path lookups via a name index; remembered answers re-validated (no tree walk per lookup)
  android_jni_detach_on_exit        # Android: a native thread attach_jni_env attached detaches at exit (ART aborted on worker exit)
)
for name in "${PATCHES[@]}"; do
  patch -p1 --forward --no-backup-if-mismatch --quiet -d "$VENDOR" \
    < "$ROOT/patches/makepad/$name.rs.patch"
  echo "Applied $name"
done

# Real API migrations for newer rustc (associated MAX/MIN, Atomic::try_update).
bash "$ROOT/scripts/fix-makepad-rust-deprecations.sh"

echo "Done. Makepad 2 at vendor/makepad (stock direction, no invert_scroll)."
