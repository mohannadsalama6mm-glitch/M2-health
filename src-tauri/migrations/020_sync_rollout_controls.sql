-- Phase 8C makes the local transport gate explicitly activatable while
-- preserving its fail-closed value. Activation remains an external approval.
ALTER TABLE sync_schema_metadata RENAME TO sync_schema_metadata_phase8b;
CREATE TABLE sync_schema_metadata (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 protocol_version INTEGER NOT NULL CHECK(protocol_version>0),
 transport_enabled INTEGER NOT NULL DEFAULT 0 CHECK(transport_enabled IN (0,1)),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
INSERT INTO sync_schema_metadata(singleton,protocol_version,transport_enabled,updated_at)
SELECT singleton,protocol_version,0,updated_at FROM sync_schema_metadata_phase8b;
DROP TABLE sync_schema_metadata_phase8b;
