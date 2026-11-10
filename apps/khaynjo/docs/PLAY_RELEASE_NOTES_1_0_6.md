# Play release notes — Khaynjo 1.0.6 (`version_code` 4)

**Bundle:** `dist/khaynjo-android/khaynjo.aab`  
**Also upload:** `native-debug-symbols.zip`, `mapping.txt` (same folder)

Previous Console builds were removed; treat this as the **first public listing** if reviewers see no prior live release — notes below work for first publish and for anyone who still had an older sideload/test build.

## en-US (Release notes)

```text
Khaynjo for restaurants — tables, kitchen tickets (KOT), billing, and day books on your phone or tablet. Works offline; your shop data stays on the device.

This release:
• Stable open and sign-in (fixes crashes right after launch)
• Layout that fits Android system bars so buttons stay tappable
• Reliable side menu / navigation on phones and tablets
• Hardened till so a background error is less likely to freeze or close the app

Recommended for every shop installing or updating Khaynjo.
```

## Short “What’s new” (≤500 chars)

```text
Khaynjo: local restaurant till — tables, KOT, billing, and books. This build opens reliably, fits system bars, and keeps the side menu working. Recommended install for all shops.
```

## First-publish listing blurb (optional, longer store description if empty)

```text
Khaynjo is a local-first restaurant operating system for the floor and the counter: open tables, send kitchen tickets, take payment, and keep day books — without needing the cloud for everyday service. Pair phones and tablets on your shop network when you want. Your books stay on the device.
```

## Pre-upload gate

```bash
./scripts/product-cert.sh --tier=store khaynjo
```

See [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md).
