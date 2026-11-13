# Tabiyat

**Tabiyat** is the Kumkaj **pharmacy / chemist** operating system:
counter medicine sell, barcode & salt search, batch/expiry bookkeeping,
khata, supplier buys, GST bills, and Tabiyat’s own books underneath.
Local-first on Makepad 2.

| | |
|---|---|
| Binary | `kumkaj-tabiyat` |
| Ids | `shop.tabiyat` · `shop.tabiyat.debug` |
| Domain | **tabiyat.shop** |
| Venue | Counter only (no floor / kitchen / tables) |

Tabiyat is a **concrete product** in the Kumkaj workspace family. It owns
all of its product code under `apps/tabiyat/`. There is **no Kumkaj core**.
Sibling apps must never be required to change for a Tabiyat fix.

## Compliance posture

Schedule H / Rx flags and prescription notes are **owner bookkeeping**,
not Drugs & Cosmetics Act compliance software. Tabiyat does not validate
prescriptions, auto-build statutory registers, or give medical advice.

## Product face

```text
Books · Sell · Buy · Medicines · Sales · Inventory · Credit · Reports · …
```

Sell = name / salt / barcode → qty → pay or credit. Buy = restock with
batch & expiry. Empty catalogue — the owner adds their own medicines.

## Run (live dev)

```bash
cargo tabiyat-dev
cargo tabiyat-android-dev
cargo tabiyat-windows-dev
./apps/tabiyat/scripts/windows-prod.sh
./apps/tabiyat/scripts/msi.sh [prod|dev]
./apps/tabiyat/scripts/dev.sh
./apps/tabiyat/scripts/package-android.sh
./apps/tabiyat/scripts/uninstall-android-dev.sh
```

## App icon / logo

Source: `assets/tabiyat-app-icon.png`. Generate with
`./apps/tabiyat/scripts/generate-icons.sh`. Theme:
[`docs/DESIGN.md`](docs/DESIGN.md).

## Docs

- [`docs/TABIYAT.md`](docs/TABIYAT.md) — product brief and feature map
- [`docs/PHARMACY_MARKET_RESEARCH.md`](docs/PHARMACY_MARKET_RESEARCH.md) — pharmacy POS research & lessons
- [`docs/TABIYAT_WEBSITE.md`](docs/TABIYAT_WEBSITE.md) — site copy notes
- Workspace doctrine: [`../../docs/`](../../docs/README.md)

If you find restaurant words (Floor, Kitchen, KOT, waiter, dine-in) in
**owner-facing** UI or this README, that is a bug.
