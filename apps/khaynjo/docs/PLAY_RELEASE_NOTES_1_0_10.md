# Play release notes — Khaynjo 1.0.10 (`version_code` 8)

**Bundle:** `dist/khaynjo-android/khaynjo.aab`  
**Also upload:** `native-debug-symbols.zip`, `mapping.txt` (same folder)  
**Windows Store:** `dist/Khaynjo-1.0.10.msix` · **Sideload:** `dist/Khaynjo-1.0.10.msi`

Follows **`7 (1.0.9)`**. Faster dish taps and category swipe, Bill & KOT sizes printed as set
with owner-sized item columns, bills for counter orders, Settings as one nav list, unlock back on your last screen, menu
templates from khaynjo.com, text selection.

## en-US (Release notes — paste this; ≤500 chars)

```text
• Faster dish taps; swipe sideways to change category.
• Bill & KOT: sizes print as set; size each item column.
• Counter, takeaway and delivery orders print a bill like tables.
• Settings is one list; put any section on the sidebar.
• Unlock opens the screen you last used.
• Ready menu templates; photos download when needed.
• Hold any text to select and copy, like a browser.
• New staff get their first PIN from the Owner in Team.
• Labels no longer clip on phones or in Gujarati.
```

## hi-IN (optional)

```text
• डिश तेज़ी से जुड़ती हैं; साइड स्वाइप से कैटेगरी बदलें।
• बिल और KOT: साइज़ वैसा ही छपता है; हर कॉलम की चौड़ाई तय करें।
• काउंटर, टेकअवे और डिलीवरी ऑर्डर का बिल टेबल की तरह छपता है।
• Settings अब एक सूची है; कोई भी सेक्शन साइडबार पर रखें।
• अनलॉक पर आपकी पिछली स्क्रीन खुलती है।
• तैयार मेनू टेम्पलेट; फोटो ज़रूरत पर डाउनलोड होते हैं।
• कोई भी टेक्स्ट दबाकर चुनें और कॉपी करें, ब्राउज़र जैसा।
• नए स्टाफ़ को पहला PIN Owner Team में देता है।
• फ़ोन और गुजराती में लेबल अब कटते नहीं।
```

## Pre-upload gate

```bash
./scripts/product-cert.sh --tier=store khaynjo
```

See [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md) and
[`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md).
