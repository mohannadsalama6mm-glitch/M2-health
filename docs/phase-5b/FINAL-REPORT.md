# M² Health Phase 5B Final Report

## A. Phase 5B Summary

Phase 5B adds the canonical Purchase Order domain and its pre-receiving lifecycle. Drafts are persisted through migration 011, linked to canonical suppliers and Catalog packages, calculated in integer minor units, protected by revisions and durable request identities, and exposed through typed IPC and a canonical Purchasing UI. The implemented lifecycle is `DRAFT → ORDERED` plus valid draft/ordered cancellation. No operation creates a batch, movement, balance change, receipt, return, payment, or accounting record.

## B. Repository Audit

The older purchase prototype in `commands/purchases.rs`, `db/models/purchases.rs`, `db/repositories/purchases.rs`, `db/services/purchases.rs`, the original Purchases UI, and the inactive `partners_purchases.sql` proposal immediately completes purchases and changes stock. It was classified as dangerous for Phase 5B and remains unregistered. Reused pieces are the route shell, design system, integer-money convention, canonical supplier UUIDs, Catalog package identity, Sales request-id patterns, and revision/document-number patterns.

## C. Migration 011

`011_purchasing.sql` is registered once as version 11 with name `purchasing`. Clean 001→011 and isolated 010→011 upgrades, apply-once behavior, and Phase 5A partner preservation are tested. Historical migrations 001–010 have no working-tree source diff. Migration 011 was never applied to the real development database.

## D. Purchase Schema

Migration 011 creates `purchase_order_sequences`, `purchase_orders`, `purchase_order_lines`, and `purchase_order_requests`, with list/supplier/status indexes and guards for retained history, immutable identities, revisions, lifecycle transitions, ordered-document freezing, draft-only line changes, and immutable requests. It creates no receipt, batch, movement, return, payable, or settlement schema.

## E. Purchase Identity / Numbering

Orders use stable UUIDs. An IMMEDIATE transaction atomically advances a daily sequence and creates `PO-YYYYMMDD-0001` numbers. Concurrent file-backed creation is tested for unique sequential values. Failed creation rolls back both document and sequence.

## F. Supplier Integration

Every order references a canonical Phase 5A supplier UUID. Rust rejects malformed, missing, or inactive suppliers and revalidates on ordering. Ordering freezes supplier code and name, so historical detail remains readable after deactivation.

## G. Catalog / Package Integration

Lines reference canonical `product_id` and stockable `product_package_id`. Backend search covers product names, scientific name, package label, manufacturer, and exact barcode. Current acquisition cost may seed the editor; selling prices and price history are never changed.

## H. Purchase Line Model

Each line stores a UUID, deterministic position, product/package foreign keys, display snapshots, positive integer quantity, integer acquisition cost, and checked total. Duplicate packages are rejected. Atomic line replacement is draft-only.

## I. Money / Totals

All money uses integer EGP minor units within the JavaScript-safe range. Rust calculates line totals, subtotal, discount, tax, and grand total; SQLite checks enforce the relationships. Frontend totals aid editing, while returned backend totals are authoritative.

## J. Purchase Lifecycle

Active transitions are `DRAFT → ORDERED`, `DRAFT → CANCELLED`, and `ORDERED → CANCELLED`. Ordered documents freeze commercial data. Cancelled documents are terminal and retained. `PARTIALLY_RECEIVED` and `RECEIVED` are reserved schema values, but Phase 5B cannot enter them.

## K. Draft Creation

Creation atomically persists supplier, optional reference/date/notes, lines, adjustments, calculated totals, UUID, request identity, and purchase number. Empty drafts may be saved; ordering requires at least one valid line. Creation has zero inventory effect.

## L. Draft Editing

Draft editing supports supplier, metadata, lines, quantities, costs, discount, and tax. Expected revision prevents stale writes. Any validation or line failure rolls back the header, totals, and line set.

## M. Ordering

Ordering explicitly validates revision, draft state, active supplier, non-empty lines, active packages, and totals before freezing snapshots. It does not create batches, movements, receipts, or stock changes.

## N. Cancellation

Cancellation explicitly supports drafts and ordered purchases, requires a bounded nonblank reason, increments revision, and retains history. Ordered cancellation preserves `ordered_at`. Receipt-aware rules are deferred until receiving exists.

## O. Idempotency

Create, order, and cancel store durable request UUIDs with exact serialized payloads. Exact retries return the existing result; request-ID reuse with a different payload is rejected. Unique kind/document constraints prevent duplicate lifecycle actions.

## P. Optimistic Concurrency

Update, order, and cancel require expected revision. Stale callers receive a structured conflict without overwriting data. Database guards require each permitted header change to increment revision exactly once.

