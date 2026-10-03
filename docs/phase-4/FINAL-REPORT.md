# M² Health — Phase 4 final report

**A. Phase 4 Summary**

Phase 4 is complete. The recovered application now has canonical SQLite Sales/POS with durable holds, atomic posting, persisted history/details/receipts, allocation-traced returns, refund caps and compensating void support. Phase 5 remains disabled.

**B. Migration 009**

`009_sales.sql` applied once at `2026-10-03T07:09:52.645Z`. The sequence is 001 branches, 002 catalog_core, 003 catalog_import, 004 inventory_core, 005 inventory_workflows, 006 stock_counts, 007 catalog_source_actions, 008 inventory_alerts, 009 sales. Upgrade, rollback, apply-once and preservation tests pass; 001–008 retain baseline hashes.

**C. Sales Schema**

Sales, daily sequences, immutable item snapshots, costed batch allocations, payments, durable requests, returns/items/allocations, refunds and voids/allocations use strict constraints, foreign keys and history triggers. No second inventory ledger or stock-balance table exists.

**D. POS Search**

Native SQLite English, Arabic and scientific-name search passed with real current prices and DEV3D availability. Queries are bounded and require active product/package scope.

**E. Barcode Verification**

Active exact-barcode resolution passes isolated Rust/frontend tests. The preserved real barcode belongs to an inactive product/package; native search correctly returned zero. Catalog data was not altered.

**F. Cart**

Repeat UI adds merge; backend duplicate packages are rejected. Quantity, line/sale discounts, optional manual tax, customer-name snapshot and notes persist. Backend totals/availability are authoritative; keyboard add/remove flows pass.

**G. Pricing / Discounts / Tax**

Safe integer minor units are used. Posted prices are immutable snapshots. Line discounts apply first; sale discount and optional tax use deterministic cumulative integer apportionment with exact residual handling. Returns use paid discounted totals. Tax is a manual amount, not a fiscal engine.

**H. Payments**

CASH, CARD and OTHER require exact total coverage and commit with posting. Backend split payments are tested; UI currently records one method. Native evidence is one CASH payment of 1,000 minor units.

**I. Sale Posting**

One IMMEDIATE transaction handles revision/current-total checks, FEFO, snapshots, allocations, payment, SALE movements, receipt number, status and request receipt. Native receipt `SAL-20261003-0001` created one item/payment/allocation/movement; DEV3D changed 6→5.

**J. FEFO Allocation**

Canonical Phase 3A FEFO runs inside posting. Inactive, empty and expired batches are excluded; branch isolation holds; unknown expiry is last. On October 3 the five-unit September 25 batch was excluded, leaving the one eligible unknown-expiry unit used by the sale.

**K. Batch Allocations / Cost**

Each allocation keeps batch, quantity, nullable cost and movement ID. Native batch `39722ede-8343-4ff8-988e-1e21ce20bd16` retained cost 100. Mixed/nullable-cost tests prove costs are not collapsed or invented.

**L. Canonical Stock Movements**

Only SALE −1 and SALE_RETURN +1 were added, both on DEV3D and the same batch, through the canonical protected service. All 17 earlier rows remain; final movement count is 19.

**M. Idempotency / Revisions**

POST/RETURN/VOID keys are durable and payload-bound. Exact native post/return retries returned existing results without duplicate sales, payments, allocations, refunds or movements. Draft/held revision guards reject stale edits; receipt number stayed stable.

**N. Hold / Resume**

One labeled HELD sale retained exact label, quantity 1, price 1,000, discounts and note with no item/payment/allocation/movement. It resumed after restart and was re-held; stock stayed 5. It remains HELD because no safe cancellation domain exists.

**O. Sales History**

SQLite pagination, sale/customer search, branch, status, payment and date filters are active. The posted and held sales persisted in native history with payment/refund summaries.

**P. Sale Details**

Details show snapshot prices/names, payments, allocation batch/cost/movement, returned/remaining quantities, returns, refund and void state. IDs persisted across restarts.

**Q. Receipt Preview**

Immutable receipt preview retained `SAL-20261003-0001`; evidence is `receipt-native.png`. Hardware printing is deferred.

**R. Returns**

The controlled unit returned against the original sale/item/allocation and exact batch. One SALE_RETURN restored DEV3D 5→6. Over-return, inactive scope and multi-batch rollback pass isolated tests.

**S. Refund Logic**

