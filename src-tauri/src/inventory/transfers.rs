use super::*;
use crate::db::{
    catalog_validation::{id, invalid, optional, required},
    connection::AppDb,
};
use models::*;
use rusqlite::{params, TransactionBehavior};
pub fn transfer(db: &AppDb, input: &TransferStock) -> Result<TransferResult, AppError> {
    id(&input.request_id)?;
    quantity(input.quantity)?;
    required(&input.reason)?;
    optional(&input.created_by)?;
    if input.source_branch_id == input.destination_branch_id {
        return Err(invalid("Transfer requires different branches."));
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    scope(
        &tx,
        &input.source_branch_id,
        &input.product_package_id,
        true,
    )?;
    scope(
        &tx,
        &input.destination_branch_id,
        &input.product_package_id,
        true,
    )?;
    let source = batches::get(&tx, &input.source_branch_id, &input.source_batch_id)?;
    if source.product_package_id != input.product_package_id {
        return Err(invalid("Source batch belongs to another package."));
    }
    let destination = batches::create_on(
        &tx,
        &CreateBatch {
            branch_id: input.destination_branch_id.clone(),
            product_package_id: input.product_package_id.clone(),
            lot_number: source.lot_number,
            expiry_date: source.expiry_date,
            received_at: Some(tx.query_row(
                "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                [],
                |r| r.get(0),
            )?),
            cost_price_minor: source.cost_price_minor,
            supplier_reference: source.supplier_reference,
        },
    )?;
    let outgoing = uuid::Uuid::new_v4().to_string();
    let incoming = uuid::Uuid::new_v4().to_string();
    tx.execute("INSERT INTO inventory_transfers(id,source_branch_id,destination_branch_id,product_package_id,source_batch_id,destination_batch_id,quantity,out_movement_id,in_movement_id,reason,created_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![input.request_id,input.source_branch_id,input.destination_branch_id,input.product_package_id,input.source_batch_id,destination.id,input.quantity,outgoing,incoming,input.reason,input.created_by])?;
    let mut op = StockOperation {
        request_id: outgoing,
        branch_id: input.source_branch_id.clone(),
        product_package_id: input.product_package_id.clone(),
        batch_id: input.source_batch_id.clone(),
        quantity: input.quantity,
        reason: input.reason.clone(),
        created_by: input.created_by.clone(),
    };
    let out_movement =
        movements::insert(&tx, &op, MovementType::TransferOut, Some(&input.request_id))?;
    op.request_id = incoming;
    op.branch_id = input.destination_branch_id.clone();
    op.batch_id = destination.id.clone();
    let in_movement =
        movements::insert(&tx, &op, MovementType::TransferIn, Some(&input.request_id))?;
    tx.commit()?;
    Ok(TransferResult {
        id: input.request_id.clone(),
        source_batch_id: input.source_batch_id.clone(),
        destination_batch_id: destination.id,
        out_movement,
        in_movement,
    })
}
