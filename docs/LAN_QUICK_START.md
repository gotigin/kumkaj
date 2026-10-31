# LAN Quick Start — Extra Devices on the Shop's Wi‑Fi

> One device holds the books (**Server Mode**). Other devices on the same
> Wi‑Fi join it (**Client Mode**) and take orders, send KOTs and bill.
> Everything lives on the Server; no internet, no cloud.
> Applies to Khaynjo (restaurants, TCP 7847 / UDP 7848) and Moklanjo
> (TCP 7857 / UDP 7858). Labels below are the app's own English wording.

**Any supported device can be the Server or a client:** a Windows 10/11
computer, a Linux computer, or an Android tablet or phone. Mix them as the
shop likes — an Android tablet can serve two Windows tills, a Linux
computer can serve Android phones. (macOS and iOS are not released yet.)
What differs between them is listed in §5.

Pick as the Server the device that stays in the shop, stays on all day and
is plugged in. It prints the kitchen tickets, so it is best next to (or on
the same network as) the kitchen printer.

---

## 1. The Server

1. Install the app and open it. At **How will this device be used?** tap
   **Server Mode**, then create (or **Open**) the shop's server. The first
   start asks you to choose the Owner PIN — there is no default PIN.
2. **Settings → Extra tills → Allow extra tills.** The screen shows a
   6‑digit **pairing code** and this device's Wi‑Fi address
   (e.g. `192.168.1.10`).
3. If **Allow other devices to connect** appears, tap it once and accept
   the admin prompt: this device's firewall is blocking the others. It
   opens only the app's two ports. The button does not appear where nothing
   blocks them (Android, or a computer with no firewall on).
4. On a computer, tap **Start with this computer** so the server is back by
   itself after a power cut or restart (see §5 — the computer must sign in
   by itself for that). On Android, keep the app open on screen, plugged in.
   While it serves, the app keeps the device from sleeping.
5. **Settings → Printers on this device:** leave blank for the device's
   default printer, pick one by name, or type a network receipt printer's
   address (e.g. `192.168.1.50`). Tap **Save printers**, then
   **Print a test page**.

The pairing code stays the same until someone taps **New pairing code**;
devices already paired keep working after a new code.

---

## 2. Each extra device

1. Install the app. At launch tap **Client Mode**.
2. Tap **Find the shop's server** — servers on this Wi‑Fi are listed by
   restaurant (or business) name and the first one's address is filled in.
   Nothing found? Type the Server's address from step 1.2 into
   **Server IP (e.g. 192.168.1.10)**.
3. Type the **Pairing code** and tap **Pair now**.
4. Unlock with your own PIN. Waiters see Take payment refused if their
   rights say so — the Server checks every PIN and right.
5. Check the device's date and time. If it is more than 5 minutes off the
   Server, the header says so — set the right time on the device.

---

## 3. During service

- Orders, KOTs and payments go to the Server at once. KOTs and
  **Print bill** from any device print at the Server, on its printers.
- **Wi‑Fi drops:** the device keeps taking orders. KOTs and payments wait on
  the device (they survive an app restart) and go in order when the Server
  answers again. The header says **Offline · N waiting**, and how old the
  screen is.
- **Settings → Connection to the shop** on a device lists what is waiting,
  how many tries each had and the last answer, plus anything the Server
  would not accept (e.g. a table settled at the counter meanwhile) — never
  dropped silently. **Checked — clear this list** clears it once read.
- A print that fails (printer off, out of paper, wrong address) is said on
  the screen of the device that printed; the bill itself is safe.

---

## 4. What happens when…

| Situation | What happens |
|---|---|
| Wi‑Fi goes down | Devices keep working; waiting writes go when it is back. |
| Server restarts or gets a new address | Devices look for it by its **server id** (never by name) and move to the new address — no pairing again. |
| A device loses power | Its waiting KOTs/payments are on its own storage and go when it is back. No pairing again. |
| Owner taps **Stop extra tills** | The Server stops listening; devices show Offline and keep their waiting writes until tills are allowed again. |
| Owner taps **Forget** on a device | That device is refused at once and says **Pair this device again**; its waiting writes are kept. |
| Backups | The Server backs up once a day (last 14 kept, on the Server itself). Keep a copy off the device too — see §5. After 7 days without one, Settings warns. |

