// Firebase web app for khaynjo.com — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
//
// Modes (do not mix):
//   Production Hosting (khaynjo.com) → real Auth, no emulator.
//   Local loopback (127.0.0.1 / localhost) → Auth emulator :9099 automatically.
//   Override: window.__KUMKAJ_AUTH_EMULATOR__ = "http://127.0.0.1:9099" (or "" to force off).
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
import {
  connectAuthEmulator,
  getAuth,
  onAuthStateChanged,
  GoogleAuthProvider,
  signInWithPopup,
  signInWithEmailAndPassword,
  createUserWithEmailAndPassword,
} from "firebase/auth";

const firebaseConfig = {
  apiKey: "AIzaSyDJXNj8zvcSWN5Me7q0OmDjMi1sCYpaIrE",
  // Custom domain so OAuth returns via https://khaynjo.com/__/auth/handler
  authDomain: "khaynjo.com",
  projectId: "khaynjo-2b2a2",
  storageBucket: "khaynjo-2b2a2.firebasestorage.app",
  messagingSenderId: "420795333879",
  appId: "1:420795333879:web:a3372828be321376cbe7f2",
  measurementId: "G-T5R5KHSY35",
};

const app = initializeApp(firebaseConfig);
const auth = getAuth(app);

function resolveAuthEmulatorHost() {
  if (typeof window === "undefined") return "";
  if (typeof window.__KUMKAJ_AUTH_EMULATOR__ === "string") {
    return window.__KUMKAJ_AUTH_EMULATOR__.trim();
  }
  const host = (location.hostname || "").toLowerCase();
  if (host === "127.0.0.1" || host === "localhost") {
    return "http://127.0.0.1:9099";
  }
  return "";
}

const emulatorHost = resolveAuthEmulatorHost();
if (emulatorHost) {
  connectAuthEmulator(auth, emulatorHost.replace(/\/$/, ""), {
    disableWarnings: true,
  });
}

let analytics = null;
if (!emulatorHost) {
  try {
    analytics = getAnalytics(app);
  } catch {
    // Analytics may fail in some local/offline contexts; Auth still works.
  }
}

function chipLabel(user) {
  if (!user) return "";
  const name = String(user.displayName || "").trim();
  if (name) {
    const parts = name.split(/\s+/).filter(Boolean);
    if (parts.length >= 2) {
      return (parts[0].charAt(0) + parts[1].charAt(0)).toUpperCase();
    }
    return name.slice(0, 2).toUpperCase();
  }
  const email = String(user.email || "").trim();
  if (email) return email.slice(0, 2).toUpperCase();
  const phone = String(user.phoneNumber || "").replace(/\D/g, "");
  if (phone.length >= 2) return phone.slice(-2);
  return "Account";
}

/** Keep `#auth=XX` in the hash so Makepad wasm can show the Login chip (no web-sys). */
function syncAuthHash(label) {
  if (typeof location === "undefined") return;
  const raw = String(location.hash || "").replace(/^#/, "");
  const parts = raw
    .split("&")
    .map((p) => p.trim())
    .filter((p) => p && !/^auth=/i.test(p));
  if (label) {
    parts.push("auth=" + encodeURIComponent(label));
  }
  const nextBody = parts.join("&");
  const next = nextBody ? "#" + nextBody : "";
  if ((location.hash || "") === next) return;
  try {
    if (nextBody) {
      // Assignment fires hashchange → Makepad Signal (replaceState alone may not).
      location.hash = nextBody;
    } else {
      history.replaceState(null, "", location.pathname + location.search);
      try {
        window.dispatchEvent(new HashChangeEvent("hashchange"));
      } catch (__) {
        try {
          window.dispatchEvent(new Event("hashchange"));
        } catch (___) {}
      }
    }
  } catch (_) {}
}

function publishAuthChip(user) {
  const label = user ? chipLabel(user) : "";
  try {
    if (user) {
      localStorage.setItem(
        "khaynjo.auth.chip",
        JSON.stringify({
          label,
          uid: user.uid,
        })
      );
    } else {
      localStorage.removeItem("khaynjo.auth.chip");
    }
  } catch (_) {}
  syncAuthHash(label);
  try {
    window.dispatchEvent(
      new CustomEvent("khaynjo:auth", { detail: { signedIn: !!user } })
    );
  } catch (_) {}
}

// Re-attach auth= after other hash bridges (#lang / #authmenu) clear themselves.
if (typeof window !== "undefined") {
  window.addEventListener("hashchange", () => {
    try {
      const hash = location.hash || "";
      // Let lang / auth-menu handlers run first; do not fight them mid-open.
      if (/(?:^|[&#])lang(?:&|$)/.test(hash) || /(?:^|[&#])authmenu(?:&|$)/.test(hash)) {
        return;
      }
      const raw = localStorage.getItem("khaynjo.auth.chip");
      if (!raw) return;
      const parsed = JSON.parse(raw);
      const label = parsed && parsed.label ? String(parsed.label) : "";
      if (!label) return;
      if (!/auth=/i.test(hash)) {
        syncAuthHash(label);
      }
    } catch (_) {}
  });
}

if (typeof window !== "undefined") {
  window.__SITE_AUTH__ = auth;
  window.__SITE_APP__ = app;
  window.__SITE_PRODUCT_NAME__ = window.__SITE_PRODUCT_NAME__ || "Khaynjo";
  window.__SITE_ACCOUNT_HREF__ = window.__SITE_ACCOUNT_HREF__ || "/account";
  window.__SITE_API_BASE__ = window.__SITE_API_BASE__ || "https://api.khaynjo.com";
  window.__SITE_SIGN_IN_GOOGLE__ = () =>
    signInWithPopup(auth, new GoogleAuthProvider());
  window.__SITE_SIGN_IN_EMAIL__ = (email, password) =>
    signInWithEmailAndPassword(auth, email, password);
  window.__SITE_REGISTER_EMAIL__ = (email, password) =>
    createUserWithEmailAndPassword(auth, email, password);

  // Paint chip hash immediately from a prior session (before Auth finishes).
  try {
    const raw = localStorage.getItem("khaynjo.auth.chip");
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && parsed.label) syncAuthHash(String(parsed.label));
    }
  } catch (_) {}

  onAuthStateChanged(
    auth,
    (user) => {
      publishAuthChip(user);
    },
    () => {
      publishAuthChip(null);
    }
  );

  // bfcache restore (history.back after login): refresh chip without remounting.
  window.addEventListener("pageshow", (ev) => {
    if (ev.persisted) {
      publishAuthChip(auth.currentUser || null);
    }
  });
}

export { app, auth, analytics };
