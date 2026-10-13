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
