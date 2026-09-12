# Catalog domain — Phase 2B

SQLite is the operational local source. This phase adds normalized Catalog storage and command contracts; it does not connect the Catalog UI or import production data. Migration 001 is immutable. Migration 002 extends the same database and migration runner.

## Reference dataset

The current canonical raw Egyptian source is the in-project CSV resolved by `config/catalog-source.json`. That configuration is the single source-path reference for tooling and future importer code; the former external attachment is no longer used. The read-only profiler `scripts/inspect_catalog_reference.py` consumes the configuration by default. Its checked-in output `catalog-reference-profile.json` contains the resolved project-relative source path, source SHA-256, counts, quality observations and six representative rows. Rows containing quoted line breaks are parsed with the CSV library; `sourceEndLine` is the actual ending physical line, including the header.

Three layers remain distinct: raw source dataset, normalized SQLite domain, and React Catalog UI. The CSV is source truth for review and future population, not a substitute for domain tables or a frontend data module. Future import should follow validation, conservative normalization, duplicate review, domain mapping and controlled SQLite insertion with an import report. No part of that production pipeline is implemented here, and the source file is not modified by profiling.

The seven columns are commercial_name_en, commercial_name_ar, scientific_name, manufacturer, drug_class, route and price_egp. There are 25,070 records; 2,308 missing scientific names, 71 missing manufacturers, 97 missing classes and 2,597 UNKNOWN routes. Both commercial-name columns are populated in this snapshot. Four full-row duplicates and six normalized English-name duplicate groups require later review. The profile records case/spacing variants in manufacturer/class names. Manufacturer strings can contain `>` relationships whose meaning is not inferred. Scientific strings include combinations, strengths and descriptive text; 9,571 contain `+`, which is not treated as a reliable parser delimiter.

There are 13 route labels. ORAL.SOLID/ORAL.LIQUID, TOPICAL and INJECTION coexist with SOAP, EFF, SPRAY, EYE, EAR, RECTAL, VAGINAL, MOUTH and UNKNOWN. They mix route, form and source classification. Approach C is used: a normalized `routes` lookup preserves the source label now; detailed dosage forms and a clinically curated route taxonomy remain deferred. UNKNOWN is allowed as a lookup label, while a missing route can be null. No lookup is seeded automatically.

The source has no distinct SKU, package-size, barcode, cost or effective-price date columns. Commercial names often contain package/strength text; it is retained intact until an importer with explicit review can establish product/package boundaries. The source's `price_egp` is destined for a reviewed package price, never for a Product column. One value, `4.415` for TETRAVITON 3 DOUBLE AMP. (N/A), exceeds two decimal places and must be flagged. No rounding or medical correction is performed.

## Entities and identity

- `manufacturers`: UUID, original name, unique normalized name, optional country/phone/email/website, active state and UTC timestamps.
- `categories`: normalized lookup mapping directly to source `drug_class`; no invented category hierarchy. Optional description, active state and timestamps.
- `routes`: source route/classification label, normalized matching name, active state and timestamps. It is not a detailed dosage-form entity.
- `active_ingredients`: curated ingredient name, normalized matching key, optional description, active state and timestamps.
- `products`: logical commercial identity with separate nullable English/Arabic names and matching keys, nullable original scientific text and its matching key, optional manufacturer/category/route foreign keys, description, notes, active state and timestamps. At least one usable commercial name is required. Product names are deliberately not unique: similar names do not establish medical equivalence.
- `product_active_ingredients`: UUID relation plus product/ingredient foreign keys, unparsed strength text, explicit order and timestamps. A unique pair prevents duplicate links. Empty ingredient lists are valid and do not contradict a populated scientific display string.
- `product_packages`: UUID sellable SKU belonging to one Product, package label, optional pack-size text, unit name, positive integer units-per-package and strength text, default/active flags and timestamps. Pack-size text can represent `100 ml` without assuming a conversion. No stock, batch or expiry fields exist.
- `barcodes`: UUID record belonging to a package, globally unique barcode text, primary flag and timestamps. Leading zeroes are preserved. A package may have zero or many barcodes.
- `product_price_history`: UUID record belonging to a package, integer selling/cost minor units, effective interval, optional reason and creation timestamp. History is retained.

All domain IDs are generated as UUID v4 in repositories. Foreign keys use RESTRICT. No hard-delete commands exist. Lookup/product/package deactivation preserves records and references. Deactivating a lookup does not silently deactivate linked products. Deactivating a Product does not rewrite its packages; Product active status governs new price changes. Package deactivation clears its default flag but retains barcodes and historical/current prices. Reactivation does not automatically reclaim default status.

Catalog identities are global in the local database; no branch ID is invented for the shared product definition. Branch stock, prices/overrides and operational associations require explicit later design. A package's default price is currently EGP and not branch-specific.

## Money and temporal rules

