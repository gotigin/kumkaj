# Publish Moklanjo on the Microsoft Store

Store package: **MSIX**  
Identity Name (stable — never rename after publish): **`MearajBhagad.Moklanjo`**  
Publisher: **`CN=373E2948-B702-453F-A991-0E7B4F78F562`** · Display: **`Mearaj Bhagad`**  
Product version: Cargo `version` in [`../Cargo.toml`](../Cargo.toml) → MSIX `a.b.c.0` (e.g. `0.1.0` → `0.1.0.0`).  
Books data: `%APPDATA%\com.moklanjo\` (packaged classic / mediumIL — same idea as MSI).  
Release tracker: [`MOKLANJO_RELEASE.md`](MOKLANJO_RELEASE.md).

## Prerequisites

- **Product cert green** before upload:

```bash
./scripts/product-cert.sh --tier=store moklanjo
```

- Partner Center account; app reserved; Identity Name + **Publisher CN** from Product Identity
- Optional Store association / signing `.pfx`
- Workspace able to cross-build Windows (root [`docs/DEV.md`](../../../docs/DEV.md) § Windows) — MSIX pack runs from Linux

## Build the MSIX

From the workspace root:

```bash
./apps/moklanjo/scripts/msix.sh
# optional: ./apps/moklanjo/scripts/msix.sh dev
```

| Partner Center | Build |
|----------------|--------|
| Package/Identity/Name | `KUMKAJ_MSIX_IDENTITY` in `apps/moklanjo/scripts/msix.sh` (`MearajBhagad.Moklanjo`) |
| Package/Identity/Publisher | `KUMKAJ_MSIX_PUBLISHER` |
| PublisherDisplayName | `KUMKAJ_MSIX_PUBLISHER_DISPLAY` |

| Artifact | Path |
|----------|------|
| MSIX | `dist/Moklanjo-<ver>.msix` |
| Layout | `dist/Moklanjo-<ver>-msix-layout/` |

The `.msix` may be **unsigned** for Partner Center (Store re-signs). Optional sideload signing:

```bash
export KUMKAJ_MSIX_PFX=/path/to/sideload.pfx
export KUMKAJ_MSIX_PFX_PASSWORD='…'
```

## Upload

1. Partner Center → Moklanjo → Packages
2. Upload the `.msix`
3. Complete listing, age rating, privacy policy (moklanjo.com/privacy), capabilities
4. Submit for certification

## Sideload / USB (not Store)

```bash
./apps/moklanjo/scripts/msi.sh
# → dist/Moklanjo-<ver>.msi
```

Folder package + `Setup.bat`: `./apps/moklanjo/scripts/package-windows.sh` → `dist/moklanjo-windows/`.

## Related

- Scripts: [`../scripts/msix.sh`](../scripts/msix.sh), [`../scripts/msi.sh`](../scripts/msi.sh)
- Play Store: [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md)
- LAN pilot: [`LAN_PILOT.md`](LAN_PILOT.md)
