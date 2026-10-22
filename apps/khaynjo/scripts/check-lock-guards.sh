#!/usr/bin/env bash
# Flag any self.sync_chrome/set_status/go call made while an app_state()
# guard (`st`) is still in scope. sync_chrome & friends lock app_state()
# themselves; std Mutex is non-reentrant, so calling them under a live
# guard deadlocks the UI thread (order-type chip freeze, 2026-10-22).
# Heuristic: brace-depth tracking over rustfmt-formatted main.rs.
set -euo pipefail
cd "$(dirname "$0")/.."
awk '
function count(ch, s,   n,i,c) { n=0; for(i=1;i<=length(s);i++){ c=substr(s,i,1); if(c==ch) n++ } return n }
BEGIN { depth=0; guard=0; guarddepth=0; guardline=0; flags=0 }
{
  line=$0
  opens=count("{", line); closes=count("}", line)
  if (guard && line ~ /drop\(st\)/) guard=0
  if (guard && line ~ /self\.(sync_chrome|set_status|go)\(cx/ && depth >= guarddepth) {
    printf "FLAG line %d (guard from %d): %s\n", NR, guardline, line; flags++
  }
  if (line ~ /let (mut )?st = app_state\(\)\.lock\(\)/) {
    guard=1; guardline=NR; guarddepth=depth + opens - closes
  }
  depth += opens - closes
  if (guard && depth < guarddepth) guard=0
}
END { if (flags > 0) exit 1 }
' src/main.rs
echo "lock-guard scan clean"
