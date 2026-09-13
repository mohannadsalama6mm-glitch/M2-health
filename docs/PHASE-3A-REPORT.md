# Phase 3A — Inventory Core foundation

## A. Phase 3A Summary

COMPLETE. Real branch/package/batch-owned Inventory now has persistent batches, an immutable ledger, opening stock, adjustments, damage/expiry foundations, atomic transfers, derived balances, reorder settings, low-stock/expiry queries and FEFO ordering. The Inventory UI is unchanged and remains a demo. No later subphase was started.

## B. Migration 004

`src-tauri/migrations/004_inventory_core.sql` is version 4, `inventory_core`, using the existing transactional runner. Migrations 001–003 were not edited. Upgrade and failed-upgrade tests preserve Catalog/branch/import records. The real database contains versions 1–4 exactly once.

## C. Inventory Schema

Four STRICT tables: inventory_batches, inventory_movements, inventory_transfers, inventory_levels. UUID identities, foreign keys and timestamps. Products and ProductPackages were not changed; no quantity field or stock cache was added.

## D. Batch Model

Each batch belongs to one branch and ProductPackage. Lot, expiry, receipt timestamp, cost and supplier reference are nullable. Repeated lot labels are allowed; no automatic grouping. Every movement requires a batch, including an explicitly created batch with unknown lot/expiry. Batch identity and historical cost are immutable.

## E. Stock Ledger Model

Signed, nonzero integer deltas with branch, package, batch, controlled type, reason, optional actor text and transfer references. Update/delete triggers prevent rewriting. Corrections use opposite adjustments with reasons identifying the earlier movement. Caller-generated UUID request IDs prevent duplicate postings; repeated requests return an error rather than replaying success.

## F. Movement Types

OPENING, PURCHASE_RECEIPT, SALE, SALE_RETURN, PURCHASE_RETURN, ADJUSTMENT_IN, ADJUSTMENT_OUT, DAMAGE, EXPIRY, TRANSFER_OUT, TRANSFER_IN, STOCK_COUNT_CORRECTION. Types/signs are constrained. Sales, purchase and stock-count-session operations are not exposed; their values are foundations only.

## G. Stock Balance Calculation

Rust/SQLite calculates SUM(quantity_delta) per branch/package or batch. A valid package with no movements returns zero. Branch inventory lists only packages introduced by batches or level configuration. Queries are paginated to at most 200 rows. React does not calculate operational stock.

## H. Negative Stock Policy

Strict no-negative batch and package balances. Immediate transactions serialize posting; Rust checks batch availability and SQLite triggers enforce lower and safe-integer upper bounds. Separate-connection tests prove competing deductions cannot overspend stock. No override endpoint exists.

## I. Opening Stock

Opening atomically creates a batch and positive movement. Another explicit command supports opening an existing empty batch. Opening is allowed only before any movement on that batch, once. Catalog import creates no stock.

## J. Adjustments / Damage / Expiry Foundations

Adjustment in/out, damage and expiry require a positive whole-package request and nonblank reason. Outgoing operations post negative deltas. Known expiry must be before the current UTC date; unknown expiry can be manually written off with a reason. There is no inferred date, automatic write-off or batch deletion.

## K. Transfer Foundation

One transaction creates the destination batch, common reference, outgoing and incoming movements. Deferred composite foreign keys require the exact paired movement IDs/types at commit. Triggers validate branch/package/batch/quantity against the header. Incomplete headers and lone transfer legs cannot commit. A forced second-leg failure rolls back everything. Transfers were exercised only in isolated tests; no development transfer/second branch was created.

## L. Inventory Levels / Reorder

One planning record per branch/package. Non-negative integer reorder/minimum/maximum; minimum cannot exceed maximum. Upserts retain identity and creation timestamp. No real development level was seeded.

## M. Low Stock Query

Derived quantity <= configured reorder level, including zero and equality. Unconfigured packages receive no guessed threshold. No notification automation.

## N. Expiry Query

Validated as-of date and bounded day horizon. Expired means before as-of; upcoming includes as-of through as-of + N days. Only active positive-stock batches with known expiry appear. Unknown dates are not guessed.

## O. FEFO Strategy

Positive-stock active batches with known unexpired dates first, ordered by expiry, receipt timestamp, creation timestamp and UUID. Unknown expiry/receipt dates sort last. Tests cover nulls, expired exclusions and receipt-time ties. No allocation or sale deduction is performed.

## P. Cost / Quantity Storage

Quantity means sellable ProductPackages, not individual tablets or inferred contents. Counts and EGP minor-unit costs are integers within 9,007,199,254,740,991. Fractional quantities and negative costs are rejected. Historical cost is immutable. The real smoke batch has null cost and expiry to avoid invented values.

## Q. Repositories / Services

`src-tauri/src/inventory/`: models, batches, movements, balances, levels, expiry, transfers, commands and mod. SQL, validation and transaction control stay in Rust.

