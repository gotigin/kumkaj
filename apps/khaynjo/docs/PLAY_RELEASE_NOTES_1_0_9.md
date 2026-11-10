# Play release notes — Khaynjo 1.0.9 (`version_code` 7)

**Bundle:** `dist/khaynjo-android/khaynjo.aab`  
**Also upload:** `native-debug-symbols.zip`, `mapping.txt` (same folder)  
**Windows:** `dist/Khaynjo-1.0.9.msi`

Follows production **`6 (1.0.8)`**. Send KOT responsiveness on the shop Server.

## en-US (Release notes — paste this; ≤500 chars)

```text
• Send KOT: kitchen slips queue faster on the counter — no long freeze after you tap Send.
• Menu: category swipe and chips stay in sync when you pick dishes.
• Device setup: smoother switch to Client mode with working Cancel.
```

## hi-IN (optional)

```text
• Send KOT: काउंटर पर टैप के बाद स्क्रीन कम अटकती है; किचन स्लिप जल्दी जाती है।
• मेनू: कैटेगरी स्वाइप और चिप्स साथ चलते हैं।
• डिवाइस सेटअप: Client मोड और Cancel बेहतर।
```

## Pre-upload gate

```bash
./scripts/product-cert.sh --tier=store khaynjo
```

See [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md).
