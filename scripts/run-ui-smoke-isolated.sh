#!/usr/bin/env bash
# Isolated UI smoke: private Xvfb display, never the user's desktop (:0 / Wayland).
#
# Seeds realistic books, launches the native Makepad binary on DISPLAY=:$N,
# waits for a window, takes root screenshots, then stops. Your session on
# :0 / wayland-0 is left alone.
#
# Usage (workspace root):
#   ./scripts/run-ui-smoke-isolated.sh              # both apps
#   ./scripts/run-ui-smoke-isolated.sh khaynjo
#   ./scripts/run-ui-smoke-isolated.sh moklanjo
#
# Env:
#   SMOKE_DISPLAY   X display number (default: first free from 99)
#   SMOKE_SECONDS   how long to keep the app up (default: 12)
#   SMOKE_OUT       screenshot dir (default: /tmp/kumkaj-ui-smoke-<pid>)
#   SKIP_SEED=1     reuse an existing KUMKAJ_DATA_DIR without reseeding
#   KUMKAJ_DATA_DIR override data dir (default: fresh under SMOKE_OUT)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

APP="${1:-all}"
case "$APP" in
  khaynjo | moklanjo | all) ;;
  *)
    echo "usage: $0 [khaynjo|moklanjo|all]" >&2
    exit 2
    ;;
esac

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "need '$1' on PATH (pacman -S xorg-server-xvfb xorg-xwd imagemagick xdotool)" >&2
    exit 1
  }
}
need Xvfb
need timeout
# import (ImageMagick) preferred; fall back to xwd|convert
HAVE_IMPORT=0
HAVE_XDOTOOL=0
command -v import >/dev/null 2>&1 && HAVE_IMPORT=1
command -v xdotool >/dev/null 2>&1 && HAVE_XDOTOOL=1
if [[ "$HAVE_IMPORT" -eq 0 ]]; then
  need xwd
  need convert
fi

# Hard refuse driving the real desktop.
if [[ "${DISPLAY:-}" == ":0" || "${DISPLAY:-}" == ":0.0" ]]; then
  # Only a warning — we will override DISPLAY below. Never inherit :0 for the app.
  echo "note: your shell DISPLAY is :0; smoke will use a private Xvfb instead"
fi
if [[ "${SMOKE_DISPLAY:-}" == "0" || "${SMOKE_DISPLAY:-}" == ":0" ]]; then
  echo "refusing SMOKE_DISPLAY=:0 — that is the user desktop" >&2
  exit 1
fi

pick_display() {
  if [[ -n "${SMOKE_DISPLAY:-}" ]]; then
    echo "${SMOKE_DISPLAY#:}"
    return
  fi
  local n
  for n in 99 98 97 96 95; do
    if [[ ! -e "/tmp/.X${n}-lock" && ! -S "/tmp/.X11-unix/X${n}" ]]; then
      echo "$n"
      return
    fi
  done
  echo "no free X display in 99..95 — set SMOKE_DISPLAY" >&2
  exit 1
}

DISP_NUM="$(pick_display)"
DISP=":${DISP_NUM}"
SMOKE_SECONDS="${SMOKE_SECONDS:-24}"
OUT="${SMOKE_OUT:-/tmp/kumkaj-ui-smoke-$$}"
mkdir -p "$OUT"
echo "smoke out: $OUT  display: $DISP  seconds: $SMOKE_SECONDS"

