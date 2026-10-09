CREATE TABLE purchase_return_sequences (
 day TEXT PRIMARY KEY NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number > 0)
) STRICT;

CREATE TABLE purchase_returns (
 id TEXT PRIMARY KEY NOT NULL,
 return_number TEXT NOT NULL UNIQUE,
 purchase_order_id TEXT NOT NULL REFERENCES purchase_orders(id),
 receipt_id TEXT NOT NULL,
 branch_id TEXT NOT NULL REFERENCES branches(id),
 supplier_id TEXT NOT NULL REFERENCES suppliers(id),
 request_id TEXT NOT NULL UNIQUE,
 request_payload TEXT NOT NULL,
 source_purchase_revision INTEGER NOT NULL CHECK(source_purchase_revision > 0),
 status TEXT NOT NULL DEFAULT 'POSTING' CHECK(status IN ('POSTING','POSTED')),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 2000),
 total_quantity INTEGER NOT NULL CHECK(total_quantity BETWEEN 1 AND 9007199254740991),
 created_by TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 CHECK((status='POSTED')=(posted_at IS NOT NULL)),
 UNIQUE(id,purchase_order_id,receipt_id,branch_id),
 FOREIGN KEY(receipt_id,purchase_order_id,branch_id) REFERENCES purchase_receipts(id,purchase_order_id,branch_id)
) STRICT;
CREATE INDEX purchase_returns_purchase ON purchase_returns(purchase_order_id,posted_at DESC,id DESC);
CREATE INDEX purchase_returns_receipt ON purchase_returns(receipt_id,posted_at DESC,id DESC);
CREATE INDEX purchase_returns_supplier ON purchase_returns(supplier_id,posted_at DESC,id DESC);
CREATE INDEX purchase_returns_branch ON purchase_returns(branch_id,posted_at DESC,id DESC);
CREATE INDEX purchase_returns_number ON purchase_returns(return_number);

CREATE UNIQUE INDEX purchase_receipt_lines_return_identity
ON purchase_receipt_lines(id,receipt_id,purchase_order_id,branch_id,product_package_id,batch_id);

CREATE TABLE purchase_return_lines (
 id TEXT PRIMARY KEY NOT NULL,
 return_id TEXT NOT NULL,
 purchase_order_id TEXT NOT NULL,
 receipt_id TEXT NOT NULL,
 branch_id TEXT NOT NULL,
 receipt_line_id TEXT NOT NULL,
 purchase_order_line_id TEXT NOT NULL,
 position INTEGER NOT NULL CHECK(position >= 0),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 product_name_snapshot TEXT NOT NULL,
 package_label_snapshot TEXT NOT NULL,
 previously_returned_quantity INTEGER NOT NULL CHECK(previously_returned_quantity BETWEEN 0 AND 9007199254740991),
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 lot_number TEXT,
 expiry_date TEXT,
 acquisition_cost_minor INTEGER NOT NULL CHECK(acquisition_cost_minor BETWEEN 0 AND 9007199254740991),
 batch_id TEXT NOT NULL,
 movement_id TEXT NOT NULL UNIQUE REFERENCES inventory_movements(id),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(return_id,position),
 UNIQUE(return_id,receipt_line_id),
 FOREIGN KEY(return_id,purchase_order_id,receipt_id,branch_id) REFERENCES purchase_returns(id,purchase_order_id,receipt_id,branch_id),
 FOREIGN KEY(receipt_line_id,receipt_id,purchase_order_id,branch_id,product_package_id,batch_id) REFERENCES purchase_receipt_lines(id,receipt_id,purchase_order_id,branch_id,product_package_id,batch_id),
 FOREIGN KEY(purchase_order_line_id,purchase_order_id) REFERENCES purchase_order_lines(id,purchase_order_id),
 FOREIGN KEY(batch_id,branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id)
) STRICT;
CREATE INDEX purchase_return_lines_receipt_line ON purchase_return_lines(receipt_line_id);
CREATE INDEX purchase_return_lines_batch ON purchase_return_lines(batch_id);

