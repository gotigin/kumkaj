/*! Kumkaj stage picker — Phase 1 stub: always still (no wasm). */
(function () {
  "use strict";
  window.KumkajStage = {
    mode: "still",
    pick: function () {
      return "still";
    },
  };
  document.documentElement.dataset.stageMode = "still";
})();
