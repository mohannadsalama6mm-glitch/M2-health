# Phase 5C Final Report

## A. Phase 5C Summary

Phase 5C implements canonical purchase receiving and partial receiving. Ordered purchases can now be received through one or more immutable purchase receipts, and each successfully posted receipt writes to the existing canonical Inventory batch and movement tables. The implementation adds no second stock ledger and does not include purchase returns, supplier accounting, Phase 5D, or Phase 6 work.

## B. Repository / Inventory Audit

The existing Inventory architecture was audited before implementation. `inventory_batches` is the canonical batch store and `inventory_movements` is the append-only stock ledger. Balances are derived from movements. Both records are scoped by canonical branch and package identities. Existing `inventory::batches::create_on` and `inventory::movements::insert_referenced` services were reused.

Canonical receipt movements use `PURCHASE_RECEIPT`, a positive integer quantity, a canonical batch, branch, and package, plus durable `reference_type` and `reference_id` source fields. Inventory writes use immediate SQLite transactions. Existing no-negative-stock protections remain authoritative for outbound movements. FEFO and Sales continue to exclude expired stock.

The Phase 5B purchase schema, state guards, snapshots, revisions, and durable request patterns were also reviewed. The Phase 5C orchestration extends those structures without reactivating the legacy prototype receiving path.

## C. Migration 012

`012_purchase_receiving.sql` is registered as migration version 12 with the name `purchase_receiving`. It adds only the durable purchasing records, constraints, indexes, sequences, and forward-only trigger evolution needed for receiving.

Migration 012 applies after migrations 001–011, applies exactly once, and is covered by clean-chain, upgrade, failure-rollback, schema, foreign-key, index, and trigger tests. Migrations 001–011 remained byte-identical to the hashes recorded at the start of Phase 5C. Migration 012 SHA-256 is `6eaea591ef77595c78022012ab7b395e1018e52a6e51b7dd92655a3d276376dc`.

## D. Receipt Schema

The migration adds:

- `purchase_receipt_sequences` for date-scoped receipt numbering.
- `purchase_receipts` for immutable receipt identity, purchase, branch, request payload, posting state, quantity, actor, and timestamps.
- `purchase_receipt_lines` for purchase-line and package identity, immutable product/package snapshots, quantity, lot, expiry, actual acquisition cost, and canonical Inventory batch and movement links.
- Foreign keys and composite constraints tying receipt allocations to the purchase, branch, package, batch, and movement they represent.
- Indexes for purchase history, branch history, receipt detail, batch traceability, and movement traceability.
- Guards that prevent mutation or deletion of posted receipt history and validate the linked canonical movement.

## E. Receipt Identity / Numbering

Every receipt has a stable UUID and a human-readable `PR-YYYYMMDD-0001` number. Date-scoped sequence allocation occurs inside the same immediate transaction as receipt posting. Concurrent numbering tests prove that successful concurrent operations receive unique numbers and failed transactions do not corrupt the sequence.

## F. Purchase Lifecycle Extension

Migration 012 replaces the Phase 5B transition guard through a forward-only migration. The canonical lifecycle now supports:

`DRAFT -> ORDERED -> PARTIALLY_RECEIVED -> RECEIVED`

Receiving is accepted only from `ORDERED` and `PARTIALLY_RECEIVED`. `DRAFT`, `CANCELLED`, and `RECEIVED` are rejected. Commercial purchase data remains frozen after ordering, and terminal states remain immutable.

## G. Branch Integration

Phase 5B purchase orders do not own a branch. Each receipt therefore requires an explicit canonical branch instead of inferring a default. The backend validates that the selected branch exists and is active. The receiving UI requires the user to select and confirm the target branch. Receipt history and detail retain the chosen branch.

## H. Ordered / Received / Remaining Quantities

For each purchase line, `received_quantity` is derived from successfully `POSTED` receipt lines. `remaining_quantity` is derived as ordered minus received. These values are backend-owned and returned in purchase detail. The backend rejects zero, invalid, foreign-line, and over-remaining allocations without clamping.

## I. Partial Receiving

A receipt may include a subset of lines, part of a line, or multiple lines. A successful receipt with outstanding quantities derives `PARTIALLY_RECEIVED`; completion of every line derives `RECEIVED`. The frontend exposes Ordered, Received, Remaining, and Receive Now values and supports partial quantities explicitly.

## J. Multiple Receipts

Multiple immutable receipts can be posted against one purchase. Later receipts create new business records instead of modifying prior receipts. Status and remaining quantities are recalculated from all successfully posted receipt lines.

## K. Batch / Lot Integration

