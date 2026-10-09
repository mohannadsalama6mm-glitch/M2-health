# Phase 6A Final Report

## A. Phase 6A Summary
Phase 6A adds one read-only canonical Reporting domain over Sales, Inventory, Purchasing, Customers, and Suppliers. Rust owns formulas, validation, aggregation, and pagination. The native Reports route uses typed IPC and canonical detail routes. No Finance behavior was introduced.

## B. Reporting Audit
The former `Reports.tsx` is an operational mock using fixture rows, decorative values, and simulated export/print actions. It remains only for non-native preview and is bypassed by the native capability boundary. The Dashboard is also a broad demo surface; replacing only some cards would mix canonical and illustrative values, so it remains isolated. Canonical Inventory ledger queries, alert thresholds, sale-time snapshots, purchasing documents, validators, IPC conventions, and design-system tables were reused.

## C. Reporting Architecture
The path is `CanonicalReports` → centralized typed `reporting.ts` → twelve thin Tauri commands → Rust Reporting services → canonical SQLite tables. Commands expose no mutation. React receives backend aggregates and bounded pages and contains no SQL or authoritative formulas.

## D. Migration / Schema Decision
No migration 015 was needed. Existing indexes cover the representative filters and relationships. No cache, report table, summary table, or duplicate ledger was added. Phase 6A did not edit migrations 001–014. The real development DB remains verified only through 009.

## E. Date & Time Semantics
Canonical timestamps remain UTC ISO timestamps. `date_from` and `date_to` are validated `YYYY-MM-DD` values and both boundaries are inclusive against the stored UTC day. A single-day range includes that complete stored UTC day. Display formatting is not used by SQL.

## F. Branch Semantics
Sales, receipts, returns, movements, batches, stock, expiry, and low-stock reports use their own canonical `branch_id`. Null means all branches. Purchase Orders have no branch owner, so order counts are organization-wide; branch filters apply to receipts and returns.

## G. Reporting Overview
The overview reports posted Sale count, gross posted Sales, discounts, taxes, linked refunds, net posted Sales, net sold units, ledger-derived stock, canonical low-stock thresholds, Purchase Order count, received quantity, and Purchase Return quantity. Labels describe operational values rather than accounting revenue or profit.

## H. Sales Report
Sales detail supports inclusive dates, branch, POSTED/VOIDED status, explicit customer UUID, search, deterministic newest-first order, and backend pagination. Its summary exposes posted count, gross, discount, tax, refunds, net amount, sold quantity, returned quantity, and net units.

## I. Void / Return / Refund Semantics
Gross is the sum of `sales.total_minor` for POSTED Sales. VOIDED Sales remain visible in detail with zero operational net and are excluded from totals. Returned quantity uses POSTED Sale Returns. Refund amount uses canonical linked refunds. Net posted Sales equals gross posted Sales minus refunds; returned quantities are not subtracted again from money.

## J. Sales by Product / Package
Product analytics group POSTED Sale items by package UUID, use sale-time product/package and money snapshots, and subtract POSTED returned quantities and refund snapshots once. Current Catalog prices never reconstruct history.

## K. Customer Sales Analytics
Only Sales with explicit canonical `customer_id` are included; walk-in Sales are separate. Amounts come from historical Sales/refunds. Current customer identity supports navigation, while edits cannot change historical Sale amounts.

## L. Inventory Stock Report
Quantity is summed from the immutable ledger by branch/package. Rows include canonical identity, branch, quantity, reorder point, state, acquisition-cost valuation, and units lacking batch cost.

## M. Inventory Acquisition-Cost Valuation
Valuation sums each positive current batch quantity × canonical batch acquisition cost. Selling price is never used. Rust uses checked `i128` accumulation and rejects results outside the safe integer range. Positive stock without cost is disclosed as `unvaluedQuantity`.

## N. Inventory Movement Report
The report reads the canonical ledger and exposes timestamp, branch, product/package, batch/lot, movement type, signed quantity, reason, and source reference. It supports backend filters and pagination.

## O. Expiry Report
Expiry derives current quantity per canonical batch and excludes zero-stock batches. It distinguishes EXPIRED, EXPIRING, VALID, and NO_EXPIRY with a validated 0–36,500-day window.

## P. Low-Stock / Reorder Report
The report reuses `inventory_levels.reorder_level` and ledger quantity. OUT_OF_STOCK is quantity ≤ 0; LOW_STOCK is positive quantity ≤ reorder level. Packages without a threshold are omitted rather than assigned one.

## Q. Purchasing Report
Purchase reporting supports date, supplier, status, search, and backend pagination. It exposes ordered quantity/value, received/returned quantities, and receipt/return counts through isolated subqueries. Ordered value is not described as paid expense.

