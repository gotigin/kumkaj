# Kumkaj Courier

Full courier-agency business operating system (books + booking desk) —
the current Kumkaj product. Local-first on Makepad 2; binary
`kumkaj-courier`; ids `com.kumkaj.courier` (prod) /
`com.kumkaj.courier.debug` (dev), from [`env/`](env/).

## Run (live dev)

```bash
cargo courier-dev                      # desktop, --hot --remote
./apps/courier/scripts/dev.sh          # same, without the cargo alias
./apps/courier/scripts/android-dev.sh  # build + install + launch on device
./apps/courier/scripts/windows-dev.sh  # Windows dev package → dist/windows-dev/
```

Packaging (root build/packaging tooling only, this app by default): root `scripts/`
(`package-windows.sh`, `build-msi.sh`, `package-android.sh`, …) —
see [`docs/DEV.md`](../../docs/DEV.md). Tooling is parameterised per app;
product code is never shared — Courier owns all of its code under this folder.

## Docs for this app

- [`docs/KUMKAJ_COURIER.md`](docs/KUMKAJ_COURIER.md) — product brief
- [`docs/COURIER_CLIENTS_REQUIREMENTS.md`](docs/COURIER_CLIENTS_REQUIREMENTS.md) — client checklist
- [`docs/COURIER_APP_TUTORIAL.md`](docs/COURIER_APP_TUTORIAL.md) — owner tutorial
- [`docs/USE_CASES.md`](docs/USE_CASES.md) — discovery notes
- Global doctrine (vision, business-books concepts checklist, DEV/DESIGN): [`../../docs/`](../../docs/README.md)
