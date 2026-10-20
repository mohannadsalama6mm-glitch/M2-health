# M² Health Website and Trial Portal

This package is intentionally separate from the Tauri desktop frontend. It contains public marketing pages, Supabase Auth, the customer portal, and versioned release delivery.

## Local development

1. Copy `.env.example` to `.env.local`.
2. Set the public Supabase project URL and modern `sb_publishable_...` key. Never place an `sb_secret_...`, signing key, or service credential in a `VITE_` variable.
3. Set `VITE_SITE_URL` to the exact local or deployed origin used by Supabase Auth redirects.
4. Run `npm install`, then `npm run dev`.

If Supabase variables are absent, the marketing and download pages remain usable while account actions fail closed with an explicit configuration message. No local account or persistence simulation is used.

## Production build

```text
npm run lint
npm test
npm run build
```

Deploy `website/dist` as an SPA. `vercel.json` provides history fallback while preserving static release and asset paths. Equivalent Cloudflare Pages or static hosting is supported if it serves `/releases/*` byte-for-byte and routes other unknown paths to `index.html`.

Required public environment:

- `VITE_SUPABASE_URL`
- `VITE_SUPABASE_PUBLISHABLE_KEY`
- `VITE_SITE_URL`
- `VITE_SUPPORT_EMAIL`

Required Supabase Edge Function secrets:

- `SUPABASE_URL`
- `SUPABASE_PUBLISHABLE_KEY`
- `LICENSE_SIGNING_PRIVATE_KEY` (PKCS#8 Ed25519 PEM; server-only)
- `PORTAL_ALLOWED_ORIGIN` (exact HTTPS origin; no wildcard)

## Backend rollout

1. Review and apply `supabase/migrations/202610210001_phase_9a2_website_portal.sql` to a non-production environment.
2. Run `supabase/tests/phase9a2_security.sql` in a rollback-only session.
3. Set Edge Function secrets without printing them and deploy `license-state`.
4. Configure Supabase Auth Site URL and redirect allow-list for `/auth/callback` and `/account/reset-password`.
5. Run synthetic registration, email verification, trial, device, revocation, and signing smoke tests.
6. Deploy the static site only after the published installer hash matches `public/releases/index.json`.

## Manual activation administration

The browser only creates an activation request. An authorized operator uses the modern server secret from process memory:

```text
node scripts/admin/portal-license-admin.mjs --action=list
node scripts/admin/portal-license-admin.mjs --action=activate --account=<uuid> --plan=SINGLE_PHARMACY --devices=1 --note=<auditable-reason>
```

Set `M2_SUPABASE_URL` and `M2_SUPABASE_SECRET_KEY` in the operator process. The tool never writes or prints the secret. The next authenticated device validation receives a newly signed state from the Edge Function.

## Payment boundary

No provider, checkout, invoice, webhook, or recurring billing implementation exists. The database keeps a stable plan and entitlement-source boundary so a future verified webhook handler can call a server-only entitlement operation without rewriting trial or device ownership logic.
