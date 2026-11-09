# Publish Khaynjo on the Microsoft Store

Store package: **MSIX**  
Identity Name (stable — never rename after publish): **`MearajBhagad.Khaynjo`**  
(Partner Center **Package/Identity/Name** — publisher prefix + product; same pattern for Moklanjo / Kirano / Tabiyat.)  
Publisher: **`CN=373E2948-B702-453F-A991-0E7B4F78F562`** · Display: **`Mearaj Bhagad`**  
Product version: Cargo `version` in [`../Cargo.toml`](../Cargo.toml) → MSIX `a.b.c.0` (e.g. `1.0.0` → `1.0.0.0`).  
Books data: `%APPDATA%\com.khaynjo\` (packaged classic / mediumIL — same idea as MSI).  
Compat: [`DATA_COMPAT.md`](DATA_COMPAT.md). Release index: [`RELEASE_1_0_0.md`](RELEASE_1_0_0.md).

## Prerequisites

- Partner Center account; app reserved; Identity Name + **Publisher CN** copied from **Product Identity** (must match the MSIX and the signing cert)
- Store association / signing `.pfx` from Partner Center
- Workspace able to cross-build Windows (see root [`docs/DEV.md`](../../../docs/DEV.md) § Windows) — MSIX pack runs from Linux; **signing** is usually on Windows with `signtool`
- Python 3 + Pillow on the pack host (Store logos from `resources/icon_512.png`)

## Build the MSIX

From the workspace root (Publisher defaults in `scripts/build-msix.sh` already match this account; override only if Partner Center changes):

```bash
# Optional — already the packer defaults:
# export KUMKAJ_MSIX_PUBLISHER='CN=373E2948-B702-453F-A991-0E7B4F78F562'
# export KUMKAJ_MSIX_PUBLISHER_DISPLAY='Mearaj Bhagad'

./apps/khaynjo/scripts/msix.sh
# optional: ./apps/khaynjo/scripts/msix.sh dev   → same Identity, file KhaynjoDev-*.msix
```

| Partner Center | Build |
|----------------|--------|
| Package/Identity/Name | `KUMKAJ_MSIX_IDENTITY` in `apps/<app>/scripts/msix.sh` |
| Package/Identity/Publisher | `KUMKAJ_MSIX_PUBLISHER` |
| PublisherDisplayName | `KUMKAJ_MSIX_PUBLISHER_DISPLAY` |

Outputs:

| Artifact | Path |
|----------|------|
| MSIX | `dist/Khaynjo-<ver>.msix` |
| Layout (MakeAppx fallback) | `dist/Khaynjo-<ver>-msix-layout/` |

Packing uses **Microsoft `makemsix` on Linux** (`scripts/msix/pack_msix.sh` → valid `AppxBlockMap`). First run builds the tool via `scripts/msix/ensure-makemsix.sh` (needs cmake, g++, ImageMagick).

The `.msix` may be **unsigned** for Partner Center — the Store re-signs after certification. Optional local sideload signing (Windows `signtool` + your own `.pfx`) is separate:

```bash
export KUMKAJ_MSIX_PFX=/path/to/sideload.pfx
export KUMKAJ_MSIX_PFX_PASSWORD='…'          # if required
```

Publisher in the manifest must match Partner Center **Package/Identity/Publisher**.

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
- Never change Identity Name `MearajBhagad.Khaynjo` after the first Store publish

## Related

- Scripts: [`../scripts/msix.sh`](../scripts/msix.sh), [`../scripts/msi.sh`](../scripts/msi.sh)
- Shared packer: [`../../../scripts/build-msix.sh`](../../../scripts/build-msix.sh), [`../../../scripts/msix/pack_msix.py`](../../../scripts/msix/pack_msix.py)
- Play Store: [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md)
- AUR: [`PUBLISH_AUR.md`](PUBLISH_AUR.md)
