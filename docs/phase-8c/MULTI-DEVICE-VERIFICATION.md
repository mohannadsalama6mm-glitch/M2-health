# Phase 8C Multi-device Verification

## Verified live (isolated authenticated harness, earlier in Phase 8C)

- Three synthetic devices across two tenants were enrolled through authenticated RPCs.
- An event pushed by one device was pulled by a second authorized device and acknowledged.
- Replaying the same event returned the original server sequence and did not duplicate it.
- The server replaced client-supplied actor identity with `auth.uid()`.
- Cross-tenant push, revoked-device push, and revoked-member push were denied.

The live harness was not re-run in the final pass because the legacy-key incident is still open.

## Verified locally (isolated in-memory SQLite, `tests/offline_sync.rs`, 21 tests)

- Durable outbox leases, exact receipt matching, checkpoint ordering, duplicate inbox identity, retry scheduling, dependency retry.
- Immutable Finance journals apply once; canonical Inventory constraints reject negative stock and roll back partially created batches.
- Catalog products replicate insert-if-absent; a diverged concurrent revision is quarantined as `REVISION_CONFLICT` instead of overwriting.
- A sale document without its stock dependencies is quarantined instead of partially applied.
- Paired branch transfers: a local transfer emits exactly one `inventory_transfer` event in the same transaction; a remote transfer applies both legs atomically and idempotently.
- A remote stock-count completion is quarantined as `UNSUPPORTED_ENTITY` and writes no movement.
- Authorized `RETRY` reopens a stock conflict without last-write-wins; `DISMISS` requires an audit note.

## Entity coverage

| Entity | Behavior on remote apply |
| --- | --- |
| `inventory_movement` (standalone) | Applied once through canonical constraints. |
| `inventory_transfer` | Both legs applied atomically; one event per transfer. |
| `finance_journal` | Balanced, immutable, applied once. |
| `catalog_lookup`, `catalog_product` | Insert if absent; identical replay is a no-op; divergence is quarantined. Later edits do not merge automatically. |
| `partner_record` (suppliers, customers) | Insert if absent; later revisions are not merged. |
| `sale_document`, `sale_return` | Full header, items, allocations, payments, batches, and movements in one transaction; missing dependencies are quarantined. |
| `purchase_receipt`, `purchase_return`, `inventory_document` | **Ledger only**: batches and stock movements converge; the document header and lines are not reconstructed, because local state-machine triggers require DRAFT→POSTED transitions that remote replay does not perform. |
| `stock_count` | **Quarantined** for operator review; count corrections are relative to each device's live balance and are not replayed. |

## Consistency limits

Two disconnected devices can each sell against stale availability. On convergence, a movement that would violate no-negative-stock is quarantined for operator resolution; stock is never silently overwritten. There is no reservation-based global oversell prevention.

Catalog and partner edits after creation are not merged across devices.
