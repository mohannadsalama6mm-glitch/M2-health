pub mod commands;
pub mod models;
mod repository;
use crate::{
    db::{
        catalog_validation::{id, invalid, optional, required, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory::{
        self, batches,
        models::{CreateBatch, MovementType, StockOperation, TransferStock},
        movements, transfers,
    },
};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn conflict(message: &str) -> AppError {
    AppError::new(ErrorCode::Conflict, message)
}
fn revision(h: &Header, expected: i64) -> Result<(), AppError> {
    if h.revision != expected {
        Err(conflict("Document changed. Reload it before continuing."))
    } else {
        Ok(())
    }
}
fn branch(c: &Connection, value: &str) -> Result<(), AppError> {
    id(value)?;
    if !c.query_row(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1 AND is_active=1)",
        [value],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("Select an active branch."));
    }
    Ok(())
}
fn validate_line(
    c: &Connection,
    kind: Kind,
    branch: &str,
    l: &LineInput,
    posting: bool,
) -> Result<(), AppError> {
    inventory::scope(c, branch, &l.product_package_id, true)?;
    inventory::quantity(l.quantity)?;
    required(&l.reason)?;
    optional(&l.lot_number)?;
    optional(&l.supplier_reference)?;
    if let Some(date) = &l.expiry_date {
        inventory::date(c, date)?;
    }
    if l.cost_price_minor
        .is_some_and(|v| !(0..=MAX_SAFE_MINOR).contains(&v))
    {
        return Err(invalid("Cost must be non-negative integer minor units."));
    }
    if !matches!(
        (kind, l.action),
        (Kind::Receipt, Action::Receive)
            | (Kind::Adjustment, Action::In | Action::Out)
            | (Kind::WriteOff, Action::Damage | Action::Expiry)
            | (Kind::Transfer, Action::Transfer)
    ) {
        return Err(invalid("Line action does not match the document type."));
    }
    if let Some(batch) = &l.batch_id {
        let b = batches::get(c, branch, batch)?;
        if !b.is_active || b.product_package_id != l.product_package_id {
            return Err(invalid(
                "Select an active batch belonging to this package and branch.",
            ));
        }
        if kind == Kind::Receipt
            && ((l.lot_number.is_some() && l.lot_number != b.lot_number)
                || (l.expiry_date.is_some() && l.expiry_date != b.expiry_date)
                || (l.cost_price_minor.is_some() && l.cost_price_minor != b.cost_price_minor)
                || (l.supplier_reference.is_some() && l.supplier_reference != b.supplier_reference))
        {
            return Err(invalid("Existing batch identity and cost cannot be changed. Receive into a new batch instead."));
        }
        if posting && l.action == Action::Expiry {
            let expired =
                c.query_row("SELECT coalesce(?1<date('now'),0)", [b.expiry_date], |r| {
                    r.get::<_, bool>(0)
                })?;
            if !expired {
                return Err(invalid(
                    "Expiry write-off requires a known expiry date that has already passed.",
                ));
            }
        }
    } else if kind != Kind::Receipt {
        return Err(invalid("A batch is required for this operation."));
    }
    if kind != Kind::Receipt
        && (l.lot_number.is_some()
            || l.expiry_date.is_some()
            || l.cost_price_minor.is_some()
            || l.supplier_reference.is_some())
    {
        return Err(invalid(
            "Batch identity fields can only be supplied on a receipt.",
        ));
    }
    Ok(())
}
fn validate_ready(c: &Connection, d: &Document) -> Result<(), AppError> {
    if d.lines.is_empty() {
        return Err(invalid("Add at least one line before marking ready."));
    }
    branch(c, &d.header.branch_id)?;
    if let Some(dest) = &d.header.destination_branch_id {
        branch(c, dest)?;
    }
    for l in &d.lines {
        validate_line(c, d.header.kind, &d.header.branch_id, &l.input, true)?;
    }
    if d.preview
        .iter()
        .any(|p| p.projected_stock < 0 || p.projected_stock > MAX_SAFE_MINOR)
    {
        return Err(invalid(
            "Insufficient stock or projected balance exceeds the supported range.",
        ));
    }
    Ok(())
}
pub fn save(db: &AppDb, input: &SaveDocument) -> Result<Document, AppError> {
    if input.kind == Kind::Reversal {
        return Err(invalid("Use the reversal operation for a posted document."));
    }
    if input.lines.len() > 200 {
        return Err(invalid("A document supports at most 200 lines."));
    }
    for text in [
        &input.reference_text,
        &input.supplier_reference,
        &input.notes,
        &input.created_by,
    ] {
        optional(text)?;
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    branch(&tx, &input.branch_id)?;
    if (input.kind == Kind::Transfer) != input.destination_branch_id.is_some() {
        return Err(invalid("Only transfers require a destination branch."));
    }
    if let Some(dest) = &input.destination_branch_id {
        branch(&tx, dest)?;
        if dest == &input.branch_id {
            return Err(invalid("Transfer requires different branches."));
        }
    }
    for l in &input.lines {
        validate_line(&tx, input.kind, &input.branch_id, l, false)?;
    }
    let doc = if let Some(doc) = &input.document_id {
        let h = repository::header(&tx, doc)?;
        revision(
            &h,
            input
                .expected_revision
                .ok_or_else(|| invalid("Document revision is required."))?,
        )?;
        if h.status != Status::Draft {
            return Err(conflict("Only draft documents can be edited."));
        }
        if h.kind != input.kind || h.branch_id != input.branch_id {
            return Err(invalid("Document type and source branch cannot change."));
        }
        tx.execute("UPDATE inventory_documents SET destination_branch_id=?2,reference_text=?3,supplier_reference=?4,notes=?5,created_by=?6,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![doc,input.destination_branch_id,input.reference_text,input.supplier_reference,input.notes,input.created_by])?;
        tx.execute(
            "DELETE FROM inventory_document_lines WHERE document_id=?1",
            [doc],
        )?;
        doc.clone()
    } else {
        let doc = uuid();
        let number = repository::number(&tx, input.kind)?;
        tx.execute("INSERT INTO inventory_documents(id,document_number,kind,branch_id,destination_branch_id,reference_text,supplier_reference,notes,created_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![doc,number,input.kind.text(),input.branch_id,input.destination_branch_id,input.reference_text,input.supplier_reference,input.notes,input.created_by])?;
        doc
    };
    for (position, l) in input.lines.iter().enumerate() {
        repository::insert_line(&tx, &doc, position, l)?;
    }
    let result = repository::detail(&tx, &doc)?;
    tx.commit()?;
    Ok(result)
}
pub fn get(db: &AppDb, doc: &str) -> Result<Document, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let result = repository::detail(&tx, doc)?;
    tx.commit()?;
    Ok(result)
}
pub fn list(db: &AppDb, q: &DocumentQuery) -> Result<DocumentPage, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let result = repository::list(&tx, q)?;
    tx.commit()?;
    Ok(result)
}
pub fn ready(db: &AppDb, input: &Transition) -> Result<Document, AppError> {
    transition(db, input, false)
}
pub fn cancel(db: &AppDb, input: &Transition) -> Result<Document, AppError> {
    transition(db, input, true)
}
fn transition(db: &AppDb, input: &Transition, cancel: bool) -> Result<Document, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let d = repository::detail(&tx, &input.document_id)?;
    revision(&d.header, input.expected_revision)?;
    if cancel {
        if !matches!(d.header.status, Status::Draft | Status::Ready) {
            return Err(conflict("Only draft or ready documents can be cancelled. Posted documents require reversal."));
        }
    } else {
        if d.header.status != Status::Draft {
            return Err(conflict("Only a draft can be marked ready."));
        }
        validate_ready(&tx, &d)?;
    }
    set_status(
        &tx,
        &input.document_id,
        if cancel { "CANCELLED" } else { "READY" },
    )?;
    let result = repository::detail(&tx, &input.document_id)?;
    tx.commit()?;
    Ok(result)
}
fn set_status(c: &Connection, doc: &str, status: &str) -> Result<(), AppError> {
    c.execute("UPDATE inventory_documents SET status=?2,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),posted_at=CASE WHEN ?2='POSTED' THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END WHERE id=?1",params![doc,status])?;
    Ok(())
}
fn request(c: &Connection, key: &str, payload: &str) -> Result<Option<Document>, AppError> {
    id(key)?;
    let existing: Option<(String, String)> = c
        .query_row(
            "SELECT payload,result_document_id FROM inventory_post_requests WHERE request_id=?1",
            [key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match existing {
        Some((old, doc)) => {
            if old != payload {
                return Err(conflict(
                    "Post request ID was already used for different input.",
                ));
            }
            Ok(Some(repository::detail(c, &doc)?))
        }
        None => Ok(None),
    }
}
fn record_request(c: &Connection, key: &str, payload: &str, doc: &str) -> Result<(), AppError> {
    c.execute("INSERT INTO inventory_post_requests(request_id,payload,result_document_id) VALUES(?1,?2,?3)",params![key,payload,doc])?;
    Ok(())
}
fn link(
    c: &Connection,
    doc: &str,
    line: &str,
    movement: &str,
    reverses: Option<&str>,
) -> Result<(), AppError> {
    c.execute("INSERT INTO inventory_document_movements(document_id,line_id,movement_id,reverses_movement_id) VALUES(?1,?2,?3,?4)",params![doc,line,movement,reverses])?;
    Ok(())
}
pub fn post(db: &AppDb, input: &PostDocument) -> Result<Document, AppError> {
    post_as(db, input, None)
}
pub fn post_as(
    db: &AppDb,
    input: &PostDocument,
    actor: Option<&str>,
) -> Result<Document, AppError> {
    let payload = format!(
        "post:{}:{}",
        serde_json::to_string(input).map_err(|_| invalid("Invalid post request."))?,
        actor.unwrap_or("")
    );
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) = request(&tx, &input.request_id, &payload)? {
        return Ok(result);
    }
    let d = repository::detail(&tx, &input.document_id)?;
    if d.header.status != Status::Ready {
        return Err(conflict("Only ready documents can be posted. This document is already posted, cancelled, or still a draft."));
    }
    revision(&d.header, input.expected_revision)?;
    validate_ready(&tx, &d)?;
    for line in &d.lines {
        let l = &line.input;
        if d.header.kind == Kind::Transfer {
            let result = transfers::transfer_on(
                &tx,
                &TransferStock {
                    request_id: uuid(),
                    source_branch_id: d.header.branch_id.clone(),
                    destination_branch_id: d
                        .header
                        .destination_branch_id
                        .clone()
                        .ok_or_else(|| invalid("Destination branch required."))?,
                    product_package_id: l.product_package_id.clone(),
                    source_batch_id: l
                        .batch_id
                        .clone()
                        .ok_or_else(|| invalid("Batch required."))?,
                    quantity: l.quantity,
                    reason: l.reason.clone(),
                    created_by: actor.map(str::to_owned),
                },
                Some(&d.header.id),
            )?;
            for m in [&result.out_movement, &result.in_movement] {
                link(&tx, &d.header.id, &line.id, &m.id, None)?;
            }
        } else {
            let batch = match &l.batch_id {
                Some(b) => b.clone(),
                None => {
                    batches::create_on(
                        &tx,
                        &CreateBatch {
                            branch_id: d.header.branch_id.clone(),
                            product_package_id: l.product_package_id.clone(),
                            lot_number: l.lot_number.clone(),
                            expiry_date: l.expiry_date.clone(),
                            cost_price_minor: l.cost_price_minor,
                            supplier_reference: l
                                .supplier_reference
                                .clone()
                                .or(d.header.supplier_reference.clone()),
                            received_at: Some(tx.query_row(
                                "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                                [],
                                |r| r.get(0),
                            )?),
                        },
                        actor,
                    )?
                    .id
                }
            };
            let kind = match l.action {
                Action::Receive => MovementType::PurchaseReceipt,
                Action::In => MovementType::AdjustmentIn,
                Action::Out => MovementType::AdjustmentOut,
                Action::Damage => MovementType::Damage,
                Action::Expiry => MovementType::Expiry,
                Action::Transfer => return Err(invalid("Invalid transfer line.")),
            };
            let m = movements::insert_referenced(
                &tx,
                &StockOperation {
                    request_id: uuid(),
                    branch_id: d.header.branch_id.clone(),
                    product_package_id: l.product_package_id.clone(),
                    batch_id: batch,
                    quantity: l.quantity,
                    reason: l.reason.clone(),
                    created_by: actor.map(str::to_owned),
                },
                kind,
                None,
                Some(("inventory_document", &d.header.id)),
            )?;
            link(&tx, &d.header.id, &line.id, &m.id, None)?;
        }
    }
    set_status(&tx, &d.header.id, "POSTED")?;
    record_request(&tx, &input.request_id, &payload, &d.header.id)?;
    let result = repository::detail(&tx, &d.header.id)?;
    tx.commit()?;
    Ok(result)
}
pub fn reverse(db: &AppDb, input: &ReverseDocument) -> Result<Document, AppError> {
    reverse_as(db, input, None)
}
pub fn reverse_as(
    db: &AppDb,
    input: &ReverseDocument,
    actor: Option<&str>,
) -> Result<Document, AppError> {
    required(&input.reason)?;
    let payload = format!(
        "reverse:{}:{}",
        serde_json::to_string(input).map_err(|_| invalid("Invalid reversal request."))?,
        actor.unwrap_or("")
    );
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) = request(&tx, &input.request_id, &payload)? {
        return Ok(result);
    }
    let original = repository::detail(&tx, &input.document_id)?;
    if original.header.status != Status::Posted || original.header.kind == Kind::Reversal {
        return Err(conflict(
            "Only an original posted document can be reversed.",
        ));
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM inventory_documents WHERE reversal_of=?1)",
        [&input.document_id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(conflict("This document already has a reversal."));
    }
    let doc = uuid();
    let number = repository::number(&tx, Kind::Reversal)?;
    tx.execute("INSERT INTO inventory_documents(id,document_number,kind,branch_id,reversal_of,reversal_reason,created_by) VALUES(?1,?2,'REVERSAL',?3,?4,?5,?6)",params![doc,number,original.header.branch_id,input.document_id,input.reason.trim(),actor])?;
    let mut work = Vec::new();
    for (pos, m) in original.movements.iter().enumerate() {
        let line = LineInput {
            product_package_id: m.product_package_id.clone(),
            batch_id: Some(m.batch_id.clone()),
            action: if m.quantity_delta > 0 {
                Action::Out
            } else {
                Action::In
            },
            quantity: m.quantity_delta.abs(),
            reason: input.reason.clone(),
            lot_number: None,
            expiry_date: None,
            cost_price_minor: None,
            supplier_reference: None,
        };
        let line_id = repository::insert_line(&tx, &doc, pos, &line)?;
        work.push((line_id, m, line));
    }
    set_status(&tx, &doc, "READY")?;
    for (line_id, original, line) in work {
        let m = movements::insert_referenced(
            &tx,
            &StockOperation {
                request_id: uuid(),
                branch_id: original.branch_id.clone(),
                product_package_id: line.product_package_id,
                batch_id: original.batch_id.clone(),
                quantity: line.quantity,
                reason: line.reason,
                created_by: actor.map(str::to_owned),
            },
            if line.action == Action::Out {
                MovementType::AdjustmentOut
            } else {
                MovementType::AdjustmentIn
            },
            None,
            Some(("inventory_document", &doc)),
        )?;
        link(&tx, &doc, &line_id, &m.id, Some(&original.movement_id))?;
    }
    set_status(&tx, &doc, "POSTED")?;
    record_request(&tx, &input.request_id, &payload, &doc)?;
    let result = repository::detail(&tx, &doc)?;
    tx.commit()?;
    Ok(result)
}
pub fn batch_choices(
    db: &AppDb,
    branch: &str,
    package: &str,
    offset: i64,
) -> Result<Vec<BatchChoice>, AppError> {
    let (_, offset) = inventory::page(Some(50), Some(offset))?;
    let c = db.lock()?;
    inventory::scope(&c, branch, package, false)?;
    let rows=c.prepare("SELECT json_object('id',b.id,'lotNumber',b.lot_number,'expiryDate',b.expiry_date,'costPriceMinor',b.cost_price_minor,'quantity',coalesce((SELECT sum(quantity_delta) FROM inventory_movements WHERE batch_id=b.id),0)) FROM inventory_batches b WHERE b.branch_id=?1 AND b.product_package_id=?2 AND b.is_active=1 ORDER BY b.expiry_date IS NULL,b.expiry_date,b.id LIMIT 50 OFFSET ?3")?.query_map(params![branch,package,offset],repository::decode)?.collect::<Result<Vec<BatchChoice>,_>>()?;
    Ok(rows)
}