Prices are signed 64-bit SQLite INTEGERs constrained to non-negative values no larger than JavaScript's exact integer maximum, 9,007,199,254,740,991. DTOs use integer-valued numbers. SQLite STRICT tables reject non-integral REAL values. `egp_to_minor` parses decimal text directly with checked integer arithmetic: 22.50 becomes 2250; exponent notation, comma separators, negatives and excess precision are rejected. Cost may be unknown/null and is never invented from selling price.

`set_package_price` takes integer values and applies a price immediately. An IMMEDIATE transaction closes the old open interval and inserts a new record. A partial unique index allows at most one current row per package. Timestamps use fixed-width UTC milliseconds; same-millisecond updates and backward clock movement advance at least one millisecond beyond the last price's start, preserving strict ordering. No caller-supplied backdating/scheduling exists in this phase. Extremely rapid updates can put the logical effective timestamp just ahead of wall time; current means the unclosed row.

Triggers reject overlapping intervals, changes to historical values and deletion. The only allowed update closes a current interval once. If inserting a new price fails, the old interval closure rolls back. Historical ordering is newest effective time first. Product/package deactivation does not erase current or past prices; new prices require both to be active. A future correction workflow must preserve the audit trail and explicitly define backdated rules.

## Normalization and nullable data

Original display values are stored exactly as submitted, including Arabic and spacing. Matching keys trim/collapse Unicode whitespace and lowercase ASCII A–Z only. Arabic spelling, diacritics, tatweel, punctuation, strength and plus signs are preserved; there is no transliteration, Unicode compatibility folding, accent removal or fuzzy merge. Non-ASCII Latin case is also left unchanged. Lookup normalized names are unique. A duplicate is reported, not silently merged or renamed.

Optional missing values use null; explicit empty/whitespace-only strings are rejected at the repository boundary. The future importer must map empty source fields to null while retaining original source provenance in a dedicated import workflow. Scientific text is not parsed into ingredients automatically. Categories/manufacturers/routes may be missing. At least one bilingual commercial name remains mandatory. The six-row test fixture exercises missing relationships, missing scientific text, UNKNOWN route and fractional prices, without providing an importer.

## Repository and IPC boundaries

`db/models/catalog.rs` defines explicit camelCase DTOs. `db/catalog_validation.rs` owns shared conservative normalization, text/UUID validation and exact money conversion. Separate repositories own manufacturers, categories, routes, ingredients, products, package records, barcodes, ingredient links and price history. SQL is absent from command handlers and React. Errors remain structured and redact SQL details.

`commands/catalog.rs` registers 26 explicit commands: create/list/set-active for each of four lookups; create/get/get-detail/list/set-active for Products; link-product-ingredient; create/list/set-active for packages; add/list barcodes; set/get-current/get-history prices. Creation and association inputs reject unknown fields. No generic CRUD command or raw SQL invocation is exposed.

`ProductDetail` aggregates Product, optional lookup records, ordered ingredient links, packages, package barcodes and optional current prices within a consistent read transaction. Details include inactive records for historical/reference use. `src/lib/tauri/catalog.types.ts` mirrors the DTO contract; no screen invokes Catalog commands yet.

Product search accepts normalized English/Arabic/scientific prefixes, manufacturer-name prefixes or an exact barcode. Wildcards are escaped as literal input. Filters support manufacturer/category/route IDs and explicit inactive inclusion. Default page size is 50, maximum 200, with stable ordering and total count from the same read snapshot. Empty search returns the page. An inactive package's barcode can still identify its parent for catalog review; ordinary Product listings exclude inactive Products by default. No FTS or fuzzy search is introduced. Prefix indexes support the individual search predicates; the combined OR query may still scan at larger scale, so query-plan/performance work belongs with real import volumes.

## Constraints and extension boundaries

Unique normalized lookup keys, unique ingredient pairs, global barcode uniqueness, one primary barcode per package, at most one active default package per Product, foreign keys, strict integer money and positive integer package-unit constraints live in SQLite. Default/primary conflicts are rejected rather than silently reassigning another record. There is no transfer-primary/default command yet. No hard-delete or general product-edit command is present; Phase 2C should add reviewed transactional edit contracts as its UI needs are defined.

Indexes cover product name/scientific matching, manufacturer/category/route references, package ownership, both ingredient join directions, barcode lookup/ownership and price history ordering/current selection. `updated_at` advances in the exposed state-changing repositories. Append-only price rows use effective intervals instead of an update timestamp. Fresh creation timestamps use the Phase 2A UTC convention.

Inventory, suppliers, customers, sales, purchases, accounting, authentication, Supabase, sync and AI remain absent. Bulk import is a separate later phase and must retain provenance, flag collisions/ambiguous package boundaries and quarantine invalid prices. SQLite remains permanent operational storage, not a temporary cache.
