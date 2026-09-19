# Egyptian Drug Catalog — Canonical Source v2

## A. Canonical v2 Adoption Summary

The supplied file is intentionally adopted as Canonical Source v2. Its difference from v1 is no longer the accidental-source blocker. The version registry identifies historical v1 separately from current v2; both remain unavailable for apply under the current configuration. The CSV has not been rewritten or repaired.

The application labels v2, verifies its exact hash/size/record count, profiles malformed column counts and disables Apply. The backend independently rejects apply before database changes. Full comparison results are in [the offline row review](review.html), [summary](summary.json) and [complete update plan](update-plan.json).

## B. v2 Source Profile

- SHA-256: `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`.
- Size: **26,437,838 bytes**.
- **25,088 tolerantly decoded data records**, plus header; **25,062** have seven columns. This is not evidence of 25,088 valid medicine identities.
- Columns: `commercial_name_en`, `commercial_name_ar`, `scientific_name`, `manufacturer`, `drug_class`, `route`, `price_egp`. All seven header names contain edge padding; the raw file remains unchanged.
- Missing values among seven-column records: **2,308** scientific names, **71** manufacturers, **97** drug classes.
- Routes: EAR **41**; EFF **1,036**; EYE **473**; INJECTION **2,635**; MOUTH **173**; ORAL.LIQUID **2,236**; ORAL.SOLID **10,585**; RECTAL **171**; SOAP **130**; SPRAY **488**; TOPICAL **4,139**; UNKNOWN **2,596**; VAGINAL **359**.
- Exact duplicate groups: **4**, each appearing twice (**4 additional duplicate records**). Edge-trimmed exact duplicates give the same count.
- Normalized commercial-name duplicate groups: **6**, covering **12 records**.
- Invalid price syntax/range: **0** among seven-column records, excluding the separately counted overprecision case. **1** price exceeds two decimals: record **22,218**, `TETRAVITON 3 DOUBLE AMP. (N/A)`, **4.415 EGP**. It is held without rounding.
- Malformed records: **55** distinct records: **26** wrong-column records and **29** additional strict quoting issues. **12** further records retain ambiguous enclosing quotes after tolerant decoding and require review.
- Strict UTF-8 decoding succeeds; **0 U+FFFD replacement characters**, **0 NUL characters**. **25,060** seven-column records contain Arabic script; none has an empty Arabic-name field. Two names are non-Arabic-script text. This is a text-integrity check, not a linguistic or medical validation.

Every record was examined. Field-level counts and routes exclude the 26 malformed fragments because assigning their fields would require guessing. All fragments, decoded fields, source record numbers and physical line spans remain in the full plan.

## C. v1 Historical Source Reference

Historical SHA-256: `35867044249d327b81c8c87988d6328c802da39359dc768eb4e7f1e9528dee0c`.

Original source: **3,785,303 bytes**, **25,070 rows**. Phase 2D run `cb5c12a4-bfcf-41d8-9012-bdcf0fd07700` completed on **2026-09-12**, importing **25,061** rows and skipping nine.

The saved apply report supplies all original row fields. All **25,061 imported provenance fingerprints** were verified against both its recorded fingerprints and independently recomputed fingerprints. No physical v1 CSV was required. The original run, provenance rows and saved report remain unchanged.

## D. Current Catalog Baseline

**25,062 products**, **25,062 packages**, **25,063 price-history records**, **25,061 imported-row provenance records**, **1 import run**. Lookup counts: manufacturers **5,545**, categories **2,446**, routes **14**. One barcode, one ingredient and one ingredient association remain.

All **27 database tables** and the complete schema match the start-of-adoption snapshot. SQLite integrity is **ok** and foreign-key violations are **0**. Active migrations remain **001 branches, 002 catalog_core, 003 catalog_import, 004 inventory_core, 005 inventory_workflows, 006 stock_counts**.

Inventory remains **3 batches**, **7 movements**, **3 documents**, **1 transfer** and **1 completed count session**. Main stock is **8**; the development branch stock is **2**. No sale, purchase, stock or count operation was performed.

## E. v2 vs Current Catalog Comparison

Every source record has exactly one primary classification:

- **UNCHANGED: 25,008**.
- **NEW: 0**.
- **CHANGED: 0** confidently actionable rows.
- **POSSIBLE_DUPLICATE: 12**.
- **REVIEW_REQUIRED: 68**.

These total **25,088**. `CHANGED: 0` does not mean all source text is identical: uncertain field differences are classified as review-required. Shared lookup display variants are recorded separately and treated as unchanged.

## F. New Rows

There are **no confidently new products**. Five well-formed records have no current Catalog candidate: the two CITICOLINE variants, two SEPTOLA variants and the overprecision TETRAVITON row. They have historical skipped-row candidates and remain duplicate/review cases. Another 26 unmatched records are malformed fragments, not proposed products.

