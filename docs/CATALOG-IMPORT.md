# Controlled Egyptian Catalog import

Open **Products → Import Products** in the desktop app. Profile reads the source configured in `config/catalog-source.json`. Dry Run validates the full file and saves an immutable JSON report without changing any database rows. Review the counts and skipped rows, then choose Apply Import and confirm. Apply consumes that in-memory plan; source or Catalog changes require a new dry run. Restarting also requires a fresh dry run before another Apply.

## Mapping policy

- Display strings remain as supplied. Matching collapses Unicode whitespace and lowers ASCII letters only. Empty optional scientific/manufacturer/class fields become null.
- Source fingerprints are SHA-256 over the JSON array of all seven original fields; record numbers start at 1 after the header. Product identity hashes the first six conservatively normalized fields, excluding price. Neither English alone nor a parsed package suffix defines identity.
- The first exact source occurrence is considered; later exact copies are skipped. Non-identical rows sharing a normalized English name (Arabic when English is absent) are all held for review. A matching existing composite identity is also held rather than attaching guessed packages or modifying prices. Already committed fingerprints are unchanged even if their Catalog products have subsequently been edited or deactivated.
- Manufacturers, categories and routes reuse exact normalized names. New names retain the first encountered source display. Missing manufacturer/class is null. `UNKNOWN` is one route lookup; mixed source terminology is preserved. Lookup statistics count distinct normalized values, not row references; reused means present before this run.
- Scientific text is preserved on Product. All ingredient relationships are deferred, including apparently simple names; this is a warning and does not block an otherwise valid row. No medical parsing or spelling correction occurs.
- Each ready source record becomes a separate Product and one active default ProductPackage, labelled `Default Package`. Size, units, strength and cost remain null. Product notes and the source-row ledger identify the package as an import-derived placeholder. No barcode or stock data is generated.
- Strict decimal text becomes integer EGP minor units; no floating-point conversion. Zero follows the existing domain's allowed non-negative policy. More than two numeric decimal places are `ambiguous_price`; negative, empty, malformed or out-of-range values are `invalid_price`. Both are skipped for review, never rounded. Initial package prices share the run's UTC timestamp and reason `Initial Egyptian catalog import`.

## Safety, reports and recovery

Migration 003 adds `catalog_import_runs` and `catalog_import_rows`; migrations 001/002 are untouched. The ledger's `(source_id,fingerprint)` primary key records committed provenance alongside Product, Package and initial price in the same transaction. No extra speculative Catalog indexes were added. Existing normalized lookup indexes are used with in-memory caches and prepared insert statements.

Apply re-reads and hashes the source and compares a deterministic database snapshot with the dry-run plan. It creates an online SQLite backup, including committed WAL data, under `%LOCALAPPDATA%/com.m2health.pharmacy/import-reports/backups/`. It then checks the Catalog snapshot again inside an immediate transaction before creating run metadata. Backups use unique plan IDs and never overwrite an earlier backup.

Imports run on Tauri's blocking worker pool, not React. Each 250-row batch is an immediate transaction; the row ledger and committed count update atomically with its Catalog records. A failed batch rolls back fully. Earlier batches remain committed. The app's database mutex serializes local Catalog operations while Apply runs; status polling uses a separate short-lived mutex, so progress remains responsive. Another connection's write is detected between batches and stops further import rather than continuing against a changed plan.

After failure or interruption, keep the database and WAL files intact. Run a fresh dry run: committed fingerprints become `already_imported`, and eligible remaining records become ready. Apply that newly reviewed plan if resumption is intended. There is no automatic retry, fake cancellation, destructive reset or automatic backup restore. An unfinished run is reported as interrupted after restart. If final report persistence fails after database commits, the ledger/run record remains the recovery authority.

Dry-run and final JSON files live in `import-reports`, not beside the raw CSV. Reports contain original values, fingerprints, each row's final disposition, source hash, initial classification counts, lookup predictions/actual creations, imported count, backup path and timing. On a failed run, remaining ready rows become `not_imported_after_failure`. The report's `counts` retain the original plan classifications; `imported` and individual row dispositions record actual outcomes. The UI returns only the first 30 skipped/review rows, while JSON includes every source row.

## Boundaries

Only the configured canonical source is exposed through IPC. Tests may inject temporary roots, but the UI cannot choose arbitrary files. Development resolves config relative to the Cargo project; resource mappings retain the same paths for future packaging. Installer distribution is not configured or verified in this phase.

Repeated import does not update existing imported products or prices. Changed source records require review if their names/identities overlap existing Catalog entries. There is no interactive correction editor, automatic ingredient parsing, fuzzy manufacturer consolidation, package extraction, price rounding, or import cancellation. Those choices require separate explicit policies.

Rust tests use isolated databases and temporary sources, including a 25,000-row fixture. Frontend tests use browser-only IPC doubles. `scripts/verify_catalog_import.py --output <path>` reads the real development DB without writes and verifies counts, integrity, provenance, representative source fields and prices. Never point automated apply tests at the development DB.
