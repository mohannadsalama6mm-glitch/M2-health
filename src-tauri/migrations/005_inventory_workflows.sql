CREATE TABLE inventory_document_sequences (
 day TEXT NOT NULL,
 prefix TEXT NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number>0),
 PRIMARY KEY(day,prefix)
) STRICT;
CREATE TABLE inventory_documents (
 id TEXT PRIMARY KEY NOT NULL,
 document_number TEXT NOT NULL UNIQUE,
 kind TEXT NOT NULL CHECK(kind IN ('RECEIPT','ADJUSTMENT','WRITE_OFF','TRANSFER','REVERSAL')),
 branch_id TEXT NOT NULL REFERENCES branches(id),
 destination_branch_id TEXT REFERENCES branches(id),
 status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT','READY','POSTED','CANCELLED')),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 reference_text TEXT,
 supplier_reference TEXT,
 notes TEXT,
 created_by TEXT,
 reversal_of TEXT REFERENCES inventory_documents(id),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 CHECK((status='POSTED')=(posted_at IS NOT NULL)),
 CHECK((kind='REVERSAL')=(reversal_of IS NOT NULL)),
 CHECK(kind!='TRANSFER' OR (destination_branch_id IS NOT NULL AND destination_branch_id!=branch_id)),
 CHECK(kind IN ('TRANSFER','REVERSAL') OR destination_branch_id IS NULL)
) STRICT;
CREATE UNIQUE INDEX inventory_document_one_reversal ON inventory_documents(reversal_of) WHERE reversal_of IS NOT NULL AND status!='CANCELLED';
CREATE INDEX inventory_documents_list ON inventory_documents(branch_id,kind,created_at DESC,id);
CREATE TABLE inventory_document_lines (
 id TEXT PRIMARY KEY NOT NULL,
 document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 position INTEGER NOT NULL CHECK(position>=0),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 batch_id TEXT REFERENCES inventory_batches(id),
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 action TEXT NOT NULL CHECK(action IN ('RECEIVE','IN','OUT','DAMAGE','EXPIRY','TRANSFER')),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 4000),
 lot_number TEXT,
 expiry_date TEXT CHECK(expiry_date IS NULL OR (length(expiry_date)=10 AND date(expiry_date,'+0 days') IS expiry_date)),
 cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
 supplier_reference TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(document_id,position),
 UNIQUE(id,document_id)
) STRICT;
CREATE TABLE inventory_document_movements (
 movement_id TEXT PRIMARY KEY NOT NULL REFERENCES inventory_movements(id),
 document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 line_id TEXT,
 reverses_movement_id TEXT UNIQUE REFERENCES inventory_movements(id),
 FOREIGN KEY(line_id,document_id) REFERENCES inventory_document_lines(id,document_id)
) STRICT;
CREATE INDEX inventory_document_movement_lookup ON inventory_document_movements(document_id);
CREATE TABLE inventory_post_requests (
 request_id TEXT PRIMARY KEY NOT NULL,
 document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 payload TEXT NOT NULL,
 result_document_id TEXT NOT NULL REFERENCES inventory_documents(id),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TRIGGER inventory_document_guard BEFORE UPDATE ON inventory_documents BEGIN
 SELECT CASE WHEN OLD.status IN ('POSTED','CANCELLED') THEN RAISE(ABORT,'inventory_document_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.document_number IS NOT OLD.document_number OR NEW.kind IS NOT OLD.kind OR NEW.branch_id IS NOT OLD.branch_id OR NEW.created_at IS NOT OLD.created_at OR NEW.reversal_of IS NOT OLD.reversal_of
 THEN RAISE(ABORT,'inventory_document_identity_immutable') END;
 SELECT CASE WHEN NOT ((OLD.status='DRAFT' AND NEW.status IN ('DRAFT','READY','CANCELLED')) OR (OLD.status='READY' AND NEW.status IN ('DRAFT','READY','POSTED','CANCELLED')))
 THEN RAISE(ABORT,'inventory_document_state_transition') END;
 SELECT CASE WHEN NEW.status='POSTED' AND NOT EXISTS(SELECT 1 FROM inventory_document_movements WHERE document_id=NEW.id)
 THEN RAISE(ABORT,'inventory_document_no_posted_movements') END;
 END;
CREATE TRIGGER inventory_document_no_delete BEFORE DELETE ON inventory_documents BEGIN SELECT RAISE(ABORT,'inventory_document_no_delete'); END;
CREATE TRIGGER inventory_line_insert_guard BEFORE INSERT ON inventory_document_lines WHEN (SELECT status FROM inventory_documents WHERE id=NEW.document_id)!='DRAFT' BEGIN SELECT RAISE(ABORT,'inventory_lines_require_draft'); END;
CREATE TRIGGER inventory_line_update_guard BEFORE UPDATE ON inventory_document_lines BEGIN SELECT RAISE(ABORT,'inventory_lines_replace_only_in_draft'); END;
CREATE TRIGGER inventory_line_delete_guard BEFORE DELETE ON inventory_document_lines WHEN (SELECT status FROM inventory_documents WHERE id=OLD.document_id)!='DRAFT' BEGIN SELECT RAISE(ABORT,'inventory_lines_require_draft'); END;
CREATE TRIGGER inventory_document_link_insert BEFORE INSERT ON inventory_document_movements WHEN (SELECT status FROM inventory_documents WHERE id=NEW.document_id)!='READY' BEGIN SELECT RAISE(ABORT,'inventory_post_requires_ready'); END;
CREATE TRIGGER inventory_document_link_no_update BEFORE UPDATE ON inventory_document_movements BEGIN SELECT RAISE(ABORT,'inventory_document_link_immutable'); END;
CREATE TRIGGER inventory_document_link_no_delete BEFORE DELETE ON inventory_document_movements BEGIN SELECT RAISE(ABORT,'inventory_document_link_immutable'); END;
CREATE TRIGGER inventory_post_request_no_update BEFORE UPDATE ON inventory_post_requests BEGIN SELECT RAISE(ABORT,'inventory_post_request_immutable'); END;
CREATE TRIGGER inventory_post_request_no_delete BEFORE DELETE ON inventory_post_requests BEGIN SELECT RAISE(ABORT,'inventory_post_request_immutable'); END;
