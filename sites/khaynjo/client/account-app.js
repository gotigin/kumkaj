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

/** Site copy in the picked language (static/js/site-i18n.js). Keys live in
 * src/locale.rs EN (the locale tests compare every call here with EN);
 * English until the pack is here, and `site:i18n` repaints what was written. */
const T = (key, english) => (window.siteT ? window.siteT(key, english) : english);

/** Override via `window.__KUMKAJ_CLOUD_API__` (no trailing slash). */
const CLOUD_API_BASE =
  (typeof window !== "undefined" && window.__KUMKAJ_CLOUD_API__) ||
  "https://api.khaynjo.com";

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

const NEXT_KEY = "khaynjo.auth.next";

/** Same-origin relative path only — never open redirect. */
function safeNext(raw) {
  if (!raw || typeof raw !== "string") return null;
  const path = raw.trim();
  if (!path.startsWith("/") || path.startsWith("//")) return null;
  if (path === "/account" || path.startsWith("/account?") || path.startsWith("/account/")) {
    return null;
  }
  return path;
}

function captureReturnTo() {
  const u = new URL(window.location.href);
  let next = safeNext(u.searchParams.get("next"));
  if (!next) {
    try {
      next = safeNext(sessionStorage.getItem(NEXT_KEY));
    } catch (_) {}
  }
  if (!next && document.referrer) {
    try {
      const ref = new URL(document.referrer);
      if (ref.origin === window.location.origin) {
        next = safeNext(ref.pathname + ref.search + ref.hash);
      }
    } catch (_) {}
  }
  if (next) {
    try {
      sessionStorage.setItem(NEXT_KEY, next);
    } catch (_) {}
  }
  return next;
}

function consumeReturnTo() {
  let next = null;
  try {
    next = safeNext(sessionStorage.getItem(NEXT_KEY));
    sessionStorage.removeItem(NEXT_KEY);
  } catch (_) {}
  if (!next) {
    next = safeNext(new URL(window.location.href).searchParams.get("next"));
  }
  return next;
}

function markHomeWarm() {
  try {
    sessionStorage.setItem("khaynjo.boot.warm", "1");
  } catch (_) {}
}

/** Prefer history.back() to home so bfcache restores the page — no boot replay. */
function goAfterAuth() {
  const next = consumeReturnTo();
  if (!next) return false;
  markHomeWarm();
  try {
    if (document.referrer) {
      const ref = new URL(document.referrer);
      if (ref.origin === window.location.origin) {
        const refPath = ref.pathname || "/";
        const nextPath = next.split("#")[0].split("?")[0] || "/";
        const refIsHome = refPath === "/" || refPath === "";
        const nextIsHome = nextPath === "/" || nextPath === "";
        if (nextIsHome && refIsHome) {
          history.back();
          return true;
        }
        if (ref.pathname + ref.search === next.split("#")[0]) {
          history.back();
          return true;
        }
      }
    }
  } catch (_) {}
  window.location.assign(next);
  return true;
}

/** ← Khaynjo: back to bfcache home when we came from there. */
function wireSoftHomeLinks() {
  document.querySelectorAll('a[href="/"], a[href="https://khaynjo.com/"]').forEach((a) => {
    a.addEventListener("click", (e) => {
      markHomeWarm();
      try {
        if (!document.referrer) return;
        const ref = new URL(document.referrer);
        if (ref.origin !== window.location.origin) return;
        if (ref.pathname !== "/" && ref.pathname !== "") return;
        e.preventDefault();
        history.back();
      } catch (_) {}
    });
  });
}

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
    "auth/invalid-email": T("acct_err_invalid_email", "Enter a valid email address."),
    "auth/weak-password": T("acct_err_weak_password", "Use a password with at least 6 characters."),
    "auth/wrong-password": T("acct_err_wrong_password", "Wrong password."),
    "auth/invalid-credential": T("acct_err_invalid_credential", "Email or password is wrong."),
    "auth/popup-closed-by-user": T("acct_err_popup_closed", "Sign-in window was closed."),
    "auth/cancelled-popup-request": T("acct_err_popup_cancelled", "Sign-in was cancelled."),
    "auth/unauthorized-domain": T("acct_err_unauthorized_domain", "This domain is not authorized for Firebase Auth."),
    "auth/operation-not-allowed": T("acct_err_method_disabled", "That sign-in method is not enabled yet."),
    "auth/too-many-requests": T("acct_err_too_many", "Too many attempts. Try again later."),
    "auth/invalid-phone-number": T("acct_err_phone_format", "Use a full phone number with country code (e.g. +91…)."),
    "auth/invalid-verification-code": T("acct_err_sms_code", "That SMS code is wrong or expired."),
    "auth/user-not-found": T("acct_err_no_account", "No account with that email. Continue to create one, or use another method."),
    "auth/requires-recent-login": T("acct_err_recent_login", "For security, sign in again, then retry this change."),
    "auth/credential-already-in-use": T("acct_err_credential_in_use", "That sign-in method is already used by another account."),
    "auth/email-already-in-use": T("acct_err_email_in_use", "That email is already used by another account."),
    "auth/provider-already-linked": T("acct_err_already_linked", "That method is already linked to this account."),
    "auth/account-exists-with-different-credential":
      T("acct_err_exists_other_method", "An account already exists with this email using a different sign-in method. Sign in with that method first, then link this one from your account page."),
  };
  return map[code] || err?.message || T("acct_err_generic", "Something went wrong.");
}

