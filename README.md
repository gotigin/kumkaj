# Kumkaj

Kumkaj (kumkaj.com) — brand for a **family of concrete business operating system apps**, local-first on Makepad 2. Each product feels purpose-built for its trade: **Kumkaj Courier** (courier-agency BOS) and **Khaynjo** (restaurant OS) ship today; other concrete products come later.

**The only shared thing is the common workspace.** All apps use the same Makepad 2 (`vendor/makepad`) and similar libraries because BOS apps have similar requirements (money in/out, parties, day/period books, receipts, reports). There is **no Kumkaj core** — no shared crate, shared books engine, or shared product code. Each app under `apps/<app>/` owns all of its code, even when it started as a copy of a sibling app: reuse is copy-then-diverge, duplication is preferable to coupling, and a change in one app must never impact another. Canonical doctrine: [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md) §1–2.

**App ids:** Courier production `com.kumkaj.courier` · development `com.kumkaj.courier.debug`; Khaynjo production `com.khaynjo` · development `com.khaynjo.debug` (each app's `env/`, baked by its `build.rs`). Platform matrix and data paths: [`docs/DEV.md`](docs/DEV.md).

Project documentation: **global** doctrine lives in [`docs/`](docs/); each app keeps its own code, docs and launch scripts under `apps/<app>/`. Root `scripts/` are shared build/packaging tooling only — they name no app and need the caller to say which; anything concrete to one app (its packages, installers, MSI codes) lives in `apps/<app>/scripts/`. Never product logic.

- [`docs/README.md`](docs/README.md) — product overview
- [`docs/DEV.md`](docs/DEV.md) — developer notes, workspace/platform, env/ids, packaging, publish checklist
- [`docs/DESIGN.md`](docs/DESIGN.md) — shipping UI tokens and IA decisions
- [`docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md`](docs/KUMKAJ_PRODUCT_VISION_AND_REQUIREMENTS.md) — product vision and requirements
- [`docs/BUSINESS_BOOKS.md`](docs/BUSINESS_BOOKS.md) — business-books concepts checklist (concepts, not shared code)
- [`docs/UNITS_AND_CONVERSIONS.md`](docs/UNITS_AND_CONVERSIONS.md) — units & conversions doctrine for all apps (standard units, chained custom units)
- [`docs/IDENTITY_AND_NAMES.md`](docs/IDENTITY_AND_NAMES.md) — identity doctrine for all apps (uniqueness by UUID; names are aliases)
- [`apps/courier/docs/KUMKAJ_COURIER.md`](apps/courier/docs/KUMKAJ_COURIER.md) — Kumkaj Courier product brief
- [`apps/khaynjo/docs/KHAYNJO.md`](apps/khaynjo/docs/KHAYNJO.md) — Khaynjo product brief
