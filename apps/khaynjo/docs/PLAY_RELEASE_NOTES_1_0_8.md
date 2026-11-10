# Play release notes — Khaynjo 1.0.8 (`version_code` 6)

**Bundle:** `dist/khaynjo-android/khaynjo.aab`  
**Also upload:** `native-debug-symbols.zip`, `mapping.txt` (same folder)

Follows production **`5 (1.0.7)`**. Stability and setup polish; no change to how books are stored.

## en-US (Release notes — paste this; ≤500 chars)

```text
• Device setup: choosing Client no longer freezes the screen; Cancel works while the till is opening.
• Add dishes: swipe sideways through menu pages; category chips follow the page you’re on.
• Keyboard: less jump when typing names, search, and amounts.
• Startup: quicker first screen after open on Android.
```

## hi-IN (optional — paste if you localize “What’s new”)

```text
• डिवाइस सेटअप: Client चुनने पर स्क्रीन अटकती नहीं; खुलते समय Cancel काम करता है।
• Add dishes: मेनू पेज स्वाइप; कैटेगरी टैब उसी पेज के साथ।
• कीबोर्ड: नाम/सर्च में कम झटका।
• ऐप खुलने पर पहली स्क्रीन जल्दी।
```

## Longer notes (internal / email — not for Play char limit)

- **DeviceSetup → Client:** role open runs off the UI thread; Cancel interrupts slow SQLite work; finishing Client mode no longer blocks the UI for a full shop reload before Pair.
- **Menu picker:** horizontal pager synced with category chips and search (single-page search unchanged).
- **Keyboard:** Android IME / soft-keyboard lift reconciled without fighting layout on every frame.
- **Cold start:** defer heavy store open to the first painted frame; splash background colour instead of a long black screen (where the build includes `MakepadLaunchBg`).

## Pre-upload gate

```bash
./scripts/product-cert.sh --tier=store khaynjo
```

See [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md).
