# Publish Khaynjo on the Arch User Repository (AUR)

Package name (suggested): **`khaynjo-bin`** (binary of the official Linux build)  
Product version: Cargo `version` → AUR `pkgver` (same as Play / Windows).  
Data dir: `~/.local/share/com.khaynjo/`  
Compat: [`DATA_COMPAT.md`](DATA_COMPAT.md). Release index: [`RELEASE_1_0_0.md`](RELEASE_1_0_0.md).

Local install **without** the AUR (workspace `package-linux.sh` / `install.sh`): [`AUR_LINUX.md`](AUR_LINUX.md) § A.

## Prerequisites

- Arch (or Arch-based) host to build the Linux package
- AUR account + SSH key on that account; `aur.archlinux.org` in `known_hosts`
- A **public HTTPS** place to host the release tarball (AUR build machines download without login)
- Workspace deps as in [`AUR_LINUX.md`](AUR_LINUX.md) (Rust, X11/GL libs, Python/Pillow, zstd, Makepad tree)

## 1. Build the Linux tree and tarball

```bash
cd /path/to/kumkaj-workspace
./apps/khaynjo/scripts/package-linux.sh
./apps/khaynjo/scripts/aur-release-tarball.sh
```

Artifacts:

| File | Role |
|------|------|
| `dist/khaynjo-linux/` | Installed layout (bin, resources, helpers) |
| `dist/khaynjo-linux-<ver>-x86_64.tar.zst` | AUR `source` |
| `dist/khaynjo-linux-<ver>-x86_64.tar.zst.sha256` | Hash for `sha256sums` |

## 2. Host the tarball

Upload the `.tar.zst` to a stable URL (GitHub/GitLab release asset, CDN, project downloads).  
Note the final HTTPS URL and the SHA-256 from the `.sha256` file.

## 3. Fill the PKGBUILD

Template: [`../packaging/aur/PKGBUILD`](../packaging/aur/PKGBUILD)

- `# Maintainer:` — your AUR account / contact
- `pkgver` — must match Cargo `version`
- `pkgrel` — `1` for a new product version; bump only for packaging-only republishes of the same `pkgver`
- `source=(...)` — public URL of the tarball
- `sha256sums=(...)` — real hash (do not leave `SKIP` on the AUR)

## 4. Local makepkg check (recommended)

```bash
mkdir -p /tmp/khaynjo-aur && cd /tmp/khaynjo-aur
cp /path/to/kumkaj-workspace/apps/khaynjo/packaging/aur/PKGBUILD .
# edit source= and sha256sums=
makepkg -si
```

Confirms install under `/usr/lib/khaynjo/` with `/usr/bin/khaynjo` and `/usr/bin/kumkaj-khaynjo`.

## 5. Push to the AUR

```bash
# Once per machine, if needed:
ssh-keyscan aur.archlinux.org >> ~/.ssh/known_hosts

git clone ssh://aur@aur.archlinux.org/khaynjo-bin.git
cd khaynjo-bin
cp /path/to/kumkaj-workspace/apps/khaynjo/packaging/aur/PKGBUILD .
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "khaynjo-bin <pkgver>-<pkgrel>"
git push origin master
```

Claim a new package name via the AUR website / docs if the empty repo does not exist yet.

## 6. End-user install

```bash
yay -S khaynjo-bin
# or: paru -S khaynjo-bin
```

## Versioning

| Field | Role |
|-------|------|
| `pkgver` | Product version — same as Play `version_name` / Windows |
| `pkgrel` | Packaging-only bump |

Do not ship a different marketing version on Arch than on Play or Windows for the same release.

## Related

- Local Arch build (no AUR): [`AUR_LINUX.md`](AUR_LINUX.md)
- Scripts: [`../scripts/package-linux.sh`](../scripts/package-linux.sh), [`../scripts/aur-release-tarball.sh`](../scripts/aur-release-tarball.sh)
- Play Store: [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md)
- Windows Store: [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md)
