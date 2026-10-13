-- Phase 8A local-only cloud foundation. Existing data is deliberately not
-- assigned to an organization; tenant adoption requires a future trusted flow.
CREATE TABLE organizations (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id)=36),
 name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 160),
 slug TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK(length(trim(slug)) BETWEEN 3 AND 80),
 status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK(status IN ('ACTIVE','SUSPENDED')),
 cloud_tenant_id TEXT UNIQUE,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;

CREATE TABLE organization_memberships (
 organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 role_code TEXT NOT NULL CHECK(role_code IN ('OWNER','ADMIN','MEMBER')),
 status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK(status IN ('ACTIVE','REVOKED')),
 linked_by_user_id TEXT REFERENCES users(id) ON DELETE RESTRICT,
 linked_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 revoked_at TEXT,
 PRIMARY KEY(organization_id,user_id),
 CHECK((status='ACTIVE' AND revoked_at IS NULL) OR (status='REVOKED' AND revoked_at IS NOT NULL))
) STRICT;

CREATE TABLE branch_tenant_ownership (
 organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 branch_id TEXT NOT NULL UNIQUE REFERENCES branches(id) ON DELETE RESTRICT,
 adopted_by_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 adopted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(organization_id,branch_id),
 FOREIGN KEY(organization_id,adopted_by_user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE organization_branch_memberships (
 organization_id TEXT NOT NULL,
 user_id TEXT NOT NULL,
 branch_id TEXT NOT NULL,
 granted_by_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 granted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(organization_id,user_id,branch_id),
 FOREIGN KEY(organization_id,user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT,
 FOREIGN KEY(organization_id,branch_id)
   REFERENCES branch_tenant_ownership(organization_id,branch_id) ON DELETE RESTRICT,
 FOREIGN KEY(organization_id,granted_by_user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT
) STRICT;

CREATE TRIGGER branch_tenant_adoption_admin BEFORE INSERT ON branch_tenant_ownership
BEGIN SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM organization_memberships m
 WHERE m.organization_id=NEW.organization_id AND m.user_id=NEW.adopted_by_user_id
   AND m.status='ACTIVE' AND m.role_code IN ('OWNER','ADMIN')
) THEN RAISE(ABORT,'active organization administrator required') END; END;

CREATE TRIGGER branch_membership_grant_admin BEFORE INSERT ON organization_branch_memberships
BEGIN SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM organization_memberships m
 WHERE m.organization_id=NEW.organization_id AND m.user_id=NEW.granted_by_user_id
   AND m.status='ACTIVE' AND m.role_code IN ('OWNER','ADMIN')
) THEN RAISE(ABORT,'active organization administrator required') END; END;

CREATE TABLE cloud_identity_links (
 organization_id TEXT NOT NULL,
 local_user_id TEXT NOT NULL,
 cloud_subject_id TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'PENDING' CHECK(status IN ('PENDING','ACTIVE','REVOKED')),
 linked_by_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 linked_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 revoked_at TEXT,
 PRIMARY KEY(organization_id,local_user_id),
 UNIQUE(organization_id,cloud_subject_id),
 FOREIGN KEY(organization_id,local_user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT,
 FOREIGN KEY(organization_id,linked_by_user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT,
 CHECK((status='REVOKED')=(revoked_at IS NOT NULL))
) STRICT;

CREATE TRIGGER cloud_identity_link_admin BEFORE INSERT ON cloud_identity_links
BEGIN SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM organization_memberships m
 WHERE m.organization_id=NEW.organization_id AND m.user_id=NEW.linked_by_user_id
   AND m.status='ACTIVE' AND m.role_code IN ('OWNER','ADMIN')
) THEN RAISE(ABORT,'active organization administrator required') END; END;

CREATE TABLE local_device_identity (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 device_id TEXT NOT NULL UNIQUE CHECK(length(device_id)=36),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
INSERT INTO local_device_identity(singleton,device_id)
VALUES(1, lower(hex(randomblob(4)))||'-'||lower(hex(randomblob(2)))||'-4'||substr(lower(hex(randomblob(2))),2)||'-a'||substr(lower(hex(randomblob(2))),2)||'-'||lower(hex(randomblob(6))));

CREATE TABLE cloud_device_enrollments (
 device_id TEXT PRIMARY KEY NOT NULL REFERENCES local_device_identity(device_id) ON DELETE RESTRICT,
 organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 cloud_device_id TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('PENDING','ENROLLED','REVOKED')),
 enrolled_by_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 enrolled_at TEXT,
 revoked_at TEXT,
 UNIQUE(organization_id,device_id),
 UNIQUE(organization_id,cloud_device_id),
 FOREIGN KEY(organization_id,enrolled_by_user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT,
 CHECK((status='PENDING' AND enrolled_at IS NULL AND revoked_at IS NULL)
    OR (status='ENROLLED' AND enrolled_at IS NOT NULL AND revoked_at IS NULL)
    OR (status='REVOKED' AND revoked_at IS NOT NULL))
) STRICT;

CREATE TRIGGER cloud_device_enrollment_admin BEFORE INSERT ON cloud_device_enrollments
BEGIN SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM organization_memberships m
 WHERE m.organization_id=NEW.organization_id AND m.user_id=NEW.enrolled_by_user_id
   AND m.status='ACTIVE' AND m.role_code IN ('OWNER','ADMIN')
) THEN RAISE(ABORT,'active organization administrator required') END; END;

CREATE TABLE sync_outbox (
 event_id TEXT PRIMARY KEY NOT NULL CHECK(length(event_id)=36),
 organization_id TEXT NOT NULL,
 branch_id TEXT,
 device_id TEXT NOT NULL,
 actor_user_id TEXT NOT NULL,
 entity_type TEXT NOT NULL CHECK(length(entity_type) BETWEEN 1 AND 80),
 entity_id TEXT NOT NULL CHECK(length(entity_id) BETWEEN 1 AND 160),
 operation TEXT NOT NULL CHECK(operation IN ('CREATED','UPDATED','APPENDED','REVOKED')),
 payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
 schema_version INTEGER NOT NULL CHECK(schema_version>0),
 causation_id TEXT,
 occurred_at TEXT NOT NULL,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 attempt_count INTEGER NOT NULL DEFAULT 0 CHECK(attempt_count>=0),
 next_attempt_at TEXT,
 last_error TEXT,
 acknowledged_at TEXT,
 FOREIGN KEY(organization_id,device_id)
   REFERENCES cloud_device_enrollments(organization_id,device_id) ON DELETE RESTRICT,
 FOREIGN KEY(organization_id,actor_user_id)
   REFERENCES organization_memberships(organization_id,user_id) ON DELETE RESTRICT,
 FOREIGN KEY(organization_id,branch_id)
   REFERENCES branch_tenant_ownership(organization_id,branch_id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX sync_outbox_pending ON sync_outbox(organization_id,acknowledged_at,next_attempt_at,created_at,event_id);

CREATE TRIGGER sync_outbox_active_enrollment BEFORE INSERT ON sync_outbox
BEGIN
 SELECT CASE WHEN NOT EXISTS(
   SELECT 1 FROM cloud_device_enrollments d
   JOIN organization_memberships m ON m.organization_id=d.organization_id AND m.user_id=NEW.actor_user_id
   WHERE d.organization_id=NEW.organization_id AND d.device_id=NEW.device_id
     AND d.status='ENROLLED' AND m.status='ACTIVE'
 ) THEN RAISE(ABORT,'active device enrollment and actor membership required') END;
 SELECT CASE WHEN NEW.branch_id IS NOT NULL AND NOT EXISTS(
   SELECT 1 FROM organization_branch_memberships b
   WHERE b.organization_id=NEW.organization_id AND b.branch_id=NEW.branch_id AND b.user_id=NEW.actor_user_id
 ) THEN RAISE(ABORT,'active branch membership required') END;
END;

CREATE TRIGGER sync_outbox_envelope_immutable BEFORE UPDATE OF
 organization_id,branch_id,device_id,actor_user_id,entity_type,entity_id,operation,
 payload_json,schema_version,causation_id,occurred_at,created_at ON sync_outbox
BEGIN SELECT RAISE(ABORT,'sync event envelope is immutable'); END;

CREATE TABLE sync_inbox (
 organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
 event_id TEXT NOT NULL CHECK(length(event_id)=36),
 branch_id TEXT,
 payload_hash TEXT NOT NULL CHECK(length(payload_hash)=64),
 status TEXT NOT NULL CHECK(status IN ('RECEIVED','APPLIED','REJECTED')),
 received_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 applied_at TEXT,
 error TEXT,
 PRIMARY KEY(organization_id,event_id),
 FOREIGN KEY(organization_id,branch_id)
   REFERENCES branch_tenant_ownership(organization_id,branch_id) ON DELETE RESTRICT,
 CHECK((status='APPLIED')=(applied_at IS NOT NULL))
) STRICT;

CREATE TABLE sync_receipts (
 organization_id TEXT NOT NULL,
 event_id TEXT NOT NULL,
 device_id TEXT NOT NULL,
 server_sequence INTEGER NOT NULL CHECK(server_sequence>=0),
 received_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(organization_id,event_id,device_id),
 FOREIGN KEY(event_id) REFERENCES sync_outbox(event_id) ON DELETE RESTRICT,
 FOREIGN KEY(organization_id,device_id)
   REFERENCES cloud_device_enrollments(organization_id,device_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE sync_checkpoints (
 organization_id TEXT NOT NULL,
 device_id TEXT NOT NULL,
 stream TEXT NOT NULL CHECK(length(stream) BETWEEN 1 AND 80),
 cursor TEXT NOT NULL,
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(organization_id,device_id,stream),
 FOREIGN KEY(organization_id,device_id)
   REFERENCES cloud_device_enrollments(organization_id,device_id) ON DELETE RESTRICT
) STRICT;

CREATE TABLE sync_schema_metadata (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 protocol_version INTEGER NOT NULL CHECK(protocol_version>0),
 transport_enabled INTEGER NOT NULL DEFAULT 0 CHECK(transport_enabled=0),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
INSERT INTO sync_schema_metadata(singleton,protocol_version,transport_enabled) VALUES(1,1,0);