function providerIds(user) {
  return (user?.providerData || []).map((p) => p.providerId);
}

function hasPassword(user) {
  return providerIds(user).includes("password");
}

function providerLabel(id) {
  const map = {
    password: T("acct_method_password", "Email & password"),
    "google.com": "Google",
    "github.com": "GitHub",
    "microsoft.com": "Microsoft",
    phone: T("acct_phone", "Phone"),
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

/** Hint and button words of the password card. */
function paintPasswordText(user) {
  const withPw = hasPassword(user);
  if (els.passwordHint) {
    els.passwordHint.textContent = withPw
      ? T("acct_pw_hint_change", "Change your password. You’ll confirm with the current one.")
      : T("acct_pw_hint_add", "Add a password so you can also sign in with email (useful if a provider is unavailable).");
  }
  if (els.btnPasswordSave) {
    els.btnPasswordSave.textContent = withPw ? T("acct_pw_change", "Change password") : T("acct_pw_add", "Add password");
  }
}

function paintPasswordCard(user) {
  const withPw = hasPassword(user);
  paintPasswordText(user);
  if (els.passwordCurrentWrap) els.passwordCurrentWrap.hidden = !withPw;
  if (els.passwordEmailWrap) els.passwordEmailWrap.hidden = withPw;
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


function pendingCheckoutPlan() {
  try {
    const u = new URL(window.location.href);
    return (u.searchParams.get("checkout") || "").trim().toLowerCase();
  } catch {
    return "";
  }
}

async function maybeAutoCheckout(user) {
  const plan = pendingCheckoutPlan();
  if (!user || !plan || !["steady", "swift", "priority"].includes(plan)) return;
  // Clear query so refresh does not loop.
  try {
    const u = new URL(window.location.href);
    u.searchParams.delete("checkout");
    window.history.replaceState({}, "", u.pathname + u.search + u.hash);
  } catch { /* ignore */ }
  const cfgRes = await fetch(`${CLOUD_API_BASE}/v1/billing/config`, {
    headers: { Authorization: `Bearer ${await user.getIdToken()}` },
  });
  if (!cfgRes.ok) return;
  const cfg = await cfgRes.json();
  if (!cfg.enabled) {
    showError(T("acct_billing_off", "Billing is not enabled on the cloud API yet."));
    return;
  }
  const providers = cfg.providers || [];
  const provider =
    (cfg.default_provider && providers.includes(cfg.default_provider) && cfg.default_provider) ||
    providers.find((p) => p !== "mock") ||
    null;
  if (!provider) {
    showError(T("acct_billing_no_provider", "No real billing provider is enabled on the cloud API (mock-only is local)."));
    return;
  }
  await startCheckout(user, plan, provider);
}

function formatInrPaise(paise) {
  if (paise == null) return T("price_quote", "Quote");
  if (paise === 0) return "₹0";
  return T("price_per_year", "₹{amount}/yr ex-tax").replace("{amount}", (paise / 100).toLocaleString("en-IN"));
}

function loadScriptOnce(src) {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[src="${src}"]`);
    if (existing) {
      if (existing.dataset.loaded === "1") resolve();
      else existing.addEventListener("load", () => resolve(), { once: true });
      return;
    }
    const s = document.createElement("script");
    s.src = src;
    s.async = true;
    s.onload = () => {
      s.dataset.loaded = "1";
      resolve();
    };
    s.onerror = () => reject(new Error(`Failed to load ${src}`));
    document.head.appendChild(s);
  });
}

/** Paddle.js overlay — keeps the account page; no full navigation. */
async function openPaddleOverlay(user, transactionId, billingCfg) {
  const token = billingCfg?.paddle_client_token;
  if (!token) throw new Error("Paddle client token missing from billing config");
  await loadScriptOnce("https://cdn.paddle.com/paddle/v2/paddle.js");
  const Paddle = window.Paddle;
  if (!Paddle) throw new Error("Paddle.js failed to initialize");
  window.__kumkajCheckoutPaid = false;
  window.__kumkajCheckoutFailed = false;
  if (!window.__kumkajPaddleReady) {
    if ((billingCfg.paddle_env || billingCfg.mode) === "sandbox") {
      Paddle.Environment.set("sandbox");
    }
    Paddle.Initialize({
      token,
      eventCallback(ev) {
        const name = ev?.name || ev?.type || "";
        if (name === "checkout.completed") {
          window.__kumkajCheckoutPaid = true;
          void syncCloudAccount(user);
          showError("");
          showOk(T("acct_pay_received", "Payment received — refreshing your plan…"));
        }
        if (
          name === "checkout.error" ||
          name === "checkout.payment.failed" ||
          name === "checkout.payment.rejected"
        ) {
          window.__kumkajCheckoutFailed = true;
          const detail =
            ev?.error?.detail ||
            ev?.data?.error?.detail ||
            ev?.data?.message ||
            T("acct_pay_declined", "Payment was declined or failed");
          showOk("");
          showError(T("acct_pay_failed_detail", "{detail}. Your plan was not changed.").replace("{detail}", detail));
          setBusy(false);
        }
        if (name === "checkout.closed") {
          setBusy(false);
          if (!window.__kumkajCheckoutPaid && !window.__kumkajCheckoutFailed) {
            showOk("");
            showError(T("acct_checkout_closed", "Checkout closed — no payment completed. Your plan is unchanged."));
          }
        }
      },
    });
    window.__kumkajPaddleReady = true;
  }
  Paddle.Checkout.open({
    transactionId,
    settings: { displayMode: "overlay", theme: "light" },
  });
}

async function startCheckout(user, planId, provider) {
  if (!user) return;
  setBusy(true);
  showError("");
  try {
    const idToken = await user.getIdToken();
    const [cfgRes, res] = await Promise.all([
      fetch(`${CLOUD_API_BASE}/v1/billing/config`, {
        headers: { Authorization: `Bearer ${idToken}` },
      }),
      fetch(`${CLOUD_API_BASE}/v1/billing/checkout`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${idToken}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ plan_id: planId, provider }),
      }),
    ]);
    const billingCfg = await cfgRes.json().catch(() => ({}));
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Checkout HTTP ${res.status}`);
    if (provider === "mock" && body.checkout_id) {
      // Unpaid mock activate is local-only; API must advertise mock in providers.
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
    if (provider === "paddle") {
      const txn =
        body.transaction_id ||
        (typeof body.url === "string"
          ? new URL(body.url, window.location.origin).searchParams.get("_ptxn")
          : null);
      if (!txn) throw new Error("Checkout missing Paddle transaction id");
      await openPaddleOverlay(user, txn, billingCfg);
      return;
    }
    throw new Error("Checkout did not return a usable payment session");
  } catch (err) {
    showError(err?.message || T("acct_checkout_failed", "Checkout failed"));
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
        "<p class=\"auth-hint\">No self-serve plans are checkout-ready yet. For Paddle, set paddle_price_id on the plans table (see BILLING_SANDBOX.md).</p>";
      return;
    }
    const providers = cfg.providers || [];
    const provider =
      (cfg.default_provider && providers.includes(cfg.default_provider) && cfg.default_provider) ||
      providers.find((p) => p !== "mock") ||
      null;
    if (!provider) {
      els.planList.innerHTML =
        "<p class=\"auth-hint\">No payable provider is enabled (Paddle). Mock activate is local-only.</p>";
      return;
    }
    for (const plan of sellable) {
      const row = document.createElement("div");
      row.className = "plan-row";
      const meta = document.createElement("div");
      meta.innerHTML = `<strong>${plan.title}</strong><span>${formatInrPaise(plan.price_inr_paise)} · ${plan.quota_gb || "—"} GB</span>`;
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "btn btn-primary";
      btn.textContent = provider === "mock" ? `Sandbox activate ${plan.title}` : T("acct_pay_with", "Pay with {provider}").replace("{provider}", provider);
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
        const planName = sub.title || sub.plan_id || "—";
        els.cloudPlan.textContent = sub.ends_at
          ? T("acct_plan_until", "{plan} · until {date}")
              .replace("{plan}", planName)
              .replace("{date}", String(sub.ends_at).slice(0, 10))
          : planName;
      }
    } else if (els.cloudPlan) {
      els.cloudPlan.textContent = T("plan_free_title", "Free forever");
    }
    await paintBillingPlans(user, idToken);
    await maybeAutoCheckout(user);
  } catch (err) {
    if (els.cloudUser) els.cloudUser.textContent = T("acct_unavailable", "Unavailable");
    if (els.cloudPlan) els.cloudPlan.textContent = "—";
    if (els.cardCloudPlans) els.cardCloudPlans.hidden = true;
    // Soft-fail: Auth still works if the API host is down or CORS-blocked.
    console.warn("cloud sync:", err?.message || err);
  }
}

