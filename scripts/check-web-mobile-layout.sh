#!/usr/bin/env bash
# Regression gate: Android Chrome Desktop-site / pinch-zoom must not lay out
# Makepad sites at a fake ~980px innerWidth. See docs/WEBSITES.md and
# scripts/web/mobile_layout_size.js.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
fail=0

echo "== unit: mobile_layout_size.js =="
node "$ROOT/scripts/web/mobile_layout_size.js"

echo "== vendor web.js has css_layout_size_for_canvas =="
WEBJS="$ROOT/vendor/makepad/platform/src/os/web/web.js"
if [[ ! -f "$WEBJS" ]]; then
  echo "error: missing $WEBJS — run ./scripts/setup-makepad.sh" >&2
  exit 1
fi
for needle in css_layout_size_for_canvas "visualViewport.addEventListener" "Desktop site" \
  "fire_resize_debounced" "canvas.width !== sw"; do
  if ! grep -q "$needle" "$WEBJS"; then
    echo "error: $WEBJS missing '$needle' (web_mobile_layout_width patch)" >&2
    fail=1
  fi
done
# Must NOT resize the WebGL canvas on every visualViewport scroll (Android
# Chrome URL-bar / pan) — that froze scroll and smeared text.
if grep -q "visualViewport.addEventListener('scroll'" "$WEBJS"; then
  echo "error: $WEBJS must not listen to visualViewport scroll for resize" >&2
  fail=1
fi

echo "== patch listed in setup-makepad.sh =="
if ! grep -q 'web_mobile_layout_width' "$ROOT/scripts/setup-makepad.sh"; then
  echo "error: scripts/setup-makepad.sh does not apply web_mobile_layout_width" >&2
  fail=1
fi
if [[ ! -f "$ROOT/patches/makepad/web_mobile_layout_width.rs.patch" ]]; then
  echo "error: missing patches/makepad/web_mobile_layout_width.rs.patch" >&2
  fail=1
fi

echo "== algorithm in patch matches scripts/web/mobile_layout_size.js =="
# Same clamp: Math.min of positive inner/outer/screen on touch.
if ! grep -q 'Math.min.apply(null, ws)' "$ROOT/patches/makepad/web_mobile_layout_width.rs.patch" \
  && ! grep -q 'Math.min.apply(null, ws)' "$WEBJS"; then
  echo "error: clamp Math.min over width candidates missing" >&2
  fail=1
fi

SITES=(khaynjo moklanjo tabiyat kirano kumkaj)
echo "== each Makepad site injects mobile-viewport-guard + hardened viewport =="
for site in "${SITES[@]}"; do
  guard="$ROOT/sites/$site/static/js/mobile-viewport-guard.js"
  css="$ROOT/sites/$site/static/css/site-shell.css"
  patch="$ROOT/sites/$site/scripts/patch-web-shell.sh"
  if [[ ! -f "$guard" ]]; then
    echo "error: missing $guard" >&2
    fail=1
  else
    grep -q 'maximum-scale=1' "$guard" || { echo "error: $guard must set maximum-scale=1" >&2; fail=1; }
    grep -q 'Math.min' "$guard" || { echo "error: $guard must clamp width" >&2; fail=1; }
  fi
  if [[ ! -f "$css" ]]; then
    echo "error: missing $css" >&2
    fail=1
  else
    grep -q 'touch-action: manipulation' "$css" || {
      echo "error: $css must set touch-action: manipulation (reduce double-tap zoom)" >&2
      fail=1
    }
    grep -q 'touch-action: none' "$css" || {
      echo "error: $css must set canvas touch-action: none (Makepad owns gestures)" >&2
      fail=1
    }
    grep -q 'overscroll-behavior: none' "$css" || {
      echo "error: $css must set overscroll-behavior: none" >&2
      fail=1
    }
  fi
  if [[ ! -f "$patch" ]]; then
    echo "error: missing $patch" >&2
    fail=1
  else
    grep -q 'mobile-viewport-guard' "$patch" || {
      echo "error: $patch must inject mobile-viewport-guard.js" >&2
      fail=1
    }
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo "check-web-mobile-layout: FAILED" >&2
  exit 1
fi
echo "check-web-mobile-layout: OK"
