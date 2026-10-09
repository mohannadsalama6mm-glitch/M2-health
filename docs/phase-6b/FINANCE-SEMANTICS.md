# Phase 6B Finance Semantics

## Scope and evidence
Finance records explicit financial effects of canonical operational events. Sales, payments, refunds, allocations, Inventory movements, Purchase documents, Customers, and Suppliers remain operational sources of truth. The existing Finance screen is an obsolete fixture-backed shell and is not an accounting source.

The implemented scope is double-entry Sales, Sales Return/refund, Sale Void reversal, operating Expenses, Trial Balance, Account Activity, Sales Financial Summary, and Expense reporting. It does not implement supplier payments, customer credit, AP, AR, payroll, banking, tax filing, statutory statements, or historical backfill.

## Currency and monetary unit
The base and only supported currency is EGP. Every stored amount is a non-negative integer minor-unit value. Floating-point currency and foreign exchange are unsupported.

## Accounts and normal balances
Accounts use stable UUIDs and deterministic codes. Types are `ASSET`, `LIABILITY`, `EQUITY`, `REVENUE`, or `EXPENSE`. ASSET and EXPENSE accounts have a DEBIT normal balance; LIABILITY, EQUITY, and REVENUE accounts have a CREDIT normal balance.

System accounts are protected from deletion and deactivation. Phase 6B seeds only accounts required by implemented mappings:

- recorded Cash, Card, and Other Sale payment assets;
- unallocated Customer Refund Clearing asset;
- Inventory asset;
- Recorded Sales Tax Clearing liability;
- Sales Revenue;
- Sales Returns contra-revenue (typed REVENUE but normally used by debits);
- Cost of Goods Sold;
- Rent, Utilities, Transport, Maintenance, and Miscellaneous operating Expenses.

Payment asset accounts record canonical payment-method events. They are not bank reconciliation, a cash session, or a complete cashbook.

## Journal invariant and posting
Only canonical services can create journals. A journal begins as `POSTING`, receives at least two valid lines, and can become `POSTED` only when its positive declared debit and credit totals match the actual line sums. Each line has exactly one side and a positive integer amount. Posted journals and lines are immutable and retained.

No editable Finance drafts are implemented. Corrections use a new compensating journal. Journal and expense numbers use transactionally incremented daily sequences.

## Source identity, idempotency, and reversals
Automatic journals use a durable `(source_type, source_id)` identity based on canonical UUIDs, never display numbers. The pair is unique. Expense request IDs and expense UUIDs are also unique. Exact Expense retries return the original result; the same request ID with a changed canonical payload is rejected.

Sale posting, Return posting, and Void posting call Finance inside the same SQLite `IMMEDIATE` transaction as their operational event. Either the operational event and journal both commit, or neither commits. React never sequences operational and Finance posting.

A reversal retains the original journal and stores `reversal_of_journal_id`. It copies every original line with DEBIT/CREDIT swapped. Original and reversal therefore net to zero while both remain auditable.

## Sale mapping
A POSTED Sale creates one journal with two independent effects.

Payment/revenue effect:

- DEBIT the recorded payment-method asset for each canonical `sale_payments` amount;
- CREDIT Sales Revenue for `subtotal_minor - discount_minor`;
- CREDIT Recorded Sales Tax Clearing for `tax_minor`, when non-zero.

The journal balances because canonical payments equal `total_minor`, and `total_minor = subtotal - discount + tax`.

Discounts are netted once into Sales Revenue. There is no separate discount account.

Tax is the canonical manually recorded Sale tax amount. It is isolated in a clearing liability for visibility, but Phase 6B makes no VAT rate, filing, recoverability, jurisdiction, or settlement claim.

COGS effect, only when every allocation has a canonical cost:

- DEBIT Cost of Goods Sold;
- CREDIT Inventory;
- amount = sum of each immutable Sale allocation quantity × its allocation `unit_cost_minor`.

If any allocation cost is absent, the Sale revenue/payment effect still posts, COGS lines are omitted, and the journal is marked `cogs_complete = 0`. Financial summaries suppress Gross Margin when any included Sale/Return lacks complete COGS. Current Catalog/package cost is never substituted.

Linked and walk-in Sales use the same mapping. Customer identity does not change accounting.

