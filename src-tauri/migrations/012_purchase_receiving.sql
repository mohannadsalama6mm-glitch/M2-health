CREATE TABLE purchase_receipt_sequences (
 day TEXT PRIMARY KEY NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number > 0)
) STRICT;

CREATE TABLE purchase_receipts (
 id TEXT PRIMARY KEY NOT NULL,
 receipt_number TEXT NOT NULL UNIQUE,
 purchase_order_id TEXT NOT NULL REFERENCES purchase_orders(id),
 branch_id TEXT NOT NULL REFERENCES branches(id),
 request_id TEXT NOT NULL UNIQUE,
 request_payload TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'POSTING' CHECK(status IN ('POSTING','POSTED')),
 total_quantity INTEGER NOT NULL CHECK(total_quantity BETWEEN 1 AND 9007199254740991),
 created_by TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 CHECK((status='POSTED')=(posted_at IS NOT NULL)),
 UNIQUE(id,purchase_order_id,branch_id)
) STRICT;
CREATE INDEX purchase_receipts_order ON purchase_receipts(purchase_order_id,created_at,id);
CREATE INDEX purchase_receipts_branch ON purchase_receipts(branch_id,created_at DESC,id DESC);

CREATE TABLE purchase_receipt_lines (
 id TEXT PRIMARY KEY NOT NULL,
 receipt_id TEXT NOT NULL,
 purchase_order_id TEXT NOT NULL,
 branch_id TEXT NOT NULL,
 purchase_order_line_id TEXT NOT NULL,
 position INTEGER NOT NULL CHECK(position >= 0),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 product_name_snapshot TEXT NOT NULL,
 package_label_snapshot TEXT NOT NULL,
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 lot_number TEXT CHECK(lot_number IS NULL OR length(trim(lot_number)) BETWEEN 1 AND 4000),
 expiry_date TEXT CHECK(expiry_date IS NULL OR (length(expiry_date)=10 AND date(expiry_date,'+0 days') IS expiry_date)),
 acquisition_cost_minor INTEGER NOT NULL CHECK(acquisition_cost_minor BETWEEN 0 AND 9007199254740991),
 batch_id TEXT NOT NULL,
 movement_id TEXT NOT NULL UNIQUE REFERENCES inventory_movements(id),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(receipt_id,position),
 FOREIGN KEY(receipt_id,purchase_order_id,branch_id) REFERENCES purchase_receipts(id,purchase_order_id,branch_id),
 FOREIGN KEY(purchase_order_line_id,purchase_order_id) REFERENCES purchase_order_lines(id,purchase_order_id),
 FOREIGN KEY(batch_id,branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id)
) STRICT;
CREATE INDEX purchase_receipt_lines_order ON purchase_receipt_lines(purchase_order_id,purchase_order_line_id);
CREATE INDEX purchase_receipt_lines_receipt ON purchase_receipt_lines(receipt_id,position);
CREATE INDEX purchase_receipt_lines_batch ON purchase_receipt_lines(batch_id);

DROP TRIGGER purchase_order_guard;
CREATE TRIGGER purchase_order_guard BEFORE UPDATE ON purchase_orders BEGIN
 SELECT CASE WHEN OLD.status IN ('CANCELLED','RECEIVED') THEN RAISE(ABORT,'purchase_order_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.purchase_number IS NOT OLD.purchase_number OR NEW.create_request_id IS NOT OLD.create_request_id OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'purchase_order_identity_immutable') END;
 SELECT CASE WHEN NEW.revision != OLD.revision + 1 THEN RAISE(ABORT,'purchase_order_revision_conflict') END;
 SELECT CASE WHEN NOT (
  (OLD.status='DRAFT' AND NEW.status IN ('DRAFT','ORDERED','CANCELLED')) OR
  (OLD.status='ORDERED' AND NEW.status IN ('PARTIALLY_RECEIVED','RECEIVED','CANCELLED')) OR
  (OLD.status='PARTIALLY_RECEIVED' AND NEW.status IN ('PARTIALLY_RECEIVED','RECEIVED'))
 ) THEN RAISE(ABORT,'purchase_order_invalid_transition') END;
 SELECT CASE WHEN OLD.status IN ('ORDERED','PARTIALLY_RECEIVED') AND (
  NEW.supplier_id IS NOT OLD.supplier_id OR NEW.supplier_code_snapshot IS NOT OLD.supplier_code_snapshot OR NEW.supplier_name_snapshot IS NOT OLD.supplier_name_snapshot OR
  NEW.supplier_reference IS NOT OLD.supplier_reference OR NEW.expected_date IS NOT OLD.expected_date OR NEW.notes IS NOT OLD.notes OR
  NEW.subtotal_minor IS NOT OLD.subtotal_minor OR NEW.discount_minor IS NOT OLD.discount_minor OR NEW.tax_minor IS NOT OLD.tax_minor OR NEW.total_minor IS NOT OLD.total_minor OR
  NEW.ordered_at IS NOT OLD.ordered_at
 ) THEN RAISE(ABORT,'ordered_purchase_frozen') END;
END;

CREATE TRIGGER purchase_receipt_guard BEFORE UPDATE ON purchase_receipts BEGIN
 SELECT CASE WHEN OLD.status!='POSTING' OR NEW.status!='POSTED' THEN RAISE(ABORT,'purchase_receipt_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.receipt_number IS NOT OLD.receipt_number OR NEW.purchase_order_id IS NOT OLD.purchase_order_id OR NEW.branch_id IS NOT OLD.branch_id OR NEW.request_id IS NOT OLD.request_id OR NEW.request_payload IS NOT OLD.request_payload OR NEW.total_quantity IS NOT OLD.total_quantity OR NEW.created_by IS NOT OLD.created_by OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'purchase_receipt_identity_immutable') END;
 SELECT CASE WHEN NEW.total_quantity != (SELECT coalesce(sum(quantity),0) FROM purchase_receipt_lines WHERE receipt_id=OLD.id) THEN RAISE(ABORT,'purchase_receipt_incomplete') END;
END;
CREATE TRIGGER purchase_receipt_no_delete BEFORE DELETE ON purchase_receipts BEGIN SELECT RAISE(ABORT,'purchase_receipt_immutable'); END;
CREATE TRIGGER purchase_receipt_line_insert BEFORE INSERT ON purchase_receipt_lines BEGIN
 SELECT CASE WHEN (SELECT status FROM purchase_receipts WHERE id=NEW.receipt_id)!='POSTING' THEN RAISE(ABORT,'purchase_receipt_lines_require_posting') END;
 SELECT CASE WHEN NOT EXISTS(
  SELECT 1 FROM inventory_movements m WHERE m.id=NEW.movement_id AND m.branch_id=NEW.branch_id AND m.product_package_id=NEW.product_package_id AND m.batch_id=NEW.batch_id AND m.movement_type='PURCHASE_RECEIPT' AND m.quantity_delta=NEW.quantity AND m.reference_type='purchase_receipt' AND m.reference_id=NEW.receipt_id
 ) THEN RAISE(ABORT,'purchase_receipt_movement_mismatch') END;
END;
CREATE TRIGGER purchase_receipt_line_no_update BEFORE UPDATE ON purchase_receipt_lines BEGIN SELECT RAISE(ABORT,'purchase_receipt_line_immutable'); END;
CREATE TRIGGER purchase_receipt_line_no_delete BEFORE DELETE ON purchase_receipt_lines BEGIN SELECT RAISE(ABORT,'purchase_receipt_line_immutable'); END;
