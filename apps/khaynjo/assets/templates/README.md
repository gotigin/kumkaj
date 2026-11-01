# Client menu templates (Khaynjo)

Each folder under `assets/templates/<id>/` is a **named menu template** a
restaurant can import in one tap (categories, dishes, prices, aliases,
photos). Today: `rajdhani/`. Tomorrow: add another folder the same way —
the app discovers every `template.json` automatically.

This is separate from the big photo shelf (`assets/items/`) and from
“Import complete template” (that path imports the whole shelf at ₹0).

## How to add a new client template

1. **Create a folder** `apps/khaynjo/assets/templates/<id>/`  
   Use a short kebab id (`rajdhani`, `hotel-xyz`). The folder name should
   match the `"id"` field in the JSON.

2. **Write `template.json`** (required) — schema below.  
   - Category / item **name** = what the owner sees first (usually English).  
   - **aliases** = other names staff search by (e.g. Gujarati).  
   - For Gujarati category aliases, omit a trailing `આઈટમ` (the product word).  
   - **price** = rupees (major units). Use `0` for season rate / set later.

3. **Photos (webp only where possible)**  
   - Prefer **`shelf:<folder>/<dish>.webp`** — reuses the shared catalogue
     under `assets/items/` (no file copy in the template folder).  
   - If nothing on the shelf fits, put a webp under `images/` and set
     `"image": "images/my-dish.webp"`. Note the licence/source in that
     template folder if you fetched a free-licensed photo.  
   - Category `image` is optional; first item’s photo is a fine default.  
   - **Client templates (e.g. Rajdhani):** every category and item must
     use a **distinct** image path — no shared refs inside the template
     JSON. The live shop can still assign the same photo to two dishes
     after import; the template itself stays unique so import never
     dumps one photo across unrelated dishes.

4. **Check**  
   ```bash
   cargo test -p kumkaj-khaynjo --lib menu_templates
   ```  
   Import path is covered by store tests that load `rajdhani`.

5. **In the app**  
   Menu → **Client menu** — the new `title` appears (as a chip when more
   than one template ships) → Import → confirm.  
   Existing dishes with the same name (ignoring case) are left alone;
   importing twice adds nothing new.

6. **UI chips**  
   The Menu card has three chip slots (`client_tmpl_0` … `_2`). A fourth
   template needs another chip LiveId in `ui_script.rs` (or the card still
   lists the first three). Prefer keeping the set small.

## Layout

```text
assets/templates/<id>/
  template.json          # required — see schema below
  images/*.webp          # optional — only photos unique to this client
```

Override discovery root with `KHAYNJO_TEMPLATES_DIR` (tests / packaging).
Dev reads `CARGO_MANIFEST_DIR/assets/templates`; packaged builds look for
`assets/templates` beside the executable (same pattern as `assets/items`).

## `template.json` schema

```json
{
  "id": "rajdhani",
  "title": "Rajdhani",
  "blurb": "Short owner-facing line.",
  "categories": [
    {
      "name": "ચીકન રોસ્ટેડ",
      "aliases": ["Chicken Roasted"],
      "image": "shelf:indian-mains/tandoori-chicken.webp",
      "items": [
        {
          "name": "ચીકન તંદુરી સીના",
          "aliases": ["Chicken Tandoori Seena"],
          "price": 160,
          "image": "shelf:indian-mains/tandoori-chicken.webp"
        }
      ]
    }
  ]
}
```

| Field | Meaning |
|---|---|
| `id` | Folder name; stable import key |
| `title` / `blurb` | Menu screen card |
| `categories[].name` | Category display name (Rajdhani: Gujarati) |
| `categories[].aliases` | Other names (Rajdhani: English), searched like item aliases |
| `categories[].image` | Optional category photo |
| `items[].name` | Dish display name (Rajdhani: Gujarati) |
| `items[].aliases` | Other names (Rajdhani: English) |
| `items[].price` | Rupees (major units). `0` = unpriced / season rate |
| `image` | `shelf:<folder>/<file>.webp` **or** `images/<file>.webp` relative to this template |

## Runtime behaviour

- Import runs on the **Server** only (`Store::import_menu_template`).
- Photos are copied into the shop’s `item_images` as `custom:…` so Android /
  LAN Clients can sync them (`GET /item-image`). The shelf / template files
  stay read-only.
- Category aliases are applied when the category is **created**, or when an
  existing category still has an empty aliases list.
- Code: `apps/khaynjo/src/menu_templates.rs`.

## Example: Rajdhani

Built from the client menu scans under
`local/khaynjo/clients/rajdhani/` (`menu-items-english.txt` +
`menu-items-gujarati.txt`). All dish photos currently point at existing
`shelf:…` webps — no unique `images/` files required.