XVFB_PID=""
APP_PID=""
cleanup() {
  if [[ -n "${APP_PID}" ]] && kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  if [[ -n "${XVFB_PID}" ]] && kill -0 "$XVFB_PID" 2>/dev/null; then
    kill "$XVFB_PID" 2>/dev/null || true
    wait "$XVFB_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

echo "starting Xvfb $DISP …"
Xvfb "$DISP" -screen 0 1280x900x24 -nolisten tcp >/dev/null 2>&1 &
XVFB_PID=$!
# Wait until the socket exists.
for _ in $(seq 1 50); do
  if [[ -S "/tmp/.X11-unix/X${DISP_NUM}" ]]; then
    break
  fi
  sleep 0.1
done
if [[ ! -S "/tmp/.X11-unix/X${DISP_NUM}" ]]; then
  echo "Xvfb failed to create $DISP" >&2
  exit 1
fi

shot() {
  local name="$1"
  local path="$OUT/${name}.png"
  if [[ "$HAVE_IMPORT" -eq 1 ]]; then
    DISPLAY="$DISP" import -window root "$path" 2>/dev/null || true
  else
    DISPLAY="$DISP" xwd -root -silent | convert xwd:- "$path" 2>/dev/null || true
  fi
  if [[ -s "$path" ]]; then
    echo "  shot $path ($(wc -c <"$path") bytes)"
  else
    echo "  shot FAILED $path" >&2
  fi
}

# Click inside the app window at fractional (fx, fy) of its geometry.
click_frac() {
  local wid="$1" fx="$2" fy="$3"
  local geom x y w h cx cy
  geom="$(DISPLAY="$DISP" xdotool getwindowgeometry --shell "$wid" 2>/dev/null)" || return 1
  eval "$geom"
  x=${X:-0}; y=${Y:-0}; w=${WIDTH:-0}; h=${HEIGHT:-0}
  if [[ "$w" -lt 100 || "$h" -lt 100 ]]; then
    return 1
  fi
  cx=$((x + (w * fx) / 100))
  cy=$((y + (h * fy) / 100))
  DISPLAY="$DISP" xdotool mousemove --sync "$cx" "$cy" click 1 2>/dev/null || true
  echo "  click ${cx},${cy} (window ${w}x${h} @ ${x},${y}; ${fx}% ${fy}%)"
}

# After smoke auto-unlock: click a side-rail peer (Floor / Ops) for a second view.
drive_after_unlock() {
  local kind="$1"
  local ids best="" best_area=0
  ids="$(DISPLAY="$DISP" xdotool search --onlyvisible --name . 2>/dev/null || true)"
  for id in $ids; do
    local geom w h area
    geom="$(DISPLAY="$DISP" xdotool getwindowgeometry --shell "$id" 2>/dev/null)" || continue
    eval "$geom"
    w=${WIDTH:-0}; h=${HEIGHT:-0}
    area=$((w * h))
    if [[ "$area" -gt "$best_area" ]]; then
      best_area=$area
      best=$id
    fi
  done
  if [[ -z "$best" ]]; then
    echo "  drive: no window yet"
    return 1
  fi
  DISPLAY="$DISP" xdotool windowactivate --sync "$best" 2>/dev/null || true
  sleep 0.4
  shot "${kind}-03-main"
  # Side rail is a narrow left column — Floor sits mid-rail on Khaynjo;
  # Ops / Book mid-rail on Moklanjo. Probe a few left-edge Y positions.
  echo "  drive: tap side rail peer…"
  for fy in 42 48 36 54 30; do
    click_frac "$best" 6 "$fy" || true
    sleep 0.4
  done
  sleep 0.8
  shot "${kind}-04-peer"
}

smoke_one() {
  local kind="$1" # khaynjo | moklanjo
  local crate bin label
  if [[ "$kind" == "khaynjo" ]]; then
    crate=kumkaj-khaynjo
    bin=kumkaj-khaynjo
    label=Khaynjo
  else
    crate=kumkaj-moklanjo
    bin=kumkaj-moklanjo
    label=Moklanjo
  fi

  local data="${KUMKAJ_DATA_DIR:-$OUT/data-$kind}"
  mkdir -p "$data"
  echo "── $label smoke  data=$data"

  if [[ "${SKIP_SEED:-0}" != "1" ]]; then
    echo "  seeding visual books…"
    if [[ "$kind" == "khaynjo" ]]; then
      # Prefer SEED_DATA_DIR (creates local server + PIN 1234).
      SEED_DATA_DIR="$data" KUMKAJ_PRINT_SINK=off \
        cargo test -p "$crate" --test visual_seed -- --ignored --exact seed_visual --nocapture
    else
      SEED_DATA_DIR="$data" KUMKAJ_PRINT_SINK=off \
        cargo test -p "$crate" --test visual_seed -- --ignored --exact seed_visual --nocapture
    fi
  fi

  echo "  building $bin (if needed)…"
  cargo build -p "$crate" --bin "$bin" -q

  local exe="$ROOT/target/debug/$bin"
  if [[ ! -x "$exe" ]]; then
    # Some workspaces use CARGO_TARGET_DIR.
    exe="$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
      | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' \
      | head -1)/debug/$bin"
  fi
  if [[ ! -x "$exe" ]]; then
    echo "binary not found for $bin" >&2
    return 1
  fi

  local log="$OUT/${kind}.log"
  echo "  launching on $DISP (print sink off, no Wayland, smoke auto Server+PIN)…"
  # Critical: unset WAYLAND_DISPLAY so Makepad uses X11 on our Xvfb, not the
  # user's compositor. Never pass DISPLAY=:0.
  # KUMKAJ_SMOKE_AUTO_* are opt-in harness hooks (ignored when KUMKAJ_ENV=production).
  env -u WAYLAND_DISPLAY \
    DISPLAY="$DISP" \
    KUMKAJ_DATA_DIR="$data" \
    KUMKAJ_PRINT_SINK=off \
    KUMKAJ_SMOKE_AUTO_SERVER=1 \
    KUMKAJ_SMOKE_AUTO_UNLOCK=1234 \
    RUST_BACKTRACE=1 \
    "$exe" >"$log" 2>&1 &
  APP_PID=$!

  # Wait for process to stay up and optionally a window.
  local up=0
  for i in $(seq 1 40); do
    if ! kill -0 "$APP_PID" 2>/dev/null; then
      echo "  FAIL: $label exited early — see $log" >&2
      tail -n 80 "$log" >&2 || true
      APP_PID=""
      return 1
    fi
    if [[ "$HAVE_XDOTOOL" -eq 1 ]]; then
      if DISPLAY="$DISP" xdotool search --name "$label" >/dev/null 2>&1 \
        || DISPLAY="$DISP" xdotool search --class "$bin" >/dev/null 2>&1 \
        || DISPLAY="$DISP" xdotool search --onlyvisible --name . >/dev/null 2>&1; then
        up=1
        break
      fi
    elif [[ "$i" -ge 15 ]]; then
      up=1
      break
    fi
    sleep 0.25
  done

  shot "${kind}-01-launch"
  sleep 2
  shot "${kind}-02-idle"

  # KUMKAJ_SMOKE_AUTO_SERVER / AUTO_UNLOCK already open the till. Optional
  # xdotool: tap Floor (Khaynjo) or Ops (Moklanjo) for a second screen shot.
  if [[ "$HAVE_XDOTOOL" -eq 1 ]]; then
    drive_after_unlock "$kind" || true
  fi

  # Keep the window up for the remaining budget so a human (or agent) can
  # re-open shots; then stop cleanly.
  local left
  left=$((SMOKE_SECONDS - 16))
  if [[ "$left" -gt 0 ]]; then
    sleep "$left"
  fi
  shot "${kind}-99-before-quit"

  if kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  APP_PID=""

  if [[ "$up" -eq 0 ]]; then
    echo "  WARN: no window title match for $label (process stayed up; shots may still be useful)"
  fi
  echo "  $label smoke done — log $log"
  # Fail if the log screams about a panic before we killed it.
  if grep -E -q "panicked at|SIGSEGV|Wayland connection" "$log" 2>/dev/null; then
    echo "  FAIL: suspicious log lines in $log" >&2
    grep -E "panicked at|SIGSEGV|Wayland connection" "$log" >&2 || true
    return 1
  fi
  return 0
}

RC=0
case "$APP" in
  khaynjo) smoke_one khaynjo || RC=1 ;;
  moklanjo) smoke_one moklanjo || RC=1 ;;
  all)
    smoke_one khaynjo || RC=1
    smoke_one moklanjo || RC=1
    ;;
esac

echo "══ UI smoke finished (exit $RC) — shots in $OUT ══"
ls -la "$OUT"/*.png 2>/dev/null || true
exit "$RC"
