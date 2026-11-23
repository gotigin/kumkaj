#!/usr/bin/env bash
# Fill the website language packs one language at a time.
# Many site texts repeat across the five sites (account / shop pages, plans,
# footer), so each distinct English text is translated once per language and
# written into every site that uses it.
#
#   ./scripts/site-translation-batch.sh export <code> [--all] > hi.todo.tsv
#       One line per distinct English text whose <code> row still holds the
#       "@@EN@@ " stand-in (--all: every text):
#         ID<TAB>ENGLISH<TAB>CURRENT<TAB>USED_IN
#       ID = site:key of its first use (stable, ASCII), CURRENT = today's
#       translation (empty for a stand-in), USED_IN = the sites showing it.
#       ENGLISH uses the pack escapes: \n = line break.
#   ./scripts/site-translation-batch.sh apply <code> <file> [--overwrite]
#       <file> lines are ID<TAB>TRANSLATION. Writes TRANSLATION into every
#       site's <code> row whose English equals the ID's English — stand-in
#       rows only, or every matching row with --overwrite. Refuses a line whose
#       {placeholders} differ from the English, that is empty, holds a tab or
#       the stand-in marker; reports unknown IDs and texts still waiting.
#   ./scripts/site-translation-batch.sh glossary <code>
#       The apps' own words for <code>: ENGLISH<TAB>TRANSLATION<TAB>app:key for
#       every apps/<app> en() key that pack translates — write the sites with
#       the terms the apps already show (Day book, Counter, KOT, khata …).
#   ./scripts/site-translation-batch.sh status
#       Stand-in rows left per language (all sites).
#
# <code> is a pack name (a table language: hi, ur, zh-Hant, …; not pt-BR /
# es-MX / yue / gom). Quality rules: docs/DEV.md § Languages (→ Websites).
# Afterwards: ./scripts/check-ui-translations.sh --site all, cargo test per
# site, ./scripts/site-cjk-font.sh <site> when zh / zh-Hant / ja / ko changed.
# Bash + gawk only (no Python).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITES=(khaynjo moklanjo kirano tabiyat kumkaj)
APPS=(khaynjo moklanjo kirano tabiyat)
MARK='@@EN@@ '

usage() { sed -n '2,33p' "$0"; exit 2; }

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

