/* Android Chrome Desktop-site / pinch-zoom guard for Makepad sites.
 * Hardens the viewport meta and clamps html/body before wasm lays out.
 * Must stay aligned with scripts/web/mobile_layout_size.js. */
(function () {
  try {
    var meta = document.querySelector('meta[name="viewport"]');
    var content =
      "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover";
    if (meta) meta.setAttribute("content", content);
    else {
      meta = document.createElement("meta");
      meta.name = "viewport";
      meta.content = content;
      document.head.appendChild(meta);
    }
    var touch = "ontouchstart" in window || (navigator.maxTouchPoints || 0) > 0;
    if (!touch) return;
    function clamp() {
      var ws = [window.innerWidth, window.outerWidth, screen.width].filter(function (x) {
        return x > 0;
      });
      var hs = [window.innerHeight, window.outerHeight, screen.height].filter(function (x) {
        return x > 0;
      });
      if (!ws.length || !hs.length) return;
      var w = Math.min.apply(null, ws);
      var h = Math.min.apply(null, hs);
      document.documentElement.style.width = w + "px";
      document.body && (document.body.style.width = w + "px");
      document.documentElement.style.height = h + "px";
      document.body && (document.body.style.height = h + "px");
    }
    clamp();
    window.addEventListener("resize", clamp);
    window.addEventListener("orientationchange", clamp);
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", clamp);
    }
  } catch (_e) {}
})();
