# Phase 5D Final Report

## A. Phase 5D Summary

Phase 5D implements immutable canonical Purchase Returns against the exact Purchase Receipt → Receipt Line → Inventory Batch chain. Returns append compensating `PURCHASE_RETURN` movements to the existing Inventory ledger and never mutate receiving history.

## B. Repository Audit

The Phase 5C receipt schema, Inventory movement service, batch balances, no-negative-stock guard, Sales compensating movements, purchase idempotency/revisions, supplier data, and Purchasing UI were audited. `PURCHASE_RETURN` already existed in the canonical movement schema and Rust enum. The shared movement service was corrected to classify it as an outbound movement.

## C. Migration 013

`013_purchase_returns.sql` is registered as version 13, `purchase_returns`. It applies cleanly after 012, applies exactly once, and rolls back completely on failure. Migrations 001–012 remained byte-identical to the hashes recorded before Phase 5D. Migration 013 SHA-256 is `cac57b462b392642ab9dcf167936b5a16c49b02cb81bc54ba7685eff67777975`.

## D. Purchase Return Schema

Migration 013 adds `purchase_return_sequences`, `purchase_returns`, and `purchase_return_lines`. Composite foreign keys bind every return line to its return, purchase, receipt, receipt line, branch, package, and exact original batch. Posted headers and lines are immutable. Database guards verify historical returnable quantity and the linked canonical movement.

## E. Return Identity / Numbering

Every return has a UUID, payload-bound request UUID, and transactionally allocated `PRT-YYYYMMDD-0001` number. Concurrent numbering is unique and failed transactions do not consume durable business state.

## F. Original Receipt Traceability

Returns are receipt-scoped: one return belongs to one purchase, supplier, receipt, and branch. The backend derives supplier and branch from the posted receipt and purchase. Frontend input contains receipt-line IDs and quantities only; it cannot choose unrelated Catalog packages or batches.

## G. Returnable Quantity Derivation

Historical returnable quantity is derived from the immutable receipt quantity minus posted return lines. Each return line also snapshots the previously returned quantity with a database consistency guard, so old return detail remains historically stable after later returns.

## H. Current Stock Validation

Each return also checks the current canonical balance of the exact original batch. Both historical returnability and live batch availability must cover the requested quantity. The shared Inventory guard independently prevents negative batch and package balances.

## I. Partial / Multiple Returns

Partial returns and multiple immutable returns against one receipt line are supported. Cumulative posted quantities cannot exceed the original receipt allocation.

## J. Multi-Line Return Rules

A return may contain multiple unique lines from one posted receipt. This enforces one supplier and one branch per return. Validation completes before posting, and failure of any line rolls back all lines and movements.

## K. Exact Batch Traceability

Every return uses the exact `batch_id` stored on its source receipt line. FEFO and same-package substitution are never used for supplier returns.

## L. Acquisition Cost Preservation

Return lines copy the original receipt allocation’s acquisition-cost snapshot in integer minor units. Catalog selling prices, purchase-order prices, Sales allocations, and Sales cost snapshots remain unchanged.

## M. Canonical Inventory Movement

Every return line appends a negative `PURCHASE_RETURN` movement to `inventory_movements`, using the exact branch, package, and batch with `reference_type = 'purchase_return'` and the return UUID. No second stock ledger exists.

## N. Atomicity

Validation, numbering, return persistence, line persistence, Inventory movements, purchase revision, and posting occur in one immediate SQLite transaction. Movement failures, invalid lines, and multi-line failures roll back the header, lines, movements, sequence, revision, and stock effect.

## O. Idempotency

Exact retries are resolved before revision validation and return the original posted result. A changed payload using the same request UUID is rejected. Exact retries cannot decrement stock or consume returnable quantity twice.

## P. Concurrency

Immediate transactions serialize historical quantity and live-stock checks. Concurrent requests cannot over-return, produce negative stock, or allocate duplicate return numbers.

## Q. Revision Protection

New returns require the current purchase revision and advance it after posting. Stale new requests fail. Exact successful replay remains valid after the revision advances.

## R. Purchase / Receipt Immutability

Original receipt headers, receipt lines, batches, acquisition costs, and `PURCHASE_RECEIPT` movements remain unchanged. Return headers and lines become immutable when posted.

## S. Purchase Status Preservation

Return posting does not change receiving status. A historically `RECEIVED` purchase remains `RECEIVED`; `PARTIALLY_RECEIVED` remains `PARTIALLY_RECEIVED`. Returns are represented separately.

