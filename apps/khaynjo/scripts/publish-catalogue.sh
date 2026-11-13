#!/usr/bin/env bash
# Stage Khaynjo catalogue packs into the Makepad site static tree
# (sites/khaynjo/static/v1/catalogue/) — same Firebase deploy as the wasm site.
#
# Usage:
#   ./apps/khaynjo/scripts/publish-catalogue.sh
#       # sync public/items + stage Khaynjo Full Template (public, optional)
#   ./apps/khaynjo/scripts/publish-catalogue.sh --private-rajdhani --code=123456789012
#
# Two optional app-level templates:
#   • Public  — Khaynjo Full Template (`khaynjo-items`, kind: shelf → all items)
#   • Private — Rajdhani (menu + shop_defaults.json from catalogue/private-src/)
#
# Neither is forced: DeviceSetup Skip leaves an empty shop. Public starters
# (cafe / khaynjo-starter) are not DeviceSetup chips.
#
# Every pack/items listing carries size + sha256 so devices resume file by file.
# Private packs → sites/khaynjo/static/v1/catalogue/private/<code>/ (gitignored).
# bash + jq only (no Python — .cursor/rules/no-python-scripts.mdc).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/khaynjo"
CDN="$ROOT/sites/khaynjo/static/v1/catalogue"
SHELF="$APP/assets/items"
# Authoring only — not scanned by menu_templates / not shipped in the APK.
PRIVATE_SRC="$APP/catalogue/private-src"

PRIVATE_CODE=""
DO_RAJDHANI=0

for arg in "$@"; do
  case "$arg" in
    --private-rajdhani) DO_RAJDHANI=1 ;;
    --code=*) PRIVATE_CODE="${arg#--code=}" ;;
    -h|--help)
      sed -n '2,20p' "$0"
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
if [[ "$DO_RAJDHANI" -eq 1 && ! "$PRIVATE_CODE" =~ ^[0-9]{12}$ ]]; then
  echo "--code= must be exactly 12 digits" >&2
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
  done < <(jq -r '.categories[] | (.image, (.items[]?.image)) | select(. != null and . != "")' \
    "$src_tmpl" | LC_ALL=C sort -u)
  if [[ "$missing" -gt 0 ]]; then
    echo "error: $id has $missing broken photo refs" >&2
    exit 1
  fi

  jq --indent 2 '
    def pack_ref: if (. // "") | startswith("shelf:") then "images/" + .[6:] else (. // "") end;
    .categories |= map(.image |= pack_ref | .items |= map(.image |= pack_ref))
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

echo "3/4 Khaynjo Full Template (public items shelf) + public/index.json …"
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

jq --indent 2 '{id, title, blurb, kind, bytes}' "$ITEMS_PACK/manifest.json" \
  | jq -s --indent 2 '{schema: 1, templates: .}' > "$CDN/public/index.json"
echo "  index.json: $(jq '.templates | length' "$CDN/public/index.json") public template (Full only)"

echo "4/4 Private Rajdhani (optional) …"
if [[ "$DO_RAJDHANI" -eq 1 ]]; then
  SRC="$PRIVATE_SRC/rajdhani"
  if [[ ! -f "$SRC/template.json" ]]; then
    echo "Missing $SRC/template.json (authoring tree; not shipped in the app)" >&2
    exit 1
  fi
  if [[ ! -f "$SRC/shop_defaults.json" ]]; then
    echo "Missing $SRC/shop_defaults.json — Rajdhani defaults must live in the pack" >&2
    exit 1
  fi
  DEST="$CDN/private/$PRIVATE_CODE"
  build_pack "$SRC" "$DEST" private
  jq --indent 2 '.blurb = "Client menu + shop defaults from this pack (optional private code)."' \
    "$DEST/manifest.json" > "$DEST/manifest.json.tmp"
  mv "$DEST/manifest.json.tmp" "$DEST/manifest.json"
  # Recompute bytes after blurb-only edit (files unchanged).
  if grep -qi rajdhani "$CDN/public/index.json"; then
    echo "error: private pack leaked into public/index.json" >&2
    exit 1
  fi
  echo "Private Rajdhani pack → $DEST (keep the code off public git / docs)"
else
  echo "  skipped (pass --private-rajdhani --code=… to stage)"
fi

echo "Done. Site static tree: $CDN"
echo "Deploy: ./sites/khaynjo/scripts/deploy-firebase.sh"
echo "Local app test: KHAYNJO_CATALOGUE_DIR=$CDN"
