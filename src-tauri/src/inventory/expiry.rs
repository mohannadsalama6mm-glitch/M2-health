use super::*;
use crate::db::{
    catalog_validation::{id, invalid},
    connection::AppDb,
};
use models::*;
use rusqlite::{params, Connection};
fn query(
    c: &Connection,
    branch: &str,
    package: Option<&str>,
    as_of: &str,
    days: Option<i64>,
    expired: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ExpiryItem>, AppError> {
    date(c, as_of)?;
    id(branch)?;
    let mut s=c.prepare(&format!("SELECT {},coalesce((SELECT sum(quantity_delta) FROM inventory_movements INDEXED BY inventory_movements_batch WHERE batch_id=b.id),0) AS quantity FROM inventory_batches b WHERE branch_id=?1 AND is_active=1 AND (?2 IS NULL OR product_package_id=?2) AND quantity>0 AND ((?3 IS NULL AND (expiry_date IS NULL OR expiry_date>=?4)) OR (?3 IS NOT NULL AND expiry_date IS NOT NULL AND ((?5=1 AND expiry_date<?4) OR (?5=0 AND expiry_date>=?4 AND expiry_date<=date(?4,'+'||?3||' days'))))) ORDER BY expiry_date IS NULL,expiry_date,received_at IS NULL,received_at,created_at,id LIMIT ?6 OFFSET ?7",batches::COLUMNS))?;
    let result = s
        .query_map(
            params![branch, package, days, as_of, expired, limit, offset],
            |r| {
                Ok(ExpiryItem {
                    batch: batches::map(r)?,
                    quantity: r.get(11)?,
                })
            },
        )?
        .collect::<Result<_, _>>()?;
    Ok(result)
}
pub fn expiring(db: &AppDb, q: &ExpiryQuery) -> Result<Vec<ExpiryItem>, AppError> {
    if !(0..=36500).contains(&q.within_days) {
        return Err(invalid("Expiry horizon must be 0–36500 days."));
    }
    let (l, o) = page(q.limit, q.offset)?;
    query(
        &*db.lock()?,
        &q.branch_id,
        None,
        &q.as_of,
        Some(q.within_days),
        q.expired_only,
        l,
        o,
    )
}
pub fn fefo(
    db: &AppDb,
    branch: &str,
    package: &str,
    as_of: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ExpiryItem>, AppError> {
    let (l, o) = page(limit, offset)?;
    let c = db.lock()?;
    scope(&c, branch, package, true)?;
    query(&c, branch, Some(package), as_of, None, false, l, o)
}
