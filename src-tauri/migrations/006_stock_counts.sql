-- Canonical count sessions; the sole stock ledger remains inventory_movements.
CREATE TABLE stock_count_sequences(day TEXT PRIMARY KEY NOT NULL, next_number INTEGER NOT NULL CHECK(next_number>0)) STRICT;
CREATE TABLE stock_count_sessions (
 id TEXT PRIMARY KEY NOT NULL, session_number TEXT NOT NULL UNIQUE,
 branch_id TEXT NOT NULL REFERENCES branches(id),
 status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT','IN_PROGRESS','REVIEW','COMPLETED','CANCELLED')),
 scope_type TEXT NOT NULL DEFAULT 'ALL_INVENTORY' CHECK(scope_type='ALL_INVENTORY'),
 notes TEXT, created_by TEXT, revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 snapshot_watermark INTEGER CHECK(snapshot_watermark>=0),
 created_at TEXT NOT NULL DEFAULT(strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 started_at TEXT, reviewed_at TEXT, completed_at TEXT, cancelled_at TEXT,
 UNIQUE(id,branch_id),
 CHECK((started_at IS NULL)=(snapshot_watermark IS NULL)),
 CHECK((status='COMPLETED')=(completed_at IS NOT NULL)),
 CHECK((status='CANCELLED')=(cancelled_at IS NOT NULL))
) STRICT;
CREATE INDEX stock_count_sessions_list ON stock_count_sessions(branch_id,status,created_at DESC,id);
CREATE TABLE stock_count_lines (
 id TEXT PRIMARY KEY NOT NULL, session_id TEXT NOT NULL, branch_id TEXT NOT NULL,
 product_package_id TEXT NOT NULL, batch_id TEXT NOT NULL,
 expected_quantity INTEGER NOT NULL CHECK(expected_quantity BETWEEN 0 AND 9007199254740991),
 counted_quantity INTEGER CHECK(counted_quantity BETWEEN 0 AND 9007199254740991),
 notes TEXT, excluded_reason TEXT CHECK(excluded_reason IS NULL OR length(trim(excluded_reason)) BETWEEN 1 AND 4000),
 counted_at TEXT, revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 product_label TEXT NOT NULL, package_label TEXT NOT NULL,
 FOREIGN KEY(session_id,branch_id) REFERENCES stock_count_sessions(id,branch_id),
 FOREIGN KEY(batch_id,branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id),
 UNIQUE(session_id,batch_id),
 CHECK((counted_quantity IS NULL)=(counted_at IS NULL)),
 CHECK(excluded_reason IS NULL OR counted_quantity IS NULL)
) STRICT;
CREATE INDEX stock_count_lines_session ON stock_count_lines(session_id,id);
CREATE TABLE stock_count_results (
 line_id TEXT PRIMARY KEY NOT NULL REFERENCES stock_count_lines(id),
 live_baseline INTEGER NOT NULL CHECK(live_baseline BETWEEN 0 AND 9007199254740991),
 correction_delta INTEGER NOT NULL CHECK(correction_delta BETWEEN -9007199254740991 AND 9007199254740991),
 intervening_movements INTEGER NOT NULL CHECK(intervening_movements>=0),
 movement_id TEXT UNIQUE REFERENCES inventory_movements(id),
 CHECK((correction_delta=0)=(movement_id IS NULL))
) STRICT;
CREATE TABLE stock_count_completion_requests (
 request_id TEXT PRIMARY KEY NOT NULL, payload TEXT NOT NULL,
 session_id TEXT NOT NULL UNIQUE REFERENCES stock_count_sessions(id),
 created_at TEXT NOT NULL DEFAULT(strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TRIGGER stock_count_session_insert BEFORE INSERT ON stock_count_sessions
 WHEN NEW.status!='DRAFT' OR NEW.revision!=1 OR NEW.snapshot_watermark IS NOT NULL
 BEGIN SELECT RAISE(ABORT,'count_requires_draft'); END;
CREATE TRIGGER stock_count_session_update BEFORE UPDATE ON stock_count_sessions BEGIN
 SELECT CASE WHEN OLD.status IN ('COMPLETED','CANCELLED') THEN RAISE(ABORT,'count_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.session_number IS NOT OLD.session_number OR NEW.branch_id IS NOT OLD.branch_id OR NEW.scope_type IS NOT OLD.scope_type OR NEW.created_at IS NOT OLD.created_at OR NEW.created_by IS NOT OLD.created_by OR NEW.notes IS NOT OLD.notes THEN RAISE(ABORT,'count_identity_immutable') END;
 SELECT CASE WHEN NEW.revision!=OLD.revision+1 THEN RAISE(ABORT,'count_stale_revision') END;
 SELECT CASE WHEN OLD.snapshot_watermark IS NOT NULL AND (NEW.snapshot_watermark IS NOT OLD.snapshot_watermark OR NEW.started_at IS NOT OLD.started_at) THEN RAISE(ABORT,'count_snapshot_immutable') END;
 SELECT CASE WHEN NOT ((OLD.status='DRAFT' AND NEW.status IN ('IN_PROGRESS','CANCELLED')) OR (OLD.status='IN_PROGRESS' AND NEW.status IN ('IN_PROGRESS','REVIEW','CANCELLED')) OR (OLD.status='REVIEW' AND NEW.status IN ('IN_PROGRESS','COMPLETED','CANCELLED'))) THEN RAISE(ABORT,'count_invalid_transition') END;
 SELECT CASE WHEN NEW.status IN ('IN_PROGRESS','REVIEW','COMPLETED') AND NEW.snapshot_watermark IS NULL THEN RAISE(ABORT,'count_requires_snapshot') END;
 SELECT CASE WHEN NEW.status='COMPLETED' AND EXISTS(SELECT 1 FROM stock_count_lines l WHERE l.session_id=NEW.id AND ((l.counted_quantity IS NULL AND l.excluded_reason IS NULL) OR NOT EXISTS(SELECT 1 FROM stock_count_results r WHERE r.line_id=l.id))) THEN RAISE(ABORT,'count_incomplete') END;
 END;
CREATE TRIGGER stock_count_session_delete BEFORE DELETE ON stock_count_sessions BEGIN SELECT RAISE(ABORT,'count_no_delete'); END;
CREATE TRIGGER stock_count_line_insert BEFORE INSERT ON stock_count_lines BEGIN
 SELECT CASE WHEN (SELECT status FROM stock_count_sessions WHERE id=NEW.session_id)!='IN_PROGRESS' OR NEW.counted_quantity IS NOT NULL OR NEW.revision!=1 OR NEW.excluded_reason IS NOT NULL THEN RAISE(ABORT,'count_line_requires_counting') END;
 SELECT CASE WHEN NEW.expected_quantity!=(SELECT coalesce(sum(m.quantity_delta),0) FROM inventory_movements m JOIN stock_count_sessions s ON s.id=NEW.session_id WHERE m.batch_id=NEW.batch_id AND m.rowid<=s.snapshot_watermark) THEN RAISE(ABORT,'count_snapshot_mismatch') END;
 END;
CREATE TRIGGER stock_count_line_update BEFORE UPDATE ON stock_count_lines BEGIN
 SELECT CASE WHEN (SELECT status FROM stock_count_sessions WHERE id=OLD.session_id)!='IN_PROGRESS' THEN RAISE(ABORT,'count_line_read_only') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.session_id IS NOT OLD.session_id OR NEW.branch_id IS NOT OLD.branch_id OR NEW.batch_id IS NOT OLD.batch_id OR NEW.product_package_id IS NOT OLD.product_package_id OR NEW.expected_quantity IS NOT OLD.expected_quantity OR NEW.product_label IS NOT OLD.product_label OR NEW.package_label IS NOT OLD.package_label THEN RAISE(ABORT,'count_snapshot_immutable') END;
 SELECT CASE WHEN NEW.revision!=OLD.revision+1 THEN RAISE(ABORT,'count_line_stale') END;
 END;
CREATE TRIGGER stock_count_line_delete BEFORE DELETE ON stock_count_lines BEGIN SELECT RAISE(ABORT,'count_line_no_delete'); END;
CREATE TRIGGER stock_count_correction_guard BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='STOCK_COUNT_CORRECTION' BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM stock_count_lines l JOIN stock_count_sessions s ON s.id=l.session_id WHERE l.id=NEW.reference_id AND NEW.reference_type='stock_count_line' AND s.status='REVIEW' AND l.excluded_reason IS NULL AND l.counted_quantity IS NOT NULL AND l.branch_id=NEW.branch_id AND l.batch_id=NEW.batch_id AND l.product_package_id=NEW.product_package_id AND NEW.quantity_delta=l.counted_quantity-(SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=l.batch_id)) THEN RAISE(ABORT,'count_correction_mismatch') END;
 END;
CREATE TRIGGER stock_count_result_insert BEFORE INSERT ON stock_count_results BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM stock_count_lines l JOIN stock_count_sessions s ON s.id=l.session_id WHERE l.id=NEW.line_id AND s.status='REVIEW' AND NEW.correction_delta=CASE WHEN l.excluded_reason IS NOT NULL THEN 0 ELSE l.counted_quantity-NEW.live_baseline END AND (NEW.movement_id IS NULL OR EXISTS(SELECT 1 FROM inventory_movements m WHERE m.id=NEW.movement_id AND m.movement_type='STOCK_COUNT_CORRECTION' AND m.reference_type='stock_count_line' AND m.reference_id=l.id AND m.quantity_delta=NEW.correction_delta))) THEN RAISE(ABORT,'count_result_mismatch') END;
 END;
CREATE TRIGGER stock_count_result_update BEFORE UPDATE ON stock_count_results BEGIN SELECT RAISE(ABORT,'count_result_immutable'); END;
CREATE TRIGGER stock_count_result_delete BEFORE DELETE ON stock_count_results BEGIN SELECT RAISE(ABORT,'count_result_immutable'); END;
CREATE TRIGGER stock_count_request_insert BEFORE INSERT ON stock_count_completion_requests WHEN (SELECT status FROM stock_count_sessions WHERE id=NEW.session_id)!='COMPLETED' BEGIN SELECT RAISE(ABORT,'count_request_requires_completion'); END;
CREATE TRIGGER stock_count_request_update BEFORE UPDATE ON stock_count_completion_requests BEGIN SELECT RAISE(ABORT,'count_request_immutable'); END;
CREATE TRIGGER stock_count_request_delete BEFORE DELETE ON stock_count_completion_requests BEGIN SELECT RAISE(ABORT,'count_request_immutable'); END;
CREATE VIEW stock_count_line_view AS
 SELECT l.*, b.lot_number,b.expiry_date,
 coalesce(r.live_baseline,(SELECT coalesce(sum(m.quantity_delta),0) FROM inventory_movements m WHERE m.batch_id=l.batch_id)) current_live,
 l.counted_quantity-l.expected_quantity snapshot_variance,
 coalesce(r.correction_delta,CASE WHEN l.excluded_reason IS NOT NULL THEN 0 ELSE l.counted_quantity-(SELECT coalesce(sum(m.quantity_delta),0) FROM inventory_movements m WHERE m.batch_id=l.batch_id) END) correction_delta,
 coalesce(r.intervening_movements,(SELECT count(*) FROM inventory_movements m WHERE m.batch_id=l.batch_id AND m.rowid>s.snapshot_watermark)) intervening_movements,
 r.movement_id
 FROM stock_count_lines l JOIN stock_count_sessions s ON s.id=l.session_id JOIN inventory_batches b ON b.id=l.batch_id LEFT JOIN stock_count_results r ON r.line_id=l.id;
