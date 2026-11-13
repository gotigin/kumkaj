#!/usr/bin/env bash
# Product Certification Kit — WACK-class gate before Linux / Play / Store ship.
#
#   ./scripts/product-cert.sh                 # all apps, default tiers
#   ./scripts/product-cert.sh khaynjo         # one app
#   ./scripts/product-cert.sh --tier=fast     # pre-push / agent gate
#   ./scripts/product-cert.sh --tier=full     # fast+ui+visual+resilience
#   ./scripts/product-cert.sh --tier=store    # full + cross-compile + store asserts
#
# Exit 0 only when every selected tier passes. Reports under local/product-cert/.
# Bash only — see .cursor/rules/no-python-scripts.mdc.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ALL_APPS=(khaynjo moklanjo kirano tabiyat)
APPS=()
TIER="default" # default = fast+ui+visual+resilience
STAMP="$(date +%Y%m%d-%H%M%S)"
REPORT_ROOT="$ROOT/local/product-cert"

usage() {
  sed -n '2,14p' "$0"
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage ;;
    --tier) TIER="${2:-}"; shift 2 ;;
    --tier=*) TIER="${1#--tier=}"; shift ;;
    khaynjo|moklanjo|kirano|tabiyat) APPS+=("$1"); shift ;;
    all) APPS=("${ALL_APPS[@]}"); shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

[[ ${#APPS[@]} -eq 0 ]] && APPS=("${ALL_APPS[@]}")

want_fast=0
want_ui=0
want_visual=0
want_resilience=0
want_xcomp=0
want_store=0
want_linux=0
want_android=0

case "$TIER" in
  fast) want_fast=1 ;;
  ui) want_fast=1; want_ui=1 ;;
  visual) want_fast=1; want_ui=1; want_visual=1 ;;
  resilience) want_fast=1; want_ui=1; want_resilience=1 ;;
  full) want_fast=1; want_ui=1; want_visual=1; want_resilience=1; want_linux=1 ;;
  linux) want_linux=1 ;;
  android) want_android=1 ;;
  store|package)
    want_fast=1; want_ui=1; want_visual=1; want_resilience=1; want_xcomp=1; want_store=1
    want_linux=1; want_android=1
    export CERT_MULTI_SIZE="${CERT_MULTI_SIZE:-1}"
    ;;
  default) want_fast=1; want_ui=1; want_visual=1; want_resilience=1 ;;
  *) echo "unknown tier: $TIER (fast|ui|visual|resilience|full|store|default|linux|android)" >&2; exit 2 ;;
esac

bin_name_for() {
  case "$1" in
    khaynjo) echo kumkaj-khaynjo ;;
    moklanjo) echo kumkaj-moklanjo ;;
    kirano) echo kumkaj-kirano ;;
    tabiyat) echo kumkaj-tabiyat ;;
    *) return 1 ;;
  esac
}

pkg_id_for() {
  case "$1" in
    khaynjo) echo com.khaynjo ;;
    moklanjo) echo com.moklanjo ;;
    kirano) echo shop.kirano ;;
    tabiyat) echo shop.tabiyat ;;
    *) return 1 ;;
  esac
}

crate_for() {
  case "$1" in
    khaynjo) echo kumkaj-khaynjo ;;
    moklanjo) echo kumkaj-moklanjo ;;
    kirano) echo kumkaj-kirano ;;
    tabiyat) echo kumkaj-tabiyat ;;
    *) return 1 ;;
  esac
}

have_cmd() { command -v "$1" >/dev/null 2>&1; }

log() { printf '%s\n' "$*"; }

fail_tier() {
  local app="$1" tier="$2" msg="$3"
  echo "CERT FAIL [$app/$tier]: $msg" | tee -a "$SUMMARY"
  echo "$msg" >"$OUT/$tier.FAIL"
  return 1
}

pass_tier() {
  local app="$1" tier="$2"
  echo "CERT PASS [$app/$tier]" | tee -a "$SUMMARY"
  echo ok >"$OUT/$tier.PASS"
}

