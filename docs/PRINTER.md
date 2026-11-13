# Printers and Bill & KOT format (Khaynjo)

Two separate things. A **printer** is a real device (and **how** jobs are
sent to it). **Bill & KOT format** is how a bill or a kitchen ticket is
**drawn** (width, type, fields, portrait/landscape). The printer does not
decide the layout. The format does not name the device.

Any found printer may print bills, kitchen tickets, or both. A standard
office queue may print KOTs; a thermal head may print bills. That is the
owner’s choice.

## Printer

Settings → **Printers** is the device list **and** each device’s send method.

- **Find printers** reads the OS list (system queues, then network, USB, and
  Bluetooth) and saves it to this shop’s `printers.json` and the running app.
- **Since 2026-11-13 (owner): the screen is Find + the saved list only** —
  names, nothing else. The per-printer send-method chips and the test-page
  picker below are hidden; a send method saved earlier on a printer still
  applies, new printers follow the Bill / KOT format's How; Test bill / Test
  KOT stay under Bill & KOT format. The rest of this list is the earlier model.
- Under **each** printer name, choose how that device is sent jobs:
  - **OS document (system spooler)** — PNG of the slip (same picture as the
    preview), given to CUPS / Windows / Android print. For system queues that
    print a page.
  - **Raw bytes to device** — the same thermal payload as ESC/POS, written as
    a raw job so the OS does not treat the slip as a document.
  - **ESC/POS thermal commands** — thermal raster/commands. The path that
    matches the on-screen preview for thermal heads.
- Find only discovers devices. It does **not** invent a send method from
  printer kind (Named / USB / Brother / …). Empty per-printer How → the Bill
  or KOT format’s How is used at send time. The owner picks How; a bad pick
  for their hardware is theirs unless the chosen path is implemented wrong.
- **Printers for a test page** + **Print a test page** send only to the
  checked device(s), using **that printer’s** How (or the Bill format How
  when unset) and the **Bill** format’s look.
- **Test KOT** / **Test bill** under Bill & KOT format use the **KOT** or
  **Bill** format look and How fallback for that tab — not always Bill.
- Find does **not** create a kitchen station; add stations under Kitchen →
  Kitchen stations, then attach printers there.

A kitchen station, and the bill, each pick one or more of these printers.
**First that works** stops after the first success. **Print on all** sends
the same job to every printer on that place. Order on the list is the try
order.

Old `printers.json` files may still contain virtual-printer rows. Load
rewrites those route slots to the real printer they pointed at. Empty
`delivery` on a printer follows the place format How (see above).

## Bill & KOT format

Settings → **Bill & KOT format** decides the **look**. The physical paper
stays the printer’s. **Content width** is how wide the drawn slip is. Bill
and KOT each have **their own** millimetre width (chip or typed 40–210)
above the tabs while that tab is open — bill default 210 (page), KOT
default 80 (wide slip). It is not a paper-width lock on the printer.

The on-screen preview uses that tab’s content width with a **proportional
fit** so 56 mm and 210 mm both stay readable — preview does not care which
physical printer will print; it only mirrors the drawn slip. Preview dots
must match print dots (`wrap_cols(mm)` → `paper_dots_for_wrap_cols`).

Portrait / landscape are chosen **separately** on the Bill tab and on the
KOT tab. **How the job is sent** on those tabs is a **fallback** for
printers that still have no `delivery` set; prefer Settings → Printers for
per-device send methods. Either job can go to any printer Find has saved.

### How the job is sent (per printer)

| Choice | What the device receives |
|---|---|
| **OS document** | A picture of the slip (bundled fonts), given to the system spooler / Android print. The slip is placed on a **210 mm-wide page** at `content_mm / 210` of that width, then fitted (CUPS + Android) or drawn at content mm (Windows) — so **150 mm stays narrower than 210 mm**. Never `fit` the bare slip to A4 alone (that made every width look the same). |
| **ESC/POS** | Thermal commands (plain text when the head has the letters, otherwise a raster picture). Network `:9100`, USB, Bluetooth, or a named queue as a raw spool. |
| **Raw bytes** | The same thermal job bytes as ESC/POS, written straight to the device or the raw queue so the OS does not treat them as a document. |

Kitchen-ticket style marks (size, weight, Start / Center / End) are drawn
in that picture for both the OS document and the ESC/POS raster. Do not
strip the marks and then left-align the plain text.

