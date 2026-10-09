# Phase 7B Final Report

## A. Scope

Phase 7B adds backend-owned operational authorization to the recovered M² Health desktop codebase. Work was limited to code, automated tests, and documentation. Phase 7C was not started.

## B. Completion result

Every protected registered IPC command is covered by the centralized Rust gate, branch and permission denials are verified, and the complete Rust and frontend regression suites pass.

**Phase 7B: COMPLETE**

## C. Command inventory

The sole production `tauri::generate_handler!` registration contains exactly **162 unique commands**. A source-driven Rust test compares each name with the executable policy registry. Missing policies and unknown commands fail closed.

## D. Public command surface

Only `get_auth_status`, `bootstrap_admin`, `login`, and compatibility-only `recovery_status` are public. Bootstrap retains its one-time transactional constraints. Every other command requires a current Rust session.

## E. Permission model

The stable permission vocabulary and grants for `ADMIN`, `PHARMACIST`, `CASHIER`, `INVENTORY_MANAGER`, and `VIEWER` are recorded in `PERMISSION-MATRIX.md`. Admin receives all registered permissions; other roles receive explicit allowlists. Finance and financial trace access remain admin-only.

## F. Session enforcement

The gate rejects missing, invalid, expired, revoked, and inactive-user sessions. Rust rechecks user activity, credential revision, roles, and branch memberships for every call. Logout, password reset, deactivation, role change, and membership change revoke affected sessions where applicable.

## G. Branch authorization

Selected-branch commands accept only their documented branch path. Non-admin all-branch requests are rejected. Transfers require both memberships. Branch listings are filtered in Rust. Irrelevant or duplicated branch fields cannot be used as authorization proof.

## H. Organization-wide entities

Catalog identities, suppliers, customers, purchase-order headers, and finance account definitions remain organization-wide according to their schemas. Purchase receipts, returns, stock, sales, expenses, inventory workflows, counts, alerts, and operational aggregates remain branch-owned.

## I. Canonical record ownership

Record commands resolve branches from persisted sale, sale-return, receipt, purchase-return, expense, journal, alert, count-session, or inventory-document rows. Cross-branch sale-ID substitution is tested. The customer-history defect found during final audit was fixed so the gate and SQL query use the same `query.branchId` rather than two caller-controlled values.

## J. Actor provenance

Frontend actor fields are normalized to the authenticated user ID for compatibility, while Rust rejects forged recognized actor fields. No supplied role, actor, selected branch, or `authorizationBranchId` is trusted as proof of authorization. Legacy nullable actor columns remain unchanged.

## K. Dashboard and reporting isolation

Dashboard and operational report queries require an allowed branch for non-admin users. Null or omitted branch filters are rejected rather than treated as unrestricted scope. Existing database-level report tests continue to verify branch filters, totals, pagination, and read-only behavior.

## L. Finance and source-detail isolation

Unauthorized roles cannot invoke Finance, journal, trial-balance, expense, source-trace, or profitability commands. Canonical finance IDs are resolved before handler entry. Admin has organization-wide Finance visibility by policy; there is currently no non-admin finance-reader role.

## M. CSV isolation

CSV exports require `reports.export`, which is admin-only in the current grants. The gate selects the branch path according to export kind and rejects unsupported or unscoped non-admin attempts. Existing CSV tests still verify exact filtering, UTF-8 behavior, formula blocking, and no writes.

## N. Identity administration

Identity commands are gated by `users.*` or `roles.*` permissions and also retain their internal Phase 7A admin-session checks. Last-admin protection, password reset, role assignment, branch assignment, and session revocation remain intact.

## O. Invocation boundary

There is one production `invoke_handler`, and it wraps the entire handler list with `authorization::guard`. The frontend also has one typed `nativeRequest` boundary, but frontend code is not trusted. Direct or modified frontend invocation still encounters Rust authorization.

## P. Asynchronous commands

The three async Catalog import inspection commands are authenticated and authorized before their futures are scheduled. Their future lifetime is not covered by the global mutex. They perform source profiling, dry-run planning, and report reads. The organization-wide apply command is authorized synchronously before its existing background job is accepted. Revoking permission after accepted dispatch does not cancel accepted background work.

## Q. Authorization lock

For synchronous protected commands, one process-local mutex covers authorization and handler completion. Role and membership changes therefore cannot interleave between a synchronous call's check and operation. A channel-coordinated test verifies this serialization. Lock acquisition order is authorization mutex, temporary session/database checks, then domain handler; temporary checks release their locks before the handler begins. No lock inversion or re-entry was found.

