use crate::{
    commands::branches::DatabaseState,
    db::{
        models::suppliers::*,
        repositories::suppliers,
    },
    error::AppError,
};
use tauri::State;
/// Thin supplier commands. Business rules and queries live in
/// db::repositories; codes are auto-generated from the name.

#[tauri::command]
pub fn create_supplier(
    state: State<'_, DatabaseState>,
    input: CreateSupplierInput,
) -> Result<Supplier, AppError> {
    suppliers::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_suppliers(
    state: State<'_, DatabaseState>,
    query: SupplierQuery,
) -> Result<SupplierPage, AppError> {
    suppliers::list(state.db()?, &query)
}
#[tauri::command]
pub fn get_supplier(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Supplier, AppError> {
    suppliers::get_supplier(state.db()?, &id)
}
#[tauri::command]
pub fn update_supplier(
    state: State<'_, DatabaseState>,
    input: UpdateSupplierInput,
) -> Result<Supplier, AppError> {
    suppliers::update(state.db()?, &input)
}
#[tauri::command]
pub fn set_supplier_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<Supplier, AppError> {
    suppliers::set_active(state.db()?, &id, active)
}