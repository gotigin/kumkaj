    // Kumkaj: PrintManager + Bluetooth SPP + USB host ESC/POS.
    // Inserted before copyToClipboard by scripts/apply-android-activity-bridge.sh.

    private static final UUID KUMKAJ_SPP_UUID =
            UUID.fromString("00001101-0000-1000-8000-00805F9B34FB");

    public String[] listBluetoothPrinters() {
        try {
            BluetoothAdapter adapter = BluetoothAdapter.getDefaultAdapter();
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
            BluetoothAdapter adapter = BluetoothAdapter.getDefaultAdapter();
            if (adapter == null) {
                return false;
            }
            BluetoothDevice device = adapter.getRemoteDevice(addr.toUpperCase(java.util.Locale.US));
            socket = device.createRfcommSocketToServiceRecord(KUMKAJ_SPP_UUID);
            adapter.cancelDiscovery();
            socket.connect();
            java.io.OutputStream out = socket.getOutputStream();
            out.write(data);
            out.flush();
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