## T. Return Detail

Canonical detail includes return and source identities, purchase, supplier snapshot, receipt, branch, reason, timestamps, quantities, prior/remaining returnability, product/package snapshots, lot, expiry, acquisition cost, batch ID, and compensating movement ID.

## U. Return History

Database-backed history provides deterministic pagination, total counts, search, and filters for purchase, supplier, receipt, branch, and date range.

## V. Purchase Detail Integration

Purchase lines expose received, returned, historically returnable, and remaining-to-receive quantities. Purchase detail includes canonical return history without changing receiving totals.

## W. Receipt Detail Integration

Receipt lines expose received, returned, historically returnable, and current exact-batch availability. Receipt detail includes related return history and an eligible return entry point.

## X. Supplier Traceability

Return detail and history retain canonical supplier identity and immutable supplier snapshots through the purchase. Backend history supports supplier filtering, and the Purchasing return history displays supplier, purchase, receipt, and branch traceability. No supplier balances or financial state were added.

## Y. IPC

The centralized typed IPC adds `create_purchase_return`, `get_purchase_return`, and `list_purchase_returns`. Tauri commands remain thin and Rust owns all validation. No prototype return command is registered.

## Z. Purchase Return UI

The UI begins from eligible receipt lines, displays supplier, purchase, receipt, branch, product/package, received/returned/returnable/live-stock quantities, lot, expiry, cost, return quantity, and required reason. It includes review, duplicate-submit locking, retry-safe identities, detail, filtered history, pagination, empty/no-results/error states, and no arbitrary Catalog or batch selector.

## AA. Rust Automated Tests

Phase 5D adds 13 Rust tests covering migration success/rollback, full/partial/multiple/multi-line returns, source validation, exact-batch and current-stock safety, acquisition cost, immutability, atomic rollback, payload-bound idempotency, revision protection, history filters, concurrent over-return, and numbering.

Full result: **411 passed, 0 failed**.

## AB. Frontend Automated Tests

Phase 5D adds 20 frontend tests covering eligibility, source values, payloads, reason validation, partial return, confirmation, over-return, structured errors, duplicate locking, timeout replay, purchase/receipt history integration, supplier traceability, prohibited selectors/handlers, focus, and viewports.

Full result: **225 passed, 0 failed**.

## AC. Automated Viewport Tests

The return workflow passed at 1366×768, 1440×900, and 1920×1080 with no page-level horizontal overflow. Quantity/reason controls and confirmation remained reachable and keyboard focusable.

## AD. Regression Status

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- `cargo test`: 411 passed, 0 failed
- `npm run lint`: PASS
- `npm run build`: PASS; informational bundle-size warning only
- `npm test`: 225 passed, 0 failed
- `git diff --check`: PASS

Catalog, Inventory, documents, alerts, FEFO, Sales, Partners, Purchasing Core, and Purchase Receiving remain green.

## AE. Files Changed

See `docs/phase-5d/files.txt`. The working tree also contains the verified uncommitted Phase 5B and 5C baselines. Migration 011 appears modified relative to repository `HEAD` but remained byte-identical during Phase 5D.

## AF. Known Limitations

- Posted Purchase Returns cannot be edited, deleted, voided, or reversed; a future explicit compensating workflow is required.
- Supplier credits, refunds, payments, balances, accounts payable, journals, and settlement are excluded.
- A return is intentionally limited to one canonical receipt, which guarantees one supplier and branch. Separate receipts require separate returns.
- Native and real-database migration/persistence verification remain deferred.

## AG. Real Development DB Status

The real development database was not read, copied, migrated, seeded, reset, or modified. Migrations 010–013 were tested only with isolated in-memory or temporary SQLite databases. The last independently verified real database version remains 009.

## AH. Native Testing Status

No Tauri application, native window, interactive server, or manual/live smoke test was launched. Browser tests used mocked/test IPC only.

## AI. Phase 5D Status

COMPLETE

## AJ. Recommended Phase 5E Plan

Phase 5E should integrate and polish the complete Phase 5 surface, connect canonical Customer identities to Sales history where appropriate, remove remaining honest placeholders, align Purchasing and Partner navigation, and run a full automated Phase 5 regression pass. It should preserve Inventory, receipt, return, Sales, supplier, and customer boundaries and should not introduce Finance or payment-accounting behavior.