# ---------- Fast ----------
run_fast() {
  local app="$1" crate out rc
  crate="$(crate_for "$app")"
  out="$OUT/fast"
  mkdir -p "$out"
  # Subshell + set -e: a `{ … } || fail` group disables errexit in bash, so a
  # mid-group `cargo test` failure used to still CERT PASS when the last oracle
  # succeeded.
  set +e
  (
    set -euo pipefail
    echo "== lock-guards =="
    bash "$ROOT/apps/$app/scripts/check-lock-guards.sh"
    echo "== ui-translations =="
    bash "$ROOT/scripts/check-ui-translations.sh" --app "$app"
    echo "== never-print =="
    bash "$ROOT/apps/$app/scripts/check-tests-never-print.sh"
    echo "== cargo test --lib =="
    cargo test -p "$crate" --lib --no-fail-fast
    echo "== cargo test --bins (shell layout / popup oracles) =="
    # Never fall back to bare `cargo test` — that re-runs --lib and can hang a
    # GUI binary on the developer's DISPLAY (futex) while fast cert waits.
    # Integration crates under tests/ are journeys/seeds — run via script below.
    cargo test -p "$crate" --bins --no-fail-fast
    echo "== user-journeys =="
    bash "$ROOT/scripts/run-user-journeys.sh" "$app"
    echo "== Play crash mitigations present (source) =="
    # Non-poison Mutex + catch_unwind on handle_event (Khaynjo Play reject lesson).
    if [[ -f "$ROOT/apps/$app/src/main.rs" ]]; then
      grep -q 'PoisonError::into_inner' "$ROOT/apps/$app/src/main.rs" \
        || { echo "missing non-poison Mutex in main.rs" >&2; exit 1; }
    fi
    if [[ -f "$ROOT/apps/$app/src/app/main_impl.rs" ]]; then
      grep -q 'catch_unwind' "$ROOT/apps/$app/src/app/main_impl.rs" \
        || { echo "missing catch_unwind in main_impl.rs" >&2; exit 1; }
      grep -q 'rail_expand_btn' "$ROOT/apps/$app/src/app/main_impl.rs" \
        || { echo "rail_expand_btn missing from HEADER_CONTROLS" >&2; exit 1; }
    fi
    echo "== UX glitch oracles (SegBtn / footer lane) =="
    if [[ -f "$ROOT/apps/$app/src/ui_script.rs" ]]; then
      # SegBtn labels must never clip (Gujarati / phone): either no wrap
      # (Moklanjo) or wrap with a Fit-height face (Khaynjo / Kirano /
      # Tabiyat since 25fcce2). A fixed 40-px Overlay shell clips either way.
      seg_block="$(awk '
        /let SegBtn = Button\{/ {inseg=1; next}
        inseg && /let [A-Za-z]+ = / {exit}
        inseg {print}
      ' "$ROOT/apps/$app/src/ui_script.rs")"
      if grep -q 'wrap: true' <<<"$seg_block" && ! grep -q 'height: Fit' <<<"$seg_block"; then
        echo "SegBtn wraps on a fixed-height face — wrapped labels will clip" >&2
        exit 1
      fi
      if grep -q 'height: 40 flow: Overlay' "$ROOT/apps/$app/src/ui_script.rs"; then
        echo "height: 40 Overlay shell clips SegBtn/ChipBtn labels" >&2
        exit 1
      fi
      if grep -q 'bottom_nav_scroll := ScrollXView{' "$ROOT/apps/$app/src/ui_script.rs"; then
        awk '
          /bottom_nav_scroll := ScrollXView\{/ {innav=1; next}
          innav && /nav_books_slot|nav_sell_slot|nav_home_slot/ {exit}
          innav {buf = buf $0 ORS}
          END {
            if (buf !~ /mouse_capture_overload: true/) exit 1
            if (buf !~ /axis_lock: true/) exit 1
          }
        ' "$ROOT/apps/$app/src/ui_script.rs" \
          || { echo "bottom_nav_scroll missing capture_overload/axis_lock" >&2; exit 1; }
      fi
      # Expense/split pay chips: must not regress to 2×2 Fill (uneven Cash vs UPI).
      if grep -q 'part_cash := SegBtn' "$ROOT/apps/$app/src/ui_script.rs"; then
        grep -qE 'part_methods_scroll := (ScrollXView|FormScrollX)\{' "$ROOT/apps/$app/src/ui_script.rs" \
          || { echo "PartsList methods missing part_methods_scroll (2×2 Fill regress)" >&2; exit 1; }
      fi
      # FieldInput wrap:true → placeholder wraps past Fixed height and crushes the next field.
      if grep -q 'let FieldInput = TextInput{' "$ROOT/apps/$app/src/ui_script.rs"; then
        awk '
          /let FieldInput = TextInput\{/ {inf=1; next}
          inf && /let [A-Za-z]+ = / {exit}
          inf && /wrap: true/ {found=1; exit}
          END {exit found?0:1}
        ' "$ROOT/apps/$app/src/ui_script.rs" \
          && { echo "FieldInput still has wrap:true — placeholders will clip" >&2; exit 1; }
        # Placeholder/value type ≤14 (was 16 — crowded phone wells).
        awk '
          /let FieldInput = TextInput\{/ {inf=1; next}
          inf && /let [A-Za-z]+ = / {exit}
          inf && /text_style \+: \{font_size: ([0-9.]+)/ {
            if ($0 ~ /font_size: 1[5-9]\.|font_size: [2-9][0-9]/) found=1
          }
          END {exit found?0:1}
        ' "$ROOT/apps/$app/src/ui_script.rs" \
          && { echo "FieldInput draw_text font_size still >14 — placeholders too large on phone" >&2; exit 1; }
      fi
      # Live nav drag must stay off — reorder is Settings → Sidebar ↑↓.
      for impl in "$ROOT/apps/$app/src/app/impl_05.rs" "$ROOT/apps/$app/src/app/impl_06.rs"; do
        [[ -f "$impl" ]] || continue
        grep -q 'fn pointer_nav_drag' "$impl" || continue
        if awk '
          /fn pointer_nav_drag\(/ {inn=1; next}
          inn && /^    fn / {exit}
          inn && /Phase::Press|nav_drag_press\(/ {found=1; exit}
          END {exit found?0:1}
        ' "$impl"; then
          echo "pointer_nav_drag still runs live chrome drag in $impl — must stay no-op" >&2
          exit 1
        fi
      done
      if grep -q 'bottom_nav := SolidView{' "$ROOT/apps/$app/src/ui_script.rs" 2>/dev/null; then
        grep -q 'move_up := QuietFitBtn' "$ROOT/apps/$app/src/ui_script.rs" \
          || { echo "Settings Sidebar missing ↑ — chrome reorder has no non-drag path" >&2; exit 1; }
      fi
      # Order −/+ must skip full reload_all and show Progress+Cancel (DEV 7b).
      if grep -q 'BumpPending' "$ROOT/apps/$app/src/net.rs" 2>/dev/null; then
        grep -q 'fn running_action_skips_full_reload' "$ROOT/apps/$app/src/net.rs" \
          || { echo "running_action_skips_full_reload missing — −/+ will freeze on reload_all" >&2; exit 1; }
        grep -q 'BumpPending' "$ROOT/apps/$app/src/net.rs" \
          && grep -A6 'fn running_action_skips_full_reload' "$ROOT/apps/$app/src/net.rs" \
            | grep -q 'BumpPending' \
          || { echo "BumpPending not in running_action_skips_full_reload" >&2; exit 1; }
        grep -q 'stepper_busy :=' "$ROOT/apps/$app/src/ui_script.rs" \
          || { echo "CartRow stepper_busy missing — no Removing…/Cancel on −/+" >&2; exit 1; }
      fi
    fi
  ) >"$out/log.txt" 2>&1
  rc=$?
  set -e
  if [[ "$rc" -ne 0 ]]; then
    fail_tier "$app" fast "see $out/log.txt"
    return 1
  fi
  pass_tier "$app" fast
}

# ---------- UI smoke (launch-to-stable + sidebar + peers) ----------
run_ui() {
  local app="$1" out
  out="$OUT/ui"
  mkdir -p "$out"
  if ! have_cmd Xvfb; then
    echo "CERT SKIP [$app/ui]: Xvfb not installed" | tee -a "$SUMMARY"
    echo skipped >"$out/SKIP"
    return 0
  fi
  # peers: cold launch + sidebar collapse/expand + chrome tour
  if ! SMOKE_SCRIPT=peers SMOKE_OUT="$out/peers" \
      bash "$ROOT/scripts/run-ui-smoke-isolated.sh" "$app" \
      >"$out/peers.log" 2>&1; then
    fail_tier "$app" ui "peers smoke failed — $out/peers.log"
    return 1
  fi
  # Launch-to-stable: unlocked shot + PASS marker + no early death
  if [[ ! -f "$out/peers/PASS" ]] && [[ ! -f "$out/peers"/*/PASS ]]; then
    # PASS may sit one level under peers/<kind>/
    if ! find "$out/peers" -name PASS -print -quit | grep -q .; then
      fail_tier "$app" ui "peers smoke produced no PASS — launch unstable?"
      return 1
    fi
  fi
  local shot_n
  shot_n="$(find "$out/peers" -name '*.png' 2>/dev/null | wc -l | tr -d ' ')"
  if [[ "${shot_n:-0}" -lt 1 ]]; then
    fail_tier "$app" ui "peers smoke produced no PNG shots"
    return 1
  fi
  # full action matrix when SMOKE_FULL=1 (default on for ui+)
  if [[ "${SMOKE_FULL:-1}" == "1" ]]; then
    if ! SMOKE_SCRIPT=full SMOKE_OUT="$out/full" \
        bash "$ROOT/scripts/run-ui-smoke-isolated.sh" "$app" \
        >"$out/full.log" 2>&1; then
      fail_tier "$app" ui "full smoke failed — $out/full.log"
      return 1
    fi
  fi
  pass_tier "$app" ui
}

# ---------- Visual baselines + multi-size Xvfb ----------
run_visual() {
  local app="$1" out base
  out="$OUT/visual"
  base="$ROOT/apps/$app/cert/baselines"
  mkdir -p "$out"
  if ! have_cmd Xvfb; then
    echo "CERT SKIP [$app/visual]: Xvfb not installed" | tee -a "$SUMMARY"
    echo skipped >"$out/SKIP"
    return 0
  fi
  # Multi-size peers (phone / tablet / desktop) — opt-in; store sets CERT_MULTI_SIZE=1
  local size failed=0
  if [[ "${CERT_MULTI_SIZE:-0}" == "1" ]]; then
    for size in "360x640" "800x1280" "1280x900"; do
      local w="${size%x*}" h="${size#*x}"
      local sdir="$out/size-$size"
      mkdir -p "$sdir"
      if ! SMOKE_SCRIPT=peers SMOKE_OUT="$sdir" \
          SMOKE_XVFB_SIZE="${w}x${h}x24" \
          bash "$ROOT/scripts/run-ui-smoke-isolated.sh" "$app" \
          >"$sdir/log.txt" 2>&1; then
        echo "size $size FAILED" >>"$out/log.txt"
        failed=1
      else
        echo "size $size OK" >>"$out/log.txt"
      fi
    done
    if [[ "$failed" -ne 0 ]]; then
      fail_tier "$app" visual "multi-size peers failed — $out/log.txt"
      return 1
    fi
  else
    echo "CERT NOTE [$app/visual]: skip multi-size (set CERT_MULTI_SIZE=1)" >>"$out/log.txt"
  fi
  if [[ ! -d "$base" ]] || [[ -z "$(find "$base" -name '*.png' 2>/dev/null | head -1)" ]]; then
    echo "CERT NOTE [$app/visual]: no golden baselines in apps/$app/cert/baselines yet" | tee -a "$SUMMARY"
    pass_tier "$app" visual
    return 0
  fi
  if ! have_cmd compare && ! have_cmd magick; then
    fail_tier "$app" visual "ImageMagick compare/magick required for baselines"
    return 1
  fi
  local smoke_pngs="$OUT/ui/peers"
  [[ -d "$smoke_pngs" ]] || smoke_pngs="$out/size-1280x900"
  failed=0
  while IFS= read -r -d '' ref; do
    local name dest
    name="$(basename "$ref")"
    dest="$(find "$OUT/ui" "$out" -name "$name" 2>/dev/null | head -1 || true)"
    if [[ -z "$dest" || ! -f "$dest" ]]; then
      echo "missing shot for baseline $name" >>"$out/log.txt"
      failed=1
      continue
    fi
    local metric
    if have_cmd compare; then
      metric="$(compare -metric AE "$ref" "$dest" "$out/diff-$name" 2>&1 || true)"
    else
      metric="$(magick compare -metric AE "$ref" "$dest" "$out/diff-$name" 2>&1 || true)"
    fi
    echo "$name AE=$metric" >>"$out/log.txt"
    if [[ "$metric" =~ ^[0-9]+$ ]] && [[ "$metric" -gt 50000 ]]; then
      failed=1
    fi
  done < <(find "$base" -name '*.png' -print0)
  if [[ "$failed" -ne 0 ]]; then
    fail_tier "$app" visual "baseline mismatch — $out/log.txt"
    return 1
  fi
  pass_tier "$app" visual
}

# ---------- Resilience (fault injection + soak + log scrape) ----------
run_resilience() {
  local app="$1" out
  out="$OUT/resilience"
  mkdir -p "$out"
  if ! have_cmd Xvfb; then
    echo "CERT SKIP [$app/resilience]: Xvfb not installed" | tee -a "$SUMMARY"
    echo skipped >"$out/SKIP"
    return 0
  fi
  # Fault injection lives in full smoke (empty-cart / empty-desk refuse,
  # settings printers panel, nav back). Re-run full here so resilience
  # stands alone when UI tier was skipped; ignore if already green.
  if [[ "${CERT_RESILIENCE_SKIP_FULL:-0}" != "1" ]]; then
    if ! SMOKE_SCRIPT=full SMOKE_OUT="$out/faults" \
        bash "$ROOT/scripts/run-ui-smoke-isolated.sh" "$app" \
        >"$out/faults.log" 2>&1; then
      fail_tier "$app" resilience "fault full smoke failed — $out/faults.log"
      return 1
    fi
    if ! find "$out/faults" \( -name '*107_*' -o -name 'steps.log' -o -name '*.log' \) \
        -print0 2>/dev/null | xargs -0 grep -E -l '107_empty_|104_settings|105_settings' \
        >/dev/null 2>&1; then
      # Soft: older smoke without 107 still OK if full PASS exists.
      if ! find "$out/faults" -name PASS -print -quit | grep -q .; then
        fail_tier "$app" resilience "fault smoke missing PASS / empty-cart marks"
        return 1
      fi
      echo "CERT NOTE [$app/resilience]: no 107_* mark (smoke may predate fault step)" >>"$out/log.txt"
    fi
  fi
  if ! KUMKAJ_SMOKE_SOAK_SECS=60 SMOKE_SCRIPT=soak SMOKE_OUT="$out/soak" \
      bash "$ROOT/scripts/run-ui-smoke-isolated.sh" "$app" >"$out/soak.log" 2>&1; then
    fail_tier "$app" resilience "soak failed — $out/soak.log"
    return 1
  fi
  if grep -E 'panicked at|SIGSEGV|^\[E\] ' "$out/soak.log" "$out/faults.log" 2>/dev/null | grep -q .; then
    fail_tier "$app" resilience "panic or [E] in soak/fault log"
    return 1
  fi
  # App log under soak/<kind>.log
  if find "$out/soak" "$out/faults" -name '*.log' -print0 2>/dev/null \
      | xargs -0 grep -E -l 'panicked at|SIGSEGV|^\[E\] ' >/dev/null 2>&1; then
    fail_tier "$app" resilience "panic or [E] in soak/fault app log"
    return 1
  fi
  pass_tier "$app" resilience
}

# ---------- Cross-compile / artifact presence ----------
run_xcomp() {
  local app="$1" out rc
  out="$OUT/xcomp"
  mkdir -p "$out"
  set +e
  (
    set -euo pipefail
    echo "== android package metadata =="
    grep -E 'version_code|version_name|^version' "$ROOT/apps/$app/Cargo.toml" || true
    echo "== play version_code not date/auto =="
    vc="$(gawk '/\[package.metadata.makepad.android\]/{p=1;next} p&&/^\[/{exit} p&&/version_code/{gsub(/[^0-9]/,"",$3);print $3;exit}' "$ROOT/apps/$app/Cargo.toml")"
    if [[ -z "$vc" ]] || [[ "$vc" =~ ^[0-9]{8,} ]] || [[ "$vc" == "auto" ]]; then
      echo "bad version_code='$vc'" >&2
      exit 1
    fi
    echo "version_code=$vc ok"
    echo "== windows scripts present =="
    [[ -f "$ROOT/apps/$app/scripts/msi.sh" ]]
    [[ -f "$ROOT/apps/$app/scripts/msix.sh" ]]
    echo "== android package script present =="
    [[ -f "$ROOT/apps/$app/scripts/package-android.sh" ]]
    echo "== apple scaffold script =="
    [[ -f "$ROOT/scripts/package-apple-scaffold.sh" ]]
    echo "== linux package script =="
    [[ -f "$ROOT/apps/$app/scripts/package-linux.sh" ]]
  ) >"$out/log.txt" 2>&1
  rc=$?
  set -e
  if [[ "$rc" -ne 0 ]]; then
    fail_tier "$app" xcomp "see $out/log.txt"
    return 1
  fi
  pass_tier "$app" xcomp
}

# ---------- Linux packaged binary short Xvfb launch ----------
run_linux_pkg() {
  local app="$1" out bin_name pkg_dir exe disp_num disp xvfb_pid app_pid log
  out="$OUT/linux"
  mkdir -p "$out"
  bin_name="$(bin_name_for "$app")"
  pkg_dir="$ROOT/dist/${app}-linux"
  if [[ ! -d "$pkg_dir" ]]; then
    if [[ "${CERT_BUILD_LINUX:-0}" == "1" ]]; then
      bash "$ROOT/apps/$app/scripts/package-linux.sh" >"$out/package.log" 2>&1 || {
        fail_tier "$app" linux "package-linux failed — $out/package.log"
        return 1
      }
    else
      echo "CERT SKIP [$app/linux]: no $pkg_dir (set CERT_BUILD_LINUX=1 to build)" | tee -a "$SUMMARY"
      echo skipped >"$out/SKIP"
      return 0
    fi
  fi
  if ! have_cmd Xvfb; then
    echo "CERT SKIP [$app/linux]: Xvfb not installed" | tee -a "$SUMMARY"
    echo skipped >"$out/SKIP"
    return 0
  fi
  exe="$pkg_dir/bin/$bin_name"
  [[ -x "$exe" ]] || exe="$pkg_dir/$bin_name"
  if [[ ! -x "$exe" ]]; then
    fail_tier "$app" linux "packaged binary missing under $pkg_dir"
    return 1
  fi
  for disp_num in 94 93 92; do
    if [[ ! -e "/tmp/.X${disp_num}-lock" && ! -S "/tmp/.X11-unix/X${disp_num}" ]]; then
      break
    fi
  done
  disp=":${disp_num}"
  Xvfb "$disp" -screen 0 1280x900x24 -nolisten tcp >/dev/null 2>&1 &
  xvfb_pid=$!
  sleep 0.4
  log="$out/launch.log"
  # Package root must be cwd so self:resources/ resolves.
  (
    cd "$pkg_dir"
    env -u WAYLAND_DISPLAY DISPLAY="$disp" KUMKAJ_ENV=production \
      KUMKAJ_PRINT_SINK=off \
      "$exe" >"$log" 2>&1
  ) &
  app_pid=$!
  sleep 5
  if ! kill -0 "$app_pid" 2>/dev/null; then
    wait "$app_pid" 2>/dev/null || true
    kill "$xvfb_pid" 2>/dev/null || true
    if grep -E 'panicked at|SIGSEGV|^\[E\] ' "$log" >/dev/null 2>&1; then
      fail_tier "$app" linux "packaged binary panicked — $log"
      return 1
    fi
    if [[ "${CERT_REQUIRE_LINUX_PKG:-0}" == "1" ]]; then
      fail_tier "$app" linux "packaged binary exited early — $log"
      return 1
    fi
    echo "CERT NOTE [$app/linux]: packaged binary exited early (see $log)" | tee -a "$SUMMARY"
    pass_tier "$app" linux
    return 0
  fi
  # Alive is not enough: startup deadlock keeps a pid with no window
  # (Play "keeps stopping" / desktop Not Responding). Prefer xlsclients;
  # fall back to the root tree. No display clients → treat as hang.
  local win_ok=0 win_dump=""
  if have_cmd xlsclients; then
    win_dump="$(DISPLAY="$disp" xlsclients 2>/dev/null || true)"
    if printf '%s\n' "$win_dump" | grep -qiE "$bin_name|${app}|makepad"; then
      win_ok=1
    elif [[ -n "$(printf '%s\n' "$win_dump" | tr -d '[:space:]')" ]]; then
      # Some builds register an opaque client name — any client on this
      # fresh Xvfb besides the server means a window mapped.
      win_ok=1
    fi
  elif have_cmd xwininfo; then
    win_dump="$(DISPLAY="$disp" xwininfo -root -tree 2>/dev/null || true)"
    if printf '%s\n' "$win_dump" | grep -qiE "$bin_name|${app}|Makepad|0x[0-9a-f]+.*: \(.*\)  [1-9]"; then
      win_ok=1
    fi
  else
    win_ok=1 # no probe tools — cannot prove a hang; pid check only
    echo "CERT NOTE [$app/linux]: install xlsclients/xwininfo to catch no-window hangs" | tee -a "$SUMMARY"
  fi
  if [[ "$win_ok" -ne 1 ]]; then
    kill -9 "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
    printf '%s\n' "$win_dump" >"$out/windows.txt"
    fail_tier "$app" linux "packaged binary alive but no window (hang?) — $log"
    return 1
  fi
  kill "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
  kill "$xvfb_pid" 2>/dev/null || true
  wait "$xvfb_pid" 2>/dev/null || true
  if grep -E 'panicked at|SIGSEGV|^\[E\] ' "$log" >/dev/null 2>&1; then
    fail_tier "$app" linux "panic/[E] in packaged launch log"
    return 1
  fi
  echo "packaged launch stayed up 5s with a window" >"$out/log.txt"
  pass_tier "$app" linux
}

# Store / Play must not go green when Android launch was never proven.
android_skip() {
  local app="$1" reason="$2" out="$3"
  echo "CERT SKIP [$app/android]: $reason" | tee -a "$SUMMARY"
  echo skipped >"$out/SKIP"
  if [[ "${CERT_REQUIRE_ADB:-0}" == "1" ]]; then
    fail_tier "$app" android "required: $reason"
    return 1
  fi
  if [[ "${want_store:-0}" -eq 1 && "${CERT_ALLOW_ANDROID_SKIP:-0}" != "1" ]]; then
    fail_tier "$app" android \
      "store tier needs device launch smoke ($reason). Plug a device or set CERT_ALLOW_ANDROID_SKIP=1"
    return 1
  fi
  return 0
}

# ---------- Android device/emulator launch smoke ----------
# Alive pid alone used to pass — a startup deadlock keeps the process and
# Play still sees "keeps stopping" after the hang. Fail on crash, ANR, or
# process-without-resumed-UI.
run_android_device() {
  local app="$1" out apk pkg adb_bin serial pid i logcat_dump focused resumed
  local aab="" apks_zip="" bt_jar="" sdk abi
  out="$OUT/android"
  mkdir -p "$out"
  pkg="$(pkg_id_for "$app")"
  adb_bin="$(command -v adb || true)"
  if [[ -z "$adb_bin" && -n "${ANDROID_HOME:-}" && -x "$ANDROID_HOME/platform-tools/adb" ]]; then
    adb_bin="$ANDROID_HOME/platform-tools/adb"
  fi
  if [[ -z "$adb_bin" ]]; then
    android_skip "$app" "adb not on PATH" "$out" || return 1
    return 0
  fi
  # Prefer a real USB device over an emulator — Play ships the AAB's arm64
  # split; the local makepad APK is often x86_64-only and hides AAB R-id bugs.
  serial="$("$adb_bin" devices 2>/dev/null | awk '/\tdevice$/{if ($1 !~ /^emulator-/) {print $1; exit}}')"
  if [[ -z "$serial" ]]; then
    serial="$("$adb_bin" devices 2>/dev/null | awk '/\tdevice$/{print $1; exit}')"
  fi
  if [[ -z "$serial" ]]; then
    android_skip "$app" "no device/emulator (adb devices empty)" "$out" || return 1
    return 0
  fi
  shopt -s nullglob
  local aabs=("$ROOT/dist/${app}-android"/*.aab)
  local apks=("$ROOT/dist/${app}-android"/*.apk)
  shopt -u nullglob
  apk=""
  if [[ ${#aabs[@]} -gt 0 ]]; then
    aab="${aabs[0]}"
    # shellcheck source=android-env.sh
    source "$ROOT/scripts/android-env.sh"
    android_assert_aab_launch_bg "$aab" || {
      fail_tier "$app" android "AAB launch-bg assert failed — $aab"
      return 1
    }
    sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
    bt_jar=""
    for cand in \
      "$sdk/bundletool/bundletool.jar" \
      "$sdk/sources/bundletool-all-1.18.1.jar" \
      "$HOME/.cache/ghal-bol/makepad-android_33_linux_x64/bundletool/bundletool.jar"; do
      [[ -f "$cand" ]] && bt_jar="$cand" && break
    done
    if [[ -n "$bt_jar" ]]; then
      apks_zip="$out/from-aab.apks"
      rm -f "$apks_zip"
      java -jar "$bt_jar" build-apks --bundle="$aab" --output="$apks_zip" --mode=universal >/dev/null
      rm -rf "$out/from-aab" && mkdir -p "$out/from-aab"
      unzip -qo "$apks_zip" -d "$out/from-aab"
      if [[ -f "$out/from-aab/universal.apk" ]]; then
        apk="$out/from-aab/universal.apk"
      fi
    fi
  fi
  if [[ -z "$apk" && ${#apks[@]} -gt 0 ]]; then
    apk="${apks[0]}"
  fi
  if [[ -z "$apk" || ! -f "$apk" ]]; then
    android_skip "$app" "no APK/AAB under dist/${app}-android" "$out" || return 1
    return 0
  fi
  if [[ "${want_store:-0}" -eq 1 && -z "$aab" ]]; then
    fail_tier "$app" android "store tier needs dist/${app}-android/*.aab (Play path), not only the local APK"
    return 1
  fi
  abi="$("$adb_bin" -s "$serial" shell getprop ro.product.cpu.abi | tr -d '\r')"
  local rc=0
  set +e
  (
    set -euo pipefail
    echo "serial=$serial apk=$apk pkg=$pkg abi=$abi aab=${aab:-none}"
    "$adb_bin" -s "$serial" logcat -c || true
    "$adb_bin" -s "$serial" install -r "$apk"
    "$adb_bin" -s "$serial" shell am force-stop "$pkg" || true
    "$adb_bin" -s "$serial" shell monkey -p "$pkg" -c android.intent.category.LAUNCHER 1 >/dev/null
    # Warm launch: poll — crash drops the pid; hang keeps it without UI.
    pid=""
    for i in 1 2 3 4 5 6 7 8 9 10; do
      sleep 1
      pid="$("$adb_bin" -s "$serial" shell pidof "$pkg" 2>/dev/null | tr -d '\r' | awk '{print $1}')"
      if [[ -z "$pid" ]]; then
        echo "FAIL: $pkg died after ${i}s (crash / keeps stopping)" >&2
        "$adb_bin" -s "$serial" logcat -d -t 120 '*:E' AndroidRuntime:E ActivityManager:E || true
        exit 1
      fi
    done
    echo "pid=$pid after 10s"
    logcat_dump="$("$adb_bin" -s "$serial" logcat -d -t 400 2>/dev/null || true)"
    printf '%s\n' "$logcat_dump" >"$out/logcat.txt"
    if printf '%s\n' "$logcat_dump" | grep -qiE \
      "FATAL EXCEPTION.*${pkg}|AndroidRuntime.*${pkg}|Process ${pkg} has died|Force finishing activity.*${pkg}|ANR in ${pkg}|Application Not Responding.*${pkg}|${pkg}.*(keeps stopping|has stopped)"; then
      echo "FAIL: crash/ANR for $pkg in logcat" >&2
      printf '%s\n' "$logcat_dump" | grep -iE "FATAL|ANR|AndroidRuntime|${pkg}" | tail -40 >&2 || true
      exit 1
    fi
    # Process without a resumed / focused activity = open-then-hang.
    resumed="$("$adb_bin" -s "$serial" shell dumpsys activity activities 2>/dev/null | tr -d '\r' || true)"
    focused="$("$adb_bin" -s "$serial" shell dumpsys window 2>/dev/null | tr -d '\r' || true)"
    if ! printf '%s\n' "$resumed" | grep -qE "mResumedActivity.*${pkg}|topResumedActivity.*${pkg}"; then
      if ! printf '%s\n' "$focused" | grep -qE "mCurrentFocus.*${pkg}|mFocusedApp.*${pkg}"; then
        echo "FAIL: $pkg pid alive but no resumed/focused UI (hang / black launch)" >&2
        printf '%s\n' "$resumed" | grep -E 'mResumedActivity|topResumedActivity' | head -5 >&2 || true
        printf '%s\n' "$focused" | grep -E 'mCurrentFocus|mFocusedApp' | head -5 >&2 || true
        exit 1
      fi
    fi
    echo "resumed_or_focused=ok"
    "$adb_bin" -s "$serial" shell am force-stop "$pkg" || true
  ) >"$out/log.txt" 2>&1
  rc=$?
  set -e
  if [[ "$rc" -ne 0 ]]; then
    fail_tier "$app" android "device launch failed — $out/log.txt"
    return 1
  fi
  pass_tier "$app" android
}

# ---------- Store pre-upload asserts ----------
run_store() {
  local app="$1" out rc
  out="$OUT/store"
  mkdir -p "$out"
  set +e
  (
    set -euo pipefail
    echo "== Play / Android =="
    vc="$(gawk '/\[package.metadata.makepad.android\]/{p=1;next} p&&/^\[/{exit} p&&/version_code/{gsub(/[^0-9]/,"",$3);print $3;exit}' "$ROOT/apps/$app/Cargo.toml")"
    vn="$(gawk '/\[package.metadata.makepad.android\]/{p=1;next} p&&/^\[/{exit} p&&/version_name/{gsub(/"/,"",$3);print $3;exit}' "$ROOT/apps/$app/Cargo.toml")"
    echo "android version_code=$vc version_name=$vn"
    [[ -n "$vc" && "$vc" -ge 1 ]]
    [[ ! "$vc" =~ ^[0-9]{8,} ]]
    aab_dir="dist/${app}-android"
    aab_found=0 sym_found=0 map_found=0
    if [[ -d "$aab_dir" ]]; then
      ls -la "$aab_dir" | head -30
      shopt -s nullglob
      aabs=("$aab_dir"/*.aab)
      shopt -u nullglob
      if [[ ${#aabs[@]} -gt 0 ]]; then
        aab_found=1
        echo "AAB: ${aabs[*]}"
        # shellcheck source=android-env.sh
        source "$ROOT/scripts/android-env.sh"
        for aab in "${aabs[@]}"; do
          android_assert_aab_launch_bg "$aab"
        done
      fi
      [[ -f "$aab_dir/native-debug-symbols.zip" ]] && sym_found=1
      [[ -f "$aab_dir/mapping.txt" ]] && map_found=1
    else
      echo "NOTE: no $aab_dir yet — run apps/$app/scripts/package-android.sh before Play upload"
    fi
    if [[ "${CERT_REQUIRE_AAB:-0}" == "1" || "$app" == "khaynjo" ]]; then
      # Khaynjo Play resubmit: refuse green store cert without AAB+symbols.
      if [[ "$aab_found" -ne 1 ]]; then
        echo "FAIL: AAB missing under $aab_dir (required for store cert)" >&2
        exit 1
      fi
      if [[ "$sym_found" -ne 1 ]]; then
        echo "FAIL: native-debug-symbols.zip missing under $aab_dir" >&2
        exit 1
      fi
      if [[ "$map_found" -ne 1 ]]; then
        echo "WARNING: mapping.txt missing under $aab_dir"
      fi
    fi
    # Refuse store green if UI tier did not pass this run (launch+sidebar).
    if [[ ! -f "$OUT/ui.PASS" ]] && [[ ! -f "$OUT/ui/SKIP" ]]; then
      echo "FAIL: UI tier not green — refuse Play-ready claim" >&2
      exit 1
    fi
    echo "== fonts for desktop packages =="
    if [[ -f "$ROOT/apps/$app/resources/fonts/fa-solid-900.ttf" ]]; then
      echo "fa-solid-900.ttf present"
    else
      echo "WARNING: fa-solid-900.ttf missing — Windows Floor glyphs will be empty boxes"
      echo "run: ./scripts/fetch-ui-fonts.sh apps/$app"
      if [[ "${CERT_REQUIRE_FONTS:-1}" == "1" ]]; then
        exit 1
      fi
    fi
    echo "== MSIX identity documented =="
    case "$app" in
      khaynjo)
        grep -q 'MearajBhagad.Khaynjo' "$ROOT/.cursor/rules/windows-packaging.mdc"
        ;;
      moklanjo)
        grep -q 'MearajBhagad.Moklanjo' "$ROOT/.cursor/rules/windows-packaging.mdc"
        ;;
      kirano)
        grep -q 'MearajBhagad.Kirano' "$ROOT/.cursor/rules/windows-packaging.mdc"
        ;;
      tabiyat)
        grep -q 'MearajBhagad.Tabiyat' "$ROOT/.cursor/rules/windows-packaging.mdc"
        ;;
    esac
    echo "== Apple scaffold runnable =="
    if [[ "${CERT_RUN_APPLE_SCAFFOLD:-0}" == "1" ]]; then
      bash "$ROOT/scripts/package-apple-scaffold.sh" "apps/$app"
    else
      echo "(set CERT_RUN_APPLE_SCAFFOLD=1 to execute scaffold)"
    fi
  ) >"$out/log.txt" 2>&1
  rc=$?
  set -e
  if [[ "$rc" -ne 0 ]]; then
    fail_tier "$app" store "see $out/log.txt"
    return 1
  fi
  pass_tier "$app" store
}

# ---------- per-app ----------
OVERALL=0
mkdir -p "$REPORT_ROOT"
MASTER="$REPORT_ROOT/SUMMARY-$STAMP.txt"
{
  echo "Product Certification Kit"
  echo "stamp=$STAMP tier=$TIER apps=${APPS[*]}"
  echo
} >"$MASTER"

for app in "${APPS[@]}"; do
  OUT="$REPORT_ROOT/$app/$STAMP"
  mkdir -p "$OUT"
  SUMMARY="$OUT/SUMMARY.txt"
  {
    echo "app=$app stamp=$STAMP tier=$TIER"
    echo
  } >"$SUMMARY"
  log "==== $app ===="
  ok=1
  if [[ "$want_fast" -eq 1 ]]; then
    run_fast "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_ui" -eq 1 ]]; then
    run_ui "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_visual" -eq 1 ]]; then
    run_visual "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_resilience" -eq 1 ]]; then
    run_resilience "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_xcomp" -eq 1 ]]; then
    run_xcomp "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_linux" -eq 1 ]]; then
    run_linux_pkg "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_android" -eq 1 ]]; then
    run_android_device "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 && "$want_store" -eq 1 ]]; then
    run_store "$app" || ok=0
  fi
  if [[ "$ok" -eq 1 ]]; then
    echo "APP PASS: $app" | tee -a "$SUMMARY" | tee -a "$MASTER"
  else
    echo "APP FAIL: $app" | tee -a "$SUMMARY" | tee -a "$MASTER"
    OVERALL=1
  fi
  cat "$SUMMARY" >>"$MASTER"
  echo >>"$MASTER"
done

echo
echo "Report: $MASTER"
exit "$OVERALL"
