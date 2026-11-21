# Kirano — design tokens

Sampled from `assets/kirano-app-icon.png` (orange cart + green leaf).

## Color tokens

| Role | Hex | Use |
|---|---|---|
| Ink | `#031303` | Primary text |
| Leaf | `#1C460E` | Secondary / help |
| Accent | `#F96303` | CTAs, status, focus |
| Warm ground | `#F9F4F0` | Panels (light) |
| Cart orange | `#E85D04` | Brand accent (from mark) |
| Leaf green | `#2D6A4F` | Brand secondary |

Shell theme lives in `src/main.rs` / `ui_script.rs`. Prefer these over
any restaurant sibling palette.

## Chrome

Counter shop: **Books · Sell · Buy · Items · …**. Do not show Floor /
Kitchen / KOT as product peers.

## Always responsive

Workspace law — not optional for Kirano: narrow fit **and** graceful
growth on wide screens. Fit chip toolbars unlock to one row when they
fit; forever-phone stacks with empty desk space are a bug. See
[`docs/DESIGN.md`](../../../docs/DESIGN.md) § Always responsive ·
[`.cursor/rules/responsive-graceful-wide.mdc`](../../../.cursor/rules/responsive-graceful-wide.mdc).
