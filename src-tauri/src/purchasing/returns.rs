use super::models::*;
use crate::{
    db::{
        catalog_validation::{id, invalid, required, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory::{
        self,
        models::{MovementType, StockOperation},
        movements,
    },
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashSet;

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn conflict(message: &str) -> AppError {
    AppError::new(ErrorCode::Conflict, message)
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

fn return_on(c: &Connection, value: &str) -> Result<PurchaseReturn, AppError> {
    id(value)?;
    let mut result = c
        .query_row(
            "SELECT x.id,x.return_number,x.purchase_order_id,o.purchase_number,x.receipt_id,r.receipt_number,x.branch_id,b.code,b.name,x.supplier_id,coalesce(o.supplier_code_snapshot,s.code),coalesce(o.supplier_name_snapshot,s.display_name),x.source_purchase_revision,x.reason,x.total_quantity,x.created_by,x.posted_at,x.created_at FROM purchase_returns x JOIN purchase_orders o ON o.id=x.purchase_order_id JOIN purchase_receipts r ON r.id=x.receipt_id JOIN branches b ON b.id=x.branch_id JOIN suppliers s ON s.id=x.supplier_id WHERE x.id=?1 AND x.status='POSTED'",
            [value],
            |r| Ok(PurchaseReturn {
                id: r.get(0)?, return_number: r.get(1)?, purchase_order_id: r.get(2)?, purchase_number: r.get(3)?,
                receipt_id: r.get(4)?, receipt_number: r.get(5)?, branch_id: r.get(6)?, branch_code: r.get(7)?, branch_name: r.get(8)?,
                supplier_id: r.get(9)?, supplier_code: r.get(10)?, supplier_name: r.get(11)?, source_purchase_revision: r.get(12)?,
                reason: r.get(13)?, total_quantity: r.get(14)?, created_by: r.get(15)?, posted_at: r.get(16)?, created_at: r.get(17)?, lines: vec![],
            }),
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Purchase return not found."))?;
    let mut statement = c.prepare("SELECT l.id,l.receipt_line_id,l.purchase_order_line_id,l.product_package_id,l.product_name_snapshot,l.package_label_snapshot,source.quantity,l.previously_returned_quantity,l.quantity,l.lot_number,l.expiry_date,l.acquisition_cost_minor,l.batch_id,l.movement_id,l.position FROM purchase_return_lines l JOIN purchase_receipt_lines source ON source.id=l.receipt_line_id WHERE l.return_id=?1 ORDER BY l.position,l.id")?;
    result.lines = statement
        .query_map([value], |r| {
            let received: i64 = r.get(6)?;
            let prior: i64 = r.get(7)?;
            let quantity: i64 = r.get(8)?;
            Ok(PurchaseReturnLine {
                id: r.get(0)?,
                receipt_line_id: r.get(1)?,
                purchase_order_line_id: r.get(2)?,
                product_package_id: r.get(3)?,
                product_name_snapshot: r.get(4)?,
                package_label_snapshot: r.get(5)?,
                received_quantity: received,
                previously_returned_quantity: prior,
                remaining_returnable_quantity: received - prior - quantity,
                quantity,
                lot_number: r.get(9)?,
                expiry_date: r.get(10)?,
                acquisition_cost_minor: r.get(11)?,
                batch_id: r.get(12)?,
                movement_id: r.get(13)?,
                position: r.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}

pub fn get(db: &AppDb, value: &str) -> Result<PurchaseReturn, AppError> {
    let c = db.lock()?;
    return_on(&c, value)
}

#[derive(Clone)]
struct Source {
    purchase_line: String,
    package: String,
    product: String,
    label: String,
    received: i64,
    lot: Option<String>,
    expiry: Option<String>,
    cost: i64,
    batch: String,
}

pub fn create(db: &AppDb, input: &CreatePurchaseReturn) -> Result<PurchaseReturn, AppError> {
    id(&input.id)?;
    id(&input.purchase_order_id)?;
    id(&input.receipt_id)?;
    id(&input.request_id)?;
    required(&input.reason)?;
    let reason = input.reason.trim();
    if reason.chars().count() > 2000 || reason.contains('\0') {
        return Err(invalid("Return reason must be at most 2000 characters."));
    }
    let payload =
        serde_json::to_string(input).map_err(|_| invalid("Invalid purchase return request."))?;
    let actor = clean(&input.created_by, 200, "Created by")?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;

    if let Some((old, result)) = tx
        .query_row(
            "SELECT request_payload,id FROM purchase_returns WHERE request_id=?1",
            [&input.request_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if old != payload {
            return Err(invalid(
                "Request ID was already used with another return payload.",
            ));
        }
        return return_on(&tx, &result);
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM purchase_returns WHERE id=?1)",
        [&input.id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Purchase return ID already exists."));
    }
    let (status, revision, supplier): (String, i64, String) = tx
        .query_row(
            "SELECT status,revision,supplier_id FROM purchase_orders WHERE id=?1",
            [&input.purchase_order_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Purchase order not found."))?;
    if !["PARTIALLY_RECEIVED", "RECEIVED"].contains(&status.as_str()) {
        return Err(invalid(
            "Only purchases with posted receipts can return stock.",
        ));
    }
    if revision != input.expected_revision {
        return Err(conflict(
            "Purchase order changed. Reload before returning stock.",
        ));
    }
    let (receipt_purchase, branch): (String, String) = tx
        .query_row("SELECT purchase_order_id,branch_id FROM purchase_receipts WHERE id=?1 AND status='POSTED'", [&input.receipt_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "Purchase receipt not found."))?;
    if receipt_purchase != input.purchase_order_id {
        return Err(invalid("Receipt does not belong to this purchase order."));
    }
    if input.lines.is_empty() || input.lines.len() > 400 {
        return Err(invalid("Add 1–400 return lines."));
    }
    let mut seen = HashSet::new();
    let mut prepared = Vec::new();
    let mut total = 0i64;
    for line in &input.lines {
        id(&line.receipt_line_id)?;
        inventory::quantity(line.quantity)?;
        if !seen.insert(line.receipt_line_id.clone()) {
            return Err(invalid("Return receipt lines must be unique."));
        }
        let source = tx
            .query_row("SELECT purchase_order_line_id,product_package_id,product_name_snapshot,package_label_snapshot,quantity,lot_number,expiry_date,acquisition_cost_minor,batch_id FROM purchase_receipt_lines WHERE id=?1 AND receipt_id=?2 AND purchase_order_id=?3 AND branch_id=?4", params![line.receipt_line_id,input.receipt_id,input.purchase_order_id,branch], |r| Ok(Source { purchase_line:r.get(0)?, package:r.get(1)?, product:r.get(2)?, label:r.get(3)?, received:r.get(4)?, lot:r.get(5)?, expiry:r.get(6)?, cost:r.get(7)?, batch:r.get(8)? }))
            .optional()?
            .ok_or_else(|| invalid("Return line must belong to the selected canonical receipt."))?;
        let returned: i64 = tx.query_row("SELECT coalesce(sum(l.quantity),0) FROM purchase_return_lines l JOIN purchase_returns r ON r.id=l.return_id WHERE l.receipt_line_id=?1 AND r.status='POSTED'", [&line.receipt_line_id], |r| r.get(0))?;
        if returned
            .checked_add(line.quantity)
            .is_none_or(|v| v > source.received)
        {
            return Err(invalid(
                "Return quantity exceeds the receipt line's remaining returnable quantity.",
            ));
        }
        inventory::scope(&tx, &branch, &source.package, true)?;
        let batch = inventory::batches::get(&tx, &branch, &source.batch)?;
        if !batch.is_active || batch.product_package_id != source.package {
            return Err(invalid("Original receipt batch is inactive or invalid."));
        }
        let available = inventory::balances::batch_on(&tx, &branch, &source.batch)?.quantity;
        if line.quantity > available {
            return Err(invalid(
                "Insufficient current stock in the original receipt batch.",
            ));
        }
        total = total
            .checked_add(line.quantity)
            .filter(|v| *v <= MAX_SAFE_MINOR)
            .ok_or_else(|| invalid("Return quantity exceeds the supported range."))?;
        prepared.push((
            line.receipt_line_id.clone(),
            line.quantity,
            source,
            returned,
        ));
    }
    let day: String = tx.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    let sequence: i64 = tx.query_row("INSERT INTO purchase_return_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1 RETURNING next_number", [&day], |r| r.get(0))?;
    let number = format!("PRT-{day}-{sequence:04}");
    tx.execute("INSERT INTO purchase_returns(id,return_number,purchase_order_id,receipt_id,branch_id,supplier_id,request_id,request_payload,source_purchase_revision,reason,total_quantity,created_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![input.id,number,input.purchase_order_id,input.receipt_id,branch,supplier,input.request_id,payload,input.expected_revision,reason,total,actor])?;
    for (position, (receipt_line, quantity, source, prior)) in prepared.into_iter().enumerate() {
        let movement = movements::insert_referenced(
            &tx,
            &StockOperation {
                request_id: uuid(),
                branch_id: branch.clone(),
                product_package_id: source.package.clone(),
                batch_id: source.batch.clone(),
                quantity,
                reason: format!("Purchase return {number}: {reason}"),
                created_by: actor.clone(),
            },
            MovementType::PurchaseReturn,
            None,
            Some(("purchase_return", &input.id)),
        )?;
        tx.execute("INSERT INTO purchase_return_lines(id,return_id,purchase_order_id,receipt_id,branch_id,receipt_line_id,purchase_order_line_id,position,product_package_id,product_name_snapshot,package_label_snapshot,previously_returned_quantity,quantity,lot_number,expiry_date,acquisition_cost_minor,batch_id,movement_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)", params![uuid(),input.id,input.purchase_order_id,input.receipt_id,branch,receipt_line,source.purchase_line,position as i64,source.package,source.product,source.label,prior,quantity,source.lot,source.expiry,source.cost,source.batch,movement.id])?;
    }
    tx.execute("UPDATE purchase_orders SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", [&input.purchase_order_id])?;
    tx.execute("UPDATE purchase_returns SET status='POSTED',posted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", [&input.id])?;
    let result = return_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(result)
}

pub fn list(db: &AppDb, query: &PurchaseReturnQuery) -> Result<PurchaseReturnPage, AppError> {
    let (limit, offset) = inventory::page(query.limit, query.offset)?;
    for value in [
        &query.purchase_order_id,
        &query.supplier_id,
        &query.receipt_id,
        &query.branch_id,
    ]
    .into_iter()
    .flatten()
    {
        id(value)?;
    }
    let search = query.search.as_deref().unwrap_or("").trim();
    if search.chars().count() > 200 {
        return Err(invalid("Search text must be at most 200 characters."));
    }
    let c = db.lock()?;
    for value in [&query.date_from, &query.date_to].into_iter().flatten() {
        inventory::date(&c, value)?;
    }
    if matches!((&query.date_from,&query.date_to),(Some(a),Some(b)) if a>b) {
        return Err(invalid("Invalid date range."));
    }
    let filter = "x.status='POSTED' AND (?1 IS NULL OR x.purchase_order_id=?1) AND (?2 IS NULL OR x.supplier_id=?2) AND (?3 IS NULL OR x.receipt_id=?3) AND (?4 IS NULL OR x.branch_id=?4) AND (?5='' OR instr(lower(x.return_number||' '||o.purchase_number||' '||r.receipt_number||' '||x.reason),lower(?5))>0) AND (?6 IS NULL OR substr(x.posted_at,1,10)>=?6) AND (?7 IS NULL OR substr(x.posted_at,1,10)<=?7)";
    let total = c.query_row(&format!("SELECT count(*) FROM purchase_returns x JOIN purchase_orders o ON o.id=x.purchase_order_id JOIN purchase_receipts r ON r.id=x.receipt_id WHERE {filter}"), params![query.purchase_order_id,query.supplier_id,query.receipt_id,query.branch_id,search,query.date_from,query.date_to], |r| r.get(0))?;
    let mut statement = c.prepare(&format!("SELECT x.id,x.return_number,x.purchase_order_id,o.purchase_number,x.receipt_id,r.receipt_number,x.branch_id,b.code,b.name,x.supplier_id,coalesce(o.supplier_code_snapshot,s.code),coalesce(o.supplier_name_snapshot,s.display_name),x.total_quantity,x.reason,x.posted_at FROM purchase_returns x JOIN purchase_orders o ON o.id=x.purchase_order_id JOIN purchase_receipts r ON r.id=x.receipt_id JOIN branches b ON b.id=x.branch_id JOIN suppliers s ON s.id=x.supplier_id WHERE {filter} ORDER BY x.posted_at DESC,x.id DESC LIMIT ?8 OFFSET ?9"))?;
    let items = statement
        .query_map(
            params![
                query.purchase_order_id,
                query.supplier_id,
                query.receipt_id,
                query.branch_id,
                search,
                query.date_from,
                query.date_to,
                limit,
                offset
            ],
            |r| {
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
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PurchaseReturnPage {
        items,
        total,
        limit,
        offset,
    })
}
