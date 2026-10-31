#!/usr/bin/env bash
# Flag any self.sync_chrome/set_status/go call made while an app_state()
# guard (`st`) is still in scope. sync_chrome & friends lock app_state()
# themselves; std Mutex is non-reentrant, so calling them under a live
# guard deadlocks the UI thread (a Khaynjo order-type chip froze this way, 2026-10-22; Moklanjo shares the shell pattern).
# Heuristic: brace-depth tracking over rustfmt-formatted UI code — every
# file the shell lives in (main.rs, app/, widgets/), not main.rs alone:
# since the shell was split into src/app/ a main.rs-only scan saw nothing.
set -euo pipefail
cd "$(dirname "$0")/.."
awk '
function count(ch, s,   n,i,c) { n=0; for(i=1;i<=length(s);i++){ c=substr(s,i,1); if(c==ch) n++ } return n }
BEGIN { flags=0 }
FNR == 1 { depth=0; guard=0; guarddepth=0; guardline=0 }
{
  line=$0
  opens=count("{", line); closes=count("}", line)
  if (guard && line ~ /drop\(st\)/) guard=0
  if (guard && line ~ /self\.(sync_chrome|set_status|go)\(cx/ && depth >= guarddepth) {
    printf "FLAG %s:%d (guard from %d): %s\n", FILENAME, FNR, guardline, line; flags++
  }
  if (line ~ /let (mut )?st = app_state\(\)\.lock\(\)/) {
    guard=1; guardline=FNR; guarddepth=depth + opens - closes
  }
  depth += opens - closes
  if (guard && depth < guarddepth) guard=0
}
END { if (flags > 0) exit 1 }
' src/main.rs src/app/*.rs src/widgets/*.rs
echo "lock-guard scan clean"
