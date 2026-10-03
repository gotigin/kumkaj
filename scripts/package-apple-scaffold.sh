#!/usr/bin/env bash
# macOS / iOS scaffold — no real Mac required for these steps.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MAKEPAD_ROOT="${MAKEPAD_ROOT:-$ROOT/../makepad}"
OUT="$ROOT/dist/apple"
mkdir -p "$OUT"

cat > "$OUT/README.txt" <<'EOF'
Kumkaj Apple platform scaffold
===============================
Policy: do everything that does not require a physical Mac; skip only
real-Mac device verification and signed App Store submission.

Doable without a Mac
--------------------
- Keep Kumkaj as a single Makepad codebase (already portable).
- Document iOS toolchain install commands for when a Mac is available:
    cargo makepad apple ios install-toolchain
- Document macOS desktop run (on a Mac):
    cargo run -p kumkaj --release

Blocked until a Mac / device is available
-----------------------------------------
- Signed macOS .app / notarization
- Real iOS device smoke tests
- App Store / TestFlight upload

When a Mac appears, run the install-toolchain commands from a Makepad
`main` checkout, then build/run Kumkaj there. Do not claim release
validation for macOS/iOS until device evidence exists.
EOF

if [[ -d "$MAKEPAD_ROOT" ]]; then
  echo "Makepad checkout present — Apple toolchain can be installed on a Mac host."
else
  echo "Makepad checkout not found; scaffold README written anyway."
fi
echo "Apple scaffold: $OUT/README.txt"
