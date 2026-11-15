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

**Sizes are literal** (owner, 2026-11-15: "30px means 30px"). A size of
N px is N dots in the printed picture, on every content width. Nothing
scales a size, a margin, a column or a line spacing by the content width,
by 80 mm or by 210 mm — no "virtual" sizes. Sizes are free from 1 to 200 px
(a typo guard only; there is no hidden minimum such as 8 or 12).

**Content width is only the picture's width** — not the paper. 210 mm is a
content width like any other. Narrowing it never changes a size; it only
moves the end edge. What lies past that edge is cut, on paper and in the
preview alike.

**The preview is the print.** It shows the exact picture the printer gets
(`wrap_cols(mm)` → `paper_dots_for_wrap_cols`, same dots), scaled by one
ratio for the whole picture so it is readable: same proportions, same
wraps, same cuts. It never reflows, squeezes or enlarges anything on its
own. If big sizes on a narrow width cut a column, the preview shows that
cut. The preview does not care which printer will print.

**Start and end, not left and right.** Every placement — line alignment,
column order, where columns begin, pair halves — follows the writing
direction of the language: in Arabic, Urdu, Persian or Hebrew, start is the
right edge.

**The owner is responsible for the output.** Printer, How (OS document /
Raw / ESC/POS), widths and sizes are the owner's choices. The app sends
what was chosen; it does not refuse or swap a How from a guess about the
printer, and it does not re-lay out a slip to "rescue" values that do not
fit. Correct values must never come out as rubbish because of the app.

### Item columns

The item table (Bill: Item, Qty, Price, Amount; KOT: Item, Qty) is laid out
in **columns the owner defines** (Khaynjo; Kirano and Tabiyat for the bill,
which is their only slip). A lone shown column still lays out in its column. Each column has:

- **Width** in mm. Columns sit side by side from the start edge, in the
  column order, each exactly its width. Hidden columns take no room.
- **Start space** and **end space** in px, **inside** the width (like a
  border box): text sits between them. The gap between two columns is the
  first column's end space plus the next one's start space.
- **Alignment** (Start / Center / End) of its text, header included.

Text longer than its column word-wraps inside the column (a word wider than
the column breaks inside the word); the row is as tall as its tallest cell.
Columns that run past the content width are cut there: Item 30 + Qty 30 +
Price 20 mm on a 56 mm content width prints Item whole, Qty cut after 26 mm
and no Price — and the preview shows exactly that. Room left after the last
column stays empty. The Bill tab has one set of columns; **each KOT kind**
(Original, Duplicate, Modified, Cancelled) has its own set — no kind is
tied to another's (Copy from copies line styles only, not columns). New formats (and formats saved before columns existed)
start with columns that fill their content width; after that the columns
keep their mm whatever the content width.

**Pair lines** (Date | Table, Cashier | Bill no) are two halves of the
content width, one per field. Each half uses its own field's size, weight
and alignment, and wraps inside its half.

Portrait / landscape are chosen **separately** on the Bill tab and on the
KOT tab. **How the job is sent** on those tabs is a **fallback** for
printers that still have no `delivery` set; prefer Settings → Printers for
per-device send methods. Either job can go to any printer Find has saved.

### How the job is sent (per printer)

| Choice | What the device receives |
|---|---|
| **OS document** | A picture of the slip (bundled fonts), given to the system spooler / Android print. The slip is placed on a **210 mm-wide page** at `content_mm / 210` of that width, then fitted (CUPS + Android) or drawn at content mm (Windows) — so **150 mm stays narrower than 210 mm**. Never `fit` the bare slip to A4 alone (that made every width look the same). |
| **ESC/POS** | Thermal commands (plain text when the head has the letters, otherwise a raster picture). Network `:9100`, USB, Bluetooth, or a named queue as a raw spool. A named queue whose paper is Letter, A4 or wider than 120 mm is refused (it would print pages of symbols). |
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
  items, total, footer, UPI. Meta lines can share a row as pairs, one half
  per field (product default: time with table, cashier with bill no).
- When **Items** is open: **Columns** (Item | Qty | Price | Amount) or
  **Stacked** (each piece on its own line). Per-piece On/Off for name, qty,
  price, amount, note, and the column header row. Each shown column has its
  width (mm), start and end space (px) and alignment (see Item columns);
  headers sit in their columns.
- When **Total** is open: On/Off for **Total qty**, **Sub total**, and
  **Grand total**.

Copies (1–3) are one queued job, not a pause between each copy. This is
configuration to compose bills in that family (and denser ones) — not a
hardcoded template of one shop's paper.

### KOT tab

- Its own content width, portrait or landscape, and fallback how-the-job-is-sent.
- Which fields show, in what order, and how many copies (1–3). Copies are
  one queued job, not a pause between each copy.
- One editor per field: show or hide, size (1–200 px), weight (Light /
  Regular / Bold), and alignment (Start / Center / End, following the
  writing direction). Fields: shop, time, staff, station, table, KOT number,
  area, item and qty, title.
