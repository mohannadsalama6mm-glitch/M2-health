use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::purchases::*,
    },
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const PURCHASE_COLUMNS: &str = "id, branch_id, supplier_id, sequence, purchase_number, invoice_number, status, subtotal_minor, discount_minor, tax_minor, total_minor, paid_minor, change_minor, payment_method, user, note, void_reason, voided_at, voided_by, completed_at, created_at, updated_at";
fn map_purchase(r: &Row<'_>) -> rusqlite::Result<Purchase> {
    Ok(Purchase {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        supplier_id: r.get(2)?,
        sequence: r.get(3)?,
        purchase_number: r.get(4)?,
        invoice_number: r.get(5)?,
        status: r.get(6)?,
        subtotal_minor: r.get(7)?,
        discount_minor: r.get(8)?,
        tax_minor: r.get(9)?,
        total_minor: r.get(10)?,
        paid_minor: r.get(11)?,
        change_minor: r.get(12)?,
        payment_method: r.get(13)?,
        user: r.get(14)?,
        note: r.get(15)?,
        void_reason: r.get(16)?,
        voided_at: r.get(17)?,
        voided_by: r.get(18)?,
        completed_at: r.get(19)?,
        created_at: r.get(20)?,
        updated_at: r.get(21)?,
    })
}
const ITEM_COLUMNS: &str = "id, purchase_id, product_package_id, product_name, package_label, quantity, unit_cost_minor, line_total_minor, position";
fn map_item(r: &Row<'_>) -> rusqlite::Result<PurchaseItem> {
    Ok(PurchaseItem {
        id: r.get(0)?,
        purchase_id: r.get(1)?,
        product_package_id: r.get(2)?,
        product_name: r.get(3)?,
        package_label: r.get(4)?,
        quantity: r.get(5)?,
        unit_cost_minor: r.get(6)?,
        line_total_minor: r.get(7)?,
        position: r.get(8)?,
    })
}
fn validate_page(limit_input: Option<i64>, offset_input: Option<i64>) -> Result<(i64, i64), AppError> {
    let limit = limit_input.unwrap_or(50);
    let offset = offset_input.unwrap_or(0);
    if !(1..=200).contains(&limit) || !(0..=MAX_SAFE_MINOR).contains(&offset) {
        return Err(invalid("Use a limit of 1–200 and a non-negative offset."));
    }
    Ok((limit, offset))
}
fn search_pattern(value: &str) -> (String, String) {
    let search = normalize(value);
    let pattern = format!(
        "{}%",
        search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    (search, pattern)
}
pub(crate) struct NewPurchase {
    pub branch_id: String,
    pub supplier_id: Option<String>,
    pub invoice_number: String,
    pub subtotal_minor: i64,
    pub discount_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub paid_minor: i64,
    pub change_minor: i64,
    pub payment_method: String,
    pub user: String,
    pub note: String,
}
/// Creates the purchase header inside a caller-owned transaction. Per-branch sequence,
/// formatted purchase number and monotonic completion timestamp are computed here.
pub(crate) fn insert_purchase_on(c: &Connection, new: &NewPurchase) -> Result<Purchase, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    let sequence: i64 =
        c.query_row("SELECT COALESCE(MAX(sequence),0)+1 FROM purchases WHERE branch_id=?1", [&new.branch_id], |r| r.get(0))?;
    let purchase_number = format!("{:05}", sequence);
    let now: String = c.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce(julianday((SELECT max(completed_at) FROM purchases))+0.001/86400,0)))",
        [],
        |r| r.get(0),
    )?;
    c.execute(
        "INSERT INTO purchases(id,branch_id,supplier_id,sequence,purchase_number,invoice_number,subtotal_minor,discount_minor,tax_minor,total_minor,paid_minor,change_minor,payment_method,user,note,completed_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        params![value, new.branch_id, new.supplier_id, sequence, purchase_number, new.invoice_number, new.subtotal_minor, new.discount_minor, new.tax_minor, new.total_minor, new.paid_minor, new.change_minor, new.payment_method, new.user, new.note, now],
    )?;
    get(c, &value)
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Purchase, AppError> {
    c.query_row(&format!("SELECT {PURCHASE_COLUMNS} FROM purchases WHERE id=?1"), [value], map_purchase)
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Purchase not found."))
}
pub(crate) fn update_void_on(c: &Connection, value: &str, reason: &str, user: &str) -> Result<Purchase, AppError> {
    c.execute(
        "UPDATE purchases SET status='void',void_reason=?1,voided_by=?2,voided_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?3",
        params![reason, user, value],
    )?;
    get(c, value)
}
pub(crate) fn insert_payment_on(
    c: &Connection,
    purchase_id: &str,
    method: &str,
    amount_minor: i64,
) -> Result<PurchasePayment, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO purchase_payments(id,purchase_id,method,amount_minor) VALUES (?1,?2,?3,?4)",
        params![value, purchase_id, method, amount_minor],
    )?;
    Ok(c.query_row(
        "SELECT id, purchase_id, method, amount_minor, created_at FROM purchase_payments WHERE id=?1",
        [value],
        |r| {
            Ok(PurchasePayment {
                id: r.get(0)?,
                purchase_id: r.get(1)?,
                method: r.get(2)?,
                amount_minor: r.get(3)?,
                created_at: r.get(4)?,
            })
        },
    )?)
}
pub(crate) fn insert_item_on(
    c: &Connection,
    purchase_id: &str,
    package_id: &str,
    product_name: &str,
    package_label: &str,
    quantity: i64,
    unit_cost_minor: i64,
    line_total_minor: i64,
    position: i64,
) -> Result<PurchaseItem, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO purchase_items(id,purchase_id,product_package_id,product_name,package_label,quantity,unit_cost_minor,line_total_minor,position) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![value, purchase_id, package_id, product_name, package_label, quantity, unit_cost_minor, line_total_minor, position],
    )?;
    Ok(c.query_row(&format!("SELECT {ITEM_COLUMNS} FROM purchase_items WHERE id=?1"), [value], map_item)?)
}
pub(crate) fn insert_item_batch_on(
    c: &Connection,
    purchase_id: &str,
    purchase_item_id: &str,
    batch_id: &str,
    quantity: i64,
    unit_cost_minor: Option<i64>,
) -> Result<PurchaseBatchAllocation, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO purchase_item_batches(id,purchase_id,purchase_item_id,batch_id,quantity,unit_cost_minor) VALUES (?1,?2,?3,?4,?5,?6)",
        params![value, purchase_id, purchase_item_id, batch_id, quantity, unit_cost_minor],
    )?;
    Ok(c.query_row(
        "SELECT id, purchase_id, purchase_item_id, batch_id, quantity, unit_cost_minor FROM purchase_item_batches WHERE id=?1",
        [value],
        |r| {
            Ok(PurchaseBatchAllocation {
                id: r.get(0)?,
                purchase_id: r.get(1)?,
                purchase_item_id: r.get(2)?,
                batch_id: r.get(3)?,
                quantity: r.get(4)?,
                unit_cost_minor: r.get(5)?,
            })
        },
    )?)
}
/// The lots booked by one purchase together with their packages, used to reverse
/// stock when the purchase is voided.
pub(crate) fn allocations_on(
    c: &Connection,
    purchase_id: &str,
) -> Result<Vec<(String, String, i64, Option<i64>)>, AppError> {
    let mut s = c.prepare(
        "SELECT b.product_package_id, pib.batch_id, pib.quantity, pib.unit_cost_minor FROM purchase_item_batches pib JOIN inventory_batches b ON b.id=pib.batch_id WHERE pib.purchase_id=?1 ORDER BY pib.id",
    )?;
    let rows = s.query_map([purchase_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, Option<i64>>(3)?,
        ))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
fn list_allocations_on(
    c: &Connection,
    purchase_id: &str,
    purchase_item_id: &str,
) -> Result<Vec<PurchaseBatchAllocationView>, AppError> {
    let mut s = c.prepare(
        "SELECT id, batch_id, quantity, unit_cost_minor FROM purchase_item_batches WHERE purchase_item_id=?1 AND purchase_id=?2 ORDER BY id",
    )?;
    let rows = s.query_map(params![purchase_item_id, purchase_id], |r| {
        Ok(PurchaseBatchAllocationView {
            id: r.get(0)?,
            batch_id: r.get(1)?,
            quantity: r.get(2)?,
            unit_cost_minor: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
pub fn detail(db: &AppDb, value: &str) -> Result<PurchaseDetail, AppError> {
    id(value)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let purchase = get(&tx, value)?;
    let branch_name: String = tx.query_row(
        "SELECT name FROM branches WHERE id=?1",
        [&purchase.branch_id],
        |r| r.get(0),
    )?;
    let supplier_name: Option<String> = match &purchase.supplier_id {
        Some(supplier_id) => Some(tx.query_row(
            "SELECT name FROM suppliers WHERE id=?1",
            [supplier_id],
            |r| r.get(0),
        )?),
        None => None,
    };
    let items = {
        let mut s = tx.prepare(&format!("SELECT {ITEM_COLUMNS} FROM purchase_items WHERE purchase_id=?1 ORDER BY position, id"))?;
        let rows = s.query_map([&purchase.id], map_item)?;
        let mut out = Vec::new();
        for row in rows {
            let item = row?;
            let batches = list_allocations_on(&tx, &purchase.id, &item.id)?;
            out.push(PurchaseItemView {
                id: item.id,
                product_package_id: item.product_package_id,
                product_name: item.product_name,
                package_label: item.package_label,
                quantity: item.quantity,
                unit_cost_minor: item.unit_cost_minor,
                line_total_minor: item.line_total_minor,
                position: item.position,
                batches,
            });
        }
        out
    };
    tx.commit()?;
    Ok(PurchaseDetail {
        purchase,
        branch_name,
        supplier_name,
        items,
    })
}
const PURCHASE_ROW_SELECT: &str = "SELECT p.id, p.branch_id, br.name, s.name AS supplier_name, p.sequence, p.purchase_number, p.invoice_number, p.status, p.total_minor, p.paid_minor, p.payment_method, (SELECT COALESCE(SUM(quantity),0) FROM purchase_items pi WHERE pi.purchase_id=p.id) AS item_count, p.user, p.completed_at, p.void_reason FROM purchases p JOIN branches br ON br.id=p.branch_id LEFT JOIN suppliers s ON s.id=p.supplier_id";
pub fn list(db: &AppDb, q: &PurchaseQuery) -> Result<PurchasePage, AppError> {
    if let Some(branch_id) = &q.branch_id {
        id(branch_id)?;
    }
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    if let Some(status) = &q.status {
        if !["completed", "void", ""].contains(&status.as_str()) {
            return Err(invalid("Unknown purchase status filter."));
        }
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let filter = r"WHERE (?1 IS NULL OR branch_id=?1) AND (?2 IS NULL OR status=?2) AND (?3='' OR purchase_number LIKE ?4 ESCAPE '\' OR invoice_number LIKE ?4 ESCAPE '\' OR supplier_name LIKE ?4 ESCAPE '\' OR user LIKE ?4 ESCAPE '\')";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({PURCHASE_ROW_SELECT}) {filter}"),
        params![q.branch_id, q.status, search, pattern],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "{PURCHASE_ROW_SELECT} {filter} ORDER BY p.completed_at DESC, p.sequence DESC, p.id LIMIT ?5 OFFSET ?6"
        ))?;
        let rows = s.query_map(
            params![q.branch_id, q.status, search, pattern, limit, offset],
            |r| {
                Ok(PurchaseRow {
                    id: r.get(0)?,
                    branch_id: r.get(1)?,
                    branch_name: r.get(2)?,
                    supplier_name: r.get(3)?,
                    sequence: r.get(4)?,
                    purchase_number: r.get(5)?,
                    invoice_number: r.get(6)?,
                    status: r.get(7)?,
                    total_minor: r.get(8)?,
                    paid_minor: r.get(9)?,
                    payment_method: r.get(10)?,
                    item_count: r.get(11)?,
                    user: r.get(12)?,
                    completed_at: r.get(13)?,
                    void_reason: r.get(14)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(PurchasePage {
        items,
        total,
        limit,
        offset,
    })
}
/// Purchase product lookup: active packages matched by name, package label,
/// barcode or lot number, each with its current cost price for reordering.
pub fn pos_search(db: &AppDb, q: &PosPurchaseQuery) -> Result<PurchaseProductPage, AppError> {
    id(&q.branch_id)?;
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let exact = q.search.as_deref().unwrap_or("").trim();
    let inner = r"SELECT pp.id AS package_id, pp.product_id, COALESCE(NULLIF(pr.commercial_name_en,''), pr.scientific_name, pr.commercial_name_ar, 'Unnamed') AS product_name, pp.package_label, pp.pack_size, (SELECT ph.cost_price_minor FROM product_price_history ph WHERE ph.product_package_id=pp.id AND ph.effective_to IS NULL LIMIT 1) AS cost_price_minor FROM product_packages pp JOIN products pr ON pr.id=pp.product_id WHERE pp.is_active=1 AND pr.is_active=1";
    let filter = r"WHERE (?2='' OR product_name LIKE ?3 ESCAPE '\' OR package_label LIKE ?3 ESCAPE '\' OR EXISTS(SELECT 1 FROM barcodes bc WHERE bc.product_package_id=package_id AND bc.barcode=?4) OR EXISTS(SELECT 1 FROM inventory_batches ib WHERE ib.branch_id=?1 AND ib.product_package_id=package_id AND ib.batch_number=?4))";
    let full = format!("SELECT * FROM ({inner}) t {filter}");
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({full})"),
        params![q.branch_id, search, pattern, exact],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "{full} ORDER BY product_name COLLATE NOCASE, package_id LIMIT ?5 OFFSET ?6"
        ))?;
        let rows = s.query_map(
            params![q.branch_id, search, pattern, exact, limit, offset],
            |r| {
                Ok(PurchaseProduct {
                    package_id: r.get(0)?,
                    product_id: r.get(1)?,
                    product_name: r.get(2)?,
                    package_label: r.get(3)?,
                    pack_size: r.get(4)?,
                    cost_price_minor: r.get(5)?,
                    status: "available".into(),
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(PurchaseProductPage {
        items,
        total,
        limit,
        offset,
    })
}