# Phase 2A implementation report

Date: 2026-09-12. Project: M² Health. Phase 1 UI and demo workflows preserved.

## A. Phase 2A Summary

Implemented the Tauri/Rust/SQLite foundation in source, versioned migrations, structured errors, branch repository/commands, centralized frontend IPC and native topbar integration. Native compilation, runtime and real-file persistence verification are blocked by Windows Application Control preventing the installed Cargo executable from running. This is not a verified desktop release.

## B. Tauri Setup

- Installed JavaScript API 2.11.1 and CLI 2.11.4. Rust `tauri` and `tauri-build` declare major version 2; exact resolved crate versions are unavailable until Cargo succeeds.
- `npm run dev` keeps the browser frontend; `npm run tauri dev` launches the desktop development workflow; `npm run tauri build -- --no-bundle` is the production executable workflow.
- Added `src-tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`, main-window capability and branded generated icons using the existing vector logo.
- Main window: 1440×900, minimum 1000×680; existing Vite app on port 1420 and production assets from `../dist`.
- Production CSP permits local assets and IPC. Development CSP additionally permits the local Vite/HMR connection. Installer bundling is disabled; no installer/signing configuration is claimed.

## C. Rust Architecture

`main.rs` delegates to `lib.rs`. Startup resolves storage, initializes the database, runs migrations, ensures a default branch and registers managed state and commands. Database initialization failures are retained as structured state so the frontend can render an error without terminating the entire window. `AppDb` owns one mutex-protected connection; repositories own SQL; handlers are thin wrappers. Runtime paths use recoverable errors rather than `unwrap`/`expect`.

## D. SQLite Setup

`rusqlite` 0.40 with `bundled` SQLite; no separate database server. Database path is resolved through Tauri `app_local_data_dir()`, followed by `m2-health.db`. Expected Windows path: `%LOCALAPPDATA%\com.m2health.pharmacy\m2-health.db`. Debug startup logs the actual path. Foreign keys ON, file journal WAL, busy timeout 5000 ms. In-memory tests omit WAL. Application data was not manually created or modified during testing.

## E. Migration System

Declared schema version: 1. Migration: `001_branches.sql`. `_schema_version` stores version, name and application timestamp. Definitions must be sequential; incompatible/newer metadata is rejected. An IMMEDIATE transaction serializes migration runners and atomically applies pending migrations plus metadata. Failures roll back without deleting/recreating the database.

## F. Initial Schema

Only `branches` is a domain table. Fields: `id` (UUID string), `code` (unique, case-insensitive), `name`, `phone`, `address`, `is_active`, `created_at`, `updated_at`. UTC timestamp defaults and field/check constraints are included. No products, stock, sales, sync or account tables were added. No update command exists yet; a future update must explicitly maintain `updated_at`.

## G. Branch Repository / Commands

`list_branches`, `get_branch`, `create_branch`, `ensure_default_branch`. Inputs are validated and SQL values parameterized. Creation uses UUID v4. Default creation runs in an IMMEDIATE transaction: empty table produces exactly one MAIN / Main Pharmacy; subsequent calls return an existing active record ordered by creation time and ID. All-inactive data is preserved without reactivation or duplicate creation. Errors distinguish database, migration, validation, notFound, conflict, io and internal failures.

## H. Frontend IPC Layer

`src/lib/tauri/client.ts` is the only invoke boundary. It checks the native runtime and normalizes structured errors. `branches.ts` provides typed branch functions and camelCase DTOs matching Rust serialization. No browser API fallback or backend service was added.

## I. UI Integration

Only the topbar Branch selector reads native branch data. Loading disables the selector; success displays the local branch; errors remain scoped to the selector with an accessible message and retry control. Retry repeats commands; fixing an initialization failure requires restarting the app. Selection is session-only. Browser previews retain demo selection and notifications. Branch management forms, dashboard metrics and every other module remain Phase 1 demos.

## J. Tests Added

12 Rust tests cover fresh schema/metadata/pragmas, repeatable migrations, transactional rollback on existing and fresh databases, incompatible schema refusal, CRUD foundation/validation/serialization, default idempotency, active preference, temporary-file reopen/WAL persistence, concurrent default creation, foreign-key enforcement and corrupt-file preservation. These tests are written but not executed because Cargo is blocked. They use in-memory databases and isolated temporary files, never real application data.

