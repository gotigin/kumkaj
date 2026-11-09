# Khaynjo 1.0.0 — release index

**Product version (all channels):** `1.0.0` from [`Cargo.toml`](../Cargo.toml)  
(`version` + Android `version_name`). One marketing number everywhere; platform-only integers (Play `version_code`, AUR `pkgrel`, MSIX fourth digit) may differ but must move forward.

Books contract: [`DATA_COMPAT.md`](DATA_COMPAT.md).

## Publish guides (one doc per channel)

| Channel | Doc |
|---------|-----|
| Google Play | [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md) |
| Microsoft Store (MSIX) | [`PUBLISH_WINDOWS_STORE.md`](PUBLISH_WINDOWS_STORE.md) |
| Arch AUR | [`PUBLISH_AUR.md`](PUBLISH_AUR.md) |

Local Arch install without publishing: [`AUR_LINUX.md`](AUR_LINUX.md).

## Shared checklist before any upload

- [ ] `cargo test -p kumkaj-khaynjo` green
- [ ] Smoke: Server + Client LAN, sale, KOT, day totals, backup/restore, Clear history / emergency PIN
- [ ] Play: bump `version_code` in [`../Cargo.toml`](../Cargo.toml) to **> last code on Play** (+1 thereafter; no `YYYYMMDDHH` auto — see [`PUBLISH_PLAY_STORE.md`](PUBLISH_PLAY_STORE.md))
- [ ] Play: upload `native-debug-symbols.zip` + `mapping.txt` with the AAB (same folder; clears both Console crash warnings)
- [ ] Play: `$HOME/khaynjo-release.jks` + `$HOME/khaynjo-release.jks.password`
- [ ] Windows: Partner Center Publisher CN + signing `.pfx`
- [ ] AUR: public HTTPS host for the Linux tarball

## After all three are live

- [ ] Clean install on each channel; create books; upgrade once; books survive
- [ ] Emergency PIN / Clear history still zero trading history and keep menu
- [ ] Tag the git release when artifacts match this Cargo version
- [ ] Next release: bump Cargo product `version` / `version_name` once for all platforms; bump Android `version_code` +1 for each new Play AAB — never divergent marketing versions
