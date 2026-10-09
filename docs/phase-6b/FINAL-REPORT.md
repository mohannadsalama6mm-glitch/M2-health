# Phase 6B Final Report

## A. Phase 6B Summary

Phase 6B adds a canonical, EGP-only Finance domain without replacing Sales, Purchasing, Inventory, Partners, or Phase 6A Reporting. It introduces migration 015, protected double-entry journals, allocation-cost COGS, compensating Return and Void entries, an atomic Expense workflow, financial reports, typed IPC, and a native Finance workspace. No historical records are backfilled.

## B. Finance / Accounting Audit

The audit confirmed that Sales persists payment methods, integer monetary snapshots, discounts, manually entered tax, refunds, voids, and immutable allocation costs. Purchase Receipts preserve acquisition costs but there is no supplier settlement or Accounts Payable workflow. Purchase Returns record physical stock compensation but no supplier cash refund. Inventory remains the sole stock ledger. The previous Finance screen was fixture-backed and its supplier/customer balances had no accounting source.

## C. Finance Semantics

The implemented contract is defined in `FINANCE-SEMANTICS.md`: EGP integer minor units, posted-only immutable journals, debit/credit lines, exact source traceability, net-of-discount revenue, allocation-cost COGS, explicit refund clearing, and limited operational profitability. Unsupported facts are deferred.

## D. Migration 015

`015_finance_core.sql` creates accounts, journal and Expense records, deterministic sequences, indexes, foreign keys, posting guards, immutability guards, source uniqueness, and 14 protected system accounts. It performs no historical backfill. The active migration chain is sequential from 001 through 015. Phase 6B did not edit migrations 001–014; the working tree's pre-existing Phase 5 migration changes were inherited from the verified baseline.

## E. Chart of Accounts

The seeded accounts cover recorded Cash, Card, and Other Sale payments; unallocated customer refund clearing; Inventory; recorded Sales tax clearing; Sales Revenue; Sales Returns; COGS; and five operating Expense categories. Codes and UUIDs are stable, account classes and normal sides are constrained, and system accounts cannot be changed or deleted.

## F. Journal Model

Journals carry a UUID, deterministic number, POSTING/POSTED lifecycle, source type and UUID, optional source parent and branch, optional reversal link, EGP totals, COGS completeness, description, and timestamps. Lines reference canonical accounts and encode one positive DEBIT or CREDIT amount.

## G. Double-Entry Invariant

Canonical posting validates positive, safe-integer lines and equal debit and credit totals. SQLite limits inserted side totals and refuses POSTED status unless at least two lines exist and both sides exactly match the declared totals. Crafted unbalanced posting is tested and rejected.

## H. Journal Immutability

Posted journal headers and all journal lines reject update and deletion. Corrections use new compensating entries. Expense records become immutable once linked to their posted journal.

## I. Source Event Traceability

Each journal has a unique `(source_type, source_id)`. Sale, Sale Return, Sale Void, and Expense source UUIDs remain navigable. Void journals retain both the Sale parent and original journal reversal link.

## J. Automatic Posting Architecture

Sales, Returns, and Voids call Finance within their existing SQLite `IMMEDIATE` transaction. The operational event, Inventory compensation, Finance journal, and retry record therefore commit or roll back together. React never sequences Finance posting after an operational write.

## K. Sales Financial Mapping

A posted Sale debits its recorded payment-method assets, credits net-of-discount Sales Revenue and recorded tax clearing, debits allocation-based COGS, and credits Inventory. Linked customers and walk-in Sales use the same source-event mapping.

## L. Discount Semantics

Canonical Sale discounts are netted once into Sales Revenue as `subtotal - discount`. No separate discount account or second subtraction exists.

## M. Tax Semantics

The canonical manually entered Sale tax is credited to a narrowly named recorded Sales tax clearing account and reversed proportionally on Return. This is event visibility, not VAT filing or statutory tax accounting.

## N. COGS

COGS is the checked integer sum of each immutable Sale allocation quantity multiplied by its allocation cost. Returns reverse original returned allocation costs. Missing cost never fabricates a value: the revenue/payment journal posts with `cogs_complete=false`, and Gross Margin and Operating Result are suppressed.

## O. Sales Return Semantics

A posted Return debits Sales Returns excluding its returned tax, debits recorded tax clearing, and credits customer refund clearing for the exact canonical refund. Physical Inventory restoration separately debits Inventory and credits COGS at original allocation cost.

## P. Refund Semantics

Finance consumes the canonical `refund_minor`; it does not derive cash from physical quantity. The settlement channel is explicitly unallocated because Sales does not retain an authoritative refund payment method.

## Q. Void / Reversal Semantics

A Void retains the original Sale journal and posts a new line-for-line opposite journal linked to it. A fully voided Sale therefore has zero net financial effect while both records remain auditable. Unknown original account codes fail safely instead of being mapped to a fallback account.

## R. Purchase Receipt Accounting Boundary

Purchase Receipts create no Finance journal in Phase 6B. Receipt does not mean supplier payment, and there is no defensible balancing AP or settlement event yet.

## S. Purchase Return Accounting Boundary

Purchase Returns also create no Finance journal. They preserve operational stock and acquisition-cost evidence without claiming a supplier refund.

## T. Expense Domain

Posted Expenses have UUID and request identity, deterministic number, optional branch, supported Expense and recorded-payment accounts, positive EGP amount, description, timestamps, and one source journal. The UI provides create, list, filtering, detail, loading, empty, error/retry, validation, and submit locking states.

## U. AP / AR Boundary

No supplier owed amount, supplier balance, customer debt, aging, credit account, or settlement workflow is exposed.

