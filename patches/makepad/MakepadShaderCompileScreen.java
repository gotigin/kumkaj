package dev.makepad.android;

import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Typeface;
import android.graphics.drawable.Drawable;
import android.text.Layout;
import android.text.SpannableString;
import android.text.Spanned;
import android.text.style.ReplacementSpan;
import android.util.DisplayMetrics;
import android.util.TypedValue;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.ScrollView;
import android.widget.TextView;

/**
 * Cover content while the first frame waits on uncached GL programs.
 *
 * Port of Makepad PR #1432 ({@code MakepadShaderCompileScreen}), adapted to
 * look up strings/colors by name (no {@code R.*} constants — AAB R.id mismatch
 * on API 26, see {@link MakepadLaunchBg}).
 *
 * Drop this file when {@code MAKEPAD_REV} includes merged
 * https://github.com/makepad/makepad/pull/1432 (and #1453 / #1459).
 */
final class MakepadShaderCompileScreen extends ScrollView {
    private final TextView mTitle;
    private final TextView mNote;
    private final ProgressBar mSpinner;

    MakepadShaderCompileScreen(Context context) {
        super(context);
        setFillViewport(true);
        setFocusable(false);
        LinearLayout content = new LinearLayout(context);
        content.setOrientation(LinearLayout.VERTICAL);
        content.setGravity(Gravity.CENTER);
        DisplayMetrics metrics = getResources().getDisplayMetrics();
        int padding = Math.round(24 * metrics.density);
        content.setPadding(padding, padding, padding, padding);

        int iconId = resId(context, "drawable", "makepad_shader_compile_icon");
        int launcherIconId = context.getApplicationInfo().icon;
        Drawable icon = null;
        if (iconId != 0) {
            icon = context.getDrawable(iconId);
        } else if (launcherIconId != 0) {
            icon = getResources().getDrawableForDensity(
                launcherIconId, DisplayMetrics.DENSITY_XXXHIGH, context.getTheme());
        }
        if (icon != null) {
            ImageView iconView = new ImageView(context);
            iconView.setImageDrawable(icon);
            iconView.setImportantForAccessibility(IMPORTANT_FOR_ACCESSIBILITY_NO);
            int iconSize = Math.round(96 * metrics.density);
            LinearLayout.LayoutParams iconParams = new LinearLayout.LayoutParams(iconSize, iconSize);
            iconParams.bottomMargin = Math.round(40 * metrics.density);
            content.addView(iconView, iconParams);
        }

        final int spinnerSize = Math.round(TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_SP, 20, metrics));
        final int spinnerGap = Math.round(TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_SP, 12, metrics));
        String titleText = stringRes(context, "makepad_shader_compile_title",
            "Compiling shaders (for UI styling)\u2060…");
        SpannableString title = new SpannableString("\u00A0" + titleText);
        title.setSpan(new ReplacementSpan() {
            @Override
            public int getSize(Paint paint, CharSequence text, int start, int end, Paint.FontMetricsInt fm) {
                if (fm != null) {
                    paint.getFontMetricsInt(fm);
                }
                return spinnerSize + spinnerGap;
            }

            @Override
            public void draw(Canvas canvas, CharSequence text, int start, int end,
                float x, int top, int y, int bottom, Paint paint) {
            }
        }, 0, 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
        mTitle = new TextView(context);
        mTitle.setText(title);
        mTitle.setTextSize(TypedValue.COMPLEX_UNIT_SP, 16);
        mTitle.setTypeface(Typeface.create("sans-serif-medium", Typeface.NORMAL));
        mTitle.setGravity(Gravity.CENTER_HORIZONTAL);
        int titleLineHeight = Math.round(TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_SP, 24, metrics));
        mTitle.setLineSpacing(titleLineHeight - mTitle.getPaint().getFontMetricsInt(null), 1f);
        mSpinner = new ProgressBar(context);
        mSpinner.setImportantForAccessibility(IMPORTANT_FOR_ACCESSIBILITY_NO);
        mTitle.addOnLayoutChangeListener(new View.OnLayoutChangeListener() {
            @Override
            public void onLayoutChange(View view, int left, int top, int right, int bottom,
                int oldLeft, int oldTop, int oldRight, int oldBottom) {
                Layout layout = mTitle.getLayout();
                if (layout == null) {
                    return;
                }
                Paint.FontMetrics fontMetrics = mTitle.getPaint().getFontMetrics();
                float textCenterY = layout.getLineBaseline(0)
                    + (fontMetrics.ascent + fontMetrics.descent) / 2;
                float gapStartX = layout.getPrimaryHorizontal(0);
                if (layout.getParagraphDirection(0) == Layout.DIR_RIGHT_TO_LEFT) {
                    gapStartX -= spinnerSize;
                }
                mSpinner.setTranslationX(left + mTitle.getTotalPaddingLeft() + gapStartX);
                mSpinner.setTranslationY(top + mTitle.getTotalPaddingTop() + textCenterY - spinnerSize / 2f);
            }
        });
        FrameLayout titleRow = new FrameLayout(context);
        titleRow.addView(mTitle, new FrameLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        titleRow.addView(mSpinner, new FrameLayout.LayoutParams(
            spinnerSize, spinnerSize, Gravity.TOP | Gravity.LEFT));
        content.addView(titleRow, new LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));

        mNote = new TextView(context);
        mNote.setText(stringRes(context, "makepad_shader_compile_note",
            "This is a one-time process that is only necessary the first time you run this app."));
        mNote.setTextSize(TypedValue.COMPLEX_UNIT_SP, 14);
        mNote.setGravity(Gravity.CENTER_HORIZONTAL);
        mNote.setMaxWidth(Math.round(288 * metrics.density));
        int noteLineHeight = Math.round(TypedValue.applyDimension(
            TypedValue.COMPLEX_UNIT_SP, 20, metrics));
        mNote.setLineSpacing(noteLineHeight - mNote.getPaint().getFontMetricsInt(null), 1f);
        LinearLayout.LayoutParams noteParams = new LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        noteParams.topMargin = Math.round(8 * metrics.density);
        content.addView(mNote, noteParams);
        addView(content, new ScrollView.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
    }

    void applyColors(int backgroundColor) {
        float luma = (0.2126f * Color.red(backgroundColor)
            + 0.7152f * Color.green(backgroundColor)
            + 0.0722f * Color.blue(backgroundColor)) / 255f;
        boolean lightBackground = luma > 0.5f;
        Context context = getContext();
        mTitle.setTextColor(colorRes(context,
            lightBackground ? "makepad_shader_compile_title_light" : "makepad_shader_compile_title_dark",
            lightBackground ? 0xFF1F2328 : 0xFFF2F2F2));
        mNote.setTextColor(colorRes(context,
            lightBackground ? "makepad_shader_compile_note_light" : "makepad_shader_compile_note_dark",
            lightBackground ? 0xFF59636E : 0xFFCCCCCC));
        mSpinner.setIndeterminateTintList(ColorStateList.valueOf(colorRes(context,
            lightBackground ? "makepad_shader_compile_spinner_light" : "makepad_shader_compile_spinner_dark",
            lightBackground ? 0xFF1F2328 : 0xFFF2F2F2)));
    }

    String accessibilityAnnouncement(Context context) {
        return stringRes(context, "makepad_shader_compile_title", "Compiling shaders…")
            + " "
            + stringRes(context, "makepad_shader_compile_note",
                "This is a one-time process that is only necessary the first time you run this app.");
    }

    private static int resId(Context context, String type, String name) {
        return context.getResources().getIdentifier(name, type, context.getPackageName());
    }

    private static String stringRes(Context context, String name, String fallback) {
        int id = resId(context, "string", name);
        if (id != 0) {
            return context.getString(id);
        }
        return fallback;
    }

    private static int colorRes(Context context, String name, int fallback) {
        int id = resId(context, "color", name);
        if (id != 0) {
            return context.getColor(id);
        }
        return fallback;
    }
}
