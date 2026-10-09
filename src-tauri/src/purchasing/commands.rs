use super::{models::*, *};
use crate::{
    authorization::AuthorizationState, commands::branches::DatabaseState, error::AppError,
};
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

#[tauri::command]
pub fn receive_purchase_order(
    state: State<'_, DatabaseState>,
    authorization: State<'_, AuthorizationState>,
    input: ReceivePurchase,
) -> Result<PurchaseReceipt, AppError> {
    let mut input = input;
    input.created_by = Some(authorization.actor()?);
    receiving::receive(state.db()?, &input)
}
#[tauri::command]
pub fn get_purchase_receipt(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<PurchaseReceipt, AppError> {
    receiving::get(state.db()?, &id)
}
#[tauri::command]
pub fn list_purchase_receipts(
    state: State<'_, DatabaseState>,
    query: PurchaseReceiptQuery,
) -> Result<PurchaseReceiptPage, AppError> {
    receiving::list(state.db()?, &query)
}

#[tauri::command]
pub fn create_purchase_return(
    state: State<'_, DatabaseState>,
    authorization: State<'_, AuthorizationState>,
    input: CreatePurchaseReturn,
) -> Result<PurchaseReturn, AppError> {
    let mut input = input;
    input.created_by = Some(authorization.actor()?);
    returns::create(state.db()?, &input)
}
#[tauri::command]
pub fn get_purchase_return(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<PurchaseReturn, AppError> {
    returns::get(state.db()?, &id)
}
#[tauri::command]
pub fn list_purchase_returns(
    state: State<'_, DatabaseState>,
    query: PurchaseReturnQuery,
) -> Result<PurchaseReturnPage, AppError> {
    returns::list(state.db()?, &query)
}
