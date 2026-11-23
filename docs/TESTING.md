# Testing principles (Gotigin / Kumkaj)

**Canonical rule for humans and agents.** Tests exist to protect product intent.
They do **not** exist so we can declare victory by making a check go green.

Incident that triggered this doc (2026-11-23): an agent **removed Stripe
`automatic_tax`** so a smoke would pass. That violated tax-at-gateway intent.
Research later showed Stripe Tax cannot serve an India-based merchant at all;
**Stripe was removed** from Kumkaj billing in favor of Paddle. This document
still forbids gutting requirements to green a check.

---

## 1. What tests are for

| Aim | Meaning |
| --- | --- |
| **Prove real behavior** | The app, site, or API still does what product docs and owners require. |
| **Fail loudly when intent breaks** | A red check means “fix the product or the environment,” not “mute the assertion.” |
| **Gate ship** | Certs / smokes / CI block release when requirements are unmet. |
| **Document contracts** | A test name and assertion should read like a requirement. |

Tests serve the product. **The product must never be rewritten to serve the test.**

---

## 2. Absolute prohibitions

Do **not**, for the sake of a green result:

1. **Disable, stub, or omit a product requirement** (tax, auth, payment, webhooks, quotas, privacy, offline rules, etc.).
2. **Comment out or weaken assertions** so a known broken path “passes.”
3. **Ship feature flags / env defaults that fake success in production or public sandbox** (e.g. unpaid “mock activate” on public APIs without an explicit local-only gate).
4. **Rewrite docs** to say a bypass is acceptable when product docs still require the real behavior.
5. **Treat harness-only success as product-ready** (e.g. `SMOKE PASSED (mock-only)` ≠ Stripe/Paddle/tax ready).
6. **Add `#[ignore]`, skip, or soft-assert** on a requirement test just to clear CI — unless the test is explicitly a manual/device soak and the skip reason is honest and temporary with an owner ticket.

If the only way to green a check is to gut the product, **leave the check red** and fix the real blocker (config, gateway, code, or docs that were wrong).

---

## 3. Allowed harnesses (honest, local, labeled)

These are OK **only** when clearly scoped and never mistaken for full product proof:

| Harness | Allowed when | Must |
| --- | --- | --- |
| Mock billing / `BILLING_ALLOW_MOCK=1` | Local Docker smoke | Stay off AWS / public Hosting; output must say **mock-only** |
| `DEV_AUTH_BYPASS` | Debug builds + local smoke | Never on AWS / release; not a substitute for Firebase Auth in product paths |
| UI smoke on Xvfb | Automated desk flows | Fail on real stalls / wrong UI; no “pid alive = pass” |
| Product cert tiers | Ship gates | See `product-cert.sh` / DEV.md — do not skip store tiers casually |

A harness proves **that harness’s contract**. It does not prove tax, live payment, or store readiness unless it explicitly exercises those.

---

## 4. When a test fails for a good reason

1. **Read the failure** — is the product wrong, the env incomplete, or the test wrong?
2. If **product is wrong** → fix product to match docs / owner intent.
3. If **env is incomplete** (e.g. Stripe Tax not registered) → **keep the failure**; complete setup; or use a gateway that can fulfill the same requirement. Do not delete the requirement.
4. If **test is wrong** (asserts something docs never required) → fix or **delete** the bogus test. Do not change the product to match a bad test.
5. If **test only exists to look green** → **remove it**. Fooling ourselves is worse than no test.

---

## 5. Bogus tests — remove them

Delete or rewrite a test that:

- Passes while the user-visible / API behavior still violates docs.
- Passes by asserting implementation trivia unrelated to intent.
- Was added only after softening production code.
- Soft-fails (`warn` / `continue`) on critical paths without failing the job.

Prefer one strict failing test over ten green lies.

---

## 6. Naming and reporting

- Prefer names that state intent: `checkout_applies_gateway_tax`, not `checkout_ok`.
- Smoke / cert output must distinguish levels, e.g.:
  - `PASSED (mock-only local harness)`
  - `PASSED (Stripe Tax + signed webhook)`
  - `FAILED — stripe_tax_inactive (requirement unmet)`
- Never rename a mock pass to imply gateway or store readiness.

---

## 7. Agents and reviewers

- Before changing product code to clear a red check, ask: **“Would a paying customer still get the required behavior?”** If no, do not change the product that way.
- Cursor rule: [`.cursor/rules/tests-serve-product.mdc`](../.cursor/rules/tests-serve-product.mdc) (`alwaysApply`).
- Billing/tax requirements: [`CLOUD_PLANS.md`](CLOUD_PLANS.md), per-cloud `docs/BILLING_SANDBOX.md`.
- Ship gates: Product Certification Kit in [`DEV.md`](DEV.md).

---

## 8. Short pledge

> We write tests so the product stays honest.  
> We do not rewrite the product so the tests look honest.
