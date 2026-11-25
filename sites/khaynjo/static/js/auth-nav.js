/**
 * Auth chrome for HTML topnav only.
 * Login is a real <a data-nav="login"> nav item. This script only swaps it
 * for avatar + click-to-open Logout menu when signed in.
 */
(function () {
  "use strict";

  var ACCOUNT =
    (typeof window !== "undefined" && window.__SITE_ACCOUNT_HREF__) || "/account";

  function el(tag, attrs, kids) {
    var n = document.createElement(tag);
    if (attrs) {
      Object.keys(attrs).forEach(function (k) {
        if (k === "className") n.className = attrs[k];
        else if (k === "text") n.textContent = attrs[k];
        else if (k === "hidden") n.hidden = !!attrs[k];
        else if (k.indexOf("on") === 0 && typeof attrs[k] === "function") {
          n.addEventListener(k.slice(2).toLowerCase(), attrs[k]);
        } else if (attrs[k] != null && attrs[k] !== false) {
          n.setAttribute(k, attrs[k] === true ? "" : String(attrs[k]));
        }
      });
    }
    (kids || []).forEach(function (c) {
      if (c) n.appendChild(c);
    });
    return n;
  }

  function nav() {
    // Prefer .topnav: legal pages wrap it in <header class="topbar">, and
    // querySelector(".topnav, .topbar") would return the outer header first —
    // insertBefore(host, #site-lang-chrome) then throws NotFoundError because
    // the lang slot is a child of .topnav, not of .topbar.
    return document.querySelector(".topnav") || document.querySelector(".topbar");
  }

  function loginLink() {
    var n = nav();
    if (!n) return null;
    return n.querySelector('a[data-nav="login"], a.site-auth-login, a[href="/account"]');
  }

  function deadOverlay() {
    var dead = document.getElementById("site-auth-chrome-overlay");
    if (dead && dead.parentNode) dead.parentNode.removeChild(dead);
  }

  function initialsFromName(name) {
    var parts = String(name || "").trim().split(/\s+/).filter(Boolean);
    if (parts.length >= 2) return (parts[0].charAt(0) + parts[1].charAt(0)).toUpperCase();
    if (parts.length === 1 && parts[0].length >= 1) return parts[0].slice(0, 2).toUpperCase();
    return "";
  }

  function identity(user) {
    if (!user) return null;
    if (user.photoURL) {
      return { kind: "photo", url: user.photoURL, label: user.displayName || user.email || "Account" };
    }
    var fromName = initialsFromName(user.displayName);
    if (fromName) return { kind: "text", text: fromName, label: user.displayName };
    var email = (user.email || "").trim();
    if (email.length >= 1) return { kind: "text", text: email.slice(0, 2).toUpperCase(), label: email };
    var phone = (user.phoneNumber || "").replace(/\D/g, "");
    if (phone.length >= 2) return { kind: "text", text: phone.slice(-2), label: user.phoneNumber };
    return { kind: "text", text: "Anonymous", label: "Anonymous" };
  }

  function ensureHost() {
    var n = nav();
    if (!n) return null;
    var host = n.querySelector("#site-auth-chrome");
    if (!host) {
      host = el("span", { id: "site-auth-chrome", className: "site-auth-chrome" });
      var lang = n.querySelector("#site-lang-chrome");
      // Insert beside the lang slot in whatever parent actually owns it.
      if (lang && lang.parentNode) lang.parentNode.insertBefore(host, lang);
      else n.appendChild(host);
    }
    return host;
  }

  function paint(user, auth) {
    deadOverlay();
    var n = nav();
    if (!n) return;
    var link = loginLink();
    var host = ensureHost();
    if (!host) return;

    if (!user) {
      host.textContent = "";
      if (link) link.hidden = false;
      return;
    }

    if (link) link.hidden = true;
    host.textContent = "";
    host.className = "site-auth-chrome site-auth-chrome--menu";

    var id = identity(user);
    var who;
    if (id.kind === "photo") {
      who = el("button", {
        type: "button",
        className: "site-auth-avatar site-auth-avatar--photo",
        title: id.label,
        "aria-haspopup": "menu",
        "aria-expanded": "false",
      });
      who.appendChild(
        el("img", { src: id.url, alt: "", width: "28", height: "28", referrerpolicy: "no-referrer" })
      );
    } else {
      who = el("button", {
        type: "button",
        className:
          "site-auth-avatar site-auth-avatar--text" +
          (id.text === "Anonymous" ? " site-auth-avatar--anon" : ""),
        title: id.label,
        text: id.text,
        "aria-haspopup": "menu",
        "aria-expanded": "false",
      });
    }

    var menu = el("div", {
      className: "site-auth-menu",
      role: "menu",
      hidden: true,
    });
    menu.appendChild(
      el("a", {
        className: "site-auth-menu-item",
        href: ACCOUNT,
        role: "menuitem",
        text: "Account",
      })
    );
    menu.appendChild(
      el("button", {
        type: "button",
        className: "site-auth-menu-item site-auth-logout",
        role: "menuitem",
        text: "Logout",
        onClick: function () {
          menu.hidden = true;
          who.setAttribute("aria-expanded", "false");
          if (auth && typeof auth.signOut === "function") auth.signOut().catch(function () {});
        },
      })
    );

    who.addEventListener("click", function (e) {
      e.preventDefault();
      e.stopPropagation();
      var open = menu.hidden;
      menu.hidden = !open;
      who.setAttribute("aria-expanded", open ? "true" : "false");
    });

    host.appendChild(who);
    host.appendChild(menu);
  }

  function bind(auth) {
    paint(auth.currentUser || null, auth);
    if (typeof auth.onAuthStateChanged === "function") {
      auth.onAuthStateChanged(function (user) {
        paint(user, auth);
      });
    }
  }

  function start() {
    deadOverlay();
    if (!nav()) return;
    var n = 0;
    var t = setInterval(function () {
      n += 1;
      if (window.__SITE_AUTH__) {
        clearInterval(t);
        bind(window.__SITE_AUTH__);
        return;
      }
      if (n > 100) {
        clearInterval(t);
        paint(null, null);
      }
    }, 50);
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
})();
