pub mod commands;
pub mod models;
mod repository;
use crate::{
    db::{
        catalog_validation::{id, invalid, optional, required, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory::{self, batches, models::CreateBatch},
};
use models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn conflict(message: &str) -> AppError {
    AppError::new(ErrorCode::Conflict, message)
}
fn active_branch(c: &Connection, branch: &str) -> Result<(), AppError> {
    id(branch)?;
    if !c.query_row(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1 AND is_active=1)",
        [branch],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(invalid("A valid active branch is required."));
    }
    Ok(())
}
fn revision(s: &Session, expected: i64) -> Result<(), AppError> {
    if s.status == Status::Completed {
        return Err(conflict("Session already completed and is read-only."));
    }
    if s.status == Status::Cancelled {
        return Err(conflict("Session cancelled and cannot be reopened."));
    }
    if s.revision != expected {
        return Err(conflict("Stale session revision. Reload before retrying."));
    }
    Ok(())
}
pub fn create(db: &AppDb, i: &Create) -> Result<Detail, AppError> {
    optional(&i.notes)?;
    optional(&i.created_by)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    active_branch(&tx, &i.branch_id)?;
    let day: String = tx.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    let sequence:i64=tx.query_row("INSERT INTO stock_count_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1 RETURNING next_number",[&day],|r|r.get(0))?;
    let id = uuid();
    tx.execute("INSERT INTO stock_count_sessions(id,session_number,branch_id,notes,created_by) VALUES(?1,?2,?3,?4,?5)",params![id,format!("CNT-{day}-{sequence:04}"),i.branch_id,i.notes,i.created_by])?;
    let d = repository::detail(&tx, &id, 50, 0)?;
    tx.commit()?;
    Ok(d)
}
pub fn get(
    db: &AppDb,
    id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Detail, AppError> {
    let (l, o) = inventory::page(limit, offset)?;
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let d = repository::detail(&tx, id, l, o)?;
    tx.commit()?;
    Ok(d)
}
pub fn list(db: &AppDb, q: &Query) -> Result<Page, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction()?;
    let d = repository::list(&tx, q)?;
    tx.commit()?;
    Ok(d)
}
pub fn start(db: &AppDb, i: &Transition) -> Result<Detail, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let s = repository::session(&tx, &i.session_id)?;
    revision(&s, i.expected_revision)?;
    if s.status != Status::Draft {
        return Err(conflict("Only a draft count can start."));
    }
    active_branch(&tx, &s.branch_id)?;
    tx.execute("UPDATE stock_count_sessions SET status='IN_PROGRESS',revision=revision+1,started_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),snapshot_watermark=(SELECT coalesce(max(rowid),0) FROM inventory_movements) WHERE id=?1",[&s.id])?;
    let s = repository::session(&tx, &s.id)?;
    let batch_ids=tx.prepare("SELECT b.id FROM inventory_batches b JOIN product_packages pk ON pk.id=b.product_package_id JOIN products p ON p.id=pk.product_id WHERE b.branch_id=?1 AND b.is_active=1 AND pk.is_active=1 AND p.is_active=1 ORDER BY b.id")?.query_map([&s.branch_id],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
    for batch in batch_ids {
        repository::insert_line(&tx, &s, &batch, None)?;
    }
    let d = repository::detail(&tx, &s.id, 50, 0)?;
    tx.commit()?;
    Ok(d)
}
pub fn record(db: &AppDb, i: &Entry) -> Result<Detail, AppError> {
    if i.counted_quantity
        .is_some_and(|q| !(0..=MAX_SAFE_MINOR).contains(&q))
    {
        return Err(invalid(
            "Physical count must be zero or a positive whole package quantity.",
        ));
    }
    optional(&i.notes)?;
    if let Some(reason) = &i.excluded_reason {
        required(reason)?;
        if i.counted_quantity.is_some() {
            return Err(invalid("Excluded lines must remain uncounted."));
        }
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let s = repository::session(&tx, &i.session_id)?;
    revision(&s, i.expected_revision)?;
    if s.status != Status::InProgress {
        return Err(conflict("Return to count entry before editing lines."));
    }
    let changed=tx.execute("UPDATE stock_count_lines SET counted_quantity=?1,notes=?2,excluded_reason=?3,counted_at=CASE WHEN ?1 IS NULL THEN NULL ELSE strftime('%Y-%m-%dT%H:%M:%fZ','now') END,revision=revision+1 WHERE id=?4 AND session_id=?5 AND revision=?6",params![i.counted_quantity,i.notes,i.excluded_reason,i.line_id,s.id,i.line_revision])?;
    if changed != 1 {
        return Err(conflict(
            "Stale line revision or invalid count line. Reload before retrying.",
        ));
    }
    tx.execute(
        "UPDATE stock_count_sessions SET revision=revision+1 WHERE id=?1",
        [&s.id],
    )?;
    let d = repository::detail(&tx, &s.id, 50, 0)?;
    tx.commit()?;
    Ok(d)
}
pub fn discover(db: &AppDb, i: &Discover) -> Result<Detail, AppError> {
    required(&i.notes)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let s = repository::session(&tx, &i.session_id)?;
    revision(&s, i.expected_revision)?;
    if s.status != Status::InProgress {
        return Err(conflict(
            "Discovered stock can only be added during count entry.",
        ));
    }
    inventory::scope(&tx, &s.branch_id, &i.product_package_id, true)?;
    let batch = if let Some(batch) = &i.batch_id {
        if i.lot_number.is_some() || i.expiry_date.is_some() {
            return Err(invalid("Existing batch metadata cannot be changed."));
        }
        let b = batches::get(&tx, &s.branch_id, batch)?;
        if b.product_package_id != i.product_package_id || !b.is_active {
            return Err(invalid("Invalid batch or wrong branch/package."));
        }
        b.id
    } else {
        batches::create_on(
            &tx,
            &CreateBatch {
                branch_id: s.branch_id.clone(),
                product_package_id: i.product_package_id.clone(),
                lot_number: i.lot_number.clone(),
                expiry_date: i.expiry_date.clone(),
                ..Default::default()
            },
            s.created_by.as_deref(),
        )?
        .id
    };
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM stock_count_lines WHERE session_id=?1 AND batch_id=?2)",
        params![s.id, batch],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(conflict("This batch is already in the count."));
    }
    repository::insert_line(&tx, &s, &batch, Some(&i.notes))?;
    tx.execute(
        "UPDATE stock_count_sessions SET revision=revision+1 WHERE id=?1",
        [&s.id],
    )?;
    let d = repository::detail(&tx, &s.id, 50, 0)?;
    tx.commit()?;
    Ok(d)
}
pub fn transition(db: &AppDb, i: &Transition, to: Status) -> Result<Detail, AppError> {
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let s = repository::session(&tx, &i.session_id)?;
    revision(&s, i.expected_revision)?;
    let target = match (&s.status, &to) {
        (Status::InProgress, Status::Review) => "REVIEW",
        (Status::Review, Status::InProgress) => "IN_PROGRESS",
        (_, Status::Cancelled) => "CANCELLED",
        _ => return Err(conflict("Invalid stock count transition.")),
    };
    tx.execute("UPDATE stock_count_sessions SET status=?1,revision=revision+1,reviewed_at=CASE WHEN ?1='REVIEW' THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE reviewed_at END,cancelled_at=CASE WHEN ?1='CANCELLED' THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END WHERE id=?2",params![target,s.id])?;
    let d = repository::detail(&tx, &s.id, 50, 0)?;
    tx.commit()?;
    Ok(d)
}
pub fn complete(db: &AppDb, i: &Complete) -> Result<Detail, AppError> {
    complete_as(db, i, None)
}
pub fn complete_as(db: &AppDb, i: &Complete, actor: Option<&str>) -> Result<Detail, AppError> {
    id(&i.request_id)?;
    let payload =
        serde_json::to_string(&(i, actor)).map_err(|_| invalid("Invalid completion request."))?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some((previous, session)) = tx
        .query_row(
            "SELECT payload,session_id FROM stock_count_completion_requests WHERE request_id=?1",
            [&i.request_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if previous != payload {
            return Err(conflict(
                "Duplicate completion request key has different content.",
            ));
        }
        return repository::detail(&tx, &session, 50, 0);
    }
    let s = repository::session(&tx, &i.session_id)?;
    revision(&s, i.expected_revision)?;
    if s.status != Status::Review {
        return Err(conflict("Review the count before completing it."));
    }
    active_branch(&tx, &s.branch_id)?;
    if repository::summary(&tx, &s.id)?.uncounted > 0 {
        return Err(invalid(
            "Uncounted lines remain. Count them or explicitly exclude each with a reason.",
        ));
    }
    if repository::token(&tx, &s)? != i.review_token {
        return Err(conflict("Stock changed during count review. Refresh review and confirm the updated corrections."));
    }
    let mut lines = repository::lines(&tx, &s.id, -1, 0)?;
    lines.sort_by_key(|l| l.correction_delta.unwrap_or(0));
    for l in lines {
        let delta = l.correction_delta.unwrap_or(0);
        let movement = if delta == 0 {
            None
        } else {
            Some(inventory::movements::count_correction(
                &tx,
                &s.branch_id,
                &l.product_package_id,
                &l.batch_id,
                delta,
                &l.id,
                &s.session_number,
                actor,
            )?)
        };
        tx.execute("INSERT INTO stock_count_results(line_id,live_baseline,correction_delta,intervening_movements,movement_id) VALUES(?1,?2,?3,?4,?5)",params![l.id,l.current_live,delta,l.intervening_movements,movement])?;
    }
    tx.execute("UPDATE stock_count_sessions SET status='COMPLETED',revision=revision+1,completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[&s.id])?;
    tx.execute("INSERT INTO stock_count_completion_requests(request_id,payload,session_id) VALUES(?1,?2,?3)",params![i.request_id,payload,s.id])?;
    let d = repository::detail(&tx, &s.id, 50, 0)?;
    tx.commit()?;
    Ok(d)
}
