# Khaynjo catalogue URLs (`/v1/catalogue`)

Remote shop starters (menus + optional shop defaults) live on the **product
base**, not inside the install binary. Fonts stay packaged; catalogue photos
and named templates download after Server is chosen on first onboard.

Sibling apps reuse the same path grammar with their own host
(`moklanjo.com`, `kirano.shop`, `tabiyat.shop`).

## Namespace

```text
https://khaynjo.com/
  download/                 # installers (planned)
  privacy/  terms/ …
  v1/
    catalogue/              # this feature — bump only `v1` on breaking changes
```

Default base (override with `KHAYNJO_CATALOGUE_BASE`):

```text
https://khaynjo.com/v1/catalogue
```

**In-repo static tree (same Firebase deploy as the Makepad site):**
`sites/khaynjo/static/v1/catalogue/` — layered onto the wasm package by
`./sites/khaynjo/scripts/deploy-firebase.sh` so
`https://khaynjo.com/v1/catalogue/…` sits on the same host as `/`,
`/privacy` (formatted from `apps/khaynjo/docs/PRIVACY.md`), and `/terms`.
Config: `sites/khaynjo/firebase.json`. Private packs may live under
`static/v1/catalogue/private/` on the host without being committed.

Offline / staging tests: `KHAYNJO_CATALOGUE_DIR` may point at that folder (or
any mirror with the same layout) so the app reads files without HTTP.

## Two optional app-level templates

Neither is forced — DeviceSetup **Skip for now** leaves an empty shop.

| Visibility | What | Where |
|---|---|---|
| **Public** | **Khaynjo Full Template** — full dish photo shelf (`kind: shelf`) | `public/index.json` → `khaynjo-items` |
| **Private** | **Rajdhani** — that client's menu + `shop_defaults.json` (staff, tax chrome, KOT, …) | `private/{12digit}/` only |

Rajdhani is **not** shipped under `assets/templates/`. Authoring lives in
`apps/khaynjo/catalogue/private-src/rajdhani/` and is published with
`publish-catalogue.sh --private-rajdhani --code=…`. The install binary has
no prestored Rajdhani menu or Zahir/Asif/Nawaz names — those come from the
downloaded pack's `template.json` + `shop_defaults.json`.

## Layout

```text
{base}/
  public/
    index.json
    templates/khaynjo-items/     # Full Template pointer (shelf)
      manifest.json
      template.json              # stub
    items/
      index.json
      {folder}/{file}.webp
  private/
    {12digit}/
      manifest.json
      template.json
      shop_defaults.json         # required when applies_shop_defaults
      images/{file}.webp
```

## `public/index.json` (schema 1)

```json
{
  "schema": 1,
  "templates": [
    {
      "id": "khaynjo-items",
      "title": "Khaynjo Full Template",
      "blurb": "Full public dish photo shelf (~1459 dishes). Optional — Skip leaves an empty shop.",
      "kind": "shelf",
      "bytes": 134302948
    }
  ]
}
```

## `manifest.json` (schema 1)

| Field | Meaning |
|---|---|
| `kind` | `app_template` or `shelf` |
| `visibility` | `public` or `private` |
| `applies_shop_defaults` | When true, apply pack-local `shop_defaults` |
| `shop_defaults` | Relative path (usually `shop_defaults.json`) |
| `shop_defaults_id` | Legacy; unused — do not hardcode client ids in the app |
| `template` | `template.json` (empty for shelf) |
| `items_index` | For shelf: path to `public/items/index.json` |
| `bytes` / `files` | Download size + resume listing |

## App helpers

| Helper | URL |
|---|---|
| Public index | `{base}/public/index.json` |
| Public pack | `{base}/public/templates/{id}/` |
| Private pack | `{base}/private/{code}/` |

Private `code` must be exactly **12 ASCII digits**.

## Device cache

```text
{data}/catalogue/public/templates/{id}/
{data}/catalogue/private/{code}/
{data}/catalogue/public/items/
```

## Publish

```text
./apps/khaynjo/scripts/publish-catalogue.sh
./apps/khaynjo/scripts/publish-catalogue.sh --private-rajdhani --code=<12 digits>
```

Stages the Full Template + items shelf; optionally builds the private
Rajdhani pack from `catalogue/private-src/rajdhani/` (requires
`shop_defaults.json`). Deploy: `./sites/khaynjo/scripts/deploy-firebase.sh`.

Checks: `cargo test -p kumkaj-khaynjo --lib catalogue_remote` and
`cargo test -p kumkaj-khaynjo --lib staged_ -- --include-ignored`.
