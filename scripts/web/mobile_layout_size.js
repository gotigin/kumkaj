#!/usr/bin/env node
/**
 * CSS layout size for Makepad full-page canvases on touch devices.
 *
 * Android Chrome "Desktop site" sets innerWidth/Height to a fake desk size
 * (~980×1900) while outer/screen stay at the phone chrome (~393×876). Laying
 * out at innerWidth makes the site unusable. Pinch-zoom shrinks
 * visualViewport.width (e.g. scale 5 → ~78px) — never use that for layout.
 *
 * Kept in sync with patches/makepad/web_mobile_layout_width.rs.patch
 * (vendor …/os/web/web.js → css_layout_size_for_canvas).
 * check-web-mobile-layout.sh runs the tests below.
 */

function css_layout_size_for_canvas(
  inner_w,
  inner_h,
  outer_w,
  outer_h,
  screen_w,
  screen_h,
  is_touch
) {
  let w = inner_w;
  let h = inner_h;
  if (is_touch) {
    const ws = [inner_w, outer_w, screen_w].filter((x) => x > 0);
    const hs = [inner_h, outer_h, screen_h].filter((x) => x > 0);
    if (ws.length) w = Math.min(...ws);
    if (hs.length) h = Math.min(...hs);
  }
  return { w, h };
}

function run_tests() {
  const assert = (cond, msg) => {
    if (!cond) throw new Error(msg);
  };
  const eq = (a, b, msg) =>
    assert(
      a.w === b.w && a.h === b.h,
      `${msg}: got ${JSON.stringify(a)} want ${JSON.stringify(b)}`
    );

  // Phone, normal mobile mode.
  eq(
    css_layout_size_for_canvas(393, 700, 393, 876, 393, 876, true),
    { w: 393, h: 700 },
    "normal mobile uses shrunk inner height (keyboard)"
  );

  // Android Chrome Desktop site — the live khaynjo.com failure (2026-11-21).
  eq(
    css_layout_size_for_canvas(980, 1906, 393, 876, 393, 876, true),
    { w: 393, h: 876 },
    "desktop-site UA must clamp to outer/screen"
  );

  // Do not follow a pinch-zoomed visualViewport width (caller must pass
  // outer/screen, not vv.width=78).
  eq(
    css_layout_size_for_canvas(980, 1906, 393, 876, 393, 876, true),
    { w: 393, h: 876 },
    "still clamped when page was zoomed (layout inputs stay outer/screen)"
  );

  // Desktop / non-touch: never clamp (ultrawide outer can exceed inner).
  eq(
    css_layout_size_for_canvas(1600, 900, 1680, 1050, 2560, 1440, false),
    { w: 1600, h: 900 },
    "non-touch keeps innerWidth"
  );

  // Touch laptop: outer usually ≥ inner → min is inner.
  eq(
    css_layout_size_for_canvas(1280, 800, 1280, 860, 1920, 1200, true),
    { w: 1280, h: 800 },
    "touch laptop keeps inner when outer is not smaller"
  );

  // Missing outer (0) still clamps via screen.
  eq(
    css_layout_size_for_canvas(980, 1906, 0, 0, 393, 876, true),
    { w: 393, h: 876 },
    "screen alone clamps desktop-site"
  );

  console.log("mobile_layout_size.js: all tests passed");
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = { css_layout_size_for_canvas, run_tests };
}

if (require.main === module) {
  run_tests();
}
