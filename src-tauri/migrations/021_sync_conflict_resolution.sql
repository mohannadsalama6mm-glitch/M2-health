-- Phase 8C auditable, authorized conflict resolution. Existing open history is preserved.
ALTER TABLE sync_conflicts ADD COLUMN resolution_action TEXT
 CHECK(resolution_action IS NULL OR resolution_action IN ('RETRIED','DISMISSED'));
ALTER TABLE sync_conflicts ADD COLUMN resolved_by_user_id TEXT REFERENCES users(id) ON DELETE RESTRICT;
ALTER TABLE sync_conflicts ADD COLUMN resolution_note TEXT CHECK(resolution_note IS NULL OR length(trim(resolution_note)) BETWEEN 1 AND 500);
CREATE INDEX sync_conflicts_resolution_actor ON sync_conflicts(resolved_by_user_id,resolved_at);
