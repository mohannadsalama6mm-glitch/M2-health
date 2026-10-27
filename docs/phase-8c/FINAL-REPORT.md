# Phase 8C Final Report

## Status

**Phase 8C: COMPLETE**

Desktop Product Closure Gate prerequisite: **COMPLETE on 2026-10-20**. See `../desktop-product-closure/COMPLETION-REPORT.md`. Phase 8C resumed only after that gate passed.

All implementation, isolated local regression, remote migration, RLS, and authenticated multi-device checks completed successfully. On 2026-10-20, the project owner disabled the legacy JWT-based API keys and the final status-only verification confirmed that both legacy keys are rejected while the approved modern credentials and security paths remain operational. No material Phase 8C blocker remains.

## A. Domain replication closure

- Catalog products, packages, barcodes, price history, lookup references, suppliers, and customers replicate through canonical records with idempotent replay and fail-closed divergence handling.
- Sales and sale returns reconstruct canonical headers, items, allocations, payments/refunds, batches, and movements through guarded local state transitions. Exact replay is a no-op.
- Purchase receipts and returns reconstruct their canonical documents, lines, batches, movements, order state, and revisions. Exact replay is a no-op.
- Inventory documents reconstruct DRAFT -> READY -> POSTED state, lines, movement links, batches, and movements. Transfers require the paired transfer dependency and fail closed when delivered out of order.
- Completed stock counts reconstruct sessions, lines, results, and correction movements only when the receiving device has the exact captured batch snapshot. Divergence returns `REVISION_CONFLICT`; the sync engine transaction quarantines the event with no partial domain writes. No last-write-wins path exists.
- Paired branch transfers remain one event and apply both legs atomically and idempotently. A malformed or incomplete pair rolls back and is quarantined.
- Conflict records retain event, organization, branch, device, entity, reason, detail, timestamps, and payload through the inbox/conflict audit trail. Authorized `RETRY` and `DISMISS` require an audit note and never directly mutate domain data.

## B. Remote migration and security evidence

- The duplicate remote transfer ledger entry was marked reverted and the already-applied transfer authorization schema was reconciled to canonical local version `202610100003`; historical migration SQL was not edited.
- Forward migration `202610130001_phase_8c_revoke_rls_auto_enable.sql` was dry-run and applied successfully.
- The linked ledger now matches local versions `202610090001`, `202610100001`, `202610100002`, `202610100003`, and `202610130001` exactly.
- A direct read-only privilege check returned `false` for `public`, `anon`, and `authenticated` execute permission on `public.rls_auto_enable()`.
- The rollback-only `supabase/tests/phase8b_security.sql` completed successfully against the development project, covering tenant/branch RLS, denied direct mutation, enrollment, idempotent push, server-derived actor, pull/ack, conflicting replay, cross-branch denial, and revoked-member denial.

## C. Authenticated multi-device evidence

The current live harness used transient modern keys held only in process memory and three synthetic authenticated users across two tenants and three devices. It passed bootstrap, membership grant, enrollment, push, duplicate replay, second-device pull, durable acknowledgment, server-derived actor identity, refresh-token exchange, cross-tenant denial, malformed-hash denial, revoked-device denial, and revoked-member denial.

Independent cleanup verification returned zero synthetic Auth users, organizations, and events.

## D. Credential incident

Repository, ignored-path, database integration, webhook, cron, foreign-server, and available log audits found no known consumer of the exposed legacy key. External consumers outside the repository still cannot be disproved because API-gateway per-key attribution is unavailable.

Final credential verification on 2026-10-20 retrieved credential material only into transient process memory and emitted status codes and booleans only:

- legacy `anon` API key: HTTP 401, rejected;
- legacy `service_role` API key: HTTP 401, rejected;
- approved publishable key against the client-safe Auth settings endpoint: HTTP 200;
- approved secret key against the server/admin Auth users endpoint: HTTP 200.

The modern-key authenticated harness then passed again. No secret value was printed, logged, copied into source, documentation, configuration, frontend storage, or this report. The credential incident is closed.

## E. Rollout gates

All rollout and transport gates remain disabled:

- `config/cloud.example.json`: `transportEnabled: false`;
- current process `M2_SYNC_TRANSPORT_ENABLED`: not enabled;
- no `M2_HEALTH_CLOUD_CONFIG` override and no debug `config/cloud.local.json` are present;
- SQLite `sync_schema_metadata.transport_enabled = 0`;
- SQLite `sync_runtime_gate.phase8c_verified = 0`.

The engine refuses network synchronization unless every gate passes. The real development SQLite database was not opened, queried, migrated, seeded, copied, or reset. Its metadata-only baseline remains unchanged at 67,641,344 bytes and `2026-10-09T13:15:34.0714897Z` last-write time.

## F. Current verification results

- Focused `offline_sync`: **25 passed, 0 failed**.
- `cargo fmt --all -- --check`: passed.
- `cargo check`: passed with `CARGO_BUILD_JOBS=1`; only the H: drive incremental hard-link fallback warning was emitted.
- Full `cargo test`: all Rust unit, integration, scale, and doc-test binaries passed using isolated workspace temp storage.
- `npm run lint`: passed.
- `npm run build`: passed; existing chunk-size advisory only.
- `npm test`: **304 passed, 0 failed** in 5.9 minutes. The first sandboxed attempts could not bind the local Vite socket and produced no test results; the approved unrestricted retry passed.
- `git diff --check`: passed; line-ending notices only.
- Historical SQLite migrations 001-020 and pre-existing cloud migration SQL are unchanged.
- Final legacy-key rejection probe: passed for both `anon` and `service_role` with HTTP 401; values were not emitted.
- Final modern-key probe: publishable client-safe endpoint and secret server/admin endpoint both returned HTTP 200.
- Rollback-only live RLS script: passed again after legacy-key deactivation, including tenant/branch RLS and revocation coverage.
- Authenticated live multi-device harness: passed again with three users, two tenants, and three devices; authentication/refresh, tenant isolation, push/pull, duplicate replay, server-derived actor, device revocation, and member revocation all passed.
- Direct `rls_auto_enable()` privilege check remains `false` for `public`, `anon`, and `authenticated`.
- Cleanup independently verified zero synthetic Auth users, organizations, and events.
- Full Rust and Playwright suites were not rerun because this final verification made no product or test-code change; their previously recorded passing results remain the applicable regression evidence.

## G. Remaining blockers

None.

No Phase 9A/9B or Phase 10 work has started.
