use crate::{
    commands::branches::DatabaseState,
    db::{
        models::inventory::*,
        repositories::{counts, movements, stocks},
        services,
    },
    error::AppError,
};
use tauri::State;
/// Thin inventory commands. Business rules live in db::services; queries in
/// db::repositories. All amounts are integer EGP minor units.

#[tauri::command]
pub fn get_stock_overview(
    state: State<'_, DatabaseState>,
    query: StockOverviewQuery,
) -> Result<StockOverviewPage, AppError> {
    stocks::overview(state.db()?, &query)
}
#[tauri::command]
pub fn get_stock_summary(
    state: State<'_, DatabaseState>,
    branch_id: String,
) -> Result<StockSummary, AppError> {
    stocks::summary(state.db()?, &branch_id)
}
#[tauri::command]
pub fn list_stock_movements(
    state: State<'_, DatabaseState>,
    query: MovementQuery,
) -> Result<StockMovementPage, AppError> {
    movements::list(state.db()?, &query)
}
#[tauri::command]
pub fn post_opening_stock(
    state: State<'_, DatabaseState>,
    input: OpeningStockInput,
) -> Result<StockMovement, AppError> {
    services::inventory::opening_stock(state.db()?, &input)
}
#[tauri::command]
pub fn adjust_stock(
    state: State<'_, DatabaseState>,
    input: AdjustStockInput,
) -> Result<AdjustResult, AppError> {
    services::inventory::adjust_stock(state.db()?, &input)
}
#[tauri::command]
pub fn write_off_stock(
    state: State<'_, DatabaseState>,
    input: WriteOffInput,
) -> Result<StockMovement, AppError> {
    services::inventory::write_off(state.db()?, &input)
}
#[tauri::command]
pub fn transfer_stock(
    state: State<'_, DatabaseState>,
    input: TransferInput,
) -> Result<TransferResult, AppError> {
    services::inventory::transfer_stock(state.db()?, &input)
}
#[tauri::command]
pub fn list_stock_counts(
    state: State<'_, DatabaseState>,
    query: CountQuery,
) -> Result<StockCountPage, AppError> {
    counts::list(state.db()?, &query)
}
#[tauri::command]
pub fn get_stock_count(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StockCountDetail, AppError> {
    counts::detail(state.db()?, &id)
}
#[tauri::command]
pub fn create_stock_count(
    state: State<'_, DatabaseState>,
    input: CreateCountInput,
) -> Result<StockCount, AppError> {
    services::counts::create_count(state.db()?, &input)
}
#[tauri::command]
pub fn save_count_item(
    state: State<'_, DatabaseState>,
    input: SaveCountItemInput,
) -> Result<StockCountItem, AppError> {
    services::counts::save_item(state.db()?, &input)
}
#[tauri::command]
pub fn complete_stock_count(
    state: State<'_, DatabaseState>,
    input: CompleteCountInput,
) -> Result<StockCountDetail, AppError> {
    services::counts::complete_count(state.db()?, &input)
}
#[tauri::command]
pub fn list_expiry(
    state: State<'_, DatabaseState>,
    query: ExpiryQuery,
) -> Result<ExpiryPage, AppError> {
    stocks::expiry(state.db()?, &query)
}
#[tauri::command]
pub fn list_low_stock(
    state: State<'_, DatabaseState>,
    query: LowStockQuery,
) -> Result<LowStockPage, AppError> {
    stocks::low_stock(state.db()?, &query)
}
#[tauri::command]
pub fn set_reorder_level(
    state: State<'_, DatabaseState>,
    input: SetReorderInput,
) -> Result<InventorySetting, AppError> {
    services::settings::set_reorder(state.db()?, &input)
}
#[tauri::command]
pub fn fefo_allocation(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
    quantity: Option<i64>,
) -> Result<Vec<FefoAllocation>, AppError> {
    stocks::fefo_allocation(state.db()?, &branch_id, &product_package_id, quantity)
}
