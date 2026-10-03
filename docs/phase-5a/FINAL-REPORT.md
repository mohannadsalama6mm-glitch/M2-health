# M² Health Phase 5A Final Report

## A. Phase 5A Summary

Phase 5A adds canonical, durable Supplier and Customer directories without starting purchasing. React uses a centralized typed IPC client, thin Tauri commands call a Rust-owned domain module, and rusqlite persists the records through migration 010. The canonical native routes provide backend-driven lists, create/edit/detail/status flows, structured errors, and honest future-history states.

## B. Repository Audit

The repository contained supplier/customer UI, models, repositories, commands, tests, and purchase code tied to an obsolete combined `partners_purchases` proposal. Those handlers were unregistered and unsafe to activate because their schema is not in the active migration sequence. They remain isolated for historical test compatibility. The established route structure and design system were reused. No prototype supplier, customer, or purchase command was registered.

## C. Migration Added

`010_partners.sql` is the only new active migration. It creates `partner_sequences`, `suppliers`, `customers`, indexes, and history-retention triggers. It creates no purchase, receipt, receiving, payment, or supplier-return table. Migrations 001–009 have no source diff.

Active sequence: `001_branches`, `002_catalog_core`, `003_catalog_import`, `004_inventory_core`, `005_inventory_workflows`, `006_stock_counts`, `007_catalog_source_actions`, `008_inventory_alerts`, `009_sales`, `010_partners`.

## D. Supplier Schema

Suppliers have a stable UUID, generated `SUP-000001` code, display name, contact person, two optional phone fields, email, address, tax/registration identifier, notes, active state, revision, and timestamps. Codes are unique case-insensitively. Non-null tax identifiers are unique case-insensitively. Indexed active/name ordering supports bounded lists. Delete triggers retain durable identity history.

## E. Customer Schema

Customers have a stable UUID, generated `CUS-000001` code, display name, two optional phone fields, email, address, notes, active state, revision, and timestamps. Codes are unique case-insensitively. Indexed active/name ordering supports bounded lists. Delete triggers retain durable identity history. No medical or health-profile fields were added.

## F. Supplier Domain Rules

Rust owns create, read, update, status, list, search, validation, normalization, and concurrency rules. Names are required and trimmed; optional blank values become null; lengths and email shape are bounded; updates and status changes use revision compare-and-set. Repeating the current status is rejected. Deactivation and reactivation preserve identity.

## G. Customer Domain Rules

Customer rules mirror the supplier foundation where appropriate. Customer operations preserve codes and UUIDs, normalize optional text, reject invalid names/emails/pagination, protect edits and status changes with revisions, and retain inactive identities.

## H. Supplier Search / Filtering / Pagination

SQLite search covers code, display name, contact person, both phone fields, email, and tax/registration identifier. Active/inactive filtering, total count, limit, and offset execute in the backend. Ordering is deterministic by case-insensitive display name and UUID. Limits are restricted to 1–100.

## I. Customer Search / Filtering / Pagination

SQLite search covers code, display name, both phone fields, and email. Active/inactive filtering, total count, limit, offset, and deterministic ordering execute in the backend.

## J. Supplier IPC

The centralized TypeScript client exposes typed list/get/create/update/status operations. Registered Tauri commands use the `canonical_supplier` namespace and immediately delegate to Rust domain functions.

## K. Customer IPC

The centralized TypeScript client exposes typed list/get/create/update/status operations. Registered Tauri commands use the `canonical_customer` namespace and immediately delegate to Rust domain functions.

## L. Suppliers UI

The native `/suppliers` route now uses canonical IPC for loading, populated, empty, filtered-empty, error/retry, pagination, create, detail, edit, activate, and deactivate states. The form includes all canonical supplier fields. The history panel explicitly says no linked workflow is active.

## M. Customers UI

The native `/customers` route uses canonical IPC for the same operational states with a customer-specific form and detail view. It contains no fake balances, loyalty information, or purchase history.

## N. Validation

Frontend required-name handling improves form feedback. Rust remains authoritative for required names, maximum lengths, null normalization, email shape, UUIDs, safe pagination, unique values, revision conflicts, and status transitions. Errors use the existing structured `AppError` codes.

