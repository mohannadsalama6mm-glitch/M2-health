CREATE TABLE inventory_batches (
 id TEXT PRIMARY KEY NOT NULL,
 branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
 product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
 lot_number TEXT CHECK(lot_number IS NULL OR length(trim(lot_number)) BETWEEN 1 AND 4000),
 expiry_date TEXT CHECK(expiry_date IS NULL OR (length(expiry_date)=10 AND date(expiry_date,'+0 days') IS expiry_date)),
 received_at TEXT CHECK(received_at IS NULL OR (length(received_at)=24 AND strftime('%Y-%m-%dT%H:%M:%fZ',received_at) IS received_at)),
 cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
 supplier_reference TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(id,branch_id,product_package_id)
) STRICT;
CREATE INDEX inventory_batches_scope ON inventory_batches(branch_id,product_package_id);
CREATE INDEX inventory_batches_expiry ON inventory_batches(branch_id,expiry_date);
CREATE TRIGGER inventory_batch_identity_immutable BEFORE UPDATE ON inventory_batches
 WHEN NEW.id IS NOT OLD.id OR NEW.branch_id IS NOT OLD.branch_id OR NEW.product_package_id IS NOT OLD.product_package_id
 OR NEW.lot_number IS NOT OLD.lot_number OR NEW.expiry_date IS NOT OLD.expiry_date OR NEW.received_at IS NOT OLD.received_at
 OR NEW.cost_price_minor IS NOT OLD.cost_price_minor OR NEW.supplier_reference IS NOT OLD.supplier_reference OR NEW.created_at IS NOT OLD.created_at
 BEGIN SELECT RAISE(ABORT,'inventory_batch_identity_immutable'); END;

