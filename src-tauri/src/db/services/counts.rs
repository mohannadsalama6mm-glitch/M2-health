use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::inventory::*,
        repositories::{batches, categories as categories_repo, counts, movements},
    },
    error::AppError,
};
use rusqlite::TransactionBehavior;
/// Stock count lifecycle. Snapshots the ledger for every in-stock batch at creation,
/// records counted quantities, and posts one count_correction ledger movement per
/// discrepancy on completion. Historical movements are never rewritten.
pub fn create_count(db: &AppDb, input: &CreateCountInput) -> Result<StockCount, AppError> {
    id(&input.branch_id)?;
    if input.scope.len() > 160 {
        return Err(invalid("The scope label is too long."));
    }
    if let Some(category) = &input.category_id {
        id(category)?;
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::db::repositories::branches::get(&tx, &input.branch_id)?;
    if input.category_id.is_some() {
        categories_repo::get(&tx, input.category_id.as_deref().unwrap_or(""))?;
    }
    let count_id = uuid::Uuid::new_v4().to_string();
    let count = counts::insert_count_on(
        &tx,
        &count_id,
        &input.branch_id,
        &input.scope.trim(),
        input.category_id.as_deref(),
    )?;
    for (batch, qty) in
        batches::active_in_branch_on(&tx, &input.branch_id, input.category_id.as_deref())?
    {
        counts::insert_item_on(&tx, &count_id, &batch.product_package_id, &batch.id, qty)?;
    }
    tx.commit()?;
    Ok(count)
}
pub fn save_item(db: &AppDb, input: &SaveCountItemInput) -> Result<StockCountItem, AppError> {
    id(&input.stock_count_id)?;
    id(&input.batch_id)?;
    if input.counted_quantity < 0 {
        return Err(invalid("A counted quantity cannot be negative."));
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let count = counts::get(&tx, &input.stock_count_id)?;
    if count.status == "completed" {
        return Err(invalid("This stock count session is already completed."));
    }
    counts::item_by_batch_on(&tx, &input.stock_count_id, &input.batch_id)?;
    counts::mark_in_progress_on(&tx, &input.stock_count_id)?;
    let item = counts::save_counted_on(
        &tx,
        &input.stock_count_id,
        &input.batch_id,
        input.counted_quantity,
    )?;
    tx.commit()?;
    Ok(item)
}
pub fn complete_count(
    db: &AppDb,
    input: &CompleteCountInput,
) -> Result<StockCountDetail, AppError> {
    id(&input.stock_count_id)?;
    {
        let mut c = db.lock()?;
        let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let count = counts::get(&tx, &input.stock_count_id)?;
        if count.status != "completed" {
            let items = counts::list_all_items_on(&tx, &input.stock_count_id)?;
            for item in items {
                if item.variance == 0 {
                    continue;
                }
                movements::insert_on(
                    &tx,
                    &count.branch_id,
                    &item.product_package_id,
                    &item.batch_id,
                    "count_correction",
                    item.variance,
                    None,
                    Some("stock_count"),
                    Some(&count.id),
                    &format!("Stock count {}", count.id),
                    "",
                )?;
            }
            counts::complete_on(&tx, &input.stock_count_id)?;
        }
        tx.commit()?;
    }
    counts::detail(db, &input.stock_count_id)
}
