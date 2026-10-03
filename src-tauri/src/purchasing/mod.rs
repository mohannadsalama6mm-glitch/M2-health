//! Canonical Phase 5B purchase orders. This module never writes Inventory.
pub mod commands;
pub mod models;
use crate::{
    db::{catalog_validation::*, connection::AppDb},
    error::{AppError, ErrorCode},
};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashSet;

fn clean(value: &Option<String>, max: usize, label: &str) -> Result<Option<String>, AppError> {
    let value = value.as_deref().unwrap_or("").trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max || value.contains('\0') {
        return Err(invalid(&format!(
            "{label} must be at most {max} characters."
        )));
    }
    Ok(Some(value.into()))
}
fn money(value: i128) -> Result<i64, AppError> {
    i64::try_from(value)
        .ok()
        .filter(|v| (0..=MAX_SAFE_MINOR).contains(v))
        .ok_or_else(|| invalid("Money values must be non-negative safe integers."))
}
fn page(limit: Option<i64>, offset: Option<i64>) -> Result<(i64, i64), AppError> {
    let l = limit.unwrap_or(25);
    let o = offset.unwrap_or(0);
    if !(1..=100).contains(&l) || !(0..=MAX_SAFE_MINOR).contains(&o) {
        return Err(invalid("Use a limit of 1–100 and a non-negative offset."));
    }
    Ok((l, o))
}
fn valid_date(c: &Connection, value: &Option<String>) -> Result<Option<String>, AppError> {
    let value = clean(value, 10, "Expected date")?;
    if let Some(v) = &value {
        let ok: bool = c.query_row(
            "SELECT length(?1)=10 AND date(?1,'+0 days') IS ?1",
            [v],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(invalid("Expected date must be a real YYYY-MM-DD date."));
        }
    }
    Ok(value)
}
type PreparedLine = (String, String, String, String, String, i64, i64, i64);
struct Prepared {
    supplier_id: String,
    supplier_code: String,
    supplier_name: String,
    reference: Option<String>,
    expected: Option<String>,
    notes: Option<String>,
    subtotal: i64,
    discount: i64,
    tax: i64,
    total: i64,
    lines: Vec<PreparedLine>,
}
fn prepare(
    c: &Connection,
    f: &PurchaseDraftFields,
    require_lines: bool,
) -> Result<Prepared, AppError> {
    id(&f.supplier_id)?;
    let supplier: Option<(String, String, bool)> = c
        .query_row(
            "SELECT code,display_name,is_active FROM suppliers WHERE id=?1",
            [&f.supplier_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let (supplier_code, supplier_name, active) =
        supplier.ok_or_else(|| AppError::new(ErrorCode::NotFound, "Supplier not found."))?;
    if !active {
        return Err(invalid("Select an active supplier."));
    }
    if f.lines.len() > 200 {
        return Err(invalid(
            "A purchase order supports at most 200 package lines.",
        ));
    }
    if require_lines && f.lines.is_empty() {
        return Err(invalid("Add at least one package before ordering."));
    }
    money(f.discount_minor as i128)?;
    money(f.tax_minor as i128)?;
    let mut seen = HashSet::new();
    let mut subtotal = 0i128;
    let mut lines = Vec::new();
    for line in &f.lines {
        id(&line.product_package_id)?;
        if !seen.insert(&line.product_package_id) {
            return Err(invalid("Merge duplicate package lines."));
        }
        if !(1..=MAX_SAFE_MINOR).contains(&line.ordered_quantity) {
            return Err(invalid(
                "Ordered quantities must be positive safe integers.",
            ));
        }
        money(line.unit_cost_minor as i128)?;
        let item:Option<(String,String,String)>=c.query_row("SELECT p.id,coalesce(p.commercial_name_en,p.commercial_name_ar,p.scientific_name,'Unnamed product'),k.package_label FROM product_packages k JOIN products p ON p.id=k.product_id WHERE k.id=?1 AND k.is_active=1 AND p.is_active=1",[&line.product_package_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
        let (product, name, label) =
            item.ok_or_else(|| invalid("Select an active Catalog package."))?;
        let total = money(line.ordered_quantity as i128 * line.unit_cost_minor as i128)?;
        subtotal += total as i128;
        lines.push((
            uuid::Uuid::new_v4().to_string(),
            product,
            line.product_package_id.clone(),
            name,
            label,
            line.ordered_quantity,
            line.unit_cost_minor,
            total,
        ));
    }
    let subtotal = money(subtotal)?;
    if f.discount_minor > subtotal {
        return Err(invalid("Discount cannot exceed the subtotal."));
    }
    let total = money(subtotal as i128 - f.discount_minor as i128 + f.tax_minor as i128)?;
    Ok(Prepared {
        supplier_id: f.supplier_id.clone(),
        supplier_code,
        supplier_name,
        reference: clean(&f.supplier_reference, 200, "Supplier reference")?,
        expected: valid_date(c, &f.expected_date)?,
        notes: clean(&f.notes, 2000, "Notes")?,
        subtotal,
        discount: f.discount_minor,
        tax: f.tax_minor,
        total,
        lines,
    })
}
fn payload<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|_| AppError::new(ErrorCode::Internal, "Could not encode purchase request."))
}
fn request(
    c: &Connection,
    request_id: &str,
    kind: &str,
    target: &str,
    payload: &str,
) -> Result<bool, AppError> {
    id(request_id)?;
    let old:Option<(String,String,String)>=c.query_row("SELECT kind,purchase_order_id,payload FROM purchase_order_requests WHERE request_id=?1",[request_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    if let Some(v) = old {
        if v != (kind.into(), target.into(), payload.into()) {
            return Err(invalid("Request ID was already used with another payload."));
        }
        return Ok(true);
    }
    Ok(false)
}
fn record(
    c: &Connection,
    request_id: &str,
    kind: &str,
    target: &str,
    payload: &str,
) -> Result<(), AppError> {
    c.execute("INSERT INTO purchase_order_requests(request_id,kind,purchase_order_id,payload) VALUES(?1,?2,?3,?4)",params![request_id,kind,target,payload])?;
    Ok(())
}
fn status(c: &Connection, value: &str) -> Result<(String, i64), AppError> {
    c.query_row(
        "SELECT status,revision FROM purchase_orders WHERE id=?1",
        [value],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Purchase order not found."))
}
fn editable(c: &Connection, value: &str, revision: i64) -> Result<(), AppError> {
    let (s, r) = status(c, value)?;
    if r != revision {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Purchase order changed. Reload before continuing.",
        ));
    }
    if s != "DRAFT" {
        return Err(invalid("Only draft purchase orders can be edited."));
    }
    Ok(())
}
fn replace_lines(c: &Connection, value: &str, p: &Prepared) -> Result<(), AppError> {
    c.execute(
        "DELETE FROM purchase_order_lines WHERE purchase_order_id=?1",
        [value],
    )?;
    for (pos, l) in p.lines.iter().enumerate() {
        c.execute("INSERT INTO purchase_order_lines(id,purchase_order_id,position,product_id,product_package_id,product_name_snapshot,package_label_snapshot,ordered_quantity,unit_cost_minor,line_total_minor) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![l.0,value,pos as i64,l.1,l.2,l.3,l.4,l.5,l.6,l.7])?;
    }
    Ok(())
}
pub fn create_draft(db: &AppDb, input: &CreatePurchaseDraft) -> Result<PurchaseOrder, AppError> {
    id(&input.id)?;
    let payload = payload(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if request(&tx, &input.request_id, "CREATE", &input.id, &payload)? {
        return detail_on(&tx, &input.id);
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM purchase_orders WHERE id=?1)",
        [&input.id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Purchase order ID already exists."));
    }
    let p = prepare(&tx, &input.fields, false)?;
    let day: String = tx.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    let n:i64=tx.query_row("INSERT INTO purchase_order_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1 RETURNING next_number",[&day],|r|r.get(0))?;
    let number = format!("PO-{day}-{n:04}");
    tx.execute("INSERT INTO purchase_orders(id,purchase_number,supplier_id,status,supplier_reference,expected_date,notes,subtotal_minor,discount_minor,tax_minor,total_minor,create_request_id) VALUES(?1,?2,?3,'DRAFT',?4,?5,?6,?7,?8,?9,?10,?11)",params![input.id,number,p.supplier_id,p.reference,p.expected,p.notes,p.subtotal,p.discount,p.tax,p.total,input.request_id])?;
    replace_lines(&tx, &input.id, &p)?;
    record(&tx, &input.request_id, "CREATE", &input.id, &payload)?;
    let out = detail_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(out)
}
pub fn update_draft(db: &AppDb, input: &UpdatePurchaseDraft) -> Result<PurchaseOrder, AppError> {
    id(&input.id)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    editable(&tx, &input.id, input.expected_revision)?;
    let p = prepare(&tx, &input.fields, false)?;
    tx.execute("UPDATE purchase_orders SET supplier_id=?2,supplier_reference=?3,expected_date=?4,notes=?5,subtotal_minor=?6,discount_minor=?7,tax_minor=?8,total_minor=?9,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![input.id,p.supplier_id,p.reference,p.expected,p.notes,p.subtotal,p.discount,p.tax,p.total])?;
    replace_lines(&tx, &input.id, &p)?;
    let out = detail_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(out)
}
pub fn order(db: &AppDb, input: &PurchaseAction) -> Result<PurchaseOrder, AppError> {
    id(&input.id)?;
    let payload = payload(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if request(&tx, &input.request_id, "ORDER", &input.id, &payload)? {
        return detail_on(&tx, &input.id);
    }
    editable(&tx, &input.id, input.expected_revision)?;
    let p = prepare(&tx, &fields_on(&tx, &input.id)?, true)?;
    tx.execute("UPDATE purchase_orders SET status='ORDERED',supplier_code_snapshot=?2,supplier_name_snapshot=?3,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),ordered_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![input.id,p.supplier_code,p.supplier_name])?;
    record(&tx, &input.request_id, "ORDER", &input.id, &payload)?;
    let out = detail_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(out)
}
pub fn cancel(db: &AppDb, input: &CancelPurchase) -> Result<PurchaseOrder, AppError> {
    id(&input.id)?;
    required(&input.reason)?;
    let reason = input.reason.trim();
    if reason.chars().count() > 2000 {
        return Err(invalid(
            "Cancellation reason must be at most 2000 characters.",
        ));
    }
    let payload = payload(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if request(&tx, &input.request_id, "CANCEL", &input.id, &payload)? {
        return detail_on(&tx, &input.id);
    }
    let (s, r) = status(&tx, &input.id)?;
    if r != input.expected_revision {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Purchase order changed. Reload before continuing.",
        ));
    }
    if !["DRAFT", "ORDERED"].contains(&s.as_str()) {
        return Err(invalid("This purchase order cannot be cancelled."));
    }
    tx.execute("UPDATE purchase_orders SET status='CANCELLED',revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),ordered_at=NULL,cancelled_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),cancellation_reason=?2 WHERE id=?1",params![input.id,reason])?;
    record(&tx, &input.request_id, "CANCEL", &input.id, &payload)?;
    let out = detail_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(out)
}
fn fields_on(c: &Connection, value: &str) -> Result<PurchaseDraftFields, AppError> {
    let mut f:PurchaseDraftFields=c.query_row("SELECT supplier_id,supplier_reference,expected_date,notes,discount_minor,tax_minor FROM purchase_orders WHERE id=?1",[value],|r|Ok(PurchaseDraftFields{supplier_id:r.get(0)?,supplier_reference:r.get(1)?,expected_date:r.get(2)?,notes:r.get(3)?,discount_minor:r.get(4)?,tax_minor:r.get(5)?,lines:vec![]}))?;
    let mut s=c.prepare("SELECT product_package_id,ordered_quantity,unit_cost_minor FROM purchase_order_lines WHERE purchase_order_id=?1 ORDER BY position")?;
    f.lines = s
        .query_map([value], |r| {
            Ok(PurchaseLineInput {
                product_package_id: r.get(0)?,
                ordered_quantity: r.get(1)?,
                unit_cost_minor: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(f)
}
fn detail_on(c: &Connection, value: &str) -> Result<PurchaseOrder, AppError> {
    id(value)?;
    let mut out=c.query_row("SELECT o.id,o.purchase_number,o.supplier_id,coalesce(o.supplier_code_snapshot,s.code),coalesce(o.supplier_name_snapshot,s.display_name),o.status,o.revision,o.supplier_reference,o.expected_date,o.notes,o.subtotal_minor,o.discount_minor,o.tax_minor,o.total_minor,o.created_at,o.updated_at,o.ordered_at,o.cancelled_at,o.cancellation_reason FROM purchase_orders o JOIN suppliers s ON s.id=o.supplier_id WHERE o.id=?1",[value],|r|Ok(PurchaseOrder{id:r.get(0)?,purchase_number:r.get(1)?,supplier_id:r.get(2)?,supplier_code:r.get(3)?,supplier_name:r.get(4)?,status:r.get(5)?,revision:r.get(6)?,supplier_reference:r.get(7)?,expected_date:r.get(8)?,notes:r.get(9)?,subtotal_minor:r.get(10)?,discount_minor:r.get(11)?,tax_minor:r.get(12)?,total_minor:r.get(13)?,created_at:r.get(14)?,updated_at:r.get(15)?,ordered_at:r.get(16)?,cancelled_at:r.get(17)?,cancellation_reason:r.get(18)?,lines:vec![]})).optional()?.ok_or_else(||AppError::new(ErrorCode::NotFound,"Purchase order not found."))?;
    let mut s=c.prepare("SELECT id,product_id,product_package_id,product_name_snapshot,package_label_snapshot,ordered_quantity,unit_cost_minor,line_total_minor,position FROM purchase_order_lines WHERE purchase_order_id=?1 ORDER BY position,id")?;
    out.lines = s
        .query_map([value], |r| {
            Ok(PurchaseLine {
                id: r.get(0)?,
                product_id: r.get(1)?,
                product_package_id: r.get(2)?,
                product_name_snapshot: r.get(3)?,
                package_label_snapshot: r.get(4)?,
                ordered_quantity: r.get(5)?,
                unit_cost_minor: r.get(6)?,
                line_total_minor: r.get(7)?,
                position: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(out)
}
pub fn detail(db: &AppDb, value: &str) -> Result<PurchaseOrder, AppError> {
    let c = db.lock()?;
    detail_on(&c, value)
}
pub fn list(db: &AppDb, q: &PurchaseQuery) -> Result<PurchasePage, AppError> {
    let (l, o) = page(q.limit, q.offset)?;
    if let Some(v) = &q.supplier_id {
        id(v)?
    }
    if q.status.as_ref().is_some_and(|v| {
        ![
            "DRAFT",
            "ORDERED",
            "PARTIALLY_RECEIVED",
            "RECEIVED",
            "CANCELLED",
        ]
        .contains(&v.as_str())
    }) {
        return Err(invalid("Unknown purchase status."));
    }
    let search = q.search.as_deref().unwrap_or("").trim();
    if search.chars().count() > 200 {
        return Err(invalid("Search text must be at most 200 characters."));
    }
    let c = db.lock()?;
    for d in [&q.date_from, &q.date_to].into_iter().flatten() {
        let ok: bool = c.query_row(
            "SELECT length(?1)=10 AND date(?1,'+0 days') IS ?1",
            [d],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(invalid("Use real YYYY-MM-DD dates."));
        }
    }
    if matches!((&q.date_from,&q.date_to),(Some(a),Some(b)) if a>b) {
        return Err(invalid("Invalid date range."));
    }
    let filter="(?1 IS NULL OR o.supplier_id=?1) AND (?2 IS NULL OR o.status=?2) AND (?3='' OR instr(lower(o.purchase_number||' '||coalesce(o.supplier_code_snapshot,s.code)||' '||coalesce(o.supplier_name_snapshot,s.display_name)||' '||coalesce(o.supplier_reference,'')),lower(?3))>0) AND (?4 IS NULL OR substr(o.created_at,1,10)>=?4) AND (?5 IS NULL OR substr(o.created_at,1,10)<=?5)";
    let total=c.query_row(&format!("SELECT count(*) FROM purchase_orders o JOIN suppliers s ON s.id=o.supplier_id WHERE {filter}"),params![q.supplier_id,q.status,search,q.date_from,q.date_to],|r|r.get(0))?;
    let mut s=c.prepare(&format!("SELECT o.id,o.purchase_number,o.supplier_id,coalesce(o.supplier_code_snapshot,s.code),coalesce(o.supplier_name_snapshot,s.display_name),o.status,o.revision,o.total_minor,(SELECT count(*) FROM purchase_order_lines l WHERE l.purchase_order_id=o.id),o.created_at,o.ordered_at FROM purchase_orders o JOIN suppliers s ON s.id=o.supplier_id WHERE {filter} ORDER BY o.created_at DESC,o.id DESC LIMIT ?6 OFFSET ?7"))?;
    let items = s
        .query_map(
            params![
                q.supplier_id,
                q.status,
                search,
                q.date_from,
                q.date_to,
                l,
                o
            ],
            |r| {
                Ok(PurchaseRow {
                    id: r.get(0)?,
                    purchase_number: r.get(1)?,
                    supplier_id: r.get(2)?,
                    supplier_code: r.get(3)?,
                    supplier_name: r.get(4)?,
                    status: r.get(5)?,
                    revision: r.get(6)?,
                    total_minor: r.get(7)?,
                    line_count: r.get(8)?,
                    created_at: r.get(9)?,
                    ordered_at: r.get(10)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PurchasePage {
        items,
        total,
        limit: l,
        offset: o,
    })
}
pub fn search_packages(db: &AppDb, q: &PackageQuery) -> Result<PackagePage, AppError> {
    let (l, o) = page(q.limit, q.offset)?;
    let search = q.search.as_deref().unwrap_or("").trim();
    if search.chars().count() > 200 {
        return Err(invalid("Search text must be at most 200 characters."));
    }
    let c = db.lock()?;
    let filter="k.is_active=1 AND p.is_active=1 AND (?1='' OR instr(lower(coalesce(p.commercial_name_en,'')||' '||coalesce(p.commercial_name_ar,'')||' '||coalesce(p.scientific_name,'')||' '||coalesce(k.package_label,'')||' '||coalesce(m.name,'')),lower(?1))>0 OR EXISTS(SELECT 1 FROM barcodes b WHERE b.product_package_id=k.id AND b.barcode=?1))";
    let total=c.query_row(&format!("SELECT count(*) FROM product_packages k JOIN products p ON p.id=k.product_id LEFT JOIN manufacturers m ON m.id=p.manufacturer_id WHERE {filter}"),[search],|r|r.get(0))?;
    let mut s=c.prepare(&format!("SELECT p.id,k.id,coalesce(p.commercial_name_en,p.commercial_name_ar,p.scientific_name,'Unnamed product'),k.package_label,m.name,(SELECT barcode FROM barcodes b WHERE b.product_package_id=k.id ORDER BY is_primary DESC,id LIMIT 1),(SELECT cost_price_minor FROM product_price_history h WHERE h.product_package_id=k.id AND h.effective_to IS NULL) FROM product_packages k JOIN products p ON p.id=k.product_id LEFT JOIN manufacturers m ON m.id=p.manufacturer_id WHERE {filter} ORDER BY p.commercial_name_en COLLATE NOCASE,k.package_label,k.id LIMIT ?2 OFFSET ?3"))?;
    let items = s
        .query_map(params![search, l, o], |r| {
            Ok(PurchasePackage {
                product_id: r.get(0)?,
                product_package_id: r.get(1)?,
                product_name: r.get(2)?,
                package_label: r.get(3)?,
                manufacturer_name: r.get(4)?,
                barcode: r.get(5)?,
                current_cost_minor: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PackagePage {
        items,
        total,
        limit: l,
        offset: o,
    })
}