DROP TRIGGER purchase_order_guard;
CREATE TRIGGER purchase_order_guard BEFORE UPDATE ON purchase_orders BEGIN
 SELECT CASE WHEN OLD.status='CANCELLED' THEN RAISE(ABORT,'purchase_order_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.purchase_number IS NOT OLD.purchase_number OR NEW.create_request_id IS NOT OLD.create_request_id OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'purchase_order_identity_immutable') END;
 SELECT CASE WHEN NEW.revision != OLD.revision + 1 THEN RAISE(ABORT,'purchase_order_revision_conflict') END;
 SELECT CASE WHEN NOT (
  (OLD.status='DRAFT' AND NEW.status IN ('DRAFT','ORDERED','CANCELLED')) OR
  (OLD.status='ORDERED' AND NEW.status IN ('PARTIALLY_RECEIVED','RECEIVED','CANCELLED')) OR
  (OLD.status='PARTIALLY_RECEIVED' AND NEW.status IN ('PARTIALLY_RECEIVED','RECEIVED')) OR
  (OLD.status='RECEIVED' AND NEW.status='RECEIVED')
 ) THEN RAISE(ABORT,'purchase_order_invalid_transition') END;
 SELECT CASE WHEN OLD.status IN ('ORDERED','PARTIALLY_RECEIVED','RECEIVED') AND (
  NEW.supplier_id IS NOT OLD.supplier_id OR NEW.supplier_code_snapshot IS NOT OLD.supplier_code_snapshot OR NEW.supplier_name_snapshot IS NOT OLD.supplier_name_snapshot OR
  NEW.supplier_reference IS NOT OLD.supplier_reference OR NEW.expected_date IS NOT OLD.expected_date OR NEW.notes IS NOT OLD.notes OR
  NEW.subtotal_minor IS NOT OLD.subtotal_minor OR NEW.discount_minor IS NOT OLD.discount_minor OR NEW.tax_minor IS NOT OLD.tax_minor OR NEW.total_minor IS NOT OLD.total_minor OR
  NEW.ordered_at IS NOT OLD.ordered_at
 ) THEN RAISE(ABORT,'ordered_purchase_frozen') END;
END;

CREATE TRIGGER purchase_return_guard BEFORE UPDATE ON purchase_returns BEGIN
 SELECT CASE WHEN OLD.status!='POSTING' OR NEW.status!='POSTED' THEN RAISE(ABORT,'purchase_return_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.return_number IS NOT OLD.return_number OR NEW.purchase_order_id IS NOT OLD.purchase_order_id OR NEW.receipt_id IS NOT OLD.receipt_id OR NEW.branch_id IS NOT OLD.branch_id OR NEW.supplier_id IS NOT OLD.supplier_id OR NEW.request_id IS NOT OLD.request_id OR NEW.request_payload IS NOT OLD.request_payload OR NEW.source_purchase_revision IS NOT OLD.source_purchase_revision OR NEW.reason IS NOT OLD.reason OR NEW.total_quantity IS NOT OLD.total_quantity OR NEW.created_by IS NOT OLD.created_by OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'purchase_return_identity_immutable') END;
 SELECT CASE WHEN NEW.total_quantity != (SELECT coalesce(sum(quantity),0) FROM purchase_return_lines WHERE return_id=OLD.id) THEN RAISE(ABORT,'purchase_return_incomplete') END;
END;
CREATE TRIGGER purchase_return_no_delete BEFORE DELETE ON purchase_returns BEGIN SELECT RAISE(ABORT,'purchase_return_immutable'); END;
CREATE TRIGGER purchase_return_line_insert BEFORE INSERT ON purchase_return_lines BEGIN
 SELECT CASE WHEN (SELECT status FROM purchase_returns WHERE id=NEW.return_id)!='POSTING' THEN RAISE(ABORT,'purchase_return_lines_require_posting') END;
 SELECT CASE WHEN NEW.previously_returned_quantity != coalesce((SELECT sum(rl.quantity) FROM purchase_return_lines rl JOIN purchase_returns r ON r.id=rl.return_id WHERE rl.receipt_line_id=NEW.receipt_line_id AND r.status='POSTED'),0) THEN RAISE(ABORT,'purchase_return_prior_mismatch') END;
 SELECT CASE WHEN NEW.quantity + coalesce((SELECT sum(rl.quantity) FROM purchase_return_lines rl JOIN purchase_returns r ON r.id=rl.return_id WHERE rl.receipt_line_id=NEW.receipt_line_id AND r.status='POSTED'),0) > (SELECT quantity FROM purchase_receipt_lines WHERE id=NEW.receipt_line_id) THEN RAISE(ABORT,'purchase_return_quantity_exceeded') END;
 SELECT CASE WHEN NOT EXISTS(
  SELECT 1 FROM inventory_movements m WHERE m.id=NEW.movement_id AND m.branch_id=NEW.branch_id AND m.product_package_id=NEW.product_package_id AND m.batch_id=NEW.batch_id AND m.movement_type='PURCHASE_RETURN' AND m.quantity_delta=-NEW.quantity AND m.reference_type='purchase_return' AND m.reference_id=NEW.return_id
 ) THEN RAISE(ABORT,'purchase_return_movement_mismatch') END;
END;
CREATE TRIGGER purchase_return_line_no_update BEFORE UPDATE ON purchase_return_lines BEGIN SELECT RAISE(ABORT,'purchase_return_line_immutable'); END;
CREATE TRIGGER purchase_return_line_no_delete BEFORE DELETE ON purchase_return_lines BEGIN SELECT RAISE(ABORT,'purchase_return_line_immutable'); END;