Native refund was exactly 1,000 minor units. Cumulative `floor(paid line total × returned quantity / sold quantity)` deltas make partial refunds deterministic, preserve full-refund totals and never exceed payment.

**T. Void Policy**

Void requires confirmation/reason and records compensation without deletion. Native void-after-return returned `A sale with posted returns cannot be voided.` Successful restoration/idempotency and rollback pass isolation; no unnecessary second real sale was created.

**U. Tauri / IPC**

Only canonical search/availability, draft/hold/resume, preview/post, list/detail, return and void handlers are registered through typed wrappers. Prototype sales/purchase handlers remain unregistered; raw movement insertion is not exposed.

**V. POS UI**

Real POS/payment/hold/history/detail/receipt/return/void-policy flows use the existing design system. Checkout remained visible at 1366×768, 1440×900 and 1920×1080 without page-level horizontal overflow. Native smoke captured zero page/console errors.

**W. Rust Tests**

`cargo fmt --check`, `cargo check`, `cargo test`: PASS. Exact total 340 passed, 0 failed: 54 canonical Phase 4 and 286 existing tests.

**X. Frontend Tests**

`npm run lint`, `npm run build`: PASS. Full Playwright: 138/138 passed, including 28 dedicated sales tests. The completed runner delayed teardown but reported no failure and left no project-owned stale process. Build has the existing chunk-size warning only.

**Y. Native Sale Smoke**

PASS: one DEV3D sale, one item/payment/allocation/SALE, 6→5, real history/detail/receipt, exact-key retry no-op. Main remained 8; DEV remained 2.

**Z. Return Smoke**

PASS: one return/refund/SALE_RETURN restored the exact batch and stock 5→6; exact-key retry was a no-op.

**AA. Hold / Resume Restart**

PASS: exact cart persisted, resumed and re-held with zero stock/payment effect.

**AB. Final Restart Verification**

PASS after two restarts: migration 009 single, sale/return/hold persisted, receipt stable, stock correct, history/Catalog/Inventory opened, no Rust panic or captured page/console error.

**AC. DB Integrity**

`integrity_check = ok`; `foreign_key_check` returned zero rows. Prior schema definitions are identical; prior ledger and alert-history rows remain.

**AD. Before / After Counts**

Schema 8→9; sales 0→2 (POSTED 1, HELD 1); items 0→1; allocations 0→1; payments 0→1; returns/items/allocations/refunds each 0→1; voids/allocations remain 0; requests 0→2; sequences 0→1; inventory movements 17→19.

Unchanged: branches 3, products 25,062, packages 25,062, prices 25,063, barcodes 1, batches 8, documents 6/lines 8/links 9, count session/line/result/request each 1, import runs 2/rows 25,061, source runs 1/rows 25,088, alert settings 3. Alerts 4→5 and history 10→12 only because the September 25 batch became expired by October 3: BATCH_EXPIRING resolved and BATCH_EXPIRED activated. Prior alert identities/history remain. Final stock: Main 8, DEV 2, DEV3D 6.

**AE. Phase 3 Preservation**

Catalog/provenance, batches, transfers, documents, counts, settings and all earlier rows passed baseline comparison. CSV SHA-256 remains `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`; config remains `c84361272efc3890978b5802cec5766ac3ae0a3110c7d6bd88affabf3c6a9c87`. Only the time-driven alert transition occurred.

**AF. Backup Path**

`D:\project\M2 health\backups\phase-4\preapply-20261003T070823Z.sqlite3`; October 3, 2026 07:08:23 UTC; 62,177,280 bytes. Integrity passed and every baseline table matched schema 8.

**AG. Files Changed**

See `files.txt`. The workspace includes earlier uncommitted phases, so Git status is not Phase 4-only. Migrations 001–008 and Catalog CSV/config were not changed.

**AH. Known Limitations**

Customer account linking, external payment/refund processing, cash change, multi-method UI, hardware printing and held-cart cancellation are deferred. Tax is manual. Successful void is isolated-test evidence. The real Catalog has no active barcode, so native evidence covers inactive exclusion while isolation covers active resolution.

**AI. Phase 4 Status**

COMPLETE

**AJ. Recommended Phase 5 Plan**

Start with a fresh baseline audit and agreed scope. For canonical purchasing, reuse inventory receipt/ledger protections, add supplier scope, immutable costs, atomic posting, durable retries and compensating reversal tests, and keep prototype migrations/handlers disabled. Complete isolation before real migration. Phase 5 has not started.
