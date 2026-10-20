# Phase 9A.2 — Website MVP and Trial Portal

Date: 2026-10-21
Status: implementation and non-production verification complete; public deployment remains a separately authorized launch action.

## Scope and preservation

Phase 9A.2 adds a separate deployable website package at `website/`. It does not replace the Tauri desktop frontend and does not alter the completed Phase 8C or Phase 9A.1 behavior. No payment gateway, recurring billing, installer rollout, updater deployment, code-signing purchase, or Phase 10 work was started.

The real development SQLite database was not opened, migrated, copied, seeded, or reset. Its final metadata matches the recorded baseline:

- path: `C:\Users\10\AppData\Local\com.m2health.pharmacy\m2-health.db`;
- size: 67,641,344 bytes;
- last write (UTC): 2026-10-09 13:15:34.

## Website architecture and pages

The website is a React 19, TypeScript, Vite single-page application with its own dependency lockfile and deployment configuration. It uses Supabase Auth only when public configuration is present. Without configuration, marketing and release downloads remain available while account actions fail closed with a visible configuration message; there is no mock account or browser-persistence simulation.

Public routes:

- `/` — product value proposition, real desktop screenshots, trial and download actions;
- `/features` — desktop capabilities and honest sync-rollout status;
- `/pricing` — Early Access, single-pharmacy, and multi-branch positioning with no fake checkout;
- `/download` — immutable versioned Windows release, checksum, requirements, and unsigned-build warning;
- `/faq` and `/support` — trial, local data, recovery, support, and manual activation guidance;
- `/login`, `/register`, `/forgot-password`, `/account/reset-password`, `/auth/callback` — Supabase Auth lifecycle.

Authenticated portal routes:

- `/portal` — entitlement, trial dates, days remaining, device count, and current release;
- `/portal/devices` — device-bound activation issuance, device list, and confirmed revocation;
- `/portal/downloads` — installer, checksum, release notes, and install guidance;
- `/portal/subscription` — current plan/state and auditable manual activation requests;
- `/portal/account` — profile, password, account identifier, and logout.

Desktop and mobile layouts were inspected at 1440 × 900 and 390 × 844. The navigation, hero, screenshots, calls to action, login failure state, and mobile menu rendered correctly; no horizontal overflow was observed.

## Authentication, trial, entitlement, and device model

Supabase migrations add forced-RLS account, entitlement, device, activation-request, and admin-audit tables. User account bootstrap is tied to `auth.users`. User-facing operations are exposed through authenticated security-definer RPCs with ownership checks; server administration is restricted to the service role.

The trial is an explicit one-time transition from `UNACTIVATED` to `TRIAL`. Account creation alone does not consume it, and a database trigger prevents resetting or moving the recorded trial start. Portal state derives trial, active, suspended, expired, and revoked behavior on the server.

Device activation is account-owned and plan-limit-aware. The Edge Function validates the bearer user with the modern publishable key, calls the authenticated claims RPC, and signs a desktop-compatible Ed25519 envelope with the server-only private key. The browser receives only the signed activation envelope. Revocation updates portal state and causes later validation to return the revoked state without deleting local pharmacy data.

## Supabase integration and security evidence

Forward migrations applied to the linked development project:

- `202610210001_phase_9a2_website_portal.sql`;
- `202610210002_phase_9a2_portal_snapshot_fix.sql`.

`npx --yes supabase@latest db push --linked --dry-run` returned `upToDate: true` with no pending migration, seed, or role changes.

The isolated authenticated live harness used two synthetic accounts and automatic cleanup. It passed all reported assertions:

- both users authenticated;
- account/tenant isolation enforced;
- first trial start succeeded and second start was denied;
- owned device registration succeeded;
- cross-account registration and revocation were denied;
- device revocation propagated;
- activation request was visible to the authorized admin path;
- non-admin access to the admin operation was denied;
- manual activation, suspension, and expiry propagated to the customer state.

No credential value was printed, written into tracked source, or placed in a browser environment variable. Client configuration accepts only the project URL and modern publishable key. The admin CLI accepts only a modern `sb_secret_...` key from process environment. The Ed25519 private key remains server-only and local/secret-managed; the public desktop verification key remains the only embedded key material.

