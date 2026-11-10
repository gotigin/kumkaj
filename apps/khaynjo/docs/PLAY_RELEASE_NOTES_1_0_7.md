# Play release notes — Khaynjo 1.0.7 (`version_code` 5)

**Bundle:** `dist/khaynjo-android/khaynjo.aab`  
**Also upload:** `native-debug-symbols.zip`, `mapping.txt` (same folder)

Previous `4 (1.0.6)` production release was discarded after a Broken Functionality
reject (open → “keeps stopping” on API 26+ from an AAB resource-id mismatch).
This build is the first release shops should install: full product intro plus that
launch crash fixed.

## en-US (Release notes — paste this; ≤500 chars)

```text
First release: tables, KOT, billing, and day books offline on your devices. Server + extra tills on shop Wi‑Fi. Fixes the open-then-crash on Android 8+ from the previous build.
```

## First-publish listing blurb (if store description still empty)

Use the Full description in [`PLAY_STORE_LISTING.md`](PLAY_STORE_LISTING.md).

## Pre-upload gate

```bash
./scripts/product-cert.sh --tier=store khaynjo
```

See [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md).
