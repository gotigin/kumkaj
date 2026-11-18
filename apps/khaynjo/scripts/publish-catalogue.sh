#!/usr/bin/env bash
# Stage Khaynjo catalogue packs into the Makepad site static tree
# (sites/khaynjo/static/v1/catalogue/) — same Firebase deploy as the wasm site.
#
# Usage:
#   ./apps/khaynjo/scripts/publish-catalogue.sh
#       # sync public/items + stage Khaynjo Full Template
#   ./apps/khaynjo/scripts/publish-catalogue.sh --public-rajdhani
#   ./apps/khaynjo/scripts/publish-catalogue.sh --public=<folder>
#       # a pack under catalogue/private-src/<folder>/, published on the public
#       # list (public/templates/<id>/). --public-rajdhani is --public=rajdhani.
#       # "Export template" (`kumkaj-khaynjo --export-template`) writes that tree.
#
# One public list (no private codes):
#   • Khaynjo Full Template (`khaynjo-items`, kind: shelf → all items)
#   • Optional authored packs (Rajdhani today) — menu + shop_defaults.json
#
# Neither is forced: DeviceSetup Skip leaves an empty shop. Café / Khaynjo
# starter folders are not DeviceSetup chips.
#
# Every pack/items listing carries size + sha256 so devices resume file by file.
# bash + jq only (no Python — .cursor/rules/no-python-scripts.mdc).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/khaynjo"
CDN="$ROOT/sites/khaynjo/static/v1/catalogue"
SHELF="$APP/assets/items"
# Authoring only — not scanned by menu_templates / not shipped in the APK.
# Published packs land on the public list, not under private/{code}/.
PACK_SRC="$APP/catalogue/private-src"

DO_PUBLIC=0
PUBLIC_NAME=""

for arg in "$@"; do
  case "$arg" in
    --public-rajdhani) DO_PUBLIC=1; PUBLIC_NAME="rajdhani" ;;
    --public=*) DO_PUBLIC=1; PUBLIC_NAME="${arg#--public=}" ;;
    --private-rajdhani|--private=*|--code=*)
      echo "error: private catalogue codes are retired. Use --public-rajdhani or --public=<folder>." >&2
      exit 1
      ;;
    -h|--help)
      sed -n '2,22p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $arg" >&2
      exit 1
      ;;
  esac
done

for tool in jq rsync sha256sum stat; do
  command -v "$tool" >/dev/null 2>&1 || { echo "error: $tool required" >&2; exit 1; }
done
if [[ ! -d "$SHELF" ]]; then
  echo "Missing shelf $SHELF" >&2
  exit 1
