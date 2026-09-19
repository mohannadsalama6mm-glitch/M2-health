# Phase 3E Inventory query and integration policy

The immutable `inventory_movements` ledger remains authoritative. Phase 3E adds read-only projections, not stock posting rules or schema.

## Relevant packages and zero stock

For the selected branch the overview includes the union of package IDs referenced by:

- Any batch, including inactive, empty and historical batches.
- Any canonical movement.
- An explicit inventory level.
- Any inventory document line whose header has that source or destination branch, including draft and cancelled documents.

This does not populate Inventory with unrelated Catalog products. Relevant inactive products/packages remain inspectable with their state displayed. A package with a net ledger balance of zero remains visible.

The SQL stock classification is OUT_OF_STOCK at zero; LOW_STOCK only for a positive balance at or below an explicitly configured reorder level; IN_STOCK otherwise. An absent reorder level never invents a threshold. The factual overview's zero state is distinct from Phase 3D's configured alert policy: an unconfigured zero package is not automatically an alert.

Batch count includes historical/empty batches. Nearest expiry is the earliest known expiry among active positive-balance batches; it may be expired. Unknown expiry displays explicitly. Batch inspection includes all scoped batches; FEFO excludes expired, empty and inactive batches and orders unknown expiry after known eligible expiry using the existing canonical backend.

## Queries

Search is parameterized SQL over commercial English/Arabic names, scientific name, manufacturer and package barcodes. Manufacturer/category/route, branch and stock-state filters are applied before SQL pagination. Text uses literal substring matching; it does not interpret user wildcards. The current Arabic normalization policy is unchanged.

Overview, batches and movements use a read transaction for a consistent page and total. UI pages request 20 rows; backend bounds requests to 1–200 rows. No Catalog-wide product or full movement-history download is required. Catalog selectors separately request bounded pages. Lookup lists use existing Catalog lookup APIs.

Movement projections join the Phase 3B movement-to-document relation and Phase 3C count-line reference. They retain legacy reference fields when no originating document exists. Filters include branch, package, batch, movement type, date and document/count reference. Ordering includes the stable ID tie-breaker.

## Writes and safety

Opening Stock uses `record_opening_stock`, a durable request UUID, an explicit confirmation and a submit lock. It creates a batch/movement; it never edits a balance. An ambiguous retry retains its request ID. The canonical service rejects duplicate movement IDs without posting extra stock.

Receiving, adjustments, damage/expiry write-offs and transfers use the existing Phase 3B document editor and post service. Counts use Phase 3C review/reconciliation. Alerts, expiry, configured low stock and FEFO use Phase 3D. Reorder editing uses `set_inventory_level`. No new React stock arithmetic, FEFO allocation or reconciliation rules were added.

Contextual links carry product/package/batch identity to the existing editor; they do not save or post automatically. Historical identity/cost remains read-only. Branch reloads select the configured default branch; this is existing app behavior, so persistence checks explicitly reselect DEV3D after reload.

## Runtime isolation

Native `inventoryUiEnabled` selects the canonical overview, batches, movements, opening and levels before any prototype route. Existing capability checks select canonical documents, counts and monitoring. Capability errors fail closed. Old prototype code and isolated tests remain in the repository; they are not an operational native fallback. Later sales/purchases capabilities remain disabled and their handlers unregistered.

The shell's demo search is disabled on native Inventory routes, and their footer identifies the selected real branch and persisted local Inventory. General dashboard, notifications and other deferred shell features remain outside Phase 3E.

## Verification boundaries

The real development database has one relevant package with positive branch totals. Native checks verify its three historical zero-balance batches; package-level zero inclusion for each policy source is verified with isolated backend and frontend tests. No extra real stock or Catalog rows were created to stage zero totals. The controlled product's Arabic-name field is `1 2 3`; native queries verify that value plus an Arabic-script no-match search, while positive Arabic-script matching is covered in the backend tests.

Full receipt/adjustment/write-off/transfer/count lifecycles remain covered by existing tests. This phase's real database smoke posts only one labelled +1 opening on DEV3D. Reorder is temporarily changed from 4 to 6 and restored to 4; minimum 0 and maximum 10 remain. This changes the level timestamp. Reading monitoring advances alert detection/update timestamps without changing alert identity/status/details/history.
