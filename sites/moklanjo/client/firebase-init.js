// Firebase web app for moklanjo.com — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
//
// Modes (do not mix):
//   Production Hosting → real project + authDomain moklanjo.com (no emulator).
//   Local isolated E2E → set window.__KUMKAJ_AUTH_EMULATOR__ before this bundle.
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
import {
  connectAuthEmulator,
  getAuth,
  GoogleAuthProvider,
  signInWithPopup,
  signInWithEmailAndPassword,
  createUserWithEmailAndPassword,
} from "firebase/auth";

const firebaseConfig = {
  apiKey: "AIzaSyB5E2V97YXGraGHdwnXYQhPsLdFpOIpDy0",
  // Custom domain so OAuth returns via https://moklanjo.com/__/auth/handler
  authDomain: "moklanjo.com",
  projectId: "moklanjo-com",
  storageBucket: "moklanjo-com.firebasestorage.app",
  messagingSenderId: "164557367573",
  appId: "1:164557367573:web:36864b22385723bb56611c",
};

const app = initializeApp(firebaseConfig);
const auth = getAuth(app);

const emulatorHost =
  typeof window !== "undefined" &&
  typeof window.__KUMKAJ_AUTH_EMULATOR__ === "string" &&
  window.__KUMKAJ_AUTH_EMULATOR__.trim();
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

if (typeof window !== "undefined") {
  window.__SITE_AUTH__ = auth;
  window.__SITE_APP__ = app;
  window.__SITE_PRODUCT_NAME__ = window.__SITE_PRODUCT_NAME__ || "Moklanjo";
  window.__SITE_ACCOUNT_HREF__ = window.__SITE_ACCOUNT_HREF__ || "/account";
  window.__SITE_API_BASE__ = window.__SITE_API_BASE__ || "https://api.moklanjo.com";
  window.__SITE_SIGN_IN_GOOGLE__ = () =>
    signInWithPopup(auth, new GoogleAuthProvider());
  window.__SITE_SIGN_IN_EMAIL__ = (email, password) =>
    signInWithEmailAndPassword(auth, email, password);
  window.__SITE_REGISTER_EMAIL__ = (email, password) =>
    createUserWithEmailAndPassword(auth, email, password);
}

export { app, auth, analytics };
