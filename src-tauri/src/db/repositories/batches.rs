use crate::{
    db::{catalog_validation::*, connection::AppDb, models::inventory::*},
    error::{AppError, ErrorCode},
};
use rusqlite::{params, Connection, OptionalExtension, Row};
const COLUMNS: &str = "id, branch_id, product_package_id, batch_number, expiry_date, cost_price_minor, is_active, created_at, updated_at";
fn map(r: &Row<'_>) -> rusqlite::Result<InventoryBatch> {
    Ok(InventoryBatch {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        product_package_id: r.get(2)?,
        batch_number: r.get(3)?,
        expiry_date: r.get(4)?,
        cost_price_minor: r.get(5)?,
        is_active: r.get(6)?,
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
    })
}
pub(crate) fn get(c: &Connection, value: &str) -> Result<InventoryBatch, AppError> {
    c.query_row(
        &format!("SELECT {COLUMNS} FROM inventory_batches WHERE id=?1"),
        [value],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Batch not found."))
}
pub(crate) fn validate_expiry(value: &str) -> Result<(), AppError> {
    if !value.is_empty()
        && !(value.len() == 10
            && value.as_bytes().iter().enumerate().all(|(i, b)| match i {
                4 | 7 => *b == b'-',
                _ => b.is_ascii_digit(),
            }))
    {
        return Err(invalid(
            "Use a valid YYYY-MM-DD expiry date, or leave it empty.",
        ));
    }
    Ok(())
}
/// Finds (or creates) the branch-scoped lot with this package, lot number and expiry.
/// Cost is only applied when the batch is created; existing batches keep their cost.
pub(crate) fn ensure_on(
    c: &Connection,
    branch_id: &str,
    product_package_id: &str,
    batch_number: &str,
    expiry_date: &str,
    cost_price_minor: Option<i64>,
) -> Result<InventoryBatch, AppError> {
    let existing = c
        .query_row(
            &format!(
                "SELECT {COLUMNS} FROM inventory_batches WHERE branch_id=?1 AND product_package_id=?2 AND batch_number=?3 AND COALESCE(expiry_date,'')=?4 LIMIT 1"
            ),
            params![branch_id, product_package_id, batch_number, expiry_date],
            map,
        )
        .optional()?;
    if let Some(batch) = existing {
        return Ok(batch);
    }
    let value = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO inventory_batches(id,branch_id,product_package_id,batch_number,expiry_date,cost_price_minor) VALUES (?1,?2,?3,?4,?5,?6)",
        params![value, branch_id, product_package_id, batch_number, expiry_date, cost_price_minor],
    )?;
    get(c, &value)
}
/// Active batches of a package in a branch ordered FEFO (earliest expiry first,
/// no-expiry lots last), each with its live ledger-derived balance.
pub(crate) fn list_active_with_balance_on(
    c: &Connection,
    branch_id: &str,
    product_package_id: &str,
) -> Result<Vec<(InventoryBatch, i64)>, AppError> {
    let mut s = c.prepare(
        "SELECT b.id, b.branch_id, b.product_package_id, b.batch_number, b.expiry_date, b.cost_price_minor, b.is_active, b.created_at, b.updated_at, COALESCE(bal.quantity,0) FROM inventory_batches b LEFT JOIN stock_balances bal ON bal.batch_id=b.id WHERE b.branch_id=?1 AND b.product_package_id=?2 AND b.is_active=1 ORDER BY CASE WHEN b.expiry_date='' THEN 1 ELSE 0 END, b.expiry_date, b.created_at, b.id",
    )?;
    let rows = s.query_map(params![branch_id, product_package_id], |r| {
        Ok((map(r)?, r.get::<_, i64>(9)?))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
/// Live ledger balance of one batch.
pub(crate) fn balance_on(c: &Connection, batch_id: &str) -> Result<i64, AppError> {
    Ok(c.query_row(
        &format!("SELECT COALESCE(quantity,0) FROM stock_balances WHERE batch_id=?1"),
        [batch_id],
        |r| r.get(0),
    )?)
}
/// Total live balance across all active batches of a package in a branch.
pub(crate) fn package_balance_on(
    c: &Connection,
    branch_id: &str,
    product_package_id: &str,
) -> Result<i64, AppError> {
    Ok(c.query_row(
        "SELECT COALESCE(SUM(COALESCE(bal.quantity,0)),0) FROM stock_balances bal JOIN inventory_batches b ON b.id=bal.batch_id WHERE b.branch_id=?1 AND b.product_package_id=?2 AND b.is_active=1",
        params![branch_id, product_package_id],
        |r| r.get(0),
    )?)
}
/// All active batches in a branch, optionally restricted to a category scope, with
/// their positive balances. Used by stock-count snapshots.
pub(crate) fn active_in_branch_on(
    c: &Connection,
    branch_id: &str,
    category_id: Option<&str>,
) -> Result<Vec<(InventoryBatch, i64)>, AppError> {
    let sql = "SELECT b.id, b.branch_id, b.product_package_id, b.batch_number, b.expiry_date, b.cost_price_minor, b.is_active, b.created_at, b.updated_at, COALESCE(bal.quantity,0) FROM inventory_batches b JOIN product_packages pp ON pp.id=b.product_package_id JOIN products pr ON pr.id=pp.product_id LEFT JOIN stock_balances bal ON bal.batch_id=b.id WHERE b.branch_id=?1 AND b.is_active=1 AND (?2 IS NULL OR pr.category_id=?2) AND COALESCE(bal.quantity,0)>0 ORDER BY b.created_at, b.id";
    let mut s = c.prepare(sql)?;
    let rows = s.query_map(params![branch_id, category_id], |r| {
        Ok((map(r)?, r.get::<_, i64>(9)?))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
pub fn set_active(db: &AppDb, value: &str, active: bool) -> Result<InventoryBatch, AppError> {
    id(value)?;
    let c = db.lock()?;
    c.execute("UPDATE inventory_batches SET is_active=?1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(updated_at)+0.001/86400)) WHERE id=?2",params![active,value])?;
    get(&c, value)
}
