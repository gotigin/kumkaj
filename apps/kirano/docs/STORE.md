# Kirano — store readiness

**Product** Kirano · **brand** Kumkaj · **company** GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED · **trademark** Gotigin.

| Channel | Status | What to run | Blocker |
|---|---|---|---|
| Google Play | Package script ready; no listing text yet | `./apps/kirano/scripts/package-android.sh` → `dist/kirano-android/` (APK; with the upload key also the Play bundle `kirano.aab`, plus `native-debug-symbols.zip` and `mapping.txt` to upload beside it). Application id `shop.kirano`, `version_code = 1` | Play Console app not created. **No upload key exists yet**: `$HOME/kirano-release.jks` + `.password` beside it (`scripts/android-env.sh`; create it once, back it up outside the repo). Without it the APK is debug-signed and no `.aab` is built. The bundle branch of the script was run end to end on 2026-11-21 with a throwaway key (124 MB bundle, symbols, mapping), so it works the day the key exists. Privacy URL to paste: **https://kirano.shop/privacy** (source `docs/PRIVACY.md`, served by `sites/kirano`). |
| Microsoft Store | MSIX script ready | `./apps/kirano/scripts/msix.sh` → `dist/Kirano-<ver>.msix`. Identity **`MearajBhagad.Kirano`** (do not rename after publish). Sideload MSI: `./apps/kirano/scripts/msi.sh` | Partner Center reservation, Publisher CN, and `signtool` signing are owner steps (`docs/DEV.md` § Windows). Privacy URL: **https://kirano.shop/privacy**. |
| Site | Live at **https://kirano.shop** (Firebase project `kirano-shop`) | `./sites/kirano/scripts/deploy-firebase.sh` | None. Account sign-in setup: `docs/FIREBASE_AUTH_SETUP.md`. |

`DOWNLOADS_LIVE` on the Kirano site stays off until a real listing or installer exists.
