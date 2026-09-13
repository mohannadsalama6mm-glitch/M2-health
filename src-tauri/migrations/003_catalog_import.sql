CREATE TABLE catalog_import_runs (
 id TEXT PRIMARY KEY NOT NULL,
 source_hash TEXT NOT NULL,
 source_path TEXT NOT NULL,
 started_at TEXT NOT NULL,
 finished_at TEXT,
 total_rows INTEGER NOT NULL,
 imported_rows INTEGER NOT NULL DEFAULT 0,
 status TEXT NOT NULL CHECK(status IN ('applying','complete','failed')),
 report_path TEXT NOT NULL,
 backup_path TEXT NOT NULL
) STRICT;
-- Committed provenance is atomic with the product/package/initial price.
CREATE TABLE catalog_import_rows (
 source_id TEXT NOT NULL,
 fingerprint TEXT NOT NULL,
 identity_key TEXT NOT NULL,
 run_id TEXT NOT NULL REFERENCES catalog_import_runs(id),
 source_row INTEGER NOT NULL,
 product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
 package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
 package_placeholder INTEGER NOT NULL DEFAULT 1 CHECK(package_placeholder=1),
 PRIMARY KEY(source_id,fingerprint)
) STRICT;
