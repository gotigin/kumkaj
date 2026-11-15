# Printers and Bill & KOT format (Khaynjo)

Two separate things. A **printer** is a real device (and **how** jobs are
sent to it). **Bill & KOT format** is how a bill or a kitchen ticket is
**drawn** (width, type, fields, portrait/landscape). The printer does not
decide the layout. The format does not name the device.

Any found printer may print bills, kitchen tickets, or both. A standard
office queue may print KOTs; a thermal head may print bills. That is the
owner’s choice.

## The owner's rules (his words — read before touching printing)

These were explained in chat many times; they are the requirements.

1. **Never concrete to one setup.** "Don't make it concrete to my printers."
   "This is commercial level public app — stop forcing people and users of
   it to have my personal setup to work." No path, guard, name, serial or
   behaviour may come from the owner's machine or printers.
2. **Sizes are real.** "All the sizes are original, they aren't virtual.
   30px means 30px." A px is the px every other app means (18 px is ordinary
   reading text, not micro), the same at every content width.
3. **The preview is the exact look, properly scaled.** "The preview shown
   actually scales so that it is readable and understandable … but that
   doesn't mean that if huge fonts are selected and the width selected is
   lesser, then it shouldn't show the actual crop happening." "The preview
   scales is not an issue but the issue is fooling the user by wrongly
   showing it instead of using proper exact scale ratio to make it readable."
4. **OS document, Raw and ESC/POS are the standard ways, nothing on top.**
   "Don't make OS document, Raw, ESC artificial — they should be the
   standard approached things", not the app's own configs over them.
   (Raw was later removed — it was ESC/POS under a second name; lesson 19.)
5. **The user is responsible for values; the app is not.** A bad value is
   the owner's; correct values must never become rubbish because of the app.
