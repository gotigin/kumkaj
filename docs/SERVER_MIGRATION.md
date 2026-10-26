# Server Migration — Move the Books to Another Device

> When the Server device is replaced — computer to computer, tablet to
> computer, or any other mix of Windows, Linux and Android. The server id
> lives **in the books**, so a backup restored on the new device is the
> same server: paired devices find it by that id and keep working — no
> pairing again. Expect a few minutes without the Server; devices keep
> taking orders offline meanwhile and send them once it is back.

---

## Steps

1. **Back up on the old Server** and get the file off it. Every backup is
   read back as it is written (a full disk or a failing USB stick is
   reported, never left looking like a backup), so there is no separate
   "verify" step. Keep the original on the old device.
   - Computer: **Settings → Backup → Create backup**. The screen shows
     where the file went ("Backup: …"); copy it to a USB drive.
   - Android: **Settings → Backup → Save a backup copy…** and pick where it
     goes — Downloads, a USB drive, or a cloud folder the new device can
     reach.
2. **Stop the old Server** — **Settings → Extra tills → Stop extra tills**,
   then close the app. Only one device may serve these books: two copies
   serving at once would split the day's orders between them.
3. **Install on the new device**, choose **Server Mode** and let it create
   a server.
4. **Restore** — **Settings → Backup → Restore backup** and pick the file
   (on Android the system's file picker opens: Downloads, USB drive or
   cloud folder). A file that is damaged or is not this app's books is
   refused before anything is replaced; the new device's own books are
   saved first, so a restore can be undone.
5. **Serve** — **Settings → Extra tills → Allow extra tills**. The pairing
   code is the old one. If **Allow other devices to connect** shows, tap
   it (one admin prompt). On a computer, tap **Start with this computer**
   again.
6. **Printers** — **Settings → Printers on this device**: the printers are
   the device's own, not in the books. Set them, **Save printers**,
   **Print a test page**.
7. **Check** — a device opens a table and sends a KOT; it prints at the new
   Server.

---

## What moves with the backup, what does not

| Moves (it is in the books) | Set again on the new device |
|---|---|
| Sales, KOTs, purchases, expenses, day closes | **Allow other devices to connect** (firewall, computers only) |
| Menu, prices, stock, recipes | **Start with this computer** (computers only) |
| Staff, PIN hashes, rights | **Printers on this device** |
| Customers and khata, suppliers | **Also copy backups to a folder…** (computers) |
| Paired devices, pairing code, server id | Window size and position |
| Language, currency, country pack, bill format | |

---

## Troubleshooting

| Problem | What to check |
|---|---|
| "… is not a backup of these books" | The file is another app's books or not a backup at all. Pick the file **Create backup** / **Save a backup copy…** made on the old Server, or a daily `auto-…sqlite3` from its backups folder. |
| "… is damaged — pick another backup" | The copy was cut short. Use the original or an older day's backup. |
| Devices stay **Offline** | Same Wi‑Fi? Is **Allow extra tills** on? On a computer, **Allow other devices to connect** if it shows. On Android, is the app open? Devices look for the server again about once a minute while it is silent. |
| A device says **Pair this device again** | It was paired after the backup was made, so the restored books do not know it. Pair it with the code. |

There is no zero-downtime migration and none is planned: one Server is the
only writer by design. Plan the move outside service hours.
