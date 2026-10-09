# M² Health — Phase 7C Final Report

## A. Baseline and preserved work

Phase 7C continued from the completed Phase 7C.1 working tree. The centralized 162-command authorization boundary, four-command public allowlist, fail-closed unknown-command behavior, authenticated frontend, canonical domain services, and all completed Phase 1–7B work remain active. No work from the shared working tree was reset or discarded.

The work used code review, static analysis, and automated tests only. The native application was not launched, no native release or installer was built, and the real development SQLite database was not opened, read, copied, or migrated.

## B. Phase 7C.2 findings and fixes

The authorization layer validated supplied actor values but allowed some legacy commands to omit the field and leave nullable attribution. Rust now exposes the validated principal inside the protected synchronous dispatch scope, and actor-bearing command adapters stamp that principal into domain transactions. An isolated scoped guard clears the principal after execution.

A minimal forward migration, 017, was justified to add immutable actor fields to Inventory batches and sale returns and to create a durable structured security-event trail. No migration from 001 through 016 changed.

## C. Actor attribution

Authenticated actor propagation now covers Sales drafts/posts/returns/void movements, Inventory batch creation and all active ledger mutations, transfers, Inventory documents and reversals, stock-count corrections, purchase receipts and returns, and expenses. The frontend no longer supplies a literal employee name for purchasing operations. Positive tests verify persisted actor IDs; negative tests verify forged actors fail and actor context cannot be used outside protected dispatch.

## D. Security events

`security_events` is strict, append-only, foreign-keyed to actor and optional subject, and limited to defined identity event types. User creation/update/activation/deactivation, password reset, role assignment/removal, and branch membership changes write events in the same `IMMEDIATE` transaction as the identity mutation. Injected event failure rolls back the mutation. Passwords and raw session tokens are absent from event data.

## E. Async authorization

The only async registered commands are the three protected read-only Catalog source/profile/report commands. Authorization occurs at dispatch. Permission changes do not retroactively cancel an already accepted read-only operation. There is no async domain mutation using a stale authorization snapshot, and the global synchronous boundary is not held across `.await`.

## F. Concurrency and revocation

Deterministic tests cover in-flight protected work racing logout and identity changes. Logout waits for the accepted synchronous request, revokes the session, and later use fails. Password reset and deactivation revoke sessions; durable role and branch membership are re-read for every request. Existing concurrent stock, receipt, return, sale, and expense tests continue to enforce atomicity, unique numbering, and no overspend. No lock re-entry or inversion was found.

## G. Phase 7C.3 findings and fixes

The audit found no unresolved critical business or accounting defect. A new complete isolated cross-domain test strengthens exact reconciliation across Catalog, partners, purchasing, Inventory, Sales, Finance, reports, and Dashboard.

## H. Catalog and package invariants

Product and package remain distinct canonical entities. Barcode ownership is package-level, duplicates are rejected, positive package-conversion metadata is preserved without guessed conversion, nullable data remains null, foreign keys remain active, and import identity/money handling remains conservative and exact. The Egyptian dataset itself was outside this task and was not read or modified.

## I. Inventory correctness

The append-only Inventory movement ledger remains the sole stock truth. FEFO, expired exclusion, batch identity, exact acquisition cost, opening stock, receiving, adjustments, counts, transfers, compensation, retry, rollback, and negative-stock protection all pass. No independent mutable balance was introduced.

## J. Sales/POS correctness

Automated coverage passes for save/hold/resume/post, multiple allocations, integer discounts and tax, payment totals, FEFO, returns/refunds, voids, snapshots, COGS, idempotency, concurrent overspend, and injected rollback failures. Sale and return actor attribution is now persisted from Rust session identity.

## K. Purchasing correctness

Orders, partial/full receipts, original-batch supplier returns, acquisition cost, supplier history, branch ownership, concurrent receiving/return protection, and idempotency pass. Purchasing continues to avoid unsupported AP, AR, settlement, cash, or payable postings.

## L. Finance integrity

Every tested posted journal balances exactly. Revenue, returns, Inventory/COGS, refunds, void reversal, expenses, sign conventions, source links, missing cost, duplicates, and rollback pass. Historical uncovered operations remain explicitly reported rather than backfilled. Finance Core is an operational subsystem and is not represented as statutory accounting.

## M. Reporting reconciliation