CREATE TABLE inventory_transfers (
 id TEXT PRIMARY KEY NOT NULL,
 source_branch_id TEXT NOT NULL REFERENCES branches(id),
 destination_branch_id TEXT NOT NULL REFERENCES branches(id),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 source_batch_id TEXT NOT NULL,
 destination_batch_id TEXT NOT NULL,
 quantity INTEGER NOT NULL CHECK(quantity BETWEEN 1 AND 9007199254740991),
 out_movement_id TEXT NOT NULL UNIQUE,
 in_movement_id TEXT NOT NULL UNIQUE,
 out_type TEXT NOT NULL DEFAULT 'TRANSFER_OUT' CHECK(out_type='TRANSFER_OUT'),
 in_type TEXT NOT NULL DEFAULT 'TRANSFER_IN' CHECK(in_type='TRANSFER_IN'),
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 4000),
 created_by TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK(source_branch_id != destination_branch_id AND out_movement_id != in_movement_id),
 FOREIGN KEY(out_movement_id,id,out_type) REFERENCES inventory_movements(id,transfer_id,movement_type) DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(in_movement_id,id,in_type) REFERENCES inventory_movements(id,transfer_id,movement_type) DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(source_batch_id,source_branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id),
 FOREIGN KEY(destination_batch_id,destination_branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id)
) STRICT;
CREATE TABLE inventory_movements (
 id TEXT PRIMARY KEY NOT NULL,
 branch_id TEXT NOT NULL REFERENCES branches(id),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 batch_id TEXT NOT NULL,
 movement_type TEXT NOT NULL CHECK(movement_type IN ('OPENING','PURCHASE_RECEIPT','SALE','SALE_RETURN','PURCHASE_RETURN','ADJUSTMENT_IN','ADJUSTMENT_OUT','DAMAGE','EXPIRY','TRANSFER_OUT','TRANSFER_IN','STOCK_COUNT_CORRECTION')),
 quantity_delta INTEGER NOT NULL CHECK(quantity_delta != 0 AND quantity_delta BETWEEN -9007199254740991 AND 9007199254740991),
 reference_type TEXT,
 reference_id TEXT,
 reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 4000),
 created_by TEXT,
 transfer_id TEXT REFERENCES inventory_transfers(id),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(id,transfer_id,movement_type),
 FOREIGN KEY(batch_id,branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id),
 CHECK((movement_type IN ('OPENING','PURCHASE_RECEIPT','SALE_RETURN','ADJUSTMENT_IN','TRANSFER_IN') AND quantity_delta > 0)
 OR (movement_type IN ('SALE','PURCHASE_RETURN','ADJUSTMENT_OUT','DAMAGE','EXPIRY','TRANSFER_OUT') AND quantity_delta < 0)
 OR movement_type='STOCK_COUNT_CORRECTION'),
 CHECK((movement_type IN ('TRANSFER_IN','TRANSFER_OUT')) = (transfer_id IS NOT NULL)),
 CHECK((reference_type IS NULL) = (reference_id IS NULL))
) STRICT;
CREATE INDEX inventory_movements_package ON inventory_movements(branch_id,product_package_id);
CREATE INDEX inventory_movements_batch ON inventory_movements(batch_id);
CREATE INDEX inventory_movements_history ON inventory_movements(branch_id,created_at,id);
CREATE UNIQUE INDEX inventory_movements_opening ON inventory_movements(batch_id) WHERE movement_type='OPENING';
CREATE UNIQUE INDEX inventory_movements_transfer_side ON inventory_movements(transfer_id,movement_type) WHERE transfer_id IS NOT NULL;
CREATE TRIGGER inventory_movement_guard BEFORE INSERT ON inventory_movements BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM branches b JOIN product_packages pk ON pk.id=NEW.product_package_id JOIN products p ON p.id=pk.product_id JOIN inventory_batches ib ON ib.id=NEW.batch_id
 WHERE b.id=NEW.branch_id AND b.is_active=1 AND pk.is_active=1 AND p.is_active=1 AND ib.is_active=1)
 THEN RAISE(ABORT,'inventory_inactive_scope') END;
 SELECT CASE WHEN NEW.movement_type='OPENING' AND EXISTS(SELECT 1 FROM inventory_movements WHERE batch_id=NEW.batch_id)
 THEN RAISE(ABORT,'opening_requires_empty_batch') END;
 SELECT CASE WHEN (SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements INDEXED BY inventory_movements_batch WHERE batch_id=NEW.batch_id)+NEW.quantity_delta NOT BETWEEN 0 AND 9007199254740991
 THEN RAISE(ABORT,'inventory_batch_balance_range') END;
 SELECT CASE WHEN (SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements INDEXED BY inventory_movements_package WHERE branch_id=NEW.branch_id AND product_package_id=NEW.product_package_id)+NEW.quantity_delta NOT BETWEEN 0 AND 9007199254740991
 THEN RAISE(ABORT,'inventory_package_balance_range') END;
 SELECT CASE WHEN NEW.transfer_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM inventory_transfers t WHERE t.id=NEW.transfer_id AND t.product_package_id=NEW.product_package_id
 AND ((NEW.movement_type='TRANSFER_OUT' AND NEW.id=t.out_movement_id AND NEW.branch_id=t.source_branch_id AND NEW.batch_id=t.source_batch_id AND NEW.quantity_delta=-t.quantity)
 OR (NEW.movement_type='TRANSFER_IN' AND NEW.id=t.in_movement_id AND NEW.branch_id=t.destination_branch_id AND NEW.batch_id=t.destination_batch_id AND NEW.quantity_delta=t.quantity)))
 THEN RAISE(ABORT,'inventory_transfer_mismatch') END;
 END;
CREATE TRIGGER inventory_movement_no_update BEFORE UPDATE ON inventory_movements BEGIN SELECT RAISE(ABORT,'inventory_ledger_immutable'); END;
CREATE TRIGGER inventory_movement_no_delete BEFORE DELETE ON inventory_movements BEGIN SELECT RAISE(ABORT,'inventory_ledger_immutable'); END;
CREATE TRIGGER inventory_transfer_no_update BEFORE UPDATE ON inventory_transfers BEGIN SELECT RAISE(ABORT,'inventory_transfer_immutable'); END;
CREATE TRIGGER inventory_transfer_no_delete BEFORE DELETE ON inventory_transfers BEGIN SELECT RAISE(ABORT,'inventory_transfer_immutable'); END;

CREATE TABLE inventory_levels (
 id TEXT PRIMARY KEY NOT NULL,
 branch_id TEXT NOT NULL REFERENCES branches(id),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 reorder_level INTEGER NOT NULL CHECK(reorder_level BETWEEN 0 AND 9007199254740991),
 minimum_stock INTEGER CHECK(minimum_stock BETWEEN 0 AND 9007199254740991),
 maximum_stock INTEGER CHECK(maximum_stock BETWEEN 0 AND 9007199254740991),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(branch_id,product_package_id),
 CHECK(minimum_stock IS NULL OR maximum_stock IS NULL OR minimum_stock <= maximum_stock)
) STRICT;
