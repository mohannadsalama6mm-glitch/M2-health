use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::sales::*,
    },
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const SALE_COLUMNS: &str = "id, branch_id, sequence, receipt_number, customer_id, customer_name, customer_phone, status, subtotal_minor, discount_minor, tax_minor, total_minor, paid_minor, change_minor, payment_method, user, note, void_reason, voided_at, voided_by, completed_at, created_at, updated_at";
fn map_sale(r: &Row<'_>) -> rusqlite::Result<Sale> {
    Ok(Sale {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        sequence: r.get(2)?,
        receipt_number: r.get(3)?,
        customer_id: r.get(4)?,
        customer_name: r.get(5)?,
        customer_phone: r.get(6)?,
        status: r.get(7)?,
        subtotal_minor: r.get(8)?,
        discount_minor: r.get(9)?,
        tax_minor: r.get(10)?,
        total_minor: r.get(11)?,
        paid_minor: r.get(12)?,
        change_minor: r.get(13)?,
        payment_method: r.get(14)?,
        user: r.get(15)?,
        note: r.get(16)?,
        void_reason: r.get(17)?,
        voided_at: r.get(18)?,
        voided_by: r.get(19)?,
        completed_at: r.get(20)?,
        created_at: r.get(21)?,
        updated_at: r.get(22)?,
    })
}
const ITEM_COLUMNS: &str = "id, sale_id, product_package_id, product_name, package_label, quantity, selling_price_minor, cost_price_minor, line_total_minor, line_cost_minor, position";
fn map_item(r: &Row<'_>) -> rusqlite::Result<SaleItem> {
    Ok(SaleItem {
        id: r.get(0)?,
        sale_id: r.get(1)?,
        product_package_id: r.get(2)?,
        product_name: r.get(3)?,
        package_label: r.get(4)?,
        quantity: r.get(5)?,
        selling_price_minor: r.get(6)?,
        cost_price_minor: r.get(7)?,
        line_total_minor: r.get(8)?,
        line_cost_minor: r.get(9)?,
        position: r.get(10)?,
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
pub(crate) struct NewSale {
    pub branch_id: String,
    pub customer_id: Option<String>,
    pub customer_name: String,
    pub customer_phone: String,
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
/// Creates the sale header inside a caller-owned transaction. Per-branch sequence,
/// formatted receipt number and monotonic completion timestamp are computed here.
pub(crate) fn insert_sale_on(c: &Connection, new: &NewSale) -> Result<Sale, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    let sequence: i64 =
        c.query_row("SELECT COALESCE(MAX(sequence),0)+1 FROM sales WHERE branch_id=?1", [&new.branch_id], |r| r.get(0))?;
    let receipt_number = format!("{:05}", sequence);
    let now: String = c.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce(julianday((SELECT max(completed_at) FROM sales))+0.001/86400,0)))",
        [],
        |r| r.get(0),
    )?;
    c.execute(
        "INSERT INTO sales(id,branch_id,sequence,receipt_number,customer_id,customer_name,customer_phone,subtotal_minor,discount_minor,tax_minor,total_minor,paid_minor,change_minor,payment_method,user,note,completed_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
        params![value, new.branch_id, sequence, receipt_number, new.customer_id, new.customer_name, new.customer_phone, new.subtotal_minor, new.discount_minor, new.tax_minor, new.total_minor, new.paid_minor, new.change_minor, new.payment_method, new.user, new.note, now],
    )?;
    get(c, &value)
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<Sale, AppError> {
    c.query_row(&format!("SELECT {SALE_COLUMNS} FROM sales WHERE id=?1"), [value], map_sale)
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Sale not found."))
}
pub(crate) fn item_on(c: &Connection, value: &str) -> Result<SaleItem, AppError> {
    c.query_row(&format!("SELECT {ITEM_COLUMNS} FROM sale_items WHERE id=?1"), [value], map_item)
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Sale item not found."))
}
pub(crate) fn update_void_on(c: &Connection, value: &str, reason: &str, user: &str) -> Result<Sale, AppError> {
    c.execute(
        "UPDATE sales SET status='void',void_reason=?1,voided_by=?2,voided_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?3",
        params![reason, user, value],
    )?;
    get(c, value)
}
pub(crate) fn insert_payment_on(
    c: &Connection,
    sale_id: &str,
    method: &str,
    amount_minor: i64,
) -> Result<SalePayment, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO sale_payments(id,sale_id,method,amount_minor) VALUES (?1,?2,?3,?4)",
        params![value, sale_id, method, amount_minor],
    )?;
    Ok(c.query_row(
        "SELECT id, sale_id, method, amount_minor, created_at FROM sale_payments WHERE id=?1",
        [value],
        |r| {
            Ok(SalePayment {
                id: r.get(0)?,
                sale_id: r.get(1)?,
                method: r.get(2)?,
                amount_minor: r.get(3)?,
                created_at: r.get(4)?,
            })
        },
    )?)
}
pub(crate) fn insert_item_on(
    c: &Connection,
    sale_id: &str,
    package_id: &str,
    product_name: &str,
    package_label: &str,
    quantity: i64,
    selling_price_minor: i64,
    cost_price_minor: Option<i64>,
    line_total_minor: i64,
    line_cost_minor: Option<i64>,
    position: i64,
) -> Result<SaleItem, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO sale_items(id,sale_id,product_package_id,product_name,package_label,quantity,selling_price_minor,cost_price_minor,line_total_minor,line_cost_minor,position) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![value, sale_id, package_id, product_name, package_label, quantity, selling_price_minor, cost_price_minor, line_total_minor, line_cost_minor, position],
    )?;
    Ok(c.query_row(&format!("SELECT {ITEM_COLUMNS} FROM sale_items WHERE id=?1"), [value], map_item)?)
}
pub(crate) fn insert_item_batch_on(
    c: &Connection,
    sale_id: &str,
    sale_item_id: &str,
    batch_id: &str,
    quantity: i64,
    cost_price_minor: Option<i64>,
) -> Result<(), AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO sale_item_batches(id,sale_id,sale_item_id,batch_id,quantity,cost_price_minor) VALUES (?1,?2,?3,?4,?5,?6)",
        params![value, sale_id, sale_item_id, batch_id, quantity, cost_price_minor],
    )?;
    Ok(())
}
/// The FEFO allocation recorded for one sale, keyed by sale item id.
pub(crate) fn allocations_on(c: &Connection, sale_id: &str) -> Result<Vec<(String, SaleBatchAllocation)>, AppError> {
    let mut s = c.prepare(
        "SELECT sib.sale_item_id, sib.batch_id, b.batch_number, b.expiry_date, sib.quantity, sib.cost_price_minor FROM sale_item_batches sib JOIN inventory_batches b ON b.id=sib.batch_id WHERE sib.sale_id=?1 ORDER BY sib.sale_item_id, sib.id",
    )?;
    let rows = s.query_map([sale_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            SaleBatchAllocation {
                batch_id: r.get(1)?,
                batch_number: r.get(2)?,
                expiry_date: r.get(3)?,
                quantity: r.get(4)?,
                cost_price_minor: r.get(5)?,
            },
        ))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
/// Already-returned quantity per sale item of a sale.
pub(crate) fn returned_by_item_on(c: &Connection, sale_id: &str) -> Result<std::collections::HashMap<String, i64>, AppError> {
    let mut s = c.prepare(
        "SELECT sri.sale_item_id, COALESCE(SUM(sri.quantity),0) FROM sale_return_items sri JOIN sale_returns sr ON sr.id=sri.sale_return_id WHERE sr.sale_id=?1 GROUP BY sri.sale_item_id",
    )?;
    let rows = s.query_map([sale_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?.into_iter().collect())
}
pub(crate) fn first_allocated_batch_on(c: &Connection, sale_item_id: &str) -> Result<Option<String>, AppError> {
    Ok(c.query_row(
        "SELECT batch_id FROM sale_item_batches WHERE sale_item_id=?1 ORDER BY id LIMIT 1",
        [sale_item_id],
        |r| r.get(0),
    )
    .optional()?)
}
pub(crate) fn insert_return_on(
    c: &Connection,
    sale_id: &str,
    branch_id: &str,
    reason: &str,
    user: &str,
    total_refund_minor: i64,
) -> Result<SaleReturn, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    let now: String = c.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),coalesce(julianday((SELECT max(returned_at) FROM sale_returns))+0.001/86400,0)))",
        [],
        |r| r.get(0),
    )?;
    c.execute(
        "INSERT INTO sale_returns(id,sale_id,branch_id,reason,user,total_refund_minor,returned_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![value, sale_id, branch_id, reason, user, total_refund_minor, now],
    )?;
    Ok(SaleReturn {
        id: value,
        sale_id: sale_id.into(),
        branch_id: branch_id.into(),
        reason: reason.into(),
        user: user.into(),
        total_refund_minor,
        returned_at: now,
        items: Vec::new(),
    })
}
pub(crate) fn insert_return_item_on(
    c: &Connection,
    sale_return_id: &str,
    sale_item_id: &str,
    package_id: &str,
    quantity: i64,
    refund_minor: i64,
) -> Result<SaleReturnItemView, AppError> {
    let value = uuid::Uuid::new_v4().to_string();
    let (product_name, package_label): (String, String) = c.query_row(
        "SELECT product_name, package_label FROM sale_items WHERE id=?1",
        [sale_item_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    c.execute(
        "INSERT INTO sale_return_items(id,sale_return_id,sale_item_id,product_package_id,quantity,refund_minor,line_refund_minor) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![value, sale_return_id, sale_item_id, package_id, quantity, refund_minor, quantity * refund_minor],
    )?;
    Ok(SaleReturnItemView {
        id: value,
        sale_item_id: sale_item_id.into(),
        product_name,
        package_label,
        quantity,
        refund_minor,
        line_refund_minor: quantity * refund_minor,
    })
}
fn list_returns_on(c: &Connection, sale_id: &str) -> Result<Vec<SaleReturn>, AppError> {
    let mut s = c.prepare(
        "SELECT id, sale_id, branch_id, reason, user, total_refund_minor, returned_at FROM sale_returns WHERE sale_id=?1 ORDER BY returned_at, id",
    )?;
    let rows = s.query_map([sale_id], |r| {
        Ok(SaleReturn {
            id: r.get(0)?,
            sale_id: r.get(1)?,
            branch_id: r.get(2)?,
            reason: r.get(3)?,
            user: r.get(4)?,
            total_refund_minor: r.get(5)?,
            returned_at: r.get(6)?,
            items: Vec::new(),
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        let mut returned = row?;
        let items = {
            let mut is = c.prepare(
                "SELECT sri.id, sri.sale_item_id, si.product_name, si.package_label, sri.quantity, sri.refund_minor, sri.line_refund_minor FROM sale_return_items sri JOIN sale_items si ON si.id=sri.sale_item_id WHERE sri.sale_return_id=?1 ORDER BY sri.id",
            )?;
            let rows = is.query_map([&returned.id], |r| {
                Ok(SaleReturnItemView {
                    id: r.get(0)?,
                    sale_item_id: r.get(1)?,
                    product_name: r.get(2)?,
                    package_label: r.get(3)?,
                    quantity: r.get(4)?,
                    refund_minor: r.get(5)?,
                    line_refund_minor: r.get(6)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        returned.items = items;
        out.push(returned);
    }
    Ok(out)
}
pub fn detail(db: &AppDb, sale_id: &str) -> Result<SaleDetail, AppError> {
    id(sale_id)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let sale = get(&tx, sale_id)?;
    let branch_name: String = tx.query_row(
        "SELECT name FROM branches WHERE id=?1",
        [&sale.branch_id],
        |r| r.get(0),
    )?;
    let allocations = allocations_on(&tx, &sale.id)?;
    let returned = returned_by_item_on(&tx, &sale.id)?;
    let items = {
        let mut s = tx.prepare(&format!("SELECT {ITEM_COLUMNS} FROM sale_items WHERE sale_id=?1 ORDER BY position, id"))?;
        let rows = s.query_map([&sale.id], map_item)?;
        let mut out = Vec::new();
        for row in rows {
            let item = row?;
            let batches = allocations
                .iter()
                .filter(|(item_id, _)| item_id == &item.id)
                .map(|(_, a)| a.clone())
                .collect();
            let returned_quantity = returned.get(&item.id).copied().unwrap_or(0);
            out.push(SaleItemView {
                id: item.id,
                product_package_id: item.product_package_id,
                product_name: item.product_name,
                package_label: item.package_label,
                quantity: item.quantity,
                selling_price_minor: item.selling_price_minor,
                cost_price_minor: item.cost_price_minor,
                line_total_minor: item.line_total_minor,
                line_cost_minor: item.line_cost_minor,
                position: item.position,
                batches,
                returned_quantity,
                returnable_quantity: item.quantity - returned_quantity,
            });
        }
        out
    };
    let returns = list_returns_on(&tx, &sale.id)?;
    tx.commit()?;
    Ok(SaleDetail {
        sale,
        branch_name,
        items,
        returns,
    })
}
const SALE_ROW_SELECT: &str = "SELECT s.id, s.branch_id, br.name, s.sequence, s.receipt_number, s.customer_name, s.status, s.total_minor, s.paid_minor, s.payment_method, (SELECT COALESCE(SUM(quantity),0) FROM sale_items si WHERE si.sale_id=s.id) AS item_count, s.user, s.completed_at, s.void_reason FROM sales s JOIN branches br ON br.id=s.branch_id";
pub fn list(db: &AppDb, q: &SaleQuery) -> Result<SalePage, AppError> {
    if let Some(branch_id) = &q.branch_id {
        id(branch_id)?;
    }
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    if let Some(status) = &q.status {
        if !["completed", "void", ""].contains(&status.as_str()) {
            return Err(invalid("Unknown sale status filter."));
        }
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let filter = r"WHERE (?1 IS NULL OR branch_id=?1) AND (?2 IS NULL OR status=?2) AND (?3='' OR receipt_number LIKE ?4 ESCAPE '\' OR customer_name LIKE ?4 ESCAPE '\' OR name LIKE ?4 ESCAPE '\' OR user LIKE ?4 ESCAPE '\')";
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let total = tx.query_row(
        &format!("SELECT count(*) FROM ({SALE_ROW_SELECT}) {filter}"),
        params![q.branch_id, q.status, search, pattern],
        |r| r.get(0),
    )?;
    let items = {
        let mut s = tx.prepare(&format!(
            "{SALE_ROW_SELECT} {filter} ORDER BY s.completed_at DESC, s.sequence DESC, s.id LIMIT ?5 OFFSET ?6"
        ))?;
        let rows = s.query_map(
            params![q.branch_id, q.status, search, pattern, limit, offset],
            |r| {
                Ok(SaleRow {
                    id: r.get(0)?,
                    branch_id: r.get(1)?,
                    branch_name: r.get(2)?,
                    sequence: r.get(3)?,
                    receipt_number: r.get(4)?,
                    customer_name: r.get(5)?,
                    status: r.get(6)?,
                    total_minor: r.get(7)?,
                    paid_minor: r.get(8)?,
                    payment_method: r.get(9)?,
                    item_count: r.get(10)?,
                    user: r.get(11)?,
                    completed_at: r.get(12)?,
                    void_reason: r.get(13)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(SalePage {
        items,
        total,
        limit,
        offset,
    })
}
/// POS product lookup: active sellable packages matched by name, package label,
/// barcode or lot number, each with its live branch balance and current selling price.
pub fn pos_search(db: &AppDb, q: &PosQuery) -> Result<PosProductPage, AppError> {
    id(&q.branch_id)?;
    if q.search.as_ref().is_some_and(|v| v.chars().count() > 4000) {
        return Err(invalid("Search text is too long."));
    }
    let (limit, offset) = validate_page(q.limit, q.offset)?;
    let (search, pattern) = search_pattern(q.search.as_deref().unwrap_or(""));
    let exact = q.search.as_deref().unwrap_or("").trim();
    let inner = r"SELECT pp.id AS package_id, pp.product_id, COALESCE(NULLIF(pr.commercial_name_en,''), pr.scientific_name, pr.commercial_name_ar, 'Unnamed') AS product_name, pp.package_label, pp.pack_size, (SELECT ph.selling_price_minor FROM product_price_history ph WHERE ph.product_package_id=pp.id AND ph.effective_to IS NULL LIMIT 1) AS selling_price_minor, (SELECT COALESCE(SUM(bal.quantity),0) FROM stock_balances bal JOIN inventory_batches b ON b.id=bal.batch_id WHERE b.branch_id=?1 AND b.product_package_id=pp.id AND b.is_active=1) AS quantity FROM product_packages pp JOIN products pr ON pr.id=pp.product_id WHERE pp.is_active=1 AND pr.is_active=1";
    let filter = r"WHERE (?2='' OR product_name LIKE ?3 ESCAPE '\' OR package_label LIKE ?3 ESCAPE '\' OR EXISTS(SELECT 1 FROM barcodes bc WHERE bc.product_package_id=package_id AND bc.barcode=?4) OR EXISTS(SELECT 1 FROM inventory_batches ib WHERE ib.branch_id=?1 AND ib.product_package_id=package_id AND ib.batch_number=?4))";
    let full = format!(
        "SELECT *, CASE WHEN quantity>0 THEN 'in_stock' ELSE 'out_of_stock' END AS status FROM ({inner}) t {filter}"
    );
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
                Ok(PosProduct {
                    package_id: r.get(0)?,
                    product_id: r.get(1)?,
                    product_name: r.get(2)?,
                    package_label: r.get(3)?,
                    pack_size: r.get(4)?,
                    selling_price_minor: r.get(5)?,
                    quantity: r.get(6)?,
                    status: r.get(7)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    tx.commit()?;
    Ok(PosProductPage {
        items,
        total,
        limit,
        offset,
    })
}