# Tabiyat — store readiness

**Product** Tabiyat · **brand** Kumkaj · **company** GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED · **trademark** Gotigin.

| Channel | Status | What to run | Blocker |
|---|---|---|---|
| Google Play | Package script ready; no listing text yet | `./apps/tabiyat/scripts/package-android.sh` → `dist/tabiyat-android/` (APK; with the upload key also the Play bundle `tabiyat.aab`, plus `native-debug-symbols.zip` and `mapping.txt` to upload beside it). Application id `shop.tabiyat`, `version_code = 1` | Play Console app not created. **No upload key exists yet**: `$HOME/tabiyat-release.jks` + `.password` beside it (`scripts/android-env.sh`; create it once, back it up outside the repo). Without it the APK is debug-signed and no `.aab` is built. The bundle branch of the script was run end to end on 2026-11-21 with a throwaway key (113 MB bundle, symbols, mapping), so it works the day the key exists. Privacy URL to paste: **https://tabiyat.shop/privacy** (source `docs/PRIVACY.md`, served by `sites/tabiyat`). |
| Microsoft Store | MSIX script ready | `./apps/tabiyat/scripts/msix.sh` → `dist/Tabiyat-<ver>.msix`. Identity **`MearajBhagad.Tabiyat`** (do not rename after publish). Sideload MSI: `./apps/tabiyat/scripts/msi.sh` | Partner Center reservation, Publisher CN, and `signtool` signing are owner steps (`docs/DEV.md` § Windows). Privacy URL: **https://tabiyat.shop/privacy**. |
| Site | Live at **https://tabiyat.shop** (Firebase project `tabiyat-shop`) | `./sites/tabiyat/scripts/deploy-firebase.sh` | None. Account sign-in setup: `docs/FIREBASE_AUTH_SETUP.md`. |

`DOWNLOADS_LIVE` on the Tabiyat site stays off until a real listing or installer exists.
