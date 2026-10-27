# Desktop Product Closure Gate completion report

Date: 2026-10-20

## Result

**Desktop Product Closure Gate: COMPLETE**

The released desktop route surface is connected to canonical SQLite-backed commands, the browser surface is explicit about preview-only or desktop-required behavior, IPC authorization is enforced, and the isolated pharmacy journey passes. Historical Phases 1–8B remain unchanged; this gate is not a replacement Phase 1.

## Preserved implementation

- Canonical Dashboard, Reports, Finance, Sales, Purchasing, Partners, branches, notifications, audit, backup/restore, and settings routes were retained and verified.
- Tauri registration and the centralized command-policy boundary were retained. Missing, invalid, and revoked sessions, forged actor fields, record-owned branch access, branch membership, all-branch scope, and the desktop-closure commands are covered by authorization tests.
- Migration `022_desktop_product_closure.sql` remains additive and last. It creates only the persisted singleton local settings record.
- The complete route-to-data-source matrix and browser behavior are recorded in `ROUTE-AUDIT.md`.

## Repairs made during verification

- Restore now validates SQLite integrity, foreign keys, sequential recognized migration history, and required schema objects before replacement and after migration.
- Restore failure now reapplies and validates the automatic safety backup. It claims that live data was retained only when rollback validation succeeds; otherwise it preserves and identifies the safety backup.
- Focused restore tests now cover forged migration history and a forced post-restore failure with verified rollback.
- The isolated pharmacy journey now covers catalog, purchase order, receipt, stock, sale, customer return, supplier return, count/adjustment, reports, finance, backup, mutation, restore, and safety-backup creation.
- Browser receiving no longer reports a simulated successful receipt. Browser operational routes explicitly require the desktop runtime and do not simulate local persistence.
- Stale migration-count tests now derive their expected version from the registered migration list, so additive migration 022 is tested without rewriting historical expectations.

## Migration and data safety evidence

- Byte-for-byte SHA-256 comparison against `HEAD` confirmed migrations `001` through `020` are unchanged.
- Existing migration `021_sync_conflict_resolution.sql` remains the predecessor; `022_desktop_product_closure.sql` is forward-only.
- `.gitignore` excludes `*.db`, `*.db-wal`, `*.db-shm`, `backups/`, `*.local`, `config/cloud.local.json`, and `supabase/.temp/`.
- Every Rust workflow used an in-memory database or a temporary database under `H:\M2-health\tmp\rust-test-temp`.
- The real development SQLite database was not opened, inspected, copied, seeded, migrated, or reset.

## Verification evidence

All commands were run from the current working tree on 2026-10-20.

- `cargo fmt --all -- --check`: passed.
- `cargo check --message-format short`: passed with `CARGO_BUILD_JOBS=1` and project-local `CARGO_TARGET_DIR`; only the known H-drive incremental hard-link warning was emitted.
- `cargo test -- --test-threads=1`: passed across every Rust test binary with `TEMP` and `TMP` set to the isolated workspace temp directory. The focused end-to-end suite passed 3 tests and the offline-sync suite passed 21 tests.
- `npm run lint`: passed.
- `npm run build`: passed; Vite emitted only its non-failing chunk-size advisory.
- `npm test -- --reporter=line`: 304 passed. The sandboxed attempt could not bind the local Vite socket; the approved local rerun passed and this was an environment restriction, not a product/test failure.
- `git diff --check`: passed; Git emitted line-ending conversion warnings only.

## Acceptance journey

The isolated acceptance test proves:

`admin/login -> catalog -> purchase order -> receipt -> stock -> sale -> return -> count/adjustment -> reports/finance -> backup -> restore`

It uses only generated identities and a temporary SQLite database. Backup restore includes exact confirmation, source validation, automatic safety backup, post-restore validation, and a tested rollback path.

## Scope and next gate

- Cloud transport remains disabled by all independent gates.
- Phase 9A/9B, Phase 10, website, payments, subscriptions, commercial licensing, installer rollout, and updater deployment were not started.
- Phase 8C resumed only after the desktop gate was proven. Its credential incident, unapplied server hardening, live reruns, and incomplete document/count replication remain separate blockers; see `docs/phase-8c/FINAL-REPORT.md`.
