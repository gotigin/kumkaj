#!/usr/bin/env bash
# Assign Rajdhani menu photos from files that actually show the dish.
#
#   ./apps/khaynjo/scripts/fix-rajdhani-images.sh          # rewrite template.json
#   ./apps/khaynjo/scripts/fix-rajdhani-images.sh --check  # exit 1 if it would change
#
# Earlier token-overlap matching put chicken soup on tandoori, masala dosa on
# mutton, and pizza on tikka. This script does not search the network and does
# not reuse a photo just because the filename shares a word. Every row below
# was decided by looking at the picture (2026-11-20 / 21).
#
# A photo is one of:
#   folder/file.webp   the public shelf, apps/khaynjo/assets/items/…  → "shelf:…"
#   images/file.webp   a photo that ships with this pack,
#                      catalogue/private-src/rajdhani/images/…        → "images/…"
#   (empty)            no honest photo — the tile shows initials, not wrong food
#
# Same-family reuse is intentional: several tandoori lines share the tandoor
# photo, several chicken gravies share butter chicken. Sources and licences of
# the photos added for this menu: images/SOURCES.md (pack) and
# assets/items/SOURCES.md (shelf files replaced because their names lied).
#
# After a menu edit: add the new rows here, run this script, then
#   ./apps/khaynjo/scripts/publish-catalogue.sh --public-rajdhani
# bash + jq only (.cursor/rules/no-python-scripts.mdc).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/khaynjo"
PACK="$APP/catalogue/private-src/rajdhani"
TEMPLATE="$PACK/template.json"
SHELF="$APP/assets/items"
CHECK=0

for arg in "$@"; do
  case "$arg" in
    --check) CHECK=1 ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "Unknown arg: $arg" >&2; exit 1 ;;
  esac
done

command -v jq >/dev/null 2>&1 || { echo "error: jq required" >&2; exit 1; }
[[ -f "$TEMPLATE" ]] || { echo "error: missing $TEMPLATE" >&2; exit 1; }

