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
   **Confirmed after use** (2026-11-15): columns keep exactly the mm typed
   and room after the last column stays empty — End aligns inside the
   column, not at the slip's edge. Asked whether the leftover should go to
   a column, the owner, having set it up with width / start / end space:
   "what we are seeing is not actually a bug but real feature … It's
   providing the exact preview with exact output … we have kept it fully
   controllable and configurable so that user could customize the way he
   wants". Do not make a column grow to fill the slip.
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
9. **The printer owns its paper; the preview shows the content on it**
   (2026-11-16). Paper width (and length for sheets) is set per printer on
   the Printers screen, filled by Find when blank and never lost by a later
   Find. The Bill and KOT screens own the content width only; their
   preview shows the content on the paper of any printer picked above it
   (every saved printer is listed). OS document and ESC/POS differ only in
   how the data is sent. § Paper and preview.
10. **The preview must never fool the user** (2026-11-16): "user doing
    mistakes is his responsibility but our app should be very accurate,
    flexible, bug free in showing the preview and it shouldn't fool user in
    any way, it should show the real expected preview." A wrong value is
    the owner's; a preview or print that differs from the real result is
    a bug in the app.

## Printer

Settings → **Printers** is the device list **and** each device’s send method.

- **Find printers** reads the OS list (system queues, then network, USB, and
  Bluetooth) and saves it to this shop’s `printers.json` and the running app.
- **Since 2026-11-16 (owner) each printer also holds its paper**: the list
  reads `name · paper` (`80 mm roll`, `210 × 297 mm`); tapping a printer
  opens Width and Length (mm, blank length = roll) where it sits. Find
  fills a blank paper and never changes one already set (§ Paper and
  preview).
- **Since 2026-11-13 (owner): the screen is Find + the saved list only** —
  names (and, since 2026-11-16, each printer's paper), nothing else. The per-printer send-method chips and the test-page
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
anything on its own. Since 2026-11-16 it shows the content on the paper of
the printer picked above it (rule 9, § Paper and preview).

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

**Units** (one scale, preview and paper): a mm is 7.2 of the picture's dots
(`DOTS_PER_MM`, 576 per 80 mm) and a px is 1/96 inch (`PX_DOTS`, 1.905
dots). A column's width is mm at that scale; its start / end space and every
type size are px; the OS document prints the picture at its own dots ÷ 7.2
mm on the printer's own paper, never fitted (§ Paper and preview). Each of these is
pinned by a test (lessons 22, 24).

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

### Logo

The bill's **Logo** line prints a picture (the owner asked for "maximum
possible control … and accurate preview"; 2026-11-15). It is a line like the
others — Show / Hide, ↑ / ↓, Start / Center / End — first in the order, above
Shop, and **hidden** until wanted, so a bill from before it prints the same.
Choosing a picture shows it. Its controls, when Logo is open:

- **Picture** — Choose picture / Remove picture; the status gives its size
  in pixels or "No picture". Kept in the books as
  `brand_images/bill_logo.webp` (lossless, transparency kept; larger than
  2048 px a side is shrunk to that), meta `bill_logo_key`; chosen on a
  Client, it is sent to the Server (`BooksOp::ImportBillLogo` /
  `ClearBillLogo`).
- **Width** and **Height** in px (1/96 inch, like type sizes), each up to
  794 px (210 mm). **Blank follows the picture**: one blank keeps the
  picture's shape, both blank print its own pixels as px (as a web page
  shows an `<img>`); both set make the box exactly that size and the
  picture **fits inside it, never stretched**, placed by the line's
  alignment and centred top to bottom.
- **No picture**: a shown Logo keeps its **Height** as blank space (paper
  for a stamp or a hand-written note); with no Height it takes no room.
  **Hide** removes the line and its space.
- **Greys** — printers print dots, each ink or paper. **Solid**: darker
  than mid-grey is ink (crisp line logos). **Shaded**: greys become a fine
  dot pattern (Floyd–Steinberg) for photos and soft shading. Transparent is
  paper.
- Wider than the content: cut at the content edges, as columns are.

The picture travels **inside the bill text** as one line,
`\u{E001}P{align}|{width}|{height}|{dots}` (the box in the printer's dots,
deflated, base64): a bill built on the Server and printed by a waiter's
device, a queued job and the preview all draw the same dots, and the
receipt file for people drops the line (`strip_kot_style_marks`). Decoded
pictures are cached, so typing a size does not decode the file again.
Tests: `logo_*` in `print_share.rs`.

