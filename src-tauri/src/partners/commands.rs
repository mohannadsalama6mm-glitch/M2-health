use super::{models::*, *};
use crate::commands::branches::DatabaseState;
use tauri::State;

#[tauri::command]
pub fn create_canonical_supplier(
    state: State<'_, DatabaseState>,
    input: SupplierFields,
) -> Result<Supplier, AppError> {
    create_supplier(state.db()?, &input)
}
#[tauri::command]
pub fn get_canonical_supplier(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Supplier, AppError> {
    get_supplier(state.db()?, &id)
}
#[tauri::command]
pub fn update_canonical_supplier(
    state: State<'_, DatabaseState>,
    input: UpdateSupplier,
) -> Result<Supplier, AppError> {
    update_supplier(state.db()?, &input)
}
#[tauri::command]
pub fn set_canonical_supplier_active(
    state: State<'_, DatabaseState>,
    input: SetPartnerActive,
) -> Result<Supplier, AppError> {
    set_supplier_active(state.db()?, &input)
}
#[tauri::command]
pub fn list_canonical_suppliers(
    state: State<'_, DatabaseState>,
    query: PartnerQuery,
) -> Result<PartnerPage<Supplier>, AppError> {
    list_suppliers(state.db()?, &query)
}

#[tauri::command]
pub fn create_canonical_customer(
    state: State<'_, DatabaseState>,
    input: CustomerFields,
) -> Result<Customer, AppError> {
    create_customer(state.db()?, &input)
}
#[tauri::command]
pub fn get_canonical_customer(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Customer, AppError> {
    get_customer(state.db()?, &id)
}
#[tauri::command]
pub fn update_canonical_customer(
    state: State<'_, DatabaseState>,
    input: UpdateCustomer,
) -> Result<Customer, AppError> {
    update_customer(state.db()?, &input)
}
#[tauri::command]
pub fn set_canonical_customer_active(
    state: State<'_, DatabaseState>,
    input: SetPartnerActive,
) -> Result<Customer, AppError> {
    set_customer_active(state.db()?, &input)
}
#[tauri::command]
pub fn list_canonical_customers(
    state: State<'_, DatabaseState>,
    query: PartnerQuery,
) -> Result<PartnerPage<Customer>, AppError> {
    list_customers(state.db()?, &query)
}