# English alias (category or item) | photo
ASSIGN="$(cat <<'TABLE'
# Categories
Chicken Roasted|indian-mains/tandoori-chicken.webp
Chicken Gravy|indian-mains/butter-chicken.webp
Chicken Fry|fast-food/fried-chicken.webp
Mutton Roasted|continental/lamb-chops.webp
Mutton Gravy|indian-mains/mutton-curry.webp
Fish & Prawns|indian-mains/macher-jhol.webp
Roti-Pav-Papad|indian-breads/chapati.webp
Eggs|indian-mains/egg-curry.webp
Rice|indian-rice/chicken-biryani.webp
# Chicken roasted — tandoor / tikka / lollipop, not satay, soup, or pizza
Chicken Tandoori Seena|indian-mains/tandoori-chicken.webp
Chicken Tandoori|indian-mains/tandoori-chicken.webp
Chicken Winter Tandoori|indian-mains/tandoori-chicken.webp
Chicken Schezwan Tandoori|indian-mains/tandoori-chicken.webp
Chicken B.B.Q. Tandoori|indian-mains/tandoori-chicken.webp
Chicken Special Tandoori|indian-mains/tandoori-chicken.webp
Chicken Tawa Tikka|indian-mains/chicken-tikka.webp
Chicken Malai Tikka|indian-mains/chicken-malai-tikka.webp
Chicken Jafrani Tikka|indian-mains/chicken-tikka.webp
Chicken Schezwan Tikka|indian-mains/chicken-tikka.webp
Chicken B.B.Q. Tikka|indian-mains/chicken-tikka.webp
Chicken Sizzler Tikka|continental/chicken-sizzler.webp
Spe. Chicken Tangdi Red 3 Piece|indian-mains/tandoori-chicken.webp
Chicken Winter Tangdi 3 Piece|indian-mains/tandoori-chicken.webp
Chicken B.B.Q. Tangdi 3 Piece|indian-mains/tandoori-chicken.webp
Chicken Schezwan Tangdi 3 Piece|indian-mains/tandoori-chicken.webp
Chicken Pathani|indian-mains/chicken-malai-tikka.webp
Chicken Lollipop|indo-chinese/chicken-lollipop.webp
Chicken Chilli Fry|indo-chinese/chilli-chicken.webp
Chicken Peri Peri Tikka|indian-mains/chicken-tikka.webp
Cheese + Butter Tandoori|indian-mains/tandoori-chicken.webp
# Chicken gravy. chicken-tikka-masala.webp is not used: it shows prawns.
Chicken Masala|indian-mains/chicken-curry.webp
Chicken Masala Green|indian-mains/chicken-saag.webp
Chicken Tawa Korma|indian-mains/chicken-korma.webp
Chicken Boneless|indian-mains/butter-chicken.webp
Chicken Tikka Masala|indian-mains/butter-chicken.webp
Chicken Tawa Masala|indian-mains/kadai-chicken.webp
Chicken Angara|indian-mains/kolhapuri-chicken.webp
Chicken Tangdi Masala|indian-mains/butter-chicken.webp
Chicken Tandoori Masala|indian-mains/butter-chicken.webp
Chicken Live Masala|indian-mains/chicken-curry.webp
Chicken Moglai Masala|indian-mains/butter-chicken.webp
Chicken Tawa Chilli|indo-chinese/chilli-chicken.webp
Chicken Gravy Lollipop|indo-chinese/chicken-lollipop.webp
# Chicken fry
Chicken Dana|indo-chinese/chicken-65.webp
Chicken 65|indo-chinese/chicken-65.webp
Chicken Crunchy|fast-food/fried-chicken.webp
# Mutton roasted — raan is the leg, chaap is chops
Mutton Raan Roasted|images/mutton-raan.webp
Mutton B.B.Q. Raan|images/mutton-raan.webp
Mutton Tawa Raan|images/mutton-raan.webp
Mutton Chaap Roasted|continental/lamb-chops.webp
Mutton B.B.Q. Roasted|continental/lamb-chops.webp
Mutton Fry Roasted|continental/lamb-chops.webp
Mutton Schezwan|images/mutton-schezwan.webp
Mutton Tawa Roasted|continental/lamb-chops.webp
Mutton Dry Choki|continental/lamb-chops.webp
# Mutton gravy
Mutton Raan Korma Masala|indian-mains/mutton-rogan-josh.webp
Mutton Tawa Raan Masala|indian-mains/mutton-curry.webp
Mutton Tawa Fry Masala|indian-mains/kosha-mangsho.webp
Mutton Tawa Korma|indian-mains/mutton-rogan-josh.webp
Mutton Tikka Masala Bota|indian-mains/mutton-curry.webp
Mutton Masala|indian-mains/mutton-curry.webp
Mutton Korma Masala|indian-mains/mutton-rogan-josh.webp
Mutton Bhuna Masala|indian-mains/kosha-mangsho.webp
Mutton Angara|indian-mains/mutton-rogan-josh.webp
Mutton Green Masala|images/mutton-green-masala.webp
Mutton Kaleji Fry|images/mutton-kaleji-fry.webp
Mutton Bheja Fry|images/mutton-bheja-fry.webp
Mutton Keema Fry|indian-mains/keema-matar.webp
Mutton Choki|indian-mains/mutton-curry.webp
# Fish and prawns
Paplet Fry|images/pomfret-fry.webp
Paplet Roasted|continental/grilled-fish.webp
Fish Masala|indian-mains/macher-jhol.webp
Jinga Masala|indian-mains/prawn-curry.webp
Fish Fry Gravy|indian-mains/macher-jhol.webp
Jinga Fry Gravy|indian-mains/prawn-curry.webp
Fish Live Masala|indian-mains/macher-jhol.webp
Jinga Live Masala|indian-mains/prawn-curry.webp
Roasted Fish|continental/grilled-fish.webp
Jinga Roasted|indian-mains/tandoori-prawns.webp
Fry Fish|indian-mains/amritsari-fish.webp
Fry Jinga|images/prawns-fry.webp
# Breads
Plain Roti|indian-breads/chapati.webp
Butter Roti|indian-breads/butter-naan.webp
Plain Pav|bakery-cafe/pav-bun.webp
Butter Pav|bakery-cafe/pav-bun.webp
Fry Papad|snacks/masala-papad.webp
Roasted Papad|snacks/masala-papad.webp
Chaas|cold-beverages/masala-chaas.webp
# Eggs — shakshuka is eggs in a tomato masala
Inda Kali|breakfast/shakshuka.webp
Inda Keema|indian-mains/egg-bhurji.webp
Live Kali|breakfast/shakshuka.webp
Live Keema|indian-mains/egg-bhurji.webp
Inda Mixture 5 Number|indian-mains/egg-bhurji.webp
Inda Masala Boil Fry 4 Number|images/egg-masala.webp
Inda Gotalo 10 Number|indian-mains/egg-bhurji.webp
Inda Moglai 8 Number|breakfast/shakshuka.webp
Inda Moti Hangama 10 Number|indian-mains/egg-bhurji.webp
Masala Half Fry 2 Number|breakfast/fried-egg.webp
Half Fry Simple 2 Number|breakfast/fried-egg.webp
Surati Kali|breakfast/shakshuka.webp
Inda Bhurji 2 Number|indian-mains/egg-bhurji.webp
Inda Bhurji Gravy 2 Number|breakfast/shakshuka.webp
Omlet 2 Number|breakfast/masala-omelette.webp
Special Omlet 2 Number|breakfast/masala-omelette.webp
Inda Boil 1 Number|breakfast/boiled-eggs.webp
Inda Schezwan 5 Number|
Inda Tawa Tikka 5 Number|
Inda Tawa Fry Cheese 5 Number|breakfast/cheese-omelette.webp
Inda B.B.Q. 5 Number|images/grilled-eggs.webp
Lasaniya Boil 4 Number|breakfast/boiled-eggs.webp
Inda Boil Fry Sada 3 Number|breakfast/boiled-eggs.webp
# Rice
Chicken Bombay Biryani|indian-rice/chicken-biryani.webp
Chicken Dum Biryani|indian-rice/lucknowi-biryani.webp
Chicken Tikka Biryani|indian-rice/chicken-biryani.webp
Chicken Tandoori Biryani|indian-rice/lucknowi-biryani.webp
Chicken Tawa Rice|indo-chinese/chicken-fried-rice.webp
Mutton Tawa Rice|images/mutton-fried-rice.webp
Mutton Tikka Biryani|indian-rice/mutton-biryani.webp
Mutton Biryani|indian-rice/mutton-biryani.webp
Fish Biryani|images/fish-biryani.webp
Jinga Biryani|indian-rice/prawn-biryani.webp
Fish Tawa Rice|images/fish-fried-rice.webp
Jinga Tawa Rice|images/prawn-fried-rice.webp
Inda Biryani|indian-rice/egg-biryani.webp
Inda Tawa Rice|indo-chinese/egg-fried-rice.webp
Fry Rice Gravy|
Chicken Gravy Rice|images/chicken-gravy-rice.webp
Inda Gravy Rice|
Jeera Rice|indian-rice/jeera-rice.webp
Rasawara Rice|images/rice-and-rassa.webp
TABLE
)"