### How the job is sent (per printer)

| Choice | What the device receives |
|---|---|
| **OS document** | A picture of the slip (the preview's picture, bundled fonts), given to the system print queue at its **real size on the printer's paper** (the width × length set for that printer), from the top-left of that paper's printable area (§ Paper and preview). CUPS: one picture per sheet, each exactly the printable width, sent with that paper named (the driver's matching size or `Custom.<w>x<l>mm`; `lp_sheet_options`). Windows: the matching or a custom `PaperSize`, each page drawn from the printable top-left (`windows_slip_script`). Android: the paper picked in the print dialog (`printSlipPages`). |
| **ESC/POS** | The slip picture (the same picture as the preview) in standard ESC/POS raster: `ESC @`, `GS v 0` bands, feed, cut (`escpos_job_format`). The whole job goes to the picked destination in one go: network `host:port` (connect, send, close), USB, Bluetooth, or the picked system queue as one raw job (`lp -o raw` / `WritePrinter`). |

Kitchen-ticket style marks (size, weight, Start / Center / End) are drawn
in that picture for both the OS document and the ESC/POS raster. Do not
strip the marks and then left-align the plain text.

USB / Bluetooth / `:9100` take ESC/POS. **OS document on those direct
targets is refused with a clear error** — pick ESC/POS, or use a system
queue that can print a page. The app does not silently
swap How, and must not silently drop that printer from a multi-printer
route without a status note.

### Paper and preview (owner's decision, 2026-11-16)

**The issue.** OS-document bills came out cut or misplaced because the page
the app sent did not match the paper in the printer. Until 2026-11-15
every OS-document bill was put on a fixed 210 mm page that CUPS shrank onto
the driver's paper (lesson 24). The first fix (2026-11-16, never committed)
put one **Paper** dropdown for every printer on the Bill / KOT screen, with
a paragraph of text under it. That was wrong: nobody asked for it. It sat
on the wrong screen, it looked like one paper for all printers, and it put
explanations on the screen. It also kept CUPS's `print-scaling=none`
fallback, which centres the page (lesson 21). It was replaced by the
model below.

**The owner's model** (his words: "the printer could have papers of any
size"; "it's a global app and the printer would not be known"; "user doing
mistakes is his responsibility but our app should be very accurate,
flexible, bug free in showing the preview and it shouldn't fool user in any
way, it should show the real expected preview"):

1. **The printer owns its paper.** Each printer has a paper width in mm and,
   for sheets, a length (blank = a roll: each job is as long as its
   content). It is set in one place only: **Settings → Printers**, by
   opening that printer where it sits in the list
   (`NamedPrinter::paper_width_mm` / `paper_length_mm`). Any job can go to
   any printer (a KOT on an office printer, a bill on a thermal head), so
   whatever prints there lands on that printer's paper.
2. **Find printers fills it in, and never takes it away.**
   - A printer with no paper gets its driver's current paper (CUPS PPD +
     `lpoptions`, Windows `PaperSizes`).
   - ESC/POS devices (network raw, USB, Bluetooth) and anything that cannot
     say get an 80 mm roll. Never A4 or any other silent guess.
   - A paper already set is never changed by Find, even when the driver now
     says something else; the driver's setting is the owner's to correct.
   - Printers are matched by their target (queue name, `ip:port`,
     `usb:VID:PID:serial`, `bt:MAC`), so a second Find keeps every paper
     (test `find_twice_keeps_every_paper`).
   - A paper name saved by the 2026-11-16 draft (`"paper": "Env10"`) is read
     once and turned into width × length from the driver's list.
3. **Bill and KOT screens own the content only:** content width, lines,
   sizes. No paper setting there.
4. **The preview shows the content on a printer's paper.**
   - A dropdown above the preview lists **every** saved printer as "name ·
     paper". Picking one only changes the preview: it does not choose who
     prints, their order, or First / All. The pick is remembered while the
     app runs. With no printer saved, the preview shows an 80 mm roll.
   - The picture is made by the **same code that makes the printed sheets**
     (`os_paper::preview_picture` over `os_document_sheets`). The paper is
     drawn whole: the content at real size from the printable top-left,
     the edges the printer cannot reach shaded, a long slip continued on
     the next sheet where the print breaks it, a slip wider than the paper
     cut where the print cuts it.
   - ESC/POS shows the roll the printer's width wide, the content from its
     left edge. ESC/POS printing itself is unchanged.
   - Nothing is explained on the screen; the ℹ beside Preview says it.
