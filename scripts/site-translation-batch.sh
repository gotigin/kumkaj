#!/usr/bin/env bash
# Fill the website language packs one language at a time, by English text.
# Many site texts repeat across the five sites (account / shop pages, plans,
# footer), so each distinct English text is translated once per language.
#
#   ./scripts/site-translation-batch.sh export <code> > /tmp/hi.todo.tsv
#       Every distinct English text whose row in sites/*/translations/<code>.tsv
#       still holds the "@@EN@@ " stand-in, one per line as ENGLISH<TAB>
#       (the current English from each site's src/locale.rs EN; \n = line break).
#   ./scripts/site-translation-batch.sh apply <code> /tmp/hi.done.tsv
#       done.tsv rows are ENGLISH<TAB>TRANSLATION (same escapes as the packs).
#       Writes TRANSLATION into every <code> pack row, in all five sites, whose
#       key has that English and still holds a stand-in. Reports rows written
#       and done.tsv lines that matched nothing. Translated rows are never
#       overwritten.
#   ./scripts/site-translation-batch.sh status
#       Stand-in rows left per language (all sites).
#
# <code> is a pack name (a table language: hi, ur, zh-Hant, …; not pt-BR /
# es-MX / yue / gom). Quality rules: docs/DEV.md § Languages (and → Websites).
# Afterwards: ./scripts/check-ui-translations.sh --site all, cargo test per site,
# ./scripts/site-cjk-font.sh <site> when zh / zh-Hant / ja / ko changed.
# Bash + gawk only (no Python).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITES=(khaynjo moklanjo kirano tabiyat kumkaj)
MARK='@@EN@@ '

usage() { sed -n '2,25p' "$0"; exit 2; }

# KEY<TAB>ENGLISH for one site's EN (Rust \" → "; other escapes stay two characters).
en_pairs() {
  gawk '
    /^pub const EN: &\[\(&str, &str\)\] = &\[/ { on = 1; next }
    on && /^\];/ { exit }
    on && match($0, /^[[:space:]]*\("([a-z0-9_]+)", "((\\.|[^"\\])*)"\),[[:space:]]*$/, m) {
      v = m[2]; gsub(/\\"/, "\"", v); print m[1] "\t" v
    }
  ' "$ROOT/sites/$1/src/locale.rs"
}

pack_of() {
  local file="$ROOT/sites/$1/translations/$2.tsv"
  [[ -f "$file" ]] || { echo "error: no $file (a table language, e.g. hi, zh-Hant)" >&2; exit 1; }
  printf '%s' "$file"
}

cmd="${1:-}"
case "$cmd" in
  export)
    code="${2:?usage: export <code>}"
    for site in "${SITES[@]}"; do
      pack="$(pack_of "$site" "$code")"
      en_pairs "$site" | gawk -F'\t' -v pack="$pack" -v mark="$MARK" '
        BEGIN {
          while ((getline line < pack) > 0) {
            i = index(line, "\t")
            if (i && substr(line, i + 1, length(mark)) == mark) stand[substr(line, 1, i - 1)] = 1
          }
        }
        ($1 in stand) { print substr($0, length($1) + 2) }
      '
    done | gawk '!seen[$0]++ { print $0 "\t" }'
    ;;
  apply)
    code="${2:?usage: apply <code> <done.tsv>}"
    done_file="${3:?usage: apply <code> <done.tsv>}"
    [[ -f "$done_file" ]] || { echo "error: no $done_file" >&2; exit 1; }
    used="$(mktemp)"
    trap 'rm -f "$used"' EXIT
    written=0
    for site in "${SITES[@]}"; do
      pack="$(pack_of "$site" "$code")"
      tmp="$(mktemp)"
      n="$(en_pairs "$site" | gawk -F'\t' -v pack="$pack" -v done_file="$done_file" -v mark="$MARK" -v used="$used" -v out="$tmp" '
        BEGIN {
          while ((getline line < done_file) > 0) {
            if (line == "" || substr(line, 1, 1) == "#") continue
            i = index(line, "\t"); if (i == 0) continue
            t = substr(line, i + 1); sub(/[[:space:]]+$/, "", t)
            if (t != "") tr[substr(line, 1, i - 1)] = t
          }
        }
        { english[$1] = substr($0, length($1) + 2) }
        END {
          while ((getline line < pack) > 0) {
            i = index(line, "\t")
            if (i && substr(line, i + 1, length(mark)) == mark) {
              k = substr(line, 1, i - 1)
              if ((k in english) && (english[k] in tr)) {
                print k "\t" tr[english[k]] > out
                print english[k] >> used
                n++
                continue
              }
            }
            print line > out
          }
          print n + 0
        }
      ')"
      mv "$tmp" "$pack"
      written=$((written + n))
      echo "$site/$code: $n rows written"
    done
    unmatched="$(gawk -F'\t' -v used="$used" '
      BEGIN { while ((getline l < used) > 0) hit[l] = 1 }
      $0 != "" && substr($0, 1, 1) != "#" && $2 != "" && !($1 in hit) { n++ }
      END { print n + 0 }
    ' "$done_file")"
    echo "$code: $written rows written; done.tsv lines that matched no stand-in: $unmatched"
    ;;
  status)
    for f in "$ROOT"/sites/khaynjo/translations/*.tsv; do
      code="$(basename "$f" .tsv)"
      n=0
      for site in "${SITES[@]}"; do
        c="$(grep -c "	$MARK" "$ROOT/sites/$site/translations/$code.tsv" || true)"
        n=$((n + c))
      done
      printf '%s\t%s\n' "$code" "$n"
    done | sort -k2,2nr -k1,1 | gawk -F'\t' '{ total += $2; print } END { print "total\t" total }'
    ;;
  *) usage ;;
esac
