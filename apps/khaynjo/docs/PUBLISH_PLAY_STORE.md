# Publish Khaynjo on Google Play

Application id: **`com.khaynjo`**  
Product version: Cargo `version` / `version_name` in [`../Cargo.toml`](../Cargo.toml) (same as Windows / AUR).  
Books data: app-scoped files; never tell shops to uninstall to “update” (that wipes books).  
Compat: [`DATA_COMPAT.md`](DATA_COMPAT.md). Release index: [`RELEASE_1_0_0.md`](RELEASE_1_0_0.md).

## Keystore defaults (Arch / Linux)

Same layout as Ghalbol (`$HOME/ghalbol-release.jks`). Packaging uses these **without env vars** when the files exist:

| Kind | Default path | Notes |
|------|----------------|-------|
| **Production / Play upload** | **`$HOME/khaynjo-release.jks`** | Override with `KUMKAJ_ANDROID_KEYSTORE` |
| **Password** | **`$HOME/khaynjo-release.jks.password`** | One line, mode `600`; override with `KUMKAJ_ANDROID_KEYSTORE_PASS` |
| **Alias sidecar** | **`$HOME/khaynjo-release.jks.makepad`** | Optional; else alias `upload` |
| **Debug** | **`$HOME/.android/debug.keystore`** | Android SDK default (`androiddebugkey` / `android`) |

Back up the `.jks` and password outside the repo. A lost key means no more Play updates.

Other Kumkaj apps: `$HOME/<app>-release.jks` + `.password` (e.g. `moklanjo-release.jks`).

## Prerequisites

- Android SDK (`ANDROID_HOME` / `ANDROID_SDK_ROOT`)
- Rust + Makepad Android toolchain (see root [`docs/DEV.md`](../../../docs/DEV.md) § Android)
- `$HOME/khaynjo-release.jks` + `$HOME/khaynjo-release.jks.password` present
- Play Console listing for `com.khaynjo` (title, screenshots, privacy policy URL, content rating, etc.)

## Build the Play bundle

From the workspace root (no keystore env needed if the defaults above exist):

```bash
./apps/khaynjo/scripts/package-android.sh
```

Outputs under `dist/khaynjo-android/`:

| File | Use |
|------|-----|
| `khaynjo.aab` | **Upload this** to Play Console |
| `kumkaj-khaynjo.apk` | Sideload / internal testing only |

If the `.jks` (or password file) is missing, the APK stays Makepad debug-signed — fine to try out, **not** for Play.

## Versioning on Play

| Field | Source | Rule |
|-------|--------|------|
| `version_name` | Cargo `package.metadata.makepad.android.version_name` | User-visible; match product version (e.g. `1.0.0`) |
| `version_code` | Release script passes `--version-code=auto` (YYYYMMDDHH) | Must **strictly increase** on every Play upload |

Bump Cargo `version` / `version_name` when the product release changes. Do not reuse a lower `version_code`.

## Upload

1. Play Console → app `com.khaynjo` → Production (or closed testing first)
2. Create release → upload `dist/khaynjo-android/khaynjo.aab`
3. Complete store listing, Data safety, and countries
4. Roll out

## After upload

- Install from Play on a clean device; create books; install the next build as an upgrade and confirm books remain
- Keep `$HOME/khaynjo-release.jks` and its password recoverable forever

## Related

- Package script: [`../scripts/package-android.sh`](../scripts/package-android.sh)
- Signing helpers: [`../../../scripts/android-env.sh`](../../../scripts/android-env.sh)
- Windows Store: [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md)
- AUR: [`PUBLISH_AUR.md`](PUBLISH_AUR.md)
