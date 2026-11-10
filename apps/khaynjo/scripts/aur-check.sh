#!/usr/bin/env bash
# Confirm the AUR package is a source build (compiles on the user's machine).
# Does not pack a binary and does not upload anything.
#   ./apps/khaynjo/scripts/aur-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/khaynjo"
PKGBUILD="$APP/packaging/aur/PKGBUILD"
VER="$(sed -n '/^\[package\]/,/^\[/p' "$APP/Cargo.toml" | sed -n 's/^version = "\(.*\)"/\1/p' | head -1)"
[[ -n "$VER" ]] || {
  echo "Could not read version from $APP/Cargo.toml" >&2
  exit 1
}

fail() {
  echo "AUR check failed: $*" >&2
  exit 1
}

grep -q '^pkgname=khaynjo$' "$PKGBUILD" || fail "pkgname must be khaynjo (source), not a -bin package"
grep -q '^pkgver='"$VER"'$' "$PKGBUILD" || fail "PKGBUILD pkgver must be $VER (apps/khaynjo/Cargo.toml)"
if grep -q '^pkgname=khaynjo-bin$' "$PKGBUILD"; then
  fail "still a binary package"
fi
grep -q 'setup-makepad.sh' "$PKGBUILD" || fail "prepare() must run setup-makepad.sh (Makepad is not in git)"
grep -q 'cargo build --frozen --release -p kumkaj-khaynjo' "$PKGBUILD" || fail "build() must cargo-build kumkaj-khaynjo"
if grep -q 'tar\.zst' "$PKGBUILD"; then
  fail "PKGBUILD must not download a prebuilt tarball"
fi

for f in \
  "$APP/resources/fonts/fa-solid-900.ttf" \
  "$APP/resources/fonts/NotoSans-Regular.ttf" \
  "$APP/resources/icon_256.png" \
  "$ROOT/scripts/setup-makepad.sh" \
  "$ROOT/patches/makepad"
do
  [[ -e "$f" ]] || fail "missing $f"
done

REL="$(sed -n 's/^pkgrel=\(.*\)/\1/p' "$PKGBUILD" | head -1)"
echo "AUR source package ok: khaynjo ${VER}-${REL} (builds from git tag v${VER})"
echo "Next: tag that commit (git tag v$VER && git push origin v$VER), then push packaging/aur/PKGBUILD to the AUR repo khaynjo."
echo "See apps/khaynjo/docs/PUBLISH_AUR.md"
