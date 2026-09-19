# Disabled migration proposals — recovery 2026-09-13

## Phase 3C update — 2026-09-14

Canonical migration 006_stock_counts and `inventory/counts` now provide real count
sessions and current-live reconciliation. The original prototype counts remain
unregistered. Runtime stockCountsEnabled gates the new count-session routes only;
the old count URL redirects to those routes. Active migrations are 001–006.
Phase 3D and later workflows are not enabled. The chronological notes below retain
their historical context; see ../phase-3c-policy.md for the implemented policy.

## Phase 3B update — 2026-09-14

The active sequence now contains canonical migrations 001–005. The new
`src-tauri/migrations/005_inventory_workflows.sql` was rebuilt and tested for
canonical documents, atomic posting, immutable traceability, durable retries and
compensating reversals. The proposal in this directory remains unregistered.
Only the new inventory document capability is enabled; counts, sales, purchases
and partner prototypes remain disabled. Future migrations must follow 005.
The sections below preserve the recovery checkpoint and its rationale.

Only `src-tauri/migrations/001_branches.sql` through `004_inventory_core.sql`
are registered. Their names, versions and contents are preserved. No new schema
is applied during recovery. These files have no migration version until reviewed
and integrated; filename ordering here is not an execution sequence.

- `obsolete_inventory.sql` (formerly 003_inventory): **OBSOLETE / COMPETING**.
  Creates an incompatible inventory_batches and a second stock_movements ledger.
  Preserved only as evidence and an isolated in-memory prototype-test fixture.
  Never register it or apply it to a user database.
- `inventory_workflows.sql` (formerly 005_inventory_workflows): **NEEDS REWORK**.
  References the canonical ledger, but document service/lifecycle/idempotency
  integration is not complete. Retained as a proposal, not READY TO APPLY.
- Stock counts (embedded in obsolete_inventory.sql): **NEEDS REWORK**.
  Must be extracted and integrated with canonical batches/movements, explicit
  counting/review/cancellation and safe reconciliation before migration approval.
- `sales.sql` (formerly 004_sales): **NEEDS REWORK**.
  Requires canonical ledger allocation, safe refunds/costs/idempotency and partner
  dependency ordering. No sale posting command is registered.
- `partners_purchases.sql` (formerly 005_partners_purchases): **NEEDS REWORK**.
  Partner schema should precede dependent sales/purchases; purchase receipts need
  canonical document/batch integration. No partner/purchase commands are registered.

Future ordering must be selected from actual dependencies, using sequential
versions after 004. None of these proposals is being blindly renumbered/applied.

## Test interpretation

`tests/inventory_core.rs` restores the 25 Phase 3A tests from commit 2bec692 and
adds canonical allocation coverage. `tests/recovery.rs` verifies the real active
migration chain. Catalog/import/foundation tests also use the active registry.

The existing later Inventory, Sales, Purchases and Partners contracts remain
enabled, with no assertions removed, against explicitly constructed **in-memory
prototype schemas**. The file-backed restart test now exercises canonical APIs.
Prototype test success does not mean those workflows are compatible or enabled.
`AppDb::open` always uses the canonical migration registry; the fixture constructor
cannot open a file or be invoked through Tauri.

## Runtime boundary

`src/lib.rs` registers Catalog/import, Phase 3A inventory, recovery status and the
canonical read-only FEFO adapter only. The later command/service source remains
for follow-up integration but is unreachable from the application's IPC handler.
The React recovery boundary fails closed and never mounts disabled workflows.
Browser prototype tests explicitly advertise prototype capability in their test
double; production recovery status is false and has no writable toggle.

Counts list SQL has been repaired by retaining aliases inside the filtered
subquery. REVIEW/CANCELLED, uncounted markers and stale-snapshot reconciliation
remain unresolved; no stock count commands are enabled.

## Existing CSV blocker

Before recovery, the checked-in CSV had already become a 26,437,838-byte padded
table, SHA-256 `5f31d97c3491dd75bae780bba8ce52bb8e33bf81a7f0f085884159f08d38de41`.
It differs from the verified source hash in catalog-reference-profile.json.
The user explicitly requested leaving it unchanged and reporting this blocker.
No CSV restoration, import application or provenance rewrite is part of recovery.
