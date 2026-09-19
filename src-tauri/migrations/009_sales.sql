CREATE TABLE sales (
 id TEXT PRIMARY KEY NOT NULL,
 sale_number TEXT UNIQUE,
 branch_id TEXT NOT NULL REFERENCES branches(id),
 customer_id TEXT CHECK(customer_id IS NULL),
 customer_name TEXT,
 status TEXT NOT NULL CHECK(status IN ('DRAFT','HELD','POSTED','VOIDED')),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 draft_json TEXT NOT NULL CHECK(json_valid(draft_json)),
 subtotal_minor INTEGER NOT NULL DEFAULT 0 CHECK(subtotal_minor BETWEEN 0 AND 9007199254740991),
 discount_minor INTEGER NOT NULL DEFAULT 0 CHECK(discount_minor BETWEEN 0 AND subtotal_minor),
 tax_minor INTEGER NOT NULL DEFAULT 0 CHECK(tax_minor BETWEEN 0 AND 9007199254740991),
 total_minor INTEGER NOT NULL DEFAULT 0 CHECK(total_minor BETWEEN 0 AND 9007199254740991 AND total_minor=subtotal_minor-discount_minor+tax_minor),
 payment_status TEXT NOT NULL DEFAULT 'UNPAID' CHECK(payment_status IN ('UNPAID','PAID','REFUNDED')),
 created_by TEXT,
 notes TEXT,
 post_request_id TEXT UNIQUE,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 voided_at TEXT,
 CHECK((status IN ('POSTED','VOIDED'))=(posted_at IS NOT NULL)),
 CHECK((status='VOIDED')=(voided_at IS NOT NULL))
) STRICT;
CREATE INDEX sales_scope ON sales(branch_id,status,created_at,id);
CREATE TABLE sale_sequences(day TEXT PRIMARY KEY NOT NULL,next_number INTEGER NOT NULL CHECK(next_number>0)) STRICT;
CREATE TABLE sale_items (
 id TEXT PRIMARY KEY NOT NULL,
 sale_id TEXT NOT NULL REFERENCES sales(id),
 position INTEGER NOT NULL CHECK(position>=0),
 product_id TEXT NOT NULL REFERENCES products(id),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 product_name_snapshot TEXT NOT NULL,
 package_label_snapshot TEXT NOT NULL,
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 unit_price_minor INTEGER NOT NULL CHECK(unit_price_minor BETWEEN 0 AND 9007199254740991),
 discount_minor INTEGER NOT NULL CHECK(discount_minor>=0),
 sale_discount_minor INTEGER NOT NULL CHECK(sale_discount_minor>=0),
 tax_minor INTEGER NOT NULL CHECK(tax_minor>=0),
 line_total_minor INTEGER NOT NULL CHECK(line_total_minor BETWEEN 0 AND 9007199254740991),
 unit_cost_minor INTEGER CHECK(unit_cost_minor BETWEEN 0 AND 9007199254740991),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(sale_id,position), UNIQUE(sale_id,product_package_id)
) STRICT;
CREATE TABLE sale_item_allocations (
 id TEXT PRIMARY KEY NOT NULL,
 sale_item_id TEXT NOT NULL REFERENCES sale_items(id),
 position INTEGER NOT NULL CHECK(position>=0),
 batch_id TEXT NOT NULL REFERENCES inventory_batches(id),
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 unit_cost_minor INTEGER CHECK(unit_cost_minor BETWEEN 0 AND 9007199254740991),
 movement_id TEXT NOT NULL UNIQUE REFERENCES inventory_movements(id),
 UNIQUE(sale_item_id,position)
) STRICT;
CREATE TABLE sale_payments (
 id TEXT PRIMARY KEY NOT NULL,
 sale_id TEXT NOT NULL REFERENCES sales(id),
 payment_method TEXT NOT NULL CHECK(payment_method IN ('CASH','CARD','OTHER')),
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 0 AND 9007199254740991),
 reference TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE sale_requests (
 request_id TEXT PRIMARY KEY NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('POST','RETURN','VOID')),
 target_id TEXT NOT NULL,
 payload TEXT NOT NULL,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(kind,target_id)
) STRICT;
CREATE TABLE sale_returns (
 id TEXT PRIMARY KEY NOT NULL,
 sale_id TEXT NOT NULL REFERENCES sales(id),
 status TEXT NOT NULL CHECK(status IN ('DRAFT','POSTED')),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 4000),
 lines_json TEXT NOT NULL CHECK(json_valid(lines_json)),
 refund_minor INTEGER NOT NULL DEFAULT 0 CHECK(refund_minor BETWEEN 0 AND 9007199254740991),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT
) STRICT;
CREATE INDEX sale_returns_sale ON sale_returns(sale_id,status);
CREATE TABLE sale_return_items (
 id TEXT PRIMARY KEY NOT NULL,
 return_id TEXT NOT NULL REFERENCES sale_returns(id),
 sale_item_id TEXT NOT NULL REFERENCES sale_items(id),
 quantity INTEGER NOT NULL CHECK(quantity>0),
 refund_minor INTEGER NOT NULL CHECK(refund_minor>=0),
 UNIQUE(return_id,sale_item_id)
) STRICT;
CREATE TABLE sale_return_allocations (
 id TEXT PRIMARY KEY NOT NULL,
 return_item_id TEXT NOT NULL REFERENCES sale_return_items(id),
 original_allocation_id TEXT NOT NULL REFERENCES sale_item_allocations(id),
 quantity INTEGER NOT NULL CHECK(quantity>0),
 movement_id TEXT NOT NULL UNIQUE REFERENCES inventory_movements(id)
) STRICT;
CREATE TABLE sale_voids (
 id TEXT PRIMARY KEY NOT NULL,
 sale_id TEXT NOT NULL UNIQUE REFERENCES sales(id),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 4000),
 refund_minor INTEGER NOT NULL CHECK(refund_minor>=0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE sale_void_allocations (
 id TEXT PRIMARY KEY NOT NULL,
 void_id TEXT NOT NULL REFERENCES sale_voids(id),
 original_allocation_id TEXT NOT NULL UNIQUE REFERENCES sale_item_allocations(id),
 movement_id TEXT NOT NULL UNIQUE REFERENCES inventory_movements(id)
) STRICT;
CREATE TABLE sale_refunds (
 id TEXT PRIMARY KEY NOT NULL,
 sale_id TEXT NOT NULL REFERENCES sales(id),
 return_id TEXT UNIQUE REFERENCES sale_returns(id),
 void_id TEXT UNIQUE REFERENCES sale_voids(id),
 amount_minor INTEGER NOT NULL CHECK(amount_minor>=0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK((return_id IS NOT NULL)!=(void_id IS NOT NULL))
) STRICT;
CREATE TRIGGER sales_no_delete BEFORE DELETE ON sales BEGIN SELECT RAISE(ABORT,'Sales history must be retained'); END;
CREATE TRIGGER sales_frozen BEFORE UPDATE ON sales WHEN OLD.status IN ('POSTED','VOIDED') AND NOT (
 OLD.status='POSTED' AND NEW.status='VOIDED' AND NEW.voided_at IS NOT NULL AND NEW.payment_status='REFUNDED'
 AND NEW.id IS OLD.id AND NEW.sale_number IS OLD.sale_number AND NEW.branch_id IS OLD.branch_id
 AND NEW.customer_id IS OLD.customer_id AND NEW.customer_name IS OLD.customer_name AND NEW.draft_json IS OLD.draft_json
 AND NEW.subtotal_minor IS OLD.subtotal_minor AND NEW.discount_minor IS OLD.discount_minor AND NEW.tax_minor IS OLD.tax_minor AND NEW.total_minor IS OLD.total_minor
 AND NEW.created_by IS OLD.created_by AND NEW.notes IS OLD.notes AND NEW.post_request_id IS OLD.post_request_id AND NEW.created_at IS OLD.created_at AND NEW.posted_at IS OLD.posted_at AND NEW.revision=OLD.revision+1
 AND EXISTS(SELECT 1 FROM sale_voids WHERE sale_id=OLD.id))
 BEGIN SELECT RAISE(ABORT,'Posted sales are immutable'); END;
CREATE TRIGGER sale_items_no_update BEFORE UPDATE ON sale_items BEGIN SELECT RAISE(ABORT,'Sale item snapshots are immutable'); END;
CREATE TRIGGER sale_items_no_delete BEFORE DELETE ON sale_items BEGIN SELECT RAISE(ABORT,'Sale item snapshots are immutable'); END;
CREATE TRIGGER sale_items_insert_guard BEFORE INSERT ON sale_items WHEN (SELECT status FROM sales WHERE id=NEW.sale_id) NOT IN ('DRAFT','HELD') BEGIN SELECT RAISE(ABORT,'Posted sale items are frozen'); END;
CREATE TRIGGER sale_allocation_no_update BEFORE UPDATE ON sale_item_allocations BEGIN SELECT RAISE(ABORT,'Sale allocations are immutable'); END;
CREATE TRIGGER sale_allocation_no_delete BEFORE DELETE ON sale_item_allocations BEGIN SELECT RAISE(ABORT,'Sale allocations are immutable'); END;
CREATE TRIGGER sale_payment_no_update BEFORE UPDATE ON sale_payments BEGIN SELECT RAISE(ABORT,'Payments are immutable'); END;
CREATE TRIGGER sale_payment_no_delete BEFORE DELETE ON sale_payments BEGIN SELECT RAISE(ABORT,'Payments are immutable'); END;
CREATE TRIGGER sale_request_no_update BEFORE UPDATE ON sale_requests BEGIN SELECT RAISE(ABORT,'Request receipts are immutable'); END;
CREATE TRIGGER sale_request_no_delete BEFORE DELETE ON sale_requests BEGIN SELECT RAISE(ABORT,'Request receipts are immutable'); END;
CREATE TRIGGER sale_return_no_update BEFORE UPDATE ON sale_returns WHEN OLD.status='POSTED' BEGIN SELECT RAISE(ABORT,'Posted returns are immutable'); END;
CREATE TRIGGER sale_return_no_delete BEFORE DELETE ON sale_returns BEGIN SELECT RAISE(ABORT,'Returns are retained'); END;
CREATE TRIGGER sale_return_item_no_update BEFORE UPDATE ON sale_return_items BEGIN SELECT RAISE(ABORT,'Return items are immutable'); END;
CREATE TRIGGER sale_return_item_no_delete BEFORE DELETE ON sale_return_items BEGIN SELECT RAISE(ABORT,'Return items are immutable'); END;
CREATE TRIGGER sale_return_allocation_no_update BEFORE UPDATE ON sale_return_allocations BEGIN SELECT RAISE(ABORT,'Return allocations are immutable'); END;
CREATE TRIGGER sale_return_allocation_no_delete BEFORE DELETE ON sale_return_allocations BEGIN SELECT RAISE(ABORT,'Return allocations are immutable'); END;
CREATE TRIGGER sale_void_no_update BEFORE UPDATE ON sale_voids BEGIN SELECT RAISE(ABORT,'Voids are immutable'); END;
CREATE TRIGGER sale_void_no_delete BEFORE DELETE ON sale_voids BEGIN SELECT RAISE(ABORT,'Voids are immutable'); END;
CREATE TRIGGER sale_void_allocation_no_update BEFORE UPDATE ON sale_void_allocations BEGIN SELECT RAISE(ABORT,'Void allocations are immutable'); END;
CREATE TRIGGER sale_void_allocation_no_delete BEFORE DELETE ON sale_void_allocations BEGIN SELECT RAISE(ABORT,'Void allocations are immutable'); END;
CREATE TRIGGER sale_refund_no_update BEFORE UPDATE ON sale_refunds BEGIN SELECT RAISE(ABORT,'Refunds are immutable'); END;
CREATE TRIGGER sale_refund_no_delete BEFORE DELETE ON sale_refunds BEGIN SELECT RAISE(ABORT,'Refunds are immutable'); END;
CREATE TRIGGER sale_allocation_insert_guard BEFORE INSERT ON sale_item_allocations BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM sale_items i JOIN sales s ON s.id=i.sale_id JOIN inventory_movements m ON m.id=NEW.movement_id JOIN inventory_batches b ON b.id=NEW.batch_id WHERE i.id=NEW.sale_item_id AND s.status IN ('DRAFT','HELD') AND m.movement_type='SALE' AND m.branch_id=s.branch_id AND m.product_package_id=i.product_package_id AND m.batch_id=NEW.batch_id AND m.quantity_delta=-NEW.quantity AND m.reference_type='sale_item_allocation' AND m.reference_id=NEW.id AND NEW.unit_cost_minor IS b.cost_price_minor) THEN RAISE(ABORT,'Invalid sale allocation') END;
END;
CREATE TRIGGER sale_payment_insert_guard BEFORE INSERT ON sale_payments WHEN (SELECT status FROM sales WHERE id=NEW.sale_id) NOT IN ('DRAFT','HELD') BEGIN SELECT RAISE(ABORT,'Posted payments are frozen'); END;
CREATE TRIGGER sale_post_guard BEFORE UPDATE OF status ON sales WHEN NEW.status='POSTED' AND OLD.status IN ('DRAFT','HELD') BEGIN
 SELECT CASE WHEN NEW.sale_number IS NULL OR NEW.post_request_id IS NULL OR NEW.payment_status!='PAID' OR NOT EXISTS(SELECT 1 FROM sale_items WHERE sale_id=NEW.id) THEN RAISE(ABORT,'Incomplete sale') END;
 SELECT CASE WHEN (SELECT coalesce(sum(amount_minor),0) FROM sale_payments WHERE sale_id=NEW.id)!=NEW.total_minor OR (SELECT coalesce(sum(line_total_minor),0) FROM sale_items WHERE sale_id=NEW.id)!=NEW.total_minor THEN RAISE(ABORT,'Sale payment/item total mismatch') END;
 SELECT CASE WHEN EXISTS(SELECT 1 FROM sale_items i WHERE i.sale_id=NEW.id AND i.quantity!=(SELECT coalesce(sum(quantity),0) FROM sale_item_allocations WHERE sale_item_id=i.id)) THEN RAISE(ABORT,'Incomplete sale allocations') END;
END;
CREATE TRIGGER sale_return_item_insert_guard BEFORE INSERT ON sale_return_items BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM sale_returns r JOIN sales s ON s.id=r.sale_id JOIN sale_items i ON i.sale_id=s.id WHERE r.id=NEW.return_id AND r.status='DRAFT' AND s.status='POSTED' AND i.id=NEW.sale_item_id AND NEW.quantity+coalesce((SELECT sum(ri.quantity) FROM sale_return_items ri WHERE ri.sale_item_id=i.id),0)<=i.quantity) THEN RAISE(ABORT,'Invalid or excess return item') END;
END;
CREATE TRIGGER sale_return_allocation_insert_guard BEFORE INSERT ON sale_return_allocations BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM sale_return_items ri JOIN sale_returns r ON r.id=ri.return_id JOIN sale_item_allocations a ON a.sale_item_id=ri.sale_item_id JOIN inventory_movements m ON m.id=NEW.movement_id WHERE ri.id=NEW.return_item_id AND r.status='DRAFT' AND a.id=NEW.original_allocation_id AND m.movement_type='SALE_RETURN' AND m.batch_id=a.batch_id AND m.quantity_delta=NEW.quantity AND m.reference_type='sale_return_allocation' AND m.reference_id=NEW.id AND NEW.quantity+coalesce((SELECT sum(ra.quantity) FROM sale_return_allocations ra WHERE ra.original_allocation_id=a.id),0)<=a.quantity) THEN RAISE(ABORT,'Invalid return allocation') END;
END;
CREATE TRIGGER sale_refund_guard BEFORE INSERT ON sale_refunds BEGIN
 SELECT CASE WHEN NEW.amount_minor+coalesce((SELECT sum(amount_minor) FROM sale_refunds WHERE sale_id=NEW.sale_id),0)>(SELECT total_minor FROM sales WHERE id=NEW.sale_id) THEN RAISE(ABORT,'Refund exceeds paid amount') END;
 SELECT CASE WHEN NEW.return_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM sale_returns WHERE id=NEW.return_id AND sale_id=NEW.sale_id AND status='DRAFT' AND refund_minor=NEW.amount_minor) THEN RAISE(ABORT,'Invalid return refund') END;
 SELECT CASE WHEN NEW.void_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM sale_voids WHERE id=NEW.void_id AND sale_id=NEW.sale_id AND refund_minor=NEW.amount_minor) THEN RAISE(ABORT,'Invalid void refund') END;
