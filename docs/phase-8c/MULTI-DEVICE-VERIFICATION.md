# Phase 8C Multi-device Verification

## Verified live on 2026-10-20

- Three synthetic authenticated users and devices across two tenants were created through modern-key administrative setup and user-scoped RPCs.
- An event pushed by one device was pulled by a second authorized device and acknowledged.
- Replaying the same event returned the original server identity and did not duplicate it.
- The server replaced client-supplied actor identity with `auth.uid()`.
- Refresh-token exchange succeeded.
- Cross-tenant push, malformed hash, revoked-device access, and revoked-member enrollment were denied.
- Cleanup was independently verified at zero synthetic users, organizations, and events.

## Verified locally in isolated SQLite

`tests/offline_sync.rs`: **25 passed, 0 failed**.

- Durable outbox leases, exact receipt matching, checkpoint ordering, duplicate inbox identity, retry scheduling, and dependency retry.
- Catalog products/packages/barcodes/prices and relevant partner references.
- Canonical purchase receipt and purchase return document reconstruction with exact stock replay.
- Canonical inventory document reconstruction with movement links.
- Canonical sale and sale-return reconstruction including items, allocations, payments/refunds, batches, and movements.
- Completed stock-count reconstruction when the receiving snapshot matches; `REVISION_CONFLICT` plus atomic rollback when it diverges.
- Paired transfers emitted once and applied atomically/idempotently; malformed pairs roll back.
- Immutable Finance journals apply once; negative stock and malformed events quarantine without partial state.
- Authorized conflict retry/dismissal is audited and never implements last-write-wins.

## Entity behavior

- `inventory_movement`: applied once through canonical stock constraints.
- `inventory_transfer`: both legs apply atomically as one event.
- `finance_journal`: balanced, immutable, and idempotent.
- `catalog_lookup`, `catalog_product`: canonical insert/replay; divergence quarantined.
- `partner_record`: canonical supplier/customer reference reconstruction; immutable clashes fail closed.
- `sale_document`, `sale_return`: full guarded document reconstruction and idempotent replay.
- `purchase_receipt`, `purchase_return`: full guarded document reconstruction and idempotent replay.
- `inventory_document`: DRAFT -> READY -> POSTED reconstruction; transfers wait for paired-transfer dependency.
- `stock_count`: exact-snapshot reconstruction; divergence quarantined as `REVISION_CONFLICT`.

## Conflict policy

Offline writes are never merged by last-write-wins. Immutable identity disagreement, stale revision, mismatched count snapshot, negative-stock consequence, unauthorized scope, malformed payload, or missing dependency is retained as an auditable conflict. The sync engine rolls back the complete event transaction, preserves inbox/event context, and advances only according to the durable conflict policy. Operator resolution is permission-checked and note-bearing.

Global offline reservations are intentionally not fabricated. Concurrent offline oversell converges to quarantine rather than silently creating negative stock.
