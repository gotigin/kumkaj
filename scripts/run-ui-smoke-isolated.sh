#!/usr/bin/env bash
# Isolated UI smoke: private Xvfb display, never the user's desktop (:0 / Wayland).
#
# Seeds realistic books, launches the native Makepad binary on DISPLAY=:$N,
# drives an in-app smoke script (peers / full desk day / soak), takes a PNG
# per step, then stops. Your session on :0 / wayland-0 is left alone.
#
# Usage (workspace root):
#   ./scripts/run-ui-smoke-isolated.sh                 # both apps, full script
#   ./scripts/run-ui-smoke-isolated.sh khaynjo
#   ./scripts/run-ui-smoke-isolated.sh moklanjo
#   SMOKE_SCRIPT=peers ./scripts/run-ui-smoke-isolated.sh khaynjo
#   SMOKE_SCRIPT=soak KUMKAJ_SMOKE_SOAK_SECS=3600 ./scripts/run-ui-smoke-isolated.sh khaynjo
#
# Env:
#   SMOKE_SCRIPT      peers | full (default) | soak | off (legacy timed shots only)
#   SMOKE_DISPLAY     X display number (default: first free from 99)
#   SMOKE_OUT         output dir (default: /tmp/kumkaj-ui-smoke-<pid>)
#   SMOKE_STEP_MS     pause between in-app steps (default 900)
#   SMOKE_XVFB_SIZE   Xvfb geometry WxHxD (default 1280x900x24)
#   KUMKAJ_SMOKE_SOAK_SECS  soak duration when SMOKE_SCRIPT=soak (default 1800)
#   SKIP_SEED=1       reuse KUMKAJ_DATA_DIR without reseeding
#   KUMKAJ_DATA_DIR   override data dir
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

APP="${1:-all}"
case "$APP" in
  khaynjo | moklanjo | kirano | tabiyat | all) ;;
  *)
    echo "usage: $0 [khaynjo|moklanjo|kirano|tabiyat|all]" >&2
    exit 2
    ;;
esac

SMOKE_SCRIPT="${SMOKE_SCRIPT:-full}"
case "$SMOKE_SCRIPT" in
  peers | full | soak | off) ;;
  *)
    echo "SMOKE_SCRIPT must be peers|full|soak|off" >&2
    exit 2
    ;;
esac

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "need '$1' on PATH (pacman -S xorg-server-xvfb imagemagick)" >&2
    exit 1
  }
}
need Xvfb
HAVE_IMPORT=0
command -v import >/dev/null 2>&1 && HAVE_IMPORT=1
if [[ "$HAVE_IMPORT" -eq 0 ]]; then
  need xwd
  need convert
fi

if [[ "${DISPLAY:-}" == ":0" || "${DISPLAY:-}" == ":0.0" ]]; then
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
OUT="${SMOKE_OUT:-/tmp/kumkaj-ui-smoke-$$}"
mkdir -p "$OUT"
echo "smoke out: $OUT  display: $DISP  script: $SMOKE_SCRIPT"

