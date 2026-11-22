// Kumkaj Cloud shop + cart (product sites). Checkout requires /account sign-in.
import { CLOUD_CATALOG, formatInrPaise } from "./shop-catalog.js";

const $ = (id) => document.getElementById(id);

const PRODUCT =
  (typeof window !== "undefined" && window.__KUMKAJ_PRODUCT__) || "khaynjo";
const PRICES_READY =
  typeof window !== "undefined" ? window.__KUMKAJ_PRICES_READY__ !== false : true;
const SUPPORT_MAIL =
  (typeof window !== "undefined" && window.__KUMKAJ_SUPPORT_MAIL__) ||
  "support@gotigin.com";
const ACCOUNT_HREF =
  (typeof window !== "undefined" && window.__KUMKAJ_ACCOUNT_HREF__) || "/account";

const CART_KEY = `kumkaj.cloud.cart.${PRODUCT}`;

function loadCart() {
  try {
    const raw = localStorage.getItem(CART_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function saveCart(ids) {
  localStorage.setItem(CART_KEY, JSON.stringify(ids));
}

function showMsg(el, text, kind) {
  if (!el) return;
  el.textContent = text || "";
  el.hidden = !text;
  el.classList.toggle("shop-error", kind === "error");
  el.classList.toggle("shop-ok", kind === "ok");
}

function paintCatalog() {
  const root = $("shop-catalog");
  if (!root) return;
  root.innerHTML = "";
  for (const item of CLOUD_CATALOG) {
    const card = document.createElement("article");
    card.className = "shop-card";
    card.dataset.planId = item.id;
    const priceLabel = !PRICES_READY && item.sellable
      ? "Pricing coming soon"
      : formatInrPaise(item.priceInrPaise);
    const actions = document.createElement("div");
    actions.className = "shop-card-actions";
    if (item.quoteOnly) {
      const a = document.createElement("a");
      a.className = "btn btn-ghost";
      a.href = `mailto:${SUPPORT_MAIL}?subject=${encodeURIComponent(
        `${PRODUCT} Custom cloud quote`
      )}`;
      a.textContent = "Request quote";
      actions.appendChild(a);
    } else if (item.sellable) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "btn btn-primary";
      btn.textContent = PRICES_READY ? "Add to cart" : "Notify me later";
      btn.disabled = !PRICES_READY;
      btn.addEventListener("click", () => addToCart(item.id));
      actions.appendChild(btn);
    } else {
      const span = document.createElement("span");
      span.className = "shop-pill";
      span.textContent = "Included";
      actions.appendChild(span);
    }
    card.innerHTML = `<h2>${item.title}</h2><p class="shop-blurb">${item.blurb}</p><p class="shop-price">${priceLabel}</p>`;
    card.appendChild(actions);
    root.appendChild(card);
  }
}

function addToCart(planId) {
  const item = CLOUD_CATALOG.find((p) => p.id === planId);
  if (!item?.sellable || !PRICES_READY) return;
  // One cloud plan at a time (subscription replace).
  saveCart([planId]);
  paintCart();
  showMsg($("shop-status"), `${item.title} is in your cart.`, "ok");
}

function removeFromCart(planId) {
  saveCart(loadCart().filter((id) => id !== planId));
  paintCart();
}

function paintCart() {
  const root = $("shop-cart-lines");
  const checkout = $("shop-checkout");
  const empty = $("shop-cart-empty");
  if (!root) return;
  const ids = loadCart();
  root.innerHTML = "";
  if (!ids.length) {
    if (empty) empty.hidden = false;
    if (checkout) checkout.disabled = true;
    return;
  }
  if (empty) empty.hidden = true;
  if (checkout) checkout.disabled = !PRICES_READY;
  for (const id of ids) {
    const item = CLOUD_CATALOG.find((p) => p.id === id);
    if (!item) continue;
    const row = document.createElement("div");
    row.className = "shop-cart-row";
    row.innerHTML = `<strong>${item.title}</strong><span>${formatInrPaise(
      item.priceInrPaise
    )}</span>`;
    const rm = document.createElement("button");
    rm.type = "button";
    rm.className = "btn btn-ghost";
    rm.textContent = "Remove";
    rm.addEventListener("click", () => removeFromCart(id));
    row.appendChild(rm);
    root.appendChild(row);
  }
}

function goCheckout() {
  const ids = loadCart();
  if (!ids.length) {
    showMsg($("shop-status"), "Cart is empty.", "error");
    return;
  }
  if (!PRICES_READY) {
    showMsg(
      $("shop-status"),
      "Self-serve prices are not published for this product yet. Request a Custom quote or check back later.",
      "error"
    );
    return;
  }
  const plan = ids[0];
  const url = `${ACCOUNT_HREF}?checkout=${encodeURIComponent(plan)}`;
  window.location.href = url;
}

function boot() {
  paintCatalog();
  paintCart();
  $("shop-checkout")?.addEventListener("click", goCheckout);
  $("shop-clear")?.addEventListener("click", () => {
    saveCart([]);
    paintCart();
    showMsg($("shop-status"), "Cart cleared.", "ok");
  });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", boot);
} else {
  boot();
}
