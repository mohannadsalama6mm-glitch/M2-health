# Phase 8A Final Report

## Result

Phase 8A implements a dormant, local-only cloud foundation without enabling synchronization or assigning existing data to a tenant. SQLite v018, a separate PostgreSQL control-plane schema, default-deny RLS policies, stable local device identity, enrollment boundaries, protocol persistence, protected status IPC, truthful native UI, focused security tests, and the required architecture documents are present.

## Security and consistency

Tenant and branch relationships use composite foreign keys. Outbox events require an active organization member, branch access where applicable, and an enrolled device; revocation blocks later enqueue attempts. Envelopes are immutable while retry metadata remains durable. Inbox identity is tenant-scoped and idempotent. Existing Inventory and Finance sources of truth are unchanged, and no last-write-wins policy was introduced.

The cloud PostgreSQL role has read-only, RLS-filtered access. Direct membership escalation, role assignment, enrollment, and event writes have no authenticated policy or grant. Trusted server endpoints remain future Phase 8B work. PostgreSQL runtime RLS verification is pending because no isolated runtime is available; static policy tests pass and completion does not claim runtime execution.

## Verification

- `cargo fmt --check`: passed.
- `cargo check`: passed.
- `cargo test`: 490 passed, 0 failed, 0 ignored across executable test targets; main and documentation targets contain no tests.
- `npm run lint`: passed with zero errors and zero warnings.
- `npm run build`: passed; the existing advisory bundle-size warning remains.
- `npm test`: 295 passed, 0 failed in 7.5 minutes, including 1366x768, 1440x900, and 1920x1080 coverage.
- `git diff --check`: passed; Git emitted only line-ending conversion notices.
- Focused migration/authorization run: 39 passed, 0 failed.
- Focused native Sync/browser run: 15 passed, 0 failed.

The complete standard suites contain 785 passing tests and zero failures. No native application, installer, real SQLite database, production database, or network synchronization was used.

SQLite migrations 001–017 match every recorded baseline SHA-256. Migration 018 is `ceb05b972182d4747b5977d9b25a4db97f586c50402b1d27744b0ace14d09df8`. The active Tauri handler has 163 unique commands and four public commands; cloud status is protected by `session.self` and unknown commands remain fail-closed.

## Defects fixed

The pre-8A model had no explicit tenant ownership, device enrollment, idempotent protocol persistence, cloud schema, or RLS boundary. The native Sync route also had no persisted implementation. These gaps are addressed by the dormant foundation. During implementation, branch reads and organization-wide event reads were tightened so ordinary members see only assigned branches and only organization administrators can read organization-wide events. Local adoption, branch grants, identity linking, and device enrollment now require an active organization owner or administrator at the database boundary.

## Runtime limitation

PostgreSQL, Docker, and Podman are not installed in the execution environment. PostgreSQL migration execution and negative RLS tests are therefore pending in a future isolated Supabase/PostgreSQL test project. This is an external readiness gate before Phase 8B deployment and no runtime-RLS pass is claimed.

## Remaining Phase 8B prerequisites

Phase 8B must deploy and runtime-test RLS in an isolated cloud project, implement trusted enrollment and identity linking, finalize versioned domain event contracts, add same-transaction outbox hooks, implement signed/authenticated transport, server idempotency and arbitration, ordered inbox application, backoff and observability, and exhaustive disconnect/retry/conflict tests. Live sync must stay disabled until Phase 8C integration authorization and verification.

**Phase 8A: COMPLETE**
