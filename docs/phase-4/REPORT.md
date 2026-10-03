# M² Health — Phase 4 implementation notes (superseded)

The completed A–AJ report is in `FINAL-REPORT.md`.

This report is in progress until native and preservation evidence is recorded.

**A. Phase 4 Summary**

Canonical SQLite Sales/POS, durable drafts and holds, atomic posting, receipt history, exact-allocation returns and compensating voids are implemented. Existing design components and canonical inventory services are reused. Phase 5 remains disabled.

**B. Migration 009**

`009_sales.sql` is the only new migration. The active sequence is 001 branches, 002 catalog_core, 003 catalog_import, 004 inventory_core, 005 inventory_workflows, 006 stock_counts, 007 catalog_source_actions, 008 inventory_alerts, 009 sales. Isolated upgrade, apply-once, rollback and preservation tests pass. Migrations 001–008 are checked against their baseline hashes.

**C. Sales Schema**

Sales, daily receipt sequences, immutable items/snapshots, allocation-specific costs, payments, durable request receipts, returns/items/allocations/refunds and voids/allocations. Foreign keys and triggers enforce retention and posting relationships. No second inventory ledger or stock-balance table is introduced.

**D. POS Search**

Bounded SQLite search covers English, Arabic and scientific names, active product/package scope and branch-specific eligible quantity. No fixture stock is used by the native sales route.

**E. Barcode Search**

Exact package barcode matching is prioritized; active-barcode behavior passes isolated Rust and frontend tests. The preserved real Catalog's sole barcode belongs to an inactive product and package, so it must remain excluded rather than being reactivated for testing.

**F. Cart**

Duplicate UI additions merge quantities; duplicate backend package lines are rejected. Quantity, line discount, sale discount, manual tax, notes and optional customer name persist in saved carts. Backend totals and stock validation control posting. Enter adds search results and Delete removes a focused cart line.

**G. Pricing / Discounts / Tax**

Safe integer minor units throughout. Prices become explicit saved values. Line discounts precede a deterministic proportional sale discount; persisted allocation accounts for every minor unit. Optional manual tax is allocated after discount. See POLICY.md for the exact cumulative-division formula and scope.

**H. Payments**

CASH, CARD and OTHER records require exact total coverage and commit with the sale. Backend supports split payments; the current UI records one method. Cash tender/change and external payment-provider processing are deferred.

**I. Sale Posting**

One IMMEDIATE transaction validates revision and stock, allocates canonical FEFO, creates item snapshots and allocation costs, inserts canonical SALE movements and payments, assigns the receipt number and stores the successful request. Failure rolls back the entire operation.

**J. FEFO Allocation**

The canonical Phase 3A query is used inside posting. Expired, inactive and empty batches are excluded; unknown expiry is last; branch isolation and insufficient-coverage rejection remain enforced.

**K. Batch Allocations / Costs**

Each allocation retains batch, quantity, cost and movement ID. Mixed costs are never collapsed to the first batch's cost; nullable costs remain unknown. Details expose allocation-level evidence.

**L. Canonical Stock Movements**

Posting creates negative SALE movements through the existing protected insertion service. Returns and voids create compensating SALE_RETURN movements with durable domain references. Existing immutable ledger rows are never edited or deleted.

**M. Idempotency / Revisions**

Durable payload-bound request keys make identical post/return/void retries safe. Changed payloads and repeat completion under new keys are rejected. Saved-cart revisions prevent stale updates. The payment UI freezes the original payload/key during retries and provides explicit saved-cart reload after failure.

**N. Hold / Resume**

Held carts persist exact package quantities, prices, discounts and notes without stock reservation or payments. Resume requires the correct revision and branch. A held cart remains held after verification because no safe cancellation domain is implemented.

**O. Sales History**

Real SQLite pagination with sale/customer search, branch selection, status, payment method and date range filters. Rows display persisted payment/refund summaries and link to details.

**P. Sale Details**

Stored names, prices, quantities, discounts, taxes, allocations, costs, batch/movement IDs, payments, returns, refunds and void state. Sold, returned and remaining returnable quantities come from backend history.

**Q. Receipt Preview**

Historical sale snapshots and recorded totals/payment are shown in the existing dialog design. Preview does not depend on current Catalog price. Hardware printing is deferred.

**R. Returns**

The UI shows sold, already returned and remaining quantities, requires a reason, previews the backend refund and posts a durable return. Original allocations are restored in recorded order. Over-returns and inactive original scope are rejected atomically.

**S. Refund Logic**

Each return refunds the change in cumulative floor(paid line total × returned quantity / sold quantity). This uses actual discounted amounts, preserves exact full-refund totals and caps refunds at paid value. Multi-item discount and multi-batch failure tests pass.

**T. Void / Reversal**

Explicit confirmation and reason are required. Void records compensation and a refund while preserving original sale/payment history. Posted returns block void. Same-key retries are safe; repeated void with a new key is rejected. Verified with isolated tests, without an additional real database void.

**U. Tauri / IPC**

Only canonical sales handlers are registered. Typed wrappers cover search/availability, draft save, hold/resume, preview/post, list/details, return preview/create/post and void. `save_sale_draft` consolidates create/update with ID and expected revision; canonical list/get/void names avoid prototype collisions. No raw stock insertion command is exposed.

**V. POS UI**

Existing cards, inputs, tables, alerts and dialogs are reused. Search/cart scrolling keeps checkout visible at 1366×768, 1440×900 and 1920×1080. Native operational routes use SQLite capabilities rather than the prototype sales screen.

**W. Rust Tests**

340 passed, including 54 canonical Phase 4 tests and all 286 existing tests. Coverage includes migration/rollback/preservation, names/barcode/availability, durable holds, revisions, concurrent overspend prevention, FEFO, mixed costs, payment rollback, discount allocation, exact refunds, return/void rollback, retries and file restart persistence. Existing prototype tests remain isolated and do not enable their runtime features.

**X. Frontend Tests**

28 Phase 4 tests passed independently. The completed full regression is 138/138; see `FINAL-REPORT.md` for all native and preservation evidence.

**Y–AD. Native and preservation evidence**

Pending completion of all automated checks, verified preapply backup, controlled native sale/hold/restart/return/retry/restart, and final read-only comparison.

**AE. Files Changed**

See files.txt for the Phase 4-only file list. The workspace also contains earlier uncommitted phases; these are not attributed to Phase 4.

**AF. Known Limitations**

Customer account links, external payment/refund processing, cash change, multi-method UI, hardware printing and held-cart cancellation are deferred. Tax is an optional manual amount. Production build has a bundle-size warning. Prototype purchases and later workflows remain disabled.

**AG. Phase 4 Status**

COMPLETE — superseded by the A–AJ decision in `FINAL-REPORT.md`.

**AH. Recommended Phase 5 Plan**

Review the recovered baseline and agree the next phase's scope before implementation. For purchases, design a new canonical domain using the existing inventory receipt/ledger protections, then cover atomic posting, cost snapshots, retries, reversals and supplier scope in isolated tests before any native migration. Do not enable the old prototype migration sequence. Phase 5 has not been started.
