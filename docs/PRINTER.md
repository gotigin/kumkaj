# Printers and Bill & KOT format (Khaynjo)

Two separate things. A **printer** is a real device the operating system
already knows. **Bill & KOT format** is how a bill or a kitchen ticket is
drawn and how those bytes are handed to that device. The printer does not
decide the layout. The format does not name the device.

## Printer

Settings → **Printers** is only the device list.

- **Find printers** reads the OS list (system queues, then network, USB, and
  Bluetooth) and saves it to this shop’s `printers.json` and the running app.
- Each row keeps the name the OS already gave it. There is no manual rename
  and no virtual printer to create.
- The app does not guess inkjet, laser, or thermal from the name.

A kitchen station, and the bill, each pick one or more of these printers.
**First that works** stops after the first success. **Print on all** sends
the same job to every printer on that place. Order on the list is the try
order.

Old `printers.json` files may still contain virtual-printer rows. Load
rewrites those route slots to the real printer they pointed at. The product
screen does not edit them.

## Bill & KOT format

Settings → **Bill & KOT format** decides the job. The physical paper stays
the printer’s. **Content width** is how wide the drawn bill and the drawn
kitchen ticket are. It is one millimetre value for both, above the Bill and
KOT tabs: 58 (narrow slip), 80 (wide), 112 (4 inch), 210 (page), or a typed
value from 40 to 210. It is not a paper-width lock on the printer, and it
is not only 58 or 80.

Portrait / landscape, and how the job is sent, are chosen **separately** on
the Bill tab and on the KOT tab. Either job can go to any printer Find has
saved.

### How the job is sent

Three ways. They cover a bill and a kitchen ticket. A further command
language (ZPL, TSPL, StarPRNT, CPCL) is for label printers and is not part
of this screen.

| Choice | What the device receives |
|---|---|
| **OS document** | A picture of the slip (bundled fonts), given to the system spooler. Inkjet, laser, and any queue that prints a page. On Android, the print service. |
| **ESC/POS** | Thermal commands (plain text when the head has the letters, otherwise a raster picture). Network port 9100, USB, and Bluetooth always take these bytes. A named desktop queue takes them as a raw spool. |
| **Raw bytes** | The same job bytes, written straight to the device or the raw queue so the OS does not treat them as a document. |

OS document is `document` on the format. ESC/POS and Raw are not: the
worker does not ask the spooler to reflow the page. Kitchen-ticket style
marks (size, weight, Start / Center / End) are drawn in that picture for
both the OS document and the ESC/POS raster. Do not strip the marks and
then left-align the plain text.

### Bill tab

- Content width (shared with the kitchen ticket).
- Portrait or landscape.
- How the job is sent.
- Type size for the bill.
- Which real printers may print the bill.
- Header text where the bill format has it.

The bill has no per-field Start / Center / End. Its type size, content
width, layout, and delivery are the Bill format used when the job is sent.

### KOT tab

- Its own portrait or landscape, and its own how-the-job-is-sent.
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
  - **Strike item names** (on by default for Cancelled; qty column stays clear).
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

Save KOT format writes that slip. The printed ticket reads the saved slip,
not an unsaved edit still sitting in the form. **Test KOT**, under that
save button, prints the preview on a kitchen printer. **Test bill**, under
Save bill format, prints a sample bill with the header, footer, size,
content width, layout, and delivery currently on the Bill tab.

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