USB / Bluetooth / `:9100` take thermal bytes (ESC/POS or Raw). **OS document
on those direct targets is refused with a clear error** — pick ESC/POS or
Raw, or use a system queue that can print a page. The app does not silently
swap How, and must not silently drop that printer from a multi-printer
route without a status note.

### Bill tab

- Content width (this tab only; not shared with KOT).
- Portrait or landscape.
- Fallback how-the-job-is-sent (prefer per-printer on Settings → Printers).
- Which real printers may print the bill.
- One editor per bill line: show or hide, point size, weight, and
  alignment (Start / Center / End), plus typed text where the line has it
  (address, FSSAI, title, footer, UPI). Lines include shop, address, FSSAI,
  title, **customer** (`Name:`), bill no, time, **cashier**, table/area,
  items, total, footer, UPI. Meta lines can share a row as left/right pairs
  (product default: time with table, cashier with bill no).
- When **Items** is open: **Columns** (Item | Qty | Price | Amount) or
  **Stacked** (each piece on its own line). Per-piece On/Off for name, qty,
  price, amount, note, and the column header row. Shown columns align under
  their headers on the slip.
- When **Total** is open: On/Off for **Total qty**, **Sub total**, and
  **Grand total**.

Copies (1–3) are one queued job, not a pause between each copy. This is
configuration to compose bills in that family (and denser ones) — not a
hardcoded template of one shop's paper.

### KOT tab

- Its own content width, portrait or landscape, and fallback how-the-job-is-sent.
- Which fields show, in what order, and how many copies (1–3). Copies are
  one queued job, not a pause between each copy.
- One editor per field: show or hide, point size (12–48), weight (Light /
  Regular / Bold), and alignment (Start / Center / End, following the
  writing direction). Fields: shop, time, staff, station, table, KOT number,
  area, item and qty, title.
- A kind for each ticket, with its own field styles and chrome: **Original**,
  **Duplicate**, **Modified**, **Cancelled**. Per kind the owner sets:
  - **Kind banner** (editable; product defaults include
    `PARTIALLY SHIFTED/CANCELLED` and `MODIFIED/UPDATED KOT`).
  - **Item prefix** on dish names (defaults `[CANCELLED] ` / `[NEW] `; blank =
    plain name). Qty stays unprefixed.
  - **Strike dish name** and **Strike dish note** are separate (on by default
    for Cancelled name only; note stays readable; qty column stays clear).
    Raster / OS-document jobs draw a real strike; plain preview approximates it.
- Per-item cancel still prints one cancel slip when a sent line is cancelled.
  **Print all cancelled** on the Order ticket prints one cancel-kind slip per
  kitchen listing every cancelled dish still on that open order (additive —
  does not replace per-item cancel).
- Title and footer text. A custom title shows in the preview and on the
  slip, including a cancelled ticket. The preview is a kitchen slip, not a
  money report.
- **Staff** is the person who printed the ticket: name, username, and role.
  The slip does not say Waiter.
- Item and Qty share one row. The qty column is the count (`1`, or
  `1.25 kg` when the dish has a unit). Do not prefix or suffix the dish
  name with `x1` or `x1×` for quantity — kind item prefixes are separate.
- Start, Center, and End are shifts across the content width in dots, not
  spaces padded in front of the line. A rule of dashes is drawn out to that
  same content edge. A wider content width lengthens the rule and moves
  Center and End.

A **reprint** (Duplicate) and **Print all cancelled** (Cancelled) use their
kind's type size and content width, the same as a ticket sent with Send
KOT — every kitchen slip goes through one setup per kind
(`kot_printers_for_kind`), whichever button printed it.

Save KOT format writes that slip. The printed ticket reads the saved slip,
not an unsaved edit still sitting in the form. **Test KOT**, under that
save button, prints the preview on a kitchen printer. **Test bill**, under
Save bill format, prints a sample bill with the header, footer, size,
content width, layout, and the printers’ send methods.

## Lessons — bugs we hit (do not redo)

Ownership model (non-negotiable):

| Owner chooses | Means | Must not |
|---|---|---|
| **Look** | Content width, type, fields, portrait/landscape | Depend on which printer exists |
| **Where** | Which printer(s) on the route / test checks | Change the drawn slip |
| **How** | ESC/POS / Raw / OS document | Be invented from Named vs USB vs Brother |

### What went wrong

