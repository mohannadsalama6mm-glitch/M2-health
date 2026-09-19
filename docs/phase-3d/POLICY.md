# Phase 3D inventory monitoring policy

The active ledger remains `inventory_movements`. Migration 008 adds advisory records only. Migrations 001–007 and Catalog import provenance are not changed by this phase. Migration 007's existing registered name is `catalog_source_actions`.

## Dates and eligibility

The business date is the UTC calendar date (`date('now')` in SQLite). The native evaluation command always uses that date; inspection and isolated service tests may supply a real, validated calendar date. The UI displays the backend business date without local-time conversion. Timestamps remain UTC.

Only active batches with positive ledger balances and known expiry participate in expiry alerts. Expiry before the business date is expired; expiry on the business date is expiring. The warning horizon is inclusive. Inspection windows are expired, 0–30, 31–60 and 61–90 days. Unknown dates are never inferred. Canonical FEFO excludes expired and exhausted batches, orders known expiry first with existing receipt/creation/UUID tie-breakers, and places unknown expiry last. Inspection neither reserves nor moves stock.

## Settings, levels and severity

Each branch defaults to 90 warning days and 30 critical days, with expiry and stock alerts enabled. Defaults become persistent on evaluation; explicit settings changes never alter `inventory_levels`. Horizons must be integers between 0 and 36500 and critical must not exceed warning.

Stock monitoring includes only explicit `inventory_levels` records. Positive stock at or below reorder generates LOW_STOCK (MEDIUM). Zero generates only OUT_OF_STOCK (CRITICAL), suppressing a simultaneous LOW_STOCK row. Packages without levels generate neither stock alert; no default reorder threshold is invented. Minimum/maximum are displayed, not substituted for the reorder threshold.

BATCH_EXPIRED is CRITICAL. BATCH_EXPIRING within the critical horizon is HIGH, and otherwise within warning is MEDIUM. Monitoring windows remain available even if the configured alert horizon differs; no matching alert is labelled accordingly.

## Lifecycles

The fingerprint is `branch:type:batch` for expiry or `branch:type:package` for stock. It is unique in SQLite. Severity changes update the same expiring alert instead of producing one row per threshold bucket. Becoming expired resolves the expiring lifecycle and creates/reuses the expired lifecycle.

Repeated evaluation preserves ID and first detection, updates last detection, and creates no additional logical row. An immutable history event is appended only when status, severity or details change. Date-based details can change once a day. Evaluation runs in one IMMEDIATE transaction, serializing competing evaluators and rolling back all partial alert changes on failure.

ACTIVE means the current condition needs attention. DISMISSED suppresses attention while the condition continues; it never changes stock. Once the condition clears, either ACTIVE or DISMISSED becomes RESOLVED and receives a resolution timestamp. A later recurrence reactivates the same ID and retains all history. Disabling a category resolves its alerts on evaluation; re-enabling reactivates remaining conditions. Alerts and history cannot be deleted; history cannot be updated.

## Execution and UI

Evaluation runs at native startup, on monitoring page/query refresh, and on manual Refresh Alerts. Successful Phase 3B posting/reversal and Phase 3C completion trigger a best-effort refresh after their transaction commits. Alert failure is logged and cannot undo a successful stock operation. There is no scheduler or high-frequency polling; other low-level inventory commands are reflected on the next manual/page/startup evaluation.

`/inventory/alerts`, `/inventory/expiry`, `/inventory/low-stock` and `/inventory/fefo` use typed Tauri commands and SQLite Catalog identity. They never read CSV. Alert filters cover branch/type/severity/status and updated-date range. List counts and rows share a read transaction. Pagination is database-side (20 per UI page, backend maximum 200). FEFO uses the canonical bounded query. Alert lookup in monitoring joins the unique fingerprint index; existing indexed ledger sums supply balances.

An expired alert links to the existing Inventory Documents workspace, where the operator explicitly selects the expiry write-off type, batch, quantity and reason, saves, readies and confirms posting. No alert action directly writes off stock. The general Notifications screen remains demo; this focused Inventory Alert Center is the real Phase 3D surface.

Phase 3E, POS, Sales, Purchases, partners, authentication, cloud sync and AI remain outside this phase. Later prototype handlers and migrations remain unregistered.
