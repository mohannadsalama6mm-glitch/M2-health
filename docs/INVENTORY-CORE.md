# Inventory Core — Phase 3A

Catalog defines products, packages, identities, barcodes and price history. Inventory defines what a particular branch actually holds. Migration 004 creates no stock for imported Catalog records and adds no quantity column to Products or ProductPackages. The existing Inventory screens remain explicitly labelled Phase 1 previews; they do not post to this backend yet.

## Quantity and money

One quantity unit is one sellable ProductPackage, not an individual tablet or an inferred number of units inside a package. Quantities and batch costs are integers within JavaScript's safe integer range. Cost is in EGP minor units; 2250 means 22.50 EGP. Quantities are never floating point. Fractional dispensing, currency conversion and valuation accounting are outside this phase.

## Batches and branch ownership

Each batch has a UUID and belongs to exactly one branch and ProductPackage. Lot, expiry, receipt timestamp, acquisition cost and supplier reference may be null. Repeated lot labels are not treated as unique identities or automatically merged. Transfers create a fresh destination batch and preserve the source lot, expiry, cost and supplier reference while recording the destination receipt time.

Every movement requires a batch UUID. When lot/expiry are unknown, a real batch with those nullable fields represents the explicitly entered stock. This deliberately avoids an unbatched quantity bucket that could bypass batch-level negative-stock checks. Composite foreign keys prevent a movement from referring to another branch's/package's batch.

Batch identity and historical cost cannot be rewritten. No batch deletion or casual metadata-editing command is provided. Parent Catalog/branch deactivation does not erase balances; new posting requires an active branch, Product, ProductPackage and batch.

## Ledger and domain operations

`inventory_movements` is the source of truth. Balance is `SUM(quantity_delta)`, scoped to a branch/package or a specific batch. `inventory_levels` stores planning thresholds only. There is no mutable balance cache.

Controlled movement values: OPENING, PURCHASE_RECEIPT, SALE, SALE_RETURN, PURCHASE_RETURN, ADJUSTMENT_IN, ADJUSTMENT_OUT, DAMAGE, EXPIRY, TRANSFER_OUT, TRANSFER_IN and STOCK_COUNT_CORRECTION. Purchase/Sales/count types are reserved schema/DTO foundations; no purchase-receiving, sale-allocation or stock-count-session operation is exposed.

- Opening can atomically create a batch and its positive movement, or post once to an explicitly created batch that has no prior movements.
- Adjustment in/out requires a positive requested count, an explicit direction, branch/package/batch and a nonblank reason. Damage and expiry write-off post negative movements with a reason.
- Expiry write-off always references a batch. A known expiry must be before the current UTC date. Unknown expiry requires an explicit manual reason; the service does not invent a date. No scheduled write-off occurs.
- Every write receives a caller-generated UUID `requestId`. Reusing it rejects the duplicate posting; it does not replay a success response. Future workflows must retain the same ID during uncertain retries and inspect the ledger before issuing a new ID.
- `createdBy` is nullable unverified audit text, not a fabricated authenticated user ID. Authentication and permissions are not implemented.

Movement update/delete triggers enforce append-only history. Correct an erroneous movement by posting an opposite adjustment with an explicit reason identifying the original movement. No history is rewritten. References on transfer movements link to their common transfer record. Full document/stock-count correction workflows are deferred.

## Negative stock and atomic transfers

All posting services use immediate SQLite transactions. Both Rust validation and SQLite triggers reject a negative batch balance. SQLite also rejects negative or out-of-range branch/package totals. Another connection must wait for the current transaction before checking and posting, preventing concurrent overspending. There is no override endpoint in this phase.

Transfer uses one transaction for a destination batch, transfer header, source negative movement and destination positive movement. Deferred composite foreign keys require the exact two linked movement IDs/types to exist at commit. Movement triggers validate branch, package, batch and quantity against the transfer header. A header cannot reference unrelated historical movements, and a single transfer leg cannot commit. Any failure rolls back all writes. Transfer headers are immutable.

## Query semantics

`get_package_stock` returns zero for a valid package with no ledger. `get_batch_stock` requires explicit branch ownership. Branch inventory lists only packages deliberately introduced through batches or reorder settings, not every Catalog item. Read lists are bounded to 1–200 rows and support offsets; no React summation is required.

Low stock means quantity <= configured reorder level, including zero. Packages without explicit inventory-level configuration are not implicitly flagged. Minimum/maximum values are optional, non-negative, and minimum cannot exceed maximum. Updating thresholds preserves the level UUID and creation timestamp.

Expiry uses caller-supplied `asOf` date and a bounded day horizon. `expiredOnly=true` means expiry before `asOf`; otherwise the range includes `asOf` through `asOf + withinDays`. Only active batches with positive stock and known expiry appear. Expiry dates are treated as valid through their stated day.

FEFO returns positive, active batches whose known expiry is not before `asOf`. Known dates sort ascending before unknown dates, then receipt timestamps (unknown last), creation timestamp and UUID. The query only suggests ordering; no allocation, reservation, sales deduction or automatic write-off occurs.

Indexes support branch/package SUM, batch SUM, branch movement chronology, batch branch/package and branch/expiry. The SUM indexes preserve insertion order for equal scope keys. The ledger remains authoritative; future high-volume profiling can justify a transactionally maintained projection, but none is added now.

## Architecture and commands

Rust modules under `src-tauri/src/inventory/`: models, batches, movements, balances, levels, expiry, transfers and commands. TypeScript contracts and the native IPC wrapper are in `src/lib/tauri/inventory.ts`. No production developer screen or generic raw movement API was added.

Commands: create_inventory_batch, record_opening_stock, record_batch_opening_stock, record_adjustment, record_damage, record_expiry_write_off, transfer_inventory_stock, get_package_stock, get_batch_stock, list_branch_inventory, list_stock_movements, set_inventory_level, list_low_stock, list_expiring_batches, get_fefo_batches.

Migration 004 is appended to the existing transactional migration registry. Migrations 001–003 are unchanged. Automated tests use only in-memory SQLite or temporary database files. `scripts/verify_inventory_core.py` reads the development database, computes Catalog/branch/import row hashes and optionally makes an online SQLite backup before the controlled smoke test.

## Boundaries

No full Inventory UI, stock-count sessions, receiving documents, POS, purchases, supplier workflows, notifications, expiry automation, authentication, Supabase or sync is implemented. UUIDs, immutable movement identity, timestamps and explicit references leave room for future synchronization; conflict resolution and distributed stock policy are not claimed here. Imported Catalog entries legitimately have zero inventory until someone explicitly posts stock.