1. **Inventing How from printer kind** (Named → OS, USB → ESC/POS) broke
   shops where a CUPS-named thermal (`80Series2`) already worked on ESC/POS.
   Find/add must leave `delivery` empty; empty follows the place format How.
2. **Healing too much** — clearing USB/BT ESC/POS stamps left direct heads
   on the Bill route with empty How → Bill’s default OS document → refused.
   One-shot heal may clear **Named/Default OS stamps only**.
3. **OS `print-scaling=none` + ignored `ppi`** — mid widths (e.g. 150 mm)
   printed huge and cropped; 210 mm looked fine only because it used `fit`.
4. **OS `fit` on the bare slip** — every content width filled A4, so
   150 mm and 210 mm looked identical. Fix: embed slip on a **210 mm page**
   at `content/210`, then fit the *page* (CUPS + Android). Windows sizes
   by content mm in GDI instead.
5. **Test KOT called `print_one_printer` with Bill format** — kitchen sample
   used Bill look/How. `print_one_printer` must use `format_for(kind)`.
6. **Preview dots ≠ print dots** at 112/150 mm — preview used
   `paper_width_dots(mm)` while print used `wrap_cols` →
   `paper_dots_for_wrap_cols`. Keep one path.
7. **Silent skip of Unsupported How** when another printer on the route
   still worked — always `note_problem` for refused dests.
8. **Windows landscape sized by image aspect** — short landscape slips
   (taller than wide after rotate) used the wrong edge. Pass
   `content_along_width = !landscape` explicitly.

### Working approaches (keep)

- ESC/POS Named → `print_system_raw` (`lp -o raw` / Windows raw) — do not
  “improve” this path while fixing OS/Raw.
- Raw = same thermal payload as ESC/POS, raw spool only — not a second
  encoding and never the OS PNG path.
- OS document PNG = same `slip_page_png` as the Settings preview.
- Empty printer How → format How; never invent from target kind.
- Unsupported How → clear error; no silent swap to ESC/POS on USB/BT/:9100.

## Paper for a waiter's device (LAN)

The printers are on the shop's Server. A paired Client (waiter tablet,
kitchen tablet) never prints a KOT, cancel slip, fired course or bill
itself; the Server prints it.

- The Server prints **after** the books commit the write. A cancel that is
  refused, or a write that rolls back, leaves no paper in the kitchen.
- The Server then waits up to 5 s for the printers to answer, and puts what
  did not print in the device's reply (`print_problem`; a reprint or Print
  all cancelled also lists per-kitchen `errors`). The waiter's status line
  reads e.g. `KOT for D3 reached Tandoor · paper: Network printer
  192.168.1.50 did not print: cannot reach it …`, never "reached" alone. A
  slip still printing after the wait says so; the Server's own screen
  still warns when it fails. A retry after a lost reply hears the same.
- A bill with no paper says `Bill not printed — no bill printer …` and that
  the table is on Bill now (printing it again counts as a reprint). It
  never says "Bill printed".
- The Server's own screen never waits: its jobs are queued and a failure
  shows as a warning when the printer gives up.

Where each "no printer" message sends people:

| Paper | Fix it at |
|---|---|
| KOT / cancel slip | Kitchen → Kitchen stations → Printers for this station |
| Bill, shift slip | Settings → Bill & KOT format → Bill → Printers for the bill |

## Moklanjo

Moklanjo prints two different jobs. A **receipt** is a booking receipt, a
period bill, or a run sheet. An **AWB label** is a sticker whose point is
the barcode. They do not share a content width.

Settings → **Printers** is still only real devices: **Find printers**, then
attach those devices to Receipt or AWB label. There is no manual rename and
no Thermal / Portrait / Landscape profile that locks the layout to the
delivery.

Under the Receipt | AWB label choice:

- Content width in millimetres (58, 80, 100, 210, or a typed 40–210). A
  receipt defaults to 80 mm. A label defaults to 100 mm.
- Portrait or landscape, chosen for that job.
- How the job is sent: OS document, raw bytes, or ESC/POS. Those three
  cover a docket and a sticker. A label command language (ZPL, TSPL) is
  not used; the label is a picture of the slip plus Code 128 bars, so a
  normal printer and a thermal head both get the barcode.
- Type size.
- **Test receipt** or **Test label**, under the save button, prints a
  sample of that job with the width, layout, delivery, and size on screen.
