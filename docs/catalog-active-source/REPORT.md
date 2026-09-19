# Active Catalog import — completed

## A. Import Summary

Performed **one controlled real APPLY** against the development SQLite Catalog. **25,012 source rows reused existing products**, with **0 new products, 0 updates and 0 failures**. All **25,088 source-row outcomes** are persisted under new run `385b6f78-2785-40a9-a1a6-d52332bf17c5`.

The current file is the active Catalog source. Existing products already represent its accepted records; the real apply recorded their source links without creating duplicates. No Catalog reset, old-run rewrite or Inventory operation occurred. No Phase 3D work was started.

## B. Source

- Path: `D:\project\M2 health\src\data\karem\egyptian-drugs.csv`.
- SHA-256: `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`.
- Size: **26,437,838 bytes**; **25,088 data records**, plus header.
- Columns: `commercial_name_en`, `commercial_name_ar`, `scientific_name`, `manufacturer`, `drug_class`, `route`, `price_egp`.

The file remains byte-for-byte unchanged. Header padding is trimmed for recognition; original decoded text and Arabic are preserved.

## C. Parsing Results

- **25,021** records safely parsed with seven fields and acceptable quoting.
- **25,020** also have valid integer-convertible price precision.
- **25,012** accepted after duplicate and identity checks.
- **26** malformed column-count records skipped.
- **41** additional quoting records skipped. The profiler flags 57 quoting-related records overall, including 16 already counted as malformed; the exclusive skip counts avoid double counting.
- **4** ambiguous identities, **4** repeated records and **1** overprecision price skipped.
- **76 total skipped records**, each with source record number, disposition/reason, original fields and fingerprint in the persisted report and [skipped-row extract](skipped-rows.json).

No malformed text was repaired, medicine identity merged or price rounded. Missing optional metadata remains allowed; ingredient links and unspecified package information remain deferred.

## D. Backup

The immediate pre-apply online backup is:

`C:\Users\Mohannad\AppData\Local\com.m2health.pharmacy\import-reports\backups\catalog-before-385b6f78-2785-40a9-a1a6-d52332bf17c5.sqlite3`

- Recorded timestamp: **2026-09-14 12:55:10.431 UTC**.
- Size: **50,814,976 bytes**.
- Backup integrity: **ok**; foreign-key violations: **0**.

A separate pre-task online backup, before migration 007, is also retained:

`D:\project\M2 health\backups\catalog-active-source\before-apply-20260914T123350Z.sqlite3`

- Timestamp: **2026-09-14 12:33:50 UTC**.
- Size: **50,790,400 bytes**.
- Integrity: **ok**; foreign-key violations: **0**.

The source database passed the same checks before writing. [Before evidence](before.json) records baseline counts.

## E. Pre-Import Catalog Counts

Products **25,062**; packages **25,062**; price history **25,063**; manufacturers **5,545**; categories **2,446**; routes **14**; active ingredients **1**; ingredient links **1**; barcodes **1**. Historical import runs **1**; historical imported-row provenance **25,061**.

## F. Real APPLY Results

- Existing/reused: **25,012**.
- Metadata or price updates: **0**.
- New products/packages: **0**.
- Skipped ambiguous: **4**.
- Skipped malformed: **26**.
- Skipped quoting: **41**.
- Skipped repeated records: **4**.
- Skipped invalid/ambiguous price: **1**.
- Failed: **0**.

The single transaction committed the new run and all row actions together. Run start: **12:55:10.431 UTC**; committed completion: **12:55:33.644 UTC**. Total importer duration including its checks/backup/report work: **51.206 seconds**.

## G. Lookup Results

No lookups were created or changed. Accepted products reference **5,534 distinct existing manufacturers**, **2,437 categories** and **13 routes**. Full Catalog totals remain **5,545 / 2,446 / 14**, respectively. Missing values were not replaced with invented labels.

## H. Price Results

New initial prices: **0**. Appended price-history records: **0**. All **25,063** existing price-history rows remain identical to the backup.

One source price, record **22,218**, `TETRAVITON 3 DOUBLE AMP. (N/A)`, is **4.415 EGP** and was skipped without rounding. The importer has isolated test coverage proving that an approved differing price appends history while retaining the original amount.

## I. Duplicate Prevention

Reuse requires a unique match across normalized English name, Arabic name, scientific text, manufacturer, class and route, plus a resolvable default package. A name alone cannot cause a merge. Conflicting source identities/prices are skipped; repeated logical records are skipped after the first valid occurrence. Current Catalog product/package counts did not grow.

## J. Post-Import Catalog Counts

All Catalog counts in section E remain unchanged. Import runs: **2 total**, comprising the untouched historical run and **1 new active-source run**. Original `catalog_import_rows`: **25,061 unchanged**. New `catalog_source_runs`: **1**; new `catalog_source_rows`: **25,088**.

Migration **007 catalog_source_actions** adds only the supplementary Catalog run/action tables. Migrations **001–006** and their stored application timestamps remain intact. [Final audit](after.json) records all table counts.

## K. Catalog UI Verification

Verified in the real native app, then repeated after restart:

- Catalog loads with **25,062 total products**; the default active filter retains **25,061 active products**.
- English search `PANADOL`: **13** results.
- Arabic search `بانادول أكوتي هياد كولد`: **1** result.
- Scientific search `BROMPHENIRAMINE+PARACETAMOL(ACETAMINOPHEN)+PSEUDOEPHEDRINE`: **2** results.
- Manufacturer search `GLAXO SMITHKLINE`: **233** results.
- Exact manufacturer filter: **185**; category filter: **143**; ORAL.SOLID route filter: **10,585**.
- Pagination advances to the expected second page.
- Product Details and package data load; the tested PANADOL package displays **62.00 EGP**, matching **6,200 minor units** in SQLite.

