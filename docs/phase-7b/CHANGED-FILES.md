# Phase 7B Changed Files

This list describes Phase 7B changes in the accumulated working tree. Earlier uncommitted phase files remain present and were not discarded.

## Backend authorization

- `src-tauri/src/authorization.rs` — complete command registry, role grants, session/permission/branch/record/actor checks, guarded dispatch, and synchronous authorization mutex.
- `src-tauri/src/lib.rs` — wraps the sole 162-command Tauri handler registration in the authorization gate and manages authorization state.
- `src-tauri/src/error.rs` — adds stable `unauthorized` and `forbidden` application errors.
- `src-tauri/src/auth/mod.rs` — exposes current principal roles and branch memberships to backend authorization.
- `src-tauri/src/auth/commands.rs` — revokes target sessions after role or branch changes.
- `src-tauri/src/commands/branches.rs` — requires a session and filters the branch directory to memberships for non-admin users.
- `src-tauri/src/sales/models.rs`, `src-tauri/src/sales/commands.rs`, `src-tauri/src/sales/mod.rs` — makes customer sale history use one branch field for both authorization and the SQL filter.

## Frontend integration

- `src/lib/tauri/client.ts` — in-memory session context, authenticated typed IPC, actor normalization, and unauthorized-session invalidation.
- `src/lib/permissions.ts` — role-aware frontend capability map.
- `src/app/AuthContext.tsx` — connects Rust sessions to the frontend and handles expiry/revocation.
- `src/app/BranchContext.tsx` — loads only authorized branches and clears stale selection.
- `src/app/navigation.ts`, `src/app/AppShell.tsx`, `src/app/AppRoutes.tsx` — permission-aware navigation and direct-route denial states.
- `src/features/management/CanonicalPeople.tsx` — describes the active authorization boundary.
- `src/features/relationships/CanonicalPartners.tsx`, `src/lib/tauri/canonical-sales.ts` — sends the selected authorized branch for customer sale history.

## Tests and documentation

- `src-tauri/tests/authorization.rs` — 162-command coverage, public allowlist, fail-closed behavior, sessions, roles, branches, actor forgery, persisted record ownership, all-branch bypasses, exports, zero-mutation denial, and concurrency serialization.
- `src-tauri/tests/canonical_sales.rs`, `src-tauri/tests/phase5_integration.rs` — adapt customer-history fixtures to the explicit branch query.
- `tests/authorization-double.ts`, `tests/authorization.spec.ts` — native authorization double and browser coverage for navigation, direct routes, branch context, invalidation, and export denial.
- `tests/auth.spec.ts` — verifies the completed Phase 7B role-boundary wording.
- `docs/phase-7b/PERMISSION-MATRIX.md` — all 162 command policies and role grants.
- `docs/phase-7b/SECURITY-BOUNDARIES.md` — trust, branch, actor, synchronization, async, and residual-risk contract.
- `docs/phase-7b/FINAL-REPORT.md` — completion evidence and handoff.

No migration file was changed during Phase 7B.