The rollback-only SQL test file is present at `supabase/tests/phase9a2_security.sql`. `supabase test db --linked` could not execute because this Supabase CLI command still requires a local Docker-compatible runtime, which is not installed. This was an environment limitation rather than a SQL assertion failure; the corresponding authenticated ownership, trial, device, revocation, state, and admin-denial paths were exercised against the linked development project by the live harness.

## Manual activation workflow

1. A signed-in customer submits an activation request from the portal.
2. An authorized operator lists pending requests with `scripts/admin/portal-license-admin.mjs` using the modern server secret from process memory.
3. The operator records the account state, plan, dates, device limit, and mandatory audit reason through the server-only RPC.
4. The customer registers the desktop device UUID in the portal.
5. The Edge Function signs account and device claims with the server-only Ed25519 private key.
6. The customer pastes the signed activation code into the desktop app.

No card details are collected and no UI action claims that payment occurred. The schema retains a stable plan and entitlement-source boundary for a future separately approved payment integration.

## Release delivery

- installer: `website/public/releases/0.4.0-beta.1/M2-Health_0.4.0-beta.1_x64-setup.exe`;
- version: `0.4.0-beta.1`;
- build ID: `20261020.1`;
- platform: Windows x64;
- size: 6,386,168 bytes;
- SHA-256: `0C9F6A7F0BA431EFEBECF358A3DB85CD92CCD3E88AE87BA4BE8377DAA5BF4F4A`;
- Authenticode: unsigned Early Access build.

The copied website artifact matches the Phase 9A.1 installer hash. A local HTTP HEAD request to the versioned download URL returned HTTP 200, `application/octet-stream`, and the expected content length. Release metadata and notes are versioned alongside the installer.

## Verification

Passed on 2026-10-21:

- `npm run lint` in `website/`;
- `npm run build` in `website/` — TypeScript check and production Vite build, 1,631 modules transformed;
- `npm test` in `website/` — 6 files and 13 tests passed;
- `scripts/admin/phase9a2-live-integration.ps1` — all 13 live lifecycle/security assertions passed;
- `npx --yes supabase@latest db push --linked --dry-run` — linked database up to date;
- installer hash, size, and local HTTP delivery checks;
- repository `npm run lint` after correcting the new admin CLI's Node lint compatibility;
- `git diff --check`.

The focused frontend tests cover public positioning, the no-payment boundary, registration/login calls, protected routes and logout, one-time trial RPC use, device revocation, Edge Function invocation, immutable release metadata, and the desktop-compatible Ed25519 envelope contract.

The full Rust and Playwright suites were not rerun because Phase 9A.2 did not materially change desktop application code. Their completed Phase 9A.1 evidence remains unchanged.

## Rollout gates and launch handoff

Cloud synchronization and transport remain disabled (`config/cloud.example.json` has `transportEnabled: false`). No installer or updater rollout gate was enabled.

Code is deployment-ready, but these launch actions remain intentionally unapplied and require the owner/deployment operator:

1. select the production hostname and static hosting target;
2. configure the public site URL, modern publishable key, support address, and Supabase Auth redirect allow-list;
3. add the existing Ed25519 private signing key to the approved Supabase secret store without exposing it;
4. set an exact HTTPS `PORTAL_ALLOWED_ORIGIN` and deploy the `license-state` Edge Function;
5. deploy `website/dist`, confirm TLS and SPA rewrites, then verify the hosted installer hash byte-for-byte;
6. run one production-host synthetic registration/email/reset/trial/device/revocation smoke and remove the synthetic records;
7. provide the real support/activation contact and Early Access operating owner.

These are deployment and operational prerequisites, not missing application implementation. Payments, subscriptions, commercial billing, production code-signing purchase, updater rollout, multi-device sync rollout, and Phase 10 remain deferred.

## Result

Phase 9A.2 implementation and verification are complete. The website is ready for the separately authorized deployment handoff; it is not represented as publicly launched until the actions above are completed.