## R. Purchase Receipt Report
Posted receipt rows preserve receipt, Purchase Order, Supplier, branch, quantity, timestamp, and acquisition-cost value traceability using receipt-line cost snapshots.

## S. Purchase Return Report
Posted return rows preserve return, source Purchase/receipt, Supplier, branch, quantity, reason, timestamp, and acquisition-cost value. No supplier cash refund is implied.

## T. Supplier Analytics
Supplier analytics provide order count/value, received quantity/value, returned quantity/value, and latest Purchase date. No balance, payable, paid, settlement, or aging metric exists.

## U. Aggregate / Double-Count Protection
One-to-many totals use correlated subqueries or independently grouped sets. The cross-domain fixture verifies exact Sales, refund, returned-unit, stock, valuation, receipt, Purchase Return, and Supplier totals and detects child-join multiplication.

## V. Money Arithmetic
Money remains integer minor units. Historical snapshots are used. Inventory valuation uses checked wide-integer multiplication; React performs formatting only.

## W. Read-Only Guarantees
Only query commands exist. Tests snapshot operational row counts before and after representative reports. No movement, revision, sequence, insert, update, or delete path is called.

## X. Performance / Index Decisions
Filtering, grouping, ordering, and pagination run in SQLite. React receives only the requested page. Existing indexes were sufficient; no speculative cache or materialized report was added.

## Y. Typed IPC
`reporting.ts` defines shared filters, overview, page, and domain row types with one named function per command. Rust denies unknown filter fields and validates dates, UUIDs, branch existence, search length, expiry window, statuses, limits, and offsets. Commands only delegate and return structured errors.

## Z. Reporting UI
The native workspace provides Overview, Sales, Inventory, and Purchasing groups; explicit Apply/Reset; inclusive dates; branch, status, and search filters; loading, empty, retry, and no-results states; summary cards; scrollable tables; canonical links; and backend pagination.

## AA. Dashboard Integration
The Dashboard remains a mixed demo surface and was not partially converted, avoiding a misleading blend of canonical and illustrative cards. Its report actions route into canonical Reports. Complete Dashboard conversion remains integration/polish work.

## AB. Rust Automated Tests
Ten new isolated tests cover reconciliation, double-counting, read-only behavior, empty data, exact values, validation, inclusive dates, pagination, movement types, walk-in separation, Returns/refunds, and voids. Full Rust: 430 passed, 0 failed.

## AC. Frontend Automated Tests
Eighteen mocked-IPC tests cover overview values, all eleven detail reports, loading, empty, retry, Apply/Reset, filter forwarding, pagination, and detail navigation. Full frontend: 252 passed, 0 failed.

## AD. Automated Viewport Tests
The canonical stock report passes page-overflow and control-reachability checks at 1366×768, 1440×900, and 1920×1080. Wide tables scroll internally.

## AE. Cross-Domain Reporting Verification
The isolated flow posts a Supplier Purchase/receipt, Purchase Return, linked Customer Sale, partial Sale Return/refund, and Inventory movements. Exact results: gross 400, refund 200, net 200, net units 1, stock 8, valuation 800, received 10, Purchase Return 1, receipt value 1000, return value 100 (minor units for money).

## AF. Full Regression Status
`cargo fmt --check`, `cargo check`, `cargo test`, `npm run lint`, `npm run build`, `npm test`, and `git diff --check` pass. Rust: 430 passed. Frontend: 252 passed. The Vite chunk-size advisory is informational.

## AG. Files Changed
See `CHANGED-FILES.md`.

## AH. Known Limitations
CSV/PDF export, charts, saved presets, local-calendar timezone conversion, and a canonical Dashboard are deferred. Receipt/return reports summarize documents; linked canonical screens retain line-level lot/expiry detail. Packages without reorder settings are absent from low-stock results; units without acquisition cost are disclosed.

## AI. Deferred Finance Scope
Journals, accounts, expenses, AP/AR, balances, supplier/customer payments, settlement, cashbook, banks, profit/loss, balance sheet, tax accounting, and accounting valuation remain outside Phase 6A.

## AJ. Real Development DB Status
The real DB was not opened, read, copied, migrated, seeded, or modified. Its previous verified level remains 009. Backend tests used in-memory databases.

## AK. Native Testing Status
No native app, `tauri dev`, manual smoke test, or live UI was started. Validation used Rust tests, TypeScript compilation, and Playwright test IPC.

## AL. Phase 6A Status
Phase 6A: COMPLETE

## AM. Recommended Phase 6B Baseline Plan
Define accounting boundaries and source-event mappings for Sales, refunds, purchases, receipts, and returns before schema work. Design a sequential migration only after journal, accounts, posting invariants, idempotency, reversal, currency, opening-balance, AP/AR, and report semantics are agreed. Keep Finance traceable to canonical documents without replacing the Inventory ledger or rewriting history, and continue isolated-DB testing.
