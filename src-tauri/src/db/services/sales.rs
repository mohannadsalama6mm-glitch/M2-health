use crate::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        models::catalog::Product,
        models::sales::*,
        repositories::{batches, movements, price_history, product_packages, products, sales},
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

/// Completes one sale. Selling price is snapshotted from the current price history;
/// unit cost is the cost of the first FEFO-allocated lot, matching the package cost
/// shown by the stock overview. Sale, items, batch allocation, payment and the sale
/// movements commit atomically.
pub fn complete_sale(db: &AppDb, input: &CompleteSaleInput) -> Result<CompleteSaleResult, AppError> {
    id(&input.branch_id)?;
    if input.lines.is_empty() {
        return Err(invalid("Add at least one product to the sale."));
    }
    if input.lines.len() > 200 {
        return Err(invalid("A sale can contain at most 200 lines."));
    }
    for line in &input.lines {
        id(&line.product_package_id)?;
        validate_proportional_quantity(line.quantity)?;
    }
    for value in [&input.discount_minor, &input.tax_minor, &input.paid_minor] {
        if !(0..=MAX_SAFE_MINOR).contains(value) {
            return Err(invalid("Amounts must be non-negative integer minor units within the supported range."));
        }
    }
    if !["cash", "card", "other"].contains(&input.payment_method.as_str()) {
        return Err(invalid("Payment method must be cash, card or other."));
    }
    if input.customer_name.len() > 120 {
        return Err(invalid("The customer name is too long."));
    }
    if input.customer_phone.len() > 40 {
        return Err(invalid("The customer phone is too long."));
    }
    let user = input.user.as_deref().unwrap_or("");
    validate_user(user)?;
    validate_note(&input.note)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::db::repositories::branches::get(&tx, &input.branch_id)?;
    if let Some(customer_id) = &input.customer_id {
        id(customer_id)?;
        crate::db::repositories::customers::get(&tx, customer_id)?;
    }
    struct PreparedLine {
        package_id: String,
        product_name: String,
        package_label: String,
        quantity: i64,
        selling_price_minor: i64,
        cost_price_minor: Option<i64>,
        line_total_minor: i64,
        line_cost_minor: Option<i64>,
        allocation: Vec<(String, i64, Option<i64>)>,
    }
    let mut prepared: Vec<PreparedLine> = Vec::new();
    let mut subtotal: i64 = 0;
    for line in &input.lines {
        let package = product_packages::get(&tx, &line.product_package_id)?;
        let product = products::get(&tx, &package.product_id)?;
        if !package.is_active || !product.is_active {
            return Err(invalid("Inactive products cannot be sold."));
        }
        let price = price_history::current_on(&tx, &package.id)?
            .ok_or_else(|| invalid("This package has no selling price; set a price first."))?;
        let line_total = checked_minor(
            price.selling_price_minor.checked_mul(line.quantity),
            "The line total exceeds the supported range.",
        )?;
        let mut remaining = line.quantity;
        let mut allocation = Vec::new();
        for (batch, qty) in batches::list_active_with_balance_on(&tx, &input.branch_id, &package.id)? {
            if qty <= 0 || remaining <= 0 {
                continue;
            }
            let take = remaining.min(qty);
            allocation.push((batch.id.clone(), take, batch.cost_price_minor));
            remaining -= take;
        }
        if remaining > 0 {
            tx.rollback()?;
            return Err(AppError::new(
                ErrorCode::Conflict,
                format!("Insufficient stock for {}.", product_label(&product)),
            ));
        }
        let unit_cost = allocation.first().and_then(|(_, _, cost)| *cost);
        let line_cost = match unit_cost {
            Some(cost) => Some(checked_minor(
                cost.checked_mul(line.quantity),
                "The line cost exceeds the supported range.",
            )?),
            None => None,
        };
        subtotal = checked_minor(
            subtotal.checked_add(line_total),
            "The sale total exceeds the supported range.",
        )?;
        prepared.push(PreparedLine {
            package_id: package.id.clone(),
            product_name: product_label(&product),
            package_label: package.package_label.trim().to_string(),
            quantity: line.quantity,
            selling_price_minor: price.selling_price_minor,
            cost_price_minor: unit_cost,
            line_total_minor: line_total,
            line_cost_minor: line_cost,
            allocation,
        });
    }
    let total = checked_minor(
        subtotal
            .checked_sub(input.discount_minor)
            .and_then(|v| v.checked_add(input.tax_minor)),
        "The discount exceeds the sale total.",
    )?;
    if input.paid_minor < total {
        tx.rollback()?;
        return Err(invalid("The amount paid must cover the sale total."));
    }
    let change = input.paid_minor - total;
    let snapshot = match &input.customer_id {
        Some(customer_id) => {
            let customer = crate::db::repositories::customers::get(&tx, customer_id)?;
            Some(customer)
        }
        None => None,
    };
    let new_sale = sales::NewSale {
        branch_id: input.branch_id.clone(),
        customer_id: input.customer_id.clone(),
        customer_name: snapshot
            .as_ref()
            .map(|c| c.name.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| input.customer_name.trim().to_string()),
        customer_phone: snapshot
            .as_ref()
            .map(|c| c.phone.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| input.customer_phone.trim().to_string()),
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
    let sale = sales::insert_sale_on(&tx, &new_sale)?;
    sales::insert_payment_on(&tx, &sale.id, &new_sale.payment_method, input.paid_minor)?;
    let mut movements = Vec::new();
    for (position, line) in prepared.iter().enumerate() {
        let item = sales::insert_item_on(
            &tx,
            &sale.id,
            &line.package_id,
            &line.product_name,
            &line.package_label,
            line.quantity,
            line.selling_price_minor,
            line.cost_price_minor,
            line.line_total_minor,
            line.line_cost_minor,
            position as i64,
        )?;
        for (batch_id, take, cost) in &line.allocation {
            sales::insert_item_batch_on(&tx, &sale.id, &item.id, batch_id, *take, *cost)?;
            movements.push(movements::insert_on(
                &tx,
                &sale.branch_id,
                &line.package_id,
                batch_id,
                "sale",
                -(*take),
                *cost,
                Some("sale"),
                Some(&sale.id),
                &format!("POS sale {}", sale.receipt_number),
                user,
            )?);
        }
    }
    tx.commit()?;
    Ok(CompleteSaleResult { sale, movements })
}

/// Customer return for a completed sale. Each returned line must not exceed what was
/// sold minus what was already returned; items are restocked into their original lot
/// when it is still active in the sale's branch, otherwise into a GENERAL lot.
pub fn return_sale(db: &AppDb, input: &ReturnSaleInput) -> Result<SaleReturn, AppError> {
    id(&input.sale_id)?;
    let reason = input.reason.trim();
    if reason.is_empty() || reason.len() > 500 {
        return Err(invalid("A reason of 1–500 characters is required."));
    }
    let user = input.user.as_deref().unwrap_or("");
    validate_user(user)?;
    if input.items.is_empty() {
        return Err(invalid("Add at least one returned line."));
    }
    if input.items.len() > 200 {
        return Err(invalid("A return can contain at most 200 lines."));
    }
    for item in &input.items {
        id(&item.sale_item_id)?;
        validate_proportional_quantity(item.quantity)?;
        if !(0..=MAX_SAFE_MINOR).contains(&item.refund_minor) {
            return Err(invalid("Refund amounts must be non-negative integer minor units."));
        }
    }
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let sale = sales::get(&tx, &input.sale_id)?;
    if sale.status != "completed" {
        tx.rollback()?;
        return Err(AppError::new(ErrorCode::Conflict, "Only completed sales can be returned."));
    }
    let returned = sales::returned_by_item_on(&tx, &sale.id)?;
    let mut granted: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut prepared = Vec::new();
    let mut total_refund: i64 = 0;
    for item in &input.items {
        let sale_item = sales::item_on(&tx, &item.sale_item_id)?;
        if sale_item.sale_id != sale.id {
            tx.rollback()?;
            return Err(invalid("The returned line does not belong to this sale."));
        }
        let already = returned.get(&sale_item.id).copied().unwrap_or(0)
            + granted.get(&sale_item.id).copied().unwrap_or(0);
        let returnable = sale_item.quantity - already;
        if item.quantity > returnable {
            tx.rollback()?;
            return Err(AppError::new(
                ErrorCode::Conflict,
                "The return exceeds the quantity sold for a line.",
            ));
        }
        if item.refund_minor > sale_item.selling_price_minor {
            tx.rollback()?;
            return Err(invalid("The refund cannot exceed the original selling price."));
        }
        let line_refund = checked_minor(
            item.quantity.checked_mul(item.refund_minor),
            "The refund total exceeds the supported range.",
        )?;
        total_refund = checked_minor(
            total_refund.checked_add(line_refund),
            "The refund total exceeds the supported range.",
        )?;
        granted.entry(sale_item.id.clone()).and_modify(|v| *v += item.quantity).or_insert(item.quantity);
        prepared.push((sale_item, item.quantity, item.refund_minor));
    }
    let returned_record = sales::insert_return_on(&tx, &sale.id, &sale.branch_id, reason, user, total_refund)?;
    let mut items = Vec::new();
    for (sale_item, quantity, refund_minor) in prepared {
        let restock = original_lot_on(&tx, &sale, &sale_item)?;
        movements::insert_on(
            &tx,
            &sale.branch_id,
            &sale_item.product_package_id,
            &restock.0,
            "customer_return",
            quantity,
            restock.1,
            Some("sale"),
            Some(&sale.id),
            reason,
            user,
        )?;
        items.push(sales::insert_return_item_on(
            &tx,
            &returned_record.id,
            &sale_item.id,
            &sale_item.product_package_id,
            quantity,
            refund_minor,
        )?);
    }
    tx.commit()?;
    Ok(SaleReturn {
        items,
        ..returned_record
    })
}
fn original_lot_on(
    tx: &rusqlite::Connection,
    sale: &Sale,
    sale_item: &SaleItem,
) -> Result<(String, Option<i64>), AppError> {
    match sales::first_allocated_batch_on(tx, &sale_item.id)? {
        Some(id) => {
            let batch = batches::get(tx, &id)?;
            if batch.is_active && batch.branch_id == sale.branch_id {
                Ok((batch.id, batch.cost_price_minor))
            } else {
                general_lot_on(tx, sale, sale_item)
            }
        }
        None => general_lot_on(tx, sale, sale_item),
    }
}
fn general_lot_on(
    tx: &rusqlite::Connection,
    sale: &Sale,
    sale_item: &SaleItem,
) -> Result<(String, Option<i64>), AppError> {
    let batch = batches::ensure_on(tx, &sale.branch_id, &sale_item.product_package_id, "GENERAL", "", None)?;
    Ok((batch.id, batch.cost_price_minor))
}

/// Voids a completed sale by reversing the exact batches it consumed. A sale with
/// customer returns cannot be voided because its stock was already restocked.
pub fn void_sale(db: &AppDb, input: &VoidSaleInput) -> Result<Sale, AppError> {
    id(&input.sale_id)?;
    let reason = input.reason.trim();
    if reason.is_empty() || reason.len() > 500 {
        return Err(invalid("A reason of 1–500 characters is required."));
    }
    let user = input.user.as_deref().unwrap_or("");
    validate_user(user)?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let sale = sales::get(&tx, &input.sale_id)?;
    if sale.status != "completed" {
        tx.rollback()?;
        return Err(AppError::new(ErrorCode::Conflict, "Only completed sales can be voided."));
    }
    let has_returns: i64 = tx.query_row(
        "SELECT count(*) FROM sale_returns WHERE sale_id=?1",
        [&sale.id],
        |r| r.get(0),
    )?;
    if has_returns > 0 {
        tx.rollback()?;
        return Err(AppError::new(
            ErrorCode::Conflict,
            "A sale with customer returns cannot be voided.",
        ));
    }
    let allocations = sales::allocations_on(&tx, &sale.id)?;
    for (_item_id, allocation) in allocations {
        let package_id: String = tx.query_row(
            "SELECT product_package_id FROM inventory_batches WHERE id=?1",
            [&allocation.batch_id],
            |r| r.get(0),
        )?;
        movements::insert_on(
            &tx,
            &sale.branch_id,
            &package_id,
            &allocation.batch_id,
            "adjustment",
            allocation.quantity,
            allocation.cost_price_minor,
            Some("sale"),
            Some(&sale.id),
            &format!("Void sale {}", sale.receipt_number),
            user,
        )?;
    }
    let updated = sales::update_void_on(&tx, &sale.id, reason, user)?;
    tx.commit()?;
    Ok(updated)
}