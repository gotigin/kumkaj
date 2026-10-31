package dev.makepad.android;

import android.os.Build;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsAnimation;

import java.util.List;

/**
 * API 30+ IME inset animation callback, in its own class so API 26–29 never
 * loads {@link WindowInsetsAnimation} during verification of {@code ResizingLayout}.
 * Attach only via {@link #attachIfSupported(View)} (no-op below API 30).
 */
public final class ImeInsetsAnimationHelper {
    private ImeInsetsAnimationHelper() {}

    public static void attachIfSupported(View view) {
        if (Build.VERSION.SDK_INT < 30 || view == null) {
            return;
        }
        view.setWindowInsetsAnimationCallback(
            new WindowInsetsAnimation.Callback(
                WindowInsetsAnimation.Callback.DISPATCH_MODE_CONTINUE_ON_SUBTREE
            ) {
                @Override
                public void onPrepare(WindowInsetsAnimation animation) {
                    if ((animation.getTypeMask() & WindowInsets.Type.ime()) != 0) {
                        MakepadImeInsets.imeAnimationInProgress = true;
                    }
                }

                @Override
                public WindowInsets onProgress(
                    WindowInsets insets,
                    List<WindowInsetsAnimation> runningAnimations
                ) {
                    MakepadImeInsets.report(view, insets, "onProgress");
                    return insets;
                }

                @Override
                public void onEnd(WindowInsetsAnimation animation) {
                    if ((animation.getTypeMask() & WindowInsets.Type.ime()) != 0) {
                        MakepadImeInsets.imeAnimationInProgress = false;
                    }
                    WindowInsets insets = view.getRootWindowInsets();
                    if (insets == null) {
                        return;
                    }
                    MakepadImeInsets.report(view, insets, "onEnd");
                }
            }
        );
    }
}
