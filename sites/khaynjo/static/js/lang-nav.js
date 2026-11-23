/**
 * Site language button — full Kumkaj 115-language catalogue (same as apps).
 *
 * Picking a language really changes the page (docs/WEBSITES.md § Language):
 * - stores it (localStorage "gotigin.site.lang") and sets html lang + dir;
 * - writes `#l=<code>` into the URL hash (other segments such as `auth=`
 *   stay) and fires hashchange: the Makepad wasm reads `l=` and fetches
 *   `/i18n/<code>.tsv` (no web-sys on the wasm side);
 * - dispatches "gotigin:lang" ({ lang, meta }): site-i18n.js repaints the
 *   HTML pages (account, shop) from the same packs.
 * A link that already carries `#l=hi` wins over the stored choice.
 * Source: sites/_shared/chrome/lang-nav.js — copied to each site by
 * scripts/sync-site-i18n.sh; do not edit the copies.
 */
(function () {
  "use strict";

  var KEY = "gotigin.site.lang";
  var LANGS = [{"code": "en", "en": "English", "label": "English", "region": "World"}, {"code": "hi", "en": "Hindi", "label": "हिन्दी", "region": "India"}, {"code": "bn", "en": "Bengali", "label": "বাংলা", "region": "India"}, {"code": "te", "en": "Telugu", "label": "తెలుగు", "region": "India"}, {"code": "mr", "en": "Marathi", "label": "मराठी", "region": "India"}, {"code": "ta", "en": "Tamil", "label": "தமிழ்", "region": "India"}, {"code": "ur", "en": "Urdu", "label": "اردو", "region": "India"}, {"code": "gu", "en": "Gujarati", "label": "ગુજરાતી", "region": "India"}, {"code": "kn", "en": "Kannada", "label": "ಕನ್ನಡ", "region": "India"}, {"code": "ml", "en": "Malayalam", "label": "മലയാളം", "region": "India"}, {"code": "or", "en": "Odia", "label": "ଓଡ଼ିଆ", "region": "India"}, {"code": "pa", "en": "Punjabi", "label": "ਪੰਜਾਬੀ", "region": "India"}, {"code": "as", "en": "Assamese", "label": "অসমীয়া", "region": "India"}, {"code": "mai", "en": "Maithili", "label": "मैथिली", "region": "India"}, {"code": "sa", "en": "Sanskrit", "label": "संस्कृतम्", "region": "India"}, {"code": "ks", "en": "Kashmiri", "label": "كٲشُر", "region": "India"}, {"code": "ne", "en": "Nepali", "label": "नेपाली", "region": "India"}, {"code": "sd", "en": "Sindhi", "label": "سنڌي", "region": "India"}, {"code": "kok", "en": "Konkani", "label": "कोंकणी", "region": "India"}, {"code": "doi", "en": "Dogri", "label": "डोगरी", "region": "India"}, {"code": "mni", "en": "Manipuri", "label": "ꯃꯤꯇꯩꯂꯣꯟ", "region": "India"}, {"code": "sat", "en": "Santali", "label": "ᱥᱟᱱᱛᱟᱲᱤ", "region": "India"}, {"code": "brx", "en": "Bodo", "label": "बर'", "region": "India"}, {"code": "bho", "en": "Bhojpuri", "label": "भोजपुरी", "region": "India"}, {"code": "awa", "en": "Awadhi", "label": "अवधी", "region": "India"}, {"code": "mag", "en": "Magahi", "label": "मगही", "region": "India"}, {"code": "hne", "en": "Chhattisgarhi", "label": "छत्तीसगढ़ी", "region": "India"}, {"code": "raj", "en": "Rajasthani", "label": "राजस्थानी", "region": "India"}, {"code": "tcy", "en": "Tulu", "label": "ತುಳು", "region": "India"}, {"code": "trp", "en": "Kokborok", "label": "ককবরক", "region": "India"}, {"code": "lus", "en": "Mizo", "label": "Mizo ṭawng", "region": "India"}, {"code": "kha", "en": "Khasi", "label": "Khasi", "region": "India"}, {"code": "gom", "en": "Goan Konkani", "label": "गोंयची कोंकणी", "region": "India"}, {"code": "bo", "en": "Tibetan", "label": "བོད་སྐད་", "region": "India"}, {"code": "si", "en": "Sinhala", "label": "සිංහල", "region": "South Asia"}, {"code": "dv", "en": "Dhivehi", "label": "ދިވެހި", "region": "South Asia"}, {"code": "ps", "en": "Pashto", "label": "پښتو", "region": "South Asia"}, {"code": "fa", "en": "Persian", "label": "فارسی", "region": "Middle East"}, {"code": "zh", "en": "Chinese (Simplified)", "label": "中文", "region": "East Asia"}, {"code": "zh-Hant", "en": "Chinese (Traditional)", "label": "繁體中文", "region": "East Asia"}, {"code": "yue", "en": "Cantonese", "label": "粵語", "region": "East Asia"}, {"code": "ja", "en": "Japanese", "label": "日本語", "region": "East Asia"}, {"code": "ko", "en": "Korean", "label": "한국어", "region": "East Asia"}, {"code": "vi", "en": "Vietnamese", "label": "Tiếng Việt", "region": "Southeast Asia"}, {"code": "th", "en": "Thai", "label": "ไทย", "region": "Southeast Asia"}, {"code": "id", "en": "Indonesian", "label": "Bahasa Indonesia", "region": "Southeast Asia"}, {"code": "ms", "en": "Malay", "label": "Bahasa Melayu", "region": "Southeast Asia"}, {"code": "fil", "en": "Filipino", "label": "Filipino", "region": "Southeast Asia"}, {"code": "my", "en": "Burmese", "label": "မြန်မာ", "region": "Southeast Asia"}, {"code": "km", "en": "Khmer", "label": "ខ្មែរ", "region": "Southeast Asia"}, {"code": "lo", "en": "Lao", "label": "ລາວ", "region": "Southeast Asia"}, {"code": "ar", "en": "Arabic", "label": "العربية", "region": "Middle East"}, {"code": "he", "en": "Hebrew", "label": "עברית", "region": "Middle East"}, {"code": "tr", "en": "Turkish", "label": "Türkçe", "region": "Middle East"}, {"code": "az", "en": "Azerbaijani", "label": "Azərbaycan", "region": "Central Asia"}, {"code": "kk", "en": "Kazakh", "label": "Қазақша", "region": "Central Asia"}, {"code": "uz", "en": "Uzbek", "label": "Oʻzbekcha", "region": "Central Asia"}, {"code": "ky", "en": "Kyrgyz", "label": "Кыргызча", "region": "Central Asia"}, {"code": "tg", "en": "Tajik", "label": "Тоҷикӣ", "region": "Central Asia"}, {"code": "tk", "en": "Turkmen", "label": "Türkmençe", "region": "Central Asia"}, {"code": "mn", "en": "Mongolian", "label": "Монгол", "region": "East Asia"}, {"code": "es", "en": "Spanish", "label": "Español", "region": "Europe"}, {"code": "fr", "en": "French", "label": "Français", "region": "Europe"}, {"code": "pt", "en": "Portuguese", "label": "Português", "region": "Europe"}, {"code": "de", "en": "German", "label": "Deutsch", "region": "Europe"}, {"code": "it", "en": "Italian", "label": "Italiano", "region": "Europe"}, {"code": "ru", "en": "Russian", "label": "Русский", "region": "Europe"}, {"code": "uk", "en": "Ukrainian", "label": "Українська", "region": "Europe"}, {"code": "pl", "en": "Polish", "label": "Polski", "region": "Europe"}, {"code": "nl", "en": "Dutch", "label": "Nederlands", "region": "Europe"}, {"code": "ro", "en": "Romanian", "label": "Română", "region": "Europe"}, {"code": "hu", "en": "Hungarian", "label": "Magyar", "region": "Europe"}, {"code": "cs", "en": "Czech", "label": "Čeština", "region": "Europe"}, {"code": "sk", "en": "Slovak", "label": "Slovenčina", "region": "Europe"}, {"code": "bg", "en": "Bulgarian", "label": "Български", "region": "Europe"}, {"code": "sr", "en": "Serbian", "label": "Српски", "region": "Europe"}, {"code": "hr", "en": "Croatian", "label": "Hrvatski", "region": "Europe"}, {"code": "bs", "en": "Bosnian", "label": "Bosanski", "region": "Europe"}, {"code": "sl", "en": "Slovenian", "label": "Slovenščina", "region": "Europe"}, {"code": "el", "en": "Greek", "label": "Ελληνικά", "region": "Europe"}, {"code": "sv", "en": "Swedish", "label": "Svenska", "region": "Europe"}, {"code": "no", "en": "Norwegian", "label": "Norsk", "region": "Europe"}, {"code": "da", "en": "Danish", "label": "Dansk", "region": "Europe"}, {"code": "fi", "en": "Finnish", "label": "Suomi", "region": "Europe"}, {"code": "is", "en": "Icelandic", "label": "Íslenska", "region": "Europe"}, {"code": "ga", "en": "Irish", "label": "Gaeilge", "region": "Europe"}, {"code": "cy", "en": "Welsh", "label": "Cymraeg", "region": "Europe"}, {"code": "ca", "en": "Catalan", "label": "Català", "region": "Europe"}, {"code": "eu", "en": "Basque", "label": "Euskara", "region": "Europe"}, {"code": "gl", "en": "Galician", "label": "Galego", "region": "Europe"}, {"code": "lt", "en": "Lithuanian", "label": "Lietuvių", "region": "Europe"}, {"code": "lv", "en": "Latvian", "label": "Latviešu", "region": "Europe"}, {"code": "et", "en": "Estonian", "label": "Eesti", "region": "Europe"}, {"code": "mt", "en": "Maltese", "label": "Malti", "region": "Europe"}, {"code": "sq", "en": "Albanian", "label": "Shqip", "region": "Europe"}, {"code": "mk", "en": "Macedonian", "label": "Македонски", "region": "Europe"}, {"code": "be", "en": "Belarusian", "label": "Беларуская", "region": "Europe"}, {"code": "ka", "en": "Georgian", "label": "ქართული", "region": "Europe"}, {"code": "hy", "en": "Armenian", "label": "Հայերեն", "region": "Europe"}, {"code": "sw", "en": "Swahili", "label": "Kiswahili", "region": "Africa"}, {"code": "am", "en": "Amharic", "label": "አማርኛ", "region": "Africa"}, {"code": "ha", "en": "Hausa", "label": "Hausa", "region": "Africa"}, {"code": "yo", "en": "Yoruba", "label": "Yorùbá", "region": "Africa"}, {"code": "ig", "en": "Igbo", "label": "Igbo", "region": "Africa"}, {"code": "zu", "en": "Zulu", "label": "isiZulu", "region": "Africa"}, {"code": "xh", "en": "Xhosa", "label": "isiXhosa", "region": "Africa"}, {"code": "af", "en": "Afrikaans", "label": "Afrikaans", "region": "Africa"}, {"code": "so", "en": "Somali", "label": "Soomaali", "region": "Africa"}, {"code": "rw", "en": "Kinyarwanda", "label": "Ikinyarwanda", "region": "Africa"}, {"code": "mg", "en": "Malagasy", "label": "Malagasy", "region": "Africa"}, {"code": "pt-BR", "en": "Portuguese (Brazil)", "label": "Português (Brasil)", "region": "Americas"}, {"code": "es-MX", "en": "Spanish (Mexico)", "label": "Español (México)", "region": "Americas"}, {"code": "ht", "en": "Haitian Creole", "label": "Kreyòl ayisyen", "region": "Americas"}, {"code": "qu", "en": "Quechua", "label": "Runa Simi", "region": "Americas"}, {"code": "gn", "en": "Guarani", "label": "Avañe'ẽ", "region": "Americas"}];
  var RTL = ["ar", "ur", "fa", "he", "dv", "ps", "sd", "ks", "yi", "ckb", "ug"];

  function isRtl(code) {
    var primary = String(code || "en").split(/[-_]/)[0].toLowerCase();
    return RTL.indexOf(primary) !== -1;
  }

  function known(code) {
    return LANGS.some(function (l) { return l.code === code; });
  }

  /** Hash segments except the ones named (`l`, `lang`, …). */
  function hashParts(drop) {
    return String(location.hash || "")
      .replace(/^#/, "")
      .split("&")
      .map(function (p) { return p.trim(); })
      .filter(function (p) {
        if (!p) return false;
        var name = p.split("=")[0].toLowerCase();
        return drop.indexOf(name) === -1;
      });
  }

  function hashLang() {
    var parts = String(location.hash || "").replace(/^#/, "").split("&");
    for (var i = 0; i < parts.length; i++) {
      var m = /^l=(.+)$/i.exec(parts[i].trim());
      if (!m) continue;
      var raw = decodeURIComponent(m[1]);
      var hit = LANGS.find(function (l) { return l.code.toLowerCase() === raw.toLowerCase(); });
      if (hit) return hit.code;
    }
    return null;
  }

  /** Put `l=<code>` in the hash (English: no `l=`) and tell the wasm. */
  function syncLangHash(code) {
    var parts = hashParts(["l"]);
    if (code !== "en") parts.push("l=" + code);
    var body = parts.join("&");
    var next = body ? "#" + body : "";
    if ((location.hash || "") === next) return;
    try {
      // No history entry per language change; Makepad listens to hashchange.
      history.replaceState(null, "", location.pathname + location.search + next);
      window.dispatchEvent(new HashChangeEvent("hashchange"));
    } catch (_) {}
  }

  function current() {
    var fromHash = hashLang();
    if (fromHash) return fromHash;
    try {
      var v = localStorage.getItem(KEY);
      if (v && LANGS.some(function (l) { return l.code === v; })) return v;
    } catch (_) {}
    var html = (document.documentElement.lang || "en").replace("_", "-");
    if (LANGS.some(function (l) { return l.code === html; })) return html;
    var primary = html.split("-")[0];
    var hit = LANGS.find(function (l) { return l.code === primary; });
    return hit ? hit.code : "en";
  }

  function metaFor(code) {
    return LANGS.find(function (l) { return l.code === code; }) || LANGS[0];
  }

  function apply(code) {
    var meta = metaFor(code);
    document.documentElement.lang = meta.code;
    document.documentElement.dir = isRtl(meta.code) ? "rtl" : "ltr";
    try {
      localStorage.setItem(KEY, meta.code);
    } catch (_) {}
    syncLangHash(meta.code);
    try {
      window.dispatchEvent(
        new CustomEvent("gotigin:lang", { detail: { lang: meta.code, meta: meta } })
      );
    } catch (_) {}
  }

  function isAccountPath() {
    var p = (location.pathname || "").replace(/\/+$/, "") || "/";
    return p === "/account" || /\/account$/.test(p);
  }

  function navSlot(elNode) {
    if (!elNode) return null;
    return elNode.closest && (elNode.closest(".topnav") || elNode.closest(".topbar"));
  }

  function mountRoot() {
    var existing = document.getElementById("site-lang-chrome");
    if (existing && navSlot(existing)) return existing;
    if (existing) existing.textContent = "";
    var dead = document.getElementById("site-auth-chrome-overlay");
    if (dead && dead.parentNode) dead.parentNode.removeChild(dead);
    return null;
  }

  function paint(root) {
    root.textContent = "";
    root.className = "site-lang-chrome";
    var code = current();
    var meta = metaFor(code);

    var btn = document.createElement("button");
    btn.type = "button";
    btn.className = "site-lang-btn";
    btn.setAttribute("aria-haspopup", "listbox");
    btn.setAttribute("aria-expanded", "false");
    btn.textContent = meta.code.toUpperCase();
    btn.title = meta.label + " — " + meta.en + " (" + LANGS.length + " languages)";

    var pop = document.createElement("div");
    pop.className = "site-lang-pop";
    pop.hidden = true;

    var search = document.createElement("input");
    search.type = "search";
    search.className = "site-lang-search";
    search.placeholder = "Search " + LANGS.length + " languages…";
    search.setAttribute("aria-label", "Search languages");
    pop.appendChild(search);

    var menu = document.createElement("ul");
    menu.className = "site-lang-menu";
    menu.setAttribute("role", "listbox");
    pop.appendChild(menu);

    function fill(filter) {
      menu.textContent = "";
      var q = (filter || "").trim().toLowerCase();
      var n = 0;
      LANGS.forEach(function (l) {
        if (q) {
          var hay = (l.code + " " + l.en + " " + l.label + " " + l.region).toLowerCase();
          if (hay.indexOf(q) === -1) return;
        }
        n += 1;
        var li = document.createElement("li");
        li.setAttribute("role", "option");
        li.dataset.code = l.code;
        var native = document.createElement("span");
        native.className = "site-lang-native";
        native.textContent = l.label;
        var en = document.createElement("span");
        en.className = "site-lang-en";
        en.textContent = l.en;
        li.appendChild(native);
        li.appendChild(en);
        if (l.code === code) li.setAttribute("aria-selected", "true");
        li.addEventListener("click", function () {
          apply(l.code);
          pop.hidden = true;
          btn.setAttribute("aria-expanded", "false");
          paint(root);
        });
        menu.appendChild(li);
      });
      if (!n) {
        var empty = document.createElement("li");
        empty.className = "site-lang-empty";
        empty.textContent = "No match";
        menu.appendChild(empty);
      }
    }

    fill("");
    search.addEventListener("input", function () {
      fill(search.value);
    });
    search.addEventListener("click", function (ev) {
      ev.stopPropagation();
    });

    btn.addEventListener("click", function (ev) {
      ev.stopPropagation();
      var open = pop.hidden;
      pop.hidden = !open;
      btn.setAttribute("aria-expanded", open ? "true" : "false");
      if (open) {
        search.value = "";
        fill("");
        setTimeout(function () {
          search.focus();
        }, 0);
      }
    });

    root.appendChild(btn);
    root.appendChild(pop);
  }

  if (!window.__SITE_LANG_DOC_BOUND__) {
    window.__SITE_LANG_DOC_BOUND__ = true;
    document.addEventListener("click", function () {
      document.querySelectorAll(".site-lang-pop").forEach(function (m) {
        m.hidden = true;
      });
      document.querySelectorAll(".site-lang-btn").forEach(function (b) {
        b.setAttribute("aria-expanded", "false");
      });
    });
  }

  /** Full 115-lang menu for Makepad EN nav chip (hash #lang) or HTML topnav slot. */
  function openPicker() {
    var host = document.getElementById("site-lang-float");
    if (!host) {
      host = document.createElement("div");
      host.id = "site-lang-float";
      host.className = "site-lang-chrome";
      host.style.cssText =
        "position:fixed;top:56px;right:16px;z-index:2147483000;";
      document.body.appendChild(host);
    }
    paint(host);
    var pop = host.querySelector(".site-lang-pop");
    var btn = host.querySelector(".site-lang-btn");
    if (pop) {
      pop.hidden = false;
      if (btn) btn.setAttribute("aria-expanded", "true");
      var search = pop.querySelector(".site-lang-search");
      if (search) {
        setTimeout(function () {
          search.focus();
        }, 0);
      }
    }
    // Hide the duplicate EN button — Makepad already shows EN in the topbar.
    if (btn) btn.style.display = "none";
  }

  function onHash() {
    // `#lang` opens the picker (Makepad EN chip); `l=` and `auth=` stay.
    var parts = String(location.hash || "")
      .replace(/^#/, "")
      .split("&")
      .map(function (p) {
        return p.trim();
      })
      .filter(Boolean);
    if (parts.indexOf("lang") !== -1) {
      openPicker();
      var rest = hashParts(["lang"]).join("&");
      try {
        history.replaceState(
          null,
          "",
          location.pathname + location.search + (rest ? "#" + rest : "")
        );
      } catch (_) {}
    }
    // A hand-edited or shared `#l=` link: follow it.
    var fromHash = hashLang();
    if (fromHash && fromHash !== (document.documentElement.lang || "en")) {
      apply(fromHash);
    }
  }

  function start() {
    window.__SITE_LANG__ = { current: current, apply: apply, isRtl: isRtl, langs: LANGS };
    apply(current());
    window.__SITE_OPEN_LANG__ = openPicker;
    onHash();
    window.addEventListener("hashchange", onHash);
    var root = mountRoot();
    if (!root) return;
    paint(root);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
