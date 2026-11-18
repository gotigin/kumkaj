    // Kumkaj: PrintManager + Bluetooth SPP + USB host ESC/POS.
    // Inserted before copyToClipboard by scripts/apply-android-activity-bridge.sh.

    private static final UUID KUMKAJ_SPP_UUID =
            UUID.fromString("00001101-0000-1000-8000-00805F9B34FB");

    // The phone's Bluetooth adapter from its BluetoothManager: the same adapter
    // BluetoothAdapter.getDefaultAdapter() gave, without that deprecated call
    // (deprecated from API 31; BluetoothManager is on every API this runs on).
    private BluetoothAdapter kumkajBluetoothAdapter() {
        BluetoothManager manager = (BluetoothManager) getSystemService(Context.BLUETOOTH_SERVICE);
        return manager == null ? null : manager.getAdapter();
    }


    public String[] listBluetoothPrinters() {
        try {
            BluetoothAdapter adapter = kumkajBluetoothAdapter();
            if (adapter == null) {
                return new String[0];
            }
            java.util.Set<BluetoothDevice> bonded = adapter.getBondedDevices();
            if (bonded == null || bonded.isEmpty()) {
                return new String[0];
            }
            java.util.ArrayList<String> out = new java.util.ArrayList<>();
            for (BluetoothDevice d : bonded) {
                String addr = d.getAddress();
                if (addr != null && addr.length() > 0) {
                    out.add("bt:" + addr.toUpperCase(java.util.Locale.US));
                }
            }
            return out.toArray(new String[0]);
        } catch (Throwable e) {
            Log.e(LOG_TAG, "listBluetoothPrinters failed", e);
            return new String[0];
        }
    }

    public boolean writeBluetoothSpp(String mac, byte[] data) {
        if (mac == null || data == null || data.length == 0) {
            return false;
        }
        String addr = mac.trim();
        if (addr.toLowerCase(java.util.Locale.US).startsWith("bt:")) {
            addr = addr.substring(3);
        }
        BluetoothSocket socket = null;
        try {
            BluetoothAdapter adapter = kumkajBluetoothAdapter();
            if (adapter == null) {
                return false;
            }
            BluetoothDevice device = adapter.getRemoteDevice(addr.toUpperCase(java.util.Locale.US));
            socket = device.createRfcommSocketToServiceRecord(KUMKAJ_SPP_UUID);
            adapter.cancelDiscovery();
            socket.connect();
            java.io.OutputStream out = socket.getOutputStream();
            // The whole job in one write (no pieces, no pauses).
            out.write(data);
            out.flush();
            // Android: write() returns once the stack has the bytes and
            // close() drops what it has not sent yet, so give the link time
            // to carry the job before closing.
            Thread.sleep(Math.min(8000L, 400L + data.length / 8));
            return true;
        } catch (Throwable e) {
            Log.e(LOG_TAG, "writeBluetoothSpp failed", e);
            return false;
        } finally {
            if (socket != null) {
                try {
                    socket.close();
                } catch (Throwable ignored) {
                }
            }
        }
    }

    public String[] listUsbPrinters() {
        try {
            UsbManager usb = (UsbManager) getSystemService(USB_SERVICE);
            if (usb == null) {
                return new String[0];
            }
            java.util.ArrayList<String> out = new java.util.ArrayList<>();
            for (UsbDevice device : usb.getDeviceList().values()) {
                String serial = null;
                try {
                    serial = device.getSerialNumber();
                } catch (Throwable ignored) {
                }
                String id = String.format(
                        java.util.Locale.US,
                        "usb:%04x:%04x",
                        device.getVendorId(),
                        device.getProductId());
                if (serial != null && serial.length() > 0) {
                    id = id + ":" + serial;
                }
                out.add(id);
            }
            return out.toArray(new String[0]);
        } catch (Throwable e) {
            Log.e(LOG_TAG, "listUsbPrinters failed", e);
            return new String[0];
        }
    }

    public boolean writeUsbPrinter(String key, byte[] data) {
        if (key == null || data == null || data.length == 0) {
            return false;
        }
        try {
            String raw = key.trim();
            if (raw.toLowerCase(java.util.Locale.US).startsWith("usb:")) {
                raw = raw.substring(4);
            }
            String[] parts = raw.split(":");
            if (parts.length < 2) {
                return false;
            }
            int vid = Integer.parseInt(parts[0], 16);
            int pid = Integer.parseInt(parts[1], 16);
            String wantSerial = parts.length >= 3 ? parts[2] : null;
            UsbManager usb = (UsbManager) getSystemService(USB_SERVICE);
            if (usb == null) {
                return false;
            }
            UsbDevice match = null;
            for (UsbDevice device : usb.getDeviceList().values()) {
                if (device.getVendorId() != vid || device.getProductId() != pid) {
                    continue;
                }
                if (wantSerial != null) {
                    String serial = null;
                    try {
                        serial = device.getSerialNumber();
                    } catch (Throwable ignored) {
                    }
                    if (serial == null || !wantSerial.equals(serial)) {
                        continue;
                    }
                }
                match = device;
                break;
            }
            if (match == null) {
                return false;
            }
            if (!usb.hasPermission(match)) {
                Log.e(LOG_TAG, "writeUsbPrinter: no USB permission");
                return false;
            }
            UsbDeviceConnection conn = usb.openDevice(match);
            if (conn == null) {
                return false;
            }
            try {
                UsbInterface iface = null;
                UsbEndpoint epOut = null;
                for (int i = 0; i < match.getInterfaceCount(); i++) {
                    UsbInterface candidate = match.getInterface(i);
                    UsbEndpoint found = null;
                    for (int e = 0; e < candidate.getEndpointCount(); e++) {
                        UsbEndpoint ep = candidate.getEndpoint(e);
                        if (ep.getType() == UsbConstants.USB_ENDPOINT_XFER_BULK
                                && ep.getDirection() == UsbConstants.USB_DIR_OUT) {
                            found = ep;
                            break;
                        }
                    }
                    if (found != null) {
                        iface = candidate;
                        epOut = found;
                        break;
                    }
                }
                if (iface == null || epOut == null) {
                    return false;
                }
                if (!conn.claimInterface(iface, true)) {
                    return false;
                }
                try {
                    // The whole job, in USB bulk packets (no pieces, no pauses).
                    int offset = 0;
                    while (offset < data.length) {
                        int chunk = Math.min(epOut.getMaxPacketSize(), data.length - offset);
                        int written = conn.bulkTransfer(epOut, data, offset, chunk, 15000);
                        if (written <= 0) {
                            return false;
                        }
                        offset += written;
                    }
                    return true;
                } finally {
                    conn.releaseInterface(iface);
                }
            } finally {
                conn.close();
            }
        } catch (Throwable e) {
            Log.e(LOG_TAG, "writeUsbPrinter failed", e);
            return false;
        }
    }

    /** Standard bill PNG — portrait A4 (compat for older JNI callers). */
    public boolean printPngDocument(final String jobName, final byte[] pngBytes) {
        return printPngDocumentOriented(jobName, pngBytes, false);
    }

    /** Standard bill: Noto-raster PNG; `landscape` selects A4 orientation. */
    public boolean printPngDocumentOriented(
            final String jobName, final byte[] pngBytes, final boolean landscape) {
        final byte[] body = pngBytes == null ? new byte[0] : pngBytes;
        final String title = (jobName == null || jobName.length() == 0) ? "Slip" : jobName;
        try {
            runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    try {
                        PrintManager pm = (PrintManager) getSystemService(PRINT_SERVICE);
                        if (pm == null) {
                            return;
                        }
                        final android.graphics.Bitmap bmp =
                                android.graphics.BitmapFactory.decodeByteArray(body, 0, body.length);
                        if (bmp == null) {
                            Log.e(LOG_TAG, "printPngDocument: bad PNG");
                            return;
                        }
                        PrintAttributes.MediaSize media = landscape
                                ? PrintAttributes.MediaSize.ISO_A4.asLandscape()
                                : PrintAttributes.MediaSize.ISO_A4;
                        // PDF points: portrait 595×842, landscape 842×595.
                        final int pageW = landscape ? 842 : 595;
                        final int pageH = landscape ? 595 : 842;
                        PrintAttributes attrs = new PrintAttributes.Builder()
                                .setMediaSize(media)
                                .setColorMode(PrintAttributes.COLOR_MODE_MONOCHROME)
                                .setMinMargins(PrintAttributes.Margins.NO_MARGINS)
                                .build();
                        pm.print(title, new PrintDocumentAdapter() {
                            @Override
                            public void onLayout(
                                    PrintAttributes oldAttributes,
                                    PrintAttributes newAttributes,
                                    android.os.CancellationSignal cancellationSignal,
                                    LayoutResultCallback callback,
                                    android.os.Bundle extras) {
                                if (cancellationSignal.isCanceled()) {
                                    callback.onLayoutCancelled();
                                    return;
                                }
                                PrintDocumentInfo info = new PrintDocumentInfo.Builder(title)
                                        .setContentType(PrintDocumentInfo.CONTENT_TYPE_PHOTO)
                                        .setPageCount(1)
                                        .build();
                                callback.onLayoutFinished(info, true);
                            }

                            @Override
                            public void onWrite(
                                    android.print.PageRange[] pages,
                                    android.os.ParcelFileDescriptor destination,
                                    android.os.CancellationSignal cancellationSignal,
                                    WriteResultCallback callback) {
                                android.graphics.pdf.PdfDocument pdf =
                                        new android.graphics.pdf.PdfDocument();
                                try {
                                    android.graphics.pdf.PdfDocument.PageInfo pageInfo =
                                            new android.graphics.pdf.PdfDocument.PageInfo.Builder(
                                                    pageW, pageH, 1).create();
                                    android.graphics.pdf.PdfDocument.Page page = pdf.startPage(pageInfo);
                                    android.graphics.Canvas canvas = page.getCanvas();
                                    float margin = 36f;
                                    float maxW = pageW - 2f * margin;
                                    float maxH = pageH - 2f * margin;
                                    float scale = Math.min(maxW / bmp.getWidth(), maxH / bmp.getHeight());
                                    float w = bmp.getWidth() * scale;
                                    float h = bmp.getHeight() * scale;
                                    android.graphics.RectF dst = new android.graphics.RectF(
                                            margin, margin, margin + w, margin + h);
                                    canvas.drawBitmap(bmp, null, dst, null);
                                    pdf.finishPage(page);
                                    java.io.FileOutputStream out =
                                            new java.io.FileOutputStream(destination.getFileDescriptor());
                                    pdf.writeTo(out);
                                    callback.onWriteFinished(
                                            new android.print.PageRange[]{android.print.PageRange.ALL_PAGES});
                                } catch (Throwable e) {
                                    Log.e(LOG_TAG, "printPngDocument write failed", e);
                                    callback.onWriteFailed(e.getMessage());
                                } finally {
                                    pdf.close();
                                }
                            }
                        }, attrs);
                    } catch (Throwable e) {
                        Log.e(LOG_TAG, "printPngDocument failed", e);
                    }
                }
            });
            return true;
        } catch (Throwable e) {
            Log.e(LOG_TAG, "printPngDocument schedule failed", e);
            return false;
        }
    }

    /**
     * OS document slip, top-left on the paper picked in the print dialog:
     * {@link #printSlipPagesPlaced} with no place and no virtual paper.
     */
    public boolean printSlipPages(
            final String jobName, final byte[] pngBytes, final int contentUm, final boolean landscape) {
        return printSlipPagesPlaced(jobName, pngBytes, contentUm, landscape, 0, 0, 0);
    }

    /**
     * OS document slip on any paper (docs/PRINTER.md § Paper). The slip is
     * upright and {@code contentUm} micrometres wide on paper; the print
     * dialog picks the printer and its paper. Each page draws the slip at
     * its real size on the part of that paper the printer reaches (its own
     * minimum margins) and the virtual paper covers ({@code virtualWidthUm}
     * × {@code virtualLengthUm} micrometres; a length of 0 is a roll, a
     * width of 0 no virtual paper), at the format's place: {@code place} is
     * down × 3 + across (0 = top left). Nothing is fitted: a longer slip
     * goes on to more pages, split in the blank band between two lines; a
     * wider one is cut at the area's edge. Where every page goes is worked
     * out by {@link KumkajSlipLayout} — the same rule as the desktop apps.
     * {@code landscape} only turns the sheet.
     */
    public boolean printSlipPagesPlaced(
            final String jobName, final byte[] pngBytes, final int contentUm, final boolean landscape,
            final int place, final int virtualWidthUm, final int virtualLengthUm) {
        final byte[] body = pngBytes == null ? new byte[0] : pngBytes;
        final String title = (jobName == null || jobName.length() == 0) ? "Slip" : jobName;
        try {
            runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    try {
                        PrintManager pm = (PrintManager) getSystemService(PRINT_SERVICE);
                        if (pm == null) {
                            return;
                        }
                        final android.graphics.Bitmap bmp =
                                android.graphics.BitmapFactory.decodeByteArray(body, 0, body.length);
                        if (bmp == null || contentUm <= 0) {
                            Log.e(LOG_TAG, "printSlipPages: bad slip");
                            return;
                        }
                        final int bw = bmp.getWidth();
                        final int bh = bmp.getHeight();
                        // Blank bands between lines (a page may end in one) and
                        // the last row with ink (nothing after it is printed).
                        final java.util.ArrayList<Integer> gapList = new java.util.ArrayList<>();
                        int inkEnd = 0;
                        int gapStart = -1;
                        final int[] row = new int[bw];
                        for (int y = 0; y < bh; y++) {
                            bmp.getPixels(row, 0, bw, 0, y, bw, 1);
                            boolean ink = false;
                            for (int x = 0; x < bw; x++) {
                                int c = row[x];
                                int lum = (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000;
                                if ((c >>> 24) > 0 && lum < 200) {
                                    ink = true;
                                    break;
                                }
                            }
                            if (ink) {
                                if (gapStart >= 0) {
                                    gapList.add((gapStart + y) / 2);
                                    gapStart = -1;
                                }
                                inkEnd = y + 1;
                            } else if (gapStart < 0) {
                                gapStart = y;
                            }
                        }
                        final int[] gaps = new int[gapList.size()];
                        for (int i = 0; i < gaps.length; i++) {
                            gaps[i] = gapList.get(i);
                        }
                        final int len = Math.max(1, inkEnd);
                        // PDF points per slip pixel: the slip's own mm on paper.
                        final float ptPerPx = (contentUm / 25400f * 72f) / bw;
                        final float ptPerUm = 72f / 25400f;
                        // page width, height, left, top, right, bottom (points)
                        final float[] page = new float[6];
                        final KumkajSlipLayout[] laid = new KumkajSlipLayout[1];
                        PrintAttributes attrs = new PrintAttributes.Builder()
                                .setMediaSize(landscape
                                        ? PrintAttributes.MediaSize.UNKNOWN_LANDSCAPE
                                        : PrintAttributes.MediaSize.UNKNOWN_PORTRAIT)
                                .setColorMode(PrintAttributes.COLOR_MODE_MONOCHROME)
                                .build();
                        pm.print(title, new PrintDocumentAdapter() {
                            @Override
                            public void onLayout(
                                    PrintAttributes oldAttributes,
                                    PrintAttributes newAttributes,
                                    android.os.CancellationSignal cancellationSignal,
                                    LayoutResultCallback callback,
                                    android.os.Bundle extras) {
                                if (cancellationSignal.isCanceled()) {
                                    callback.onLayoutCancelled();
                                    return;
                                }
                                PrintAttributes.MediaSize media = newAttributes.getMediaSize();
                                if (media == null || media.getWidthMils() <= 0 || media.getHeightMils() <= 0) {
                                    callback.onLayoutFailed("No paper size");
                                    return;
                                }
                                PrintAttributes.Margins m = newAttributes.getMinMargins();
                                page[0] = media.getWidthMils() * 72f / 1000f;
                                page[1] = media.getHeightMils() * 72f / 1000f;
                                page[2] = m == null ? 0f : m.getLeftMils() * 72f / 1000f;
                                page[3] = m == null ? 0f : m.getTopMils() * 72f / 1000f;
                                page[4] = m == null ? 0f : m.getRightMils() * 72f / 1000f;
                                page[5] = m == null ? 0f : m.getBottomMils() * 72f / 1000f;
                                // The virtual paper as the slip lies on the
                                // dialog's page: turned with it for landscape.
                                // 0 that way = no limit (none set, or a roll's
                                // length).
                                float vAlong = (landscape ? virtualLengthUm : virtualWidthUm) * ptPerUm;
                                float vDown = (landscape ? virtualWidthUm : virtualLengthUm) * ptPerUm;
                                boolean roll = virtualWidthUm > 0 && virtualLengthUm <= 0;
                                laid[0] = KumkajSlipLayout.plan(
                                        bw, len, gaps,
                                        (page[0] - page[2] - page[4]) / ptPerPx,
                                        (page[1] - page[3] - page[5]) / ptPerPx,
                                        vAlong > 0f ? (vAlong - page[2] - page[4]) / ptPerPx : 0.0,
                                        vDown > 0f ? (vDown - page[3] - page[5]) / ptPerPx : 0.0,
                                        roll, place);
                                PrintDocumentInfo info = new PrintDocumentInfo.Builder(title)
                                        .setContentType(PrintDocumentInfo.CONTENT_TYPE_DOCUMENT)
                                        .setPageCount(laid[0].pages.length)
                                        .build();
                                callback.onLayoutFinished(info, true);
                            }

                            @Override
                            public void onWrite(
                                    android.print.PageRange[] pages,
                                    android.os.ParcelFileDescriptor destination,
                                    android.os.CancellationSignal cancellationSignal,
                                    WriteResultCallback callback) {
                                android.graphics.pdf.PdfDocument pdf =
                                        new android.graphics.pdf.PdfDocument();
                                try {
                                    final KumkajSlipLayout plan = laid[0];
                                    if (plan == null) {
                                        callback.onWriteFailed("No layout");
                                        return;
                                    }
                                    int pw = Math.max(1, Math.round(page[0]));
                                    int ph = Math.max(1, Math.round(page[1]));
                                    for (int i = 0; i < plan.pages.length; i++) {
                                        boolean wanted = false;
                                        for (android.print.PageRange r : pages) {
                                            if (i >= r.getStart() && i <= r.getEnd()) {
                                                wanted = true;
                                                break;
                                            }
                                        }
                                        if (!wanted) {
                                            continue;
                                        }
                                        if (cancellationSignal.isCanceled()) {
                                            callback.onWriteCancelled();
                                            return;
                                        }
                                        android.graphics.pdf.PdfDocument.Page p = pdf.startPage(
                                                new android.graphics.pdf.PdfDocument.PageInfo.Builder(pw, ph, i).create());
                                        android.graphics.Canvas canvas = p.getCanvas();
                                        int[] s = plan.pages[i];
                                        canvas.save();
                                        // Nothing lands outside the area: not on
                                        // the edges the printer cannot reach, not
                                        // past the virtual paper.
                                        canvas.clipRect(
                                                page[2], page[3],
                                                page[2] + plan.areaW * ptPerPx, page[3] + plan.areaH * ptPerPx);
                                        android.graphics.Rect src = new android.graphics.Rect(0, s[0], bw, s[1]);
                                        float left = page[2] + plan.x * ptPerPx;
                                        float top = page[3] + plan.y * ptPerPx;
                                        android.graphics.RectF dst = new android.graphics.RectF(
                                                left, top,
                                                left + bw * ptPerPx, top + (s[1] - s[0]) * ptPerPx);
                                        canvas.drawBitmap(bmp, src, dst, null);
                                        canvas.restore();
                                        pdf.finishPage(p);
                                    }
                                    java.io.FileOutputStream out =
                                            new java.io.FileOutputStream(destination.getFileDescriptor());
                                    pdf.writeTo(out);
                                    callback.onWriteFinished(pages);
                                } catch (Throwable e) {
                                    Log.e(LOG_TAG, "printSlipPages write failed", e);
                                    callback.onWriteFailed(e.getMessage());
                                } finally {
                                    pdf.close();
                                }
                            }
                        }, attrs);
                    } catch (Throwable e) {
                        Log.e(LOG_TAG, "printSlipPages failed", e);
                    }
                }
            });
            return true;
        } catch (Throwable e) {
            Log.e(LOG_TAG, "printSlipPages schedule failed", e);
            return false;
        }
    }

    public boolean printDocument(final String jobName, final String text) {
        final String body = text == null ? "" : text;
        final String title = (jobName == null || jobName.length() == 0) ? "Slip" : jobName;
        try {
            runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    try {
                        PrintManager pm = (PrintManager) getSystemService(PRINT_SERVICE);
                        if (pm == null) {
                            return;
                        }
                        PrintAttributes attrs = new PrintAttributes.Builder()
                                .setMediaSize(PrintAttributes.MediaSize.ISO_A4)
                                .setColorMode(PrintAttributes.COLOR_MODE_MONOCHROME)
                                .setMinMargins(PrintAttributes.Margins.NO_MARGINS)
                                .build();
                        pm.print(title, new PrintDocumentAdapter() {
                            @Override
                            public void onLayout(
                                    PrintAttributes oldAttributes,
                                    PrintAttributes newAttributes,
                                    android.os.CancellationSignal cancellationSignal,
                                    LayoutResultCallback callback,
                                    android.os.Bundle extras) {
                                if (cancellationSignal.isCanceled()) {
                                    callback.onLayoutCancelled();
                                    return;
                                }
                                PrintDocumentInfo info = new PrintDocumentInfo.Builder(title)
                                        .setContentType(PrintDocumentInfo.CONTENT_TYPE_DOCUMENT)
                                        .setPageCount(1)
                                        .build();
                                callback.onLayoutFinished(info, true);
                            }

                            @Override
                            public void onWrite(
                                    android.print.PageRange[] pages,
                                    android.os.ParcelFileDescriptor destination,
                                    android.os.CancellationSignal cancellationSignal,
                                    WriteResultCallback callback) {
                                android.graphics.pdf.PdfDocument pdf =
                                        new android.graphics.pdf.PdfDocument();
                                try {
                                    android.graphics.pdf.PdfDocument.PageInfo pageInfo =
                                            new android.graphics.pdf.PdfDocument.PageInfo.Builder(
                                                    595, 842, 1).create();
                                    android.graphics.pdf.PdfDocument.Page page = pdf.startPage(pageInfo);
                                    android.graphics.Canvas canvas = page.getCanvas();
                                    android.graphics.Paint paint = new android.graphics.Paint();
                                    paint.setColor(android.graphics.Color.BLACK);
                                    paint.setTextSize(11f);
                                    float y = 40f;
                                    float x = 40f;
                                    float maxWidth = 515f;
                                    for (String line : body.split("\n", -1)) {
                                        if (cancellationSignal.isCanceled()) {
                                            callback.onWriteCancelled();
                                            return;
                                        }
                                        if (line.length() == 0) {
                                            y += paint.getFontSpacing();
                                            continue;
                                        }
                                        int start = 0;
                                        while (start < line.length()) {
                                            int count = paint.breakText(
                                                    line, start, line.length(), true, maxWidth, null);
                                            canvas.drawText(line, start, start + count, x, y, paint);
                                            start += count;
                                            y += paint.getFontSpacing();
                                            if (y > 800f) {
                                                break;
                                            }
                                        }
                                        if (y > 800f) {
                                            break;
                                        }
                                    }
                                    pdf.finishPage(page);
                                    java.io.FileOutputStream out =
                                            new java.io.FileOutputStream(destination.getFileDescriptor());
                                    pdf.writeTo(out);
                                    callback.onWriteFinished(
                                            new android.print.PageRange[]{android.print.PageRange.ALL_PAGES});
                                } catch (Throwable e) {
                                    Log.e(LOG_TAG, "printDocument write failed", e);
                                    callback.onWriteFailed(e.getMessage());
                                } finally {
                                    pdf.close();
                                }
                            }
                        }, attrs);
                    } catch (Throwable e) {
                        Log.e(LOG_TAG, "printDocument failed", e);
                    }
                }
            });
            return true;
        } catch (Throwable e) {
            Log.e(LOG_TAG, "printDocument schedule failed", e);
            return false;
        }
    }
