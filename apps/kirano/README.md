# Kirano

**Kirano** is the Kumkaj **grocery / kirana** shop operating system:
counter sell, stock, khata, supplier buys, GST cash memo / tax invoice,
and Kirano’s own books underneath. Local-first on Makepad 2.

| | |
|---|---|
| Binary | `kumkaj-kirano` |
| Ids | `shop.kirano` · `shop.kirano.debug` |
| Domain | **kirano.shop** |
| Venue | Counter only (no floor / kitchen / tables) |

Kirano is a **concrete product** in the Kumkaj workspace family. It owns
all of its product code under `apps/kirano/`. There is **no Kumkaj core**
and no shared books crate — reuse is copy-then-diverge only. Sibling apps
(Moklanjo, Khaynjo, Tabiyat) must never be required to change for a
Kirano fix.

## Product face

```text
Books · Sell · Buy · Items · Sales · Inventory · Credit · Reports · …
```

Sell = item / short code / barcode → qty (kg/pc) → cash, UPI, or khata.
Buy = restock (optional batch/expiry). Empty catalogue — the owner adds
their own SKUs. Floor, Kitchen, and KOT are not part of this product.

## Run (live dev)

```bash
cargo kirano-dev                      # desktop, --hot --remote
cargo kirano-android-dev              # build + install + launch on device
cargo kirano-windows-dev              # Windows dev package → dist/kirano-windows-dev/
./apps/kirano/scripts/windows-prod.sh  # Windows prod package → dist/kirano-windows/
./apps/kirano/scripts/msi.sh [prod|dev] # MSI → dist/Kirano-<ver>.msi
./apps/kirano/scripts/dev.sh
./apps/kirano/scripts/package-android.sh
./apps/kirano/scripts/uninstall-android-dev.sh
```

Packaging and scripts live under `apps/kirano/scripts/` (own MSI upgrade
codes, `Setup.bat`, install/uninstall). Root `scripts/` is shared
platform tooling only.

## App icon / logo

Source: `assets/kirano-app-icon.png`. Generate icon set:

```bash
./apps/kirano/scripts/generate-icons.sh
```

Theme tokens: [`docs/DESIGN.md`](docs/DESIGN.md) (sampled from the logo).

## Docs

- [`docs/KIRANO.md`](docs/KIRANO.md) — product brief and feature map
- [`docs/KIRANA_MARKET_RESEARCH.md`](docs/KIRANA_MARKET_RESEARCH.md) — kirana POS research & lessons
- [`docs/KIRANO_WEBSITE.md`](docs/KIRANO_WEBSITE.md) — site copy notes
- Workspace doctrine: [`../../docs/`](../../docs/README.md)

If you find restaurant words (Floor, Kitchen, KOT, waiter, dine-in) in
**owner-facing** UI or this README, that is a bug.