Receipt allocations use canonical Inventory batches. The service reuses an active batch only when branch, package, lot, expiry, acquisition cost, and supplier reference are exactly compatible. Otherwise it creates a canonical batch through the Inventory service. Batches with different identity-defining attributes are not merged. Lot may be omitted when canonical Inventory rules permit it.

## L. Expiry Handling

Expiry uses the canonical optional Inventory date representation and strict date validation. No-expiry batches remain supported. Canonical Inventory permits receipt of already-expired stock; Phase 5C preserves that behavior. Such stock remains excluded from FEFO and Sales allocation and remains visible to Inventory expiry monitoring.

## M. Acquisition Cost

All costs remain integer minor units. Each receipt line snapshots the actual acquisition cost used for its Inventory batch. It defaults to the ordered unit cost and may be explicitly changed at receipt time. Receipt posting never changes Catalog selling prices or historical Sales cost snapshots.

## N. Canonical Inventory Posting

Each received allocation creates or reuses a canonical Inventory batch and appends one canonical Inventory movement. Inventory remains the sole stock source of truth. No purchasing stock table, balance table, receipt ledger, or alternate movement table was introduced.

## O. PURCHASE_RECEIPT Movements

Each stock effect uses movement type `PURCHASE_RECEIPT`, the receipt quantity, canonical branch/package/batch identities, and `reference_type = 'purchase_receipt'` with the durable receipt UUID. A database trigger verifies that every linked movement has the expected type, source, quantity, branch, package, and batch.

## P. Atomicity

Posting runs in one SQLite immediate transaction: idempotency lookup, purchase/revision/status validation, remaining-quantity validation, branch/package/batch validation, receipt allocation, batch creation or reuse, canonical movement insertion, receipt-line linkage, derived purchase status/revision, and final receipt posting. Any error rolls back the receipt, lines, batches, movements, purchase update, and receipt sequence allocation.

Automated tests inject invalid lines and failures at batch, movement, and status stages and verify that no partial state remains.

## Q. Idempotency

Receiving requires a durable request UUID and stores a canonical payload representation. An exact retry returns the original posted receipt before stale-revision validation, so network retries cannot duplicate stock. Reusing the request UUID with a changed payload is rejected. Tests verify one receipt, one set of movements, and one stock increase after repeated exact requests and simulated timeout replay.

## R. Concurrency

Immediate transactions serialize the authoritative remaining-quantity check with receipt posting. In a race where two requests each attempt to consume the same remaining quantity, only one succeeds. Tests verify no over-receipt, correct final received and stock totals, and unique concurrent receipt numbers.

## S. Revision Protection

Every new receipt requires the expected purchase revision. A successful receipt advances the revision. Stale new requests fail, while exact replay of an already successful request returns the original result despite the advanced revision.

## T. Receipt-Aware Cancellation

Draft cancellation behavior is preserved. An ordered purchase with zero receipts may still be cancelled. `PARTIALLY_RECEIVED` and `RECEIVED` purchases cannot be normally cancelled. Posted receipts and movements are never deleted or altered to imitate cancellation; future corrections require a purchase-return workflow.

## U. Traceability

The persisted chain is:

`Purchase -> Purchase line -> Purchase receipt -> Receipt line -> Inventory batch -> PURCHASE_RECEIPT movement`

Receipt lines retain immutable supplier, product, package, quantity, lot, expiry, cost, branch, batch, and movement identities. Inventory movement source fields trace back to the receipt UUID.

## V. Purchase Detail Extension

Canonical purchase detail now returns per-line ordered, received, and remaining quantities plus receipt summaries. Values are derived from posted receipt records and immutable purchase snapshots rather than current mutable Catalog or Supplier values.

## W. Receipt History / Detail

Canonical paginated receipt history supports purchase, supplier, and branch filters. Receipt detail includes receipt identity, timestamps, posting state, branch, supplier and purchase snapshots, line quantities, lot, expiry, acquisition cost, and canonical batch/movement IDs. The Purchasing UI renders receipt history and navigable receipt detail.

## X. IPC

The canonical typed IPC surface adds only:

- `receive_purchase_order`
- `get_purchase_receipt`
- `list_purchase_receipts`

Tauri commands remain thin and Rust owns validation and business rules. No React SQL exists. Legacy prototype complete, void, and receive commands remain absent from the active Tauri command registry.

## Y. Receiving UI

The Purchasing UI now provides a canonical receiving route and receipt-detail route. It displays ordered, received, remaining, and receive-now quantities; requires a branch; captures lot, optional expiry, and actual cost; supports line selection and multiple lines; presents a review step; keeps the same request UUID for retries; prevents duplicate submission; displays structured errors; and refreshes canonical purchase and receipt history after success.