Eight imported products have no complete matched v2 record because their source text appears in malformed multiline fragments: BRETTA WIN 75 GM GEL, DEPO LUTIN 30 CAPS., MERFO 30 PIECES, ORDEX 30 PIECES, PROSTENAL CONTROL 30 TABS., PSYLLOSE 10 SACHETS, TALK-FREE MOUTH SPRAY 60 ML and VONASPIRE 20 MG 14 F.C.TABS. Their Catalog records are preserved; absence from a complete match is not a deletion instruction.

## G. Changed Rows

**387 rows** have literal metadata differences against current Catalog values. **348** are only shared-lookup case/whitespace display differences; **39** remaining rows require parsing or identity review. No substantive local edits since v1 were detected among uniquely matched imported records.

Overlapping field-impact counts: English name **4**, Arabic name **23**, scientific text **2**, manufacturer **41**, class **317**, route **0**. Manufacturer/class figures include lookup display variants. All exact old/new values are retained in the plan; no scientific text or product identity was corrected.

## H. Price Changes

**0 confirmed price changes** among uniquely matched valid-price candidates. No price-history entry was added, rewritten, ended or deleted. Invalid/unmatched/ambiguous records are not treated as proof of equal prices. The `4.415` value remains held for explicit review.

## I. Possible Duplicates

**12 records in 6 normalized-name groups** are held. Four groups contain exact repeats; the other two contain competing CITICOLINE and SEPTOLA source variants. No medicines, manufacturers or package identities were merged.

## J. Review-Required Rows

**68**: 26 wrong-column records, 29 other strict CSV quoting issues, 12 additional ambiguous quoted-text records and one overprecision price. Their source numbers, physical line spans, raw fields, candidate IDs and reasons are available in the full plan and offline review.

## K. Proposed Update Strategy

Keep all rows unapplied. Review the 80 duplicate/review records first and approve a parsing contract or a separately fingerprinted source revision. Do not infer that extra parser records are new medicines.

A future updater must revalidate source and Catalog snapshots, protect local edits, require field-level approval, record committed actions and append approved new prices without rewriting history. Shared lookup display variants need no update. See [POLICY.md](POLICY.md) for precise matching, parsing and future journal requirements.

## L. Metadata / Provenance Changes

Configuration now registers v1 and v2 explicitly. New application report profiles carry `sourceVersion`, `applyAllowed` and `malformedRows`, with backward-compatible historical deserialization. The complete dry-run plan records source version/hash, raw row fingerprint, proposed action and `applied: false` for every record.

**No database metadata or migration was changed.** A durable update-action journal belongs to a separately approved future APPLY implementation. Original Phase 2D run fields retain their historical meaning.

## M. Dry-Run Results

**25,008 skip-unchanged**, **80 held**, **0 insert/update actions**, **0 database writes**. Source bytes, every database table, schema and historical import report match the start-of-adoption baseline. [Preservation evidence](preservation.json) includes complete table hashes and balances.

The application’s legacy insert planner is deliberately blocked by malformed column counts. The delivered versioned comparison is a separate complete report, not a silently applied or partially accepted insert plan.

## N. Automated Checks

- `npm run lint`: **PASS**, no lint errors.
- `npm run build`: **PASS**. Vite reports the existing bundle-size advisory; it does not fail the build.
- `npm test`: **71 passed, 0 failed**.
- `cargo fmt --check`: **PASS**.
- `cargo check`: **PASS**.
- `cargo test`: **221 passed, 0 failed**. Breakdown: Catalog 22; import 16; foundation 12; legacy isolated inventory 20; canonical inventory 26; inventory documents 39; partners 11; purchases 14; recovery 2; sales 16; stock-count sessions 43.
- Python comparison tests: **6 passed**.
- Offline review checks: pagination, classification filtering, empty results, exact overprecision row details and no JavaScript errors passed.
- Independent Rust/Python parsing: the entire decoded record array has the same SHA-256, `e984db1bae206527dabdaced9bc92bc94fe91534226b23b84079f219e68f40da`.

Four Rust tests and one browser test were added. Existing tests were not weakened. Tests used isolated fixtures/databases; the actual source test only reads the approved file and uses an in-memory database. No native application restart or write workflow was needed for this adoption.

## O. Files Changed

Task-specific tracked edits: `config/catalog-source.json`; `README.md`; `src-tauri/src/catalog_import/{source,mod,planner,importer}.rs`; `src-tauri/tests/catalog_import.rs`; `src/lib/tauri/catalog-import.ts`; `src/features/catalog/ImportProducts.tsx`; `tests/catalog-import.spec.ts`.

Added tools: `scripts/catalog_v2_review.py`, `scripts/test_catalog_v2_review.py`, `scripts/render_catalog_v2_review.py`.

Added evidence under `docs/catalog-v2/`: this report, policy, baseline, profile, summary, full update plan, preservation audit, offline HTML and screenshot, check results and Rust test log. Earlier recovery and Phase 3 work in the shared working tree was retained.

**The CSV, migrations 001–006, database records and historical import report were not modified.**

## P. APPLY STATUS

NOT APPLIED

## Q. Recommended Next Step

Review the 80 held records and decide the exact treatment of CSV padding/quoting and the overprecision price. Then commission a separately approved parsing/update policy. Keep Apply disabled until that review is complete. No next task has been started.
