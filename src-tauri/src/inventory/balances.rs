use super::*;
use crate::db::{catalog_validation::id, connection::AppDb};
use models::*;
use rusqlite::{params, Connection};
pub(crate) fn package_on(
    c: &Connection,
    branch: &str,
    package: &str,
) -> Result<InventoryBalance, AppError> {
    scope(c, branch, package, false)?;
    let quantity=c.query_row("SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements INDEXED BY inventory_movements_package WHERE branch_id=?1 AND product_package_id=?2",params![branch,package],|r|r.get(0))?;
    Ok(InventoryBalance {
        branch_id: branch.into(),
        product_package_id: package.into(),
        batch_id: None,
        quantity,
    })
}
pub(crate) fn batch_on(
    c: &Connection,
    branch: &str,
    batch: &str,
) -> Result<InventoryBalance, AppError> {
    let b = batches::get(c, branch, batch)?;
    let quantity=c.query_row("SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements INDEXED BY inventory_movements_batch WHERE batch_id=?1",[batch],|r|r.get(0))?;
    Ok(InventoryBalance {
        branch_id: branch.into(),
        product_package_id: b.product_package_id,
        batch_id: Some(batch.into()),
        quantity,
    })
}
pub fn package(db: &AppDb, branch: &str, package: &str) -> Result<InventoryBalance, AppError> {
    package_on(&*db.lock()?, branch, package)
}
pub fn batch(db: &AppDb, branch: &str, batch: &str) -> Result<InventoryBalance, AppError> {
    batch_on(&*db.lock()?, branch, batch)
}
/// Only packages intentionally introduced to branch inventory/configuration appear.
pub fn list(db: &AppDb, q: &InventoryQuery) -> Result<Vec<InventoryBalance>, AppError> {
    id(&q.branch_id)?;
    if let Some(p) = &q.product_package_id {
        id(p)?;
    }
    if q.batch_id.is_some() {
        return Err(invalid("Use get_batch_stock for a batch balance."));
    }
    let (l, o) = page(q.limit, q.offset)?;
    let c = db.lock()?;
    let mut s=c.prepare("WITH packages AS (SELECT product_package_id FROM inventory_batches WHERE branch_id=?1 UNION SELECT product_package_id FROM inventory_levels WHERE branch_id=?1) SELECT p.product_package_id,coalesce((SELECT sum(quantity_delta) FROM inventory_movements INDEXED BY inventory_movements_package WHERE branch_id=?1 AND product_package_id=p.product_package_id),0) FROM packages p WHERE (?2 IS NULL OR p.product_package_id=?2) ORDER BY p.product_package_id LIMIT ?3 OFFSET ?4")?;
    let result = s
        .query_map(params![q.branch_id, q.product_package_id, l, o], |r| {
            Ok(InventoryBalance {
                branch_id: q.branch_id.clone(),
                product_package_id: r.get(0)?,
                batch_id: None,
                quantity: r.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(result)
}