## O. Transactions / Concurrency

Creation uses an IMMEDIATE transaction. Independent supplier/customer sequence rows generate deterministic codes atomically. Failed unique inserts roll back their sequence increment. Database uniqueness protects concurrent writes. Updates and status changes use a single revision-guarded statement. File-backed concurrent creation is covered by automated tests.

## P. Sales Compatibility

Migration 009 and canonical Sales were not changed. The existing nullable `customer_id` check remains intentionally unlinked, while sale-time `customer_name` and item snapshots remain authoritative. A later controlled integration can attach customer IDs without making historical receipts depend on mutable customer records.

## Q. Future Purchasing Compatibility

Stable supplier UUIDs and immutable codes are ready for future foreign-key references. Phase 5A adds no purchase orders, lines, receiving, partial receiving, invoices, supplier returns, payments, or inventory posting paths. Existing prototype purchase handlers remain unregistered.

## R. Rust Tests

The new canonical partner suite contains 17 tests. It covers clean migration, 009→010 upgrade, apply-once behavior, rollback, Phase 4 data preservation, complete field persistence, null normalization, independent and concurrent code generation, uniqueness rollback, validation, structured not-found errors, stale updates, activation/deactivation/reactivation, search fields, filtering, pagination, ordering, row retention, and Sales schema compatibility.

Full Rust result: **357 passed, 0 failed**.

## S. Frontend Tests

The new canonical partner suite contains 21 tests. It covers route/IPC isolation, supplier/customer rendering, loading, empty, filtered-empty, retryable errors, backend search/filter/pagination requests, full create payloads, validation, error handling, detail, honest history state, revision-aware edit/status operations, and focus behavior.

Full frontend result: **159 passed, 0 failed**.

## T. Automated Viewport Tests

The canonical partner list passed at 1366×768, 1440×900, and 1920×1080 without page-level horizontal overflow. The minimum-size supplier dialog focuses the name field and retains access to the save action through its existing internal scrolling.

## U. Regression Status

The full Rust and Playwright suites pass. Catalog, imports, canonical Inventory and ledger behavior, documents, counts, alerts, FEFO, Sales/POS, holds, posting, receipts, returns, refunds, voids, history, and idempotency suites remain green. No second ledger or alternate stock posting path was added.

Automated checks:

- `cargo fmt --check`: pass
- `cargo check`: pass
- `cargo test`: 357 passed, 0 failed
- `npm run lint`: pass
- `npm run build`: pass
- `npm test`: 159 passed, 0 failed

## V. Files Changed

Phase 5A files are listed in `docs/phase-5a/files.txt`. Pre-existing Phase 4 working-tree changes were preserved and are outside this phase.

## W. Known Limitations

- Customers are deliberately not linked to Sales yet; immutable Phase 4 snapshots remain unchanged.
- Supplier transaction history is unavailable until purchasing exists.
- The obsolete prototype partner/purchase source and tests remain in the repository but are not registered or used by canonical native routes.
- No full audit-event subsystem was added; stable IDs, revisions, status, and timestamps preserve the foundation for a later audit phase.

## X. Real Development DB Status

The real development database was not opened, read, migrated, seeded, or modified during Phase 5A. Migration 010 was exercised only with in-memory and temporary test databases. The verified Phase 4 stock and Catalog baseline was not used as test data.

## Y. Native Testing Status

No native application was launched. `npm run tauri dev`, manual UI interaction, native smoke testing, and real-app restart testing were not performed. All UI verification used mocked IPC in headless browser automation.

## Z. Phase 5A Status

COMPLETE

## AA. Recommended Phase 5B Plan

Design a new sequential purchasing migration that references canonical supplier UUIDs and preserves purchase-time supplier and product snapshots. Implement draft purchase documents, validation, and idempotency first; then add controlled receiving through the existing canonical Inventory document/ledger path, partial receiving rules, supplier traceability, and isolated Rust/frontend tests. Keep purchase returns and more advanced settlement concerns in explicitly scoped later subphases if Phase 5B is divided further.
