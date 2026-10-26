    // Kumkaj: a device that other devices depend on (it serves the shop's
    // books) keeps its screen on, so Android does not doze it and drop them.
    // Inserted before copyToClipboard by scripts/apply-android-activity-bridge.sh.
    public void keepScreenOn(final boolean on) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    if (on) {
                        getWindow().addFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                    } else {
                        getWindow().clearFlags(android.view.WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                    }
                } catch (Throwable e) {
                    Log.e(LOG_TAG, "keepScreenOn failed", e);
                }
            }
        });
    }

