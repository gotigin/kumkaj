// Unified register / sign-in / recovery / link for khaynjo.com (Firebase Auth).
// Providers: email, Google, Microsoft, GitHub, phone. Apple postponed.
import { auth } from "./firebase-init.js";
import {
  GoogleAuthProvider,
  GithubAuthProvider,
  OAuthProvider,
  EmailAuthProvider,
  RecaptchaVerifier,
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  sendEmailVerification,
  sendPasswordResetEmail,
  signInWithPopup,
  signInWithPhoneNumber,
  linkWithPopup,
  linkWithCredential,
  linkWithPhoneNumber,
  reauthenticateWithCredential,
  reauthenticateWithPopup,
  updatePassword,
  fetchSignInMethodsForEmail,
  signOut,
  onAuthStateChanged,
  updateProfile,
} from "firebase/auth";

const $ = (id) => document.getElementById(id);

/** Override via `window.__KUMKAJ_CLOUD_API__` (no trailing slash). IPs change on redeploy. */
const CLOUD_API_BASE =
  (typeof window !== "undefined" && window.__KUMKAJ_CLOUD_API__) ||
  "http://16.4.32.208:8080";

const els = {
  gate: $("auth-gate"),
  session: $("auth-session"),
  status: $("auth-status"),
  error: $("auth-error"),
  ok: $("auth-ok"),
  email: $("auth-email"),
  password: $("auth-password"),
  displayName: $("auth-display-name"),
  phone: $("auth-phone"),
  smsCode: $("auth-sms-code"),
  phoneStep: $("auth-phone-step"),
  smsStep: $("auth-sms-step"),
  forgotPanel: $("forgot-panel"),
  forgotEmail: $("auth-forgot-email"),
  userEmail: $("session-email"),
  userVerified: $("session-verified"),
  userProviders: $("session-providers"),
  cloudUser: $("session-cloud-user"),
  cloudPlan: $("session-cloud-plan"),
  cardCloudPlans: $("card-cloud-plans"),
  planList: $("plan-list"),
  billingHint: $("billing-hint"),
  billingStatus: $("billing-status"),
  passwordHint: $("password-hint"),
  passwordCurrentWrap: $("password-current-wrap"),
  passwordEmailWrap: $("password-email-wrap"),
  currentPassword: $("auth-current-password"),
  setEmail: $("auth-set-email"),
  newPassword: $("auth-new-password"),
  newPassword2: $("auth-new-password2"),
  btnPasswordSave: $("btn-password-save"),
  btnLinkGoogle: $("btn-link-google"),
  btnLinkMicrosoft: $("btn-link-microsoft"),
  btnLinkGithub: $("btn-link-github"),
  linkPhoneWrap: $("link-phone-wrap"),
  linkPhone: $("auth-link-phone"),
  formLinkSms: $("form-link-sms"),
  linkSmsCode: $("auth-link-sms-code"),
  btnResendVerify: $("btn-resend-verify"),
};

let confirmationResult = null;
let linkConfirmationResult = null;
let recaptcha = null;
let linkRecaptcha = null;

function showError(msg) {
  els.error.textContent = msg || "";
  els.error.hidden = !msg;
  if (msg) showOk("");
}

function showOk(msg) {
  els.ok.textContent = msg || "";
  els.ok.hidden = !msg;
}

