use crate::{
    commands::branches::DatabaseState,
    db::{
        models::purchases::*,
        repositories::{purchases},
        services,
    },
    error::AppError,
};
use tauri::State;
/// Thin purchase commands. Business rules live in db::services; queries in
/// db::repositories. All amounts are integer EGP minor units.

#[tauri::command]
pub fn complete_purchase(
    state: State<'_, DatabaseState>,
    input: CompletePurchaseInput,
) -> Result<CompletePurchaseResult, AppError> {
    services::purchases::complete_purchase(state.db()?, &input)
}
#[tauri::command]
pub fn void_purchase(
    state: State<'_, DatabaseState>,
    input: VoidPurchaseInput,
) -> Result<Purchase, AppError> {
    services::purchases::void_purchase(state.db()?, &input)
}
#[tauri::command]
pub fn list_purchases(
    state: State<'_, DatabaseState>,
    query: PurchaseQuery,
) -> Result<PurchasePage, AppError> {
    purchases::list(state.db()?, &query)
}
#[tauri::command]
pub fn get_purchase(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<PurchaseDetail, AppError> {
    purchases::detail(state.db()?, &id)
}
#[tauri::command]
pub fn purchase_pos_search(
    state: State<'_, DatabaseState>,
    query: PosPurchaseQuery,
) -> Result<PurchaseProductPage, AppError> {
    purchases::pos_search(state.db()?, &query)
}