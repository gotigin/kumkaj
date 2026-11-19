# Moklanjo — LAN realtime pilot checklist

Owner-run validation for offline-first Server + Clients. Product code readiness:
[`../../../docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md`](../../../docs/PRODUCTION_STATE_FOR_OFFLINE_USAGE.md).
Owner guide: [`../../../docs/LAN_QUICK_START.md`](../../../docs/LAN_QUICK_START.md).
Ports: TCP **7857**, UDP **7858**. Protocol: `LAN_PROTOCOL` **2** — **same build on every device**.

Moklanjo is a **complete courier OS**. This checklist proves Extra tills work on
that OS; it does not invent franchise-only or solo-only builds.

## Setup

- [ ] Counter (Windows or Linux) **Server Mode** → Settings → Extra tills → **Allow**
- [ ] Firewall: **Allow other devices to connect** if prompted (Windows rule name **Moklanjo LAN**)
- [ ] Optional: **Start with this computer** after a power cut
- [ ] Two+ tablets/phones on the **same SSID** (not guest / client-isolation VLAN)
- [ ] Pair each with the six-digit code (or manual Server IP) → Staff PIN → header **Live**
- [ ] Printers on the Server: Receipt + AWB label routes set

## Instant books

- [ ] Book / settle / Ops status change on counter → second device shows the same day book / Ops without waiting only for the 20 s safety pull
- [ ] Kill Wi‑Fi on one Client → **Offline · N waiting** (if queued); restore → **Sending** → **Live**, no money double
- [ ] Rider path (if used): rider mode write → outbox drains to Server over office Wi‑Fi

## AP / sleep / mixed path

- [ ] Flip AP client isolation briefly: Clients go Offline; off again → Live (or manual IP still works)
- [ ] Phone sleep 2–5 min → wake → Live within a few seconds
- [ ] Mixed protocol: old APK vs new Server → **Update the app**, queue kept
- [ ] Lunch-length soak: no Live↔Offline flap on a quiet desk; no UI freeze on sync

## Pass

**Live** under load, gap recovery via one `/sync`, sole-writer money safe, no silent dupes.
Mark `LAN_PILOT_PASSED` on the website only after this run ([`MOKLANJO_RELEASE.md`](MOKLANJO_RELEASE.md) O6 / W3).

## Doctrine reminders

- One Server books file per install ([`offline-single-server`](../../../.cursor/rules/offline-single-server.mdc)).
- Multi-outlet = separate install or OS user; Branches screen is **export/import day**, not in-book multi-server switch (Add branch stays hidden).
- Shop face (hide Riders, cash-only, Ops depth) = Settings / templates — not a second app.
