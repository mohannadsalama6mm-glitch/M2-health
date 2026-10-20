# Phase 9A.3 — Early Access Deployment and Launch Readiness

Date: 2026-10-21
Status: **NOT READY — deployment authorization/connection prerequisites remain**

## Scope and preservation

This pass inspected the completed Phase 9A.2 website and its launch prerequisites without redoing Phase 8C, Phase 9A.1, or Phase 9A.2. No payment, subscription billing, Phase 10, installer updater, or cloud synchronization rollout work was started.

The real development SQLite database was not opened, migrated, copied, seeded, or reset. Cloud synchronization and transport remain disabled in `config/cloud.example.json` (`transportEnabled: false`).

## Hosting target

Preferred target: Vercel's existing free/low-cost static hosting path.

The repository's `website/vercel.json` is suitable for the Vite SPA when the Vercel project root is configured as `website/`. It preserves `/releases`, `/images`, and built `/assets` paths while rewriting application routes to `index.html`.

No Vercel project is linked locally in either `.vercel/project.json` or `website/.vercel/project.json`. The Vercel plugin was declined, so the supported CLI deployment path was selected instead. The CLI can deploy the current `website/` directory without committing or pushing the wider dirty tree. No deployment URL was invented or claimed.

The current `main` checkout matches `origin/main` at `e307ee2`, but the completed website and the wider Phase 8C/9A work are uncommitted. The CLI path avoids requiring a commit/push for this launch pass. Existing unrelated and historical working-tree changes were preserved.

During Vercel CLI authentication, a diagnostic-mode retry unexpectedly emitted short-lived access, refresh, and identity credentials into the local tool transcript. The session was treated as compromised and `vercel logout` completed successfully; inspection of the installed CLI confirms that this logout path sends a server-side revocation request before deleting the local credential store. The exposed session will not be reused. A fresh non-debug device flow was started but still requires the owner to approve it in Vercel.

## Production environment configuration

Required browser-safe variables are already documented and separated:

- `VITE_SUPABASE_URL`;
- `VITE_SUPABASE_PUBLISHABLE_KEY`;
- `VITE_SITE_URL`;
- `VITE_SUPPORT_EMAIL`.

Only the Supabase URL and modern publishable key may enter the frontend build. Service-role credentials, modern secret keys, the Ed25519 private key, admin tokens, and database passwords must remain outside Vercel's browser-exposed variables.

The existing Phase 9A.2 credential-shaped bundle scan was clean. A new final hosted-bundle scan remains pending until Vercel produces the actual deployment artifact.

## Supabase Auth redirects

Pending the final hosted URL. The production Site URL and allowed redirects must be configured to the exact Vercel/custom origin for:

- `/auth/callback`;
- `/account/reset-password`.

Registration, confirmation-link return, password recovery, login, session recovery, logout, and useful localhost development redirects must be verified after the hosted URL exists. No placeholder hostname was written into the live Auth configuration.

## License Edge Function

The linked development Supabase project currently reports:

- deployed Edge Functions: none;
- user-configured Edge Function secrets: none.

Accordingly, `license-state` has not been deployed and no signing secret has been uploaded. This is a safe closed state, not a launch-ready state.

The existing ignored local signing private key is present, remains Git-ignored, and derives the same Ed25519 public key embedded by the desktop build. The Phase 9A.2 function contract remains the single compatible signing system; no second key pair or licensing format was created.

After the hosted origin is known, the authorized deployment must set the private key only in Supabase's server-side secret store, configure an exact `PORTAL_ALLOWED_ORIGIN`, configure the server-side modern publishable key, and deploy the existing `supabase/functions/license-state` implementation without logging any value.

## Release download baseline

The immutable release waiting for hosted verification is:

- version: `0.4.0-beta.1`;
- build ID: `20261020.1`;
- file: `M2-Health_0.4.0-beta.1_x64-setup.exe`;
- size: 6,386,168 bytes;
- SHA-256: `0C9F6A7F0BA431EFEBECF358A3DB85CD92CCD3E88AE87BA4BE8377DAA5BF4F4A`;
- Authenticode: unsigned Early Access build.

The local website metadata, versioned installer, release notes, and unsigned warning were already verified in Phase 9A.2. The public hosted URL and byte-for-byte hosted checksum cannot be recorded until deployment succeeds.

## Desktop interoperability evidence

- The ignored private signing key and desktop public verification key match.
- The existing signed-envelope contract uses the desktop's version 1, device-bound Ed25519 format.
- The backend maps active trial entitlement to a time-limited signed `ACTIVE` envelope, which the desktop accepts and expires according to the signed timestamp.
- Manual active, suspended, and revoked states use the desktop-supported values.
- The desktop preserves read-only behavior for expired, suspended, and revoked states and its existing three-day offline grace logic.

A focused Rust licensing test retry initially failed because the Windows paging file was too small while loading Rust metadata. A single-job isolated retry advanced through dependency compilation without reproducing a source assertion failure, but did not reach test execution before the stalled build was terminated. This is recorded as an environment-limited check; the previously completed Phase 9A.1 focused licensing tests and Phase 9A.2 JavaScript contract tests remain the current passing evidence. Hosted issuance cannot be proven until the Edge Function is deployed.

## Hosted flow and security smoke

Not run because no hosted URL or deployed signing function exists. No synthetic user or device record was created during this blocked launch pass, so no cleanup was required.

The required post-deployment smoke remains:

1. public homepage, features, pricing, download, FAQ, and support routes;
2. registration, confirmation redirect, login, session recovery, logout, and password-reset redirect;
3. portal overview, one-time trial, device list, downloads, subscription/request, and account routes;
4. signed trial issuance, desktop signature verification, manual activation, suspension/revocation, and future validation;
5. cross-account device/portal denial and unauthorized admin denial;
6. hosted installer size/hash and release-notes verification;
7. synthetic account, license, device, request, audit, and Auth cleanup.

## Manual sales and support readiness

The portal already supports a free trial and auditable activation/upgrade request. The secure server-admin CLI already supports request listing and manual plan/status/device-limit assignment without printing its secret. Device revocation is customer-owned; suspension and expiry are server-admin operations.

The deployment still needs a real owner-approved support email. The placeholder `support@example.invalid` must not be used for a public launch.

## Remaining launch blockers and minimum next action

1. Complete a fresh non-debug Vercel CLI device authorization; do not reuse the revoked diagnostic session.
2. Provide the real Early Access support email to publish.
3. After those inputs, create/link the Vercel project from `website/`, deploy it, configure Auth redirects and server-only Edge secrets, deploy `license-state`, and run the complete hosted synthetic smoke with cleanup.

## Result

Early Access Launch is not ready because the hosted application and license function do not yet exist. The diagnostic Vercel session that appeared in tool output was revoked immediately and will not be reused. No unsafe partial rollout was enabled.