/** Words of the signed-in / signed-out view; `site:i18n` repaints them. */
function paintSessionText(user) {
  if (!user) {
    els.status.textContent = T("acct_continue_hint", "Continue with email, a provider, or phone.");
    return;
  }
  els.userEmail.textContent =
    user.email || user.phoneNumber || user.displayName || T("acct_signed_in_fallback", "(signed in)");
  els.userVerified.textContent = user.email
    ? user.emailVerified
      ? T("acct_email_verified", "Email verified")
      : T("acct_email_unverified", "Email not verified yet — check your inbox")
    : user.phoneNumber
      ? T("acct_signed_in_phone", "Signed in with phone")
      : T("acct_signed_in", "Signed in");
  if (els.userProviders) {
    const labels = providerIds(user).map(providerLabel);
    els.userProviders.textContent = labels.length ? labels.join(" · ") : T("acct_none_listed", "None listed yet");
  }
  paintPasswordText(user);
  els.status.textContent = T("acct_signed_in_status", "You’re signed in.");
}

function renderSession(user) {
  if (user) {
    els.gate.hidden = true;
    els.session.hidden = false;
    if (els.btnResendVerify) {
      els.btnResendVerify.hidden = !(user.email && !user.emailVerified);
    }
    paintLinkButtons(user);
    paintPasswordCard(user);
    paintSessionText(user);
    void syncCloudAccount(user);
  } else {
    els.gate.hidden = false;
    els.session.hidden = true;
    paintSessionText(null);
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
  // Came from another site page → send them back. Direct /account visits stay here.
  if (goAfterAuth()) return;
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
      showOk(T("acct_created", "Account created. Check your email to verify when you can."));
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
    showOk(T("acct_reset_sent", "If an account exists for that email, a reset link is on its way. Check your inbox."));
    if (els.forgotPanel) els.forgotPanel.hidden = true;
  } catch (err) {
    // Avoid account enumeration: still show soft success for user-not-found.
    if (err?.code === "auth/user-not-found") {
      showOk(T("acct_reset_sent", "If an account exists for that email, a reset link is on its way. Check your inbox."));
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
        const pretty = methods.map(providerLabel).join(", ") || T("acct_another_method", "another method");
        showError(
          T("acct_exists_via", "An account for {email} already exists via {method}. Sign in that way first, then link this provider from your account page.")
            .replace("{email}", err.customData.email)
            .replace("{method}", pretty)
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
    if (!current) throw { message: T("acct_enter_current_pw", "Enter your current password.") };
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
  throw { message: T("acct_relogin_retry", "Sign out and sign in again, then retry.") };
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
    showError(T("acct_err_weak_password", "Use a password with at least 6 characters."));
    return;
  }
  if (next !== next2) {
    showError(T("acct_pw_mismatch", "New password and confirmation do not match."));
    return;
  }
  setBusy(true);
  try {
    if (hasPassword(user)) {
      await reauthForSensitive(user);
      await updatePassword(user, next);
      showOk(T("acct_pw_updated", "Password updated."));
    } else {
      const email = (els.setEmail?.value || user.email || "").trim();
      if (!email) {
        showError(T("acct_enter_email_pw", "Enter an email to use with password sign-in."));
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
      showOk(T("acct_pw_added", "Password added. You can also sign in with email now."));
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
    showOk(T("acct_method_linked", "Sign-in method linked."));
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
    showOk(T("acct_phone_linked", "Phone linked to this account."));
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
  wireSoftHomeLinks();

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
      showOk(T("acct_verify_sent", "Verification email sent — check your inbox."));
      els.userVerified.textContent = T("acct_verify_sent", "Verification email sent — check your inbox.");
    } catch (err) {
      showError(friendlyError(err));
    }
  });

  captureReturnTo();

  // Language picked or its pack arrived (site-i18n.js): repaint the words this
  // script wrote. No network: the cloud / checkout calls are not repeated.
  window.addEventListener("site:i18n", () => paintSessionText(auth.currentUser));

  onAuthStateChanged(auth, (user) => {
    // Already signed in and arrived with ?next= → continue to that page.
    if (user) {
      const params = new URL(window.location.href).searchParams;
      if (params.has("next") && goAfterAuth()) return;
    }
    renderSession(user);
  });
}

wire();
