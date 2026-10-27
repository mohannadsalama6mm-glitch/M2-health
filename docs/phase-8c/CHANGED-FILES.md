# Phase 8C Changed Files

The working tree preserves all uncommitted Phase 8A and Phase 8B work. The inherited path-level inventories remain in `docs/phase-8a/CHANGED-FILES.md` and `docs/phase-8b/CHANGED-FILES.md`.

## Phase 8C additions

- `src-tauri/migrations/020_sync_rollout_controls.sql`
- `supabase/migrations/202610100002_phase_8c_security_integration.sql`
- `scripts/admin/phase8c-live-integration.ps1`
- `tests/phase8c-sync.spec.ts`
- `docs/phase-8c/AUTHENTICATED-INTEGRATION.md`
- `docs/phase-8c/MULTI-DEVICE-VERIFICATION.md`
- `docs/phase-8c/SECURITY-RELIABILITY.md`
- `docs/phase-8c/ROLLOUT-ACTIVATION.md`
- `docs/phase-8c/FINAL-REPORT.md`
- `docs/phase-8c/CHANGED-FILES.md`

## Phase 8C final-pass additions

- `src-tauri/src/cloud/replication.rs` — full canonical reconstruction for Catalog, partners, sales/returns, purchasing/returns, inventory documents, exact-snapshot counts, paired transfers, and fail-closed conflicts.
- `supabase/migrations/202610100003_phase_8c_transfer_authorization.sql` — unchanged canonical SQL; remote ledger reconciled to this version.
- `supabase/migrations/202610130001_phase_8c_revoke_rls_auto_enable.sql` — forward migration applied and privilege-verified remotely.
- `docs/phase-8c/LEGACY-KEY-DEACTIVATION.md`

## Phase 8C final-pass updates

- `src-tauri/src/cloud/mod.rs`, `outbox.rs`, `engine.rs` — replication module, enrolled-actor fallback, document-owned outbox suppression, paired transfer event, retry of open dependency/revision conflicts.
- Domain hooks emitting replication events inside the existing transactions: `db/repositories/products.rs`, `manufacturers.rs`, `categories.rs`, `routes.rs`, `active_ingredients.rs`, `product_packages.rs`, `partners/mod.rs`, `sales/mod.rs`, `sales/returns.rs`, `purchasing/receiving.rs`, `purchasing/returns.rs`, `inventory/counts/mod.rs`, `inventory/documents/mod.rs`.
- `src-tauri/tests/offline_sync.rs` — 25 focused tests including canonical purchasing, inventory documents, sales/returns, exact-snapshot counts, divergence rollback, paired transfers, revocation, and audited retry.
- `src-tauri/tests/authorization.rs` — registered command count 169.

## Phase 8C updates to inherited work

- `playwright.config.ts` — moves the automated test server to port 1421 because port 1420 is reserved on this host; application runtime configuration is unchanged.
- `src-tauri/src/db/migrations.rs` — registers SQLite migration 020.
- `src-tauri/src/cloud/mod.rs` — adds real success, failure, conflict, retry, and cloud-subject status.
- `src-tauri/src/cloud/commands.rs` — supplies authenticated subject state to the status gate.
- `src/lib/tauri/cloud.ts` — expands typed status fields and sign-in/sign-out/sync contracts.
- `src/features/system/CanonicalSync.tsx` — implements the real authenticated readiness and synchronization UI.
- `src-tauri/tests/cloud_foundation.rs` — advances isolated migration expectations and verifies the Phase 8C PostgreSQL boundary.
- Migration-version expectations in `src-tauri/tests/auth.rs`, `catalog.rs`, `finance.rs`, `foundation.rs`, `inventory_alerts.rs`, `inventory_core.rs`, `inventory_documents.rs`, `purchase_returns.rs`, `recovery.rs`, and `stock_count_sessions.rs` now expect v020.
- `tests/catalog-double.ts` and `tests/native-boundary.spec.ts` — cover the expanded cloud status/session contract and fail-closed UI.
- `supabase/tests/phase8b_security.sql` — uses the correct canonical payload hash required by the new server integrity trigger.

## Preserved Phase 8A/8B implementation

The remaining modified and untracked paths shown by Git belong to the inherited Phase 8A/8B foundation: ignored configuration rules, Rust HTTP dependency files, centralized authorization registration, Inventory/Finance transactional outbox hooks, the cloud engine and transport modules, SQLite migrations 018/019, Phase 8A/8B PostgreSQL migrations and tests, frontend routing, existing doubles, and Phase 8A/8B documentation. They were not reset or discarded.

Ignored and untracked secret-bearing paths remain:

- `config/cloud.local.json`
- `supabase/.temp/`

`git ls-files` and tracked-content secret scans confirm neither is tracked. Tracked source contains only explicit example placeholders; no real publishable key, secret key, legacy JWT, project URL, or project reference appears.
