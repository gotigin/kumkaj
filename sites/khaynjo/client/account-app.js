// Unified register / sign-in for khaynjo.com (Firebase Auth).
// Same provider set as Wadhu site Auth: email, Google, Microsoft, GitHub, phone.
import { auth } from "./firebase-init.js";
import {
  GoogleAuthProvider,
  GithubAuthProvider,
  OAuthProvider,
  RecaptchaVerifier,
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  sendEmailVerification,
  signInWithPopup,
  signInWithPhoneNumber,
  signOut,
  onAuthStateChanged,
  updateProfile,
} from "firebase/auth";

const $ = (id) => document.getElementById(id);

const els = {
  gate: $("auth-gate"),
  session: $("auth-session"),
  status: $("auth-status"),
  error: $("auth-error"),
  email: $("auth-email"),
  password: $("auth-password"),
  displayName: $("auth-display-name"),
  phone: $("auth-phone"),
  smsCode: $("auth-sms-code"),
  phoneStep: $("auth-phone-step"),
  smsStep: $("auth-sms-step"),
  userEmail: $("session-email"),
  userVerified: $("session-verified"),
};

let confirmationResult = null;
let recaptcha = null;

function showError(msg) {
  els.error.textContent = msg || "";
  els.error.hidden = !msg;
}

function setBusy(busy) {
  els.gate?.querySelectorAll("button, input").forEach((el) => {
    el.disabled = !!busy;
  });
}

function friendlyError(err) {
  const code = err?.code || "";
  const map = {
    "auth/invalid-email": "Enter a valid email address.",
    "auth/weak-password": "Use a password with at least 6 characters.",
    "auth/wrong-password": "Wrong password.",
    "auth/invalid-credential": "Email or password is wrong.",
    "auth/popup-closed-by-user": "Sign-in window was closed.",
    "auth/cancelled-popup-request": "Sign-in was cancelled.",
    "auth/unauthorized-domain": "This domain is not authorized for Firebase Auth.",
    "auth/operation-not-allowed": "That sign-in method is not enabled yet.",
    "auth/too-many-requests": "Too many attempts. Try again later.",
    "auth/invalid-phone-number": "Use a full phone number with country code (e.g. +91…).",
    "auth/invalid-verification-code": "That SMS code is wrong or expired.",
  };
  return map[code] || err?.message || "Something went wrong.";
}

function renderSession(user) {
  if (user) {
    els.gate.hidden = true;
    els.session.hidden = false;
    els.userEmail.textContent = user.email || user.phoneNumber || "(no email)";
    els.userVerified.textContent = user.email
      ? user.emailVerified
        ? "Email verified"
        : "Email not verified yet — check your inbox"
      : user.phoneNumber
        ? "Signed in with phone"
        : "Signed in";
    els.status.textContent = "You’re signed in.";
  } else {
    els.gate.hidden = false;
    els.session.hidden = true;
    els.status.textContent = "Continue with email, a provider, or phone.";
  }
}

async function afterAuth(user, { sendVerify } = {}) {
  if (sendVerify && user?.email && !user.emailVerified) {
    try {
      await sendEmailVerification(user);
    } catch {
      // Non-fatal; user can request again later.
    }
  }
  showError("");
  renderSession(user);
}

async function handleEmail(e) {
  e.preventDefault();
  showError("");
  setBusy(true);
  try {
    const email = els.email.value.trim();
    const password = els.password.value;
    const name = els.displayName.value.trim();
    try {
      const cred = await createUserWithEmailAndPassword(auth, email, password);
      if (name) await updateProfile(cred.user, { displayName: name });
      await afterAuth(cred.user, { sendVerify: true });
    } catch (err) {
      if (err?.code !== "auth/email-already-in-use") throw err;
      const cred = await signInWithEmailAndPassword(auth, email, password);
      await afterAuth(cred.user);
    }
  } catch (err) {
    showError(friendlyError(err));
  } finally {
    setBusy(false);
  }
}

async function popup(provider) {
  showError("");
  setBusy(true);
  try {
    const cred = await signInWithPopup(auth, provider);
    await afterAuth(cred.user);
  } catch (err) {
    showError(friendlyError(err));
  } finally {
    setBusy(false);
  }
}

function ensureRecaptcha() {
  if (recaptcha) return recaptcha;
  recaptcha = new RecaptchaVerifier(auth, "recaptcha-container", {
    size: "invisible",
  });
  return recaptcha;
}

async function sendSms(e) {
  e.preventDefault();
  showError("");
  setBusy(true);
  try {
    const phone = els.phone.value.trim();
    const appVerifier = ensureRecaptcha();
    confirmationResult = await signInWithPhoneNumber(auth, phone, appVerifier);
    els.phoneStep.hidden = true;
    els.smsStep.hidden = false;
    els.smsCode.focus();
  } catch (err) {
    showError(friendlyError(err));
    try {
      recaptcha?.clear();
    } catch {
      /* ignore */
    }
    recaptcha = null;
  } finally {
    setBusy(false);
  }
}

async function confirmSms(e) {
  e.preventDefault();
  showError("");
  setBusy(true);
  try {
    const code = els.smsCode.value.trim();
    const cred = await confirmationResult.confirm(code);
    await afterAuth(cred.user);
  } catch (err) {
    showError(friendlyError(err));
  } finally {
    setBusy(false);
  }
}

async function doSignOut() {
  showError("");
  await signOut(auth);
  confirmationResult = null;
  els.phoneStep.hidden = false;
  els.smsStep.hidden = true;
  els.password.value = "";
  els.smsCode.value = "";
}

function wire() {
  $("form-email")?.addEventListener("submit", handleEmail);
  $("btn-google")?.addEventListener("click", () => popup(new GoogleAuthProvider()));
  $("btn-github")?.addEventListener("click", () => popup(new GithubAuthProvider()));
  $("btn-microsoft")?.addEventListener("click", () => {
    const p = new OAuthProvider("microsoft.com");
    p.setCustomParameters({ prompt: "select_account" });
    return popup(p);
  });
  $("form-phone")?.addEventListener("submit", sendSms);
  $("form-sms")?.addEventListener("submit", confirmSms);
  $("btn-signout")?.addEventListener("click", doSignOut);
  $("btn-resend-verify")?.addEventListener("click", async () => {
    const u = auth.currentUser;
    if (!u?.email) return;
    try {
      await sendEmailVerification(u);
      els.userVerified.textContent = "Verification email sent — check your inbox";
    } catch (err) {
      showError(friendlyError(err));
    }
  });

  onAuthStateChanged(auth, (user) => {
    renderSession(user);
  });
}

wire();
