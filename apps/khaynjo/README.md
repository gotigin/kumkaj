# Khaynjo

Restaurant operating system under the Kumkaj brand — floor/tables,
ordering, KOT/kitchen, billing, and Khaynjo's own books underneath.
Local-first on Makepad 2; binary `kumkaj-khaynjo`; ids
`com.khaynjo` (prod) / `com.khaynjo.debug` (dev), from
[`env/`](env/). Domain: khaynjo.com. "Khaynjo" is the Memoni word for
food.

Khaynjo is a **separate concrete Kumkaj product** from Kumkaj Courier.
It was created by copying the Courier crate — the proven local store,
LAN server/client, money/timestamp/fiscal code, and Makepad shell —
and re-facing it as a restaurant product. That copy is now Khaynjo's
own code: do not reinvent in Khaynjo what Courier already solved —
**copy from Courier and diverge; never extract into shared code**
(there is no Kumkaj core, vision §1, and a Courier change must never
impact Khaynjo, nor the reverse).

The courier product itself has since been **removed** from this
crate (Oct 2026): no booking desk, AWB, zones, carriers, manifests,
COD, riders, or tracking — only Khaynjo's own copy of the books/billing
code remains, wearing
a restaurant face (Order · Floor · Kitchen · Menu · Books). If you
find a courier word in Khaynjo's UI or strings, it is a bug. New
code here borrows from Courier's *internals* only, never its face.

## Run (live dev)

```bash
cargo khaynjo-dev                      # desktop, --hot --remote
cargo khaynjo-android-dev              # build + install + launch on device
cargo khaynjo-windows-dev              # Windows dev package → dist/khaynjo-windows-dev/
./apps/khaynjo/scripts/windows-prod.sh  # Windows prod package → dist/khaynjo-windows/
./apps/khaynjo/scripts/msi.sh [prod|dev] # MSI (per-user, own upgrade codes) → dist/Khaynjo-<ver>.msi
./apps/khaynjo/scripts/dev.sh          # desktop, without the cargo alias
./apps/khaynjo/scripts/package-android.sh   # tablet APK → dist/khaynjo-android/
./apps/khaynjo/scripts/uninstall-android-dev.sh  # adb uninstall (…-prod.sh for com.khaynjo)
```

Tooling mirrors Courier's naming (`courier-dev` → `khaynjo-dev`,
`courier-android-dev` → `khaynjo-android-dev`, `courier-windows-dev`
→ `khaynjo-windows-dev`; per-app scripts under `scripts/` in each
app). One deliberate difference: Courier's ids are tied to the
kumkaj.com domain (`com.kumkaj.courier`); Khaynjo has its own domain,
so its ids are `com.khaynjo` / `com.khaynjo.debug` — see
[`docs/KHAYNJO.md`](docs/KHAYNJO.md).

Everything that names Khaynjo lives in `apps/khaynjo/scripts/` — its
package scripts, its own `Setup.bat` / `install.ps1` / `uninstall.ps1`,
`desktop-ids.sh` and MSI `License.rtf`. Root `scripts/` is shared
tooling that names no app (docs/DEV.md § Common workspace → Scripts).

## App icon / logo

Source mark: `assets/khaynjo-app-icon.png` (owner-provided). The full
icon set — `resources/icon_*`, `resources/icon.ico`, in-app
`resources/kumkaj-mark*` (WebP/AVIF), and Android `mipmap-*` — is
generated from it the same way Courier's is:

```bash
./apps/khaynjo/scripts/generate-icons.sh
```

Dev scripts export `MAKEPAD_APP_ICON_*` pointing at this app's icons
(the workspace `.cargo/config.toml` defaults are Courier's; Cargo
`[env]` does not override exported vars).

**Theme:** the shell palette is sampled from this logo (deep red
`#9A0E10`, maroon `#69090C`, red-orange `#D64D21`, gold `#F69D2C`,
warm cream `#F9F4F0`) — see `docs/DESIGN.md` § Color tokens (Khaynjo)
and the theme block in `src/main.rs`.

## Docs for this app

- [`docs/KHAYNJO.md`](docs/KHAYNJO.md) — product brief and build plan
- Global doctrine (vision, business-books concepts checklist, DEV/DESIGN): [`../../docs/`](../../docs/README.md)
- Sibling product it was copied from: [`../courier/`](../courier/)
