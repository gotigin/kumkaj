/**
 * Makepad home: signed-in chip → #authmenu → Logout / Account menu.
 * Does not replace the Login/EN nav chips (those stay in wasm).
 */
(function () {
  "use strict";

  var ACCOUNT =
    (typeof window !== "undefined" && window.__SITE_ACCOUNT_HREF__) || "/account";
  var MENU_ID = "site-auth-session-menu";

  function authSegment() {
    return (location.hash || "")
      .replace(/^#/, "")
      .split("&")
      .filter(function (p) {
        return /^auth=/i.test(p);
      })[0];
  }

  function restoreAuthHashOnly() {
    var auth = authSegment();
    var next = auth
      ? location.pathname + location.search + "#" + auth
      : location.pathname + location.search;
    try {
      history.replaceState(null, "", next);
    } catch (_) {}
  }

  function closeMenu() {
    var host = document.getElementById(MENU_ID);
    if (host && host.parentNode) host.parentNode.removeChild(host);
    document.removeEventListener("click", onDocClick, true);
    document.removeEventListener("keydown", onKey, true);
  }

  function onDocClick(e) {
    var host = document.getElementById(MENU_ID);
    if (!host) return;
    if (host.contains(e.target)) return;
    closeMenu();
  }

  function onKey(e) {
    if (e.key === "Escape") closeMenu();
  }

  function signOut() {
    var auth = window.__SITE_AUTH__;
    closeMenu();
    if (auth && typeof auth.signOut === "function") {
      auth.signOut().catch(function () {});
    }
  }

  function openMenu() {
    closeMenu();
    var host = document.createElement("div");
    host.id = MENU_ID;
    host.setAttribute("role", "menu");
    host.style.cssText =
      "position:fixed;top:56px;right:16px;z-index:2147483001;" +
      "min-width:10.5rem;padding:0.35rem;border-radius:0.55rem;" +
      "background:#111018;color:#f7f6fb;box-shadow:0 12px 40px rgba(0,0,0,.35);" +
      "font:600 0.9rem/1.3 system-ui,sans-serif;";

    var account = document.createElement("a");
    account.href = ACCOUNT;
    account.role = "menuitem";
    account.textContent = "Account";
    account.style.cssText =
      "display:block;padding:0.65rem 0.85rem;border-radius:0.4rem;color:inherit;text-decoration:none;";
    account.addEventListener("mouseenter", function () {
      account.style.background = "rgba(255,255,255,.1)";
    });
    account.addEventListener("mouseleave", function () {
      account.style.background = "transparent";
    });

    var logout = document.createElement("button");
    logout.type = "button";
    logout.role = "menuitem";
    logout.textContent = "Logout";
    logout.style.cssText =
      "display:block;width:100%;text-align:left;padding:0.65rem 0.85rem;" +
      "border:0;border-radius:0.4rem;background:transparent;color:inherit;" +
      "font:inherit;cursor:pointer;";
    logout.addEventListener("mouseenter", function () {
      logout.style.background = "rgba(255,255,255,.1)";
    });
    logout.addEventListener("mouseleave", function () {
      logout.style.background = "transparent";
    });
    logout.addEventListener("click", function (e) {
      e.preventDefault();
      signOut();
    });

    host.appendChild(account);
    host.appendChild(logout);
    document.body.appendChild(host);
    setTimeout(function () {
      document.addEventListener("click", onDocClick, true);
      document.addEventListener("keydown", onKey, true);
    }, 0);
  }

  function onHash() {
    var parts = (location.hash || "")
      .replace(/^#/, "")
      .split("&")
      .map(function (p) {
        return p.trim();
      })
      .filter(Boolean);
    if (parts.indexOf("authmenu") === -1) return;
    openMenu();
    restoreAuthHashOnly();
  }

  function start() {
    window.__SITE_OPEN_AUTH_MENU__ = openMenu;
    onHash();
    window.addEventListener("hashchange", onHash);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
