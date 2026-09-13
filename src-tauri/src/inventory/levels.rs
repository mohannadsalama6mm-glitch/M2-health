use super::*;
use crate::db::{
    catalog_validation::{invalid, MAX_SAFE_MINOR},
    connection::AppDb,
};
use models::*;
use rusqlite::{params, TransactionBehavior};
pub fn set(db: &AppDb, input: &SetLevel) -> Result<InventoryLevel, AppError> {
    for value in [
        Some(input.reorder_level),
        input.minimum_stock,
        input.maximum_stock,
    ]
    .into_iter()
    .flatten()
    {
        if !(0..=MAX_SAFE_MINOR).contains(&value) {
            return Err(invalid(
                "Inventory levels must be non-negative whole package counts.",
            ));
        }
    }
    if input
        .minimum_stock
        .zip(input.maximum_stock)
        .is_some_and(|(a, b)| a > b)
    {
        return Err(invalid("Minimum stock must not exceed maximum stock."));
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    scope(&tx, &input.branch_id, &input.product_package_id, true)?;
    tx.execute("INSERT INTO inventory_levels(id,branch_id,product_package_id,reorder_level,minimum_stock,maximum_stock) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(branch_id,product_package_id) DO UPDATE SET reorder_level=excluded.reorder_level,minimum_stock=excluded.minimum_stock,maximum_stock=excluded.maximum_stock,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![uuid::Uuid::new_v4().to_string(),input.branch_id,input.product_package_id,input.reorder_level,input.minimum_stock,input.maximum_stock])?;
    let result=tx.query_row("SELECT id,branch_id,product_package_id,reorder_level,minimum_stock,maximum_stock,created_at,updated_at FROM inventory_levels WHERE branch_id=?1 AND product_package_id=?2",params![input.branch_id,input.product_package_id],|r|Ok(InventoryLevel{id:r.get(0)?,branch_id:r.get(1)?,product_package_id:r.get(2)?,reorder_level:r.get(3)?,minimum_stock:r.get(4)?,maximum_stock:r.get(5)?,created_at:r.get(6)?,updated_at:r.get(7)?}))?;
    tx.commit()?;
    Ok(result)
}
pub fn low(db: &AppDb, q: &InventoryQuery) -> Result<Vec<LowStockItem>, AppError> {
    crate::db::catalog_validation::id(&q.branch_id)?;
    let (l, o) = page(q.limit, q.offset)?;
    if q.batch_id.is_some() {
        return Err(invalid("Reorder levels are package-scoped."));
    }
    if let Some(p) = &q.product_package_id {
        crate::db::catalog_validation::id(p)?;
    }
    let c = db.lock()?;
    let mut s=c.prepare("SELECT l.product_package_id,l.reorder_level,coalesce((SELECT sum(quantity_delta) FROM inventory_movements INDEXED BY inventory_movements_package WHERE branch_id=l.branch_id AND product_package_id=l.product_package_id),0) AS quantity FROM inventory_levels l WHERE l.branch_id=?1 AND (?2 IS NULL OR l.product_package_id=?2) AND quantity<=l.reorder_level ORDER BY l.product_package_id LIMIT ?3 OFFSET ?4")?;
    let result = s
        .query_map(params![q.branch_id, q.product_package_id, l, o], |r| {
            Ok(LowStockItem {
                branch_id: q.branch_id.clone(),
                product_package_id: r.get(0)?,
                reorder_level: r.get(1)?,
                quantity: r.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(result)
}