Receive controls appear only for `ORDERED` and `PARTIALLY_RECEIVED` purchases. Cancellation controls are unavailable after any receipt. Responsive tables use contained horizontal scrolling, dialogs remain internally scrollable, and confirmation actions remain reachable at the required viewports.

## Z. Supplier Traceability

Receipt detail retains the immutable supplier snapshot inherited from the purchase, and the paginated backend receipt query supports supplier filtering. No supplier balances, payments, fabricated metrics, or accounting records were added.

## AA. Rust Automated Tests

The new `purchase_receiving` suite contains 23 comprehensive tests covering migration evolution, lifecycle eligibility, full and partial receipts, multiple receipts, quantities, batches, lot/expiry, actual cost, Catalog isolation, Inventory movements, rollback, idempotency, concurrency, revisions, cancellation, traceability, immutability, and pagination/filter behavior.

Full Rust result: **398 passed, 0 failed**. This is the verified Phase 5B baseline of 375 plus 23 Phase 5C tests.

## AB. Frontend Automated Tests

The new Phase 5C frontend suite contains 19 tests covering action eligibility, line quantities, branch selection, lot/expiry/cost input, partial and multiline payloads, review, validation, over-receipt and stale errors, duplicate-submit prevention, timeout replay with the same request key, partial/final status, receipt history/detail, focus behavior, legacy-handler isolation, and viewports.

Full frontend result: **205 passed, 0 failed**. This is the verified Phase 5B baseline of 186 plus 19 Phase 5C tests.

## AC. Automated Viewport Tests

Mocked-IPC browser tests passed at:

- 1366 x 768
- 1440 x 900
- 1920 x 1080

They verify no page-level horizontal overflow, usable contained receipt tables, accessible quantity/lot/expiry controls, reachable review and confirmation actions, internal dialog scrolling, and practical focus behavior.

## AD. Regression Status

All required full checks passed:

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- `cargo test`: 398 passed, 0 failed
- `npm run lint`: PASS
- `npm run build`: PASS; only the existing informational bundle-size warning was emitted
- `npm test`: 205 passed, 0 failed
- `git diff --check`: PASS

Catalog, import, Inventory, documents, batches, movements, stock counts, alerts, FEFO, Sales/POS, returns/refunds/voids/holds, Partners, and Phase 5B Purchasing tests remain green.

## AE. Files Changed

The Phase 5C changed-file manifest is in `docs/phase-5c/files.txt`. The working tree also contains the verified Phase 5B baseline changes that predated Phase 5C; those are not claimed as newly authored Phase 5C work. Migration 011 appears modified relative to repository `HEAD` because it belongs to that uncommitted Phase 5B baseline, but its bytes and recorded SHA-256 did not change during Phase 5C.

## AF. Known Limitations

- Purchase returns and compensating stock reversals are deferred to Phase 5D.
- Supplier payments, balances, accounts payable, settlement, journals, and other accounting are outside this phase.
- The UI accepts one allocation per purchase line in a single receipt. Additional lots for that line can be posted through additional receipts; the backend already supports multiple allocations for the same purchase line in one request.
- Phase 5B orders have no fixed branch. Each receipt explicitly selects its target active branch, so separate receipts for one purchase may intentionally target different branches.
- Existing duplicate compatible legacy batches, if any, are resolved deterministically to the earliest active compatible batch; Phase 5C creates no new duplicate when an exact compatible batch exists.
- Live/native and real-database migration/persistence verification remain deferred under the project-wide development mode.

## AG. Real Development DB Status

The real development database was not read, copied, migrated, seeded, reset, or modified. Migrations 010, 011, and 012 were exercised only against isolated temporary/in-memory SQLite databases. The last independently verified real-database migration remains version 009.

## AH. Native Testing Status

No Tauri application, native window, interactive server, mouse/keyboard session, or manual/live smoke test was launched. All UI evidence comes from automated browser tests using mocked/test IPC, as required.

## AI. Phase 5C Status

COMPLETE

## AJ. Recommended Phase 5D Plan

Phase 5D should implement canonical Purchase Returns against the original receipt allocation. A return should identify the original purchase, receipt, receipt line, Inventory batch, movement, branch, package, quantity, and acquisition-cost snapshot; append explicit compensating canonical Inventory movements; prevent returning more than the net received quantity; preserve FEFO and no-negative-stock rules; use atomic payload-bound idempotency and optimistic concurrency; and expose paginated supplier purchase/receipt/return traceability. Supplier accounting and payments should remain a separate later scope.