XVFB_PID=""
APP_PID=""
WATCH_PID=""
cleanup() {
  if [[ -n "${WATCH_PID}" ]] && kill -0 "$WATCH_PID" 2>/dev/null; then
    kill "$WATCH_PID" 2>/dev/null || true
    wait "$WATCH_PID" 2>/dev/null || true
  fi
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

XVFB_SIZE="${SMOKE_XVFB_SIZE:-1280x900x24}"
echo "starting Xvfb $DISP ($XVFB_SIZE) …"
Xvfb "$DISP" -screen 0 "$XVFB_SIZE" -nolisten tcp >/dev/null 2>&1 &
XVFB_PID=$!
for _ in $(seq 1 50); do
  [[ -S "/tmp/.X11-unix/X${DISP_NUM}" ]] && break
  sleep 0.1
done
if [[ ! -S "/tmp/.X11-unix/X${DISP_NUM}" ]]; then
  echo "Xvfb failed to create $DISP" >&2
  exit 1
fi

take_shot() {
  local path="$1"
  if [[ "$HAVE_IMPORT" -eq 1 ]]; then
    DISPLAY="$DISP" import -window root "$path" 2>/dev/null || true
  else
    DISPLAY="$DISP" xwd -root -silent | convert xwd:- "$path" 2>/dev/null || true
  fi
}

# Poll need_shot files written by the in-app script; save PNGs; ack by unlink.
watch_shots() {
  local shot_dir="$1"
  mkdir -p "$shot_dir"
  while true; do
    if [[ -f "$shot_dir/need_shot" ]]; then
      local name
      name="$(tr -cd 'A-Za-z0-9._-' <"$shot_dir/need_shot" | head -c 120)"
      [[ -z "$name" ]] && name="step"
      local path="$shot_dir/${name}.png"
      # Brief settle so Makepad finishes paint.
      sleep 0.25
      take_shot "$path"
      if [[ -s "$path" ]]; then
        echo "  shot $path ($(wc -c <"$path") bytes)"
      else
        echo "  shot FAILED $path" >&2
      fi
      rm -f "$shot_dir/need_shot"
    fi
    # Done markers from the app.
    if [[ -f "$shot_dir/PASS" || -f "$shot_dir/FAIL" || -f "$shot_dir/RESULT.txt" ]]; then
      # One last chance for a trailing need_shot.
      sleep 0.4
      if [[ -f "$shot_dir/need_shot" ]]; then
        continue
      fi
      break
    fi
    sleep 0.15
  done
}

smoke_one() {
  local kind="$1"
  local crate bin label
  if [[ "$kind" == "khaynjo" ]]; then
    crate=kumkaj-khaynjo
    bin=kumkaj-khaynjo
    label=Khaynjo
  elif [[ "$kind" == "kirano" ]]; then
    crate=kumkaj-kirano
    bin=kumkaj-kirano
    label=Kirano
  elif [[ "$kind" == "tabiyat" ]]; then
    crate=kumkaj-tabiyat
    bin=kumkaj-tabiyat
    label=Tabiyat
  else
    crate=kumkaj-moklanjo
    bin=kumkaj-moklanjo
    label=Moklanjo
  fi

  local data="${KUMKAJ_DATA_DIR:-$OUT/data-$kind}"
  local shot_dir="$OUT/$kind"
  mkdir -p "$data" "$shot_dir"
  echo "── $label smoke  data=$data  shots=$shot_dir  script=$SMOKE_SCRIPT"

  if [[ "${SKIP_SEED:-0}" != "1" ]]; then
    echo "  seeding visual books…"
    SEED_DATA_DIR="$data" KUMKAJ_PRINT_SINK=off \
      cargo test -p "$crate" --test visual_seed -- --ignored --exact seed_visual --nocapture
  fi

  echo "  building $bin…"
  cargo build -p "$crate" --bin "$bin" -q

  local exe="$ROOT/target/debug/$bin"
  if [[ ! -x "$exe" ]]; then
    exe="$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
      | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' \
      | head -1)/debug/$bin"
  fi
  if [[ ! -x "$exe" ]]; then
    echo "binary not found for $bin" >&2
    return 1
  fi

  local log="$OUT/${kind}.log"
  local script_env=()
  if [[ "$SMOKE_SCRIPT" != "off" ]]; then
    script_env=(
      KUMKAJ_SMOKE_SCRIPT="$SMOKE_SCRIPT"
      KUMKAJ_SMOKE_OUT="$shot_dir"
      KUMKAJ_SMOKE_STEP_MS="${SMOKE_STEP_MS:-900}"
      KUMKAJ_SMOKE_QUIT=1
    )
    if [[ "$SMOKE_SCRIPT" == "soak" ]]; then
      script_env+=(KUMKAJ_SMOKE_SOAK_SECS="${KUMKAJ_SMOKE_SOAK_SECS:-1800}")
    fi
  fi

  echo "  launching on $DISP…"
  env -u WAYLAND_DISPLAY \
    DISPLAY="$DISP" \
    KUMKAJ_DATA_DIR="$data" \
    KUMKAJ_PRINT_SINK=off \
    KUMKAJ_SMOKE_AUTO_SERVER=1 \
    KUMKAJ_SMOKE_AUTO_UNLOCK=1234 \
    "${script_env[@]}" \
    RUST_BACKTRACE=1 \
    "$exe" >"$log" 2>&1 &
  APP_PID=$!

  # Wait until the process is alive briefly.
  sleep 1
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "  FAIL: $label exited early — see $log" >&2
    tail -n 100 "$log" >&2 || true
    APP_PID=""
    return 1
  fi

  if [[ "$SMOKE_SCRIPT" != "off" ]]; then
    watch_shots "$shot_dir" &
    WATCH_PID=$!
    # Wait for the app to finish (it exits itself) or a long ceiling.
    local ceiling=3600
    if [[ "$SMOKE_SCRIPT" == "soak" ]]; then
      ceiling=$(( ${KUMKAJ_SMOKE_SOAK_SECS:-1800} + 600 ))
    elif [[ "$SMOKE_SCRIPT" == "peers" ]]; then
      ceiling=600
    else
      ceiling=1200
    fi
    local waited=0
    while kill -0 "$APP_PID" 2>/dev/null; do
      sleep 1
      waited=$((waited + 1))
      if [[ "$waited" -ge "$ceiling" ]]; then
        echo "  FAIL: $label smoke exceeded ${ceiling}s" >&2
        kill "$APP_PID" 2>/dev/null || true
        break
      fi
    done
    wait "$APP_PID" 2>/dev/null || true
    local app_rc=$?
    APP_PID=""
    # Let watcher drain RESULT.
    sleep 1
    if [[ -n "${WATCH_PID}" ]]; then
      kill "$WATCH_PID" 2>/dev/null || true
      wait "$WATCH_PID" 2>/dev/null || true
      WATCH_PID=""
    fi
    echo "  $label app exit=$app_rc"
    if [[ -f "$shot_dir/FAIL" ]]; then
      echo "  FAIL: in-app smoke reported failures:" >&2
      cat "$shot_dir/RESULT.txt" >&2 || true
      return 1
    fi
    if [[ ! -f "$shot_dir/PASS" && ! -f "$shot_dir/RESULT.txt" ]]; then
      echo "  FAIL: no RESULT from smoke — see $log" >&2
      tail -n 80 "$log" >&2 || true
      return 1
    fi
    if grep -E -q "panicked at|SIGSEGV" "$log" 2>/dev/null; then
      echo "  FAIL: panic/segfault in $log" >&2
      return 1
    fi
    # Makepad reports DSL mistakes (`fit: Stretch`, a property the widget
    # lacks) only at runtime as `[E]` lines, and the UI silently drops that
    # setting — Kirano/Tabiyat receipt slip, 2026-11-10. Errors only:
    # `[W]` stays informational (stale shader cache, etc.).
    if grep -E -q "^\[E\] " "$log" 2>/dev/null; then
      echo "  FAIL: runtime errors in $log:" >&2
      grep -E "^\[E\] " "$log" | sort -u | head -n 20 >&2
      return 1
    fi
    echo "  $label smoke PASS — $(ls "$shot_dir"/*.png 2>/dev/null | wc -l) shots"
    return 0
  fi

  # Legacy off mode: timed shots then kill.
  take_shot "$shot_dir/01-launch.png"
  sleep 8
  take_shot "$shot_dir/02-idle.png"
  if kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  APP_PID=""
  echo "  $label timed smoke done"
  return 0
}

RC=0
case "$APP" in
  khaynjo) smoke_one khaynjo || RC=1 ;;
  moklanjo) smoke_one moklanjo || RC=1 ;;
  kirano) smoke_one kirano || RC=1 ;;
  tabiyat) smoke_one tabiyat || RC=1 ;;
  all)
    smoke_one khaynjo || RC=1
    smoke_one moklanjo || RC=1
    smoke_one kirano || RC=1
    smoke_one tabiyat || RC=1
    ;;
esac

echo "══ UI smoke finished (exit $RC) — out $OUT ══"
find "$OUT" -name '*.png' | wc -l | xargs -I{} echo "PNG count: {}"
exit "$RC"
