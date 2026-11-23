// Firebase web app for tabiyat.shop — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
//
// Modes (do not mix):
//   Production Hosting → real project + authDomain tabiyat.shop (no emulator).
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
// tabiyat-shop.firebaseapp.com) so OAuth callbacks use
// https://tabiyat.shop/__/auth/handler.
const firebaseConfig = {
  apiKey: "AIzaSyD75kxX6GHrsoqvpP8Mynb1k8PT0lrEMDk",
  authDomain: "tabiyat.shop",
  projectId: "tabiyat-shop",
  storageBucket: "tabiyat-shop.firebasestorage.app",
  messagingSenderId: "1032658084956",
  appId: "1:1032658084956:web:78e84cbe109bdd7d907299",
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
  window.__SITE_PRODUCT_NAME__ = window.__SITE_PRODUCT_NAME__ || "Tabiyat";
  window.__SITE_ACCOUNT_HREF__ = window.__SITE_ACCOUNT_HREF__ || "/account";
  window.__SITE_API_BASE__ = window.__SITE_API_BASE__ || "https://api.tabiyat.shop";
  window.__SITE_SIGN_IN_GOOGLE__ = () =>
    signInWithPopup(auth, new GoogleAuthProvider());
  window.__SITE_SIGN_IN_EMAIL__ = (email, password) =>
    signInWithEmailAndPassword(auth, email, password);
  window.__SITE_REGISTER_EMAIL__ = (email, password) =>
    createUserWithEmailAndPassword(auth, email, password);
}

export { app, auth, analytics };
