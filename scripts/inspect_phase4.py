"""Read-only emergency inspection for the controlled Phase 4 native smoke."""
import json
import os
import sqlite3
from pathlib import Path

path = Path(os.environ["LOCALAPPDATA"]) / "com.m2health.pharmacy" / "m2-health.db"
db = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)
result = {
    "versions": db.execute("SELECT version,name FROM _schema_version ORDER BY version").fetchall(),
    "sales": db.execute("SELECT id,sale_number,branch_id,status,revision,total_minor,post_request_id,created_at FROM sales ORDER BY created_at").fetchall(),
    "items": db.execute("SELECT sale_id,product_package_id,quantity,line_total_minor FROM sale_items").fetchall(),
    "payments": db.execute("SELECT sale_id,payment_method,amount_minor FROM sale_payments").fetchall(),
    "allocations": db.execute("SELECT sale_item_id,batch_id,quantity,movement_id FROM sale_item_allocations").fetchall(),
    "requests": db.execute("SELECT request_id,kind,target_id,payload FROM sale_requests ORDER BY created_at").fetchall(),
    "returns": db.execute("SELECT id,sale_id,status,reason,refund_minor,posted_at FROM sale_returns ORDER BY created_at").fetchall(),
    "return_items": db.execute("SELECT id,return_id,sale_item_id,quantity,refund_minor FROM sale_return_items").fetchall(),
    "return_allocations": db.execute("SELECT return_item_id,original_allocation_id,quantity,movement_id FROM sale_return_allocations").fetchall(),
    "refunds": db.execute("SELECT sale_id,return_id,void_id,amount_minor FROM sale_refunds").fetchall(),
    "movements": db.execute("SELECT id,branch_id,movement_type,quantity_delta,batch_id,reference_type,reference_id FROM inventory_movements WHERE movement_type IN ('SALE','SALE_RETURN') ORDER BY created_at").fetchall(),
    "stock": dict(db.execute("SELECT branch_id,sum(quantity_delta) FROM inventory_movements GROUP BY branch_id")),
}
baseline = sqlite3.connect((Path(__file__).resolve().parents[1] / "backups/phase-4/before-20260915T133426Z.sqlite3").as_uri() + "?mode=ro", uri=True)
alert_columns = "id,branch_id,alert_type,product_package_id,batch_id,status,severity,fingerprint,first_detected_at,last_detected_at,resolved_at,details_json,created_at,updated_at"
result["alerts_before"] = baseline.execute(f"SELECT {alert_columns} FROM inventory_alerts ORDER BY id").fetchall()
result["alerts_after"] = db.execute(f"SELECT {alert_columns} FROM inventory_alerts ORDER BY id").fetchall()
print(json.dumps(result, indent=2))
