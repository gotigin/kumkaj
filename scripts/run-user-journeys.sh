#!/usr/bin/env bash
# Comprehensive books-level user journeys for Khaynjo + Moklanjo.
# Never reaches the OS printer/opener (fake lp/lpr/xdg-open + Capture sink).
#
# Usage (workspace root):
#   ./scripts/run-user-journeys.sh
#   ./scripts/run-user-journeys.sh khaynjo
#   ./scripts/run-user-journeys.sh moklanjo
# Extra libtest filters after -- :
#   ./scripts/run-user-journeys.sh -- journey_cod_dispatch
#   ./scripts/run-user-journeys.sh khaynjo -- journey_dine_in
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

APP=all
FILTERS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    khaynjo | moklanjo | kirano | tabiyat | all) APP=$1; shift ;;
    --) shift; FILTERS+=("$@"); break ;;
    *) FILTERS+=("$1"); shift ;;
  esac
done

FAKE="$(mktemp -d)"
LOG="$FAKE/calls.log"
: >"$LOG"
cleanup() { rm -rf "$FAKE"; }
trap cleanup EXIT

for tool in lp lpr xdg-open; do
  printf '#!/bin/sh\necho "%s $*" >> "%s"\n' "$tool" "$LOG" >"$FAKE/$tool"
  chmod +x "$FAKE/$tool"
done

export KUMKAJ_PRINT_SINK=capture
export PATH="$FAKE:$PATH"

run_pkg() {
  local pkg="$1"
  echo "══ user journeys: $pkg ══"
  if [[ ${#FILTERS[@]} -gt 0 ]]; then
    cargo test -p "$pkg" --test user_journeys -- --nocapture "${FILTERS[@]}"
  else
    cargo test -p "$pkg" --test user_journeys -- --nocapture
  fi
}

case "$APP" in
  khaynjo) run_pkg kumkaj-khaynjo ;;
  moklanjo) run_pkg kumkaj-moklanjo ;;
  kirano) run_pkg kumkaj-kirano ;;
  tabiyat) run_pkg kumkaj-tabiyat ;;
  all)
    run_pkg kumkaj-khaynjo
    run_pkg kumkaj-moklanjo
    run_pkg kumkaj-kirano
    run_pkg kumkaj-tabiyat
    ;;
  *)
    echo "usage: $0 [khaynjo|moklanjo|kirano|tabiyat|all] [-- test_filter…]" >&2
    exit 2
    ;;
esac

if [[ -s "$LOG" ]]; then
  echo "FAIL: journeys handed jobs to the OS printer/opener:" >&2
  cat "$LOG" >&2
  exit 1
fi
echo "user journeys clean: no lp / lpr / xdg-open"