check_code() {
  case "$1" in
    ''|*/*|*..*) echo "error: bad language code '$1'" >&2; exit 2 ;;
  esac
  for site in "${SITES[@]}"; do
    [[ -f "$ROOT/sites/$site/translations/$1.tsv" ]] || {
      echo "error: no sites/$site/translations/$1.tsv (use a table language: hi, zh-Hant, …)" >&2
      exit 1
    }
  done
}

# SITE<TAB>KEY<TAB>ENGLISH<TAB>PACK_VALUE for every EN row of every site.
rows() {
  local code="$1" site
  for site in "${SITES[@]}"; do
    en_pairs "$site" | gawk -F'\t' -v site="$site" -v pack="$ROOT/sites/$site/translations/$code.tsv" '
      BEGIN {
        while ((getline line < pack) > 0) {
          if (line == "" || substr(line, 1, 1) == "#") continue
          i = index(line, "\t"); if (i) val[substr(line, 1, i - 1)] = substr(line, i + 1)
        }
      }
      { k = $1; e = substr($0, length(k) + 2); print site "\t" k "\t" e "\t" ((k in val) ? val[k] : "") }
    '
  done
}

cmd="${1:-}"
case "$cmd" in
  export)
    code="${2:?usage: export <code> [--all]}"
    all="${3:-}"
    check_code "$code"
    rows "$code" | gawk -F'\t' -v mark="$MARK" -v all="$all" '
      {
        site = $1; key = $2; e = $3; v = $4
        stand = (v == "" || index(v, mark) == 1)
        if (!(e in id)) { id[e] = site ":" key; order[++n] = e; cur[e] = ""; used[e] = "" }
        if (index("," used[e] ",", "," site ",") == 0) used[e] = used[e] (used[e] == "" ? "" : ",") site
        if (stand) waiting[e] = 1
        else if (cur[e] == "") cur[e] = v
      }
      END {
        for (i = 1; i <= n; i++) {
          e = order[i]
          if (all == "--all" || (e in waiting)) print id[e] "\t" e "\t" cur[e] "\t" used[e]
        }
      }
    '
    ;;
  apply)
    code="${2:?usage: apply <code> <file> [--overwrite]}"
    file="${3:?usage: apply <code> <file> [--overwrite]}"
    overwrite="${4:-}"
    check_code "$code"
    [[ -f "$file" ]] || { echo "error: no $file" >&2; exit 1; }
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT
    rows "$code" > "$work/rows.tsv"
    # Decide what each site row gets: SITE<TAB>KEY<TAB>TRANSLATION, plus a report.
    gawk -F'\t' -v mark="$MARK" -v overwrite="$overwrite" -v plan="$work/plan.tsv" '
      function holders(s,   out, rest) {
        out = ""; rest = s
        while (match(rest, /\{[a-z_]+\}/)) { out = out substr(rest, RSTART, RLENGTH) " "; rest = substr(rest, RSTART + RLENGTH) }
        return out
      }
      function sorted(s,   a, n, i, out) {
        n = split(s, a, " "); asort(a); out = ""
        for (i = 1; i <= n; i++) if (a[i] != "") out = out a[i] " "
        return out
      }
      FNR == NR {
        site = $1; key = $2; e = $3; v = $4
        eng[site ":" key] = e
        nrow++; rsite[nrow] = site; rkey[nrow] = key; reng[nrow] = e; rval[nrow] = v
        next
      }
      /^[[:space:]]*$/ || /^#/ { next }
      {
        line = $0
        sub(/\r$/, "", line)
        i = index(line, "\t")
        if (i == 0) { bad++; print "skip (no tab): " substr(line, 1, 60) > "/dev/stderr"; next }
        idv = substr(line, 1, i - 1); t = substr(line, i + 1)
        sub(/^[[:space:]]+/, "", t); sub(/[[:space:]]+$/, "", t)
        if (!(idv in eng)) { unknown++; print "unknown ID: " idv > "/dev/stderr"; next }
        e = eng[idv]
        if (t == "") { bad++; print "empty: " idv > "/dev/stderr"; next }
        if (index(t, "\t")) { bad++; print "tab inside: " idv > "/dev/stderr"; next }
        if (index(t, mark)) { bad++; print "stand-in marker: " idv > "/dev/stderr"; next }
        if (sorted(holders(t)) != sorted(holders(e))) {
          bad++; print "placeholders differ: " idv " — English " holders(e) "/ got " holders(t) > "/dev/stderr"; next
        }
        if (gsub(/\\n/, "&", e) != gsub(/\\n/, "&", t)) print "note: line breaks (\\n) differ: " idv > "/dev/stderr"
        tr[e] = t; given++
      }
      END {
        for (r = 1; r <= nrow; r++) {
          e = reng[r]; v = rval[r]
          if (!(e in tr)) continue
          stand = (v == "" || index(v, mark) == 1)
          if (stand || overwrite == "--overwrite") {
            if (v != tr[e]) { print rsite[r] "\t" rkey[r] "\t" tr[e] > plan; written[rsite[r]]++ }
          }
        }
        for (r = 1; r <= nrow; r++) {
          e = reng[r]; v = rval[r]
          if ((v == "" || index(v, mark) == 1) && !(e in tr)) left[e] = 1
        }
        nleft = 0; for (e in left) nleft++
        printf "lines accepted %d, refused %d, unknown IDs %d; distinct texts still waiting: %d\n", given, bad + 0, unknown + 0, nleft
        for (s in written) printf "  %s: %d rows written\n", s, written[s]
      }
    ' "$work/rows.tsv" "$file"
    touch "$work/plan.tsv"
    for site in "${SITES[@]}"; do
      pack="$ROOT/sites/$site/translations/$code.tsv"
      gawk -F'\t' -v site="$site" -v plan="$work/plan.tsv" '
        BEGIN { while ((getline l < plan) > 0) { split(l, p, "\t"); if (p[1] == site) { i = index(l, "\t"); r = substr(l, i + 1); j = index(r, "\t"); nv[substr(r, 1, j - 1)] = substr(r, j + 1) } } }
        /^#/ || $0 == "" { print; next }
        { i = index($0, "\t"); k = substr($0, 1, i - 1); if (i && (k in nv)) print k "\t" nv[k]; else print }
      ' "$pack" > "$work/pack.tmp"
      cat "$work/pack.tmp" > "$pack"
    done
    ;;
  glossary)
    code="${2:?usage: glossary <code>}"
    check_code "$code"
    for app in "${APPS[@]}"; do
      loc="$ROOT/apps/$app/src/locale.rs"
      pack="$ROOT/apps/$app/translations/$code.tsv"
      [[ -f "$loc" && -f "$pack" ]] || continue
      gawk -v app="$app" -v pack="$pack" '
        BEGIN {
          while ((getline line < pack) > 0) {
            if (line == "" || substr(line, 1, 1) == "#") continue
            i = index(line, "\t"); if (i) tr[substr(line, 1, i - 1)] = substr(line, i + 1)
          }
        }
        /fn en\(key: &str\) -> String \{/ { on = 1; next }
        on && /_ => key,/ { exit }
        on && match($0, /^[[:space:]]*"([^"]+)"[[:space:]]*=>[[:space:]]*"((\\.|[^"\\])*)"/, m) {
          k = m[1]; e = m[2]; gsub(/\\"/, "\"", e)
          if ((k in tr) && tr[k] != "" && tr[k] != e) print e "\t" tr[k] "\t" app ":" k
        }
      ' "$loc"
    done | gawk -F'\t' '!seen[$1 "\t" $2]++'
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
