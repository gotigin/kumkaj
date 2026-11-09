# Publish Khaynjo on the Microsoft Store

Store package: **MSIX**  
Identity Name (stable — never rename after publish): **`Kumkaj.Khaynjo`**  
Product version: Cargo `version` in [`../Cargo.toml`](../Cargo.toml) → MSIX `a.b.c.0` (e.g. `1.0.0` → `1.0.0.0`).  
Books data: `%APPDATA%\com.khaynjo\` (packaged classic / mediumIL — same idea as MSI).  
Compat: [`DATA_COMPAT.md`](DATA_COMPAT.md). Release index: [`RELEASE_1_0_0.md`](RELEASE_1_0_0.md).

## Prerequisites

- Partner Center account; app name reserved; **Publisher CN** known (must match the signing cert)
- Store association / signing `.pfx` from Partner Center
- Workspace able to cross-build Windows (see root [`docs/DEV.md`](../../../docs/DEV.md) § Windows) — MSIX pack runs from Linux; **signing** is usually on Windows with `signtool`
- Python 3 + Pillow on the pack host (Store logos from `resources/icon_512.png`)

## Build the MSIX

From the workspace root:

```bash
export KUMKAJ_MSIX_PUBLISHER='CN=Your Partner Center Publisher'
export KUMKAJ_MSIX_PUBLISHER_DISPLAY='Your publisher display name'

./apps/khaynjo/scripts/msix.sh
# optional: ./apps/khaynjo/scripts/msix.sh dev   → Kumkaj.Khaynjo.Dev / KhaynjoDev-*.msix
```

Outputs:

| Artifact | Path |
|----------|------|
| MSIX | `dist/Khaynjo-<ver>.msix` |
| Layout (MakeAppx fallback) | `dist/Khaynjo-<ver>-msix-layout/` |

The package from Linux is **unsigned** unless `signtool` is on `PATH` and you set:

```bash
export KUMKAJ_MSIX_PFX=/path/to/StoreAssociation.pfx
export KUMKAJ_MSIX_PFX_PASSWORD='…'          # if required
export KUMKAJ_MSIX_TIMESTAMP_URL='http://timestamp.digicert.com'  # optional
```

### Sign on Windows (typical)

```bat
signtool sign /fd SHA256 /a /f StoreAssociation.pfx /p <password> /tr http://timestamp.digicert.com /td SHA256 dist\Khaynjo-1.0.0.msix
```

Publisher in the manifest (`KUMKAJ_MSIX_PUBLISHER`) must match the cert CN.

## Upload

1. Partner Center → Khaynjo → Packages
2. Upload the signed `.msix`
3. Complete Store listing, age rating, privacy policy, capabilities review (`runFullTrust`, network)
4. Submit for certification

## Sideload / USB (not Store)

Per-user MSI (no admin), still product version from Cargo:

```bash
./apps/khaynjo/scripts/msi.sh
# → dist/Khaynjo-<ver>.msi
```

Folder package + `Setup.bat`: `./apps/khaynjo/scripts/windows-prod.sh` → `dist/khaynjo-windows/`.  
Uninstall keeps books by default; wipe is an explicit checkbox dialog.

## After upload

- Install from Store (or sideload signed MSIX); create books; upgrade once and confirm `%APPDATA%\com.khaynjo\` survives
- Never change Identity Name `Kumkaj.Khaynjo` after the first Store publish

## Related

- Scripts: [`../scripts/msix.sh`](../scripts/msix.sh), [`../scripts/msi.sh`](../scripts/msi.sh)
- Shared packer: [`../../../scripts/build-msix.sh`](../../../scripts/build-msix.sh), [`../../../scripts/msix/pack_msix.py`](../../../scripts/msix/pack_msix.py)
- Play Store: [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md)
- AUR: [`PUBLISH_AUR.md`](PUBLISH_AUR.md)
