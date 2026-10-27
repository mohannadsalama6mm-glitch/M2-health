-- Phase 9A.1 device-scoped Early Access license cache. The existing Phase 8
-- device identity remains authoritative; pharmacy domain data is never owned
-- by or deleted with a license record.
CREATE TABLE local_license_state (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 device_id TEXT NOT NULL REFERENCES local_device_identity(device_id) ON DELETE RESTRICT,
 license_id TEXT,
 license_kind TEXT NOT NULL DEFAULT 'UNACTIVATED'
   CHECK(license_kind IN ('UNACTIVATED','TRIAL','ACTIVE','SUSPENDED','REVOKED')),
 activated_at INTEGER,
 expires_at INTEGER,
 last_validated_at INTEGER,
 next_revalidation_at INTEGER,
 offline_grace_seconds INTEGER NOT NULL DEFAULT 259200
   CHECK(offline_grace_seconds BETWEEN 0 AND 2592000),
 max_observed_at INTEGER NOT NULL DEFAULT (unixepoch()),
 signed_payload TEXT,
 signature TEXT,
 updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
 CHECK((license_kind='UNACTIVATED' AND activated_at IS NULL)
    OR (license_kind<>'UNACTIVATED' AND activated_at IS NOT NULL)),
 CHECK(license_kind<>'TRIAL' OR expires_at IS NOT NULL),
 CHECK((signed_payload IS NULL)=(signature IS NULL))
) STRICT;

INSERT INTO local_license_state(singleton,device_id)
SELECT 1,device_id FROM local_device_identity WHERE singleton=1;

CREATE TABLE local_license_events (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id)=36),
 event_type TEXT NOT NULL CHECK(event_type IN (
   'TRIAL_STARTED','LICENSE_ACTIVATED','LICENSE_REVALIDATED',
   'LICENSE_REJECTED','CLOCK_ROLLBACK_DETECTED'
 )),
 occurred_at INTEGER NOT NULL DEFAULT (unixepoch()),
 detail TEXT NOT NULL DEFAULT '' CHECK(length(detail)<=240)
) STRICT;

CREATE TRIGGER local_license_state_no_delete
BEFORE DELETE ON local_license_state
BEGIN SELECT RAISE(ABORT,'local_license_state_is_required'); END;

CREATE TRIGGER local_license_events_immutable_update
BEFORE UPDATE ON local_license_events
BEGIN SELECT RAISE(ABORT,'local_license_events_are_immutable'); END;

CREATE TRIGGER local_license_events_immutable_delete
BEFORE DELETE ON local_license_events
BEGIN SELECT RAISE(ABORT,'local_license_events_are_immutable'); END;
