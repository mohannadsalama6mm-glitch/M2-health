# Phase 2D — controlled Catalog import

## A. Apply Summary

One controlled development Apply succeeded. It imported 25,061 of 25,070 source rows in 49.966 seconds (501.55 imported rows/second). Four exact duplicate occurrences and five review-required rows were skipped. Zero failures. No second development Apply occurred.

## B. Source Verification

Canonical config: `D:/project/M2 health/config/catalog-source.json`.
Source: `D:/project/M2 health/src/data/karem/egyptian-drugs.csv`.
Size: 3,785,303 bytes; rows: 25,070 excluding the header.
SHA-256: `35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c`.
Columns: commercial_name_en, commercial_name_ar, scientific_name, manufacturer, drug_class, route, price_egp.

The importer rechecked source hash and Catalog snapshot before writing. The dry-run report predicted 25,061 ready, four exact duplicates, four possible duplicates and one ambiguous price. Classification policy was not changed during continuation.

## C. Pre-Import DB Counts

Branches 1; products 1; packages 1; manufacturers 1; categories 1; routes 1; active ingredients 1; product–ingredient relations 1; barcodes 1; price-history records 2. Import runs and provenance rows were zero after migration 003. These were the earlier inactive Phase 2C verification records.

## D. Backup Details

SQLite online backup:
`C:/Users/Mohannad/AppData/Local/com.m2health.pharmacy/import-reports/backups/catalog-before-cb5c12a4-bfcf-41d8-9012-bdcf0fd07700.sqlite3`

Timestamp: 2026-09-12 20:33:32.844673 UTC. Size: 204,800 bytes. Integrity check: `ok`. Backup table counts match the recorded pre-import counts. Every pre-existing row was compared with the final database and is unchanged. The backup was not restored.

## E. Apply Results

Run ID: `cb5c12a4-bfcf-41d8-9012-bdcf0fd07700`.
Started: 2026-09-12T20:33:32.844Z; finished: 2026-09-12T20:34:22.110Z.
Status: complete. Source rows imported: 25,061; skipped: nine; failed: zero. Transactions used batches of 250. The UI animation frame responded during Apply, and progress showed intermediate committed counts.

## F. Products / Packages / Prices Created

25,061 Products; 25,061 default ProductPackages; 25,061 initial price-history rows. Price history uses exact integer minor units and a consistent effective timestamp. Default packages have null size, unit, unit count and strength; notes and provenance mark their placeholder origin. Zero imported barcodes, ingredient relations or stock records.

## G. Lookup Mapping Results

Created 5,544 manufacturers, 2,445 categories and 13 routes. Zero distinct existing Catalog lookups matched the source; repeated source values reused cached normalized lookup IDs. Missing manufacturers/classes remain null. UNKNOWN maps to one preserved source route lookup. Scientific text remains unchanged; all ingredient relations are deferred.

## H. Exact Duplicates

Four redundant occurrences were skipped at logical source rows 2000, 10340, 19937 and 20421: AVICHEST adult syrup, HESSTA cream, SCAR NOT gel and SIGMACYN vial. Each corresponding unique source record was imported only once. Exact duplicate rows share a fingerprint with their imported first occurrence; they do not create additional ledger/Product/Package rows.

## I. Review-Required Rows

- Rows 4518–4519: CITICOLINE 500 MG 20 CAPSULES variants, conflicting manufacturer/class/price information.
- Rows 20232–20233: SEPTOLA LOZENGES 24 TAB HONEY AND LEMON variants, differing manufacturer/class values.
- Row 22201: TETRAVITON 3 DOUBLE AMP. (N/A), price `4.415`.

All five fingerprints are absent from the committed ledger. No rounding, merging or medical correction was performed.

## J. UI Verification

The restarted native Catalog displayed 25,061 active imported products. Verified English (`ABDOMINAL BELT`), Arabic (`أبدومينال`), scientific (`CARBOCYSTEINE`) and manufacturer (`HIKMA PHARMA`) searches, plus manufacturer, category and UNKNOWN-route filters. Product details and current package prices loaded. Imported packages display “No barcodes assigned” and no stock values.

Real data exposed an existing Pagination component rendering all 2,507 page buttons. A bounded page window with endpoints and ellipses fixes that verified scale defect, retaining existing controls/styles. Native page 2 loaded the correct next records. A browser regression verifies endpoint navigation, bounded button count and correct SQL-query offset requests. No importer/classification changes were made for this issue. Native console/page errors observed: zero.

