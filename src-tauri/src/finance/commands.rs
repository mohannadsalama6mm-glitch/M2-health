use super::{models::*, *};
use crate::{
    authorization::AuthorizationState, commands::branches::DatabaseState, error::AppError,
};
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn list_finance_accounts(state: State<'_, DatabaseState>) -> Result<Vec<Value>, AppError> {
    accounts(state.db()?)
}
#[tauri::command]
pub fn create_finance_expense(
    state: State<'_, DatabaseState>,
    authorization: State<'_, AuthorizationState>,
    input: CreateExpense,
) -> Result<Value, AppError> {
    let mut input = input;
    input.created_by = Some(authorization.actor()?);
    create_expense(state.db()?, &input)
}
#[tauri::command]
pub fn get_finance_expense(state: State<'_, DatabaseState>, id: String) -> Result<Value, AppError> {
    expense(state.db()?, &id)
}
#[tauri::command]
pub fn list_finance_expenses(
    state: State<'_, DatabaseState>,
    query: FinanceQuery,
) -> Result<Value, AppError> {
    expenses(state.db()?, &query)
}
#[tauri::command]
pub fn get_finance_journal(state: State<'_, DatabaseState>, id: String) -> Result<Value, AppError> {
    journal(state.db()?, &id)
}
#[tauri::command]
pub fn list_finance_journals(
    state: State<'_, DatabaseState>,
    query: FinanceQuery,
) -> Result<Value, AppError> {
    journals(state.db()?, &query)
}
#[tauri::command]
pub fn get_trial_balance(
    state: State<'_, DatabaseState>,
    query: FinanceQuery,
) -> Result<Value, AppError> {
    trial_balance(state.db()?, &query)
}
#[tauri::command]
pub fn get_account_activity(
    state: State<'_, DatabaseState>,
    query: FinanceQuery,
) -> Result<Value, AppError> {
    account_activity(state.db()?, &query)
}
#[tauri::command]
pub fn get_sales_financial_summary(
    state: State<'_, DatabaseState>,
    query: FinanceQuery,
) -> Result<Value, AppError> {
    sales_summary(state.db()?, &query)
}
#[tauri::command]
pub fn get_expense_report(
    state: State<'_, DatabaseState>,
    query: FinanceQuery,
) -> Result<Value, AppError> {
    expense_report(state.db()?, &query)
}
