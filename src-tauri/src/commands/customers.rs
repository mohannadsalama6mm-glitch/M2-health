use crate::{
    commands::branches::DatabaseState,
    db::{
        models::customers::*,
        repositories::customers,
    },
    error::AppError,
};
use tauri::State;
/// Thin customer commands. Business rules and queries live in
/// db::repositories; codes are auto-generated from the name.

#[tauri::command]
pub fn create_customer(
    state: State<'_, DatabaseState>,
    input: CreateCustomerInput,
) -> Result<Customer, AppError> {
    customers::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_customers(
    state: State<'_, DatabaseState>,
    query: CustomerQuery,
) -> Result<CustomerPage, AppError> {
    customers::list(state.db()?, &query)
}
#[tauri::command]
pub fn get_customer(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Customer, AppError> {
    customers::get_customer(state.db()?, &id)
}
#[tauri::command]
pub fn update_customer(
    state: State<'_, DatabaseState>,
    input: UpdateCustomerInput,
) -> Result<Customer, AppError> {
    customers::update(state.db()?, &input)
}
#[tauri::command]
pub fn set_customer_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<Customer, AppError> {
    customers::set_active(state.db()?, &id, active)
}