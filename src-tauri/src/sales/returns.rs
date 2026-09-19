use super::*;
struct Plan {
    item: String,
    package: String,
    quantity: i64,
    refund: i64,
    allocations: Vec<(String, String, i64)>,
}
fn plan(c: &Connection, sale: &str, lines: &[ReturnLine]) -> Result<Vec<Plan>, AppError> {
    let (status, _, cart) = stored(c, sale)?;
    if status != "POSTED" {
        return Err(invalid("Only posted, non-voided sales can be returned."));
    }
    branch(c, &cart.branch_id)?;
    if lines.is_empty() || lines.len() > 200 {
        return Err(invalid("Select 1–200 return lines."));
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for l in lines {
        id(&l.sale_item_id)?;
        inventory::quantity(l.quantity)?;
        if !seen.insert(&l.sale_item_id) {
            return Err(invalid("Return items must be unique."));
        }
        let(package,sold,total):(String,i64,i64)=c.query_row("SELECT product_package_id,quantity,line_total_minor FROM sale_items WHERE id=?1 AND sale_id=?2",params![l.sale_item_id,sale],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let prior:i64=c.query_row("SELECT coalesce(sum(i.quantity),0) FROM sale_return_items i JOIN sale_returns r ON r.id=i.return_id WHERE i.sale_item_id=?1 AND r.status='POSTED'",[&l.sale_item_id],|r|r.get(0))?;
        if l.quantity > sold - prior {
            return Err(invalid(
                "Return quantity exceeds the remaining sold quantity.",
            ));
        }
        // Successive portions sum exactly to the actual paid line total, including sale discount/tax.
        let refund = ((prior as i128 + l.quantity as i128) * total as i128 / sold as i128
            - prior as i128 * total as i128 / sold as i128) as i64;
        let mut s=c.prepare("SELECT a.id,a.batch_id,a.quantity-coalesce((SELECT sum(ra.quantity) FROM sale_return_allocations ra JOIN sale_return_items ri ON ri.id=ra.return_item_id JOIN sale_returns r ON r.id=ri.return_id WHERE ra.original_allocation_id=a.id AND r.status='POSTED'),0) FROM sale_item_allocations a WHERE a.sale_item_id=?1 ORDER BY a.position")?;
        let rows = s
            .query_map([&l.sale_item_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut remaining = l.quantity;
        let mut allocations = Vec::new();
        for (original, batch, available) in rows {
            if available <= 0 {
                continue;
            }
            let take = remaining.min(available);
            let b = inventory::batches::get(c, &cart.branch_id, &batch)?;
            inventory::scope(c, &cart.branch_id, &package, true)?;
            if !b.is_active {
                return Err(invalid("Original return batch is inactive. Exact restoration requires explicit inventory review."));
            }
            allocations.push((original, batch, take));
            remaining -= take;
            if remaining == 0 {
                break;
            }
        }
        if remaining != 0 {
            return Err(invalid(
                "Original sale allocations do not cover this return.",
            ));
        }
        out.push(Plan {
            item: l.sale_item_id.clone(),
            package,
            quantity: l.quantity,
            refund,
            allocations,
        });
    }
    Ok(out)
}
pub fn preview_return(db: &AppDb, input: &ReturnDraft) -> Result<Value, AppError> {
    let c = db.lock()?;
    required(&input.reason)?;
    let p = plan(&c, &input.sale_id, &input.lines)?;
    Ok(
        json!({"refundMinor":money(p.iter().map(|l|l.refund as i128).sum())?,"lines":p.iter().map(|l|json!({"saleItemId":l.item,"quantity":l.quantity,"refundMinor":l.refund,"allocations":l.allocations.iter().map(|(a,b,q)|json!({"originalAllocationId":a,"batchId":b,"quantity":q})).collect::<Vec<_>>()})).collect::<Vec<_>>()}),
    )
}
pub fn create_return(db: &AppDb, input: &ReturnDraft) -> Result<Value, AppError> {
    id(&input.id)?;
    required(&input.reason)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let existing: Option<(String, String, String)> = tx
        .query_row(
            "SELECT sale_id,reason,lines_json FROM sale_returns WHERE id=?1",
            [&input.id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some(old) = existing {
        if old
            != (
                input.sale_id.clone(),
                input.reason.clone(),
                text(&input.lines)?,
            )
        {
            return Err(invalid("Return ID already belongs to another payload."));
        }
        return return_detail(&tx, &input.id);
    }
    let p = plan(&tx, &input.sale_id, &input.lines)?;
    let refund = money(p.iter().map(|l| l.refund as i128).sum())?;
    tx.execute("INSERT INTO sale_returns(id,sale_id,status,reason,lines_json,refund_minor) VALUES(?1,?2,'DRAFT',?3,?4,?5)",params![input.id,input.sale_id,input.reason,text(&input.lines)?,refund])?;
    let result = return_detail(&tx, &input.id)?;
    tx.commit()?;
    Ok(result)
}
fn return_detail(c: &Connection, id: &str) -> Result<Value, AppError> {
    Ok(c.query_row("SELECT json_object('id',id,'saleId',sale_id,'status',status,'reason',reason,'lines',json(lines_json),'refundMinor',refund_minor) FROM sale_returns WHERE id=?1",[id],jsonrow)?)
}
fn restore(
    c: &Connection,
    branch: &str,
    package: &str,
    batch: &str,
    quantity: i64,
    reason: &str,
    reference: (&str, &str),
) -> Result<String, AppError> {
    let op = StockOperation {
        request_id: uuid(),
        branch_id: branch.into(),
        product_package_id: package.into(),
        batch_id: batch.into(),
        quantity,
        reason: reason.into(),
        created_by: None,
    };
    Ok(movements::insert_referenced(c, &op, MovementType::SaleReturn, None, Some(reference))?.id)
}
pub fn post_return(db: &AppDb, input: &ReturnPost) -> Result<Value, AppError> {
    id(&input.id)?;
    let payload = text(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (sale, status, reason, lines, expected): (String, String, String, String, i64) = tx
        .query_row(
            "SELECT sale_id,status,reason,lines_json,refund_minor FROM sale_returns WHERE id=?1",
            [&input.id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;
    if retry(&tx, &input.request_id, "RETURN", &input.id, &payload)? {
        return detail_on(&tx, &sale);
    }
    if status != "DRAFT" {
        return Err(invalid("Return is already posted."));
    }
    let (_, _, cart) = stored(&tx, &sale)?;
    let p = plan(&tx, &sale, &decode::<Vec<ReturnLine>>(&lines)?)?;
    let refund = money(p.iter().map(|l| l.refund as i128).sum())?;
    if refund != expected {
        return Err(invalid(
            "Refund changed after another return. Create a fresh return preview.",
        ));
    }
    for l in p {
        let item = uuid();
        tx.execute("INSERT INTO sale_return_items(id,return_id,sale_item_id,quantity,refund_minor) VALUES(?1,?2,?3,?4,?5)",params![item,input.id,l.item,l.quantity,l.refund])?;
        for (original, batch, quantity) in l.allocations {
            let allocation = uuid();
            let m = restore(
                &tx,
                &cart.branch_id,
                &l.package,
                &batch,
                quantity,
                &reason,
                ("sale_return_allocation", &allocation),
            )?;
            tx.execute("INSERT INTO sale_return_allocations(id,return_item_id,original_allocation_id,quantity,movement_id) VALUES(?1,?2,?3,?4,?5)",params![allocation,item,original,quantity,m])?;
        }
    }
    let total: i64 = tx.query_row("SELECT total_minor FROM sales WHERE id=?1", [&sale], |r| {
        r.get(0)
    })?;
    let prior: i64 = tx.query_row(
        "SELECT coalesce(sum(amount_minor),0) FROM sale_refunds WHERE sale_id=?1",
        [&sale],
        |r| r.get(0),
    )?;
    if refund > total - prior {
        return Err(invalid("Refund would exceed the amount paid."));
    }
    tx.execute(
        "INSERT INTO sale_refunds(id,sale_id,return_id,amount_minor) VALUES(?1,?2,?3,?4)",
        params![uuid(), sale, input.id, refund],
    )?;
    tx.execute("UPDATE sale_returns SET status='POSTED',posted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[&input.id])?;
    receipt(&tx, &input.request_id, "RETURN", &input.id, &payload)?;
    let result = detail_on(&tx, &sale)?;
    tx.commit()?;
    drop(c);
    inventory::alerts::refresh_all(db);
    Ok(result)
}
pub fn void(db: &AppDb, input: &Void) -> Result<Value, AppError> {
    required(&input.reason)?;
    let payload = text(input)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if retry(&tx, &input.request_id, "VOID", &input.id, &payload)? {
        return detail_on(&tx, &input.id);
    }
    let (status, _, cart) = stored(&tx, &input.id)?;
    if status != "POSTED" {
        return Err(invalid("Only a posted sale can be voided."));
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sale_returns WHERE sale_id=?1 AND status='POSTED')",
        [&input.id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("A sale with posted returns cannot be voided."));
    }
    let total: i64 = tx.query_row(
        "SELECT total_minor FROM sales WHERE id=?1",
        [&input.id],
        |r| r.get(0),
    )?;
    let void = uuid();
    tx.execute(
        "INSERT INTO sale_voids(id,sale_id,reason,refund_minor) VALUES(?1,?2,?3,?4)",
        params![void, input.id, input.reason, total],
    )?;
    let rows = {
        let mut s=tx.prepare("SELECT a.id,a.batch_id,a.quantity,i.product_package_id FROM sale_item_allocations a JOIN sale_items i ON i.id=a.sale_item_id WHERE i.sale_id=?1 ORDER BY i.position,a.position")?;
        let rows = s
            .query_map([&input.id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for (original, batch, quantity, package) in rows {
        let allocation = uuid();
        let m = restore(
            &tx,
            &cart.branch_id,
            &package,
            &batch,
            quantity,
            &input.reason,
            ("sale_void_allocation", &allocation),
        )?;
        tx.execute("INSERT INTO sale_void_allocations(id,void_id,original_allocation_id,movement_id) VALUES(?1,?2,?3,?4)",params![allocation,void,original,m])?;
    }
    tx.execute(
        "INSERT INTO sale_refunds(id,sale_id,void_id,amount_minor) VALUES(?1,?2,?3,?4)",
        params![uuid(), input.id, void, total],
    )?;
    tx.execute("UPDATE sales SET status='VOIDED',payment_status='REFUNDED',voided_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),revision=revision+1 WHERE id=?1",[&input.id])?;
    receipt(&tx, &input.request_id, "VOID", &input.id, &payload)?;
    let result = detail_on(&tx, &input.id)?;
    tx.commit()?;
    drop(c);
    inventory::alerts::refresh_all(db);
    Ok(result)
}
