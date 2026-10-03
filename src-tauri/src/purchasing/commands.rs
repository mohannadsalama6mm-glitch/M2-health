use super::{models::*, *};
use crate::{commands::branches::DatabaseState, error::AppError};
use tauri::State;
#[tauri::command]
pub fn create_purchase_order_draft(
    state: State<'_, DatabaseState>,
    input: CreatePurchaseDraft,
) -> Result<PurchaseOrder, AppError> {
    create_draft(state.db()?, &input)
}
#[tauri::command]
pub fn update_purchase_order_draft(
    state: State<'_, DatabaseState>,
    input: UpdatePurchaseDraft,
) -> Result<PurchaseOrder, AppError> {
    update_draft(state.db()?, &input)
}
#[tauri::command]
pub fn order_purchase_order(
    state: State<'_, DatabaseState>,
    input: PurchaseAction,
) -> Result<PurchaseOrder, AppError> {
    order(state.db()?, &input)
}
#[tauri::command]
pub fn cancel_purchase_order(
    state: State<'_, DatabaseState>,
    input: CancelPurchase,
) -> Result<PurchaseOrder, AppError> {
    cancel(state.db()?, &input)
}
#[tauri::command]
pub fn get_purchase_order(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<PurchaseOrder, AppError> {
    detail(state.db()?, &id)
}
#[tauri::command]
pub fn list_purchase_orders(
    state: State<'_, DatabaseState>,
    query: PurchaseQuery,
) -> Result<PurchasePage, AppError> {
    list(state.db()?, &query)
}
#[tauri::command]
pub fn search_purchase_packages(
    state: State<'_, DatabaseState>,
    query: PackageQuery,
) -> Result<PackagePage, AppError> {
    search_packages(state.db()?, &query)
}
