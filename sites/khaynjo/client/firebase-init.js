// Firebase web app (Analytics) for khaynjo.com — bundled into static/js/firebase-init.js.
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";

const firebaseConfig = {
  apiKey: "AIzaSyDJXNj8zvcSWN5Me7q0OmDjMi1sCYpaIrE",
  authDomain: "khaynjo-2b2a2.firebaseapp.com",
  projectId: "khaynjo-2b2a2",
  storageBucket: "khaynjo-2b2a2.firebasestorage.app",
  messagingSenderId: "420795333879",
  appId: "1:420795333879:web:a3372828be321376cbe7f2",
  measurementId: "G-T5R5KHSY35",
};

const app = initializeApp(firebaseConfig);
const analytics = getAnalytics(app);

export { app, analytics };
