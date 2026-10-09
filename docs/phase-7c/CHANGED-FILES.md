# Phase 7C Changed Files

## Backend and schema

- `src-tauri/migrations/017_security_events.sql` — immutable security events plus batch and sale-return actor columns.
- `src-tauri/src/db/migrations.rs` — registers migration 017.
- `src-tauri/src/authorization.rs` — guarded principal scope and authoritative actor access.
- `src-tauri/src/auth/mod.rs`, `src-tauri/src/auth/commands.rs` — atomic structured identity events and validated administrator attribution.
- `src-tauri/src/finance/commands.rs` — authenticated expense actor.
- `src-tauri/src/purchasing/commands.rs`, `src-tauri/src/purchasing/mod.rs` — authenticated receipt/return actors.
- `src-tauri/src/inventory/batches.rs`, `movements.rs`, `transfers.rs`, `commands.rs` — authenticated batch and ledger attribution.
- `src-tauri/src/inventory/documents/mod.rs`, `documents/commands.rs` — authenticated save/post/reversal attribution.
- `src-tauri/src/inventory/counts/mod.rs`, `counts/commands.rs` — authenticated count and correction attribution.
- `src-tauri/src/sales/mod.rs`, `sales/returns.rs`, `sales/commands.rs` — authenticated sale, return, void, and ledger attribution.

## Automated tests

- `src-tauri/tests/auth.rs`
- `src-tauri/tests/authorization.rs`
- `src-tauri/tests/canonical_sales.rs`
- `src-tauri/tests/inventory_core.rs`
- `src-tauri/tests/phase5_integration.rs`
- Latest-schema expectations in foundation/recovery and domain migration tests were advanced from 16 to 17 without weakening earlier migration fixtures.
- `tests/authorization.spec.ts`
- `tests/purchase-receiving.spec.ts`
- `tests/purchase-returns.spec.ts`
- `tests/native-boundary.spec.ts`

## Frontend

- `src/features/purchases/CanonicalPurchases.tsx` — removes literal actor names from receipt and return requests.
- `src/app/AppRoutes.tsx` — prevents Phase 1 System fixtures from rendering in native mode.

## Documentation

- `docs/phase-7c/7C1-SECURITY-REPORT.md`
- `docs/phase-7c/7C2-ACTOR-CONCURRENCY-REPORT.md`
- `docs/phase-7c/7C3-BUSINESS-INTEGRITY-REPORT.md`
- `docs/phase-7c/7C4-FRONTEND-REGRESSION-REPORT.md`
- `docs/phase-7c/SECURITY-INTEGRATION-AUDIT.md`
- `docs/phase-7c/BRANCH-ISOLATION-VERIFICATION.md`
- `docs/phase-7c/CROSS-DOMAIN-REGRESSION.md`
- `docs/phase-7c/FINAL-REPORT.md`

The working tree also contains completed work from earlier phases. Phase 7C preserved those changes and did not reset or replace them.
