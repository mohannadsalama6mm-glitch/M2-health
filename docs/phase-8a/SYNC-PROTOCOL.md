# Sync Protocol and Entity Classification

## Envelope

The version 1 envelope has an immutable event UUID, organization UUID, optional branch UUID, enrolled device UUID, actor identity, entity type and stable entity ID, operation, JSON payload, schema version, optional causation UUID, and occurrence timestamp. Server ingestion assigns an organization-scoped monotonic sequence. Event identity and receipts provide idempotency; checkpoints advance only after durable application. Retry metadata is mutable but the event envelope is not.

## Entity matrix

| Entity | Scope and source of truth | Direction / mutability | Conflict and readiness |
|---|---|---|---|
| Products, packages, ingredients, categories, manufacturers | Organization; canonical Catalog rows | Bidirectional reference data; mutable with revision | Server revision/arbitration required; not yet safe |
| Barcodes | Organization; canonical package ownership | Bidirectional; mutable uniqueness mapping | Server uniqueness arbitration required; not yet safe |
| Prices | Organization or branch by future contract; price history | Append effective-dated entries | Reject overlapping/duplicate periods; contract pending |
| Inventory batches | Branch; canonical batch identity | Append/create plus constrained metadata correction | Server validates branch and package; hooks pending |
| Inventory movements | Branch; sole stock truth | Append-only in causal order | Never last-write-wins; server validates no overspend and transfer pairing; hooks pending |
| Derived stock balances and alerts | Branch; projection of movements/settings | Recompute, do not synchronize as truth | Safe only as disposable projection |
| Sales, holds, returns, refunds, voids | Branch; canonical Sales records and linked movements/journals | State machine plus append-only posted effects | Server arbitration, idempotent operation IDs, causal ordering; hooks pending |
| Purchase orders, receipts, supplier returns | Branch; canonical Purchasing records | State machine plus append-only receipts/returns | Server validates remaining quantities and linked movements; hooks pending |
| Suppliers and customers | Organization; canonical partner rows | Bidirectional mutable reference data | Revision merge/manual resolution; identity contract pending |
| Expenses | Organization or branch; canonical expense and journal | Draft mutable, posted append-only | Posted entries never overwritten; hooks pending |
| Finance journals and lines | Organization/branch; canonical balanced journal | Append-only | Never last-write-wins; server balance/source uniqueness checks required |
| Security events | Organization; append-only audit source | Upload append-only | Server actor/device attribution; no client rewrite |
| Users, roles, organization/branch memberships | Trusted cloud control plane | Server-to-device projection | Never general sync; trusted administrative APIs only |
| Reports, Dashboard, aggregates | Derived from canonical domains | Do not synchronize as source records | Recompute from acknowledged canonical data |

## Ordering and conflicts

UUIDs provide stable identity, not order. The client retains causal IDs and occurrence time; the server sequence is authoritative for pull ordering. Clocks are informational. Duplicate event IDs return the existing receipt. Conflicting mutable reference data requires revisions and explicit resolution. Inventory availability, receipt/return quantities, Sales finalization, and Finance posting require a server transaction against acknowledged prior state.

Phase 8A persists empty protocol structures only. It does not assert that existing business transactions emit reliable events. Phase 8B must add same-transaction outbox hooks and domain-specific apply handlers before any entity above becomes synchronizable.
