// Firebase web app for kumkaj.com — Auth for site chrome.
// Bundled into static/js/firebase-init.js.
import { initializeApp } from "firebase/app";
import {
  connectAuthEmulator,
  getAuth,
  GoogleAuthProvider,
  signInWithPopup,
  signInWithEmailAndPassword,
  createUserWithEmailAndPassword,
} from "firebase/auth";

const firebaseConfig = {
  apiKey: "AIzaSyCNJi5AM2EbjdKDeg3afxDIaufGuz5dks8",
  authDomain: "kumkaj.com",
  projectId: "kumkaj-b32abc",
  storageBucket: "kumkaj-b32abc.firebasestorage.app",
  messagingSenderId: "707475164705",
  appId: "1:707475164705:web:66e19a525b1d568ab91d60",
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

if (typeof window !== "undefined") {
  window.__SITE_AUTH__ = auth;
  window.__SITE_APP__ = app;
  window.__SITE_PRODUCT_NAME__ = window.__SITE_PRODUCT_NAME__ || "Kumkaj";
  window.__SITE_ACCOUNT_HREF__ = window.__SITE_ACCOUNT_HREF__ || "/account";
  window.__SITE_API_BASE__ = window.__SITE_API_BASE__ || "";
  window.__SITE_SIGN_IN_GOOGLE__ = () =>
    signInWithPopup(auth, new GoogleAuthProvider());
  window.__SITE_SIGN_IN_EMAIL__ = (email, password) =>
    signInWithEmailAndPassword(auth, email, password);
  window.__SITE_REGISTER_EMAIL__ = (email, password) =>
    createUserWithEmailAndPassword(auth, email, password);
}

export { app, auth };
