# Publish Khaynjo on the Arch User Repository (AUR)

Package name: **`khaynjo`**. It is a source package. `makepkg` fetches
Makepad, applies `patches/makepad/`, and compiles `kumkaj-khaynjo` on the
machine that installs it. Do not publish a `-bin` package or a prebuilt
Linux tarball.

Product version: Cargo `version` → AUR `pkgver` (same as Play / Windows).  
Data dir: `~/.local/share/com.khaynjo/`  
Compat: [`DATA_COMPAT.md`](DATA_COMPAT.md). Release index: [`RELEASE_1_0_0.md`](RELEASE_1_0_0.md).

Local install from this workspace (no AUR): [`AUR_LINUX.md`](AUR_LINUX.md).

## Prerequisites

- Arch host with the usual build tools (`base-devel`, `git`, `cargo`, `rust`, `rsync`, `clang`)
- AUR account + SSH key; `aur.archlinux.org` in `known_hosts`
- Public git tag **`v<pkgver>`** on the commit whose `apps/khaynjo/Cargo.toml` `version` is that `pkgver`. `vendor/makepad` is gitignored; the PKGBUILD runs `scripts/setup-makepad.sh` (pinned Makepad commit + patches).

## 1. Check the PKGBUILD

Template: [`../packaging/aur/PKGBUILD`](../packaging/aur/PKGBUILD)

```bash
./apps/khaynjo/scripts/aur-check.sh
```

`pkgver` must match Cargo `version`. `pkgrel` stays `1` for a new product version; bump it only when republishing the same `pkgver` for a packaging-only fix.

## 2. Tag the source

```bash
git tag v1.0.5
git push origin v1.0.5
```

The tag name is `v` plus `pkgver`. The AUR `source=` line clones that tag.

## 3. Local makepkg check

```bash
mkdir -p /tmp/khaynjo-aur && cd /tmp/khaynjo-aur
cp /path/to/kumkaj-workspace/apps/khaynjo/packaging/aur/PKGBUILD .
makepkg -si
```

This compiles. It installs under `/usr/lib/khaynjo/` with `/usr/bin/khaynjo` (a wrapper that cds into that directory, then runs the binary).

## 4. Push to the AUR

```bash
ssh-keyscan aur.archlinux.org >> ~/.ssh/known_hosts   # once per machine

git clone ssh://aur@aur.archlinux.org/khaynjo.git
cd khaynjo
cp /path/to/kumkaj-workspace/apps/khaynjo/packaging/aur/PKGBUILD .
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "khaynjo <pkgver>-<pkgrel>"
git push origin master
```

Claim the name `khaynjo` on the AUR site first if that repo does not exist yet. `replaces=('khaynjo-bin')` drops the old binary package name if it was ever published.

## 5. End-user install

```bash
yay -S khaynjo
# or: paru -S khaynjo
```

The user's machine compiles it. Books stay in `~/.local/share/com.khaynjo/`. Removing the package with pacman does not delete those books.

## Versioning

| Field | Role |
|-------|------|
| `pkgver` | Product version — same as Play `version_name` / Windows. Git tag `v<pkgver>`. |
| `pkgrel` | Packaging-only bump of the same source |

Do not ship a different marketing version on Arch than on Play or Windows for the same release.

## Related

- Local Arch build (no AUR): [`AUR_LINUX.md`](AUR_LINUX.md)
- PKGBUILD: [`../packaging/aur/PKGBUILD`](../packaging/aur/PKGBUILD)
- Check: [`../scripts/aur-check.sh`](../scripts/aur-check.sh)
- Play Store: [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md)
- Windows Store: [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md)
