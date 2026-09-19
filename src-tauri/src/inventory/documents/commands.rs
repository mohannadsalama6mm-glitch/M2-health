use super::{models::*, *};
use crate::commands::branches::DatabaseState;
use tauri::State;
#[tauri::command]
pub fn save_inventory_document(
    state: State<'_, DatabaseState>,
    input: SaveDocument,
) -> Result<Document, AppError> {
    save(state.db()?, &input)
}
#[tauri::command]
pub fn get_inventory_document(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Document, AppError> {
    get(state.db()?, &id)
}
#[tauri::command]
pub fn list_inventory_documents(
    state: State<'_, DatabaseState>,
    query: DocumentQuery,
) -> Result<DocumentPage, AppError> {
    list(state.db()?, &query)
}
#[tauri::command]
pub fn mark_inventory_document_ready(
    state: State<'_, DatabaseState>,
    input: Transition,
) -> Result<Document, AppError> {
    ready(state.db()?, &input)
}
#[tauri::command]
pub fn cancel_inventory_document(
    state: State<'_, DatabaseState>,
    input: Transition,
) -> Result<Document, AppError> {
    cancel(state.db()?, &input)
}
#[tauri::command]
pub fn post_inventory_document(
    state: State<'_, DatabaseState>,
    input: PostDocument,
) -> Result<Document, AppError> {
    let db = state.db()?;
    let result = post(db, &input)?;
    crate::inventory::alerts::refresh_all(db);
    Ok(result)
}
#[tauri::command]
pub fn reverse_inventory_document(
    state: State<'_, DatabaseState>,
    input: ReverseDocument,
) -> Result<Document, AppError> {
    let db = state.db()?;
    let result = reverse(db, &input)?;
    crate::inventory::alerts::refresh_all(db);
    Ok(result)
}
#[tauri::command]
pub fn list_workflow_batches(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
    offset: Option<i64>,
) -> Result<Vec<BatchChoice>, AppError> {
    batch_choices(
        state.db()?,
        &branch_id,
        &product_package_id,
        offset.unwrap_or(0),
    )
}
