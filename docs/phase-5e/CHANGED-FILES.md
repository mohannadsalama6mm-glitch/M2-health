# Phase 5E Changed Files

## Schema and registry
- `src-tauri/migrations/014_customer_sales.sql`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/lib.rs`

## Rust customer-sales integration
- `src-tauri/src/sales/models.rs`
- `src-tauri/src/sales/mod.rs`
- `src-tauri/src/sales/commands.rs`

## Frontend integration
- `src/lib/tauri/canonical-sales.ts`
- `src/features/sales/CanonicalSales.tsx`
- `src/features/sales/sales.css`
- `src/features/relationships/CanonicalPartners.tsx`

## Automated tests
- `src-tauri/tests/phase5_integration.rs`
- `src-tauri/tests/canonical_sales.rs`
- `src-tauri/tests/canonical_partners.rs`
- `src-tauri/tests/foundation.rs`
- `src-tauri/tests/recovery.rs`
- `src-tauri/tests/catalog.rs`
- `src-tauri/tests/inventory_alerts.rs`
- `src-tauri/tests/inventory_core.rs`
- `src-tauri/tests/inventory_documents.rs`
- `src-tauri/tests/stock_count_sessions.rs`
- `src-tauri/tests/purchase_returns.rs`
- `tests/canonical-sales-double.ts`
- `tests/canonical-sales.spec.ts`
- `tests/canonical-partners-double.ts`
- `tests/canonical-partners.spec.ts`

Schema-version assertion changes are required because isolated tests now end at migration 014. Migrations 001–013 were not edited by Phase 5E.
