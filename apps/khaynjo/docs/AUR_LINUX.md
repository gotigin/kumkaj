# Khaynjo on Arch Linux (local build, no AUR required)

Run or install from this workspace on an Arch host.  
**Publishing** to the AUR is a separate guide: [`PUBLISH_AUR.md`](PUBLISH_AUR.md). The AUR package compiles from the git tag on the user's machine. It is not a prebuilt binary.

Product version: [`../Cargo.toml`](../Cargo.toml) (`version`).  
Data dir: `~/.local/share/com.khaynjo/`  
Compat: [`DATA_COMPAT.md`](DATA_COMPAT.md).

## Dependencies (typical)

```bash
sudo pacman -S --needed base-devel rustup git clang mold \
  libx11 libxcursor libxi libxrandr libglvnd \
  python python-pillow zstd
rustup default stable   # if not already
```

Also need the vendored Makepad tree the workspace expects (`vendor/makepad`
or `MAKEPAD_ROOT` — see root [`docs/DEV.md`](../../../docs/DEV.md)).

## Production package + local install

```bash
cd /path/to/kumkaj-workspace
./apps/khaynjo/scripts/package-linux.sh
# → dist/khaynjo-linux/

./dist/khaynjo-linux/install.sh
# App menu entry + ~/.local/lib/kumkaj-khaynjo/
# Uninstall: app menu "Uninstall Khaynjo" or that tree's uninstall.sh
```

Portable (no install):

```bash
./dist/khaynjo-linux/run.sh
```

Dev build (separate id `com.khaynjo.debug`):

```bash
./apps/khaynjo/scripts/package-linux-dev.sh
./dist/khaynjo-linux-dev/install.sh
```

## Cross-builds from Arch

Play / Windows Store packaging from this host: see  
[`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md) and  
[`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md)  
(and root `docs/DEV.md` for SDK / cross-compile notes).
