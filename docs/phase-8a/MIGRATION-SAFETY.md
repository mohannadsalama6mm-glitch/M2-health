# Migration Safety

SQLite migrations 001–017 remain byte-for-byte equal to the recorded Phase 7A/7C hashes. Migration 018 is forward-only and registered as `cloud_foundation`; its SHA-256 is `ceb05b972182d4747b5977d9b25a4db97f586c50402b1d27744b0ace14d09df8`.

Automated isolated tests cover fresh initialization at v018, repeated runner idempotency, upgrade from a populated v017 fixture, preservation of the existing branch, no automatic tenant adoption, foreign-key enforcement, tenant/branch composite constraints, and complete rollback of a failing synthetic v019 migration. The existing future-version rejection also moves to v019.

The real development database was not opened, read, copied, migrated, seeded, or reset. Its last externally reported version remains v009. Startup of the native application was prohibited and did not occur.

The PostgreSQL migration is stored separately under `supabase/migrations`. It is not registered in SQLite and was not executed because no isolated PostgreSQL runtime is installed. Applying it later requires a dedicated test project and the RLS procedure in `POSTGRES-RLS.md`.
