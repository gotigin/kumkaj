    // Kumkaj: write a file the save-file picker created (a backup copy saved
    // to a USB stick, an SD card or Downloads), replacing what was there.
    // Inserted before copyToClipboard by scripts/apply-android-activity-bridge.sh.
    public boolean writeContentUri(String uri, byte[] data) {
        if (uri == null || data == null || !uri.startsWith("content://")) {
            return false;
        }
        java.io.OutputStream out = null;
        try {
            out = getContentResolver().openOutputStream(Uri.parse(uri), "wt");
            if (out == null) {
                return false;
            }
            out.write(data);
            out.flush();
            return true;
        } catch (Throwable e) {
            Log.e(LOG_TAG, "writeContentUri failed", e);
            return false;
        } finally {
            if (out != null) {
                try {
                    out.close();
                } catch (Throwable ignored) {
                }
            }
        }
    }
