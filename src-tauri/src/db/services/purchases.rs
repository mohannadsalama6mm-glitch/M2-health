use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::catalog::Product,
        models::purchases::*,
        repositories::{batches, movements, product_packages, products, purchases},
    },
    error::{AppError, ErrorCode},
};
use rusqlite::TransactionBehavior;

fn validate_user(value: &str) -> Result<(), AppError> {
    if value.len() > 160 {
        return Err(invalid("The operator name is too long."));
    }
    Ok(())
}
fn validate_note(value: &str) -> Result<(), AppError> {
    if value.len() > 500 {
        return Err(invalid("The note is too long."));
    }
    Ok(())
}
fn validate_invoice_number(value: &str) -> Result<(), AppError> {
    if value.len() > 60 {
        return Err(invalid("The invoice number is too long."));
    }
    Ok(())
}
fn validate_proportional_quantity(value: i64) -> Result<(), AppError> {
    if value <= 0 || value > MAX_SAFE_MINOR {
        return Err(invalid("Quantity must be a positive count within the supported range."));
    }
    Ok(())
}
fn checked_minor(value: Option<i64>, message: &str) -> Result<i64, AppError> {
    value
        .filter(|v| (0..=MAX_SAFE_MINOR).contains(v))
        .ok_or_else(|| invalid(message))
}
fn product_label(product: &Product) -> String {
    product
        .commercial_name_en
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(str::trim)
        .map(str::to_string)
        .or_else(|| {
            product
                .scientific_name
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(str::trim)
                .map(str::to_string)
        })
        .or_else(|| {
            product
                .commercial_name_ar
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(str::trim)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Unnamed product".into())
}

/// Completes one purchase. Unit costs are snapshotted from the line input; each
/// line is booked into a per-position PURCHASE-N lot in the branch's FEFO pool.
/// Purchase, items, batch allocation, payment and the purchase movements commit
/// atomically.
pub fn complete_purchase(db: &AppDb, input: &CompletePurchaseInput) -> Result<CompletePurchaseResult, AppError> {
    id(&input.branch_id)?;
    if input.lines.is_empty() {
        return Err(invalid("Add at least one line to the purchase."));
    }
    if input.lines.len() > 200 {
        return Err(invalid("A purchase can contain at most 200 lines."));
    }
    if let Some(supplier_id) = &input.supplier_id {
        id(supplier_id)?;
    }
    for line in &input.lines {
        id(&line.product_package_id)?;
        validate_proportional_quantity(line.quantity)?;
        if !(0..=MAX_SAFE_MINOR).contains(&line.unit_cost_minor) {
            return Err(invalid("Unit costs must be non-negative integer minor units within the supported range."));
        }
    }
    for value in [&input.discount_minor, &input.tax_minor, &input.paid_minor] {
        if !(0..=MAX_SAFE_MINOR).contains(value) {
            return Err(invalid("Amounts must be non-negative integer minor units within the supported range."));
        }
    }
    if !["cash", "card", "other"].contains(&input.payment_method.as_str()) {
        return Err(invalid("Payment method must be cash, card or other."));
    }
    let user = input.user.as_deref().unwrap_or("");
    validate_user(user)?;
    validate_note(&input.note)?;
    validate_invoice_number(&input.invoice_number)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::db::repositories::branches::get(&tx, &input.branch_id)?;
    if let Some(supplier_id) = &input.supplier_id {
        crate::db::repositories::suppliers::get(&tx, supplier_id)?;
    }
    struct PreparedLine {
        package_id: String,
        product_name: String,
        package_label: String,
        quantity: i64,
        unit_cost_minor: i64,
        line_total_minor: i64,
    }
    let mut prepared: Vec<PreparedLine> = Vec::new();
    let mut subtotal: i64 = 0;
    for line in &input.lines {
        let package = product_packages::get(&tx, &line.product_package_id)?;
        let product = products::get(&tx, &package.product_id)?;
        if !package.is_active || !product.is_active {
            return Err(invalid("Inactive products cannot be purchased."));
        }
        let line_total = checked_minor(
            line.unit_cost_minor.checked_mul(line.quantity),
            "The line total exceeds the supported range.",
        )?;
        subtotal = checked_minor(
            subtotal.checked_add(line_total),
            "The purchase total exceeds the supported range.",
        )?;
        prepared.push(PreparedLine {
            package_id: package.id.clone(),
            product_name: product_label(&product),
            package_label: package.package_label.trim().to_string(),
            quantity: line.quantity,
            unit_cost_minor: line.unit_cost_minor,
            line_total_minor: line_total,
        });
    }
    let total = checked_minor(
        subtotal
            .checked_sub(input.discount_minor)
            .and_then(|v| v.checked_add(input.tax_minor)),
        "The discount exceeds the purchase total.",
    )?;
    if input.paid_minor < total {
        tx.rollback()?;
        return Err(invalid("The amount paid must cover the purchase total."));
    }
    let change = input.paid_minor - total;
    let new_purchase = purchases::NewPurchase {
        branch_id: input.branch_id.clone(),
        supplier_id: input.supplier_id.clone(),
        invoice_number: input.invoice_number.trim().to_string(),
        subtotal_minor: subtotal,
        discount_minor: input.discount_minor,
        tax_minor: input.tax_minor,
        total_minor: total,
        paid_minor: input.paid_minor,
        change_minor: change,
        payment_method: input.payment_method.trim().to_string(),
        user: user.into(),
        note: input.note.trim().to_string(),
    };
    let purchase = purchases::insert_purchase_on(&tx, &new_purchase)?;
    purchases::insert_payment_on(&tx, &purchase.id, &new_purchase.payment_method, input.paid_minor)?;
    let mut movements = Vec::new();
    for (position, line) in prepared.iter().enumerate() {
        let batch = batches::ensure_on(
            &tx,
            &input.branch_id,
            &line.package_id,
            &format!("PURCHASE-{position}"),
            "",
            Some(line.unit_cost_minor),
        )?;
        let item = purchases::insert_item_on(
            &tx,
            &purchase.id,
            &line.package_id,
            &line.product_name,
            &line.package_label,
            line.quantity,
            line.unit_cost_minor,
            line.line_total_minor,
            position as i64,
        )?;
        purchases::insert_item_batch_on(
            &tx,
            &purchase.id,
            &item.id,
            &batch.id,
            line.quantity,
            Some(line.unit_cost_minor),
        )?;
        movements.push(movements::insert_on(
            &tx,
            &input.branch_id,
            &line.package_id,
            &batch.id,
            "purchase",
            line.quantity,
            Some(line.unit_cost_minor),
            Some("purchase"),
            Some(&purchase.id),
            &format!("Purchase {}", purchase.purchase_number),
            user,
        )?);
    }
    tx.commit()?;
    Ok(CompletePurchaseResult { purchase, movements })
}

/// Voids a completed purchase by reversing the lots it booked back out of stock.
pub fn void_purchase(db: &AppDb, input: &VoidPurchaseInput) -> Result<Purchase, AppError> {
    id(&input.purchase_id)?;
    let reason = input.reason.trim();
    if reason.is_empty() || reason.len() > 500 {
        return Err(invalid("A reason of 1–500 characters is required."));
    }
    let user = input.user.as_deref().unwrap_or("");
    validate_user(user)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let purchase = purchases::get(&tx, &input.purchase_id)?;
    if purchase.status != "completed" {
        tx.rollback()?;
        return Err(AppError::new(ErrorCode::Conflict, "Only completed purchases can be voided."));
    }
    let allocations = purchases::allocations_on(&tx, &purchase.id)?;
    for (package_id, batch_id, quantity, cost) in allocations {
        movements::insert_on(
            &tx,
            &purchase.branch_id,
            &package_id,
            &batch_id,
            "adjustment",
            -quantity,
            cost,
            Some("purchase"),
            Some(&purchase.id),
            &format!("Void purchase {}", purchase.purchase_number),
            user,
        )?;
    }
    let updated = purchases::update_void_on(&tx, &purchase.id, reason, user)?;
    tx.commit()?;
    Ok(updated)
}