CREATE TABLE purchase_order_sequences (
 day TEXT PRIMARY KEY NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number > 0)
) STRICT;

CREATE TABLE purchase_orders (
 id TEXT PRIMARY KEY NOT NULL,
 purchase_number TEXT NOT NULL UNIQUE,
 supplier_id TEXT NOT NULL REFERENCES suppliers(id),
 supplier_code_snapshot TEXT,
 supplier_name_snapshot TEXT,
 status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT','ORDERED','PARTIALLY_RECEIVED','RECEIVED','CANCELLED')),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
 supplier_reference TEXT,
 expected_date TEXT CHECK(expected_date IS NULL OR (length(expected_date)=10 AND date(expected_date,'+0 days') IS expected_date)),
 notes TEXT,
 subtotal_minor INTEGER NOT NULL CHECK(subtotal_minor BETWEEN 0 AND 9007199254740991),
 discount_minor INTEGER NOT NULL DEFAULT 0 CHECK(discount_minor BETWEEN 0 AND subtotal_minor),
 tax_minor INTEGER NOT NULL DEFAULT 0 CHECK(tax_minor BETWEEN 0 AND 9007199254740991),
 total_minor INTEGER NOT NULL CHECK(total_minor=subtotal_minor-discount_minor+tax_minor AND total_minor BETWEEN 0 AND 9007199254740991),
 create_request_id TEXT NOT NULL UNIQUE,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 ordered_at TEXT,
 cancelled_at TEXT,
 cancellation_reason TEXT,
 CHECK(status NOT IN ('ORDERED','PARTIALLY_RECEIVED','RECEIVED') OR ordered_at IS NOT NULL),
 CHECK((status='CANCELLED')=(cancelled_at IS NOT NULL)),
 CHECK(status!='CANCELLED' OR length(trim(cancellation_reason)) BETWEEN 1 AND 2000),
 CHECK(status!='ORDERED' OR (supplier_code_snapshot IS NOT NULL AND supplier_name_snapshot IS NOT NULL))
) STRICT;
CREATE INDEX purchase_orders_list ON purchase_orders(created_at DESC,id DESC);
CREATE INDEX purchase_orders_supplier ON purchase_orders(supplier_id,status,created_at DESC,id DESC);
CREATE INDEX purchase_orders_status ON purchase_orders(status,created_at DESC,id DESC);

CREATE TABLE purchase_order_lines (
 id TEXT PRIMARY KEY NOT NULL,
 purchase_order_id TEXT NOT NULL REFERENCES purchase_orders(id),
 position INTEGER NOT NULL CHECK(position >= 0),
 product_id TEXT NOT NULL REFERENCES products(id),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 product_name_snapshot TEXT NOT NULL,
 package_label_snapshot TEXT NOT NULL,
 ordered_quantity INTEGER NOT NULL CHECK(ordered_quantity BETWEEN 1 AND 9007199254740991),
 unit_cost_minor INTEGER NOT NULL CHECK(unit_cost_minor BETWEEN 0 AND 9007199254740991),
 line_total_minor INTEGER NOT NULL CHECK(line_total_minor BETWEEN 0 AND 9007199254740991 AND line_total_minor=ordered_quantity*unit_cost_minor),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(purchase_order_id,position),
 UNIQUE(purchase_order_id,product_package_id),
 UNIQUE(id,purchase_order_id)
) STRICT;
CREATE INDEX purchase_order_lines_order ON purchase_order_lines(purchase_order_id,position);

CREATE TABLE purchase_order_requests (
 request_id TEXT PRIMARY KEY NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('CREATE','ORDER','CANCEL')),
 purchase_order_id TEXT NOT NULL REFERENCES purchase_orders(id),
 payload TEXT NOT NULL,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(kind,purchase_order_id)
) STRICT;

CREATE TRIGGER purchase_orders_no_delete BEFORE DELETE ON purchase_orders BEGIN
 SELECT RAISE(ABORT,'purchase_order_history_must_be_retained');
END;
CREATE TRIGGER purchase_order_guard BEFORE UPDATE ON purchase_orders BEGIN
 SELECT CASE WHEN OLD.status IN ('CANCELLED','PARTIALLY_RECEIVED','RECEIVED') THEN RAISE(ABORT,'purchase_order_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.purchase_number IS NOT OLD.purchase_number OR NEW.create_request_id IS NOT OLD.create_request_id OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'purchase_order_identity_immutable') END;
 SELECT CASE WHEN NEW.revision != OLD.revision + 1 THEN RAISE(ABORT,'purchase_order_revision_conflict') END;
 SELECT CASE WHEN NOT ((OLD.status='DRAFT' AND NEW.status IN ('DRAFT','ORDERED','CANCELLED')) OR (OLD.status='ORDERED' AND NEW.status='CANCELLED')) THEN RAISE(ABORT,'purchase_order_invalid_transition') END;
 SELECT CASE WHEN OLD.status='ORDERED' AND (NEW.supplier_id IS NOT OLD.supplier_id OR NEW.supplier_reference IS NOT OLD.supplier_reference OR NEW.expected_date IS NOT OLD.expected_date OR NEW.notes IS NOT OLD.notes OR NEW.subtotal_minor IS NOT OLD.subtotal_minor OR NEW.discount_minor IS NOT OLD.discount_minor OR NEW.tax_minor IS NOT OLD.tax_minor OR NEW.total_minor IS NOT OLD.total_minor) THEN RAISE(ABORT,'ordered_purchase_frozen') END;
END;
CREATE TRIGGER purchase_order_line_insert_guard BEFORE INSERT ON purchase_order_lines WHEN (SELECT status FROM purchase_orders WHERE id=NEW.purchase_order_id)!='DRAFT' BEGIN SELECT RAISE(ABORT,'purchase_lines_require_draft'); END;
CREATE TRIGGER purchase_order_line_update_guard BEFORE UPDATE ON purchase_order_lines WHEN (SELECT status FROM purchase_orders WHERE id=OLD.purchase_order_id)!='DRAFT' BEGIN SELECT RAISE(ABORT,'purchase_lines_require_draft'); END;
CREATE TRIGGER purchase_order_line_delete_guard BEFORE DELETE ON purchase_order_lines WHEN (SELECT status FROM purchase_orders WHERE id=OLD.purchase_order_id)!='DRAFT' BEGIN SELECT RAISE(ABORT,'purchase_lines_require_draft'); END;
CREATE TRIGGER purchase_order_requests_no_update BEFORE UPDATE ON purchase_order_requests BEGIN SELECT RAISE(ABORT,'purchase_request_immutable'); END;
CREATE TRIGGER purchase_order_requests_no_delete BEFORE DELETE ON purchase_order_requests BEGIN SELECT RAISE(ABORT,'purchase_request_immutable'); END;
