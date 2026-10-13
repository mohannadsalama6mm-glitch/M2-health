# Phase 8B Changed Files

This working tree contains the inherited, uncommitted Phase 8A foundation and the Phase 8B continuation. No existing work was reset or discarded.

## Repository and configuration

- `.gitignore` — ignores local cloud configuration and Supabase CLI state.
- `config/cloud.example.json` — placeholder-only, disabled cloud configuration example.
- `src-tauri/Cargo.toml` — adds the blocking Rustls HTTP client.
- `src-tauri/Cargo.lock` — locks the transport dependency graph.

## Inherited Phase 8A documentation

- `docs/phase-8a/ARCHITECTURE-AUDIT.md`
- `docs/phase-8a/CHANGED-FILES.md`
- `docs/phase-8a/CLOUD-IDENTITY.md`
- `docs/phase-8a/FINAL-REPORT.md`
- `docs/phase-8a/LOCAL-FOUNDATION.md`
- `docs/phase-8a/MIGRATION-SAFETY.md`
- `docs/phase-8a/POSTGRES-RLS.md`
- `docs/phase-8a/SYNC-PROTOCOL.md`
- `docs/phase-8a/TENANCY-MODEL.md`

## Phase 8B documentation

- `docs/phase-8b/CHANGED-FILES.md`
- `docs/phase-8b/FINAL-REPORT.md`
- `docs/phase-8b/SECURITY-VERIFICATION.md`
- `docs/phase-8b/SYNC-ENGINE.md`

## SQLite and Rust cloud engine

- `src-tauri/migrations/018_cloud_foundation.sql` — inherited dormant tenant/device/protocol foundation.
- `src-tauri/migrations/019_offline_sync_engine.sql` — leases, sequence/hash identity, inbox envelope, conflicts, attempts, apply context, and Phase 8C gate.
- `src-tauri/src/cloud/commands.rs` — protected status, cloud sign-in/out, and sync-cycle IPC.
- `src-tauri/src/cloud/config.rs` — environment/ignored-file configuration and validation.
- `src-tauri/src/cloud/engine.rs` — push, acknowledgment, pull, inbox, application, conflict, checkpoint, and retry engine.
- `src-tauri/src/cloud/mod.rs` — cloud status and fail-closed gate composition.
- `src-tauri/src/cloud/models.rs` — typed event, receipt, session, and result contracts.
- `src-tauri/src/cloud/outbox.rs` — canonical payload hashing and transactional enqueue.
- `src-tauri/src/cloud/transport.rs` — authenticated Supabase Auth/RPC transport and memory-only token refresh.
- `src-tauri/src/db/migrations.rs` — registers migrations 018 and 019.
- `src-tauri/src/lib.rs` — manages cloud state and registers cloud commands.
- `src-tauri/src/authorization.rs` — explicit policies for all cloud IPC.
- `src-tauri/src/inventory/models.rs` — serializable canonical movement types.
- `src-tauri/src/inventory/movements.rs` — same-transaction inventory outbox hook.
- `src-tauri/src/finance/mod.rs` — same-transaction Finance journal outbox hook and canonical actors.

## Rust tests

- `src-tauri/tests/cloud_foundation.rs` — Phase 8A isolation, schema, RLS, and migration tests.
- `src-tauri/tests/offline_sync.rs` — Phase 8B gates, identity, atomicity, retry, receipt, inbox, conflict, inventory, and Finance tests.
- `src-tauri/tests/auth.rs`
- `src-tauri/tests/authorization.rs`
- `src-tauri/tests/catalog.rs`
- `src-tauri/tests/finance.rs`
- `src-tauri/tests/foundation.rs`
- `src-tauri/tests/inventory_alerts.rs`
- `src-tauri/tests/inventory_core.rs`
- `src-tauri/tests/inventory_documents.rs`
- `src-tauri/tests/purchase_returns.rs`
- `src-tauri/tests/recovery.rs`
- `src-tauri/tests/stock_count_sessions.rs`

The existing tests listed after `offline_sync.rs` were updated only for the forward schema sequence and isolated migration expectations.

## Frontend and Playwright

- `src/app/AppRoutes.tsx` — routes Sync to the canonical status page.
- `src/features/system/CanonicalSync.tsx` — real local cloud/sync status UI.
- `src/lib/tauri/cloud.ts` — typed cloud IPC; credentials are never persisted.
- `tests/catalog-double.ts` — cloud status test double.
- `tests/continuation.spec.ts` — canonical Sync route expectation.
- `tests/native-boundary.spec.ts` — native cloud-status boundary coverage.

## Supabase

- `supabase/migrations/202610090001_phase_8a_cloud_foundation.sql` — inherited PostgreSQL tenant/RLS foundation.
- `supabase/migrations/202610100001_phase_8b_sync_engine.sql` — authenticated enrollment and push/pull/ack RPC protocol.
- `supabase/tests/phase8b_security.sql` — rollback-only runtime tenant, branch, device, replay, actor, acknowledgment, and revocation checks.

`config/cloud.local.json` and `supabase/.temp/` are intentionally ignored and are not tracked files.