Four new Playwright tests cover native-boundary success, loading, error/retry and unchanged browser demo branches. The original 23 tests remain intact. IPC doubles exist only in tests and do not exercise SQLite or Rust.

## K. Verification Results

- `npm run lint`: passed.
- `npm run build`: passed, including `tsc --noEmit` and Vite production output.
- `npm test`: passed, 27 tests in 56.1 seconds, exit code 0. The first sandboxed run passed all assertions but stalled during server cleanup; the completed run was outside the sandbox.
- Direct installed `rustfmt --edition 2021 --check` on entrypoints/build/tests and their modules: passed.
- `cargo fmt --check`, `cargo check`, `cargo test`: blocked by Windows Application Control.
- `npm run tauri dev`: blocked at `cargo metadata`; no native window launched.
- `npm run tauri build -- --no-bundle`: blocked at `cargo metadata`; no executable produced.
- Tauri icon generation: passed.
- Isolated in-memory SQLite SQL smoke check: migration syntax, case-insensitive unique code and UTC timestamp default passed. This does not substitute for Rust tests or native persistence verification.
- App-data database creation, migrations at native startup, actual IPC, window rendering, restart persistence and duplicate prevention have not been runtime-verified.

## L. Packages / Crates Added

JavaScript: `@tauri-apps/api` ^2.11.1 and development `@tauri-apps/cli` ^2.11.4. Rust declarations: `tauri` 2, `tauri-build` 2, `serde` 1/derive, `rusqlite` 0.40/bundled, `uuid` 1/v4. Test-only: `tempfile` 3 and `serde_json` 1. npm installation reported zero audit vulnerabilities. No additional system software was installed.

## M. Files Added / Changed

Added the native manifest/configuration/build script, capabilities, icon source/generated icons, main/lib/error modules, commands modules, database connection/migration/model/repository modules, migration SQL and Rust integration tests under `src-tauri/`. Added `src/lib/tauri/client.ts`, `src/lib/tauri/branches.ts`, `src/app/BranchSelector.tsx`, `tests/native-boundary.spec.ts` and this report. Updated `src/app/AppShell.tsx`, `src/app/desktop.css`, `package.json`, `package-lock.json`, `.gitignore` and `README.md`. Existing Phase 1 test files and screen implementations were preserved.

## N. Issues Found / Fixed

Separated native branch UUIDs from incompatible Phase 1 fixture identities, preventing demo workflow changes. Kept initialization errors recoverable for the UI. Serialized default creation and migration execution to avoid cross-connection races. Added local HMR allowance without opening production network access. Fixed explicit connection-guard borrowing in repository calls. Formatted Rust directly because Cargo cannot launch.

The environment has stable MSVC Rust 1.98.1 and Visual Studio 18 Community installed. Windows reports: “An Application Control policy has blocked this file” for the installed `cargo.exe`, with Tauri reporting OS error 4551. The same failure occurs outside the sandbox. No allowlist/policy change or executable workaround was attempted.

## O. Remaining Limitations

An administrator must permit the installed Cargo through the managed application-control policy before native checks can proceed. Then resolve dependencies and retain `Cargo.lock`, run formatting/check/tests and both native workflows, inspect the debug database path, restart twice and confirm one migration record and one unchanged default branch. Native compilation may reveal additional issues; installed C++/SDK/WebView2 prerequisites cannot be considered end-to-end verified until then. This phase has no installer, authentication, encryption, production pharmacy workflows or sync. Phase 2B has not started.

## P. Phase 2A Status

NOT COMPLETE

## Q. Recommended Phase 2B Scope

After Phase 2A native verification succeeds, design the Catalog domain model: Products, Product Packages, Barcodes, Manufacturers, Categories, Dosage Forms, Active Ingredients and Price History. Define identities, package/unit relationships, uniqueness, branch scope and migration boundaries before connecting real Catalog workflows. No Phase 2B implementation was performed.

Implementation references: [Tauri configuration](https://v2.tauri.app/reference/config/), [calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/), [rusqlite connection API](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html).
