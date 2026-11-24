// Kumkaj bridge of Makepad PR #1432 — fragments for apply-android-activity-bridge.sh.
// DROP when MAKEPAD_REV / makepad-full already ships showShaderCompileScreen + cold-cache UX.
// Not a complete class: fields go with other Activity fields; methods near cover helpers.

    // --- Kumkaj: Makepad PR #1432 compile-screen bridge (drop when upstream lands) ---
    // Cover waits for the first GL present so Mali never flashes an uninitialized
    // black SurfaceView. Rust calls showShaderCompileScreen() only when the GL
    // program cache is cold (empty for this device fingerprint).
    private boolean mIsCoverWaitingForFirstFrame = false;
    private MakepadShaderCompileScreen mShaderCompileScreen;
    private int mAppBackgroundColor = MakepadLaunchBg.FALLBACK_COLOR;
    // --- end Kumkaj PR #1432 fields ---

    // --- Kumkaj: Makepad PR #1432 compile-screen methods (drop when upstream lands) ---
    // Rust: cold GL program cache → show the compile note on the cream cover.
    public void showShaderCompileScreen() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (!mIsCoverWaitingForFirstFrame || mSurfaceCoverOverlay == null
                    || mShaderCompileScreen != null) {
                    return;
                }
                mShaderCompileScreen = new MakepadShaderCompileScreen(MakepadActivity.this);
                mShaderCompileScreen.applyColors(mAppBackgroundColor);
                mSurfaceCoverOverlay.addView(mShaderCompileScreen, new FrameLayout.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
                mShaderCompileScreen.announceForAccessibility(
                    mShaderCompileScreen.accessibilityAnnouncement(MakepadActivity.this));
            }
        });
    }

    private void removeShaderCompileScreen() {
        final MakepadShaderCompileScreen compileScreen = mShaderCompileScreen;
        if (compileScreen == null) {
            return;
        }
        mShaderCompileScreen = null;
        compileScreen.animate()
            .alpha(0.0f)
            .setDuration(SURFACE_COVER_FADE_OUT_MS)
            .withEndAction(new Runnable() {
                @Override
                public void run() {
                    ViewGroup parent = (ViewGroup) compileScreen.getParent();
                    if (parent != null) {
                        parent.removeView(compileScreen);
                    }
                }
            });
    }
    // --- end Kumkaj PR #1432 methods ---
