// Firebase web app for khaynjo.com — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
import { getAuth } from "firebase/auth";

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

let analytics = null;
try {
  analytics = getAnalytics(app);
} catch {
  // Analytics may fail in some local/offline contexts; Auth still works.
}

export { app, auth, analytics };
