# Phase 9A.3 — Early Access Launch Report

Date: 2026-10-21
Status: **READY**

## Production deployment

- Canonical hosted URL: `https://m2-health.vercel.app`
- Vercel project: `m2-health`
- Production deployment: `READY`
- Production deployment inspected: `m2-health-19f4qoyoh-m2-f89c.vercel.app`
- Website source commit: `9940402` (`Launch Early Access website and trial portal`)
- Supabase Edge Function: `license-state`, version 1, status `ACTIVE`, JWT verification enabled
- Visible Early Access support email: `viralnest902@gmail.com`

The hosted website and signing function were already deployed before this compatibility repair. This pass did not redeploy either artifact and did not start payment, subscription, licensing-commerce, updater, or Phase 10 work.

## Modern secret-key compatibility repair

The live launch smoke exposed a real incompatibility in the two server-admin RPCs. They granted execution only to the database `service_role`, but also required `auth.role() = 'service_role'` inside the function body. That inner check assumed the legacy API key itself was a JWT.

Forward migration `202610210003_phase_9a3_modern_server_admin.sql` replaces both functions without changing either public signature. Authorization now relies on PostgreSQL EXECUTE privileges at the RPC boundary:

- `service_role`: EXECUTE granted;
- `authenticated`: explicitly revoked;
- `anon`: explicitly revoked;
- `public`: explicitly revoked.

Supabase's hosted API gateway maps an approved modern `sb_secret_` API key to the built-in `service_role` database role. The opaque key is sent only in the `apikey` header; it is no longer sent as a bearer JWT. The RPCs remain `SECURITY DEFINER` with fixed search paths, and the state-changing RPC continues to write the existing immutable administrative audit record.

No secret or admin capability was added to the browser application. The modern secret remains limited to the controlled server/admin tool path. The customer-facing Edge Function continues to use an authenticated user access token plus the client-safe publishable key and does not invoke these admin RPCs.

## Migration and focused security evidence

Applied to the linked development Supabase project:

- `npx --yes supabase@latest db push --linked --dry-run` identified only `202610210003_phase_9a3_modern_server_admin.sql`;
- `npx --yes supabase@latest db push --linked` applied that migration successfully;
- final `npx --yes supabase@latest db push --linked --dry-run` returned `upToDate: true` with no pending migrations, seeds, or role changes.

Focused rollback-only SQL test: `supabase/tests/phase9a3_modern_admin.sql`.

It passed against the linked development project and proves:

- both RPCs are executable by `service_role` without a legacy JWT role claim, modelling the gateway role produced by an approved modern secret key;
- both RPCs reject `authenticated`;
- both RPCs reject `anon`;
- function privilege metadata grants neither RPC to `public`, `anon`, or `authenticated`;
- cross-account device claim and revoke operations remain denied;
- all SQL fixtures roll back.

## Focused live smoke

Only the Phase 9A.2/9A.3 lifecycle and security path was rerun. The successful live result contained 14 assertions:

- two synthetic users authenticated with the modern publishable key;
- account isolation held;
- first trial start succeeded and a second start was denied;
- device registration succeeded;
- cross-account registration and revoke were denied;
- device revocation propagated;
- the modern secret-key server path listed activation requests;
- authenticated admin mutation was denied;
- anonymous admin listing was denied;
- manual activation propagated;
- suspension propagated;
- expiry propagated.

The first live attempt received a transient HTTP 520 before reaching an assertion. A status-only retry confirmed the Auth admin endpoint accepted the modern secret in the `apikey` header, and the complete focused smoke then passed.

## Synthetic cleanup

The final audit found six historical `phase9a2-…@example.invalid` synthetic Auth fixtures, including two with portal records. Only that exact synthetic namespace was deleted in dependency order. The post-cleanup audit returned:

- synthetic Auth users: `0`;
- synthetic customer profiles: `0`;
- synthetic license accounts: `0`.

No customer namespace or non-synthetic record was targeted.

## Safety and rollout status

- No credential value was printed, committed, written to the report, or placed in browser code.
- No legacy `service_role` key was re-enabled or used.
- `config/cloud.example.json` still has `transportEnabled: false`.
- The real development SQLite database was not opened or modified. Its metadata remains 67,641,344 bytes with last-write time `2026-10-09T13:15:34.0714897Z`.
- Full Rust and Playwright suites were not rerun because this repair changes only the hosted SQL authorization boundary and server-side admin request headers.

## Result

No material Phase 9A.3 blocker remains.

Early Access Launch: READY
