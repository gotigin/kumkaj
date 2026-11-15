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
| **Private** | A client's pack — e.g. **Rajdhani**: that client's menu + `shop_defaults.json` (shop name + logo, staff, tax chrome, KOT, …); exported packs carry the whole setup (see "Export template") | `private/{12digit}/` only |

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
./apps/khaynjo/scripts/publish-catalogue.sh --private=<folder> --code=<12 digits>
```

Stages the Full Template + items shelf; optionally builds a private client
pack from `catalogue/private-src/<folder>/` (`--private-rajdhani` =
`--private=rajdhani`; requires `shop_defaults.json`). The script refuses a pack
whose `app` is not `khaynjo` or whose staff carry a `pin`. Deploy:
`./sites/khaynjo/scripts/deploy-firebase.sh`.

Checks: `cargo test -p kumkaj-khaynjo --lib catalogue_remote` and
`cargo test -p kumkaj-khaynjo --lib staged_ -- --include-ignored`.

### Shop name and logo in a private pack (2026-11-14)

`shop_defaults.json` may carry `"shop_name"` (the venue title) and
`"logo_base64"` (the shop mark: a WebP / PNG / JPEG, base64). Applying the App
Template writes both — the Owner PIN screen and Set up Khaynjo show them, and
the Owner can change the name there or replace the logo with Choose logo. The
logo rides **inside** `shop_defaults.json` because a pack ships only
`template.json`, `shop_defaults.json` and the menu photos `template.json`
names — the publish script and the downloader stay as they are. The app stores
it at most 512 px (`Store::VENUE_ICON_MAX_EDGE`), so encode it at 512 px:

```bash
# from the client's square logo (local/khaynjo/clients/<client>/…)
magick logo.jpeg -resize 512x512 -quality 88 logo.webp
# then set "logo_base64" in catalogue/private-src/<client>/shop_defaults.json to: base64 -w0 logo.webp
```

Then publish with `--private-rajdhani` and deploy. A Menu → Private template
re-import (items only) never touches the shop's name or logo.

## Export template — a client's own setup as a pack (2026-11-14)

We can't ask a client for every font size, KOT field and setting. They set the
shop up on their own device; the shop is then exported as a pack — the same
three things `catalogue/private-src/<client>/` holds — and we publish it under
their private code. A new device, branch or reinstall that applies it comes up
the same. Khaynjo only so far; how the client sends the file to us is still
open (owner, 2026-11-14), so today the export runs on our side from their books
(e.g. a backup copy):

```bash
# a copy is made first — opening books can upgrade them; theirs never change
./target/debug/kumkaj-khaynjo --export-template <books.sqlite3 | books folder> \
  apps/khaynjo/catalogue/private-src/<client>
# what is wrong with a pack, and what changed since the published one
./target/debug/kumkaj-khaynjo --check-template apps/khaynjo/catalogue/private-src/<client> \
  [sites/khaynjo/static/v1/catalogue/private/<code>]
./apps/khaynjo/scripts/publish-catalogue.sh --private=<client> --code=<12 digits>
# before deploying: download → apply → export must give back the same pack
PACK_CDN=sites/khaynjo/static/v1/catalogue PACK_CODE=<code> \
PACK_SOURCE=apps/khaynjo/catalogue/private-src/<client> \
  cargo test -p kumkaj-khaynjo --lib published_pack -- --ignored
```

**Format 2.** Exported packs carry `"format": 2, "app": "khaynjo"` in both
files (`menu_templates::PACK_FORMAT`). Every new field is optional, so a
hand-written pack (format 0, like Rajdhani's) reads as before; an app refuses a
pack for another app or a newer format rather than half-apply it. Code:
`store_template.rs` (export, apply, lint, diff), `menu_templates.rs` (item
fields), `template_cli.rs`.

| File | Carries |
|---|---|
| `template.json` | categories (aliases, photo) and items: price (`price_paise` when not whole rupees), kind, on/off the menu, unit, group, **station**, **tax rate**, tax bps, stock tracked (never the count), reorder level, size / add-on groups, area prices, timings, allergens, recipe; uncategorised items under `items` |
| `shop_defaults.json` | `shop_name`, `logo_base64`; owner name / username; `staff` with their screens, rights and stations (**no PINs, no passwords**); `prefs` (the settings in `store::template_pref_keys()` — every `SHOP_PREF_KEYS` setting plus country and tax wording, document names, the Floor's look); `stations`, `tax_rates`, `units`, `unit_conversions`, `areas`, `tables` (area, waiter, layout), `schemes`; `bill_format`, `kot_format`, `print_formats` (layouts only); `payment_qr_base64`, `stamp_base64` |
| `images/` | menu photos, named by their content hash |

Stations, tax rates, areas, recipe items, scheme items and waiters are
**named**, never ids. Never in a pack: customers and suppliers, sales, money,
stock counts, the audit log, PINs and passwords, pairing and devices, a
device's printers and print routes, the device language.

**Applying** (`Store::apply_app_template_pack`) goes in order: stations, tax
rates, units, areas → menu → staff, tables, schemes, settings, faces, layouts,
QR and stamp. For each list the pack's list is the list: what the new shop has
and the pack lacks (seeded GST rates, T1–T24) is removed when nothing uses it,
switched off otherwise. A Menu → Private template import stays items-only.

**No PINs.** People come in without a PIN and cannot sign in until the Owner
gives each a first one: Team → pick the person → *Starting PIN* (shown only
while they have none) → Save (`set_first_staff_pin`, `BooksOp::SetFirstStaffPin`).
After that only they change it, as before. Clients learn who has a PIN from
`domain::PIN_ON_SERVER`, never from a hash.

**The guard.** `an_exported_shop_applied_to_a_fresh_shop_exports_the_same_pack`
sets up a demo shop with something in every section (plus a customer and a
sale), exports, applies to a fresh shop, exports again and requires identical
files — a field the export writes but the import drops fails it. A new
setting that changes what a shop looks like goes in `SHOP_PREF_KEYS` or
`TEMPLATE_PREF_EXTRA_KEYS` (and so into packs); a new kind of setup data gets a
pack field and a line in that test's shop.