6. **Columns** (owner, 2026-11-15): each column has its own width (mm);
   overflow word-wraps inside it; 210 mm is the content width, not the
   paper; "if column A has width of 30mm, column B has width of 30mm, and
   column C has the width of 20mm and the content width is 56mm then
   obviously column B itself will be cropped … and the preview should
   reflect that"; each column has its own start and end space inside its
   width ("2 px on its right and 2 px on its left … the space between them
   would be 4px"); left/right are start/end and follow LTR / RTL; each KOT
   kind has its own columns ("shouldn't be restricted by other kind").
7. **The small request that started the size work** (2026-11-14): "Why the
   width decrease causes the font size and everything else to be
   decreased?" — a size must not depend on the content width. Nothing more.
8. **The format form is smart, once, at every width** (2026-11-15): "So many
   repeated things could be avoided, so much top to bottom scroll could be
   avoided if only it were done smartly without breaking any single
   functionality." "Shop show 18 Regular Center … the entire input could go
   in that one row." "Don't just limit yourself to what concrete issue I
   mention, making me explain each and everything on the screen." "Never
   ever do it for single size screen — we are supporting all the sizes
   screens with responsive approach." (Lesson 20.)

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

**Sizes** (rule 2). A size of N px is N standard pixels — 1/96 inch each,
what every app means by px. The picture has 576 dots per 80 mm (7.2 per
mm), so 1 px is `PX_DOTS` ≈ 1.905 dots (`receipt_raster::px_dots`): type,
line spacing and column start/end spaces. The same size prints the same on
58, 80 or 210 mm, in the preview and on paper; nothing scales it by the
content width. Sizes are free from 1 to 200 px (a typo guard only).

**Content width is only the picture's width** — not the paper. 210 mm is a
content width like any other. Narrowing it never changes a size; it only
moves the end edge. What lies past that edge is cut, on paper and in the
preview alike.

**The preview is the print** (rule 3). It shows the exact picture the
printer gets (`wrap_cols(mm)` → `paper_dots_for_wrap_cols`, same dots) with
a **proportional fit** so 56 mm and 210 mm both stay readable: one ratio for
the whole picture — fit the column (and ~62 % of the window) while body
type stays ≥ 15 px on screen, else hold that size and let the page scroll
(owner brief 2026-11-08; a fixed-zoom preview was rejected). Same
proportions, same wraps, same cuts: if big sizes on a narrow width cut a
column, the preview shows that cut. It never reflows, squeezes or enlarges
anything on its own, and does not care which printer will print.

**Start and end, not left and right.** Every placement — line alignment,
column order, where columns begin, pair halves — follows the writing
direction of the language: in Arabic, Urdu, Persian or Hebrew, start is the
right edge.

**The owner is responsible for the output.** Printer, How (OS document /
ESC/POS), widths and sizes are the owner's choices. The app sends
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
| **OS document** | A picture of the slip (the preview's picture, bundled fonts), given to the system print queue. CUPS: the slip on a **210 mm-wide page** at `content_mm / 210` of its width, fitted into the printer's own printable area, portrait (`print-scaling=fit`, `ppi`, `orientation-requested=3`) — upright, inside the margins, the same text size at every content width, no paper named. Windows: drawn at its content width in mm. Android: the A4 `PrintManager` page. |
| **ESC/POS** | The slip picture (the same picture as the preview) in standard ESC/POS raster: `ESC @`, `GS v 0` bands, feed, cut (`escpos_job_format`). The whole job goes to the picked destination in one go: network `host:port` (connect, send, close), USB, Bluetooth, or the picked system queue as one raw job (`lp -o raw` / `WritePrinter`). |

Kitchen-ticket style marks (size, weight, Start / Center / End) are drawn
in that picture for both the OS document and the ESC/POS raster. Do not
strip the marks and then left-align the plain text.

USB / Bluetooth / `:9100` take ESC/POS. **OS document on those direct
targets is refused with a clear error** — pick ESC/POS, or use a system
queue that can print a page. The app does not silently
swap How, and must not silently drop that printer from a multi-printer
route without a status note.

### How the Bill and KOT tabs are laid out (2026-11-15)

Both tabs use one pattern, so nothing is set in two places and nothing needs
a scroll to the bottom and back (Kirano and Tabiyat's Bill & Slip format uses
the same Bill form):

- **Paper card** first: portrait or landscape, how the job is sent, the
  printers (Bill), and **Copies**.
- **One list of lines in print order.** A closed line reads
  `name · show · size · weight · align`. Tapping it opens it **where it
  sits**: ↑ / ↓ (only for lines that can move), Show / Hide, Font size (px),
  weight and alignment, then that line's own extras right under it. There is
  no second "Print order" list and no separate field picker.
- **Preview** beside the form when the form card still keeps about 640 px
  (wide screens); the preview then moves down to sit next to the open line.
  On narrower screens the preview follows the form.
- **Every width** (`layout_slip_editors`): each group of controls goes onto
  one row when its card is wide enough and stacks when it is not — the
  open line in one row, or name + size over weight + align, or all
  stacked; switches two per row or one; a column's header box beside its
  three numbers or above them (the three numbers always share a row).

### Bill tab

- Content width (this tab only; not shared with KOT).
- Portrait or landscape.
- Fallback how-the-job-is-sent (prefer per-printer on Settings → Printers).
- Which real printers may print the bill, and with several of them **First
  that works** or **Print on all** (the same choice as a kitchen station,
  shown above Printers for the bill).
- The list of bill lines; the open line has show or hide, size (px),
  weight, and alignment (Start / Center / End), plus typed text where the
  line has it (address, FSSAI, title, footer, UPI). Lines include shop, address, FSSAI,
  title, **customer** (`Name:`), bill no, time, **cashier**, table/area,
  items, total, footer, UPI. Meta lines can share a row as pairs, one half
  per field (product default: time with table, cashier with bill no), or be
  **Alone** on their own row — with every line Alone there are no pairs.
  A closed line that shares a row names its partner (`Bill no + Cashier`).
- When **Items** is open: **Columns** or **Stacked**, Column headers,
  Column rules and Dish note switches, then **one row per column** (Dish
  name, Qty, Price, Amount) in print order: ↑ / ↓, On / Off, its header
  text (while headers are on), width (mm), start and end space (px) and
  alignment (see Item columns).
- When **Total** is open: one row each for **Total qty**, **Sub total** and
  **Grand total** — On / Off and its caption; the Grand total row also holds
  the caption a **paid** bill prints instead.

Copies (1–3) are one queued job, not a pause between each copy. This is
configuration to compose bills in that family (and denser ones) — not a
hardcoded template of one shop's paper.

### KOT tab

- Its own content width, portrait or landscape, fallback
  how-the-job-is-sent, and copies (1–3). Copies are one queued job, not a
  pause between each copy.
- The kind being styled (below), **Copy from** another kind, then the list
  of lines in print order: **Title** first (the kind banner prints above
  it), the seven info lines (time, KOT number, area, table, shop, staff,
  station — ↑ / ↓ move these), **Item / Qty** last. The open line has show
  or hide, size (1–200 px), weight (Light / Regular / Bold), and alignment
  (Start / Center / End, following the writing direction).
- **Title** open: the title text and this kind's banner.
- **Item / Qty** open (it always prints, so no Show / Hide): Column headers,
  Strike dish name, Strike dish note, Item prefix, one row per column (Dish
  name, Qty: header text, width, start and end space, alignment) and the
  kitchen note.
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
| **How** | ESC/POS / OS document | Be invented from Named vs USB vs Brother |

### What went wrong

1. **Inventing How from printer kind** (Named → OS, USB → ESC/POS) broke
   shops where a CUPS-named thermal already worked on ESC/POS.
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
9. **Bypassing the queue for a USB printer — rejected 2026-11-15, do not
   redo.** To dodge CUPS holding a USB thermal ~8 s per job, `print_system_raw`
   wrote ESC/POS for any CUPS queue whose device was a local USB printer
   straight to `/dev/usb/lpN`, in paced pieces (one per picture band, a
   pause before each). That made one machine's printer the rule for every
   shop: a standard printer on a USB queue got each piece as its own job,
   and a single bill printed page after page. The owner: this is a public
   app — never make it depend on one setup. Removed in all four apps (with
   its tests, which carried that printer's name and serial): a named queue
   gets the whole job through `lp -o raw` in one piece; the queue the owner
   picked is the way the job goes.
10. **The usblp lookup never matched** (2026-11-14). `print_usb` read
    `/sys/class/usb/lpN` (current kernels: `/sys/class/usbmisc/lpN`) and
    walked the *relative* `device` link from the process's working
    directory, so a `usb:VID:PID:serial` printer was always "the first
    /dev/usb/lp*" — wrong with two USB printers. Canonicalize the link and
    try `usbmisc`, then `usb` (`a_usblp_node_is_traced_to_its_usb_device_through_usbmisc`).
11. **Sizes went wrong three ways** (2026-11-07 … 11-15). `page_em_scale`
    (Cursor, 2026-11-07) drew sizes at width / 576 dots above 80 mm, so
    narrowing a bill 210 → 80 mm shrank every field (the owner's small
    request). `TYPE_SCALE` then drew every width at the 210 mm factor (80 mm
    slips 2.6× their values, long jumbled slips). Then a size was read as
    one printer dot — 18 px came out about 2.5 mm, micro text on paper and
    in the preview ("Who told you 18px is micro text?"). A size is standard
    px at `PX_DOTS` dots each, on every width. Do not add a width- or
    page-based factor, and do not read px as dots.
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

14. **Layers over the standard Hows — removed 2026-11-15, do not redo**
    (rule 4). The owner: "they should be the standard approached things and
    you kept on [putting] your own configs over them". Removed in all four
    apps: a CUPS socket queue bypassed to talk TCP directly; ESC/POS cut into
    pieces with pauses between them (Rust, PowerShell and the Android
    bridge); network connect retries, a DLE EOT status query after the job
    and a "drain" fallback; watching an `lp` job ~3 s and cancelling it; a
    512 KB "too large" refusal; guessing plain text vs picture from the
    text; writing to the first `/dev/usb/lp*` (or any Bluetooth serial port)
    when the picked printer was not found — that is another printer.
15. **Tests printed on the owner's printers** (2026-11-15). The app's own
    tests (`AppState::boot()`) opened this machine's books and
    `printers.json`, and the library's capture-in-tests does not reach them
    (they link the library built normally), so tickets printed on every
    run. `boot()` under `cfg!(test)` now never opens this machine's data and
    forces `PrintSink::Capture`; guard test
    `app_tests_never_reach_this_machines_printers` (all four apps).
16. **The bill's First that works / Print on all** (2026-11-15). The setting
    existed with no control; it is shown on the Bill tab now (all three
    restaurant / shop apps). With First that works, a bill with two printers
    prints on the first that works — the documented behaviour, not a fault
    (the owner checked it the same day). The app never switches it itself.
17. **A true-size PDF for the OS document — rejected the same day**
    (2026-11-15, do not redo). The slip went to CUPS as a PDF page exactly
    its size. CUPS turned every page wider than tall (a short portrait bill
    printed landscape; `orientation-requested=3` did not stop it), and with
    `print-scaling=none` every printer cut the top and bottom (above Name,
    and Thanks) — the page has no printable margin. The previous image path
    is back: the slip on the 210 mm page, fitted into each printer's
    printable area, portrait. Checked with `cupsfilter` (no printing):
    upright, inside the margins, same text size at 80 and 210 mm.
18. **Alone did not stay alone** (2026-11-15). Every meta line can share a
    row (Date | Table, Cashier | Bill no) or be Alone, but clearing the last
    pair refilled both product pairs, so Cashier could not get its own row.
    An empty list now means no pairs; a save from before pairs existed still
    gets the product pairs (`a_line_set_alone_stays_on_its_own_row`).
19. **"Raw bytes to device" removed** (2026-11-15). It sent exactly the
    ESC/POS bytes. In practice *raw* is how a job is sent (no driver or
    filter changes it — CUPS `-o raw`, the Windows RAW data type) and
    *ESC/POS* is what the bytes are (the receipt-printer language); an
    ESC/POS job is normally sent raw, which is what the ESC/POS choice does.
    A printer saved as Raw prints ESC/POS (`normalize_delivery`).

20. **The Bill & KOT format form made the owner scroll and explain it**
    (2026-11-15). Every line had its controls in one shared block far below
    the list, a second hidden "Print order" list with Move up / Move down,
    a column picker that edited one column at a time, labels repeated over
    their own boxes, and rows laid out for one width. The owner: "for each
    field I have to scroll top to bottom to change its value"; "the entire
    input could go in that one row"; "never ever do it for single size
    screen". Now (owner rule 8, § How the Bill and KOT tabs are laid out):
    one list, the open line edited where it sits, columns and totals one row
    each, the preview beside the open line on wide screens, every row sized
    from its own card. Do not bring back: a second order list or picker
    (`bill_ord_*`, `kot_ord_*`, `kot_col_item` / `kot_col_qty` pick chips,
    `bill_item_col_edit_idx`, `kot_field_edit_idx`, `kot_item_col_edit`),
    hidden "legacy hot-reload" blocks, a label above a box whose own
    caption already names it, or `set_visible` on a text box (it ignores it
    — the column header boxes stayed on screen with headers Off; they hide
    through `col_header_box` now). Rows are sized from the form card, not
    the page ([`DEV.md`](DEV.md) Makepad traps 20–22). Check a change at
    ~420, ~1024 and ~1440 px before calling it done.

### Working approaches (keep)

- ESC/POS on a named queue → `print_system_raw`: the whole job as one
  raw job on that queue (`lp -o raw` / Windows `WritePrinter`); `lp`
  accepting it means it is queued — the app does not watch or cancel it.
  Never route around the queue the owner picked ("What went wrong" 9, 14).
- Network `host:port` → connect, the whole job, close.
- OS document = the same picture as the Settings preview (`slip_page_png`),
  on the 210 mm page and fitted into the printer's printable area on CUPS.
- Empty printer How → format How; never invent from target kind.
- Unsupported How → clear error; no silent swap to ESC/POS on USB/BT/:9100.
- Bill & KOT format form: positions, not fields, in the DSL — `bill_pick_*`
  / `bill_after_*` and `kot_pick_N` / `kot_after_N` are slots before and
  after the open line, filled from the print order (`ordered()`,
  `kot_lines`); ↑ / ↓ move the open line (`kot_move_line` keeps Title first
  and Item / Qty last). `layout_slip_editors` is the one place that sets
  every row's flow; `align_slip_previews` moves the side preview after a
  draw, only while the panel is shown.

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
- How the job is sent: OS document or ESC/POS (Raw was removed in all four
  apps — lesson 19). Those two cover a docket and a sticker. A label command language (ZPL, TSPL) is
  not used; the label is a picture of the slip plus Code 128 bars, so a
  normal printer and a thermal head both get the barcode.
- Type size.
- **Test receipt** or **Test label**, under the save button, prints a
  sample of that job with the width, layout, delivery, and size on screen.
