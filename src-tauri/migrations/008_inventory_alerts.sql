CREATE TABLE inventory_alert_settings (
 branch_id TEXT PRIMARY KEY NOT NULL REFERENCES branches(id),
 expiry_warning_days INTEGER NOT NULL DEFAULT 90 CHECK(expiry_warning_days BETWEEN 0 AND 36500),
 critical_expiry_days INTEGER NOT NULL DEFAULT 30 CHECK(critical_expiry_days BETWEEN 0 AND expiry_warning_days),
 expiry_enabled INTEGER NOT NULL DEFAULT 1 CHECK(expiry_enabled IN (0,1)),
 low_stock_enabled INTEGER NOT NULL DEFAULT 1 CHECK(low_stock_enabled IN (0,1)),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE inventory_alerts (
 id TEXT PRIMARY KEY NOT NULL,
 branch_id TEXT NOT NULL REFERENCES branches(id),
 alert_type TEXT NOT NULL CHECK(alert_type IN ('BATCH_EXPIRED','BATCH_EXPIRING','LOW_STOCK','OUT_OF_STOCK')),
 product_package_id TEXT NOT NULL REFERENCES product_packages(id),
 batch_id TEXT,
 severity TEXT NOT NULL CHECK(severity IN ('CRITICAL','HIGH','MEDIUM')),
 status TEXT NOT NULL CHECK(status IN ('ACTIVE','RESOLVED','DISMISSED')),
 fingerprint TEXT NOT NULL UNIQUE,
 details_json TEXT NOT NULL CHECK(json_valid(details_json)),
 first_detected_at TEXT NOT NULL,
 last_detected_at TEXT NOT NULL,
 resolved_at TEXT,
 created_at TEXT NOT NULL,
 updated_at TEXT NOT NULL,
 FOREIGN KEY(batch_id,branch_id,product_package_id) REFERENCES inventory_batches(id,branch_id,product_package_id),
 CHECK((alert_type IN ('BATCH_EXPIRED','BATCH_EXPIRING')) = (batch_id IS NOT NULL)),
 CHECK((status='RESOLVED') = (resolved_at IS NOT NULL))
) STRICT;
CREATE INDEX inventory_alerts_scope ON inventory_alerts(branch_id,status,alert_type,updated_at,id);
CREATE TABLE inventory_alert_history (
 id TEXT PRIMARY KEY NOT NULL,
 alert_id TEXT NOT NULL REFERENCES inventory_alerts(id),
 status TEXT NOT NULL CHECK(status IN ('ACTIVE','RESOLVED','DISMISSED')),
 severity TEXT NOT NULL CHECK(severity IN ('CRITICAL','HIGH','MEDIUM')),
 details_json TEXT NOT NULL CHECK(json_valid(details_json)),
 occurred_at TEXT NOT NULL
) STRICT;
CREATE INDEX inventory_alert_history_scope ON inventory_alert_history(alert_id,occurred_at,id);
CREATE TRIGGER inventory_alert_no_delete BEFORE DELETE ON inventory_alerts BEGIN SELECT RAISE(ABORT,'Alert history must be retained'); END;
CREATE TRIGGER inventory_alert_history_no_update BEFORE UPDATE ON inventory_alert_history BEGIN SELECT RAISE(ABORT,'Alert events are immutable'); END;
CREATE TRIGGER inventory_alert_history_no_delete BEFORE DELETE ON inventory_alert_history BEGIN SELECT RAISE(ABORT,'Alert events are immutable'); END;
CREATE VIEW inventory_alert_batch_view AS
 SELECT b.*,p.commercial_name_en product_name,k.package_label,
 coalesce((SELECT sum(m.quantity_delta) FROM inventory_movements m INDEXED BY inventory_movements_batch WHERE m.batch_id=b.id),0) quantity
 FROM inventory_batches b JOIN product_packages k ON k.id=b.product_package_id JOIN products p ON p.id=k.product_id;
CREATE VIEW inventory_alert_level_view AS
 SELECT l.*,p.commercial_name_en product_name,k.package_label,
 coalesce((SELECT sum(m.quantity_delta) FROM inventory_movements m INDEXED BY inventory_movements_package WHERE m.branch_id=l.branch_id AND m.product_package_id=l.product_package_id),0) quantity
 FROM inventory_levels l JOIN product_packages k ON k.id=l.product_package_id JOIN products p ON p.id=k.product_id;