## K. Representative Data Validation

- `1 2 3 (ONE TWO THREE) 20 F.C.TABS.`: English/Arabic names and full scientific combination preserved; 10.00 EGP.
- `4 WET INTIMATE GEL 100 ML`: scientific name stays null; 40.00 EGP.
- `ABDOMINAL BELT`: manufacturer stays null; UNKNOWN route; Arabic name preserved; 170.00 EGP.
- `2M WHITES BEEGU MARIN SPRAY`: UNKNOWN route and full combination text preserved; 32.00 EGP.
- `ABIONEM CREAM 60 GM`: source 20.95 becomes exactly 2095 minor units and displays 20.95 EGP.

Both read-only SQLite comparisons and native detail/package screens were inspected. No medical judgment was applied.

## L. Restart Persistence

The application was restarted with `npm run tauri dev`. Catalog data remained present, schema versions 1–3 remained correct, and MAIN branch ID `f319a917-95d0-47bf-b007-ab76b7c044ae` remained unchanged. Exactly one import run and 25,061 provenance rows persisted. No automatic import occurred. Integrity: `ok`; foreign-key errors: none.

## M. Post-Import Idempotency / Second Dry Run

Second dry run: 25,061 already imported, zero ready/new inserts, four exact duplicate occurrences and the same five review rows. All new-lookup predictions were zero. Apply was disabled. The development database did not receive a second Apply; repeat Apply is covered by isolated Rust tests.

Second report:
`C:/Users/Mohannad/AppData/Local/com.m2health.pharmacy/import-reports/6f0db72e-8aa0-41a5-b56f-e86c9ffdf8fc-dry-run.json`.

## N. Source Hash After Import

`35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c`.
Matches the reviewed dry run and pre-import profile byte-for-byte.

## O. Automated Checks

Final checks: npm lint, production build, all 32 Playwright tests, cargo fmt --check, cargo check and all 46 Rust integration tests. The Rust suite includes exact money, bilingual/multiline parsing, migration preservation, dry-run zero writes, lookup reuse, source/existing duplicates, invalid required values, backup contents, stale plans, repeated Apply, failed-batch retry, external connection changes and a 25,000-row import.

Build succeeds with Vite's advisory that the main JavaScript chunk exceeds 500 kB. Packaging/installer distribution is not verified. The final test completion evidence is recorded with the task output.

## P. Final Import Report Path

Full 25,070-row immutable JSON, 11,035,977 bytes:
`C:/Users/Mohannad/AppData/Local/com.m2health.pharmacy/import-reports/cb5c12a4-bfcf-41d8-9012-bdcf0fd07700-apply.json`.

Compact verified summary: `docs/phase-2d-final-summary.json`. Source profile, before/after/restart audits and native UI evidence are also in `docs/phase-2d-*.json`. Policy/recovery instructions: `docs/CATALOG-IMPORT.md`.

## Q. Final Database Counts

Branches 1; products 25,062; packages 25,062; manufacturers 5,545; categories 2,446; routes 14; active ingredients 1; product–ingredient relations 1; barcodes 1; price-history records 25,063. Import runs 1; provenance rows 25,061. Existing Phase 2C records explain the difference between created and final counts.

## R. Known Remaining Review Queue

Five held rows listed above. Scientific ingredient relations remain deferred. No package-size extraction, fuzzy name merging, interactive correction UI, price rounding, automatic existing-price updates or cancellation was added. The source remains raw and unchanged. See the policy guide for safe retry and report interpretation.

## S. Phase 2D Status

COMPLETE

## T. Recommended Phase 3 Plan

Inventory + Batches + Expiry + Stock Ledger: first define branch-owned stock and batch identity, immutable quantity movements and reconciliation rules; then implement transactional receiving/adjustments and expiry/FEFO views with isolated tests. Do not infer opening stock from this Catalog import. Phase 3 has not been implemented.

## Files added / changed

Added `src-tauri/src/catalog_import/{mod,source,planner,importer,commands}.rs`, migration `003_catalog_import.sql`, Rust import tests, typed TypeScript import boundary, ImportProducts screen, UI import/pagination tests, read-only verification scripts and import documentation/evidence. Updated Cargo dependencies/lock, migration registry, Tauri handlers/resource mapping, canonical config role, Catalog import link, route registration, bounded shared Pagination and schema-version test expectations. Migrations 001/002 and the canonical CSV were not modified.
