#!/usr/bin/env bash
# Run tabiyat.shop (Makepad fullpage) in the browser.
# Serves at http://127.0.0.1:8014 — open that URL after the build finishes.
# (kumkaj.com runs on :8010; served locally, each site links to the others'
# local ports instead of the public domains.)
#
# Uses cargo-makepad from vendor/makepad-full so packaging matches the
# vendored Makepad the apps build against. A newer global `cargo-makepad`
# may require a font-assets custom section this tree does not emit yet.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
exec cargo run -q --manifest-path vendor/makepad-full/tools/cargo_makepad/Cargo.toml -- \
  wasm --no-threads --port=8014 run -p tabiyat-site "$@"
