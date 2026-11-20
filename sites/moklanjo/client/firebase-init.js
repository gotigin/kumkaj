// Firebase web app for moklanjo.com — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
import { getAuth } from "firebase/auth";

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

let analytics = null;
try {
  analytics = getAnalytics(app);
} catch {
  // Analytics may fail in some local/offline contexts; Auth still works.
}

export { app, auth, analytics };
