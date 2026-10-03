//! Runtime boundary for the recovered baseline. Prototype handlers are unregistered.
use crate::{
    commands::branches::DatabaseState,
    db::{connection::AppDb, models::inventory::FefoAllocation},
    error::AppError,
    inventory,
};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStatus {
    pub later_workflows_enabled: bool,
    pub inventory_workflows_enabled: bool,
    pub stock_counts_enabled: bool,
    pub inventory_alerts_enabled: bool,
    pub inventory_ui_enabled: bool,
    pub sales_enabled: bool,
    pub partners_enabled: bool,
    pub purchasing_enabled: bool,
}

#[tauri::command]
pub fn recovery_status() -> RecoveryStatus {
    RecoveryStatus {
        later_workflows_enabled: false,
        inventory_workflows_enabled: true,
        stock_counts_enabled: true,
        inventory_alerts_enabled: true,
        inventory_ui_enabled: true,
        sales_enabled: true,
        partners_enabled: true,
        purchasing_enabled: true,
    }
}

/// Read-only allocation adapter for later callers. Never reserves or posts stock.
/// The canonical Phase 3A query enforces branch/package scope and expiry ordering.
pub fn allocation(
    db: &AppDb,
    branch: &str,
    package: &str,
    quantity: Option<i64>,
) -> Result<Vec<FefoAllocation>, AppError> {
    if let Some(q) = quantity {
        inventory::quantity(q)?;
    }
    let today: String = db
        .lock()?
        .query_row("SELECT date('now')", [], |r| r.get(0))?;
    let mut offset = 0;
    let mut remaining = quantity.unwrap_or(0);
    let mut result = Vec::new();
    loop {
        let rows = inventory::expiry::fefo(db, branch, package, &today, Some(200), Some(offset))?;
        let size = rows.len();
        for row in rows {
            let take = remaining.min(row.quantity);
            remaining -= take;
            result.push(FefoAllocation {
                batch_id: row.batch.id,
                batch_number: row.batch.lot_number.unwrap_or_default(),
                expiry_date: row.batch.expiry_date.unwrap_or_default(),
                available: row.quantity,
                take,
            });
            if quantity.is_some() && remaining == 0 {
                return Ok(result);
            }
        }
        if size < 200 {
            break;
        }
        offset += 200;
    }
    if remaining > 0 {
        return Err(crate::db::catalog_validation::invalid(
            "Insufficient non-expired stock for this allocation.",
        ));
    }
    Ok(result)
}

#[tauri::command]
pub fn fefo_allocation(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
    quantity: Option<i64>,
) -> Result<Vec<FefoAllocation>, AppError> {
    allocation(state.db()?, &branch_id, &product_package_id, quantity)
}
