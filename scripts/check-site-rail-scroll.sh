#!/usr/bin/env bash
# Regression: chapter rail / scroll-spy math must stay in sync with the page.
# Pins journey_progress_from + active_chapter_from (Moklanjo) so the orange
# fill advances between beads and completes when scroll ends — even when the
# short last chapter never crosses the sticky spy mark.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== moklanjo-site rail / scroll unit tests =="
# Filter matches the three rail_fill_* regression tests (and only those).
cargo test -p moklanjo-site rail_fill_ -- --nocapture

echo "== typecheck chapter-nav sites =="
cargo check -p moklanjo-site -p kumkaj-site -p kirano-site -p tabiyat-site

echo "OK — rail/scroll regression checks passed."