# alias → photo, as one JSON object. A row listed twice is a mistake.
MAP="$(printf '%s\n' "$ASSIGN" | jq -R -s '
  [split("\n")[] | select(length > 0 and (startswith("#") | not))
    | (index("|") // error("row without a |: \(.)")) as $i
    | {key: .[:$i], value: .[$i + 1:]}]
  | (group_by(.key) | map(select(length > 1) | .[0].key)) as $twice
  | if ($twice | length) > 0 then error("listed twice: \($twice | join(", "))") else from_entries end
')"

# Every photo named above must be on disk.
missing=0
while IFS= read -r photo; do
  [[ -n "$photo" ]] || continue
  if [[ "$photo" == images/* ]]; then
    file="$PACK/$photo"
  else
    file="$SHELF/$photo"
  fi
  if [[ ! -f "$file" ]]; then
    echo "error: missing photo $photo (${file#"$ROOT"/})" >&2
    missing=$((missing + 1))
  fi
done < <(jq -r '[.[]] | unique | .[]' <<<"$MAP")
[[ "$missing" -eq 0 ]] || exit 1

# A pack photo nothing uses would still be committed and never published.
if [[ -d "$PACK/images" ]]; then
  while IFS= read -r file; do
    rel="images/$(basename "$file")"
    if ! jq -e --arg p "$rel" 'any(.[]; . == $p)' <<<"$MAP" >/dev/null; then
      echo "error: $rel is not assigned to any row" >&2
      exit 1
    fi
  done < <(find "$PACK/images" -maxdepth 1 -type f -name '*.webp' | LC_ALL=C sort)
fi

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT
jq --indent 2 --argjson map "$MAP" '
  def alias: (.aliases // [])[0] // error("missing alias on \(.name)");
  def photo:
    alias as $k
    | if ($map | has($k)) then $map[$k] else error("no photo decision for \($k)") end
    | if . == "" or startswith("images/") then . else "shelf:" + . end;
  ([.categories[] | alias, (.items[] | alias)]) as $rows
  | ($rows | group_by(.) | map(select(length > 1) | .[0])) as $twice
  | if ($twice | length) > 0 then error("alias used twice in the menu: \($twice | join(", "))") else . end
  | (($map | keys) - $rows) as $unused
  | if ($unused | length) > 0 then error("decisions for rows the menu no longer has: \($unused | join(", "))") else . end
  | .categories |= map(.image = photo | .items |= map(.image = photo))
' "$TEMPLATE" > "$TMP"

rows="$(jq '[.categories[] | 1, (.items[] | 1)] | length' "$TMP")"
empty="$(jq '[.categories[] | .image, (.items[] | .image) | select(. == "")] | length' "$TMP")"
changed="$(jq -n --slurpfile a "$TEMPLATE" --slurpfile b "$TMP" '
  def photos: [.categories[] | .image, (.items[] | .image)];
  [($a[0] | photos), ($b[0] | photos)] | transpose | map(select(.[0] != .[1])) | length')"

if [[ "$CHECK" -eq 1 ]]; then
  if ! cmp -s "$TMP" "$TEMPLATE"; then
    echo "template.json is out of date: $changed photo(s) would change — run this script" >&2
    exit 1
  fi
  echo "up to date: $rows rows, $empty without a photo"
  exit 0
fi

if ! cmp -s "$TMP" "$TEMPLATE"; then
  cat "$TMP" > "$TEMPLATE"
fi
echo "updated $changed photos; $empty left without a photo; $rows rows"
