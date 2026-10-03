    // Kumkaj: receipt share sheet and Storage Access Framework reads.
    // Inserted before copyToClipboard by scripts/apply-android-activity-bridge.sh.
    public void shareText(final String text) {
        final String body = text == null ? "" : text;
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    Intent send = new Intent(Intent.ACTION_SEND);
                    send.setType("text/plain");
                    send.putExtra(Intent.EXTRA_TEXT, body);
                    startActivity(Intent.createChooser(send, "Share receipt"));
                } catch (Throwable e) {
                    Log.e(LOG_TAG, "shareText failed", e);
                }
            }
        });
    }

    public byte[] readContentUri(String uri) {
        if (uri == null || !uri.startsWith("content://")) {
            return null;
        }
        InputStream in = null;
        try {
            in = getContentResolver().openInputStream(Uri.parse(uri));
            if (in == null) {
                return null;
            }
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            byte[] buf = new byte[8192];
            int n;
            while ((n = in.read(buf)) >= 0) {
                out.write(buf, 0, n);
            }
            return out.toByteArray();
        } catch (Throwable e) {
            Log.e(LOG_TAG, "readContentUri failed", e);
            return null;
        } finally {
            if (in != null) {
                try {
                    in.close();
                } catch (Throwable ignored) {
                }
            }
        }
    }

