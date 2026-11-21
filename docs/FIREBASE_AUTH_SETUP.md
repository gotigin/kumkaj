# Firebase Auth setup (product websites)

Each Kumkaj **product** site has its own Firebase project and Auth.
Kumkaj products each own a domain (`khaynjo.com`, `moklanjo.com`,
`kirano.shop`, `tabiyat.shop`), so Auth is enabled **per product site** —
not one shared brand login. (Wadhu’s products do not each have their own
domain; Auth there lives on the Wadhu site instead.)

This is site Auth for register / sign-in — not Wadhu Connect.

Cloud sync / one logical shop DB / offline guarantees: working discussion
in [`CLOUD_AND_SYNC_DISCUSSION.md`](CLOUD_AND_SYNC_DISCUSSION.md).

| Product | Domain | Firebase project |
| --- | --- | --- |
| Khaynjo | khaynjo.com | `khaynjo-2b2a2` |
| Moklanjo | moklanjo.com | `moklanjo-com` |
| Kirano | kirano.shop | `kirano-shop` |
| Tabiyat | tabiyat.shop | `tabiyat-shop` |

## Enabled now (same provider set as Wadhu site Auth)

| Method | Status |
| --- | --- |
| Email / password | On |
| Google | On |
| Phone | On (SMS needs Blaze billing on the project) |
| GitHub | On (Gotigin org OAuth app; secrets in `local/product-auth/`) |
| Microsoft | On (Entra app; secrets in `local/product-auth/`) |
| Apple | Postponed (same reason as Wadhu — Apple Developer membership) |

## Site UI (`/account`)

Per site under `sites/<product>/`:

- Page: `static/account/index.html`
- Client: `client/account-app.js` + `client/firebase-init.js`
- Build: `npm run build` → `static/js/account.js` + `firebase-init.js`
- Deploy: `./sites/<product>/scripts/deploy-firebase.sh`

`authDomain` is the **custom domain** (not `*.firebaseapp.com` from the
Console snippet) so popups/redirects use `https://<domain>/__/auth/handler`.
Google redirect URIs list the custom handler first.

### Auth UX covered (Firebase; Apple still postponed)

| Flow | How |
| --- | --- |
| Register / sign-in (email) | One Continue form (`create` then fall back to `signIn`) |
| Federated | Google, Microsoft, GitHub popups |
| Phone | SMS + invisible reCAPTCHA |
| Email verification | Sent on signup; resend when signed in |
| Forgot password | `sendPasswordResetEmail` (soft copy; no account enumeration) |
| Change password | Re-auth (password or linked OAuth) then `updatePassword` |
| Add password | `linkWithCredential` when account has only OAuth/phone |
| Link more methods | `linkWithPopup` / `linkWithPhoneNumber` while signed in |
| Existing email, other method | Friendly `account-exists-with-different-credential` + `fetchSignInMethodsForEmail` |
| Sign out | `signOut` |

### Native apps (Lock screen)

Each Makepad app owns a copy of Identity Toolkit REST + cloud API client
(`apps/<product>/src/cloud_auth.rs`, `cloud_api.rs`) — same Firebase project
as that product’s site. Lock screen:

- **Continue** — email register-then-sign-in (same as site), then
  `GET /v1/me` (ensure account) + subscription/plans.
- **Forgot password?** — `sendOobCode` PASSWORD_RESET (soft success).
- **Google and other methods on the website** — opens
  `https://<domain>/account` in the system browser (no in-app OAuth popup).
- Tokens in sidecar `cloud_auth.secret` next to the books DB (wiped with
  uninstall); non-secret `cloud_*` meta in SQLite.
- Override API host: `<PRODUCT>_CLOUD_API_BASE` (Fargate task IPs change).
- Local PIN + shop staff username unlock stay offline forever.

Do not invent a second auth system.

## Owner steps

1. **OAuth callbacks** — on the Gotigin GitHub and Microsoft OAuth apps, add
   each product’s `https://<domain>/__/auth/handler` (and the
   `*.firebaseapp.com` fallback if needed). Prefer the custom domain.
2. **Kirano / Tabiyat Hosting** — attach `kirano.shop` / `tabiyat.shop` in
   Firebase Console → Hosting, then deploy.
3. **Blaze** — Phone SMS requires Blaze on that Firebase project (same as Wadhu).

## Deploy Auth providers

```bash
cd sites/khaynjo && firebase deploy --only auth
# …repeat per site, or use Identity Toolkit admin for phone / IdPs
```

Federated IdP secrets: gitignored [`local/product-auth/`](../local/product-auth/).
