# Phase 8C Final Report

## Status

**Phase 8C: NOT COMPLETE**

Code, tests, and fail-closed rollout controls pass. Completion is blocked by:

1. **Open credential incident.** The exposed legacy Supabase `service_role` JWT is still active (confirmed by a redacted HTTP 200 probe). Deactivation is prepared in `LEGACY-KEY-DEACTIVATION.md` and awaits explicit owner approval.
2. **Open server finding.** `public.rls_auto_enable()` remains executable by `anon` and `authenticated`; the revoke migration exists locally but is not applied.
3. **Partial document replication.** Purchase receipts, purchase returns, and inventory documents converge at the stock-ledger level only; stock counts are quarantined rather than replicated.

## A. Final-pass work

- New `cloud/replication.rs`: Catalog lookups and products, suppliers and customers, sales, sale returns, purchase receipts and returns, inventory documents, and stock counts emit one replication event inside the domain transaction.
- Remote apply is insert-if-absent and fail-closed. Divergent Catalog revisions raise `REVISION_CONFLICT`; immutable identity clashes raise `IMMUTABLE_CONFLICT`; missing dependencies raise `MISSING_DEPENDENCY`; stock counts raise `UNSUPPORTED_ENTITY`. Nothing uses last-write-wins.
- Paired branch transfers: one `inventory_transfer` event per transfer; remote apply writes both legs atomically and idempotently. Transfer legs are never emitted or applied independently.
- Document-owned movements are no longer emitted as standalone `inventory_movement` events, so stock is never double-applied.
- Open `MISSING_DEPENDENCY`, `REVISION_CONFLICT`, and `APPLY_FAILED` inbox rows are retried on later cycles; authorized `RETRY`/`DISMISS` with audit note is available.

## B. Dependency audit (read-only)

No tracked or untracked consumer of the legacy key. No Edge Functions, database webhooks, `pg_net`, `pg_cron`, or foreign servers. Last-24-hour Postgres, PgBouncer, and PostgREST logs contain no `service_role` mentions. API-gateway logs are not exposed, so per-key request attribution is unavailable. The declined vault query was not retried. External consumers outside this repository cannot be ruled out. Details: `LEGACY-KEY-DEACTIVATION.md`.

## C. Multi-device and offline

See `MULTI-DEVICE-VERIFICATION.md`. The live authenticated harness passed earlier in Phase 8C and was not re-run in this pass because the credential incident is open.

## D. Security and RLS

The rollback-only PostgreSQL security script (`supabase/tests/phase8b_security.sql`) was re-run against the development project on 2026-10-13 and passed every assertion: tenant and branch RLS, denied direct mutation, enrollment, idempotent push, server-derived actor, pull, durable acknowledgment, conflicting-replay rejection, cross-branch denial, and revoked-member denial. A follow-up query found 0 fixture users, organizations, or events; the project holds 0 organizations and 0 events overall.

Security advisor: `rls_auto_enable` WARN (open); eight `authenticated` sync RPCs are the intended API.

Remote migration ledger: `202610090001`, `202610100001`, `202610100002`, and `20261013202018` (`phase_8c_transfer_authorization`, which corresponds to local file `202610100003`). This mismatch must be reconciled before the next CLI push.

## E. Rollout gates

All remain disabled: `config/cloud.example.json` `transportEnabled: false`; SQLite `sync_schema_metadata.transport_enabled = 0`; `sync_runtime_gate.phase8c_verified = 0`. The engine refuses to run unless every gate passes.

## F. Test results (final pass)

- Focused `offline_sync`: 21 passed.
- Focused `tests/phase8c-sync.spec.ts`: 6 passed.
- `cargo fmt --check`: passed.
- Full `cargo test` (single-threaded): **517 passed, 0 failed**.
- `npm run lint`: passed. `npm run build`: passed (existing chunk-size advisory only).
- Full Playwright: **301 passed, 0 failed**.
- PostgreSQL rollback-only security script: passed.

## G. Secrets and local data

- `git ls-files` contains no `config/cloud.local.json`, `supabase/.temp/`, or `.env`.
- Tracked and untracked non-ignored files contain no legacy JWT or `sb_secret_` value; the only `sb_publishable_` matches are `replace…` placeholders. The project reference was removed from `AUTHENTICATED-INTEGRATION.md`.
- Real development SQLite (`%LOCALAPPDATA%\com.m2health.pharmacy\m2-health.db`) was not opened: last write 2026-10-09 13:15:34 UTC, size 67,641,344 bytes, unchanged.

## H. Outstanding items before completion

1. Owner approval and execution of `LEGACY-KEY-DEACTIVATION.md`, followed by a redacted 401 probe and harness re-run.
2. Apply `202610130001_phase_8c_revoke_rls_auto_enable.sql` (approval required; the previous attempt hit a ledger-version collision) and reconcile the transfer migration version.
3. Decide whether ledger-level convergence for purchase and inventory documents and quarantine for stock counts are acceptable, or design full document-state replay.
4. Decide whether quarantining offline oversells is sufficient.

## I. Phase 9

Not started.
