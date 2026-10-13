use super::*;
use crate::{
    cloud::outbox,
    db::{
        catalog_validation::{id, invalid, optional, required},
        connection::AppDb,
    },
};
use models::*;
use rusqlite::{params, Connection, Row, TransactionBehavior};
const COLUMNS:&str="id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at";
fn map(r: &Row) -> rusqlite::Result<InventoryMovement> {
    let kind: String = r.get(4)?;
    let movement_type = serde_json::from_value(serde_json::Value::String(kind)).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(InventoryMovement {
        id: r.get(0)?,
        branch_id: r.get(1)?,
        product_package_id: r.get(2)?,
        batch_id: r.get(3)?,
        movement_type,
        quantity_delta: r.get(5)?,
        reference_type: r.get(6)?,
        reference_id: r.get(7)?,
        reason: r.get(8)?,
        created_by: r.get(9)?,
        transfer_id: r.get(10)?,
        created_at: r.get(11)?,
    })
}
pub(crate) fn insert(
    c: &Connection,
    op: &StockOperation,
    kind: MovementType,
    transfer: Option<&str>,
) -> Result<InventoryMovement, AppError> {
    insert_referenced(c, op, kind, transfer, None)
}
pub(crate) fn insert_referenced(
    c: &Connection,
    op: &StockOperation,
    kind: MovementType,
    transfer: Option<&str>,
    reference: Option<(&str, &str)>,
) -> Result<InventoryMovement, AppError> {
    id(&op.request_id)?;
    quantity(op.quantity)?;
    required(&op.reason)?;
    optional(&op.created_by)?;
    scope(c, &op.branch_id, &op.product_package_id, true)?;
    let batch = batches::get(c, &op.branch_id, &op.batch_id)?;
    if batch.product_package_id != op.product_package_id || !batch.is_active {
        return Err(invalid("Batch must be active and belong to this package."));
    }
    if kind == MovementType::Sale
        && c.query_row(
            "SELECT coalesce(?1 < date('now'),0)",
            [&batch.expiry_date],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Err(invalid("Expired batches cannot supply a sale."));
    }
    if kind == MovementType::Expiry {
        if let Some(expiry) = &batch.expiry_date {
            let expired: bool = c.query_row("SELECT ?1 < date('now')", [expiry], |r| r.get(0))?;
            if !expired {
                return Err(invalid("Known expiry date has not passed."));
            }
        }
    }
    let delta = match kind {
        MovementType::Sale
        | MovementType::PurchaseReturn
        | MovementType::AdjustmentOut
        | MovementType::Damage
        | MovementType::Expiry
        | MovementType::TransferOut => -op.quantity,
        _ => op.quantity,
    };
    let balance = balances::batch_on(c, &op.branch_id, &op.batch_id)?.quantity;
    if delta < 0 && balance < op.quantity {
        return Err(invalid(
            "Insufficient stock in this batch. Negative stock is not allowed.",
        ));
    }
    let reference = reference.or_else(|| transfer.map(|id| ("inventory_transfer", id)));
    c.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reason,created_by,transfer_id,reference_type,reference_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![op.request_id,op.branch_id,op.product_package_id,op.batch_id,kind.as_str(),delta,op.reason,op.created_by,transfer,reference.map(|r|r.0),reference.map(|r|r.1)])?;
    let movement = c.query_row(
        &format!("SELECT {COLUMNS} FROM inventory_movements WHERE id=?1"),
        [&op.request_id],
        map,
    )?;
    outbox::inventory_movement(c, &movement)?;
    Ok(movement)
}
pub fn opening(db: &AppDb, input: &OpeningStock) -> Result<InventoryMovement, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let b = batches::create_on(&tx, &input.batch, input.created_by.as_deref())?;
    let op = StockOperation {
        request_id: input.request_id.clone(),
        branch_id: b.branch_id,
        product_package_id: b.product_package_id,
        batch_id: b.id,
        quantity: input.quantity,
        reason: input.reason.clone(),
        created_by: input.created_by.clone(),
    };
    let m = insert(&tx, &op, MovementType::Opening, None)?;
    tx.commit()?;
    Ok(m)
}
/// Only count completion calls this signed correction path, inside its IMMEDIATE transaction.
/// Existing canonical ledger constraints and stock limits remain authoritative.
pub(crate) fn count_correction(
    c: &Connection,
    branch: &str,
    package: &str,
    batch: &str,
    delta: i64,
    line: &str,
    number: &str,
    actor: Option<&str>,
) -> Result<String, AppError> {
    quantity(
        delta
            .checked_abs()
            .ok_or_else(|| invalid("Invalid count correction."))?,
    )?;
    scope(c, branch, package, true)?;
    let b = batches::get(c, branch, batch)?;
    if b.product_package_id != package || !b.is_active {
        return Err(invalid("Invalid batch or wrong branch/package."));
    }
    let live = balances::batch_on(c, branch, batch)?.quantity;
    if live.checked_add(delta).is_none_or(|q| q < 0) {
        return Err(invalid("Count correction cannot create negative stock."));
    }
    let id = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by) VALUES(?1,?2,?3,?4,'STOCK_COUNT_CORRECTION',?5,'stock_count_line',?6,?7,?8)",params![id,branch,package,batch,delta,line,format!("Stock count {number} reconciliation"),actor])?;
    let movement = c.query_row(
        &format!("SELECT {COLUMNS} FROM inventory_movements WHERE id=?1"),
        [&id],
        map,
    )?;
    outbox::inventory_movement(c, &movement)?;
    Ok(id)
}
pub fn opening_existing(db: &AppDb, op: &StockOperation) -> Result<InventoryMovement, AppError> {
    post(db, op, MovementType::Opening)
}
pub fn adjust(db: &AppDb, input: &Adjustment) -> Result<InventoryMovement, AppError> {
    post(
        db,
        &input.operation,
        match input.direction {
            AdjustmentDirection::In => MovementType::AdjustmentIn,
            AdjustmentDirection::Out => MovementType::AdjustmentOut,
        },
    )
}
pub fn damage(db: &AppDb, op: &StockOperation) -> Result<InventoryMovement, AppError> {
    post(db, op, MovementType::Damage)
}
pub fn write_off(db: &AppDb, op: &StockOperation) -> Result<InventoryMovement, AppError> {
    post(db, op, MovementType::Expiry)
}
fn post(
    db: &AppDb,
    op: &StockOperation,
    kind: MovementType,
) -> Result<InventoryMovement, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let m = insert(&tx, op, kind, None)?;
    tx.commit()?;
    Ok(m)
}
pub fn list(db: &AppDb, q: &InventoryQuery) -> Result<Vec<InventoryMovement>, AppError> {
    id(&q.branch_id)?;
    if let Some(p) = &q.product_package_id {
        id(p)?;
    }
    if let Some(b) = &q.batch_id {
        id(b)?;
    }
    let (l, o) = page(q.limit, q.offset)?;
    let c = db.lock()?;
    let mut stmt=c.prepare(&format!("SELECT {COLUMNS} FROM inventory_movements WHERE branch_id=?1 AND (?2 IS NULL OR product_package_id=?2) AND (?3 IS NULL OR batch_id=?3) ORDER BY created_at DESC,id DESC LIMIT ?4 OFFSET ?5"))?;
    let result = stmt
        .query_map(
            params![q.branch_id, q.product_package_id, q.batch_id, l, o],
            map,
        )?
        .collect::<Result<_, _>>()?;
    Ok(result)
}
