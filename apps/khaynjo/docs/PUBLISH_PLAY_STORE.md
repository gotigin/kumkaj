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
| `native-debug-symbols.zip` | **Upload with the AAB** — native crash / ANR symbolication |
| `mapping.txt` | **Upload with the AAB** — clears the “no deobfuscation file” warning |
| `kumkaj-khaynjo.apk` | Sideload / internal testing only |

If the `.jks` (or password file) is missing, the APK stays Makepad debug-signed — fine to try out, **not** for Play.

## Versioning on Play

| Field | Source | Rule |
|-------|--------|------|
| `version_name` | Cargo `package.metadata.makepad.android.version_name` | User-visible; match product version (e.g. `1.0.0`). Major/minor/patch changes live **here only**. |
| `version_code` | Cargo `package.metadata.makepad.android.version_code` | Opaque integer Play uses for upgrades. **Manual** small counter — see below. |

### `version_code` rules (all Kumkaj apps)

These apply to **every** app in this workspace (Moklanjo, Kirano, Tabiyat, future products), not only Khaynjo:

1. **New application id → start at `1`.** Never seed with a date, `YYYYMMDDHH`, unix time, or “roomy” thousands.
2. **Each Play upload → bump by `1`** (or a few if you skipped a build). Do not encode semver into the integer. `2.0.0` can be code `13` after `1.9.0` was `12`.
3. **Never `--version-code=auto`.** Timestamps jump into the billions, trip Console’s “significantly higher than previous” warning, and burn Play’s max **`2_100_000_000`**. Packaging must not pass `auto`; cargo-makepad reads Cargo only.
4. **Never decrease.** Once a code is accepted (or even rejected after upload) for that package id, the next must be **strictly greater**. You cannot reset to `1` for the same id.

Workspace note: [`docs/DEV.md`](../../../docs/DEV.md) § Android packaging. Khaynjo and siblings all start at `version_code = 1`.

**`targetSdkVersion`:** must be **≥ 36** (Play policy). Set in vendored cargo-makepad (`vendor/makepad-full/.../android/sdk.rs` `target_sdk_version`). `min_sdk_version` in Cargo stays lower (e.g. 26) for device reach.

**Each Play upload:**

1. Open [`../Cargo.toml`](../Cargo.toml) → `[package.metadata.makepad.android]`.
2. First upload: leave `version_code = 1`. Later uploads: set to **one greater than the highest code already on Play** (App bundle explorer / last release). Never invent date/timestamp schemes.
3. Bump `version` / `version_name` only when the product marketing version changes.
4. Run `./apps/khaynjo/scripts/package-android.sh` (uses the Cargo `version_code`; no auto override).

Do not reuse a lower `version_code` once a higher one is on Play for this package id.

## Upload

1. Play Console → app `com.khaynjo` → Production (or closed testing first)
2. Create release → upload `dist/khaynjo-android/khaynjo.aab`
3. **Crash files (do not skip — clears the two Play warnings):**
   - On the uploaded AAB row, open **⋮** → **Upload native debug symbols (.zip)** → choose `dist/khaynjo-android/native-debug-symbols.zip`
   - Same menu / App bundle explorer → **Downloads** → **Assets** → upload **Deobfuscation file** → `dist/khaynjo-android/mapping.txt`
4. Complete store listing, Data safety, and countries
5. Roll out

### Why both files

| Play warning | Cause for this stack | Our file |
|--------------|----------------------|----------|
| “no deobfuscation file” / R8/ProGuard | Makepad compiles the Java shell with **D8 only** — **no R8 minify/obfuscate** (class names stay `dev.makepad.android.*`). Play still nags. | `mapping.txt` — **identity** map of every dex class (`Foo -> Foo:`) so Console has a file and stack traces stay readable |
| “native code… upload debug symbols” | Almost all app logic is Rust in `libmakepad.so` | `native-debug-symbols.zip` — `arm64-v8a/libmakepad.so` (and other ABIs) with symbol table for Android vitals |

Rebuild always regenerates both next to the AAB (`android_write_play_crash_files` in `scripts/android-env.sh`). Upload the pair that matches **this** `version_code` — never reuse symbols from an older build.

### Useful in production **and** development

These artifacts are not “Play paperwork only.” Almost every crash or ANR we care about is in **Rust** (`libmakepad.so`), so the same symbol table that Play needs is what we use locally.

| Mode | How we use them |
|------|-----------------|
| **Production (Play)** | Upload `native-debug-symbols.zip` + `mapping.txt` with each AAB. Android vitals / Play Console then show **function names** in native crashes and ANRs instead of bare addresses. Match files to the exact `version_code` that crashed. |
| **Production (sideload / shop APK)** | Keep the `native-debug-symbols.zip` from that package run next to the APK (or archive it with the release). When a shop device tombstones, pull `tombstone_*` / `logcat` and run **`ndk-stack`** against the unzipped `arm64-v8a/` from **that** build’s zip. |
| **Development (`com.khaynjo.debug`)** | Dev installs are debuggable and usually crash under `adb logcat`, but native backtraces still need symbols. After a device crash: unzip a recent package’s `native-debug-symbols.zip` (or use `target/android/aarch64-linux-android/{android-dev,release}/libkumkaj_khaynjo.so` renamed conceptually as `libmakepad.so`) and feed the tombstone through `ndk-stack`. Same idea for Moklanjo / other apps. |
| **Comparing builds** | Archive `khaynjo.aab` + `native-debug-symbols.zip` + `mapping.txt` per `version_code`. Without the matching zip, an old Play vitals stack is useless. |

**Local symbolicate (dev or prod APK crash):**

```bash
# After package-android.sh (or any build that wrote the zip):
unzip -d /tmp/khaynjo-syms dist/khaynjo-android/native-debug-symbols.zip
# Paste tombstone / tombstone-like log into crash.txt, then:
"$ANDROID_HOME"/ndk/*/ndk-stack -sym /tmp/khaynjo-syms/arm64-v8a -dump crash.txt
```

`mapping.txt` matters less day-to-day (Java is not obfuscated) but keeps Play Console clean and is the right place if we ever turn on R8. The **native zip** is the load-bearing piece for both shop crashes and Play vitals.

## After upload

- Confirm App bundle explorer → Downloads → Assets shows both **Native debug symbols** and **Deobfuscation file** for this version
- Install from Play on a clean device; create books; install the next build as an upgrade and confirm books remain
- Keep `$HOME/khaynjo-release.jks` and its password recoverable forever

## Related

- Package script: [`../scripts/package-android.sh`](../scripts/package-android.sh)
- Signing helpers: [`../../../scripts/android-env.sh`](../../../scripts/android-env.sh)
- Windows Store: [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md)
- AUR: [`PUBLISH_AUR.md`](PUBLISH_AUR.md)
