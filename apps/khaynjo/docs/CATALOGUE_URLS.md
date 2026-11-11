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

## Layout

```text
{base}/
  public/
    index.json
    templates/{id}/
      manifest.json
      template.json
      images/{file}.webp
    items/
      index.json                        # folders + file list
      {folder}/{file}.webp              # full public dish shelf
  private/
    {12digit}/
      manifest.json
      template.json
      images/{file}.webp
```

- **Public** — listed in `public/index.json`; anyone may download. Live packs:
  **Khaynjo starter menu** (`app_template`), **Café starter**, **Khaynjo Items
  Template** (`shelf` → `public/items/`).
- **Private** — code-as-path (e.g. Rajdhani). Never list private packs in
  `index.json`. Revoke by deleting that folder on the host.
- The Makepad site loads **curated** dish stills from its own
  `resources/dishes/` (not the full 132MB shelf inside wasm). The CDN shelf
  is for the app + deep links.

## `public/index.json` (schema 1)

```json
{
  "schema": 1,
  "templates": [
    {
      "id": "indian-restaurant-starter",
      "title": "Indian restaurant starter",
      "blurb": "Common categories and dishes to edit.",
      "kind": "app_template",
      "bytes": 4200000
    }
  ]
}
```

## `manifest.json` (schema 1)

```json
{
  "schema": 1,
  "id": "rajdhani",
  "title": "Rajdhani Menu Template",
  "blurb": "Client menu + shop defaults.",
  "kind": "app_template",
  "visibility": "private",
  "applies_shop_defaults": true,
  "shop_defaults_id": "rajdhani",
  "template": "template.json",
  "bytes": 1234567,
  "sha256": "…"
}
```

| Field | Meaning |
|---|---|
| `kind` | `app_template` (DeviceSetup), `menu_template`, or `shelf` (full photo shelf) |
| `visibility` | `public` or `private` |
| `applies_shop_defaults` | When true, apply named shop defaults after menu import |
| `shop_defaults_id` | e.g. `rajdhani` → Owner Zahir / Admin Asif / … |
| `template` | Relative path to `template.json` inside the pack (empty for shelf) |
| `items_index` | For `shelf`: relative path to `public/items/index.json` |
| `sha256` | Optional hex digest of `template.json` |

`template.json` matches the in-app `MenuTemplate` schema (see
`assets/templates/README.md`). Prefer `"image": "images/….webp"` so packs are
self-contained.

## App helpers

| Helper | URL |
|---|---|
| Public index | `{base}/public/index.json` |
| Public pack | `{base}/public/templates/{id}/` |
| Private pack | `{base}/private/{code}/` |

Private `code` must be exactly **12 ASCII digits**.

## Device cache

After download, packs land under the app data dir:

```text
{data}/catalogue/public/templates/{id}/
{data}/catalogue/private/{code}/
```

Import reads `template.json` + `images/` (and pack-local `items/` for
legacy `shelf:` keys) from that cache.

## Publish

```text
./apps/khaynjo/scripts/publish-catalogue.sh
./apps/khaynjo/scripts/publish-catalogue.sh --private-rajdhani --code=123456789012
```

Syncs the full `assets/items/` tree to `public/items/`, rebuilds
**khaynjo-starter** (self-contained world-cuisine app template) and the
**khaynjo-items** shelf pointer, refreshes `public/index.json`. Private
packs under `…/private/{code}/` are gitignored — deploy with the site or
upload separately. App-template packs rewrite `shelf:…` into pack
`images/`.
