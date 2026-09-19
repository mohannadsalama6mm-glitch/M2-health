-- Canonical Phase 3B documents. Extends, never replaces, the Phase 3A ledger.
CREATE TABLE inventory_document_sequences (
 day TEXT NOT NULL, prefix TEXT NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number>0), PRIMARY KEY(day,prefix)
) STRICT;
CREATE TABLE inventory_documents (
 id TEXT PRIMARY KEY NOT NULL,
 document_number TEXT NOT NULL UNIQUE,
 kind TEXT NOT NULL CHECK(kind IN ('RECEIPT','ADJUSTMENT','WRITE_OFF','TRANSFER','REVERSAL')),
 branch_id TEXT NOT NULL REFERENCES branches(id),
 destination_branch_id TEXT REFERENCES branches(id),
 status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT','READY','POSTED','CANCELLED')),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 reference_text TEXT, supplier_reference TEXT, notes TEXT, created_by TEXT,
 reversal_of TEXT UNIQUE REFERENCES inventory_documents(id),
 reversal_reason TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 CHECK((status='POSTED')=(posted_at IS NOT NULL)),
 CHECK((kind='REVERSAL')=(reversal_of IS NOT NULL)),
 CHECK(kind!='REVERSAL' OR length(trim(reversal_reason))>0),
 CHECK((kind='TRANSFER')=(destination_branch_id IS NOT NULL)),
 CHECK(destination_branch_id IS NULL OR destination_branch_id!=branch_id)
) STRICT;
CREATE INDEX inventory_documents_list ON inventory_documents(branch_id,created_at DESC,id);
CREATE INDEX inventory_documents_filters ON inventory_documents(branch_id,kind,status,created_at DESC,id);
CREATE TABLE inventory_document_lines (
 id TEXT PRIMARY KEY NOT NULL,
 document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 position INTEGER NOT NULL CHECK(position>=0),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 batch_id TEXT REFERENCES inventory_batches(id),
 action TEXT NOT NULL CHECK(action IN ('RECEIVE','IN','OUT','DAMAGE','EXPIRY','TRANSFER')),
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 4000),
 lot_number TEXT CHECK(lot_number IS NULL OR length(trim(lot_number)) BETWEEN 1 AND 4000),
 expiry_date TEXT CHECK(expiry_date IS NULL OR (length(expiry_date)=10 AND date(expiry_date,'+0 days') IS expiry_date)),
 cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
 supplier_reference TEXT,
 product_label TEXT NOT NULL, package_label TEXT NOT NULL,
 UNIQUE(document_id,position), UNIQUE(id,document_id)
) STRICT;
CREATE TABLE inventory_document_movements (
 movement_id TEXT PRIMARY KEY NOT NULL REFERENCES inventory_movements(id),
 document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 line_id TEXT NOT NULL,
 reverses_movement_id TEXT UNIQUE REFERENCES inventory_movements(id),
 FOREIGN KEY(line_id,document_id) REFERENCES inventory_document_lines(id,document_id)
) STRICT;
CREATE INDEX inventory_document_movement_lookup ON inventory_document_movements(document_id,line_id);
CREATE TABLE inventory_post_requests (
 request_id TEXT PRIMARY KEY NOT NULL,
 payload TEXT NOT NULL,
 result_document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TRIGGER inventory_document_insert_guard BEFORE INSERT ON inventory_documents
 WHEN NEW.status!='DRAFT' OR NEW.revision!=1
 BEGIN SELECT RAISE(ABORT,'document_requires_draft'); END;
CREATE TRIGGER inventory_document_guard BEFORE UPDATE ON inventory_documents BEGIN
 SELECT CASE WHEN OLD.status IN ('POSTED','CANCELLED') THEN RAISE(ABORT,'document_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.document_number IS NOT OLD.document_number OR NEW.kind IS NOT OLD.kind OR NEW.branch_id IS NOT OLD.branch_id OR NEW.created_at IS NOT OLD.created_at OR NEW.reversal_of IS NOT OLD.reversal_of OR NEW.reversal_reason IS NOT OLD.reversal_reason
 THEN RAISE(ABORT,'document_identity_immutable') END;
 SELECT CASE WHEN NEW.revision!=OLD.revision+1 THEN RAISE(ABORT,'document_revision_conflict') END;
 SELECT CASE WHEN NOT ((OLD.status='DRAFT' AND NEW.status IN ('DRAFT','READY','CANCELLED')) OR (OLD.status='READY' AND NEW.status IN ('POSTED','CANCELLED')))
 THEN RAISE(ABORT,'document_invalid_transition') END;
 SELECT CASE WHEN OLD.status='READY' AND (NEW.destination_branch_id IS NOT OLD.destination_branch_id OR NEW.reference_text IS NOT OLD.reference_text OR NEW.supplier_reference IS NOT OLD.supplier_reference OR NEW.notes IS NOT OLD.notes OR NEW.created_by IS NOT OLD.created_by)
 THEN RAISE(ABORT,'ready_document_frozen') END;
 SELECT CASE WHEN NEW.status IN ('READY','POSTED') AND NOT EXISTS(SELECT 1 FROM inventory_document_lines WHERE document_id=NEW.id)
 THEN RAISE(ABORT,'document_requires_lines') END;
 SELECT CASE WHEN NEW.status='POSTED' AND EXISTS(
 SELECT 1 FROM inventory_document_lines l WHERE l.document_id=NEW.id AND
 (SELECT count(*) FROM inventory_document_movements dm WHERE dm.line_id=l.id) != CASE WHEN NEW.kind='TRANSFER' THEN 2 ELSE 1 END)
 THEN RAISE(ABORT,'document_incomplete_posting') END;
 END;
CREATE TRIGGER inventory_document_no_delete BEFORE DELETE ON inventory_documents BEGIN SELECT RAISE(ABORT,'document_no_delete'); END;
CREATE TRIGGER inventory_document_line_insert BEFORE INSERT ON inventory_document_lines BEGIN
 SELECT CASE WHEN (SELECT status FROM inventory_documents WHERE id=NEW.document_id)!='DRAFT' THEN RAISE(ABORT,'lines_require_draft') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM inventory_documents d WHERE d.id=NEW.document_id AND
 ((d.kind='RECEIPT' AND NEW.action='RECEIVE') OR (d.kind IN ('ADJUSTMENT','REVERSAL') AND NEW.action IN ('IN','OUT')) OR (d.kind='WRITE_OFF' AND NEW.action IN ('DAMAGE','EXPIRY')) OR (d.kind='TRANSFER' AND NEW.action='TRANSFER')))
 THEN RAISE(ABORT,'document_line_kind') END;
 SELECT CASE WHEN NEW.action!='RECEIVE' AND NEW.batch_id IS NULL THEN RAISE(ABORT,'document_line_requires_batch') END;
 END;
CREATE TRIGGER inventory_document_line_update BEFORE UPDATE ON inventory_document_lines BEGIN SELECT RAISE(ABORT,'replace_lines_in_draft'); END;
CREATE TRIGGER inventory_document_line_delete BEFORE DELETE ON inventory_document_lines WHEN (SELECT status FROM inventory_documents WHERE id=OLD.document_id)!='DRAFT' BEGIN SELECT RAISE(ABORT,'lines_require_draft'); END;
CREATE TRIGGER inventory_document_link_insert BEFORE INSERT ON inventory_document_movements BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM inventory_documents d JOIN inventory_document_lines l ON l.document_id=d.id JOIN inventory_movements m ON m.id=NEW.movement_id
 WHERE d.id=NEW.document_id AND l.id=NEW.line_id AND d.status='READY' AND m.reference_type='inventory_document' AND m.reference_id=d.id AND m.product_package_id=l.product_package_id AND abs(m.quantity_delta)=l.quantity)
 THEN RAISE(ABORT,'document_movement_mismatch') END;
 SELECT CASE WHEN NEW.reverses_movement_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM inventory_movements original JOIN inventory_document_movements link ON link.movement_id=original.id JOIN inventory_documents d ON d.reversal_of=link.document_id JOIN inventory_movements reverse ON reverse.id=NEW.movement_id
 WHERE d.id=NEW.document_id AND original.id=NEW.reverses_movement_id AND reverse.batch_id=original.batch_id AND reverse.branch_id=original.branch_id AND reverse.quantity_delta=-original.quantity_delta)
 THEN RAISE(ABORT,'document_reversal_mismatch') END;
 END;
CREATE TRIGGER inventory_document_link_no_update BEFORE UPDATE ON inventory_document_movements BEGIN SELECT RAISE(ABORT,'document_link_immutable'); END;
CREATE TRIGGER inventory_document_link_no_delete BEFORE DELETE ON inventory_document_movements BEGIN SELECT RAISE(ABORT,'document_link_immutable'); END;
CREATE TRIGGER inventory_post_request_insert BEFORE INSERT ON inventory_post_requests WHEN (SELECT status FROM inventory_documents WHERE id=NEW.result_document_id)!='POSTED' BEGIN SELECT RAISE(ABORT,'post_request_requires_posted'); END;
CREATE TRIGGER inventory_post_request_no_update BEFORE UPDATE ON inventory_post_requests BEGIN SELECT RAISE(ABORT,'post_request_immutable'); END;
CREATE TRIGGER inventory_post_request_no_delete BEFORE DELETE ON inventory_post_requests BEGIN SELECT RAISE(ABORT,'post_request_immutable'); END;
