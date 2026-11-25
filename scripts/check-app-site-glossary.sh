#!/usr/bin/env bash
# Fail when an app pack's business headword disagrees with the site packs for
# the same language and the same English label. Catches horseman-for-rider /
# tip-as-advice style drift after site reviews (see APP_GLOSSARY_ISSUES.md §
# Without native speakers).
#
#   ./scripts/check-app-site-glossary.sh           # every table language
#   ./scripts/check-app-site-glossary.sh hi ur zu  # only these codes
#
# Exact English labels only (shop buttons / short chrome). Help sentences are
# not compared. Bash + gawk only — no Python.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPS=(khaynjo moklanjo kirano tabiyat)
SITES=(khaynjo moklanjo kirano tabiyat kumkaj)

# Short shop labels that must match across apps and sites (exact English).
HEADWORDS=(
  Rider Riders Parcel Carrier Consignor Consignee
  Tip Occupied Served Parties Party
  Sell Buy Order Counter Floor Kitchen
  Waiter Items Offline OFFLINE
)

CODES=("$@")
if [[ ${#CODES[@]} -eq 0 ]]; then
  mapfile -t CODES < <(
    for f in "$ROOT"/apps/khaynjo/translations/*.tsv; do
      basename "$f" .tsv
    done | sort -u
  )
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
problems="$tmp/problems.txt"
: > "$problems"

HEADS="$(printf '%s\n' "${HEADWORDS[@]}")"
export HEADS

# ENGLISH<TAB>TRANSLATION from an app locale.rs + pack (first English wins).
pack_en_tr() {
  local locale="$1" pack="$2" out="$3"
  gawk -v pack="$pack" '
    BEGIN {
      while ((getline line < pack) > 0) {
        if (line == "" || substr(line, 1, 1) == "#") continue
        i = index(line, "\t"); if (!i) continue
        tr[substr(line, 1, i - 1)] = substr(line, i + 1)
      }
    }
    /fn en\(key: &str\) -> String \{/ { on = 1; next }
    on && /_ => key,/ { exit }
    on && match($0, /^[[:space:]]*"([^"]+)"[[:space:]]*=>[[:space:]]*"((\\.|[^"\\])*)"/, m) {
      k = m[1]; e = m[2]; gsub(/\\"/, "\"", e)
      if ((k in tr) && tr[k] != "" && !(e in seen)) {
        seen[e] = 1
        print e "\t" tr[k]
      }
    }
  ' "$locale" > "$out"
}

# Same for a site EN table + pack.
site_en_tr() {
  local locale="$1" pack="$2" out="$3"
  gawk -v pack="$pack" '
    BEGIN {
      while ((getline line < pack) > 0) {
        if (line == "" || substr(line, 1, 1) == "#") continue
        i = index(line, "\t"); if (!i) continue
        tr[substr(line, 1, i - 1)] = substr(line, i + 1)
      }
    }
    /^pub const EN: &\[\(&str, &str\)\] = &\[/ { on = 1; next }
    on && /^\];/ { exit }
    on && match($0, /^[[:space:]]*\("([a-z0-9_]+)", "((\\.|[^"\\])*)"\),[[:space:]]*$/, m) {
      k = m[1]; e = m[2]; gsub(/\\"/, "\"", e)
      if ((k in tr) && tr[k] != "" && tr[k] !~ /^@@EN@@ / && !(e in seen)) {
        seen[e] = 1
        print e "\t" tr[k]
      }
    }
  ' "$locale" > "$out"
}

for code in "${CODES[@]}"; do
  site_merged="$tmp/site-$code.tsv"
  : > "$site_merged"
  for site in "${SITES[@]}"; do
    sp="$ROOT/sites/$site/translations/$code.tsv"
    sl="$ROOT/sites/$site/src/locale.rs"
    [[ -f "$sp" && -f "$sl" ]] || continue
    site_en_tr "$sl" "$sp" "$tmp/one-site.tsv"
    cat "$tmp/one-site.tsv" >> "$site_merged"
  done
  [[ -s "$site_merged" ]] || continue
  gawk -F'\t' '!seen[$1]++ { print }' "$site_merged" > "$tmp/site-uniq.tsv"

  for app in "${APPS[@]}"; do
    ap="$ROOT/apps/$app/translations/$code.tsv"
    al="$ROOT/apps/$app/src/locale.rs"
    [[ -f "$ap" && -f "$al" ]] || continue
    pack_en_tr "$al" "$ap" "$tmp/app.tsv"
    gawk -F'\t' -v code="$code" -v app="$app" '
      function norm(s) {
        # U+2018 LEFT SINGLE QUOTATION, U+2019 RIGHT, U+02BC MODIFIER LETTER APOSTROPHE
        gsub("\xe2\x80\x98", "\x27", s)
        gsub("\xe2\x80\x99", "\x27", s)
        gsub("\xca\xbc", "\x27", s)
        return s
      }
      BEGIN {
        n = split(ENVIRON["HEADS"], h, "\n")
        for (i = 1; i <= n; i++) if (h[i] != "") want[h[i]] = 1
        while ((getline line < ARGV[1]) > 0) {
          split(line, p, "\t")
          if (p[1] != "") site[p[1]] = p[2]
        }
        delete ARGV[1]
      }
      ($1 in want) && ($1 in site) {
        if (norm($2) != norm(site[$1]))
          printf "%s\t%s\t%s\tapp=%s\tsite=%s\n", code, app, $1, $2, site[$1]
      }
    ' "$tmp/site-uniq.tsv" "$tmp/app.tsv" >> "$problems"
  done
done

if [[ -s "$problems" ]]; then
  echo "app/site glossary mismatches (exact English headwords):" >&2
  sort -u "$problems" | head -200 >&2
  n="$(sort -u "$problems" | wc -l)"
  echo "... $n distinct mismatch line(s). Align app packs with site wording (exportall / appapply)." >&2
  exit 1
fi

echo "app/site glossary headwords: OK (${#CODES[@]} language(s))"
