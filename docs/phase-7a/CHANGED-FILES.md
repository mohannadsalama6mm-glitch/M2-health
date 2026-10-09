# Phase 7A Changed Files

## Security contract and completion evidence

- `docs/phase-7a/SECURITY-SEMANTICS.md`
- `docs/phase-7a/FINAL-REPORT.md`
- `docs/phase-7a/CHANGED-FILES.md`

## Identity schema and Rust backend

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/migrations/016_identity_foundation.sql`
- `src-tauri/src/auth/mod.rs`
- `src-tauri/src/auth/models.rs`
- `src-tauri/src/auth/commands.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/recovery.rs`

## Rust tests and latest-schema expectations

- `src-tauri/tests/auth.rs`
- `src-tauri/tests/foundation.rs`
- `src-tauri/tests/catalog.rs`
- `src-tauri/tests/finance.rs`
- `src-tauri/tests/inventory_alerts.rs`
- `src-tauri/tests/inventory_core.rs`
- `src-tauri/tests/inventory_documents.rs`
- `src-tauri/tests/purchase_returns.rs`
- `src-tauri/tests/recovery.rs`
- `src-tauri/tests/stock_count_sessions.rs`

## Canonical frontend

- `src/lib/tauri/auth.ts`
- `src/lib/tauri/recovery.ts`
- `src/app/AuthContext.tsx`
- `src/app/App.tsx`
- `src/app/AppRoutes.tsx`
- `src/app/AppShell.tsx`
- `src/app/desktop.css`
- `src/features/auth/AuthGate.tsx`
- `src/features/management/CanonicalPeople.tsx`

## Frontend tests

- `tests/auth-double.ts`
- `tests/auth.spec.ts`
- `tests/catalog-import.spec.ts`
- `tests/native-boundary.spec.ts`
- `tests/canonical-partners.spec.ts`

Migration files 001–015 were not edited by Phase 7A. Migration 011 remains shown as modified relative to repository `HEAD` because it belongs to the inherited, verified Phase 5B working-tree baseline. Migrations 012–015 are likewise inherited untracked baseline files. Phase 7A appends migration 016 and updates only tests whose latest-schema expectation moved from 15 to 16. Other modified and untracked Phase 4–6 files visible in the shared working tree predate Phase 7A and are outside this manifest.