- A kind for each ticket, with its own field styles and chrome: **Original**,
  **Duplicate**, **Modified**, **Cancelled**. Per kind the owner sets:
  - **Kind banner** (editable; product defaults include
    `PARTIALLY SHIFTED/CANCELLED` and `MODIFIED/UPDATED KOT`).
  - **Item prefix** on dish names (defaults `[CANCELLED] ` / `[NEW] `; blank =
    plain name). Qty stays unprefixed.
  - **Item columns** (Item, Qty): width, start and end space, alignment —
    this kind's own (see Item columns).
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
- Item and Qty share one row, in their columns. The qty column is the count (`1`, or
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
9. **A CUPS USB queue holds the printer ~8 s per job** (2026-11-14). CUPS's
   usb backend writes the ticket at once, then waits 7 s for the printer's
   back channel and 1 s more before letting go (kernel log: `usblp0:
   removed` … `usblp0: USB Bidirectional printer` 8.2 s later). Every KOT
   on a CUPS-named USB thermal (`80Series2`) came back "still printing"
   after `PAPER_WAIT`, and the next ticket waited behind it. Fix:
   `print_system_raw` writes raw ESC/POS for a CUPS queue whose device is
   `usb://…?serial=X` straight to that printer's `/dev/usb/lpN`, paced like
   `usb:` (diag `usb_direct`) — when the node is there, free (usblp has one
   writer; CUPS printing = node gone) and CUPS has nothing queued for the
   queue. Otherwise `lp -o raw` as before. No serial in the URI → CUPS (two
   identical printers could not be told apart). Linux only.
10. **The usblp lookup never matched** (2026-11-14). `print_usb` read
    `/sys/class/usb/lpN` (current kernels: `/sys/class/usbmisc/lpN`) and
    walked the *relative* `device` link from the process's working
    directory, so a `usb:VID:PID:serial` printer was always "the first
    /dev/usb/lp*" — wrong with two USB printers. Canonicalize the link and
    try `usbmisc`, then `usb` (`a_usblp_node_is_traced_to_its_usb_device_through_usbmisc`).
11. **Sizes were made virtual — twice** (2026-11-15). `page_em_scale`
    (Cursor, 2026-11-07) drew sizes at width / 576 dots above 80 mm, so
    narrowing a bill 210 → 80 mm shrank every field. The first fix,
    `TYPE_SCALE`, drew every width at the 210 mm factor (2.6 dots per
    size): 80 mm slips printed 2.6× their values, item columns no longer
    fitted and correct settings came out as jumbled, overlapping, very long
    slips. Both are wrong. A size is literal: N px = N dots (see Sizes are
    literal). Do not add any width- or page-based factor again.
12. **The preview drew the print squeezed into half its card** (fixed
    2026-11-15). The slip `Image` and the zero-height row `PortalList`
    sat side by side in the card (`flow: Right` default), two `Fill`
    widths split the card, and the picture was stretched into half of it —
    text looked tall and narrow, not like the paper. The card is
    `flow: Down` now (Khaynjo `KotSlipPreview`, Kirano / Tabiyat
    `SlipPreview`). The preview's size rule did not change: fit the column
    and ~62 % of the window while body type stays ≥ 15 px, else hold that
    size and let the page scroll (owner brief 2026-11-08) — a fixed-zoom
    preview tried the same day was rejected and reverted.
13. **Rejected the same day — do not redo** (2026-11-15). (a) Refusing
    ESC/POS / Raw on a CUPS queue whose paper looked like Letter / A4:
    a guess about one printer; How is the owner's choice. (b) Re-laying out
    item columns that did not fit (shrinking gaps, moving the name above
    the numbers): the app must print the owner's columns as defined and cut
    at the content width, not invent a layout. (c) Diagnosing from the
    development desktop's CUPS logs: the owner tests on other systems; fix
    the generic pipeline, verified by rendering the print picture.

### Working approaches (keep)

- ESC/POS Named → `print_system_raw`: a CUPS socket queue → its host:port
  directly; a CUPS USB queue with a serial → its usblp node directly (free,
  nothing queued); otherwise `lp -o raw` / Windows raw — do not “improve”
  this path while fixing OS/Raw.
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

**No printer saved on this device at all** (all four apps, owner
2026-11-14): nothing says "go to Settings → Printers". The fix comes to
the owner instead:

- A printer checklist (Printers for the bill / this station, Test KOT, test
  page, assign) shows one row, a **Find printers** button.
- A "no printer" warning shows a **Find printers** button on the warning.
- Either opens the **Printers sheet** over the current screen: the same
  Find, Cancel and saved list as Settings → Printers. Done reopens the
  checklist that opened it, now listing the printers just found, ready to
  tick. Code and rules: `docs/DEV.md` § Dropdowns rule 10.

## Moklanjo

Moklanjo prints two different jobs. A **receipt** is a booking receipt, a
period bill, or a run sheet. An **AWB label** is a sticker whose point is
the barcode. They do not share a content width.

Settings → **Printers** is still only real devices: **Find printers**, then
attach those devices to Receipt or AWB label. When a label or run sheet had
no printer, its warning's **Find printers** opens the Printers sheet: Find
there saves what it found, and tapping a printer gives it that job
(receipt or AWB label) — no trip to Settings. There is no manual rename and
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
