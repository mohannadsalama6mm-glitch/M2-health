use crate::{
    commands::branches::DatabaseState,
    db::{
        models::sales::*,
        repositories::{sales},
        services,
    },
    error::AppError,
};
use tauri::State;
/// Thin POS commands. Business rules live in db::services; queries in
/// db::repositories. All amounts are integer EGP minor units.

#[tauri::command]
pub fn complete_sale(
    state: State<'_, DatabaseState>,
    input: CompleteSaleInput,
) -> Result<CompleteSaleResult, AppError> {
    services::sales::complete_sale(state.db()?, &input)
}
#[tauri::command]
pub fn return_sale(
    state: State<'_, DatabaseState>,
    input: ReturnSaleInput,
) -> Result<SaleReturn, AppError> {
    services::sales::return_sale(state.db()?, &input)
}
#[tauri::command]
pub fn void_sale(
    state: State<'_, DatabaseState>,
    input: VoidSaleInput,
) -> Result<Sale, AppError> {
    services::sales::void_sale(state.db()?, &input)
}
#[tauri::command]
pub fn list_sales(
    state: State<'_, DatabaseState>,
    query: SaleQuery,
) -> Result<SalePage, AppError> {
    sales::list(state.db()?, &query)
}
#[tauri::command]
pub fn get_sale(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<SaleDetail, AppError> {
    sales::detail(state.db()?, &id)
}
#[tauri::command]
pub fn pos_search(
    state: State<'_, DatabaseState>,
    query: PosQuery,
) -> Result<PosProductPage, AppError> {
    sales::pos_search(state.db()?, &query)
}