use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::inventory::*,
        repositories::{batches, movements, product_packages},
    },
    error::{AppError, ErrorCode},
};
use rusqlite::TransactionBehavior;
/// Every inventory write is a durable ledger movement inside one IMMEDIATE
/// transaction. Balances are derived from movements; nothing is rewritten in place.

fn validate_reason_user(reason: &str, user: &str) -> Result<(), AppError> {
    if reason.trim().is_empty() {
        return Err(invalid(
            "A reason is required for this inventory operation.",
        ));
    }
    if reason.len() > 500 {
        return Err(invalid("The reason is too long."));
    }
    if user.len() > 160 {
        return Err(invalid("The operator name is too long."));
    }
    Ok(())
}
fn validate_quantity(quantity: i64) -> Result<(), AppError> {
    if quantity <= 0 {
        return Err(invalid("Quantity must be positive."));
    }
    Ok(())
}
/// Opening stock: ensures the branch-scoped lot exists and posts the first ledger
/// entry. Reusing the same lot appends further opening movements rather than
/// overwriting history.
pub fn opening_stock(db: &AppDb, input: &OpeningStockInput) -> Result<StockMovement, AppError> {
    id(&input.branch_id)?;
    id(&input.product_package_id)?;
    validate_quantity(input.quantity)?;
    validate_reason_user(&input.reason, input.user.as_deref().unwrap_or(""))?;
    let batch_number = input.batch_number.trim();
    if batch_number.is_empty() || batch_number.len() > 64 {
        return Err(invalid(
            "A batch/lot number of 1–64 characters is required.",
        ));
    }
    batches::validate_expiry(&input.expiry_date)?;
    if input
        .cost_price_minor
        .is_some_and(|v| !(0..=MAX_SAFE_MINOR).contains(&v))
    {
        return Err(invalid(
            "Cost must be non-negative integer minor units within the supported range.",
        ));
    }
    let user = input.user.as_deref().unwrap_or("");
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::db::repositories::branches::get(&tx, &input.branch_id)?;
    product_packages::get(&tx, &input.product_package_id)?;
    let batch = batches::ensure_on(
        &tx,
        &input.branch_id,
        &input.product_package_id,
        batch_number,
        &input.expiry_date,
        input.cost_price_minor,
    )?;
    let movement = movements::insert_on(
        &tx,
        &input.branch_id,
        &input.product_package_id,
        &batch.id,
        "opening",
        input.quantity,
        input.cost_price_minor,
        None,
        None,
        &input.reason.trim(),
        user,
    )?;
    tx.commit()?;
    Ok(movement)
}
/// Controlled adjustment of a package's stock in a branch. Without a batch id the
/// decrease is applied FEFO (earliest expiry first) and the increase targets a
/// GENERAL lot (or the supplied batch number/expiry). A no-op returns no movements.
pub fn adjust_stock(db: &AppDb, input: &AdjustStockInput) -> Result<AdjustResult, AppError> {
    id(&input.branch_id)?;
    id(&input.product_package_id)?;
    if input.new_quantity < 0 {
        return Err(invalid("New stock quantity cannot be negative."));
    }
    validate_reason_user(&input.reason, input.user.as_deref().unwrap_or(""))?;
    if input.batch_id.is_some() {
        id(input.batch_id.as_deref().unwrap_or(""))?;
    }
    if let Some(expiry) = &input.expiry_date {
        batches::validate_expiry(expiry)?;
    }
    if let Some(number) = &input.batch_number {
        let t = number.trim();
        if t.is_empty() || t.len() > 64 {
            return Err(invalid("Batch number must be 1–64 characters."));
        }
    }
    let user = input.user.as_deref().unwrap_or("");
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::db::repositories::branches::get(&tx, &input.branch_id)?;
    product_packages::get(&tx, &input.product_package_id)?;
    let current = match &input.batch_id {
        Some(batch_id) => {
            let batch = batches::get(&tx, batch_id)?;
            if batch.branch_id != input.branch_id
                || batch.product_package_id != input.product_package_id
            {
                return Err(invalid(
                    "The batch does not belong to this branch and package.",
                ));
            }
            batches::balance_on(&tx, &batch.id)?
        }
        None => batches::package_balance_on(&tx, &input.branch_id, &input.product_package_id)?,
    };
    let delta = input.new_quantity - current;
    if delta == 0 {
        tx.commit()?;
        return Ok(AdjustResult {
            new_quantity: input.new_quantity,
            movements: Vec::new(),
        });
    }
    let reason = input.reason.trim();
    let mut movements_out = Vec::new();
    if delta > 0 {
        let batch = batch_for_increase(&tx, input)?;
        movements_out.push(movements::insert_on(
            &tx,
            &input.branch_id,
            &input.product_package_id,
            &batch.id,
            "adjustment",
            delta,
            batch.cost_price_minor,
            None,
            None,
            reason,
            user,
        )?);
    } else {
        let mut remaining = -delta;
        let mut applied = false;
        for (batch, qty) in
            batches::list_active_with_balance_on(&tx, &input.branch_id, &input.product_package_id)?
        {
            if qty <= 0 || remaining <= 0 {
                continue;
            }
            let take = remaining.min(qty);
            movements_out.push(movements::insert_on(
                &tx,
                &input.branch_id,
                &input.product_package_id,
                &batch.id,
                "adjustment",
                -take,
                batch.cost_price_minor,
                None,
                None,
                reason,
                user,
            )?);
            remaining -= take;
            applied = true;
        }
        if !applied || remaining > 0 {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "Insufficient stock to adjust to the requested quantity.",
            ));
        }
    }
    tx.commit()?;
    Ok(AdjustResult {
        new_quantity: input.new_quantity,
        movements: movements_out,
    })
}
fn batch_for_increase(
    tx: &rusqlite::Connection,
    input: &AdjustStockInput,
) -> Result<InventoryBatch, AppError> {
    if let Some(id) = &input.batch_id {
        return batches::get(tx, id);
    }
    let number = input.batch_number.as_deref().map(str::trim);
    let expiry = input.expiry_date.as_deref().unwrap_or("");
    match number {
        Some(n) if !n.is_empty() => batches::ensure_on(
            tx,
            &input.branch_id,
            &input.product_package_id,
            n,
            expiry,
            None,
        ),
        _ => batches::ensure_on(
            tx,
            &input.branch_id,
            &input.product_package_id,
            "GENERAL",
            "",
            None,
        ),
    }
}
/// Outbound write-off: damage, expired or supplier return. Quantity is removed from
/// the ledger, never deleted from history.
pub fn write_off(db: &AppDb, input: &WriteOffInput) -> Result<StockMovement, AppError> {
    id(&input.batch_id)?;
    validate_quantity(input.quantity)?;
    if !["damage", "expired", "supplier_return"].contains(&input.kind.as_str()) {
        return Err(invalid(
            "Write-off kind must be damage, expired or supplier_return.",
        ));
    }
    validate_reason_user(&input.reason, input.user.as_deref().unwrap_or(""))?;
    let user = input.user.as_deref().unwrap_or("");
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let batch = batches::get(&tx, &input.batch_id)?;
    if !batch.is_active || batches::balance_on(&tx, &batch.id)? < input.quantity {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Insufficient stock in the batch for this write-off.",
        ));
    }
    let movement = movements::insert_on(
        &tx,
        &batch.branch_id,
        &batch.product_package_id,
        &batch.id,
        &input.kind,
        -input.quantity,
        batch.cost_price_minor,
        None,
        None,
        &input.reason.trim(),
        user,
    )?;
    tx.commit()?;
    Ok(movement)
}
/// Branch-to-branch transfer. Posts a paired transfer_out / transfer_in and reuses
/// an existing destination lot with the same number and expiry, preserving cost and
/// expiry across the move.
pub fn transfer_stock(db: &AppDb, input: &TransferInput) -> Result<TransferResult, AppError> {
    id(&input.from_branch_id)?;
    id(&input.to_branch_id)?;
    id(&input.product_package_id)?;
    id(&input.batch_id)?;
    if input.from_branch_id == input.to_branch_id {
        return Err(invalid(
            "A transfer must move stock between two different branches.",
        ));
    }
    if input.reason.len() > 500 {
        return Err(invalid("The reason is too long."));
    }
    if input.user.as_deref().is_some_and(|u| u.len() > 160) {
        return Err(invalid("The operator name is too long."));
    }
    validate_quantity(input.quantity)?;
    let user = input.user.as_deref().unwrap_or("");
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let source = batches::get(&tx, &input.batch_id)?;
    if source.branch_id != input.from_branch_id
        || source.product_package_id != input.product_package_id
    {
        return Err(invalid("The batch does not belong to the source branch."));
    }
    if !source.is_active || batches::balance_on(&tx, &source.id)? < input.quantity {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Insufficient stock in the source batch for this transfer.",
        ));
    }
    crate::db::repositories::branches::get(&tx, &input.to_branch_id)?;
    let transfer_id = uuid::Uuid::new_v4().to_string();
    let out = movements::insert_on(
        &tx,
        &input.from_branch_id,
        &input.product_package_id,
        &source.id,
        "transfer_out",
        -input.quantity,
        source.cost_price_minor,
        Some("transfer"),
        Some(&transfer_id),
        &input.reason.trim(),
        user,
    )?;
    let destination = batches::ensure_on(
        &tx,
        &input.to_branch_id,
        &input.product_package_id,
        &source.batch_number,
        &source.expiry_date,
        source.cost_price_minor,
    )?;
    let incoming = movements::insert_on(
        &tx,
        &input.to_branch_id,
        &input.product_package_id,
        &destination.id,
        "transfer_in",
        input.quantity,
        destination.cost_price_minor,
        Some("transfer"),
        Some(&transfer_id),
        &input.reason.trim(),
        user,
    )?;
    tx.commit()?;
    Ok(TransferResult {
        out,
        incoming,
        from_batch_id: source.id,
        to_batch_id: destination.id,
    })
}
