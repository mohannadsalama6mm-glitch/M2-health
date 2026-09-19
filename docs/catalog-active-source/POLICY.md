# Active canonical Catalog import

The current `src/data/karem/egyptian-drugs.csv` is the user-approved active source, SHA-256 `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`. The user explicitly authorized one real apply. This supersedes the earlier comparison-only policy; no further old/new CSV comparison is part of this task.

## Parsing

The approved hash and size remain pinned. Headers are edge-trimmed; original decoded field values and Arabic UTF-8 are preserved. Variable-column records are skipped individually. A field-boundary quoting check rejects unexpected quotes or characters after a closing quote without trying to repair the text. Valid CSV quoting, escaped quotes, commas and multiline fields remain supported. Invalid or overprecision prices are skipped; monetary conversion uses integer arithmetic only.

Missing optional scientific/manufacturer/class values are allowed. No medical correction, barcode, ingredient association, package size, expiry or stock is invented. UNKNOWN route retains the existing normalized lookup behavior.

## Identity and updates

An existing product must uniquely match the normalized combination of English name, Arabic name, scientific text, manufacturer, class and route, and have a resolvable default package. English name alone cannot cause reuse. Normalization changes ASCII case and whitespace only. Conflicting source identities/prices sharing a name are skipped. Exact logical repeats use the first valid record and skip subsequent repetitions.

Without an exact multi-field match, any existing English or Arabic name collision is held as ambiguous. Confidently new rows create a Product, separate default ProductPackage, needed normalized lookups and initial price. Source text is retained verbatim; absent optional lookups stay NULL.

For a matched product, normalized-equivalent metadata is reused without rewriting display text. Non-equivalent identity metadata is held, not silently overwritten. A differing price uses the existing price-history service, retaining the old amount and cost, closing only its current validity interval and appending the new price. Inactive products/packages are never silently reactivated for price updates.

## Backup, transactions and provenance

The pre-task online backup is recorded in `before.json`. The importer also creates its own verified online SQLite backup immediately before Catalog writes; the new run records backup path, timestamp and byte size. Both source and backup must pass integrity and foreign-key checks.

One IMMEDIATE transaction atomically records the run, Catalog changes and every source-row action. It rechecks the Catalog snapshot after backup. Failure rolls back all Catalog changes and row actions; a separate failed-run record reports the failure. No partial committed Catalog batch is left behind.

Migration **007 catalog_source_actions** adds only `catalog_source_runs` and `catalog_source_rows`. It does not modify migrations 001–006 or original import tables. New metadata supplements the original run with source version, reused/updated/skipped/failed counts and immutable per-row fingerprints/actions, product/package IDs and appended price ID when applicable. Source path/hash/timing/status remain in a new `catalog_import_runs` record. The old run and old `catalog_import_rows` are untouched.

There is no startup import. Reconciliation after apply is non-writing and should classify valid rows as already imported, with no mass insertion or price updates.

## Operational preservation

The importer never calls Inventory or count posting code. Verification compares all pre-existing Catalog and operational rows against the online backup. For this supplied file, the expected result is reuse rather than Catalog changes. Main stock must remain 8 and DEV stock 2; documents, transfers, count sessions and corrections must remain identical.

`scripts/catalog-active-source-native.mjs apply` has a durable single-attempt guard. Never rerun that stage after an attempted apply without inspecting persisted state. `verify` and `restart` are read-only stages. `scripts/verify_active_catalog_import.py` opens both the backup and real database read-only and checks preservation, the new run and all row actions.
