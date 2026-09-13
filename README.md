# M² Health

Smart Pharmacy Management System — Phase 1 UI preserved, with the Tauri/Rust/SQLite foundation and Phase 2B Catalog domain. See [Catalog architecture](docs/CATALOG-ARCHITECTURE.md) and the [Phase 2B report](docs/PHASE-2B-REPORT.md). Catalog UI integration and bulk import are deferred.

## Run locally

Node.js 22.12+ and npm are required (development verified with Node 24.18).

```powershell
cd 'D:\project\M2 health'
npm install
npm run dev
```

Open http://127.0.0.1:1420/dashboard. The development-only component showcase is at `/design-system`; it is excluded from normal pharmacy navigation and production routing.

```powershell
npm run lint
npm run typecheck
npm run build
npm test
npm run format
```

Browser tests use installed Microsoft Edge through Playwright. `npm run preview` serves the production build. If port 1420 is occupied by this project's existing Vite server, use that server rather than starting another instance.

## Product workspace

- Operations: dashboard, POS, sale details, held baskets, purchases, purchase drafts and receiving.
- Catalog and stock: product list/details/add/edit, inventory, movement ledger, counts and expiry review.
- Relationships: supplier and customer directories, details, contacts and history.
- Management: reports, finance shell, employees, permissions and branches.
- System: notifications, audit details, backup/restore, synchronization scenarios and settings.

The application owns its viewport. Main content scrolls independently of the sidebar and topbar. The sidebar navigation scrolls internally when needed. POS has independent product and cart scrolling, with visible checkout controls at the target desktop sizes.

## Current Catalog and import behavior

The desktop Catalog now uses real typed Tauri commands and local SQLite for list/search/filter/pagination, details, products, packages, lookups, barcodes and price history. **Products → Import Products** provides the controlled canonical Egyptian Catalog import with a read-only dry run, confirmation, online SQLite backup, transactional batches and persistent provenance. See [the import guide](docs/CATALOG-IMPORT.md) and [Phase 2D verification](docs/PHASE-2D-REPORT.md). The development import created 25,061 products, packages and initial prices; nine source rows were skipped. There is no automatic startup import.

Phase 3A adds the real Inventory backend: branch/package/batch ownership, append-only movements, derived balances, opening/adjustments/write-offs, atomic transfers, reorder levels and expiry/FEFO queries. See [Inventory Core](docs/INVENTORY-CORE.md) and [Phase 3A verification](docs/PHASE-3A-REPORT.md). The existing Inventory UI remains a Phase 1 preview. One clearly labelled development batch retains a test balance of 8 after +10 opening and -2 adjustment.

Other operational workspaces still use Phase 1 UI fixtures. Full receiving/transfer workflows, POS/purchases persistence, authentication and synchronization are not implemented. The notes below describe earlier milestones.

## Demo behavior

Contact changes, purchase drafts, receiving status, sales and held baskets still use an in-memory React context. Reloading restores fixtures. Other screen-specific previews, such as permission switches or settings, are local component state and may reset when leaving the screen. The desktop Branch selector and Catalog use Rust/SQLite; browser Catalog routes explain that the desktop app is required. Browser previews retain the original demo branch selector. No authentication, cloud service or real pharmacy transaction is implemented.

Operational fixtures are centralized in `src/mock/`. The original dashboard's illustrative summary fixtures remain in `src/data/demo.ts`; aggregate dashboard figures are not a live roll-up of the small catalog sample. Branch selection is a UI context, not isolated branch databases. Transfers, write-offs, posting, receipts, backups and synchronization actions explicitly remain previews. Report dates and finance sections do not run reporting or accounting engines.

Multiple Catalog packages, barcodes and ingredients persist through native editing. SQLite enforces barcode uniqueness, but does not validate registered pharmacy identifiers. The decorative BarcodeDisplay is not scannable. POS accepts demo barcode text through a keyboard-style input.

## Structure

- `src/app/`: viewport shell, routes, navigation and in-memory DemoContext.
- `src/design-system/tokens/`: semantic color, type, spacing, radius, shadow, layer, motion and control tokens.
- `src/design-system/components/`: controls, overlays and pharmacy-specific primitives.
- `src/design-system/patterns/`: reusable Logo/background, list/detail/form/workflow/state patterns.
- `src/mock/`: typed Phase 1B fixtures; `src/data/`: original dashboard fixtures.
- `src/features/`: dashboard, catalog, sales, inventory, purchases, relationships, management, reports, finance, system and internal showcase.
- `tests/`: route/viewport, keyboard and demo-workflow regressions.
- `docs/PHASE-1B-REPORT.md`: final continuation report and scope boundaries.
- `docs/PHASE-1-REPORT.md`: historical Phase 1A handoff.

