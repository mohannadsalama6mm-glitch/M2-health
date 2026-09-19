# Phase 4 sales policy

The active sales domain is `src-tauri/src/sales`. Migration 009 extends the recovered schema; migrations 001–008 and the Catalog source are preserved. Prototype sales/purchases migrations and commands remain outside the active registry.

## Money and costs

All persisted money uses safe integer minor units. The UI accepts EGP decimals and converts them exactly. Line discounts apply first. A sale discount is apportioned over net line amounts in stable cart order using cumulative integer division; allocated amounts sum exactly to the entered discount. Optional manual tax is apportioned over discounted amounts the same way. This is a manual amount facility, not a tax-rate or fiscal compliance engine.

Each allocation stores its own batch cost, including unknown costs. A line cost is populated only when every allocation has the same cost. Posted names, packages and selling prices are immutable snapshots.

## Posting and concurrency

Draft and held carts persist without reserving stock or recording payments. Updates require the expected revision. Posting revalidates current stock and canonical FEFO inside one SQLite IMMEDIATE transaction. Items, allocations, payments, canonical SALE movements, receipt numbering, status and request receipt commit together. Payments must cover the exact total.

FEFO excludes inactive, empty and expired batches, keeps branches separate, and orders unknown expiry last. Every stock change uses the existing canonical movement service and its no-negative-stock checks. No second stock ledger is introduced.

Durable POST, RETURN and VOID request keys are payload-bound. An identical retry returns the existing result. A changed payload or a new key for an already completed operation is rejected.

## Returns and voids

Returns consume original allocations in their recorded order, subtracting earlier returns. They restore the exact original batch through canonical SALE_RETURN movements. Inactive scope or inactive original batches reject the operation atomically. Returning to an expired original batch does not make it eligible for sale.

Refunds use the original discounted, tax-inclusive line total. For cumulative returned quantity r out of sold quantity q, the cumulative refundable amount is floor(line total × r / q). Each return refunds the difference from prior returns. This preserves every minor unit on full return and prevents refunds above the amount paid.

Void is an administrative compensation, requires a reason and explicit UI confirmation, and preserves the original sale and payment. Posted returns block void. A successful void restores original allocations and records a full refund without deleting history.

## Scope

Customer name is optional; customer accounts and linked customer IDs are deferred. Payment methods are local records only, with no payment-provider charge or refund transmission. Receipt preview uses stored data; printer integration is deferred. Held-cart cancellation is not implemented, so controlled held carts remain durably held. Phase 5 has not been started.
