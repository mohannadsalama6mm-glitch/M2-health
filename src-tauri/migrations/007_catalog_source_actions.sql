-- New source runs supplement, never reinterpret, the original import provenance.
CREATE TABLE catalog_source_runs (
 run_id TEXT PRIMARY KEY NOT NULL REFERENCES catalog_import_runs(id) ON DELETE RESTRICT,
 source_version TEXT NOT NULL,
 reused_rows INTEGER NOT NULL DEFAULT 0 CHECK(reused_rows>=0),
 updated_rows INTEGER NOT NULL DEFAULT 0 CHECK(updated_rows>=0),
 malformed_rows INTEGER NOT NULL DEFAULT 0 CHECK(malformed_rows>=0),
 quoting_rows INTEGER NOT NULL DEFAULT 0 CHECK(quoting_rows>=0),
 ambiguous_rows INTEGER NOT NULL DEFAULT 0 CHECK(ambiguous_rows>=0),
 duplicate_rows INTEGER NOT NULL DEFAULT 0 CHECK(duplicate_rows>=0),
 invalid_price_rows INTEGER NOT NULL DEFAULT 0 CHECK(invalid_price_rows>=0),
 failed_rows INTEGER NOT NULL DEFAULT 0 CHECK(failed_rows>=0),
 backup_at TEXT NOT NULL,
 backup_size INTEGER NOT NULL CHECK(backup_size>0)
) STRICT;
CREATE TABLE catalog_source_rows (
 run_id TEXT NOT NULL REFERENCES catalog_source_runs(run_id) ON DELETE RESTRICT,
 source_row INTEGER NOT NULL CHECK(source_row>0),
 fingerprint TEXT NOT NULL,
 identity_key TEXT NOT NULL,
 action TEXT NOT NULL,
 reason TEXT,
 product_id TEXT REFERENCES products(id) ON DELETE RESTRICT,
 package_id TEXT REFERENCES product_packages(id) ON DELETE RESTRICT,
 price_history_id TEXT REFERENCES product_price_history(id) ON DELETE RESTRICT,
 PRIMARY KEY(run_id,source_row)
) STRICT;
CREATE INDEX catalog_source_fingerprint ON catalog_source_rows(fingerprint);
CREATE TRIGGER catalog_source_rows_no_update BEFORE UPDATE ON catalog_source_rows
BEGIN SELECT RAISE(ABORT,'Source action provenance is immutable'); END;
CREATE TRIGGER catalog_source_rows_no_delete BEFORE DELETE ON catalog_source_rows
BEGIN SELECT RAISE(ABORT,'Source action provenance is immutable'); END;