## Desktop runtime — Phase 2A

The desktop foundation uses Tauri v2 → Rust → bundled SQLite. SQLite is the operational local database; no separate SQLite server or installation is required. Supabase, authentication and synchronization are intentionally deferred.

Windows development requires a permitted stable Rust MSVC toolchain, Visual Studio C++ build tools/Windows SDK, and WebView2. Cargo is now working, and native checks run successfully. The earlier Phase 2A report records a historical application-control blocker that no longer applies. Desktop execution requires access to its normal application-data directory; a restricted agent sandbox may prevent that access.

```powershell
npm run tauri dev
npm run tauri build -- --no-bundle
```

Tauri starts Vite automatically on port 1420. Stop an independently running Vite server first so the before-dev command can own that port. The production command builds the executable; installer bundling/signing is not configured. The existing frontend build stays available through `npm run build`.

Startup resolves `app.path().app_local_data_dir()` using application identifier `com.m2health.pharmacy`, creates that directory, opens `m2-health.db`, applies migrations and ensures one default branch. On Windows the expected location is `%LOCALAPPDATA%\com.m2health.pharmacy\m2-health.db`; the actual resolved path is logged in debug builds. No user-specific path is hardcoded. WAL may create adjacent `-wal` and `-shm` files. Never delete these files to recover from an initialization/migration error.

The connection enables foreign keys, WAL for file databases and a 5-second busy timeout. One managed `AppDb` owns a mutex-protected connection. Versioned SQL lives in `src-tauri/migrations/`; `_schema_version` records successful versions. Pending migrations and metadata updates run in an immediate transaction. Failures roll back; unknown/newer schemas are refused without deleting user data.

Schema version 1 adds `branches`: UUID identity, unique case-insensitive code, name, contact details, active status and UTC creation/update timestamps. `ensure_default_branch` creates MAIN / Main Pharmacy only for an empty table and serializes concurrent callers. Otherwise it returns an existing active branch, preferring the oldest stable record. If all records are inactive, it returns one without silently reactivating it or adding a duplicate. Migration 002 adds the normalized Catalog model and preserves existing branches. Migration 001 remains unchanged.

The topbar uses `src/lib/tauri/branches.ts`; all `invoke` calls pass through `client.ts`. Native loading and errors stay within the selector. Retry repeats commands; a retained startup initialization failure requires restarting the app after fixing the cause. Selection is session-only and does not filter or persist changes to the Phase 1 demo modules. The Branches management screen remains a demo; its forms do not create real SQLite branches.

```powershell
Push-Location src-tauri
cargo fmt --check
cargo check
cargo test
Pop-Location
```

Rust tests use in-memory SQLite or temporary directories only. Frontend tests mock only the native IPC boundary. Neither suite targets the real application database. `src-tauri/Cargo.lock` is retained. Current native verification checks schema version 4, the unchanged branch/Catalog, one controlled import run, and one Inventory smoke batch with two immutable movements.

Architecture: `src-tauri/src/commands/` contains thin IPC handlers; `db/connection.rs` owns connection setup; `db/migrations.rs` applies migrations; `db/models/` defines DTOs; `db/repositories/` contains branch SQL; `error.rs` provides safe structured errors. No filesystem, SQL, shell or network plugin is exposed to the frontend.

## Catalog reference and Phase 2B

The in-project `egyptian-drugs.csv` is the canonical raw Egyptian catalog source. Its location is defined in `config/catalog-source.json`, consumed by the Rust importer. It informed bilingual naming, optional relationships, source route labels and exact EGP price handling. The raw file, normalized SQLite domain, and React UI remain separate layers. No real Catalog import runs during startup or automated testing, and the raw CSV is not imported into the frontend JavaScript bundle.

```powershell
python scripts/inspect_catalog_reference.py
```

This read-only utility resolves the canonical source configuration, prints JSON and never opens a database. Pass `--output <report.json>` to save a profile deliberately; output cannot overwrite the source file. Six representative rows from this profile exercise the Rust model in isolated tests. The model separates logical Products from sellable ProductPackages; barcodes and integer-minor-unit price history belong to packages. Scientific display text is preserved separately from curated ingredient relationships. Catalog commands and TypeScript DTO contracts power the native Catalog screens. SQLite remains the local operational source; Supabase remains deferred.

Fonts are locally bundled. The editable shared logo reproduces the supplied emerald M, clear raised 2 and integrated medical cross. Full Arabic translation, actual authorization, native hardware, fiscal receipts and clinical/operational validation remain later work.
