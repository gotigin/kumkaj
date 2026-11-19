# Publish Moklanjo on Google Play

Application id: **`com.moklanjo`**  
Product version: Cargo `version` / `version_name` in [`../Cargo.toml`](../Cargo.toml) (same as Windows / Linux).  
Books data: app-scoped files; never tell shops to uninstall to “update” (that wipes books).  
Release tracker: [`MOKLANJO_RELEASE.md`](MOKLANJO_RELEASE.md). Privacy: [`PRIVACY.md`](PRIVACY.md) · site `/privacy`.

## Keystore defaults (Arch / Linux)

Same layout as sibling apps (`$HOME/<app>-release.jks`). Packaging uses these **without env vars** when the files exist:

| Kind | Default path | Notes |
|------|----------------|-------|
| **Production / Play upload** | **`$HOME/moklanjo-release.jks`** | Override with `KUMKAJ_ANDROID_KEYSTORE` |
| **Password** | **`$HOME/moklanjo-release.jks.password`** | One line, mode `600`; override with `KUMKAJ_ANDROID_KEYSTORE_PASS` |
| **Alias sidecar** | **`$HOME/moklanjo-release.jks.makepad`** | Optional; else alias `upload` |
| **Debug** | **`$HOME/.android/debug.keystore`** | Android SDK default |

Back up the `.jks` and password outside the repo. A lost key means no more Play updates.

## Prerequisites

- Android SDK (`ANDROID_HOME` / `ANDROID_SDK_ROOT`)
- Rust + Makepad Android toolchain (see root [`docs/DEV.md`](../../../docs/DEV.md) § Android)
- `$HOME/moklanjo-release.jks` + `.password` present for a Play-signed AAB (created 2026-11-19; same layout as Khaynjo — **do not regenerate**)
- Play Console listing for `com.moklanjo` (title, screenshots, privacy policy URL → moklanjo.com/privacy, content rating)
- **Product cert green** before upload:

```bash
./scripts/product-cert.sh --tier=store moklanjo
# Needs adb + a device + dist/moklanjo-android/*.apk from package-android.sh
```

## Build the Play bundle

From the workspace root:

```bash
./apps/moklanjo/scripts/package-android.sh
```

Outputs under `dist/moklanjo-android/`:

| File | Use |
|------|-----|
| `moklanjo.aab` (or `*.aab`) | **Upload this** to Play Console |
| `native-debug-symbols.zip` | Upload with the AAB when present |
| `mapping.txt` | Upload with the AAB when present |
| `*.apk` | Sideload / internal testing only |

If the `.jks` is missing, the APK stays Makepad debug-signed — fine to try out, **not** for Play.

## Versioning on Play

| Field | Source | Rule |
|-------|--------|------|
| `version_name` | Cargo `package.metadata.makepad.android.version_name` | User-visible; match product version (e.g. `0.1.0`). |
| `version_code` | Cargo `package.metadata.makepad.android.version_code` | Opaque integer. **New id starts at `1`**; bump by `1` per upload. Never dates / `--version-code=auto`. |

First public upload for `com.moklanjo`: keep `version_code = 1` until a higher code is accepted.

## Upload

1. Play Console → Moklanjo → Production (or testing track)
2. Upload the AAB (+ symbols / mapping)
3. Complete listing, privacy URL, content rating
4. Submit — only after store-tier cert is green

## Related

- Scripts: [`../scripts/package-android.sh`](../scripts/package-android.sh)
- Windows Store: [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md)
- LAN pilot: [`LAN_PILOT.md`](LAN_PILOT.md)
- Root packaging notes: [`../../../docs/DEV.md`](../../../docs/DEV.md)