function setBusy(busy) {
  document.querySelectorAll(".account-main button, .account-main input").forEach((el) => {
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
    "auth/user-not-found": "No account with that email. Continue to create one, or use another method.",
    "auth/requires-recent-login": "For security, sign in again, then retry this change.",
    "auth/credential-already-in-use": "That sign-in method is already used by another account.",
    "auth/email-already-in-use": "That email is already used by another account.",
    "auth/provider-already-linked": "That method is already linked to this account.",
    "auth/account-exists-with-different-credential":
      "An account already exists with this email using a different sign-in method. Sign in with that method first, then link this one from your account page.",
  };
  return map[code] || err?.message || "Something went wrong.";
}

function providerIds(user) {
  return (user?.providerData || []).map((p) => p.providerId);
}

function hasPassword(user) {
  return providerIds(user).includes("password");
}

function providerLabel(id) {
  const map = {
    password: "Email & password",
    "google.com": "Google",
    "github.com": "GitHub",
    "microsoft.com": "Microsoft",
    phone: "Phone",
  };
  return map[id] || id;
}

function googleProvider() {
  return new GoogleAuthProvider();
}
function githubProvider() {
  return new GithubAuthProvider();
}
function microsoftProvider() {
  const p = new OAuthProvider("microsoft.com");
  p.setCustomParameters({ prompt: "select_account" });
  return p;
}

function paintLinkButtons(user) {
  const ids = providerIds(user);
  const set = (btn, id) => {
    if (!btn) return;
    const linked = ids.includes(id);
    btn.hidden = linked;
    btn.disabled = linked;
  };
  set(els.btnLinkGoogle, "google.com");
  set(els.btnLinkMicrosoft, "microsoft.com");
  set(els.btnLinkGithub, "github.com");
  if (els.linkPhoneWrap) {
    els.linkPhoneWrap.hidden = ids.includes("phone");
  }
}

function paintPasswordCard(user) {
  const withPw = hasPassword(user);
  if (els.passwordHint) {
    els.passwordHint.textContent = withPw
      ? "Change your password. You’ll confirm with the current one."
      : "Add a password so you can also sign in with email (useful if a provider is unavailable).";
  }
  if (els.passwordCurrentWrap) els.passwordCurrentWrap.hidden = !withPw;
  if (els.passwordEmailWrap) els.passwordEmailWrap.hidden = withPw;
  if (els.btnPasswordSave) {
    els.btnPasswordSave.textContent = withPw ? "Change password" : "Add password";
  }
  if (els.setEmail && user?.email) els.setEmail.value = user.email;
  if (els.currentPassword) els.currentPassword.value = "";
  if (els.newPassword) els.newPassword.value = "";
  if (els.newPassword2) els.newPassword2.value = "";
}

function paintCloudPlaceholders() {
  if (els.cloudUser) els.cloudUser.textContent = "—";
  if (els.cloudPlan) els.cloudPlan.textContent = "—";
  if (els.cardCloudPlans) els.cardCloudPlans.hidden = true;
  if (els.planList) els.planList.innerHTML = "";
  if (els.billingStatus) els.billingStatus.textContent = "";
}

function formatInrPaise(paise) {
  if (paise == null) return "Quote";
  if (paise === 0) return "₹0";
  return `₹${(paise / 100).toLocaleString("en-IN")}/yr ex-tax`;
}

async function startCheckout(user, planId, provider) {
  if (!user) return;
  setBusy(true);
  showError("");
  try {
    const idToken = await user.getIdToken();
    const res = await fetch(`${CLOUD_API_BASE}/v1/billing/checkout`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${idToken}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ plan_id: planId, provider }),
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Checkout HTTP ${res.status}`);
    if (provider === "mock" && body.checkout_id) {
      const done = await fetch(`${CLOUD_API_BASE}/v1/billing/sandbox/complete`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${idToken}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ checkout_id: body.checkout_id }),
      });
      const doneBody = await done.json().catch(() => ({}));
      if (!done.ok) throw new Error(doneBody.error || `Complete HTTP ${done.status}`);
      showOk(`Sandbox: activated ${doneBody.plan_id} until ${String(doneBody.ends_at || "").slice(0, 10)}`);
      await syncCloudAccount(user);
      return;
    }
    if (body.url) {
      window.location.href = body.url;
      return;
    }
    throw new Error("Checkout did not return a URL");
  } catch (err) {
    showError(err?.message || "Checkout failed");
  } finally {
    setBusy(false);
  }
}

async function paintBillingPlans(user, idToken) {
  if (!els.cardCloudPlans || !els.planList) return;
  try {
    const [cfgRes, plansRes] = await Promise.all([
      fetch(`${CLOUD_API_BASE}/v1/billing/config`, {
        headers: { Authorization: `Bearer ${idToken}` },
      }),
      fetch(`${CLOUD_API_BASE}/v1/plans`, {
        headers: { Authorization: `Bearer ${idToken}` },
      }),
    ]);
    if (!cfgRes.ok || !plansRes.ok) {
      els.cardCloudPlans.hidden = true;
      return;
    }
    const cfg = await cfgRes.json();
    const plans = await plansRes.json();
    if (!cfg.enabled) {
      els.cardCloudPlans.hidden = true;
      return;
    }
    els.cardCloudPlans.hidden = false;
    if (els.billingStatus) {
      els.billingStatus.textContent = `Billing mode: ${cfg.mode} · providers: ${(cfg.providers || []).join(", ") || "none"} · default: ${cfg.default_provider || "—"}`;
    }
    const sellable = (plans || []).filter(
      (p) => p.includes_live_cloud && p.plan_id !== "custom" && p.checkout_ready
    );
    els.planList.innerHTML = "";
    if (!sellable.length) {
      els.planList.innerHTML =
        "<p class=\"auth-hint\">No self-serve plans are checkout-ready yet. For Stripe/Paddle, set price ids on the plans table (see BILLING_SANDBOX.md).</p>";
      return;
    }
    const provider = cfg.default_provider || "mock";
    for (const plan of sellable) {
      const row = document.createElement("div");
      row.className = "plan-row";
      const meta = document.createElement("div");
      meta.innerHTML = `<strong>${plan.title}</strong><span>${formatInrPaise(plan.price_inr_paise)} · ${plan.quota_gb || "—"} GB</span>`;
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "btn btn-primary";
      btn.textContent = provider === "mock" ? `Sandbox activate ${plan.title}` : `Pay with ${provider}`;
      btn.addEventListener("click", () => void startCheckout(user, plan.plan_id, provider));
      row.appendChild(meta);
      row.appendChild(btn);
      els.planList.appendChild(row);
    }
  } catch (err) {
    console.warn("billing plans:", err?.message || err);
    els.cardCloudPlans.hidden = true;
  }
}

async function syncCloudAccount(user) {
  if (!user || !els.cloudUser) return;
  try {
    const idToken = await user.getIdToken();
    const meRes = await fetch(`${CLOUD_API_BASE}/v1/me`, {
      headers: { Authorization: `Bearer ${idToken}` },
    });
    if (!meRes.ok) {
      const body = await meRes.json().catch(() => ({}));
      throw new Error(body.error || `Cloud HTTP ${meRes.status}`);
    }
    const me = await meRes.json();
    els.cloudUser.textContent = me.username || me.account_id || "—";
    const subRes = await fetch(`${CLOUD_API_BASE}/v1/me/subscription`, {
      headers: { Authorization: `Bearer ${idToken}` },
    });
    if (subRes.ok) {
      const sub = await subRes.json();
      if (els.cloudPlan) {
        const until = sub.ends_at ? ` · until ${String(sub.ends_at).slice(0, 10)}` : "";
        els.cloudPlan.textContent = `${sub.title || sub.plan_id || "—"}${until}`;
      }
    } else if (els.cloudPlan) {
      els.cloudPlan.textContent = "Free forever";
    }
    await paintBillingPlans(user, idToken);
  } catch (err) {
    if (els.cloudUser) els.cloudUser.textContent = "Unavailable";
    if (els.cloudPlan) els.cloudPlan.textContent = "—";
    if (els.cardCloudPlans) els.cardCloudPlans.hidden = true;
    // Soft-fail: Auth still works if the API host is down or CORS-blocked.
    console.warn("cloud sync:", err?.message || err);
  }
}

function renderSession(user) {
  if (user) {
    els.gate.hidden = true;
    els.session.hidden = false;
    els.userEmail.textContent =
      user.email || user.phoneNumber || user.displayName || "(signed in)";
    els.userVerified.textContent = user.email
      ? user.emailVerified
        ? "Email verified"
        : "Email not verified yet — check your inbox"
      : user.phoneNumber
        ? "Signed in with phone"
        : "Signed in";
    if (els.userProviders) {
      const labels = providerIds(user).map(providerLabel);
      els.userProviders.textContent = labels.length ? labels.join(" · ") : "None listed yet";
    }
    if (els.btnResendVerify) {
      els.btnResendVerify.hidden = !(user.email && !user.emailVerified);
    }
    paintLinkButtons(user);
    paintPasswordCard(user);
    els.status.textContent = "You’re signed in.";
    void syncCloudAccount(user);
  } else {
    els.gate.hidden = false;
    els.session.hidden = true;
    els.status.textContent = "Continue with email, a provider, or phone.";
    paintCloudPlaceholders();
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
  showOk("");
  setBusy(true);
  try {
    const email = els.email.value.trim();
    const password = els.password.value;
    const name = els.displayName.value.trim();
    try {
      const cred = await createUserWithEmailAndPassword(auth, email, password);
      if (name) await updateProfile(cred.user, { displayName: name });
      await afterAuth(cred.user, { sendVerify: true });
      showOk("Account created. Check your email to verify when you can.");
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

async function handleForgot(e) {
  e.preventDefault();
  showError("");
  showOk("");
  setBusy(true);
  try {
    const email = (els.forgotEmail?.value || els.email?.value || "").trim();
    if (!email) throw { code: "auth/invalid-email" };
    await sendPasswordResetEmail(auth, email);
    showOk("If an account exists for that email, a reset link is on its way. Check your inbox.");
    if (els.forgotPanel) els.forgotPanel.hidden = true;
  } catch (err) {
    // Avoid account enumeration: still show soft success for user-not-found.
    if (err?.code === "auth/user-not-found") {
      showOk("If an account exists for that email, a reset link is on its way. Check your inbox.");
      if (els.forgotPanel) els.forgotPanel.hidden = true;
    } else {
      showError(friendlyError(err));
    }
  } finally {
    setBusy(false);
  }
}

async function popupSignIn(provider) {
  showError("");
  showOk("");
  setBusy(true);
  try {
    const cred = await signInWithPopup(auth, provider);
    await afterAuth(cred.user);
  } catch (err) {
    if (err?.code === "auth/account-exists-with-different-credential" && err.customData?.email) {
      try {
        const methods = await fetchSignInMethodsForEmail(auth, err.customData.email);
        const pretty = methods.map(providerLabel).join(", ") || "another method";
        showError(
          `An account for ${err.customData.email} already exists via ${pretty}. Sign in that way first, then link this provider from your account page.`
        );
      } catch {
        showError(friendlyError(err));
      }
    } else {
      showError(friendlyError(err));
    }
  } finally {
    setBusy(false);
  }
}

function ensureRecaptcha() {
  if (recaptcha) return recaptcha;
  recaptcha = new RecaptchaVerifier(auth, "recaptcha-container", { size: "invisible" });
  return recaptcha;
}

function ensureLinkRecaptcha() {
  if (linkRecaptcha) return linkRecaptcha;
  linkRecaptcha = new RecaptchaVerifier(auth, "recaptcha-link-container", {
    size: "invisible",
  });
  return linkRecaptcha;
}

async function sendSms(e) {
  e.preventDefault();
  showError("");
  showOk("");
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
  showOk("");
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
  showOk("");
  await signOut(auth);
  confirmationResult = null;
  linkConfirmationResult = null;
  els.phoneStep.hidden = false;
  els.smsStep.hidden = true;
  if (els.formLinkSms) els.formLinkSms.hidden = true;
  if (els.password) els.password.value = "";
  if (els.smsCode) els.smsCode.value = "";
}

async function reauthForSensitive(user) {
  const ids = providerIds(user);
  if (ids.includes("password")) {
    const current = els.currentPassword?.value || "";
    if (!current) throw { message: "Enter your current password." };
    const cred = EmailAuthProvider.credential(user.email, current);
    await reauthenticateWithCredential(user, cred);
    return;
  }
  // Prefer an OAuth provider already on the account.
  if (ids.includes("google.com")) {
    await reauthenticateWithPopup(user, googleProvider());
    return;
  }
  if (ids.includes("microsoft.com")) {
    await reauthenticateWithPopup(user, microsoftProvider());
    return;
  }
  if (ids.includes("github.com")) {
    await reauthenticateWithPopup(user, githubProvider());
    return;
  }
  throw { message: "Sign out and sign in again, then retry." };
}

async function handlePasswordSave(e) {
  e.preventDefault();
  showError("");
  showOk("");
  const user = auth.currentUser;
  if (!user) return;
  const next = els.newPassword?.value || "";
  const next2 = els.newPassword2?.value || "";
  if (next.length < 6) {
    showError("Use a password with at least 6 characters.");
    return;
  }
  if (next !== next2) {
    showError("New password and confirmation do not match.");
    return;
  }
  setBusy(true);
  try {
    if (hasPassword(user)) {
      await reauthForSensitive(user);
      await updatePassword(user, next);
      showOk("Password updated.");
    } else {
      const email = (els.setEmail?.value || user.email || "").trim();
      if (!email) {
        showError("Enter an email to use with password sign-in.");
        return;
      }
      const cred = EmailAuthProvider.credential(email, next);
      await linkWithCredential(user, cred);
      if (!user.emailVerified) {
        try {
          await sendEmailVerification(user);
        } catch {
          /* ignore */
        }
      }
      showOk("Password added. You can also sign in with email now.");
    }
    paintPasswordCard(auth.currentUser);
    paintLinkButtons(auth.currentUser);
    renderSession(auth.currentUser);
  } catch (err) {
    showError(friendlyError(err));
  } finally {
    setBusy(false);
  }
}

async function linkProvider(provider) {
  showError("");
  showOk("");
  const user = auth.currentUser;
  if (!user) return;
  setBusy(true);
  try {
    await linkWithPopup(user, provider);
    showOk("Sign-in method linked.");
    renderSession(auth.currentUser);
  } catch (err) {
    showError(friendlyError(err));
  } finally {
    setBusy(false);
  }
}

async function sendLinkSms(e) {
  e.preventDefault();
  showError("");
  showOk("");
  const user = auth.currentUser;
  if (!user) return;
  setBusy(true);
  try {
    const phone = els.linkPhone.value.trim();
    const verifier = ensureLinkRecaptcha();
    linkConfirmationResult = await linkWithPhoneNumber(user, phone, verifier);
    $("form-link-phone").hidden = true;
    els.formLinkSms.hidden = false;
    els.linkSmsCode.focus();
  } catch (err) {
    showError(friendlyError(err));
    try {
      linkRecaptcha?.clear();
    } catch {
      /* ignore */
    }
    linkRecaptcha = null;
  } finally {
    setBusy(false);
  }
}

async function confirmLinkSms(e) {
  e.preventDefault();
  showError("");
  showOk("");
  setBusy(true);
  try {
    const code = els.linkSmsCode.value.trim();
    await linkConfirmationResult.confirm(code);
    showOk("Phone linked to this account.");
    $("form-link-phone").hidden = false;
    els.formLinkSms.hidden = true;
    els.linkPhone.value = "";
    els.linkSmsCode.value = "";
    renderSession(auth.currentUser);
  } catch (err) {
    showError(friendlyError(err));
  } finally {
    setBusy(false);
  }
}

function wire() {
  $("form-email")?.addEventListener("submit", handleEmail);
  $("form-forgot")?.addEventListener("submit", handleForgot);
  $("btn-forgot-toggle")?.addEventListener("click", () => {
    const open = els.forgotPanel?.hidden;
    if (els.forgotPanel) els.forgotPanel.hidden = !open;
    if (open && els.forgotEmail) {
      els.forgotEmail.value = els.email?.value?.trim() || "";
      els.forgotEmail.focus();
    }
    showError("");
    showOk("");
  });
  $("btn-forgot-cancel")?.addEventListener("click", () => {
    if (els.forgotPanel) els.forgotPanel.hidden = true;
  });

  $("btn-google")?.addEventListener("click", () => popupSignIn(googleProvider()));
  $("btn-github")?.addEventListener("click", () => popupSignIn(githubProvider()));
  $("btn-microsoft")?.addEventListener("click", () => popupSignIn(microsoftProvider()));

  $("form-phone")?.addEventListener("submit", sendSms);
  $("form-sms")?.addEventListener("submit", confirmSms);
  $("btn-signout")?.addEventListener("click", doSignOut);
  $("form-password")?.addEventListener("submit", handlePasswordSave);

  els.btnLinkGoogle?.addEventListener("click", () => linkProvider(googleProvider()));
  els.btnLinkMicrosoft?.addEventListener("click", () => linkProvider(microsoftProvider()));
  els.btnLinkGithub?.addEventListener("click", () => linkProvider(githubProvider()));
  $("form-link-phone")?.addEventListener("submit", sendLinkSms);
  $("form-link-sms")?.addEventListener("submit", confirmLinkSms);

  els.btnResendVerify?.addEventListener("click", async () => {
    const u = auth.currentUser;
    if (!u?.email) return;
    showError("");
    showOk("");
    try {
      await sendEmailVerification(u);
      showOk("Verification email sent — check your inbox.");
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
