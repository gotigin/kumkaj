/**
 * Site copy on the HTML pages (account, shop, …) in the visitor's language.
 *
 * The words come from the same packs the Makepad page uses: the site's
 * `src/locale.rs` EN (English, inline in the HTML) and `translations/<code>.tsv`,
 * staged as `/i18n/<code>.tsv`. Mark copy with
 *   data-i18n="key"                — element text
 *   data-i18n-placeholder="key"    — input placeholder
 *   data-i18n-aria-label="key"     — aria-label
 *   data-i18n-title="key"          — title attribute
 * The element's own English is the fallback (kept in data-i18n-en). Scripts
 * use `window.siteT(key, english)` for text they write, and listen for
 * "site:i18n" to repaint it. Repaints on load and on "gotigin:lang"
 * (lang-nav.js). `cargo test -p <site>-site` checks every key is in EN and
 * the inline English matches it.
 * Source: sites/_shared/chrome/site-i18n.js — copied to each site by
 * scripts/sync-site-i18n.sh; do not edit the copies.
 */
(function () {
  "use strict";

  var STAND_IN = "@@EN@@ ";
  var packs = {}; // table code → { key: text }
  var shown = "en";

  function tableOf(code) {
    switch (code) {
      case "pt-BR": return "pt";
      case "es-MX": return "es";
      case "yue": return "zh-Hant";
      case "gom": return "kok";
      default: return code;
    }
  }

  function unescape(v) {
    return v.replace(/\\(n|t|\\)/g, function (_, c) {
      return c === "n" ? "\n" : c === "t" ? "\t" : "\\";
    });
  }

  function parse(text) {
    var out = {};
    if (!text || /^\s*</.test(text)) return out; // an HTML fallback page is not a pack
    text.split("\n").forEach(function (line) {
      if (!line || line.charAt(0) === "#") return;
      var tab = line.indexOf("\t");
      if (tab < 1) return;
      var value = line.slice(tab + 1).replace(/\r$/, "");
      if (value.indexOf(STAND_IN) === 0) return; // untranslated row → English
      out[line.slice(0, tab).trim()] = unescape(value);
    });
    return out;
  }

  function load(code) {
    var table = tableOf(code);
    if (code === "en" || packs[table]) return Promise.resolve(packs[table] || {});
    return fetch("/i18n/" + encodeURIComponent(table) + ".tsv", { cache: "default" })
      .then(function (r) { return r.ok ? r.text() : ""; })
      .then(function (t) { packs[table] = parse(t); return packs[table]; })
      .catch(function () { packs[table] = {}; return packs[table]; });
  }

  /** Text for `key` in the shown language, else `english`. */
  function siteT(key, english) {
    var pack = shown === "en" ? null : packs[tableOf(shown)];
    return (pack && pack[key]) || english;
  }

  var ATTRS = [
    ["data-i18n-placeholder", "placeholder"],
    ["data-i18n-aria-label", "aria-label"],
    ["data-i18n-title", "title"]
  ];

  function paint() {
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      if (!el.hasAttribute("data-i18n-en")) el.setAttribute("data-i18n-en", el.textContent);
      var text = siteT(el.getAttribute("data-i18n"), el.getAttribute("data-i18n-en"));
      if (el.textContent !== text) el.textContent = text;
    });
    ATTRS.forEach(function (pair) {
      document.querySelectorAll("[" + pair[0] + "]").forEach(function (el) {
        var keep = pair[0] + "-en";
        if (!el.hasAttribute(keep)) el.setAttribute(keep, el.getAttribute(pair[1]) || "");
        el.setAttribute(pair[1], siteT(el.getAttribute(pair[0]), el.getAttribute(keep)));
      });
    });
    try {
      window.dispatchEvent(new CustomEvent("site:i18n", { detail: { lang: shown } }));
    } catch (_) {}
  }

  function show(code) {
    var wanted = code || "en";
    return load(wanted).then(function () {
      shown = wanted;
      paint();
    });
  }

  window.siteT = siteT;
  window.__SITE_I18N__ = { show: show, paint: paint };

  function start() {
    var api = window.__SITE_LANG__;
    show(api ? api.current() : document.documentElement.lang || "en");
    window.addEventListener("gotigin:lang", function (ev) {
      show(ev && ev.detail && ev.detail.lang);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