Operational reports, supplier/customer histories, Dashboard, Finance reports, filters, pagination, branch scope, and CSV exports reconcile to canonical records. Arabic/UTF-8 data, quoting, CRLF, and spreadsheet-formula neutralization pass.

## N. End-to-end automated pharmacy scenarios

The new Phase 7C flow creates product/package, supplier, and customer records; orders and receives five packages; returns one to the supplier; sells two; refunds one; records an expense; retries idempotent requests; checks histories, reports, Dashboard and Finance; and ends with `integrity_check = ok` and zero foreign-key violations. Final stock is exactly three. Cross-branch attempts are denied by the complementary direct authorization tests.

## O. Phase 7C.4 findings and fixes

The production purchasing UI’s hardcoded `Mohannad` actor values were removed. Browser tests prove receipt and supplier-return payloads leave actor selection to the authenticated Rust boundary. Legacy Notifications, Audit Log, Backup, Sync, and Settings routes no longer render Phase 1 fixtures in native mode; they show an explicit unavailable state until a persisted backend exists.

## P. Frontend completeness

Implemented Dashboard, Catalog, Inventory, POS, purchasing, supplier/customer, reports, Finance, authentication, users/roles, and branch-selection workflows use typed native requests in desktop mode. Navigation, loading, empty, error, conflict, retry, permission-denied, validation, and duplicate-submit states pass. Browser/demo fixtures remain outside production native flows, unsupported System previews fail honestly in native mode, and canonical backend responses own persisted totals.

## Q. Viewport results

Automated coverage passed at 1366×768, 1440×900, and 1920×1080 for authentication, users, permission denial, Dashboard, POS, purchasing, receiving, returns, partners, reports, Finance, tables, dialogs, and shell layouts. The tested pages have no document-level horizontal overflow.

## R. IPC security regression

The application still registers exactly 162 unique commands through one guarded handler. Public commands remain `get_auth_status`, `bootstrap_admin`, `login`, and `recovery_status`. Every other command has one explicit policy; unknown commands, malformed protected requests, missing sessions, invalid sessions, revoked sessions, insufficient permissions, and unauthorized branches fail before domain execution. Actor-bearing adapters use the protected principal.

## S. Migrations and real database preservation

SHA-256 values for migrations 001–016 exactly match the Phase 7A baseline. Migration 017 is forward-only, has SHA-256 `5409a80f3a75c4e4776fab220282a999dd3ad360b4efbca5c3b27f2887133963`, and was tested only on isolated databases. It adds `inventory_batches.created_by`, `sale_returns.created_by`, their immutability triggers, and immutable `security_events`. The real development database remains untouched and at its previously reported state.

## T. Exact test results

- `cargo fmt --check`: passed.
- `cargo check`: passed.
- `cargo test`: 482 passed, 0 failed, 0 ignored across executable test targets; main and documentation targets contain no tests.
- `npm run lint`: passed with zero errors and zero warnings.
- `npm run build`: passed; only the existing advisory chunk-size warning was emitted.
- `npm test`: 295 passed, 0 failed in 7.9 minutes.
- Focused Rust actor/security run: 121 passed, 0 failed.
- Focused cross-domain run: 1 passed, 0 failed.
- Focused browser security/purchasing run: 49 passed, 0 failed.
- Focused native System boundary run: 5 passed, 0 failed.
- `git diff --check`: passed.

The complete standard Rust and frontend suites contain 777 passing tests and zero failures.

## U. Remaining risks and limitations

The security boundary protects the local process and IPC/domain path; it does not add database encryption, OS keychain storage, remote identity, device-compromise protection, binary-integrity protection, or synchronization security. Accepted async reads are not cancelled after later revocation. Finance has no non-admin accountant role and remains admin-only. Finance Core excludes statutory accounting, AP/AR, tax compliance, bank reconciliation, payroll, and supplier settlement. The production JavaScript bundle still triggers Vite’s advisory size warning.

## V. Documentation and changed files

Phase reports, the security integration audit, branch-isolation evidence, cross-domain evidence, and a Phase 7C file manifest are present in `docs/phase-7c`. `CHANGED-FILES.md` separates Phase 7C work from the inherited dirty working tree.

## W. Readiness for Phase 7D

The codebase is ready for a separately authorized Phase 7D dataset-quality audit. Phase 7D was not started. That task should preserve the verified authorization and ledger boundaries, use a deliberate data-audit plan, and continue to avoid the real development database unless the user explicitly changes the current restriction.

**Phase 7C: COMPLETE**
