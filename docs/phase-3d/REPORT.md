# M² Health — Phase 3D completion report

Verified 15 September 2026. Phase 3E, Phase 4 and Phase 5 were not started.

## A. Phase 3D Summary

Implemented and verified real expiry, low-stock, out-of-stock, persistent inventory alerts, branch settings, lifecycle history and canonical FEFO inspection. Native startup, controlled DEV3D stock workflows and a full native restart passed. Catalog import was not repeated. The canonical inventory ledger and previous phase guarantees remain active.

## B. Migration 008

`008_inventory_alerts.sql` is registered once after the unchanged migration 007. The real database contains exactly versions 1–8, with migration 008 applied at `2026-09-15T07:55:33.281Z`. Restart did not reapply it. Versions 1–7 retain their original metadata.

Active sequence: `001_branches`, `002_catalog_core`, `003_catalog_import`, `004_inventory_core`, `005_inventory_workflows`, `006_stock_counts`, `007_catalog_source_actions`, `008_inventory_alerts`.

The actual existing 007 name remains `catalog_source_actions`; it was not renamed. Its SHA-256 is `df5d72f61f6a2c0f98fe3a6ad318ece16df060a491f7efda3f985cd3f7546074`. Migration upgrade, partial-failure rollback, prior metadata preservation and populated Catalog/Inventory/document/count preservation passed in isolated tests. Later prototype migrations and handlers remain unregistered. No duplicate Tauri commands occur.

## C. Alert Schema

Added `inventory_alert_settings`, `inventory_alerts`, and append-only `inventory_alert_history`. Alert types and statuses are constrained; expiry alerts reference their branch/package/batch through a composite foreign key. Fingerprints are unique. Alert deletion and history update/deletion are blocked. Indexed views derive quantity from the existing ledger; there is no second stock ledger or stored alert-owned quantity balance.

## D. Settings

Default per branch: warning 90 days, critical 30 days, expiry enabled and stock alerts enabled. Invalid horizons are rejected. DEV3D persisted a non-default 75-day warning and 20-day critical horizon. At the end of smoke, expiry is enabled and stock alerts are disabled to verify persistence of a non-default boolean. Main retains 90/30 and both enabled. The one DEV3D reorder-level record remains independently configured at 4, minimum 0, maximum 10.

## E. Expiry Rules

Uses the backend UTC calendar date consistently. Only active batches with positive stock and known expiry participate. Expiry before today is expired; today through the configured horizon is expiring, inclusive. Zero stock is excluded; unknown expiry is not guessed. Impossible calendar dates are rejected. UI windows are expired, 0–30, 31–60 and 61–90 days.

## F. Low-Stock / Out-of-Stock Rules

Only explicit `inventory_levels` records participate. Positive quantity at or below reorder produces LOW_STOCK. Zero produces OUT_OF_STOCK alone. A package without a configured level produces neither stock alert. No default threshold was introduced. Native evaluation before configuring the DEV3D level generated expiry alerts only; after configuring it, LOW_STOCK appeared.

## G. FEFO Verification

Native inspection returned the near-expiry batch, then the later known expiry, then unknown expiry. The positive expired batch was absent. At zero stock, native FEFO returned no batches. Main inspection contained no DEV3D batches and retained quantity 8. The existing Phase 3A query remains authoritative, including receipt-time, creation-time and UUID tie-breakers; isolated regression tests continue to pass. Inspection never reserves or changes stock.

## H. Deduplication

Ten repeated evaluations before resolution retained the same three logical alert IDs. After restart, another ten evaluations retained the same four lifecycle IDs and statuses, with unchanged history lengths. The final database has four unique fingerprints, one active alert and three resolved alerts. Evaluation is transactional and concurrent evaluators serialize safely.

## I. Resolution

