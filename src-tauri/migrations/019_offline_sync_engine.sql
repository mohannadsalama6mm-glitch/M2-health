-- Phase 8B durable engine state. Transport remains gated until Phase 8C.
CREATE TABLE sync_local_sequence (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 next_value INTEGER NOT NULL CHECK(next_value>0)
) STRICT;
INSERT INTO sync_local_sequence(singleton,next_value) VALUES(1,1);

ALTER TABLE sync_outbox ADD COLUMN local_sequence INTEGER;
ALTER TABLE sync_outbox ADD COLUMN payload_hash TEXT CHECK(payload_hash IS NULL OR length(payload_hash)=64);
ALTER TABLE sync_outbox ADD COLUMN lease_owner TEXT;
ALTER TABLE sync_outbox ADD COLUMN lease_expires_at TEXT;
ALTER TABLE sync_outbox ADD COLUMN last_attempt_at TEXT;
CREATE UNIQUE INDEX sync_outbox_local_sequence ON sync_outbox(local_sequence) WHERE local_sequence IS NOT NULL;
CREATE INDEX sync_outbox_retry ON sync_outbox(acknowledged_at,next_attempt_at,lease_expires_at,local_sequence);

CREATE TRIGGER sync_outbox_engine_fields BEFORE INSERT ON sync_outbox
WHEN NEW.local_sequence IS NULL OR NEW.payload_hash IS NULL
BEGIN SELECT RAISE(ABORT,'sync engine sequence and payload hash required'); END;
CREATE TRIGGER sync_outbox_engine_envelope_immutable BEFORE UPDATE OF local_sequence,payload_hash ON sync_outbox
BEGIN SELECT RAISE(ABORT,'sync event envelope is immutable'); END;

ALTER TABLE sync_inbox ADD COLUMN server_sequence INTEGER CHECK(server_sequence IS NULL OR server_sequence>=0);
ALTER TABLE sync_inbox ADD COLUMN device_id TEXT;
ALTER TABLE sync_inbox ADD COLUMN actor_user_id TEXT;
ALTER TABLE sync_inbox ADD COLUMN entity_type TEXT;
ALTER TABLE sync_inbox ADD COLUMN entity_id TEXT;
ALTER TABLE sync_inbox ADD COLUMN operation TEXT;
ALTER TABLE sync_inbox ADD COLUMN payload_json TEXT CHECK(payload_json IS NULL OR json_valid(payload_json));
ALTER TABLE sync_inbox ADD COLUMN schema_version INTEGER CHECK(schema_version IS NULL OR schema_version>0);
ALTER TABLE sync_inbox ADD COLUMN causation_id TEXT;
ALTER TABLE sync_inbox ADD COLUMN occurred_at TEXT;
ALTER TABLE sync_inbox ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 0 CHECK(attempt_count>=0);
CREATE UNIQUE INDEX sync_inbox_server_sequence ON sync_inbox(organization_id,server_sequence) WHERE server_sequence IS NOT NULL;
CREATE TRIGGER sync_inbox_envelope_immutable BEFORE UPDATE OF
 branch_id,payload_hash,server_sequence,device_id,actor_user_id,entity_type,entity_id,
 operation,payload_json,schema_version,causation_id,occurred_at ON sync_inbox
BEGIN SELECT RAISE(ABORT,'sync inbox envelope is immutable'); END;

CREATE TABLE sync_conflicts (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id)=36),
 organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 event_id TEXT NOT NULL,
 entity_type TEXT NOT NULL,
 entity_id TEXT NOT NULL,
 reason_code TEXT NOT NULL CHECK(reason_code IN ('UNSUPPORTED_ENTITY','MISSING_DEPENDENCY','REVISION_CONFLICT','IMMUTABLE_CONFLICT','INVALID_PAYLOAD','APPLY_FAILED')),
 detail TEXT NOT NULL CHECK(length(detail) BETWEEN 1 AND 1000),
 status TEXT NOT NULL DEFAULT 'OPEN' CHECK(status IN ('OPEN','RESOLVED','DISMISSED')),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 resolved_at TEXT,
 UNIQUE(organization_id,event_id),
 FOREIGN KEY(organization_id,event_id) REFERENCES sync_inbox(organization_id,event_id) ON DELETE RESTRICT,
 CHECK((status='OPEN' AND resolved_at IS NULL) OR (status IN ('RESOLVED','DISMISSED') AND resolved_at IS NOT NULL))
) STRICT;
CREATE INDEX sync_conflicts_status ON sync_conflicts(organization_id,status,created_at,id);

CREATE TABLE sync_attempts (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id)=36),
 organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 device_id TEXT NOT NULL,
 direction TEXT NOT NULL CHECK(direction IN ('PUSH','PULL','APPLY')),
 outcome TEXT NOT NULL CHECK(outcome IN ('STARTED','SUCCEEDED','FAILED','SKIPPED')),
 event_count INTEGER NOT NULL DEFAULT 0 CHECK(event_count>=0),
 safe_message TEXT,
 started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 finished_at TEXT,
 FOREIGN KEY(organization_id,device_id) REFERENCES cloud_device_enrollments(organization_id,device_id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX sync_attempts_recent ON sync_attempts(organization_id,started_at DESC,id DESC);

CREATE TABLE sync_runtime_gate (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 phase8c_verified INTEGER NOT NULL DEFAULT 0 CHECK(phase8c_verified IN (0,1)),
 verified_at TEXT,
 CHECK((phase8c_verified=0 AND verified_at IS NULL) OR (phase8c_verified=1 AND verified_at IS NOT NULL))
) STRICT;
INSERT INTO sync_runtime_gate(singleton,phase8c_verified) VALUES(1,0);

CREATE TABLE sync_apply_context (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 applying INTEGER NOT NULL DEFAULT 0 CHECK(applying IN (0,1))
) STRICT;
INSERT INTO sync_apply_context(singleton,applying) VALUES(1,0);
