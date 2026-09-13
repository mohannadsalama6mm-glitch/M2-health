use crate::{
    db::{catalog_validation::*, connection::AppDb, models::inventory::*},
    error::AppError,
};
use rusqlite::{params, TransactionBehavior};
pub fn set_reorder(db: &AppDb, input: &SetReorderInput) -> Result<InventorySetting, AppError> {
    id(&input.branch_id)?;
    id(&input.product_package_id)?;
    if !(0..=MAX_SAFE_MINOR).contains(&input.reorder_level) {
        return Err(invalid(
            "Reorder level must be a non-negative integer within the supported range.",
        ));
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    super::branches::get(&tx, &input.branch_id)?;
    super::product_packages::get(&tx, &input.product_package_id)?;
    tx.execute(
        "INSERT INTO inventory_settings(branch_id,product_package_id,reorder_level) VALUES (?1,?2,?3) ON CONFLICT(branch_id,product_package_id) DO UPDATE SET reorder_level=excluded.reorder_level,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ',max(julianday('now'),julianday(inventory_settings.updated_at)+0.001/86400))",
        params![input.branch_id, input.product_package_id, input.reorder_level],
    )?;
    let setting = tx.query_row(
        "SELECT branch_id, product_package_id, reorder_level, created_at, updated_at FROM inventory_settings WHERE branch_id=?1 AND product_package_id=?2",
        params![input.branch_id, input.product_package_id],
        |r| {
            Ok(InventorySetting {
                branch_id: r.get(0)?,
                product_package_id: r.get(1)?,
                reorder_level: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
            })
        },
    )?;
    tx.commit()?;
    Ok(setting)
}