## Sales Return and refund mapping
A POSTED physical Return and its canonical refund are one source event but remain distinct components.

Refund effect:

- DEBIT Sales Returns for the canonical refund excluding the proportionate returned tax;
- DEBIT Recorded Sales Tax Clearing for the proportionate returned tax, when non-zero;
- CREDIT Unallocated Customer Refund Clearing for the exact canonical `sale_refunds.amount_minor`.

The original payment method for an outgoing refund is not persisted, so Finance does not guess Cash/Card/Other. The clearing credit records the money-return event without claiming its settlement channel.

Returned tax uses the same cumulative integer apportionment principle as Sales Returns: each returned portion receives its exact successive share of the immutable Sale-item tax snapshot. The portions sum exactly across a full return.

Inventory/COGS restoration, only when every returned original allocation has cost:

- DEBIT Inventory;
- CREDIT Cost of Goods Sold;
- amount = returned allocation quantity × the original allocation cost.

Physical quantity never determines refund money. The canonical refund snapshot determines money; returned original allocations determine cost.

## Sale Void mapping
A void is allowed only for a posted Sale without posted Returns. Finance creates a new journal that reverses every line of the original Sale journal exactly. The original remains immutable. The reversal references both the void source UUID and original journal. The pair has zero net effect, including payment, revenue, tax, Inventory, and COGS.

## Purchase Receipt and Purchase Return boundary
No Purchase Receipt or Purchase Return journals are created in Phase 6B. A receipt proves acquisition and Inventory movement but does not prove supplier payment, a payable, payment terms, or settlement. A Purchase Return does not prove supplier cash refund or credit-note settlement. Adding an artificial clearing account here would fabricate an AP boundary that the product cannot currently reconcile.

Phase 6A operational received/returned quantities and acquisition-cost analytics remain authoritative. Future AP work may add an explicit, reconciled purchase-accounting workflow and opt-in backfill.

## Expenses
An Expense is a posted, immutable Finance document with UUID, deterministic number, posting date, optional canonical branch, EGP amount, Expense account, payment asset account, description, request identity, timestamps, and source journal.

Atomic posting is:

validate → allocate number → insert Expense → create balanced journal (DEBIT Expense, CREDIT payment asset) → link journal → commit.

Failure rolls back the sequence allocation, Expense, journal, and lines. Payroll is excluded.

## AP, AR, cash, and opening balances
AP and AR are not implemented. Finance exposes no supplier owed amount, supplier balance, customer debt, aging, or settlement. Recorded payment-method assets are event classifications, not verified drawer/bank balances.

No opening balance is inferred from operational or development data. Production onboarding will require an explicit, separately reviewed opening-balance or historical reconciliation procedure.

## Dates and branches
Journal `posted_at` is the UTC posting timestamp from the atomic source transaction. Reports accept inclusive validated UTC `YYYY-MM-DD` boundaries. Display formatting does not drive queries.

Sales and Expenses preserve canonical branch ownership. Null branch filtering means all branches. Purchase Orders have no inferred branch. No Purchase accounting journal exists in this phase.

## Financial reports
Trial Balance aggregates POSTED journal lines only and must have equal total debits and credits. Account Activity is a paginated posted-line audit trail with source UUIDs. Expense reporting uses posted Expense documents.

Sales Financial Summary derives only from Finance journals:

- Sales Revenue = credits minus debits on Sales Revenue;
- Sales Returns = debits minus credits on Sales Returns;
- Net Sales Revenue = Sales Revenue minus Sales Returns;
- Net COGS = debits minus credits on Cost of Goods Sold;
- Gross Margin = Net Sales Revenue minus Net COGS only when COGS is complete;
- Refund amount = credits minus debits on Customer Refund Clearing;
- Operating Expenses = debit-normal activity on Phase 6B Expense accounts;
- Operating Result = Gross Margin minus Operating Expenses only when Gross Margin is available.

These are limited operational financial summaries, not statutory Net Profit, Profit & Loss, Balance Sheet, or Cash Flow statements.

## Historical data and backfill
Migration 015 seeds accounts and creates no journals for historical Sales, Returns, Voids, receipts, returns, or Inventory. Only source events posted after Finance-aware code is active generate journals. Any future production backfill must be explicit, previewable, reconciled, idempotent, and separately authorized.
