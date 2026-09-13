use super::{models::*, *};
use crate::{commands::branches::DatabaseState, error::AppError};
use tauri::State;
#[tauri::command]
pub fn create_inventory_batch(
    state: State<'_, DatabaseState>,
    input: CreateBatch,
) -> Result<InventoryBatch, AppError> {
    batches::create(state.db()?, &input)
}
#[tauri::command]
pub fn record_opening_stock(
    state: State<'_, DatabaseState>,
    input: OpeningStock,
) -> Result<InventoryMovement, AppError> {
    movements::opening(state.db()?, &input)
}
#[tauri::command]
pub fn record_batch_opening_stock(
    state: State<'_, DatabaseState>,
    input: StockOperation,
) -> Result<InventoryMovement, AppError> {
    movements::opening_existing(state.db()?, &input)
}
#[tauri::command]
pub fn record_adjustment(
    state: State<'_, DatabaseState>,
    input: Adjustment,
) -> Result<InventoryMovement, AppError> {
    movements::adjust(state.db()?, &input)
}
#[tauri::command]
pub fn record_damage(
    state: State<'_, DatabaseState>,
    input: StockOperation,
) -> Result<InventoryMovement, AppError> {
    movements::damage(state.db()?, &input)
}
#[tauri::command]
pub fn record_expiry_write_off(
    state: State<'_, DatabaseState>,
    input: StockOperation,
) -> Result<InventoryMovement, AppError> {
    movements::write_off(state.db()?, &input)
}
#[tauri::command]
pub fn transfer_inventory_stock(
    state: State<'_, DatabaseState>,
    input: TransferStock,
) -> Result<TransferResult, AppError> {
    transfers::transfer(state.db()?, &input)
}
#[tauri::command]
pub fn get_package_stock(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
) -> Result<InventoryBalance, AppError> {
    balances::package(state.db()?, &branch_id, &product_package_id)
}
#[tauri::command]
pub fn get_batch_stock(
    state: State<'_, DatabaseState>,
    branch_id: String,
    batch_id: String,
) -> Result<InventoryBalance, AppError> {
    balances::batch(state.db()?, &branch_id, &batch_id)
}
#[tauri::command]
pub fn list_branch_inventory(
    state: State<'_, DatabaseState>,
    query: InventoryQuery,
) -> Result<Vec<InventoryBalance>, AppError> {
    balances::list(state.db()?, &query)
}
#[tauri::command]
pub fn list_stock_movements(
    state: State<'_, DatabaseState>,
    query: InventoryQuery,
) -> Result<Vec<InventoryMovement>, AppError> {
    movements::list(state.db()?, &query)
}
#[tauri::command]
pub fn set_inventory_level(
    state: State<'_, DatabaseState>,
    input: SetLevel,
) -> Result<InventoryLevel, AppError> {
    levels::set(state.db()?, &input)
}
#[tauri::command]
pub fn list_low_stock(
    state: State<'_, DatabaseState>,
    query: InventoryQuery,
) -> Result<Vec<LowStockItem>, AppError> {
    levels::low(state.db()?, &query)
}
#[tauri::command]
pub fn list_expiring_batches(
    state: State<'_, DatabaseState>,
    query: ExpiryQuery,
) -> Result<Vec<ExpiryItem>, AppError> {
    expiry::expiring(state.db()?, &query)
}
#[tauri::command]
pub fn get_fefo_batches(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
    as_of: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ExpiryItem>, AppError> {
    expiry::fefo(
        state.db()?,
        &branch_id,
        &product_package_id,
        &as_of,
        limit,
        offset,
    )
}