An explicit expiry write-off resolved BATCH_EXPIRED. An explicit outgoing adjustment exhausted the remaining DEV3D stock: expiry and low-stock alerts resolved and OUT_OF_STOCK activated. An explicit incoming adjustment restored stock above reorder: OUT_OF_STOCK resolved and BATCH_EXPIRING reactivated under its existing ID. The final history contains 10 immutable events. Dismissal, recurrence and retained history passed isolated tests; dismissal never changes stock.

## J. Workflow Integration

Best-effort alert refresh runs after successful canonical document posting/reversal and stock-count completion, outside the successful stock transaction. Tests demonstrate that an alert refresh failure cannot undo a receipt. Native resolution appeared immediately after posting without requiring an intervening manual evaluation. Startup and page/manual refresh are also supported. No scheduler or automatic write-off/purchase creation was added.

## K. UI Integration

Real native routes: `/inventory/alerts`, `/inventory/expiry`, `/inventory/low-stock`, `/inventory/fefo`. They provide branch selection, backend pagination, type/severity/status/date filters, settings, manual refresh, history, expiry windows and FEFO package selection from SQLite Catalog. Browser-only preview explains the desktop requirement and shows no fake alerts.

The nine registered commands are `get_inventory_alert_settings`, `update_inventory_alert_settings`, `evaluate_inventory_alerts`, `list_inventory_alerts`, `get_inventory_alert`, `dismiss_inventory_alert`, `list_expiry_inventory`, `list_low_stock_inventory`, and `get_fefo_inventory_batches`.

Expired-alert actions navigate to the existing document workspace for an explicit expiry write-off. They do not post from the alert record. The general Notifications screen remains demo. The same-filter loading regression was fixed; viewport checks passed at 1366, 1440 and 1920 pixels.

## L. Automated Tests

- `npm run lint`: PASS, including the final native smoke helper changes.
- `npm run build`: PASS; existing bundle-size advisory remains non-blocking.
- `npm test`: **86 passed**, including 14 dedicated new alert browser tests and updated canonical expiry workflow checks.
- `cargo fmt --check`: PASS.
- `cargo check`: PASS.
- `cargo test`: **272 passed**, including **43 dedicated alert tests**. The earlier 41-test milestone grew to 43 with populated upgrade preservation and invalid-range coverage.

Rust suite counts: Catalog 22; active source 8; import 16; foundation 12; isolated prototype inventory 20; alerts 43; canonical inventory 26; documents 39; isolated partners 11; isolated purchases 14; recovery 2; isolated sales 16; counts 43. Prototype tests remain isolated and do not enable those runtime features.

No production code changed during native verification. Only smoke/verification helpers were extended for the requested native FEFO cases, so the full completed suites were not unnecessarily repeated.

## M. DEV3D Smoke Test

Created branch `DEV3D`, ID `0fc16eaf-cefa-4576-8201-9b03541036f8`, labelled “DEV Phase 3D Alerts”. It reuses an existing Catalog package; no Catalog products were created or updated.

Four minimal batches cover the requested native FEFO cases: near expiry, later known expiry, expired, and unknown expiry. Each received one explicit opening unit. One reorder level was configured. Three canonical documents then performed expiry write-off, outgoing adjustment and incoming adjustment. This generated nine DEV3D movements total, with immutable history retained. Final DEV3D stock is 5; only its near-expiry batch retains stock.

Expiry detection, no-threshold behavior, LOW_STOCK, OUT_OF_STOCK, zero-stock expiry exclusion, FEFO order, branch isolation, deduplication, resolution and settings all passed. Native evidence is in `native.json`; screenshots accompany it.

## N. Restart Verification

Closed the native app gracefully and restarted with `npm run tauri dev`. Schema initialization succeeded again. Settings, alerts, ACTIVE/RESOLVED states and event history persisted. Repeated evaluation created no duplicate. Catalog, Dashboard, Inventory and FEFO routes opened successfully. No console/page errors, migration rejection or Rust panic were observed. Main remains 8; existing DEV remains 2.

## O. DB Integrity