## V. Cash / Payment Boundary

Payment accounts classify recorded Sale or Expense payment methods. They are not represented as verified cash drawer, bank, reconciliation, or cash-session balances.

## W. Date Semantics

Financial posting uses the atomic source transaction's UTC timestamp. Report date boundaries are inclusive validated `YYYY-MM-DD` values over `posted_at`.

## X. Branch Semantics

Sale and branch-scoped Expense journals preserve canonical branch identity. Null means organization-wide/all branches. Purchase Orders do not acquire an inferred branch.

## Y. Trial Balance

Trial Balance aggregates only filtered POSTED lines, returns account debit, credit, and normal-side balance, and exposes exactly equal report totals. The account list uses a filtered activity CTE so out-of-range lines cannot leak through a left join.

## Z. Account Activity

Account Activity is a backend-paginated, filterable POSTED-line audit trail with journal, source, branch, description, memo, side, amount, and posting timestamp.

## AA. Sales Financial Summary

The summary exposes Sales Revenue, Sales Returns, Net Sales Revenue, customer refund clearing outflow, Net COGS, missing-cost journal count, Gross Margin, operating Expenses, and Operating Result from Finance postings only.

## AB. Expense Report

The Expense report supports inclusive dates, branch, Expense account, search, pagination, count, and total posted amount.

## AC. Gross Margin / Profitability Boundary

Gross Margin is Net Sales Revenue minus Net COGS only when all included Sale and Return journals have complete cost. Operating Result subtracts supported operating Expenses. The UI does not claim Net Profit, a Balance Sheet, Cash Flow, or statutory Profit and Loss.

## AD. Atomicity

Injected journal-line and Expense-link failures roll back Expense number allocation, Expense, journal, lines, and journal number allocation. Account validation failures also leave no partial record. Sale-family Finance hooks share their operational transaction.

## AE. Idempotency

Unique source identity prevents duplicate automatic journals. Expense request identity returns the original record for an exact retry and rejects a changed payload. Existing Sale Return and Void retry semantics remain green.

## AF. Concurrency

Two independent file-backed connections concurrently posting Expenses receive distinct Expense and journal numbers. Existing concurrent Sale and Inventory protection remains green.

## AG. Historical Data / Backfill Policy

Migration 015 creates no historical journals or opening balances. A future deployment backfill requires an explicit preview, reconciliation, idempotency, and separate authorization.

## AH. Typed Finance IPC

Ten thin Tauri commands expose accounts, Expense create/detail/list, journal detail/list, Trial Balance, Account Activity, Sales Financial Summary, and Expense Report through centralized frontend types.

## AI. Finance UI

The native capability route provides Overview, Expenses, Journals, Trial Balance, and Account Activity. It uses canonical IPC data only, links source documents, labels limitations, and contains no prototype supplier/customer balances or mock financial values.

## AJ. Rust Automated Tests

The full Rust suite passes: **440 passed, 0 failed**. The focused Finance suite contains **10 passed** tests covering migration, accounts/schema, exact cross-domain posting, idempotency, rollback, immutability, unbalanced rejection, purchase boundary, filtering/read-only behavior, missing COGS, and concurrency.

## AK. Frontend Automated Tests

The full Playwright suite passes: **268 passed, 0 failed**. Phase 6B adds **16** Finance tests covering canonical data, loading, empty, retry, filters, pagination, Expense creation/validation/detail, journal/source/reversal traceability, Trial Balance, Account Activity, scope safety, and viewports.

## AL. Automated Viewport Tests

Finance passes at 1366×768, 1440×900, and 1920×1080 with no page-level horizontal overflow. Trial Balance and the Expense workflow remain accessible.

## AM. Cross-Domain Finance Verification

The exact isolated scenario receives two units at 100 and three at 130, sells five for subtotal 1,000 with 100 total discount and 100 tax, returns two with a 400 refund and 200 cost restoration, posts and voids another Sale, and records a 150 Expense. Results are Sales Revenue 900, Sales Returns 360, Net Revenue 540, Net COGS 390, Gross Margin 150, Expenses 150, Operating Result 0, refunds 400, balanced Trial Balance, and final stock 2.

## AN. Full Regression Status

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- `cargo test`: PASS — 440 passed, 0 failed
- `npm run lint`: PASS
- `npm run build`: PASS
- `npm test`: PASS — 268 passed, 0 failed
- `git diff --check`: PASS (line-ending notices only)
- Phase 6A reporting: 10 Rust tests and 18 frontend tests remain green

## AO. Files Changed

See `CHANGED-FILES.md` for the Phase 6B manifest.

## AP. Known Limitations

Deferred areas are Accounts Payable, Accounts Receivable, supplier/customer settlement, verified cash and bank management, opening balances, manual journals, payroll, depreciation, interest, tax filing, accrual adjustments, multi-currency, statutory statements, Purchase financial posting, and historical backfill.

## AQ. Real Development DB Status

The real development database was not read, copied, seeded, migrated, or modified. Its previously verified live level remains v009; migrations 010–015 remain source/test-only.

## AR. Native Testing Status

No Tauri/native application was launched. No manual or live testing occurred. All browser validation used mocked/test IPC.

## AS. Phase 6B Status

Phase 6B: COMPLETE

## AT. Recommended Phase 6C Baseline Plan

Phase 6C should verify cross-surface Reporting and Finance consistency, integrate only defensible canonical Dashboard figures, assess export requirements, polish shared loading/error/table behavior, run final Phase 6 automated verification, and review Phase 6 scope boundaries. It should retain the current no-live-DB/no-native mode until the whole project reaches its authorized final verification stage.