fi
if [[ "$DO_PUBLIC" -eq 1 && ! "$PUBLIC_NAME" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
  echo "--public= takes a folder name under catalogue/private-src/ (a-z, 0-9, -)" >&2
  exit 1
fi

mkdir -p "$CDN/public/templates" "$CDN/public/items"

# JSON array of {path, bytes, sha256} for files (relative to $1) read from stdin.
listing_json() {
  local base="$1"
  (
    cd "$base"
    while IFS= read -r rel; do
      [[ -n "$rel" ]] || continue
      printf '%s\t%s\t%s\n' "$rel" "$(stat -c %s -- "$rel")" \
        "$(sha256sum -- "$rel" | cut -d' ' -f1)"
    done
  ) | jq -R -s '
    split("\n") | map(select(length > 0) | split("\t")
      | {path: .[0], bytes: (.[1] | tonumber), sha256: .[2]})'
}

# ---------------------------------------------------------------------------
echo "1/4 Sync public/items from assets/items …"
rsync -a --delete --include='*/' --include='*.webp' --exclude='*' "$SHELF/" "$CDN/public/items/"

(cd "$CDN/public/items" && find . -mindepth 2 -maxdepth 2 -type f -name '*.webp') \
  | sed 's|^\./||' | LC_ALL=C sort | listing_json "$CDN/public/items" \
  | jq '. as $entries |
  {
    schema: 1,
    folders: ($entries | map(.path | split("/")[0]) | unique),
    files: ($entries | map(.path)),
    count: ($entries | length),
    bytes: ($entries | map(.bytes) | add // 0),
    entries: $entries
  }' > "$CDN/public/items/index.json.tmp"
mv "$CDN/public/items/index.json.tmp" "$CDN/public/items/index.json"
ITEMS_COUNT="$(jq '.count' "$CDN/public/items/index.json")"
ITEMS_BYTES="$(jq '.bytes' "$CDN/public/items/index.json")"
echo "  items index: $(jq '.folders | length' "$CDN/public/items/index.json") folders, ${ITEMS_COUNT} webps, ${ITEMS_BYTES} bytes"

# ---------------------------------------------------------------------------
# build_pack <source template dir> <dest pack dir> <visibility>
# Self-contained app-template pack: template.json with shelf: → images/…,
# optional shop_defaults.json, photos copied in, manifest with full file listing.
build_pack() {
  local src_dir="$1" dest="$2" visibility="$3"
  local src_tmpl="$src_dir/template.json"
  if [[ ! -f "$src_tmpl" ]]; then
    echo "Missing $src_tmpl" >&2
    exit 1
  fi
  local id title blurb
  # Exported packs (format 2) say whose they are; hand-written ones don't.
  if ! jq -e '(.app // "khaynjo") == "khaynjo"' "$src_tmpl" >/dev/null; then
    echo "error: $src_tmpl is not a Khaynjo template" >&2
    exit 1
  fi
  if [[ -f "$src_dir/shop_defaults.json" ]]; then
    if ! jq -e '(.app // "khaynjo") == "khaynjo"' "$src_dir/shop_defaults.json" >/dev/null; then
      echo "error: $src_dir/shop_defaults.json is not a Khaynjo template" >&2
      exit 1
    fi
    # A pack never carries PINs (owner, 2026-11-14): the Owner gives each
    # person a first PIN in Team after setup.
    if jq -e '[.staff[]? | select((.pin // "") != "")] | length > 0' \
      "$src_dir/shop_defaults.json" >/dev/null; then
      echo "error: $src_dir/shop_defaults.json has staff PINs — remove them" >&2
      exit 1
    fi
  fi
  id="$(jq -r '.id' "$src_tmpl")"
  title="$(jq -r '.title' "$src_tmpl")"
  blurb="$(jq -r '.blurb // ""' "$src_tmpl")"

  rm -rf "$dest"
  mkdir -p "$dest/images"
  local missing=0 ref rel
  while IFS= read -r ref; do
    case "$ref" in
      shelf:*)
        rel="${ref#shelf:}"
        if [[ ! -f "$SHELF/$rel" ]]; then
          echo "  error: $id → missing shelf photo $rel" >&2
          missing=$((missing + 1))
          continue
        fi
        mkdir -p "$(dirname "$dest/images/$rel")"
        cp -p "$SHELF/$rel" "$dest/images/$rel"
        ;;
      images/*)
        if [[ ! -f "$src_dir/$ref" ]]; then
          echo "  error: $id → missing pack photo $ref" >&2
          missing=$((missing + 1))
          continue
        fi
        mkdir -p "$(dirname "$dest/$ref")"
        cp -p "$src_dir/$ref" "$dest/$ref"
        ;;
      *)
        echo "  error: $id → unsupported image ref '$ref'" >&2
        missing=$((missing + 1))
        ;;
    esac
  done < <(jq -r '(.categories[] | (.image, (.items[]?.image))), (.items[]?.image)
    | select(. != null and . != "")' "$src_tmpl" | LC_ALL=C sort -u)
  if [[ "$missing" -gt 0 ]]; then
    echo "error: $id has $missing broken photo refs" >&2
    exit 1
  fi

  jq --indent 2 '
    def pack_ref: if (. // "") | startswith("shelf:") then "images/" + .[6:] else (. // "") end;
    .categories |= map(.image |= pack_ref | .items |= map(.image |= pack_ref))
    | if has("items") then .items |= map(.image |= pack_ref) else . end
  ' "$src_tmpl" > "$dest/template.json"

  local defaults_file=""
  if [[ -f "$src_dir/shop_defaults.json" ]]; then
    cp -p "$src_dir/shop_defaults.json" "$dest/shop_defaults.json"
    defaults_file="shop_defaults.json"
  fi

  # Listing goes through stdin — a 1,000-photo pack overflows argv.
  (
    echo template.json
    [[ -n "$defaults_file" ]] && echo "$defaults_file"
    cd "$dest" && find images -type f | LC_ALL=C sort
  ) | listing_json "$dest" \
    | jq --indent 2 \
    --arg id "$id" --arg title "$title" --arg blurb "$blurb" \
    --arg visibility "$visibility" --arg defaults_file "$defaults_file" \
    --arg sha "$(sha256sum "$dest/template.json" | cut -d' ' -f1)" '
    . as $files |
    {
      schema: 1,
      id: $id,
      title: $title,
      blurb: $blurb,
      kind: "app_template",
      visibility: $visibility,
      applies_shop_defaults: ($defaults_file != ""),
      shop_defaults_id: "",
      shop_defaults: $defaults_file,
      template: "template.json",
      bytes: ($files | map(.bytes) | add // 0),
      sha256: $sha,
      files: $files
    }' > "$dest/manifest.json"
  find "$dest" -type d -empty -delete
  echo "  $id: $(jq '.files | length' "$dest/manifest.json") files, $(jq '.bytes' "$dest/manifest.json") bytes ($visibility)"
}

echo "2/4 Skip legacy public starters (not DeviceSetup chips) …"
# cafe-starter / khaynjo-starter may still exist under assets/templates for
# experiments; they are not listed. Remove stale CDN packs if present.
for stale in cafe-starter khaynjo-starter; do
  if [[ -d "$CDN/public/templates/$stale" ]]; then
    rm -rf "$CDN/public/templates/$stale"
    echo "  removed stale public/templates/$stale"
  fi
done

echo "3/4 Khaynjo Full Template (photo shelf) …"
ITEMS_PACK="$CDN/public/templates/khaynjo-items"
mkdir -p "$ITEMS_PACK"
jq -n --indent 2 --argjson n "$ITEMS_COUNT" --argjson bytes "$ITEMS_BYTES" '
  {
    schema: 1,
    id: "khaynjo-items",
    title: "Khaynjo Full Template",
    blurb: "Full public dish photo shelf (~\($n) dishes). Optional — Skip leaves an empty shop.",
    kind: "shelf",
    visibility: "public",
    applies_shop_defaults: false,
    shop_defaults_id: "",
    shop_defaults: "",
    template: "",
    items_index: "../../items/index.json",
    bytes: $bytes
  }' > "$ITEMS_PACK/manifest.json"
jq -n --indent 2 '
  {
    id: "khaynjo-items",
    title: "Khaynjo Full Template",
    blurb: "Full public dish photo shelf.",
    categories: []
  }' > "$ITEMS_PACK/template.json"
echo "  khaynjo-items: shelf (${ITEMS_COUNT} files, ${ITEMS_BYTES} bytes)"

echo "4/4 Public authored pack (optional) + public/index.json …"
if [[ "$DO_PUBLIC" -eq 1 ]]; then
  SRC="$PACK_SRC/$PUBLIC_NAME"
  if [[ ! -f "$SRC/template.json" ]]; then
    echo "Missing $SRC/template.json (authoring tree; not shipped in the app)" >&2
    exit 1
  fi
  if [[ ! -f "$SRC/shop_defaults.json" ]]; then
    echo "Missing $SRC/shop_defaults.json — a shop template's defaults must live in the pack" >&2
    exit 1
  fi
  PACK_ID="$(jq -r '.id' "$SRC/template.json")"
  if [[ ! "$PACK_ID" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo "error: $SRC/template.json id must be a public template id (a-z, 0-9, -)" >&2
    exit 1
  fi
  if [[ "$PACK_ID" == "khaynjo-items" ]]; then
    echo "error: $PACK_ID is the photo shelf, not an authored pack" >&2
    exit 1
  fi
  DEST="$CDN/public/templates/$PACK_ID"
  build_pack "$SRC" "$DEST" public
  echo "Public $PUBLIC_NAME pack → $DEST"
else
  echo "  no authored pack (pass --public-rajdhani or --public=<folder> to stage one)"
fi

# khaynjo-items first, then any other public packs, so the shelf stays a chip
# beside Rajdhani (and later packs) on the one public list.
index_files=("$ITEMS_PACK/manifest.json")
while IFS= read -r man; do
  [[ -n "$man" ]] || continue
  index_files+=("$man")
done < <(find "$CDN/public/templates" -mindepth 2 -maxdepth 2 -name manifest.json \
  ! -path '*/khaynjo-items/manifest.json' | LC_ALL=C sort)
jq -s --indent 2 '{schema: 1, templates: map({id, title, blurb, kind, bytes})}' \
  "${index_files[@]}" > "$CDN/public/index.json.tmp"
mv "$CDN/public/index.json.tmp" "$CDN/public/index.json"
echo "  index.json: $(jq '.templates | length' "$CDN/public/index.json") public template(s)"

echo "Done. Site static tree: $CDN"
echo "Deploy: ./sites/khaynjo/scripts/deploy-firebase.sh"
echo "Local app test: KHAYNJO_CATALOGUE_DIR=$CDN"
