# Phase 2B final continuation report

Date: 2026-09-12. Project: M² Health. This report includes the original Phase 2B implementation and its canonical-source/restart continuation. No Phase 2C work or bulk import was performed.

## A. Continuation Summary

Continued the existing implementation without redesign. Recognized the user-copied in-project dataset, established one canonical path configuration, refreshed the read-only profile and documentation, reran automated checks, and verified desktop startup, native reads and migration idempotency. Phase 1 UI and demos remain intact.

## B. Canonical Egyptian Dataset

Exact path: `D:\project\M2 health\src\data\karem\egyptian-drugs.csv`. Filename: `egyptian-drugs.csv`. This is the only file found under the inspected directory. Format: UTF-8 CSV, comma separated, with a header and quoted multiline values. Size: 3,785,303 bytes. Data rows: 25,070, excluding the header.

Columns: `commercial_name_en`, `commercial_name_ar`, `scientific_name`, `manufacturer`, `drug_class`, `route`, `price_egp`.

`config/catalog-source.json` owns the canonical project-relative path. Tooling consumes it instead of the previous external attachment. SHA-256: `35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c`. The source remains byte-for-byte unchanged.

## C. Dataset Quality Review

Missing values: scientific name 2,308; manufacturer 71; drug class 97. English name, Arabic name, route and price have zero blank values in this snapshot. There are four duplicate full rows beyond the first occurrence and six normalized English-name groups with repeated names. These are review candidates, not automatically merged products.

Manufacturer and class names have case/spacing variants. Some manufacturer strings use `>` with uncertain relationship semantics. Scientific text can contain combinations and strengths; 9,571 strings contain `+`, without assuming that each separator reliably denotes a curated ingredient.

Thirteen route labels mix route/form concepts: ORAL.SOLID, ORAL.LIQUID, TOPICAL, INJECTION, SOAP, EFF, SPRAY, RECTAL, EYE, EAR, VAGINAL, MOUTH and UNKNOWN. UNKNOWN occurs 2,597 times. One price is over-precise: `4.415`, TETRAVITON 3 DOUBLE AMP. (N/A), ending physical CSV line 22225. It is flagged rather than rounded. Detailed counts and representative rows are in `catalog-reference-profile.json`.

## D. Migration 002 Final Verification

`002_catalog_core.sql` extends schema v1 transactionally. The v1 branch survives the isolated upgrade test. Reapplying through the migration framework leaves each version once. A deliberate conflict during actual migration 002 rolls back partial tables and preserves existing branch/schema data. Migration 001 was not edited.

## E. Final Catalog Schema

Nine normalized tables: manufacturers, categories, routes, active_ingredients, products, product_active_ingredients, product_packages, barcodes and product_price_history. UUID domain IDs, explicit foreign keys and UTC timestamps support future synchronization without adding a sync engine. No inventory, stock, batch, expiry or transaction tables were added.

## F. Product vs ProductPackage Model

Product holds logical commercial identity. ProductPackage holds the sellable SKU, package description, optional pack size/unit/strength, positive integer unit count and active/default flags. Each package can have its own barcodes and prices. Product has no barcode, stock or price column. Source names are not automatically split into product/package identities.

## G. Arabic / English Product Naming

Separate nullable English and Arabic commercial display fields and matching keys. At least one usable commercial name is required. Original text is retained. Scientific display text is optional and preserved independently of curated ingredient links. No automatic transliteration or medical spelling changes.

## H. Manufacturer / Drug Class / Route / Ingredient Model

Normalized lookups preserve display text. CSV `drug_class` maps to categories. Route labels are preserved in routes without pretending they are detailed dosage forms; richer dosage forms are deferred. Missing relationships may be null. Product/ingredient links are many-to-many with optional unparsed strength, order and a unique pair constraint. No automatic scientific-string splitting.

## I. Barcode Model

Barcodes belong to packages. Global uniqueness, leading-zero preservation and at most one primary barcode per package. Packages may have none or multiple. No barcodes were invented from the source CSV.

## J. Price History Model

An immediate transaction closes the prior current interval and appends a new price. Historical amounts cannot be edited/deleted. Partial uniqueness permits one current price per package; triggers prevent overlapping intervals. Failed insertions roll back the prior closure. Same-millisecond updates use monotonically increasing UTC milliseconds. Future scheduling/backdating is deferred.

## K. Money Storage Strategy

EGP amounts are INTEGER minor units; 22.50 EGP becomes 2250. Cost is optional. Strict tables and constraints reject fractional, negative or out-of-range stored values. The exact decimal-text helper rejects excess precision, ambiguous separators and exponent notation without floating-point conversion or silent rounding. Maximum persisted values stay within JavaScript's exact integer range.

## L. Normalization Strategy

Matching keys trim and collapse Unicode whitespace and lowercase ASCII A–Z. Original display strings, Arabic spelling/diacritics, punctuation and non-ASCII Latin case remain unchanged. No fuzzy merging. Empty optional source values must be mapped to null in a future controlled importer, retaining raw provenance separately.

## M. Repository Architecture

Separate repositories for each lookup, Products, packages, barcodes, ingredient links and price history. Shared validation lives in `db/catalog_validation.rs`; SQL stays in database/repository code. Existing `AppDb` managed connection and structured error conventions remain. Soft deactivation preserves historical references.

## N. Tauri Commands / DTOs

26 explicit Catalog commands cover lookup creation/listing/state, Product creation/get/detail/list/state, ingredient association, packages, barcodes and price operations. Thin handlers delegate to repositories. CamelCase Rust DTOs and `catalog.types.ts` provide stable contracts. ProductDetail aggregates lookups, ingredients, packages, barcodes and current prices within a consistent read transaction. Catalog screens do not consume these commands yet.

