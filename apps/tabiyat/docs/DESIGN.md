# Tabiyat — design tokens

Sampled from `assets/tabiyat-app-icon.png` (teal / lime chemist mark).

## Color tokens

| Role | Hex | Use |
|---|---|---|
| Ink | `#0B1F1A` | Primary text |
| Teal | `#0D9488` | Brand / CTAs |
| Lime | `#84CC16` | Accents / alerts soft |
| Warn | `#CD380E` | Expiry / Rx reminders |
| Ground | `#F0FDFA` | Light panels |

Shell theme lives in `src/main.rs` / `ui_script.rs`. Prefer these over
any restaurant sibling palette.

## Chrome

Pharmacy counter: **Books · Sell · Buy · Medicines · …**. Do not show
Floor / Kitchen / KOT as product peers.

## Always responsive

Workspace law — not optional for Tabiyat: narrow fit **and** graceful
growth on wide screens. Fit chip toolbars unlock to one row when they
fit; forever-phone stacks with empty desk space are a bug. See
[`docs/DESIGN.md`](../../../docs/DESIGN.md) § Always responsive ·
[`.cursor/rules/responsive-graceful-wide.mdc`](../../../.cursor/rules/responsive-graceful-wide.mdc).
