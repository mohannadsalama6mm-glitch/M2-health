# Phase 6B Changed Files

## Accounting contract and evidence

- `docs/phase-6b/FINANCE-SEMANTICS.md`
- `docs/phase-6b/FINAL-REPORT.md`
- `docs/phase-6b/CHANGED-FILES.md`

## Database and Rust domain

- `src-tauri/migrations/015_finance_core.sql`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/finance/mod.rs`
- `src-tauri/src/finance/models.rs`
- `src-tauri/src/finance/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/recovery.rs`
- `src-tauri/src/sales/mod.rs`
- `src-tauri/src/sales/returns.rs`

## Rust tests and schema-version expectations

- `src-tauri/tests/finance.rs`
- `src-tauri/tests/foundation.rs`
- `src-tauri/tests/catalog.rs`
- `src-tauri/tests/inventory_alerts.rs`
- `src-tauri/tests/inventory_core.rs`
- `src-tauri/tests/inventory_documents.rs`
- `src-tauri/tests/purchase_returns.rs`
- `src-tauri/tests/recovery.rs`
- `src-tauri/tests/stock_count_sessions.rs`

## Canonical frontend

- `src/lib/tauri/finance.ts`
- `src/lib/tauri/recovery.ts`
- `src/features/finance/CanonicalFinance.tsx`
- `src/features/finance/finance.css`
- `src/app/RecoveryBoundary.tsx`
- `src/app/AppRoutes.tsx`
- `src/app/AppShell.tsx`

## Frontend tests

- `tests/finance-double.ts`
- `tests/finance.spec.ts`

Migrations 001–014 were not edited by Phase 6B. Other modified/untracked files visible in the working tree belong to the inherited Phase 5/6A baseline and are outside this manifest.
