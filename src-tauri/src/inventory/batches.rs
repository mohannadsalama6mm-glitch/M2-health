use super::*;
use crate::db::{
    catalog_validation::{id, invalid, optional, MAX_SAFE_MINOR},
    connection::AppDb,
};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
pub(crate) const COLUMNS:&str="id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at";
pub(crate) fn map(r: &Row) -> rusqlite::Result<InventoryBatch> {
    Ok(InventoryBatch {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        product_package_id: r.get(2)?,
        lot_number: r.get(3)?,
        expiry_date: r.get(4)?,
        received_at: r.get(5)?,
        cost_price_minor: r.get(6)?,
        supplier_reference: r.get(7)?,
        is_active: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
    })
}
pub(crate) fn get(c: &Connection, branch: &str, batch: &str) -> Result<InventoryBatch, AppError> {
    id(branch)?;
    id(batch)?;
    c.query_row(
        &format!("SELECT {COLUMNS} FROM inventory_batches WHERE id=?1 AND branch_id=?2"),
        params![batch, branch],
        map,
    )
    .optional()?
    .ok_or_else(|| invalid("Batch does not belong to the specified branch."))
}
pub(crate) fn create_on(c: &Connection, input: &CreateBatch) -> Result<InventoryBatch, AppError> {
    scope(c, &input.branch_id, &input.product_package_id, true)?;
    optional(&input.lot_number)?;
    optional(&input.supplier_reference)?;
    if let Some(d) = &input.expiry_date {
        date(c, d)?;
    }
    if input
        .cost_price_minor
        .is_some_and(|v| !(0..=MAX_SAFE_MINOR).contains(&v))
    {
        return Err(invalid("Cost must be non-negative integer minor units."));
    }
    let id = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![id,input.branch_id,input.product_package_id,input.lot_number,input.expiry_date,input.received_at,input.cost_price_minor,input.supplier_reference])?;
    get(c, &input.branch_id, &id)
}
pub fn create(db: &AppDb, input: &CreateBatch) -> Result<InventoryBatch, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let b = create_on(&tx, input)?;
    tx.commit()?;
    Ok(b)
}