## O. Database Constraints / Indexes

Foreign keys, unique normalized lookups, unique ingredient pairs, global barcode uniqueness, one primary barcode, one active default package, integer price bounds and positive package units are enforced in SQLite. Indexes cover normalized name/scientific search, lookup relationships, package ownership, ingredient joins, barcode lookup and price ordering/current state. Search supports prefixes, exact barcodes, lookup filters and bounded pagination; no FTS or fuzzy matching.

## P. Tests Added

15 Catalog Rust tests added, alongside 12 preserved/adapted foundation tests: 27 total. Coverage includes migration upgrade/rerun/rollback, branch preservation, lookup normalization/state, bilingual/null values, relationships, multiple ingredients/packages/barcodes, uniqueness/default rules, integer money, history append/immutability/rollback/concurrency, search/pagination, ProductDetail and inactive behavior. All tests use memory or temporary files. The 27 frontend tests were preserved.

## Q. CSV-aware Validation

The profiler reads the canonical configuration and dataset without opening SQLite. It records source hash, actual fields/counts, quality findings and six representative rows. Rust tests map those samples into isolated normalized databases, including missing relationships, UNKNOWN route and decimal prices. Synthetic test packages only prove price representability; they are not inferred import mappings. Output cannot overwrite the input CSV. No production importer exists.

## R. Restart / Migration Idempotency Verification

The standard `npm run tauri dev` workflow starts Vite and the native application. Repeated startup retains branch UUID `f319a917-95d0-47bf-b007-ab76b7c044ae`, originally created at `2026-09-12T09:31:52.894Z`. Migration records remain exactly once each. Native Dashboard and Products screens render; the topbar shows Main Pharmacy while Catalog displays its existing demo records. Native read commands return the real branch and valid empty Catalog results. No migration or observed browser-console errors.

## S. Real Development DB State

Resolved database: `C:\Users\Mohannad\AppData\Local\com.m2health.pharmacy\m2-health.db`. Read-only inspection reports schema version 2; version 1/branches once; version 2/catalog_core once. Branch count: 1, with MAIN once. Products: 0. Product packages: 0. Barcodes: 0. Product price history: 0. Manufacturers, categories, routes, ingredients and product/ingredient links are also all zero. `foreign_key_check` returns no violations. No verification rows were inserted, deleted or updated in the real development database.

## T. Canonical Dataset Integration Status

Complete for Phase 2B: the in-project raw file is recognized and documented; one configuration owns its location; the profiler reads it and isolated schema validation uses its representative profile. The three layers remain raw source, normalized SQLite and React UI. Future import can consume this configuration. No raw CSV import into React, automatic startup seed or bulk database import was added.

## U. Verification Results

- `npm run lint`: passed. Generated Tauri target/schema files are excluded from application linting.
- `npm run build`: passed, including TypeScript and Vite production output.
- `npm test`: 27 passed.
- `cargo fmt --check`: passed.
- `cargo check`: passed without warnings.
- `cargo test`: 27 passed, plus successful doc-test run.
- `npm run tauri dev`: native startup/restart passed.
- Read-only canonical CSV and development DB checks: passed.

The restricted sandbox could not access Cargo's build lock or the app-data database, and one sandboxed Playwright run stalled during server cleanup. Verification completed outside that sandbox. Cargo itself now works; no Windows security policy was modified. A localhost-only development WebView debugging session was used for read-only IPC/console checks, without changing committed runtime configuration.

On the final restart, Windows input automation reported `SendInput sent 0 of 1 events; GetLastError=87` after a geometry refresh. Catalog navigation was verified through the development WebView instead; its demo search/control content was present and console errors remained empty. This was an automation-driver limitation, not a failed Catalog assertion. Final frontend run: 27 passed in 2.0 minutes; final Rust run: 15 Catalog and 12 foundation tests passed.

## V. Files Added / Changed

Added migration 002, Catalog DTO/validation modules, nine Catalog repository modules, Catalog command handlers, `src-tauri/tests/catalog.rs`, frontend DTO contracts, canonical-source configuration, read-only profiler, dataset profile, architecture documentation and this report. Updated migration registration/module exports, command registration, shared error mapping, foundation schema-version tests, ESLint exclusions and README. The user supplied `src/data/karem/egyptian-drugs.csv`; its bytes were not modified. Existing screen implementations, demo fixtures and frontend test files were preserved. No new crates or npm packages were needed for Phase 2B.

## W. Known Limitations

Catalog UI remains demo-based. General editing, reassignment of default/primary records, richer dosage forms, controlled import, duplicate review, provenance staging and backdated prices remain future work. Prefix/OR search needs profiling against real imported volumes. Lookup normalization is deliberately conservative. Product/package boundaries and medical combinations require review. The source is raw reference data with documented anomalies, not automatically curated clinical data. Installer packaging and production operational modules are outside this phase.

## X. Phase 2B Status

COMPLETE

## Y. Recommended Phase 2C Plan

Connect the existing Catalog UI to SQLite with real search, filters, pagination, details and add/edit flows; then package, barcode, ingredient and price-history management with loading/error/empty states. Add explicit transactional edit/default/primary contracts as needed. Prepare the controlled importer design around the canonical-source configuration: validation, normalization, duplicate review, lookup mapping, product/package review, exact price conversion, insertion and import reporting. Keep bulk execution as a separately authorized import phase. Phase 2C was not started.
