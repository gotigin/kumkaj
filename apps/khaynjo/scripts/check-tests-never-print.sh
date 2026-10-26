#!/usr/bin/env bash
# Run Khaynjo's tests with fake `lp` / `lpr` / `xdg-open` first on PATH and fail
# if any test reached the OS printer or opener. Tests capture print jobs
# (print_share::PrintSink::Capture) and assert on their text instead — paper
# is never the test oracle (2026-10-23: a LAN test printed on every run).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
FAKE="$(mktemp -d)"
LOG="$FAKE/calls.log"
: > "$LOG"
for tool in lp lpr xdg-open; do
  printf '#!/bin/sh\necho "%s $*" >> "%s"\n' "$tool" "$LOG" > "$FAKE/$tool"
  chmod +x "$FAKE/$tool"
done
cd "$ROOT"
PATH="$FAKE:$PATH" cargo test -p kumkaj-khaynjo --quiet "$@"
if [ -s "$LOG" ]; then
  echo "FAIL: tests handed jobs to the OS printer/opener:" >&2
  cat "$LOG" >&2
  rm -rf "$FAKE"
  exit 1
fi
rm -rf "$FAKE"
echo "no-print check clean: tests never reached lp / lpr / xdg-open"