5. **Sending (OS document): a page exactly the printer's paper.** The
   content goes at real size from the printable top-left, with no scaling
   and nothing centred. A long slip goes on more sheets.
   - **CUPS:** the driver's own size when one matches width × length, else
     `Custom.<w>x<l>mm`. A roll is one page as long as the slip. Sent with
     `print-scaling=fit` (1:1, since each sheet is exactly the printable
     width), `position=top-left`, portrait (`lp_sheet_options`).
   - **Windows:** the driver's paper that matches, else a custom
     `PaperSize`, each page drawn from the printable top-left.
   - **Android:** the system print dialog chooses the paper (it opens on
     the printer's own); pick the printer's paper there.
   - **Driver unreadable:** the page is still the printer's paper, named
     `Custom.<w>x<l>mm`, from the top-left with the driver's margins
     unknown. The status says so; it is never centred.
   - **ESC/POS:** unchanged.

**TODO, decision not made: where the content sits on the paper.** The owner
wants to place the content on a bigger paper (e.g. 105 mm content on A4):
nine places (top / middle / bottom × left / centre / right), remembered.
Not decided: **one position per format** (Bill, KOT; "centre" is centred on
every printer's paper) or **one per printer**. Known limits: on a roll only
across counts; a slip longer than one sheet starts at the top; content
wider than the paper starts at the left so its beginning is never lost.
Until decided, content sits at the printable top-left.

**More about the model:**
- Settings → Printers opens **any number** of printers where they sit (owner,
  2026-11-16: "this shouldn't be restricted"): one list in two parts around
  the open printer's Width / Length (`PrinterPaperRows`, `part: "before"` /
  `"after"`; only the rows on screen draw). No slots, no read-only tail.
- The picture always follows the face. The dropdown names the printer
  whose paper the picture is drawn on; when that printer changes without a
  pick (the tab's printers changed, Find added one, a paper was typed, a
  driver answered), the picture is drawn again, not only the face
  (`sync_preview_paper_ui` compares `KotSlipPreview::shown_paper` /
  `SlipPreview::shown_paper` with `preview_paper` — before 2026-11-16 the
  face could name the new printer over the old printer's picture).
- A driver that takes **no custom sizes** (CUPS without
  `*CustomPageSize True`, Windows with no `PaperSize` of `Kind` Custom — e.g.
  "Microsoft Print to PDF") cannot have a page made to the printer's paper:
  the job goes on the driver's own paper, from the top-left, and the status
  says so (`PrinterPapers::custom_ok`, `PagePlan::fallback`, the Windows
  script's `KUMKAJ-NOTE:` line). The preview draws the same page.
- A printer with no paper is asked for it whenever it is listed (Printers,
  the preview's dropdown, a preview, a job): its driver answers on its own
  thread and the answer fills it and is saved. Only Find gives the 80 mm
  default to a driver that does not answer; a screen repaint never saves
  one (`fill_paper`'s `default_when_unanswered`).
- Each paper is what the driver says (A4 is 595 × 842 pt = 209.9 × 297.0
  mm); a typed 210 still uses the driver's A4 (sizes within 0.6 mm match).

**Checked without printing (2026-11-16)** — do it again after any change:
- Tests: `the_preview_shows_exactly_the_printed_sheets_on_the_paper` (the
  preview's printable areas hold the printed sheets' pixels, breaking where
  the print breaks), `the_page_is_the_printers_paper_named_by_the_driver_
  when_it_can`, `a_preview_in_printer_dots_is_the_same_picture_at_any_dot_
  size`, `an_escpos_preview_is_the_roll_with_the_picture_from_its_left_
  edge`, `find_twice_keeps_every_paper`, `a_draft_paper_name_loads_and_is_
  kept`, `a_job_goes_on_the_printers_paper_or_what_find_would_give_it`.
- CUPS, through this machine's own filters up to the PDF the driver gets:
  `write_os_document_sheets` (ignored test; `KHAYNJO_PPD`,
  `KHAYNJO_PAPER_MM=<w>x<l>` or `<w>` for a roll, `KHAYNJO_BILL_MM`,
  `KHAYNJO_LINES`, `KHAYNJO_SHEETS_OUT`), then `cupsfilter -p <ppd> -m
  application/vnd.cups-pdf <its lp options>` and measure the PDF (72 dpi):
  a generic PCL driver's Envelope #10 (105 mm bill) → 297 × 684 pt page,
  `media=Env10`, ink at 25, 39 pt (the driver's 18, 36 pt margins + the
  bill's own 2.2 mm side), cut at the 92.1 mm printable width; a 4 in roll
  (80 mm bill, 30 lines) → `Custom.101.6x443.26mm`, one page as long as the
  bill; Letter → 612 × 792 pt; a custom 4 × 6 in sheet with a long bill → 4
  pages, each from 25 pt. The 80 mm bill is 213–214 pt (75.1–75.5 mm) wide
  on every paper. Never centred.
- Screens on a private Xvfb (`--remote`, the neutral `visual_seed` shop,
  made-up printers and driver files served by a throwaway local server as
  `CUPS_SERVER`): Printers list `name · paper`, a printer opened in place,
  Width / Length typed and saved, the preview on a 4 × 6 in sheet, A4, an
  80 mm ESC/POS roll and an A5 filled from its driver, at 1440 and 420 px.
- Windows, in the Windows VM (`write_windows_os_document_check`, printing
  only to "Microsoft Print to PDF" with PrintToFile): an A5 paper → 5 pages
  of 419.52 × 595.32 pt, the bill 213 pt wide from 7 pt; an 80 mm roll on
  that driver (no custom sizes) → its own A4, from the top-left, with
  "KUMKAJ-NOTE: its driver has no such paper and takes no other size;
  printed on its own paper, A4". Android was checked for the earlier draft
  only.
- The same model in Kirano and Tabiyat (Bill) and Moklanjo (receipt and AWB
  label, § Moklanjo), 2026-11-16: each app's paper tests (18) and suites
  green; on Xvfb the Printers list `name · paper`, a printer opened in
  place, and the preview on an A4 driver paper, a 101.6 mm roll and an
  80 mm ESC/POS roll.

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
- **Preview** on a printer's paper: the dropdown above it lists every saved
  printer (`name · paper`); picking one shows the bill on that paper only
  (§ Paper and preview). No paper is set here. The KOT tab has the same.
- The list of bill lines; the open line has show or hide, size (px),
  weight, and alignment (Start / Center / End), plus typed text where the
  line has it (address, FSSAI, title, footer, UPI). Lines include **logo**
  (§ Logo — its box size instead of type size and weight), shop, address, FSSAI,
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
  station — ↑ / ↓ move these), **Items** last. The open line has show
  or hide, size (1–200 px), weight (Light / Regular / Bold), and alignment
  (Start / Center / End, following the writing direction).
- **Title** open: the title text and this kind's banner.
- **Items** open (the dish table; it always prints, so no Show / Hide):
  its size and weight are the whole table's; its Start / Center / End place
  the dish notes and the kitchen note (each column has its own alignment).
  Then Column headers, Strike dish name, Strike dish note, Item prefix, one
  row per column (**Dish name**, **Qty**: Header, width, start and end
  space, alignment) and the kitchen note.
- **One name per thing** (owner, 2026-11-15: "We have Item · Qty, then
  later Dish name and then again Qty? I don't understand this"). The table
  line is **Items** on both tabs (it was "Item · Qty" on KOT); a column is
  named once, by its row title, and its header box is captioned **Header**
  (it was "Item header" under "Dish name"). Do not caption a box with a
  second name for what its row already names.
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
21. **The OS-document bill printed under a big blank band** (2026-11-15).
    The owner's bill (OS document, CUPS) came out in the lower half of the
    sheet, and a change of width "had no impact". The picture the app sent
    was right — 210 mm at 17:53, about 180 mm at 17:56, the same text size
    in both. CUPS **centres** a picture on the page unless the job says
    where, so a short bill sat mid-page; lesson 17's `cupsfilter` check
    looked at size and margins, not at where the slip sat. Fix: the
    standard `position=top-left` (`lp_image_options`, test
    `os_document_starts_at_the_top_left_of_the_page`); Android and Windows
    already drew it at the top-left. Checked with `cupsfilter` (no
    printing) at 58 / 80 / 112 / 210 mm on an A4 queue and an 80 mm roll
    queue: one page, top-left, the same title size at 80, 112 and 210 mm.
    Rejected in the same check — do not redo: `print-scaling=none` or
    `auto` (CUPS centres the picture and ignores `position`); `ppi` alone
    (true size, but a picture wider than the printable area goes on to a
    second and third sheet — a 210 mm bill on A4, an 80 mm slip on a roll
    with 76 mm printable). When checking a print, look at **where** it sits
    on the page, not only its size.
    Text at 58 mm and below: lesson 22. A sheet narrower than the queue's
    paper (Letter / A4 queue, smaller sheet) also lost its left edge — the
    printer centres the page on the sheet. That was put down to "the
    queue's paper setting, not the app": wrong — the app laid out a 210 mm
    page and never said which paper it meant (lesson 24, § Paper and preview).
22. **Column and paper units were not what the form says** (2026-11-15,
    found when the owner asked for the column feature to be "technically
    correct and standard"). The preview always matched the paper (one
    picture), but three units were off: (a) a column's start / end space
    is typed in px but the directive's number was drawn as dots — a 4 px
    space was 4 dots (~0.56 mm), half its size; `parse_slip_columns` now
    turns px into dots (`px_dots`), so a 4 px space is 7.6 dots (~1.06 mm)
    and owners' gaps print ~0.5 mm wider per 4 px than before. (b) The
    OS document sized the slip from the rounded wrap-column count
    (`content_mm_for_format`, removed): 70 mm printed 71, 112 printed 113,
    text ~1 % large; it is the picture's dots ÷ 7.2 now
    (`os_document_content_mm`), so a px is 1/96 inch in the picture at
    every width (`os_document_slip_is_7_2_dots_per_mm_at_every_width`) —
    "on paper" was claimed here and was wrong until lesson 24: the 210 mm
    page was then fitted to the queue's paper. The
    Windows PNG's density stamp said a third of its real density (dots, not
    the 2–3× PNG pixels). (c) A column's mm became dots at the picture's
    own ratio (7.2 ± 1.4 % from rounding); it is exactly 7.2 above 58 mm
    (`content_dots_per_mm`). The column model itself is the owner's (rule 6)
    and did not change.
    Still open, the owner's call: (1) every content width of 58 mm or less
    — and 59 — draws the 58 mm head's 384 dots, so 40 and 58 mm give the
    same picture, its columns spread over the typed mm, and the OS document
    (dots ÷ 7.2) prints it 53.3 mm wide; one 7.2 scale for every width would
    make 58 mm 417 dots — wider than a 58 mm head — and move columns saved
    at those widths. (2) The content width reaches the picture as a
    wrap-column count (~0.55 per mm) rounded to 8 dots, so the content edge
    can sit up to ~1.2 mm inside the typed width (99 mm → 97.8 mm); exact
    edges need the print format to carry the mm itself.
23. **Found while adding the Logo** (2026-11-15). Fixed: (a) the open
    bill line was two facts — the line and a stored index — and a new
    first line (Logo) would have opened Shop's editor in Logo's place; the
    line is the one fact now and its place is found in the order
    (`BILL_PICK_BTNS`, no `bill_field_edit_idx`). (b) The WhatsApp bill PDF
    was made from the marked print text: every line mark, the column
    directive (and now a picture) printed as `?…` junk; it uses the
    receipt file's words now (`strip_kot_style_marks`). (c) The Bill line
    editor still held a hidden "kept for older saves" block of five old
    inputs (address, FSSAI, UPI ×2, footer) that nothing read — removed with
    their five placeholder strings (lesson 20: no hidden legacy blocks).
    Still open: (1)
    the bill line names other than Logo are English on the Bill tab in
    every language (`BillStyleField::label`); (2) the Stamp and Payment
    QR pictures print as the words `[Stamp]` / `[Payment QR]` — the
    picture line can print the real pictures; (3) a Client device does not
    get the Server's bill styles (only address, FSSAI, footer and UPI), so
    its Bill tab preview shows its own defaults; bills are built on the
    Server, so what prints is right on every device; (4) the Logo is
    Khaynjo's only — Kirano and Tabiyat share the Bill form and need it
    ported.
24. **The OS document assumed A4** (2026-11-15 / 16). The owner's 4-inch
    sheet in a printer whose queue said Letter came out with its left part
    missing, and his client's small printer did the same. Every OS-document
    bill was put on a fixed 210 mm page (`PAGE_MM`) that CUPS then fitted to
    whatever paper the queue had; the app never asked the system for the
    paper and never named one. Measured with `cupsfilter` on the owner's own
    22:20 bill: 97 % size on Letter, 94 % on A4, 65 % with the queue set to
    A5, a 400 mm bill on A4 at 68 % (shrunk onto one page); a sheet narrower
    than the queue's paper lost its left side (centred); Android drew an A4
    page inside fixed 12.7 mm margins (88 % by its code); Windows shrank a
    bill wider or longer than the page. The picture was right, the paper was
    not — lessons 21 and 22 claimed otherwise. Fix: § Paper and preview —
    the page is the printer's own paper (set on Printers, filled by Find
    from its driver), the bill at its real size from the printable corner,
    more sheets for a long bill, a cut for a wide one, all shown in the
    preview. (The first fix's Paper dropdown on the Bill / KOT screen was
    wrong and was removed the same day, see that section.) Not
    the rejected true-size PDF (lesson 17): the sheets are pictures exactly
    the printable area wide, sent portrait with the paper named, so nothing
    turns and nothing falls in the unprintable margin.
    Also found: (a) the Android build kept an older print bridge —
    `apply-android-activity-bridge.sh` skipped the print snippet whenever
    the old block's `kumkajEscposPieceEnds` was present, so APKs still had
    the paced Bluetooth / USB writes lesson 14 removed; the script now
    replaces the block whenever it differs from the snippet. (b) The
    content-width presets said "112 mm — 4 inch slip" (4 in is 101.6 mm) and
    "210 mm — page", and the width tips "A4-wide": they are plain mm now.
    (c) A list row's right-hand text takes its full width (`SuggestRowView`
    subtitle is `Fit`): a long one left the title no room at phone width and
    the whole list drew nothing — keep that text short (list rows give
    only short sizes there).
    Kirano, Tabiyat and Moklanjo printed OS documents the old way until
    the same day (Kirano / Tabiyat the 210 mm page and Android's A4;
    Moklanjo `lp -o fit-to-page` and a Windows page fitted into fixed
    margins, so an 80 mm receipt came out page-wide); all three now use
    `os_paper.rs`, the printer's paper and the preview on it.

### Working approaches (keep)

- ESC/POS on a named queue → `print_system_raw`: the whole job as one
  raw job on that queue (`lp -o raw` / Windows `WritePrinter`); `lp`
  accepting it means it is queued — the app does not watch or cancel it.
  Never route around the queue the owner picked ("What went wrong" 9, 14).
- Network `host:port` → connect, the whole job, close.
- OS document = the same picture as the Settings preview (`slip_page_png`),
  at its real size on the printer's paper (width × length set on Printers),
  from the printable top-left, one sheet per printable height (§ Paper and
  preview, lesson 24). The preview is made from the same sheets. Never a
  size the app assumed, never `print-scaling=none` (it centres).
- A picture on a slip (the Logo) is a `\u{E001}P` line in the slip text and
  a `LineColumns::Picture` row in the raster — one path for preview,
  OS document and ESC/POS (§ Logo).
- Empty printer How → format How; never invent from target kind.
- Unsupported How → clear error; no silent swap to ESC/POS on USB/BT/:9100.
- Bill & KOT format form: positions, not fields, in the DSL — `bill_pick_*`
  / `bill_after_*` and `kot_pick_N` / `kot_after_N` are slots before and
  after the open line, filled from the print order (`ordered()`,
  `kot_lines`); ↑ / ↓ move the open line (`kot_move_line` keeps Title first
  and Items (`ItemQty`) last). `layout_slip_editors` is the one place that sets
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
attach those devices to Receipt or AWB label. Each saved printer is listed as
`name · paper` and opens where it sits for its Width / Length (§ Paper and
preview); Find fills a blank paper from the driver (an 80 mm roll when it
cannot say) and never changes one already set. When a label or run sheet had
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
- **Preview** (2026-11-16, § Paper and preview): the job's own sample —
  the one Test receipt / Test label prints — on the paper of the printer
  picked in the dropdown above it. Every saved printer is listed; the pick
  changes the preview only; with none picked it is that job's first
  printer. OS document: the whole paper, the content at real size from the
  printable top-left. ESC/POS: the roll the printer's width wide, the
  picture from its left edge — a 100 mm label on an 80 mm roll is drawn
  cut at the roll's edge, as it prints (`SlipPicture`, `slip_gray`: the
  job's own dots, no PNG round trip).
- **Test receipt** or **Test label**, under the save button, prints a
  sample of that job with the width, layout, delivery, and size on screen.
