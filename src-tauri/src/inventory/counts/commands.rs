use super::{models::*, *};
use crate::commands::branches::DatabaseState;
use tauri::State;
#[tauri::command]
pub fn create_count_session(
    state: State<'_, DatabaseState>,
    input: Create,
) -> Result<Detail, AppError> {
    create(state.db()?, &input)
}
#[tauri::command]
pub fn get_count_session(
    state: State<'_, DatabaseState>,
    id: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Detail, AppError> {
    get(state.db()?, &id, limit, offset)
}
#[tauri::command]
pub fn list_count_sessions(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Page, AppError> {
    list(state.db()?, &query)
}
#[tauri::command]
pub fn start_count_session(
    state: State<'_, DatabaseState>,
    input: Transition,
) -> Result<Detail, AppError> {
    start(state.db()?, &input)
}
#[tauri::command]
pub fn record_count_entry(
    state: State<'_, DatabaseState>,
    input: Entry,
) -> Result<Detail, AppError> {
    record(state.db()?, &input)
}
#[tauri::command]
pub fn discover_count_stock(
    state: State<'_, DatabaseState>,
    input: Discover,
) -> Result<Detail, AppError> {
    discover(state.db()?, &input)
}
#[tauri::command]
pub fn review_count_session(
    state: State<'_, DatabaseState>,
    input: Transition,
) -> Result<Detail, AppError> {
    transition(state.db()?, &input, Status::Review)
}
#[tauri::command]
pub fn resume_count_session(
    state: State<'_, DatabaseState>,
    input: Transition,
) -> Result<Detail, AppError> {
    transition(state.db()?, &input, Status::InProgress)
}
#[tauri::command]
pub fn cancel_count_session(
    state: State<'_, DatabaseState>,
    input: Transition,
) -> Result<Detail, AppError> {
    transition(state.db()?, &input, Status::Cancelled)
}
#[tauri::command]
pub fn complete_count_session(
    state: State<'_, DatabaseState>,
    input: Complete,
) -> Result<Detail, AppError> {
    let db = state.db()?;
    let result = complete(db, &input)?;
    crate::inventory::alerts::refresh_all(db);
    Ok(result)
}
