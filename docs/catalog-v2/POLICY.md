# Canonical Source v2: adopted, comparison only

The user intentionally approved the current Egyptian Drug Catalog file as v2. Its different hash is **not** the former accidental-source blocker. This adoption supersedes those earlier blocker statements; historical reports remain unchanged.

Current v2 SHA-256: `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`, 26,437,838 bytes, 25,088 tolerantly decoded source records.

Historical v1 SHA-256: `35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c`, 3,785,303 bytes, 25,070 source rows. The original Phase 2D run imported 25,061 rows. No physical v1 CSV is required: its saved apply report retains original row fields and fingerprints, verified against all imported provenance rows.

## Parsing and comparison

- Preserve the entire v2 file, byte-for-byte. Only approved v2 headers are edge-trimmed for schema recognition; decoded row values remain raw in the application and full plan.
- Read-only profiling tolerates variable field counts and reports their record numbers. Insert planning rejects those records before opening a database transaction. Historical unversioned insert behavior remains strict.
- The offline review checks every tolerantly decoded record with a separate strict syntax check, preserving physical line spans. There are 26 variable-column records and 29 additional strict quoting issues. Twelve other seven-column records retain ambiguous enclosing quotes. Do not silently repair or join any of them.
- Field statistics describe the 25,062 seven-column records. Malformed fragments are not treated as additional medicines; missing values within them are unknown, not guessed.
- Match a unique normalized English commercial name conservatively. Normalization changes only ASCII case and whitespace; strength, pack size and punctuation are never removed. Arabic-only matches require identity review. Multiple candidates and duplicate source names are held.
- Compare edge-trimmed text. Shared manufacturer/class/route labels can retain the first imported spelling of their normalized key. Record such display-only differences separately; they do not establish a Catalog edit or meaningful source change.
- Monetary comparison uses integer minor units with at most two decimal places. Do not round `4.415` or infer a price from malformed records.
- `UNCHANGED` permits shared-lookup display variants. `NEW` and `CHANGED` require an unambiguous, complete, valid candidate. Review concerns take precedence; underlying field differences remain visible even on held rows. Classifications are mutually exclusive, but field-impact counts overlap them.

## Application policy

`config/catalog-source.json` selects v2 and records both versions, approved hashes, sizes, row counts and `apply_allowed: false`. Profiling verifies the selected version against its exact bytes. The UI labels v2 comparison-only and disables Apply, including after a dry run. The backend independently reloads the source policy and rejects applying v2 before backups, run creation, or Catalog writes. A forged report flag cannot enable it.

The existing application's Dry Run is an insert eligibility planner, not a metadata-update engine. For this file, malformed column counts block that planner with an explicit error. The complete versioned comparison is `update-plan.json` and the offline `review.html`; no partial insert plan is represented as a safe v2 update.

## Provenance and future updates

No schema migration is needed for this adoption. Migrations 001–006 and original import tables remain unchanged. The original run's stored v1 hash, path, row numbers, fingerprints and imported count retain their meaning. A path now pointing to v2 never relabels that run as v2.

New report profiles carry `sourceVersion`, `applyAllowed` and `malformedRows`; legacy reports deserialize with an unspecified version and preserve their original hash. Resolve historical versions from the immutable hash registry, not the current path.

Every proposed row in `update-plan.json` records `sourceVersion`, `sourceHash`, `sourceRow`, `sourceRowFingerprint`, logical fingerprint, physical line span, candidate IDs, field deltas, warnings, classification, `proposedAction` and `applied: false`. Fingerprints are SHA-256 of compact UTF-8 JSON arrays of the untrimmed decoded fields, matching the existing importer convention. The complete file hash anchors source bytes.

Before a future APPLY task, approve a parsing contract or a newly fingerprinted source revision, review all held identities and choose an explicit update policy. An actual update implementation must persist the approved source version/hash/row fingerprint and committed action in an append-only update journal linked to product/package and run IDs, without repurposing original import-row uniqueness or historical dispositions. That future migration and updater are not implemented or enabled here.

If a future approved row changes price, append a new dated price record through the established price policy; never rewrite the original price. Metadata changes require field-level approval and old-value checks against a fresh baseline. Do not edit shared lookup labels globally: resolve a reviewed lookup ID for that product. Preserve local edits, stock, documents, counts and all existing provenance. Removed/unmatched source rows are never deletion instructions.

## Reproduce

Run `scripts/catalog_v2_review.py` with Python. It opens the development database with `mode=ro`, enables `query_only`, and takes a consistent read transaction. It writes review artifacts only. `baseline.json` is create-once; subsequent runs refuse a changed database or CSV instead of silently replacing the baseline.

Run `scripts/catalog_v2_review.py --verify` to compare all table counts and complete-row hashes, schema, migrations, branch balances and CSV hash with the baseline. Run `scripts/test_catalog_v2_review.py` for six isolated comparison tests. `scripts/render_catalog_v2_review.py` renders the offline HTML from the JSON artifacts and never connects to SQLite.

**APPLY STATUS: NOT APPLIED.**
