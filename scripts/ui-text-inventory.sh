#!/usr/bin/env bash
# Visible-text inventory: every long text a screen shows outside an ℹ tip.
#
# Owner rule (DESIGN.md "Explanations live in ℹ"): explanations go in the
# heading's InfoHelp tip — never a grey paragraph, an instruction line or a
# status restating what a control shows. The layout guard
# (`layout_labels_carry_no_explanations`) catches literal text in the layout;
# this catches what code paints at run time. Run it before calling a screen
# clean, on a copy of a NEUTRAL demo shop (never a client's books).
#
#   ./scripts/ui-text-inventory.sh khaynjo /path/to/copy-of-demo-data [PIN]
#
# Launches the debug binary on a private Xvfb with --remote (never :0 /
# Wayland), opens the books as Server, unlocks with PIN (default 1234), walks
# every sidebar screen and Settings panel, prints "## screen" then
# "id<TAB>type<TAB>text" for each visible non-ℹ text of MIN+ chars (default
# 30), and quits with /gq. With apps/<app>/cert/ui-text-allowlist.txt (one
# widget id per line, `#` comments) any other id is NEW and the exit code is
# 1. Only data widgets belong in the allowlist (totals, report rows, a status
# with numbers); an explanation belongs in an ℹ, never in the allowlist.
set -uo pipefail
APP="${1:?usage: $0 <app> <data-dir copy> [PIN]}"
DATA="${2:?usage: $0 <app> <data-dir copy> [PIN]}"
PIN="${3:-1234}"
MIN="${MIN:-30}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/kumkaj-$APP}"
OUT="$(mktemp -d "${TMPDIR:-/tmp}/ui-text-inv.XXXXXX")"
[[ -x "$BIN" ]] || { echo "build first: cargo build -p kumkaj-$APP" >&2; exit 2; }
for t in Xvfb curl xdotool; do command -v "$t" >/dev/null || { echo "need $t" >&2; exit 2; }; done

disp=""
for n in $(seq 140 199); do [[ -e /tmp/.X11-unix/X$n ]] || { disp=":$n"; break; }; done
[[ -n "$disp" ]] || { echo "no free display" >&2; exit 2; }
# Tall enough that a screen fits without scrolling (page keys can move focus).
Xvfb "$disp" -screen 0 1600x3000x24 -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
for _ in $(seq 1 50); do [[ -S /tmp/.X11-unix/X${disp#:} ]] && break; sleep 0.1; done
cd "$ROOT"
env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 DISPLAY="$disp" \
  KUMKAJ_DATA_DIR="$DATA" KUMKAJ_PRINT_SINK=off "$BIN" --remote >"$OUT/app.log" 2>&1 &
app=$!
cleanup() {
  [[ -n "${port:-}" ]] && curl -s "http://127.0.0.1:$port/gq" >/dev/null 2>&1
  sleep 1; kill "$app" "$xvfb" 2>/dev/null
}
trap cleanup EXIT
port=""
for _ in $(seq 1 240); do
  port=$(grep -o 'listening on 127.0.0.1:[0-9]*' "$OUT/app.log" 2>/dev/null | head -1 | grep -o '[0-9]*$')
  [[ -n "$port" ]] && break; sleep 0.25
done
[[ -n "$port" ]] || { echo "app did not start (see $OUT/app.log)" >&2; exit 2; }
r() { curl -s "http://127.0.0.1:$port/$1"; }
widgets() { # id<TAB>type<TAB>x<TAB>y<TAB>w<TAB>h<TAB>text
  r snap | tr '}' '\n' | grep -o '"i":"[^"]*","ty":"[^"]*","r":\[[^]]*\],"w":[0-9]*,"t":"[^"]*"' |
    sed -E 's/"i":"([^"]*)","ty":"([^"]*)","r":\[([^,]*),([^,]*),([^,]*),([^]]*)\],"w":[0-9]*,"t":"(.*)"$/\1\t\2\t\3\t\4\t\5\t\6\t\7/'
}
clk() { # click the first visible widget with this id
  local line x y w h
  line=$(widgets | awk -F'\t' -v id="$1" '$1 == id {print; exit}')
  [[ -z "$line" ]] && return 1
  IFS=$'\t' read -r _ _ x y w h _ <<<"$line"
  r "click?x=$(( ${x%.*} + ${w%.*} / 2 ))&y=$(( ${y%.*} + ${h%.*} / 2 ))&wait=1" >/dev/null
}
long() { widgets | awk -F'\t' -v n="$MIN" '$2 != "InfoHelp" && $2 != "Button" && length($7) >= n {print $1 "\t" $2 "\t" $7}'; }
dump() { long; }

wid=""
for _ in $(seq 1 60); do
  wid=$(DISPLAY=$disp xdotool search --onlyvisible --pid "$app" 2>/dev/null | head -1)
  [[ -z "$wid" ]] && wid=$(DISPLAY=$disp xdotool search --onlyvisible --name "Khaynjo|Kumkaj|Moklanjo|Kirano|Tabiyat" 2>/dev/null | head -1)
  [[ -n "$wid" ]] && break; sleep 0.25
done
[[ -n "$wid" ]] && DISPLAY=$disp xdotool windowsize "$wid" 1600 2900
sleep 3
clk device_role_server_on || clk device_role_server
sleep 3
r "click?x=600&y=284&wait=1" >/dev/null; r "t?t=$PIN" >/dev/null; sleep 0.3; clk pin_ok; sleep 4
{
  rails=$(widgets | awk -F'\t' '$1 ~ /^rail_/ && $2 == "Button" {print $1 "\t" $7}')
  while IFS=$'\t' read -r id label; do
    [[ -z "$id" || "$label" =~ ^[[:space:]]*(Lock)$ ]] && continue
    clk "$id" || continue; sleep 1.5
    echo "## $label"; dump
    on_hub() { widgets | awk -F'\t' '$1 ~ /^settings_hub_/ && $2 == "Button" {f=1} END {exit !f}'; }
    if on_hub; then
      for hub in $(widgets | awk -F'\t' '$1 ~ /^settings_hub_/ && $2 == "Button" {print $1}'); do
        for _ in 1 2 3; do on_hub && break; clk header_back; sleep 1; done
        # Back can be hidden on a panel; the Settings rail item (plain or
        # selected "_on") opens the hub again.
        on_hub || { clk "${id%_on}_on" || clk "${id%_on}"; sleep 1.2; }
        clk "$hub" || continue; sleep 1.5
        echo "## $label / ${hub#settings_hub_}"; dump
      done
    fi
  done <<<"$rails"
} | tee "$OUT/inventory.txt"

allow="$ROOT/apps/$APP/cert/ui-text-allowlist.txt"
if [[ -f "$allow" ]]; then
  new=$(grep -v '^## ' "$OUT/inventory.txt" | grep . | while IFS=$'\t' read -r id _ text; do
    grep -qxF "$id" <(grep -v '^#' "$allow") || printf '%s\t%s\n' "$id" "$text"
  done)
  if [[ -n "$new" ]]; then
    echo; echo "NEW long texts (not in ${allow#$ROOT/}) — make explanations ℹ tips, or allow data lines:" >&2
    printf '%s\n' "$new" >&2
    exit 1
  fi
fi
echo; echo "inventory: $OUT/inventory.txt"