END;
CREATE TRIGGER sale_return_post_guard BEFORE UPDATE OF status ON sale_returns WHEN NEW.status='POSTED' BEGIN
 SELECT CASE WHEN NEW.refund_minor!=(SELECT coalesce(sum(refund_minor),0) FROM sale_return_items WHERE return_id=NEW.id) OR NOT EXISTS(SELECT 1 FROM sale_refunds WHERE return_id=NEW.id) THEN RAISE(ABORT,'Incomplete return') END;
 SELECT CASE WHEN EXISTS(SELECT 1 FROM sale_return_items i WHERE i.return_id=NEW.id AND i.quantity!=(SELECT coalesce(sum(quantity),0) FROM sale_return_allocations WHERE return_item_id=i.id)) THEN RAISE(ABORT,'Incomplete return allocations') END;
END;
CREATE TRIGGER sale_void_allocation_guard BEFORE INSERT ON sale_void_allocations BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM sale_voids v JOIN sale_items i ON i.sale_id=v.sale_id JOIN sale_item_allocations a ON a.sale_item_id=i.id JOIN inventory_movements m ON m.id=NEW.movement_id JOIN sales s ON s.id=v.sale_id WHERE v.id=NEW.void_id AND a.id=NEW.original_allocation_id AND s.status='POSTED' AND m.batch_id=a.batch_id AND m.quantity_delta=a.quantity AND m.movement_type='SALE_RETURN' AND m.reference_type='sale_void_allocation' AND m.reference_id=NEW.id) THEN RAISE(ABORT,'Invalid void allocation') END;
END;