---

## 5. Per device type

| | Windows 10/11 | Linux | Android |
|---|---|---|---|
| Server Mode | Yes | Yes | Yes |
| Client Mode | Yes | Yes | Yes |
| Firewall | **Allow other devices to connect** adds a rule for the shop's network (one admin prompt) | Same button, when firewalld or ufw is on (one admin password prompt); nothing to do otherwise | Nothing to do |
| After a power cut | **Start with this computer** — starts when Windows signs in, so turn on automatic sign-in for the shop's account | **Start with this computer** — starts at desktop login, so turn on automatic login | Not available: open the app again; keep the tablet plugged in |
| Stays awake while serving | Yes | Yes | Screen stays on while the app is open; if Android closes the app, devices wait (Offline) until it is opened again |
| Printing | Settings → **Find printers** (system list + Wi‑Fi scan for port 9100), then add to the pool and ordered routes per feature (bill / each kitchen KOT, or receipt / AWB label) — first tried first; empty = don't print | Same | **Find printers** scans this Wi‑Fi for network receipt printers (port 9100); only those addresses print — without a printable route, KOTs stay off this device and a bill opens Android's Share sheet |
| Second backup copy | **Also copy backups to a folder…** (e.g. a USB drive) — copied every day | Same | **Save a backup copy…** — pick Downloads, a USB drive or a cloud folder; do it at least weekly |

**Network receipt printers** (an address like `192.168.1.50`) take the
ticket straight over the network (port 9100, ESC/POS, 58 or 80 mm paper).
Dish names in Gujarati, Hindi, Tamil, Bengali, Urdu, Arabic, Thai and the
other Indian scripts print as written — the app draws them itself.
Chinese, Japanese and Korean do not (they print as "?"): for those, use a
printer installed on a Windows or Linux computer.

---

## 6. Guest and kitchen screens (Khaynjo)

Any phone, tablet or TV browser **on the shop's Wi‑Fi** can open these
pages from the Server — type the Server's address from step 1.2, its
port `7847`, then the page, e.g. `http://192.168.1.10:7847/display`.
They need no pairing; guests on mobile data cannot reach them.

| Page | Shows |
|---|---|
| `/display` (or `/display?table=D3`) | the order being billed and its UPI QR — a second screen facing the guest |
| `/tokens` | takeaway tokens that are ready |
| `/expo` | kitchen tickets marked Ready, for the pass |
| `/kiosk` | self-order menu; orders land as a Parcel table's pending order |
| `/t/D3` | table D3's QR menu: dishes guests add show on the table's order **before** anyone sends a KOT — staff check and send |
| `/feedback` | a one-tap rating form |

Only dishes on the menu and not sold out can be added from these pages,
never raw materials, and at most 50 at a time.

## 7. Troubleshooting

| Problem | What to check |
|---|---|
| **Find the shop's server** finds nothing | Same Wi‑Fi on both? Guest networks and "AP isolation" block devices from each other. Type the address instead. |
| Devices time out on a computer Server | **Settings → Allow other devices to connect** on the Server (one admin prompt). Another security program may also need the app's ports opened. |
| Devices time out on an Android Server | Is the app still open on the Server's screen? Is it on the same Wi‑Fi (not mobile data)? |
| "Wrong pairing code" | Read the code on the Server again. After 5 wrong tries that device's address must wait 5 minutes. |
| **Pair this device again** | The device was forgotten, or the Server's books were replaced by a backup older than the pairing. Pair with the current code. |
| KOTs do not print | On the Server: **Settings → Printers on this device → Print a test page**. A network printer must be on the same Wi‑Fi and switched on. |
| "This device's clock is … off" | Set the date and time on that device (automatic time is best). |
| "Update the app — this device and the shop's server have different versions" | Install the same version of the app on the Server and every device. Waiting KOTs and payments stay on the device and go once both match. |
| Waiting writes say "the shop's server said: database is locked" (or "disk is full") | The Server was busy or its disk is full. Nothing is lost: the device sends them again by itself. A full disk: free space on the Server (Settings warns when it is low). |

Moving the books to another device: [`SERVER_MIGRATION.md`](SERVER_MIGRATION.md).
