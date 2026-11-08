#!/usr/bin/env bash
# Run kirano.shop as a native desktop window (fastest local preview; also the
# offline presenter). SITE_PREVIEW=390x844:2 opens a phone-sized window on
# chapter 2; SITE_LINKS=local sends cross-site links to the local servers.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
exec cargo run -p kirano-site "$@"
