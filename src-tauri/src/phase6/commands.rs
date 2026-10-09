use super::{models::*, *};
use crate::{commands::branches::DatabaseState, error::AppError};
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn get_phase6_dashboard(
    state: State<'_, DatabaseState>,
    filter: Phase6Filter,
) -> Result<Value, AppError> {
    dashboard(state.db()?, &filter)
}

#[tauri::command]
pub fn get_phase6_reconciliation(
    state: State<'_, DatabaseState>,
    filter: Phase6Filter,
) -> Result<Value, AppError> {
    reconciliation(state.db()?, &filter)
}

#[tauri::command]
pub fn get_sale_finance_trace(
    state: State<'_, DatabaseState>,
    sale_id: String,
) -> Result<Value, AppError> {
    sale_finance_trace(state.db()?, &sale_id)
}

#[tauri::command]
pub fn export_phase6_csv(
    state: State<'_, DatabaseState>,
    input: CsvExportInput,
) -> Result<Value, AppError> {
    export_csv(state.db()?, &input)
}
