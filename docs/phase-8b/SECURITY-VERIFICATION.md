# Phase 8B Security Verification

## PostgreSQL boundary

The Phase 8A and Phase 8B PostgreSQL migrations are deployed to the isolated development Supabase project identified by the corrected project reference. Migration history previously confirmed both local migrations match the remote migration records. This continuation did not redeploy or mutate remote state.

`supabase db lint --linked --level warning` passed with no schema findings. The transactional runtime script in `supabase/tests/phase8b_security.sql` passed and rolled back all fixtures. Its assertions cover tenant-filtered organization reads, branch-filtered reads, denied direct table mutation, authenticated device enrollment, first push, idempotent replay, server-derived actor identity, ordered pull, durable acknowledgment, changed-payload rejection, cross-branch rejection, and revoked-membership denial. A follow-up query confirmed that no fixture organizations or events remained.

The migration revokes RPC execution from `public` and `anon`, grants it to `authenticated`, derives the subject with `auth.uid()`, and performs tenant, branch, membership, and device checks inside the security-definer functions. Base tables retain their Phase 8A default-deny and RLS boundaries.

## Desktop boundary

The active Tauri handler registers 167 commands. Automated authorization coverage verifies every registered command has one explicit policy, the public allowlist remains exactly four bootstrap/session-recovery commands, and unknown commands fail closed. Cloud status requires `session.self`; cloud session mutation and sync require `cloud.manage`.

The publishable key and endpoint are loaded from environment variables or the ignored `config/cloud.local.json`. The committed `config/cloud.example.json` contains placeholders and disables transport. The actual project reference and publishable-key pattern do not occur in tracked files. `config/cloud.local.json` and `supabase/.temp/` are ignored and untracked. No service-role or secret key is used or requested.

React receives only non-secret session status. Access and refresh tokens remain inside Rust process memory and are bound to the authenticated local user. The frontend contains no localStorage, sessionStorage, or IndexedDB persistence path for cloud credentials. Sync attempt records contain safe fixed messages rather than secrets or response bodies.

## Runtime gates

Synchronization requires all of the following:

1. a valid local authenticated session with `cloud.manage`;
2. a configured HTTPS Supabase endpoint and publishable key;
3. configuration `transportEnabled=true`;
4. an authenticated in-memory cloud session owned by the local user;
5. an active cloud identity link and active organization membership;
6. an enrolled local device and authorized branch scope;
7. `sync_runtime_gate.phase8c_verified=1`.

The committed and local configuration both currently keep transport disabled, and migration 019 initializes the database gate to zero. A defect found during final verification was fixed so UI status now includes every gate and cannot claim that synchronization is available while execution would be denied.

## Verification limits

No real cloud Auth account and enrollment were provisioned for this phase, so an end-to-end HTTP sign-in/push/pull cycle was not claimed. PostgreSQL runtime authorization was verified with transactional JWT role simulation, while the Rust transport and engine were verified with isolated deterministic transports and databases. Live authenticated integration, network interruption, token refresh against Supabase Auth, multiple enrolled devices, and rollout enablement remain Phase 8C gates.

The native application was not launched, no installer was built, and the real development SQLite database was not opened or migrated.
