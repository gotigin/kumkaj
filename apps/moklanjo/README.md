# Moklanjo

Courier-agency business operating system under the Kumkaj brand —
books + booking desk (AWB, carriers, COD, tracking). Local-first on
Makepad 2; binary `kumkaj-moklanjo`; ids `com.moklanjo` (prod) /
`com.moklanjo.debug` (dev), from [`env/`](env/). Domain: moklanjo.com.
"Moklanjo" is the Memoni word for to send / the act of sending.

## Run (live dev)

```bash
cargo moklanjo-dev                      # desktop, --hot --remote
./apps/moklanjo/scripts/dev.sh          # same, without the cargo alias
./apps/moklanjo/scripts/android-dev.sh  # build + install + launch on device
./apps/moklanjo/scripts/windows-dev.sh  # Windows dev package → dist/moklanjo-windows-dev/
./apps/moklanjo/scripts/msi.sh [prod|dev] # MSI → dist/Moklanjo-<ver>.msi
```

Packaging: root `scripts/` (`build-msi.sh`, shared stage helpers, …) are
parameterised per app — see [`docs/DEV.md`](../../docs/DEV.md). Product
code is never shared — Moklanjo owns all of its code under this folder.

## Docs for this app

- [`docs/MOKLANJO.md`](docs/MOKLANJO.md) — product brief
- [`docs/MOKLANJO_CLIENTS_REQUIREMENTS.md`](docs/MOKLANJO_CLIENTS_REQUIREMENTS.md) — client checklist
- [`docs/COURIER_RESEARCH.md`](docs/COURIER_RESEARCH.md) — courier software market comparison
- [`docs/MOKLANJO_APP_TUTORIAL.md`](docs/MOKLANJO_APP_TUTORIAL.md) — owner tutorial
- [`docs/USE_CASES.md`](docs/USE_CASES.md) — discovery notes
- Global doctrine (vision, business-books concepts checklist, DEV/DESIGN, LAN pilot state): [`../../docs/`](../../docs/README.md)
