// Firebase web app for kirano.shop — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
//
// Modes (do not mix):
//   Production Hosting → real project + authDomain kirano.shop (no emulator).
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

// Same web app as Firebase Console; authDomain is our custom host (not
// kirano-shop.firebaseapp.com) so OAuth callbacks use
// https://kirano.shop/__/auth/handler.
const firebaseConfig = {
  apiKey: "AIzaSyD7Bn0dxBBjuUge8MX3JEXjAW8Dx5T0vyQ",
  authDomain: "kirano.shop",
  projectId: "kirano-shop",
  storageBucket: "kirano-shop.firebasestorage.app",
  messagingSenderId: "986588138001",
  appId: "1:986588138001:web:0138f40e645596076c3fb0",
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
  window.__SITE_PRODUCT_NAME__ = window.__SITE_PRODUCT_NAME__ || "Kirano";
  window.__SITE_ACCOUNT_HREF__ = window.__SITE_ACCOUNT_HREF__ || "/account";
  window.__SITE_API_BASE__ = window.__SITE_API_BASE__ || "https://api.kirano.shop";
  window.__SITE_SIGN_IN_GOOGLE__ = () =>
    signInWithPopup(auth, new GoogleAuthProvider());
  window.__SITE_SIGN_IN_EMAIL__ = (email, password) =>
    signInWithEmailAndPassword(auth, email, password);
  window.__SITE_REGISTER_EMAIL__ = (email, password) =>
    createUserWithEmailAndPassword(auth, email, password);
}

export { app, auth, analytics };
