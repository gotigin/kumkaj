#!/usr/bin/env bash
# Keep a Kumkaj site's language packs in step with its English (locale.rs EN).
#
#   ./scripts/scaffold-site-translations.sh <site>      # khaynjo | moklanjo | kirano | tabiyat | kumkaj
#   (same as ./scripts/scaffold-ui-translations.sh --site <site>)
#
# For every table language of the shared catalogue (sites/_shared/i18n/languages.rs,
# minus English and the languages that read another table: pt-BR es-MX yue gom)
# it rewrites sites/<site>/translations/<code>.tsv so the rows follow EN order:
# a row that already has a translation keeps it; a new key gets the English
# behind the marker "@@EN@@ " — the site tests fail while any marker is left,
# so a stand-in can never ship. Removed keys are dropped.
#
# Then translate the marked rows for real (docs/DEV.md § Languages) and run
#   cargo test -p <site>-site     (kumkaj.com: kumkaj-site)
# Bash + gawk only (no Python).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITE="${1:?usage: scaffold-site-translations.sh <site>}"
DIR="$ROOT/sites/$SITE"
LOC="$DIR/src/locale.rs"
LANGS="$ROOT/sites/_shared/i18n/languages.rs"
[[ -f "$LOC" ]] || { echo "error: no $LOC" >&2; exit 1; }
mkdir -p "$DIR/translations"

# KEY<TAB>ENGLISH for every EN row (Rust \" → "; other escapes stay two characters).
en_pairs() {
  gawk '
    /^pub const EN: &\[\(&str, &str\)\] = &\[/ { on = 1; next }
    on && /^\];/ { exit }
    on && match($0, /^[[:space:]]*\("([a-z0-9_]+)", "((\\.|[^"\\])*)"\),[[:space:]]*$/, m) {
      v = m[2]; gsub(/\\"/, "\"", v); print m[1] "\t" v
    }
  ' "$LOC"
}

# CODE<TAB>NAME<TAB>NATIVE for every catalogue language.
catalogue() {
  gawk 'match($0, /code: "([^"]+)", name_en: "([^"]*)", native_name: "([^"]*)"/, m) { print m[1] "\t" m[2] "\t" m[3] }' "$LANGS"
}

PAIRS="$(mktemp)"
trap 'rm -f "$PAIRS"' EXIT
en_pairs > "$PAIRS"
n_keys="$(wc -l < "$PAIRS")"
if [[ "$n_keys" -lt 5 ]]; then
  echo "error: read only $n_keys EN rows from $LOC — parse failed?" >&2
  exit 1
fi

made=0
marked=0
while IFS=$'\t' read -r code name native; do
  case "$code" in en|pt-BR|es-MX|yue|gom) continue ;; esac
  file="$DIR/translations/$code.tsv"
  tmp="$(mktemp)"
  {
    printf '# %s — %s (%s). key<TAB>text in locale.rs EN order; keep {placeholders}; \\n = line break.\n' "$name" "$native" "$code"
    gawk -F'\t' -v old="$file" '
      BEGIN {
        if ((getline line < old) > 0) {
          do {
            if (line == "" || substr(line, 1, 1) == "#") continue
            i = index(line, "\t"); if (i == 0) continue
            have[substr(line, 1, i - 1)] = substr(line, i + 1)
          } while ((getline line < old) > 0)
        }
      }
      {
        key = $1; english = substr($0, length($1) + 2)
        if (key in have && have[key] != "") print key "\t" have[key]
        else { print key "\t@@EN@@ " english; marked++ }
      }
      END { print marked + 0 > "/dev/stderr" }
    ' "$PAIRS" 2> "$tmp.n"
  } > "$tmp"
  [[ -f "$file" ]] || made=$((made + 1))
  marked=$((marked + $(cat "$tmp.n")))
  rm -f "$tmp.n"
  mv "$tmp" "$file"
done < <(catalogue)

left="$(cat "$DIR"/translations/*.tsv | grep -c '@@EN@@ ' || true)"
echo "$SITE: $n_keys keys; packs created $made; new stand-ins $marked; stand-in rows left to translate: $left"
