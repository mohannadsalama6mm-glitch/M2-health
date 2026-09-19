use super::{models::*, *};
use crate::commands::branches::DatabaseState;
use tauri::State;
#[tauri::command]
pub fn get_inventory_alert_settings(
    state: State<'_, DatabaseState>,
    branch_id: String,
) -> Result<Settings, AppError> {
    settings(state.db()?, &branch_id)
}
#[tauri::command]
pub fn update_inventory_alert_settings(
    state: State<'_, DatabaseState>,
    input: Settings,
) -> Result<Settings, AppError> {
    let db = state.db()?;
    let result = update_settings(db, &input)?;
    refresh_all(db);
    Ok(result)
}
#[tauri::command]
pub fn evaluate_inventory_alerts(
    state: State<'_, DatabaseState>,
    branch_id: String,
) -> Result<serde_json::Value, AppError> {
    evaluate(state.db()?, &branch_id, None)
}
#[tauri::command]
pub fn list_inventory_alerts(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Page<Alert>, AppError> {
    list(state.db()?, &query)
}
#[tauri::command]
pub fn get_inventory_alert(
    state: State<'_, DatabaseState>,
    branch_id: String,
    id: String,
) -> Result<serde_json::Value, AppError> {
    get(state.db()?, &branch_id, &id)
}
#[tauri::command]
pub fn dismiss_inventory_alert(
    state: State<'_, DatabaseState>,
    branch_id: String,
    id: String,
) -> Result<(), AppError> {
    dismiss(state.db()?, &branch_id, &id)
}
#[tauri::command]
pub fn list_expiry_inventory(
    state: State<'_, DatabaseState>,
    query: MonitorQuery,
) -> Result<Page<Monitor>, AppError> {
    monitor(state.db()?, &query, false)
}
#[tauri::command]
pub fn list_low_stock_inventory(
    state: State<'_, DatabaseState>,
    query: MonitorQuery,
) -> Result<Page<Monitor>, AppError> {
    monitor(state.db()?, &query, true)
}
#[tauri::command]
pub fn get_fefo_inventory_batches(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<super::super::models::ExpiryItem>, AppError> {
    let db = state.db()?;
    let date = business_date(&*db.lock()?, None)?;
    super::super::expiry::fefo(db, &branch_id, &product_package_id, &date, limit, offset)
}
