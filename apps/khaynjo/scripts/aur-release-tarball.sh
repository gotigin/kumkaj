#!/usr/bin/env bash
# Pack dist/khaynjo-linux for AUR (khaynjo-bin). Bumps nothing — reads Cargo version.
#   ./apps/khaynjo/scripts/package-linux.sh   # first, if needed
#   ./apps/khaynjo/scripts/aur-release-tarball.sh
# → dist/khaynjo-linux-<ver>-x86_64.tar.zst (+ .sha256)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/khaynjo"
SRC="$ROOT/dist/khaynjo-linux"
VER="$(sed -n '/^\[package\]/,/^\[/p' "$APP/Cargo.toml" | sed -n 's/^version = "\(.*\)"/\1/p' | head -1)"
[[ -n "$VER" ]] || VER="1.0.0"

if [[ ! -x "$SRC/bin/kumkaj-khaynjo" ]]; then
  echo "Missing $SRC/bin/kumkaj-khaynjo — run apps/khaynjo/scripts/package-linux.sh first" >&2
  exit 1
fi

OUT="$ROOT/dist/khaynjo-linux-${VER}-x86_64.tar.zst"
rm -f "$OUT" "$OUT.sha256"
# Archive root folder name khaynjo-linux/ matches PKGBUILD.
tar -C "$ROOT/dist" -c khaynjo-linux | zstd -T0 -19 -o "$OUT"
( cd "$ROOT/dist" && sha256sum "$(basename "$OUT")" >"$(basename "$OUT").sha256" )
echo "AUR tarball → $OUT"
echo "SHA256: $(cut -d' ' -f1 "$OUT.sha256")"
echo "Next: host this file on a public HTTPS URL, set source=/sha256sums in apps/khaynjo/packaging/aur/PKGBUILD — see apps/khaynjo/docs/PUBLISH_AUR.md"
