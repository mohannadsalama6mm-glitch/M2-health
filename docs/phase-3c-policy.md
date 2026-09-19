# Phase 3C stock count policy

The active implementation is `src-tauri/src/inventory/counts`, with fresh migration
006. The disabled prototype count service and schema are not registered.
Migrations 001–005 are unchanged. `inventory_movements` remains the only stock ledger.

## Scope and snapshot

ALL_INVENTORY captures active batches of active packages/products in the selected
active branch, including zero-stock batches. The snapshot is taken in an IMMEDIATE
transaction when DRAFT starts. Expected quantities and the snapshot watermark are
stored permanently. The unit is a whole sellable ProductPackage, not a tablet.

Existing or previously unrecorded batches may be explicitly added during entry.
An existing batch's expected quantity is its balance at the session watermark;
a newly identified batch has expected zero. New batch identity fields remain null
unless supplied; registration creates no movement or quantity. Duplicate batch
lines and cross-branch/package combinations are rejected. Unrecorded active
packages with no batch are added explicitly rather than fabricating Catalog stock.

## Entry and exclusions

Counted quantity starts NULL and remains uncounted until explicitly entered. Zero
is a real count. Entry may be reset to NULL. An uncounted line blocks completion
unless explicitly excluded with a nonblank persisted reason. Excluded lines have
no correction. Inline row saves use both session and line revisions. Unsaved rows
block Review and pagination. An explicit reload action can discard unsaved edits
after a stale revision error.

## Review and reconciliation

Snapshot variance is physical count minus snapshot expected. Posting correction
is physical count minus current live balance. Review shows both, and counts all
intervening movement activity, including incoming/outgoing pairs whose net is zero.

Completion carries a hash of the reviewed session revision and per-line ledger
state. An IMMEDIATE transaction re-reads the canonical ledger and rejects a changed
review token. The user refreshes Review and confirms the updated exact number of
nonzero corrections. The writer lock prevents another writer from changing stock
between validation and commit. Negative corrections run before positive ones to
avoid a temporary package-cap overflow when redistributing counted quantities.
Existing batch/package stock caps and no-negative constraints remain active.

Every nonzero correction is STOCK_COUNT_CORRECTION with reference_type
stock_count_line and reference_id identifying its line. The line identifies the
session. Zero differences have an audit result but no zero-quantity movement.
All correction movements, immutable results, COMPLETED state and the completion
request commit together. Failures roll back the entire reconciliation.

## Lifecycle and retries

DRAFT → IN_PROGRESS → REVIEW → COMPLETED; REVIEW may return to IN_PROGRESS.
DRAFT, IN_PROGRESS and REVIEW may cancel. COMPLETED and CANCELLED are immutable.
Cancellation does not post stock. Completed result rows preserve the exact live
baseline used at reconciliation, physical count, delta, intervention count and
movement ID even when later stock activity changes balances.

Completion request UUIDs and full payloads are stored durably. Identical retries
return the completed result; a reused key with changed content is rejected, as is
a second completion under a different key. The frontend retains the request UUID
per session/review token through reloads. The native restart check retries that
original UUID.

## Deferred scope

Category/shelf/product scopes, fractional units, expiry automation and later
operational modules are deferred. The existing altered CSV import-source blocker
is unchanged. No Phase 3D work is included.
