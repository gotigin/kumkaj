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

`bytes` is the full download size of the pack (template + photos; shelf:
every photo) — the same number as that pack's `manifest.json`.

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
  "bytes": 12245562,
  "sha256": "…",
  "files": [
    {"path": "template.json", "bytes": 30282, "sha256": "…"},
    {"path": "images/indian-mains/tandoori-chicken.webp", "bytes": 91234, "sha256": "…"}
  ]
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
| `bytes` | Total download size: every file in the pack (shelf: every photo) |
| `sha256` | Optional hex digest of `template.json` |
| `files` | Every pack file with exact `bytes` + `sha256` (resume / verify). Optional — older packs derive the list from `template.json` |

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
{data}/catalogue/public/items/          # khaynjo-items shelf
```

Import reads `template.json` + `images/` (and pack-local `items/` for
legacy `shelf:` keys) from that cache.

### Resumable download (every platform)

Windows, Linux and Android use the same code (`catalogue_remote.rs`):
HTTPS at `KHAYNJO_CATALOGUE_BASE`, or the `KHAYNJO_CATALOGUE_DIR` mirror.
No platform reads APK / package-bundled menu assets while
`assets/CATALOGUE_REMOTE` is present.

- Files download **one at a time** into a staging sibling
  (`…/items.part/`, `…/templates/{id}.part/`). Each file is written as
  `<file>.dl` and renamed, so a file under its real name is whole.
- `download.state.json` in staging records the listing (sha256 of the
  manifest or items index) and every finished file (`bytes`, `sha256`).
- **Cancel, lost connection, crash or app restart** keep staging. The next
  Import skips files already there (state + size, or sha256 when the state
  file was lost) and fetches only the rest. DeviceSetup says
  "Import again to resume".
- **New publish** (listing sha changed): files proven by sha256 are kept,
  unlisted ones (removed dishes) are pruned, the rest are fetched.
- When every file is in, staging is renamed over the final dir in one step —
  the shelf scanner and import never see a half pack. A finished pack whose
  listing still matches is reused with no file fetches; offline, a finished
  pack is reused when the manifest cannot load.
- Each file is retried twice on a network error / HTTP 5xx before the
  attempt stops. A listed file that 404s reports `Missing on server: …`
  (a publish fault — not "Code not found").
- Progress: DeviceSetup shows `Downloading… 37% (47.4 MB / 128.1 MB)` + Cancel
  (file counts for packs under 0.5 MB).
- Manual test on a desktop / device: `KHAYNJO_CATALOGUE_DIR=<staged tree>`
  plus `KHAYNJO_CATALOGUE_DELAY_MS=40` slows the mirror to ~1 min for the
  full shelf so Cancel → kill → relaunch → Import can be tried by hand.

## Publish

```text
./apps/khaynjo/scripts/publish-catalogue.sh
./apps/khaynjo/scripts/publish-catalogue.sh --private-rajdhani --code=<12 digits>
```

Bash + `jq` (no Python). Syncs the full `assets/items/` tree to
`public/items/` and writes `public/items/index.json` (`files`, plus
`entries` with size + sha256 and the total `bytes`). Builds every public
app-template pack from `apps/khaynjo/assets/templates/<id>/template.json`
(**khaynjo-starter**, **cafe-starter**): `shelf:…` photos are copied into the
pack's `images/`, and `manifest.json` lists every file. Writes the
**khaynjo-items** shelf pointer and `public/index.json` (`bytes` = full
download size, same as each manifest). A broken photo ref fails the publish.
Private packs under `…/private/{code}/` are gitignored — they deploy with the
site from whichever machine staged them, so stage only the real code and
delete test codes before `deploy-firebase.sh`.

Checks: `cargo test -p kumkaj-khaynjo --lib catalogue_remote` (includes
`staged_tree_publishes_complete_listings`) and
`cargo test -p kumkaj-khaynjo --lib staged_ -- --include-ignored`
(downloads + imports every staged pack, full shelf with cancel/resume).
