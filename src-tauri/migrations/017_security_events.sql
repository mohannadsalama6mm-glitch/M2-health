ALTER TABLE inventory_batches ADD COLUMN created_by TEXT;
ALTER TABLE sale_returns ADD COLUMN created_by TEXT;
CREATE TRIGGER inventory_batch_actor_immutable
BEFORE UPDATE OF created_by ON inventory_batches
WHEN NEW.created_by IS NOT OLD.created_by
BEGIN SELECT RAISE(ABORT,'inventory_batch_identity_immutable'); END;

CREATE TRIGGER sale_return_actor_immutable
BEFORE UPDATE OF created_by ON sale_returns
WHEN NEW.created_by IS NOT OLD.created_by
BEGIN SELECT RAISE(ABORT,'sale_return_identity_immutable'); END;

CREATE TABLE security_events (
 id TEXT PRIMARY KEY NOT NULL,
 actor_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 subject_user_id TEXT REFERENCES users(id) ON DELETE RESTRICT,
 event_type TEXT NOT NULL CHECK(event_type IN (
  'USER_CREATED',
  'USER_UPDATED',
  'USER_ACTIVATED',
  'USER_DEACTIVATED',
  'PASSWORD_RESET',
  'ROLE_ASSIGNED',
  'ROLE_REMOVED',
  'BRANCH_MEMBERSHIP_CHANGED'
 )),
 detail TEXT NOT NULL DEFAULT '' CHECK(length(detail)<=1000),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX security_events_actor ON security_events(actor_user_id,created_at DESC,id DESC);
CREATE INDEX security_events_subject ON security_events(subject_user_id,created_at DESC,id DESC);

CREATE TRIGGER security_events_no_update
BEFORE UPDATE ON security_events
BEGIN SELECT RAISE(ABORT,'security_events_are_immutable'); END;

CREATE TRIGGER security_events_no_delete
BEFORE DELETE ON security_events
BEGIN SELECT RAISE(ABORT,'security_events_are_immutable'); END;
