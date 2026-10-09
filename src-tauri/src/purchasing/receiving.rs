use super::models::*;
use crate::{
    db::{
        catalog_validation::{id, invalid, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory::{
        self, batches,
        models::{CreateBatch, MovementType, StockOperation},
        movements,
    },
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashMap;

fn conflict(message: &str) -> AppError {
    AppError::new(ErrorCode::Conflict, message)
}
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn clean(value: &Option<String>, max: usize, label: &str) -> Result<Option<String>, AppError> {
    let value = value.as_deref().unwrap_or("").trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max || value.contains('\0') {
        return Err(invalid(&format!(
            "{label} must be at most {max} characters."
        )));
    }
    Ok(Some(value.to_string()))
}
fn receipt_on(c: &Connection, value: &str) -> Result<PurchaseReceipt, AppError> {
    id(value)?;
    let mut receipt=c.query_row("SELECT r.id,r.receipt_number,r.purchase_order_id,o.purchase_number,r.branch_id,b.code,b.name,o.supplier_id,coalesce(o.supplier_code_snapshot,s.code),coalesce(o.supplier_name_snapshot,s.display_name),r.total_quantity,r.created_by,r.posted_at,r.created_at FROM purchase_receipts r JOIN purchase_orders o ON o.id=r.purchase_order_id JOIN suppliers s ON s.id=o.supplier_id JOIN branches b ON b.id=r.branch_id WHERE r.id=?1 AND r.status='POSTED'",[value],|r|Ok(PurchaseReceipt{id:r.get(0)?,receipt_number:r.get(1)?,purchase_order_id:r.get(2)?,purchase_number:r.get(3)?,branch_id:r.get(4)?,branch_code:r.get(5)?,branch_name:r.get(6)?,supplier_id:r.get(7)?,supplier_code:r.get(8)?,supplier_name:r.get(9)?,total_quantity:r.get(10)?,created_by:r.get(11)?,posted_at:r.get(12)?,created_at:r.get(13)?,lines:vec![],returns:vec![]})).optional()?.ok_or_else(||AppError::new(ErrorCode::NotFound,"Purchase receipt not found."))?;
    let mut lines=c.prepare("SELECT l.id,l.purchase_order_line_id,l.product_package_id,l.product_name_snapshot,l.package_label_snapshot,l.quantity,l.lot_number,l.expiry_date,l.acquisition_cost_minor,l.batch_id,l.movement_id,l.position,coalesce((SELECT sum(x.quantity) FROM purchase_return_lines x JOIN purchase_returns r ON r.id=x.return_id WHERE x.receipt_line_id=l.id AND r.status='POSTED'),0),coalesce((SELECT sum(m.quantity_delta) FROM inventory_movements m WHERE m.batch_id=l.batch_id),0) FROM purchase_receipt_lines l WHERE l.receipt_id=?1 ORDER BY l.position,l.id")?;
    receipt.lines = lines
        .query_map([value], |r| {
            let quantity: i64 = r.get(5)?;
            let returned: i64 = r.get(12)?;
            Ok(PurchaseReceiptLine {
                id: r.get(0)?,
                purchase_order_line_id: r.get(1)?,
                product_package_id: r.get(2)?,
                product_name_snapshot: r.get(3)?,
                package_label_snapshot: r.get(4)?,
                quantity,
                lot_number: r.get(6)?,
                expiry_date: r.get(7)?,
                acquisition_cost_minor: r.get(8)?,
                batch_id: r.get(9)?,
                movement_id: r.get(10)?,
                position: r.get(11)?,
                returned_quantity: returned,
                remaining_returnable_quantity: quantity - returned,
                current_batch_quantity: r.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut returns=c.prepare("SELECT x.id,x.return_number,x.purchase_order_id,o.purchase_number,x.receipt_id,r.receipt_number,x.branch_id,b.code,b.name,x.supplier_id,coalesce(o.supplier_code_snapshot,s.code),coalesce(o.supplier_name_snapshot,s.display_name),x.total_quantity,x.reason,x.posted_at FROM purchase_returns x JOIN purchase_orders o ON o.id=x.purchase_order_id JOIN purchase_receipts r ON r.id=x.receipt_id JOIN branches b ON b.id=x.branch_id JOIN suppliers s ON s.id=x.supplier_id WHERE x.receipt_id=?1 AND x.status='POSTED' ORDER BY x.posted_at,x.id")?;
    receipt.returns = returns
        .query_map([value], |r| {
            Ok(PurchaseReturnSummary {
                id: r.get(0)?,
                return_number: r.get(1)?,
                purchase_order_id: r.get(2)?,
                purchase_number: r.get(3)?,
                receipt_id: r.get(4)?,
                receipt_number: r.get(5)?,
                branch_id: r.get(6)?,
                branch_code: r.get(7)?,
                branch_name: r.get(8)?,
                supplier_id: r.get(9)?,
                supplier_code: r.get(10)?,
                supplier_name: r.get(11)?,
                total_quantity: r.get(12)?,
                reason: r.get(13)?,
                posted_at: r.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(receipt)
}
pub fn get(db: &AppDb, value: &str) -> Result<PurchaseReceipt, AppError> {
    let connection = db.lock()?;
    receipt_on(&connection, value)
}

#[derive(Clone)]
struct OrderedLine {
    package: String,
    product: String,
    label: String,
    ordered: i64,
    cost: i64,
}

pub fn receive(db: &AppDb, input: &ReceivePurchase) -> Result<PurchaseReceipt, AppError> {
    id(&input.id)?;
    id(&input.purchase_order_id)?;
    id(&input.request_id)?;
    id(&input.branch_id)?;
    let payload = serde_json::to_string(input).map_err(|_| invalid("Invalid receipt request."))?;
    let actor = clean(&input.created_by, 200, "Created by")?;
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some((old, result)) = tx
        .query_row(
            "SELECT request_payload,id FROM purchase_receipts WHERE request_id=?1",
            [&input.request_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if old != payload {
            return Err(invalid(
                "Request ID was already used with another receipt payload.",
            ));
        }
        return receipt_on(&tx, &result);
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM purchase_receipts WHERE id=?1)",
        [&input.id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Purchase receipt ID already exists."));
    }
    let (status, revision, supplier_reference): (String, i64, Option<String>) = tx
        .query_row(
            "SELECT status,revision,supplier_reference FROM purchase_orders WHERE id=?1",
            [&input.purchase_order_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Purchase order not found."))?;
    if !["ORDERED", "PARTIALLY_RECEIVED"].contains(&status.as_str()) {
        return Err(invalid(
            "Only ordered or partially received purchases can receive stock.",
        ));
    }
    if revision != input.expected_revision {
        return Err(conflict("Purchase order changed. Reload before receiving."));
    }
    if input.lines.is_empty() || input.lines.len() > 400 {
        return Err(invalid("Add 1–400 receipt allocations."));
    }
    let branch_active = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1 AND is_active=1)",
        [&input.branch_id],
        |r| r.get::<_, bool>(0),
    )?;
    if !branch_active {
        return Err(invalid("Select an active receiving branch."));
    }
    let mut statement=tx.prepare("SELECT id,product_package_id,product_name_snapshot,package_label_snapshot,ordered_quantity,unit_cost_minor FROM purchase_order_lines WHERE purchase_order_id=?1")?;
    let ordered: HashMap<String, OrderedLine> = statement
        .query_map([&input.purchase_order_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                OrderedLine {
                    package: r.get(1)?,
                    product: r.get(2)?,
                    label: r.get(3)?,
                    ordered: r.get(4)?,
                    cost: r.get(5)?,
                },
            ))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    drop(statement);
    let mut requested = HashMap::<String, i64>::new();
    let mut prepared = Vec::new();
    let mut total = 0i64;
    for line in &input.lines {
        id(&line.purchase_order_line_id)?;
        inventory::quantity(line.quantity)?;
        let source = ordered
            .get(&line.purchase_order_line_id)
            .ok_or_else(|| invalid("Receipt line does not belong to this purchase order."))?;
        inventory::scope(&tx, &input.branch_id, &source.package, true)?;
        let lot = clean(&line.lot_number, 4000, "Lot number")?;
        let expiry = clean(&line.expiry_date, 10, "Expiry date")?;
        if let Some(value) = &expiry {
            inventory::date(&tx, value)?;
        }
        let cost = line.acquisition_cost_minor.unwrap_or(source.cost);
        if !(0..=MAX_SAFE_MINOR).contains(&cost) {
            return Err(invalid(
                "Acquisition cost must be non-negative integer minor units.",
            ));
        }
        let already:i64=tx.query_row("SELECT coalesce(sum(quantity),0) FROM purchase_receipt_lines WHERE purchase_order_line_id=?1",[&line.purchase_order_line_id],|r|r.get(0))?;
        let cumulative = requested
            .entry(line.purchase_order_line_id.clone())
            .or_default();
        *cumulative = cumulative
            .checked_add(line.quantity)
            .ok_or_else(|| invalid("Receipt quantity exceeds the supported range."))?;
        if already
            .checked_add(*cumulative)
            .is_none_or(|v| v > source.ordered)
        {
            return Err(invalid(
                "Received quantity cannot exceed the purchase line remaining quantity.",
            ));
        }
        total = total
            .checked_add(line.quantity)
            .filter(|v| *v <= MAX_SAFE_MINOR)
            .ok_or_else(|| invalid("Receipt quantity exceeds the supported range."))?;
        prepared.push((
            line.purchase_order_line_id.clone(),
            source.clone(),
            line.quantity,
            lot,
            expiry,
            cost,
        ));
    }
    let day: String = tx.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    let sequence:i64=tx.query_row("INSERT INTO purchase_receipt_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1 RETURNING next_number",[&day],|r|r.get(0))?;
    let number = format!("PR-{day}-{sequence:04}");
    tx.execute("INSERT INTO purchase_receipts(id,receipt_number,purchase_order_id,branch_id,request_id,request_payload,total_quantity,created_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![input.id,number,input.purchase_order_id,input.branch_id,input.request_id,payload,total,actor])?;
    for (position, (order_line, source, quantity, lot, expiry, cost)) in
        prepared.into_iter().enumerate()
    {
        let batch:Option<String>=tx.query_row("SELECT id FROM inventory_batches WHERE branch_id=?1 AND product_package_id=?2 AND is_active=1 AND lot_number IS ?3 AND expiry_date IS ?4 AND cost_price_minor IS ?5 AND supplier_reference IS ?6 ORDER BY created_at,id LIMIT 1",params![input.branch_id,source.package,lot,expiry,cost,supplier_reference],|r|r.get(0)).optional()?;
        let batch = match batch {
            Some(value) => value,
            None => {
                batches::create_on(
                    &tx,
                    &CreateBatch {
                        branch_id: input.branch_id.clone(),
                        product_package_id: source.package.clone(),
                        lot_number: lot.clone(),
                        expiry_date: expiry.clone(),
                        received_at: Some(tx.query_row(
                            "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                            [],
                            |r| r.get(0),
                        )?),
                        cost_price_minor: Some(cost),
                        supplier_reference: supplier_reference.clone(),
                    },
                    actor.as_deref(),
                )?
                .id
            }
        };
        let movement = movements::insert_referenced(
            &tx,
            &StockOperation {
                request_id: uuid(),
                branch_id: input.branch_id.clone(),
                product_package_id: source.package.clone(),
                batch_id: batch.clone(),
                quantity,
                reason: format!("Purchase receipt {number}"),
                created_by: actor.clone(),
            },
            MovementType::PurchaseReceipt,
            None,
            Some(("purchase_receipt", &input.id)),
        )?;
        tx.execute("INSERT INTO purchase_receipt_lines(id,receipt_id,purchase_order_id,branch_id,purchase_order_line_id,position,product_package_id,product_name_snapshot,package_label_snapshot,quantity,lot_number,expiry_date,acquisition_cost_minor,batch_id,movement_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",params![uuid(),input.id,input.purchase_order_id,input.branch_id,order_line,position as i64,source.package,source.product,source.label,quantity,lot,expiry,cost,batch,movement.id])?;
    }
    let complete:bool=tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM purchase_order_lines l WHERE l.purchase_order_id=?1 AND coalesce((SELECT sum(rl.quantity) FROM purchase_receipt_lines rl WHERE rl.purchase_order_line_id=l.id),0)<l.ordered_quantity)",[&input.purchase_order_id],|r|r.get(0))?;
    let next = if complete {
        "RECEIVED"
    } else {
        "PARTIALLY_RECEIVED"
    };
    tx.execute("UPDATE purchase_orders SET status=?2,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![input.purchase_order_id,next])?;
    tx.execute("UPDATE purchase_receipts SET status='POSTED',posted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[&input.id])?;
    let result = receipt_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(result)
}

pub fn list(db: &AppDb, query: &PurchaseReceiptQuery) -> Result<PurchaseReceiptPage, AppError> {
    let (limit, offset) = inventory::page(query.limit, query.offset)?;
    for value in [
        &query.purchase_order_id,
        &query.supplier_id,
        &query.branch_id,
    ]
    .into_iter()
    .flatten()
    {
        id(value)?;
    }
    let connection = db.lock()?;
    let filter="r.status='POSTED' AND (?1 IS NULL OR r.purchase_order_id=?1) AND (?2 IS NULL OR o.supplier_id=?2) AND (?3 IS NULL OR r.branch_id=?3)";
    let total=connection.query_row(&format!("SELECT count(*) FROM purchase_receipts r JOIN purchase_orders o ON o.id=r.purchase_order_id WHERE {filter}"),params![query.purchase_order_id,query.supplier_id,query.branch_id],|r|r.get(0))?;
    let mut statement=connection.prepare(&format!("SELECT r.id,r.receipt_number,r.branch_id,b.code,b.name,r.total_quantity,r.posted_at FROM purchase_receipts r JOIN purchase_orders o ON o.id=r.purchase_order_id JOIN branches b ON b.id=r.branch_id WHERE {filter} ORDER BY r.posted_at DESC,r.id DESC LIMIT ?4 OFFSET ?5"))?;
    let items = statement
        .query_map(
            params![
                query.purchase_order_id,
                query.supplier_id,
                query.branch_id,
                limit,
                offset
            ],
            |r| {
                Ok(PurchaseReceiptSummary {
                    id: r.get(0)?,
                    receipt_number: r.get(1)?,
                    branch_id: r.get(2)?,
                    branch_code: r.get(3)?,
                    branch_name: r.get(4)?,
                    total_quantity: r.get(5)?,
                    posted_at: r.get(6)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PurchaseReceiptPage {
        items,
        total,
        limit,
        offset,
    })
}