[Restart screenshot](catalog-restart.png) and [native evidence](native.json) retain the results.

## L. Inventory Preservation

Every pre-existing Inventory row matches the pre-task backup. **Main stock: 8; DEV3B stock: 2**. Inventory remains **3 batches, 7 movements, 1 transfer**. The import created **zero inventory movements**.

## M. Phase 3B / Phase 3C Preservation

All document headers, lines, movement links, posting requests and sequences are unchanged. Counts remain **3 documents, 3 document lines, 4 movement links, 3 posting requests**.

The single completed count session, line, result, completion request and sequence are unchanged. Its correction remains intact. Native reads confirmed the completed status and original document movement references before and after restart. No operational command was executed.

## N. Restart Verification

Restarted with `npm run tauri dev`. The Catalog and the same completed source run persisted. There is still only **one active-source run** and **25,088 row actions**; no import occurred automatically. Stock, documents and count data remain intact. No browser console errors, migration rejection or Rust panic was observed.

## O. Idempotency Verification

Reran reconciliation in **non-writing mode** after apply. **25,012 already imported**, **0 new inserts**, **0 price updates**, with the same **76 skips**. No second real APPLY was performed. The helper records `applyCommandCalls: 1` and has a durable single-attempt guard.

## P. DB Integrity

Final, post-restart `integrity_check`: **ok**. `foreign_key_check`: **0 violations**. All pre-existing Catalog, lookup, barcode, price, Inventory and count tables compare equal to the pre-task backup. The old import run and old row provenance remain unchanged. The source SHA-256 still matches the approved hash.

## Q. Automated Checks

- `npm run lint`: **PASS**.
- `npm run build`: **PASS**; Vite's non-failing bundle-size advisory remains.
- `npm test`: **72 passed, 0 failed**.
- `cargo fmt --check`: **PASS**.
- `cargo check`: **PASS**.
- `cargo test`: **229 passed, 0 failed**.

Rust breakdown: Catalog 22; active-source import 8; existing import 16; foundation 12; isolated legacy Inventory 20; canonical Inventory 26; Inventory documents 39; partners 11; purchases 14; recovery 2; sales 16; count sessions 43.

The task added eight importer safety tests and one browser completion test. Schema expectations now explicitly cover version 7 and a failing future version 8; rollback assertions were retained. Existing tests were not weakened. Tests use isolated databases; native verification performed the one authorized Catalog apply only.

## R. Import Report Path

Authoritative persisted report:

`C:\Users\Mohannad\AppData\Local\com.m2health.pharmacy\import-reports\385b6f78-2785-40a9-a1a6-d52332bf17c5-apply.json`

Project copy: [apply-report.json](apply-report.json). Supporting evidence: [native.json](native.json), [after.json](after.json), [skipped-rows.json](skipped-rows.json), [cargo-test.log](cargo-test.log).

## S. Files Changed

- Active source configuration and current documentation: `config/catalog-source.json`, `README.md`, `.gitignore` (exclude backup directory), `docs/catalog-active-source/`.
- Import implementation: `src-tauri/src/catalog_import/active.rs`, `source.rs`, `mod.rs`, `planner.rs`, `importer.rs`, `commands.rs`.
- Catalog provenance migration: `src-tauri/migrations/007_catalog_source_actions.sql`, registered in `src-tauri/src/db/migrations.rs`.
- UI and typed client: `src/features/catalog/ImportProducts.tsx`, `src/lib/tauri/catalog-import.ts`.
- Tests: new `src-tauri/tests/catalog_active_source.rs`; updates to `catalog_import.rs`, `foundation.rs`, `recovery.rs`, `catalog.rs`, `inventory_core.rs`, `inventory_documents.rs`, `stock_count_sessions.rs`, and `tests/catalog-import.spec.ts`. Inventory test edits only adjust expected schema version.
- Verification: `scripts/catalog-active-source-native.mjs`, `scripts/verify_active_catalog_import.py`.

The CSV and migrations 001–006 were not changed by this task. Existing unrelated working-tree changes were retained.

## T. Known Skipped / Review Rows

Full original values, fingerprints and reasons are in [skipped-rows.json](skipped-rows.json).

- Ambiguous: **4520, 4521, 20248, 20249** (CITICOLINE and SEPTOLA variants).
- Repeated: **2000, 10343, 19953, 20437**.
- Overprecision price: **22218**, `4.415`.
- Malformed: **3088, 3089, 3090, 5801, 5802, 13952, 13953, 16404–16412, 18052, 18053, 18154–18157, 21918, 21919, 24324, 24325**.
- Quoting: **1092, 2218, 2812, 3246, 3649, 3651, 3652, 5959, 5960, 6547, 6548, 7077, 7544, 9163, 9164, 9902, 16630–16634, 16638, 16639, 16721–16723, 16726–16729, 17255, 17539, 18829, 20654, 21343, 21606, 22009, 23999, 24000, 24180, 24233**.

Numbers are one-based parsed source-record numbers excluding the header. Skips do not remove any existing Catalog record.

## U. FINAL STATUS

COMPLETE

Stopped after the real Catalog import and verification. Phase 3D was not started.