## Q. Historical Snapshots

Ordered documents preserve supplier code/name. Lines preserve product/package labels, quantity, cost, and total. Purchase identity, number, creation timestamp, and request records are immutable.

## R. Purchase List / Search / Pagination

SQLite performs search across purchase number, supplier code/name, and supplier reference; supplier/status/date filters; total count; bounded pagination; and deterministic newest-first ordering. React requests 20 rows at a time.

## S. Purchase Detail

Detail exposes identity, number, state, revision, supplier identity/snapshots, dates, references, notes, lifecycle metadata, complete lines, and backend totals. It shows no fabricated receiving data.

## T. IPC

The typed client exposes create draft, update draft, order, cancel, get, list, and package search. Thin Tauri commands delegate to Rust. No SQL or scattered raw invoke was added, and prototype completion/void handlers remain absent from the registry.

## U. Purchasing UI

The native workspace provides list/search/filter/pagination, creation, active-supplier search, Catalog package search, line editing, totals, save/edit/detail, order confirmation, cancellation reason, loading/empty/no-result/error/retry states, stale-conflict feedback, and pending-action locks. Ordered/cancelled records are read-only. Receive controls are disabled and the legacy receive URL is non-operational with honest messaging.

## V. Inventory Isolation Evidence

The module imports no Inventory posting service and its migration writes no Inventory tables. Isolated tests snapshot movements, batches, and balances around create, edit, order, and cancel and prove they remain unchanged. Browser tests prove receive entry points invoke no completion, receipt, document-posting, or stock command.

## W. Rust Automated Tests

The 18 new tests cover migrations/schema scope, validation, drafts/totals, rollback, concurrent numbering, editing, stale writes, ordering/snapshots/idempotency, cancellation, terminal behavior, database list/detail, Catalog price preservation, and inventory isolation.

Full Rust result: **375 passed, 0 failed**.

## X. Frontend Automated Tests

The 27 new tests cover canonical command isolation, all list states, backend filters/pagination, supplier/package selectors, line editing, totals, save/edit conflicts, lifecycle confirmations, cancellation, read-only states, IPC errors, duplicate submission, and non-operational receiving.

Full frontend result: **186 passed, 0 failed**.

## Y. Automated Viewport Tests

The purchase editor passed at 1366×768, 1440×900, and 1920×1080. Tests verify no page-level horizontal overflow and a reachable Save action. The line table scrolls within its card, while selectors and totals reflow responsively.

## Z. Regression Status

Catalog/import, Inventory ledger/documents/counts/alerts/FEFO, Sales/POS/returns/voids, partners, and foundations remain green.

- `cargo fmt --check`: pass
- `cargo check`: pass
- `cargo test`: 375 passed, 0 failed
- `npm run lint`: pass
- `npm run build`: pass (informational bundle-size warning only)
- `npm test`: 186 passed, 0 failed
- `git diff --check`: pass

## AA. Files Changed

The Phase 5B manifest is `docs/phase-5b/files.txt`. Earlier Phase 4 and Phase 5A baseline work was preserved.

## AB. Known Limitations

- Receiving, partial receiving, received quantities, lots, expiry, batches, and `PURCHASE_RECEIPT` movements are intentionally absent.
- Migration 011 reserves receiving states while its transition guard blocks entry. Phase 5C must replace that guard in a new migration once receipt-backed transitions exist.
- Ordered cancellation is valid because Phase 5B cannot receive. Phase 5C must reject it after any receipt.
- Purchase returns and supplier accounting remain out of scope.
- Prototype source remains for historical regression coverage but no prototype native command is registered.

## AC. Real Development DB Status

The real development database was not opened, read, copied, migrated, seeded, reset, or modified. All migration and domain tests used isolated databases. The last verified real database migration remains 009.

## AD. Native Testing Status

No native application was launched. `npm run tauri dev`, manual interaction, native smoke testing, and real-app restart testing were not performed. UI verification used mocked IPC in headless browser automation.

## AE. Phase 5B Status

COMPLETE

## AF. Recommended Phase 5C Plan

Add a sequential receiving migration with immutable receipt headers/lines, ordered-versus-received quantities, and durable receipt request identities. Implement partial/multiple receipts as one atomic Rust transaction that validates remaining quantities, creates or reuses canonical batches with lot/expiry/acquisition-cost snapshots, posts `PURCHASE_RECEIPT` through the existing Inventory ledger, and derives `PARTIALLY_RECEIVED` or `RECEIVED` from posted receipts. Replace the migration 011 transition guard only through that migration, add receipt-aware cancellation, typed IPC/UI, and isolated idempotency, concurrency, persistence, and traceability tests.
