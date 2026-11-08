package dev.makepad.android;

import android.app.Activity;
import android.graphics.Rect;
import android.graphics.drawable.ColorDrawable;
import android.util.Log;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.view.ViewTreeObserver;
import android.view.WindowManager;
import android.widget.PopupWindow;

/**
 * Measures the soft keyboard on API 26–29.
 *
 * The activity runs {@code adjustNothing}. Below API 30 WindowManager then
 * sets the activity window's visible frame equal to its content frame, so
 * neither its insets nor {@code getWindowVisibleDisplayFrame()} ever change
 * when the keyboard opens (measured on Huawei AGS2-W09 / API 26: app window
 * {@code visible=[0,48][1920,1133]} with SwiftKey covering y 469..1133).
 * Without this probe native code never sees the keyboard at all.
 *
 * A 1 px wide, transparent, untouchable panel attached to the activity
 * window in {@code adjustResize} mode IS resized by WindowManager: its frame
 * ends at the keyboard's top edge. Its global layout fires on every show and
 * hide, so the overlap reported here is a measurement, never an estimate.
 * {@code INPUT_METHOD_NEEDED} keeps the keyboard layered above the panel,
 * which is what makes WindowManager resize it. The panel does not take focus,
 * touches or the input connection — the Makepad surface keeps all three.
 *
 * API 30+ reads {@code WindowInsets.Type.ime()} directly and never creates it.
 */
final class MakepadImeProbe implements ViewTreeObserver.OnGlobalLayoutListener {
    private final View surface;
    private final View panel;
    private final PopupWindow popup;
    private final Rect frame = new Rect();
    private final int[] surfaceLoc = new int[2];

    // SOFT_INPUT_ADJUST_RESIZE is deprecated from API 30; this runs below 30.
    @SuppressWarnings("deprecation")
    MakepadImeProbe(Activity activity, View surface) {
        this.surface = surface;
        panel = new View(activity);
        panel.setLayoutParams(new ViewGroup.LayoutParams(1, ViewGroup.LayoutParams.MATCH_PARENT));
        popup = new PopupWindow(panel, 1, WindowManager.LayoutParams.MATCH_PARENT, false);
        popup.setBackgroundDrawable(new ColorDrawable(0));
        popup.setTouchable(false);
        popup.setOutsideTouchable(false);
        // Keep clipping on: setClippingEnabled(false) adds
        // FLAG_LAYOUT_NO_LIMITS, and WindowManager then lays the panel out
        // against a ±10000 frame that never shrinks for the keyboard
        // (measured: panel stayed [0,0][1,1200] with SwiftKey up).
        popup.setAnimationStyle(0);
        popup.setSoftInputMode(
            WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
                | WindowManager.LayoutParams.SOFT_INPUT_STATE_UNCHANGED
        );
        popup.setInputMethodMode(PopupWindow.INPUT_METHOD_NEEDED);
        panel.getViewTreeObserver().addOnGlobalLayoutListener(this);
    }

    /** Show the panel once the surface is attached to the activity window. */
    void attach() {
        surface.post(new Runnable() {
            @Override
            public void run() {
                if (popup.isShowing() || surface.getWindowToken() == null) {
                    return;
                }
                try {
                    popup.showAtLocation(surface, Gravity.NO_GRAVITY, 0, 0);
                } catch (RuntimeException e) {
                    // Window not ready (activity finishing). The fallback is
                    // the old behaviour: no keyboard height, no lift.
                    Log.w("MakepadIME", "ime probe: showAtLocation failed", e);
                }
            }
        });
    }

    void detach() {
        panel.getViewTreeObserver().removeOnGlobalLayoutListener(this);
        if (popup.isShowing()) {
            popup.dismiss();
        }
        MakepadImeInsets.probeOverlapPx = 0;
    }

    @Override
    public void onGlobalLayout() {
        if (panel.getWindowToken() == null || surface.getHeight() <= 0) {
            return;
        }
        panel.getWindowVisibleDisplayFrame(frame);
        if (frame.isEmpty()) {
            return;
        }
        surface.getLocationOnScreen(surfaceLoc);
        int surfaceBottom = surfaceLoc[1] + surface.getHeight();
        // getWindowVisibleDisplayFrame asks WindowManager for the panel's
        // current frame, which is fresher than the panel's last laid-out
        // height while the keyboard is still settling.
        int overlap = Math.max(0, surfaceBottom - frame.bottom);
        MakepadImeInsets.probeOverlapPx = overlap;
        MakepadImeInsets.report(surface, surface.getRootWindowInsets(), "imeProbe");
    }
}
