# Kirano — store readiness

**Product** Kirano · **brand** Kumkaj · **company** GOTIGIN SOFTWARE & HARDWARE PRIVATE LIMITED · **trademark** Gotigin.

| Channel | Status | What to run | Blocker |
|---|---|---|---|
| Google Play | Package script ready; no listing text yet | `./apps/kirano/scripts/package-android.sh` → `dist/kirano-android/` (APK, and the Play `.aab` when the upload key exists). Application id `shop.kirano`, `version_code = 1` | Play Console app not created. Upload key: `$HOME/kirano-release.jks` + `.password` beside it (`scripts/android-env.sh`); without it the APK is debug-signed and no `.aab` is built. Privacy URL to paste: **https://kirano.shop/privacy** (source `docs/PRIVACY.md`, served by `sites/kirano`). |
| Microsoft Store | MSIX script ready | `./apps/kirano/scripts/msix.sh` → `dist/Kirano-<ver>.msix`. Identity **`MearajBhagad.Kirano`** (do not rename after publish). Sideload MSI: `./apps/kirano/scripts/msi.sh` | Partner Center reservation, Publisher CN, and `signtool` signing are owner steps (`docs/DEV.md` § Windows). Privacy URL: **https://kirano.shop/privacy**. |
| Site | Live at **https://kirano.shop** (Firebase project `kirano-shop`) | `./sites/kirano/scripts/deploy-firebase.sh` | None. Account sign-in setup: `docs/FIREBASE_AUTH_SETUP.md`. |

`DOWNLOADS_LIVE` on the Kirano site stays off until a real listing or installer exists.