Final read-only checks: `integrity_check = ok`; `foreign_key_check = []`. The final comparison reads the real development DB in query-only mode and compares it with `backups/phase-3d/before-20260914T185350Z.sqlite3`. All original rows remain intact except intentional global document-number counter advancement attributable to the three DEV3D documents. That advancement is checked exactly.

## P. Before / After Counts

- Migrations: 7 → 8; branches: 2 → 3.
- Catalog products: 25,062 → 25,062; packages: 25,062 → 25,062; price-history rows: 25,063 → 25,063; barcodes: 1 → 1.
- Manufacturers: 5,545 → 5,545; categories: 2,446 → 2,446; routes: 14 → 14; active ingredients: 1 → 1.
- Import runs: 2 → 2; import rows: 25,061 → 25,061; active-source runs: 1 → 1; source actions: 25,088 → 25,088.
- Batches: 3 → 7; movements: 7 → 16; transfers: 1 → 1; inventory levels: 0 → 1.
- Documents: 3 → 6; document lines: 3 → 8; document movement links: 4 → 9; post requests: 3 → 6; document sequence rows: 3 → 5.
- Count sessions, lines, results, completion requests and count sequence rows: each 1 → 1.
- New alert schema: 3 settings rows, 4 alerts (1 ACTIVE, 3 RESOLVED), 10 history events.
- Stock: Main 8 → 8; existing DEV 2 → 2; new DEV3D 0 → 5.

## Q. Existing Data Preservation

All original Catalog, prices, import records, source actions, inventory batches/movements/transfers, Phase 3B documents and Phase 3C counts retain their data. New stock records belong only to DEV3D. Default alert settings were materialized for all three branches, without changing their existing inventory levels or balances.

The active CSV SHA-256 remains `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`. Source configuration SHA-256 remains `c84361272efc3890978b5802cec5766ac3ae0a3110c7d6bd88affabf3c6a9c87`. No CSV apply, import rerun, or migration 007 edit occurred.

## R. Files Changed

New production files: `src-tauri/migrations/008_inventory_alerts.sql`; `src-tauri/src/inventory/alerts/{mod,models,commands}.rs`; `src/lib/tauri/inventory-alerts.ts`; `src/features/inventory/InventoryAlerts.tsx`.

Integration changes: `src-tauri/src/db/migrations.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/inventory/mod.rs`, `src-tauri/src/inventory/documents/commands.rs`, `src-tauri/src/inventory/counts/commands.rs`, `src-tauri/src/recovery.rs`, `src/app/AppRoutes.tsx`, `src/app/RecoveryBoundary.tsx`, `src/lib/tauri/recovery.ts`.

Tests: new `src-tauri/tests/inventory_alerts.rs`, `tests/inventory-alerts-double.ts`, `tests/inventory-alerts.spec.ts`; schema expectations updated in foundation/Catalog/core/documents/count/recovery Rust tests; obsolete expiry-dialog tests updated in `tests/inventory.spec.ts` and `tests/continuation.spec.ts`, with explicit expired-batch support in `tests/inventory-documents-double.ts`.

Verification and documentation: `scripts/phase3d-native-smoke.mjs`, `scripts/verify_inventory_alerts.py`, and `docs/phase-3d/` evidence, policy, screenshots, checks and this report. Pre-existing changes from earlier phases were preserved.

## S. Known Limitations

Business date is explicitly UTC, not a configurable local pharmacy timezone. There is no background scheduler, push notification service or automatic stock action. Direct low-level inventory operations are reflected at the next page/manual/startup refresh. Alert history loads per alert; the principal lists are paginated. The write-off link opens the document workspace without automatically filling a document. The broader Inventory redesign and general Notifications integration remain deferred. Build output includes a non-blocking bundle-size advisory.

## T. Phase 3D Status

COMPLETE

## U. Recommended Phase 3E Plan

Unify the remaining Inventory screens around the canonical services: batch/package detail, balance and ledger navigation, links to documents/counts/alerts, clear loading/error/empty states, and accessible responsive layouts. Preserve all stock protections and validate using isolated tests followed by controlled DEV smoke. Phase 3E has not been started.