## R. Transaction and zero-mutation guarantees

The authorization mutex is not a SQLite transaction. It serializes the process boundary; domain transactions still provide atomic writes. A dispatch-core test proves denial never enters its operation closure. The full domain suite verifies rollback, idempotency, immutable ledgers, no-negative-stock protection, FEFO, receiving, returns, sales, finance, and reporting invariants.

## S. Authenticated frontend IPC

The current token and user ID exist only in module memory and are injected at the typed native boundary. Authentication secrets are absent from local storage, session storage, URLs, frontend console output, Rust logs, and persistent database session rows.

## T. Navigation and direct routes

Sidebar navigation is filtered by role capability. Direct route guards cover dashboards, sales, catalog reads and writes, Inventory reads/counts/adjustments, purchasing reads/writes/receiving, partners, reports, Finance, users, roles, and branches. Unauthorized routes render an explicit access-denied state; backend checks remain authoritative.

## U. Branch selection UX

The selector contains only branches returned by authenticated `list_branches`. Primary membership is preferred, stale selections are removed, and selected context is propagated to branch-aware requests. A selected branch does not widen backend authorization.

## V. Session expiry UX

An `unauthorized` native response clears the in-memory session and returns the user to login with an expiry message. A `forbidden` response does not destroy a valid session. Automated tests confirm persistent browser storage remains empty.

## W. Rust security tests

The Phase 7B authorization target has **11 passed, 0 failed**. It covers exact inventory registration, public allowlisting, unknown-command denial, missing/invalid/revoked sessions, insufficient roles, unauthorized branches, forged actors, null/all-branch bypasses, export denial, persisted sale ownership, unauthorized Finance, denial before operation, legitimate operations, and concurrent role-change serialization. Phase 7A tests separately cover expiry and inactive users.

## X. Complete Rust regression

`cargo test` passed **467 tests, 0 failed, 0 ignored**. This is the prior 456-test baseline plus 11 Phase 7B authorization tests. Test listing independently confirmed the exact count.

## Y. Complete frontend regression and viewports

`npm test` passed **288 tests, 0 failed, 0 skipped** in 8.4 minutes. This is the prior 284-test baseline plus four Phase 7B authorization tests. Authentication and user administration, authorization navigation, branch propagation, session invalidation, export denial, and the complete pre-existing product suite pass. Automated viewport coverage includes 1366×768, 1440×900, and 1920×1080.

## Z. Final automated checks

- `cargo fmt --check`: passed
- `cargo check`: passed
- `cargo test`: 467 passed, 0 failed, 0 ignored
- `npm run lint`: passed with zero errors
- `npm run build`: passed; only the existing Vite chunk-size advisory was emitted
- `npm test`: 288 passed, 0 failed, 0 skipped
- `git diff --check`: passed

Combined standard-suite total: **755 passed, 0 failed**.

## AA. Migration preservation

No migration was added. SHA-256 checks for migrations 001–016 exactly match the Phase 7A report, including migration 016 `949287ad42864d425a39b8a1f3543f9ab9c15ea20eab7bdfc617aae01db01bcc`. Isolated migration tests pass through v016; no real database was migrated.

## AB. Development database boundary

The native application was not launched. The real development database was not opened, read, copied, migrated, seeded, reset, or modified. Its last verified state remains **v009**.

## AC. Files and documentation

`CHANGED-FILES.md` records the Phase 7B implementation surface. `PERMISSION-MATRIX.md` contains every registered command and role grant. `SECURITY-BOUNDARIES.md` records trust, synchronization, async behavior, and residual limitations.

## AD. Remaining limitations

- Authorization is local to the application process and does not add database encryption, keychain storage, remote identity, or malicious-binary resistance.
- The mutex covers synchronous handler execution; async futures use dispatch-time authorization.
- Accepted background Catalog import work is not cancelled by a later role change.
- Legacy actor columns were not migrated to identity foreign keys.
- Future command registrations must remain inside the single guarded handler and must add an explicit policy; future async mutations need revalidation or an authorization-aware job capability.

These limitations do not create a known bypass in the 162-command Phase 7B surface, but they prevent claims beyond the documented local desktop threat model.

## AE. Recommended Phase 7C baseline

Begin Phase 7C with an isolated audit of durable audit-event attribution and security administration UX. Define any new migrations before implementation, preserve the single guarded registration, require explicit policies for new commands, add revalidation for any new async mutation, and retain the code-and-automated-test-only database restrictions until a separately authorized live validation task.

Do not implement Phase 7C as part of this phase.

**Phase 7B: COMPLETE**
