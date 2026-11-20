// Firebase web app for tabiyat.shop — Analytics + Auth.
// Bundled into static/js/firebase-init.js and reused by account.js.
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
import { getAuth } from "firebase/auth";

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

let analytics = null;
try {
  analytics = getAnalytics(app);
} catch {
  // Analytics may fail in some local/offline contexts; Auth still works.
}

export { app, auth, analytics };
