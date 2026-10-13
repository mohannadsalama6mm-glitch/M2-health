# Phase 8C Final Report

## Status

**Phase 8C: NOT COMPLETE**

The cloud protocol, isolated authenticated integration, regression suites, UI, and fail-closed rollout controls pass. Completion is blocked because a legacy Supabase service-role JWT was exposed by CLI command output during the audit and has not been disabled or rotated. The current replication contract also does not satisfy the requested concurrent Catalog and complete business-document multi-device coverage.

## A. Implemented features

- Forward-only SQLite migration 020 makes the existing transport metadata gate explicitly activatable while preserving every installation at disabled.
- PostgreSQL Phase 8C migration adds canonical payload hashing, server-side hash verification, authenticated organization bootstrap, owner-controlled membership, device revocation, and member revocation.
- An isolated administrative integration harness provisions synthetic Auth users, runs authenticated tests, and cleans up all remote state.
- Cloud status now includes the authenticated subject, last attempt and success, rejected inbox count, conflicts, retry count, and next retry.
- The Sync UI provides real sign-in/sign-out, enrollment and gate state, retry/conflict visibility, actionable errors, and a disabled sync action until every backend gate passes.

## B. Authenticated cloud test results

The isolated live run passed with 3 authenticated users, 2 organizations, 2 branches, and 3 devices. One event was pushed, replayed idempotently, pulled by a second device, and acknowledged. Actor derivation, refresh-token exchange, cross-tenant denial, malformed-hash denial, device revocation, and member revocation passed. Cleanup left 0 Phase 8C organizations, 0 events, and 0 Auth users.

## C. Multi-device test results

Device-to-device event delivery and tenant isolation passed. Inventory and Finance application are idempotent and constraint protected. Complete Catalog mutation and sales/return/receipt/count document graphs are not replication entities, so the requested concurrent coverage for those document types is not complete.

## D. Offline and reconnect tests

Outbox atomicity, failed-push retry scheduling, exact receipt matching, delayed dependency retry, duplicate inbox delivery, checkpoint durability, invalid-envelope rollback, and echo suppression pass. The suite does not fabricate a successful network state.

## E. Security and RLS results

Forced RLS, narrow authenticated RPC grants, JWT actor derivation, tenant/branch/device validation, membership and device revocation, token refresh, payload-integrity rejection, and tracked-file secret scans pass. Linked database lint reports no issues. The rollback-only PostgreSQL security script passes.

A legacy service-role credential was exposed in CLI output during this audit. It is absent from tracked files and application storage, but it must be disabled or rotated before rollout.

## F. Inventory and Finance verification

Remote Inventory applies only through canonical append-only movements and no-negative-stock checks. Violations roll back and quarantine. Finance journals require balanced immutable lines and apply exactly once. Disconnected devices can still oversell stale availability; convergence quarantines the violating event rather than silently overwriting stock.

## G. Branch transfer status

Cloud branch transfers remain unsupported. An individual transfer leg is quarantined as a missing dependency. No path silently applies one side.

## H. Sync UI status

The UI reflects backend mode, endpoint configuration, subject identity, enrollment, transport gate, last synchronization, pending events, retries, and conflicts. Passwords clear after sign-in and are absent from browser persistence and rendered HTML. Automated viewport checks pass at 1366×768, 1440×900, and 1920×1080.

## I. Migration details

- SQLite 001–017: no working-tree diff.
- SQLite 018 SHA-256: `ceb05b972182d4747b5977d9b25a4db97f586c50402b1d27744b0ace14d09df8`.
- SQLite 019 SHA-256: `4fd65d44e289d71186fb676e835b6200665bcf14e24e2eb413f9764a1da9eb7c`.
- SQLite 020 `sync_rollout_controls` SHA-256: `6f5d1105e3f53c67c6d4f6f80b9e1b0c43c4d0496d1b5429cde1c1f296e7bfbb`.
- PostgreSQL Phase 8C SHA-256: `4cd29e6ea977dd3dc54b9808776bc3e839b90f3404d6603a03c2c30659f1d594`.
- Linked Supabase ledger contains matching local/remote versions `202610090001`, `202610100001`, and `202610100002`.

All SQLite migration tests use isolated databases. The real development SQLite database was not opened.

## J. Full regression results

- `cargo fmt --all -- --check`: passed.
- `cargo check`: passed.
- `cargo test`: 511 passed, 0 failed, 0 ignored; zero-test binary and documentation targets are excluded from the passing count.
- `npm run lint`: passed with 0 errors.
- `npm run build`: passed; Vite emitted only its existing chunk-size advisory.
- `npm test`: 300 passed, 0 failed, 0 skipped in 11.2 minutes.
- `git diff --check`: passed; line-ending notices only.
- Executable Rust plus Playwright total: 811 passed, 0 failed, 0 skipped.
- Focused Phase 8C browser/native-boundary run: 10 passed.
- Focused cloud-foundation Rust run: 9 passed.
- Linked PostgreSQL lint: passed with no issues.
- Rollback-only PostgreSQL security script: passed.

Native launch, installer build, manual testing, real development SQLite access, and rollout activation were skipped because the task prohibits them.

## K. Outstanding limitations

1. Disable or rotate the exposed legacy service-role JWT and repeat the isolated integration/security checks.
2. Define and verify replication for Catalog mutations and complete sales, returns, receipts, and stock-count documents if those must converge across devices.
3. Decide whether quarantining stale-stock oversells is sufficient or add an online reservation/serialization design.
4. Design a paired atomic transfer contract before enabling cloud transfers.

## L. Rollout readiness

Not ready. Local configuration and both SQLite rollout gates remain disabled. Activation requires a separate approved task following `ROLLOUT-ACTIVATION.md`, and must never target the existing real development database.

## M. Changed files

The preserved Phase 8A–8C working tree contains 25 modified tracked files and 40 untracked files after this report and changed-file inventory are added. See `CHANGED-FILES.md` for the path-level breakdown. No credential-bearing local configuration is tracked.

## N. Recommended Phase 9 scope

Do not start Phase 9 yet. First close the Phase 8C credential and replication blockers and rerun the gates. After Phase 8C can be declared complete, scope Phase 9 around deployment operations, encrypted backup/recovery, observability, controlled update delivery, and an isolated canary rollout with explicit rollback criteria.
