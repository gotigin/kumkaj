// Firebase web app for kirano.shop — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
import { getAuth } from "firebase/auth";

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

let analytics = null;
try {
  analytics = getAnalytics(app);
} catch {
  // Analytics may fail in some local/offline contexts; Auth still works.
}

export { app, auth, analytics };