## R. Tauri Commands / DTOs

Fifteen commands: create_inventory_batch, record_opening_stock, record_batch_opening_stock, record_adjustment, record_damage, record_expiry_write_off, transfer_inventory_stock, get_package_stock, get_batch_stock, list_branch_inventory, list_stock_movements, set_inventory_level, list_low_stock, list_expiring_batches, get_fefo_batches.

CamelCase DTOs and TypeScript wrappers are in `src/lib/tauri/inventory.ts`. No generic raw-movement API or production verification screen. createdBy is nullable unverified text, not a fabricated authenticated identity.

## S. Database Constraints / Indexes

Composite batch/scope FKs, enum/sign/nonzero/integer/range checks, date validation, immutable ledger/batch identity/transfers, one opening per batch, one level per branch/package and deferred transfer-pair integrity. Indexes support package/batch SUM, branch chronology and branch expiry. Query-plan tests confirm index use. No balance cache.

## T. Rust Tests

25 Inventory tests cover zero stock, migration preservation/rollback, nullable/repeated lots/cost, opening/multiple batches, adjustments/compensation/damage/expiry, isolation, no-negative stock, immutability, duplicate requests, validation/ranges, levels/low stock, expiry/FEFO/ties/nulls, atomic/incomplete/failed transfers, FKs, indexed/paginated queries, inactive scopes and concurrent deductions. All automated tests use isolated databases.

## U. Development DB Smoke Test

Pre-migration online SQLite backup:
`C:/Users/Mohannad/AppData/Local/com.m2health.pharmacy/backups/before-phase-3a-61d2f58a-7ed5-471d-b5af-57ca614aae9c.sqlite3` (50,528,256 bytes). Integrity verified; no restore.

- MAIN: `f319a917-95d0-47bf-b007-ab76b7c044ae`.
- Product: `1 2 3 (ONE TWO THREE) 20 F.C.TABS.`.
- Imported package: `40bad73e-196c-4a7e-81e7-b6dfb90e64af` (Default Package).
- Batch: `5f5c70bd-40c9-461f-9345-326bc1feeda3`.
- Lot: `PHASE3A-DEV-SMOKE-20260913`.
- Opening: `99ab7852-3348-40dc-8f91-42cc280abdae`, +10.
- Adjustment: `d9bfe09a-9841-4c3b-9631-ea38d05a3d81`, -2.

Initially package stock was zero and branch inventory empty. Both package and batch read 10 after opening and 8 after adjustment. Reasons explicitly identify development verification, not actual pharmacy stock. Exactly one batch and two movements remain; no transfers/levels or other stock were created. This intentional history was retained without deletion.

## V. Restart Verification

Closed and restarted using npm run tauri dev. Version 004 remained once. The batch and both movements persisted; package/batch balances remained 8. FEFO returned the one unknown-expiry batch; expiry and unconfigured low-stock queries returned empty. MAIN and Catalog queries worked; console/page errors: zero.

All Catalog/branch/import row hashes match the baseline. Counts remain products/packages 25,062 each; manufacturers 5,545; categories 2,446; routes 14; ingredients 1; ingredient links 1; barcodes 1; prices 25,063; import runs 1; provenance rows 25,061. Integrity: ok; foreign-key errors: none. Source SHA-256 remains `35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c`. Migrations 001–003 and Inventory UI hashes are unchanged.

## W. Automated Checks

Passed npm lint, build and 32 Playwright tests; cargo fmt --check, cargo check and 71 Rust tests (22 Catalog, 12 Import, 12 Foundation, 25 Inventory). The existing Vite advisory about a JavaScript chunk over 500 kB remains; build succeeds. Native startup, real IPC smoke operations, demo UI labelling, Catalog queries and restart were separately verified.

## X. Files Added / Changed

Added migration 004, nine Inventory Rust modules, Inventory tests, typed TypeScript boundary, read-only audit/backup script, policy/report documents and phase-3a JSON evidence. Updated module/handler registration, migration registry, schema-version test expectations, domain-neutral constraint-error wording and README. Inventory UI, migrations 001–003 and CSV are unchanged.

## Y. Known Limitations

Inventory UI remains a demo and will not show this backend balance until later integration. The retained 8 packages are development test stock. No auth/override, fractional dispensing, valuation engine, document reversal workflow, stock-count sessions, sales allocation, purchase receiving, automated expiry/notifications or sync. Level configuration has timestamps but no separate revision ledger. Indexed SUM may warrant measured optimization at larger movement volumes.

## Z. Phase 3A Status

COMPLETE

## AA. Recommended Phase 3B Plan

Receiving / Adjustments / Transfers as complete real workflows: explicit document/draft identity, validation, review/posting states, durable request IDs, references and correction behavior over these services. Preserve the ledger and stock semantics. Do not infer opening stock from Catalog. Full broad Inventory UI integration remains separately scoped. Phase 3B was not implemented.
