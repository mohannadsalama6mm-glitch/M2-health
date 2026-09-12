use crate::{
    db::{
        connection::AppDb,
        models::branch::{Branch, CreateBranch},
        repositories::branches,
    },
    error::AppError,
};
use tauri::State;

/// Retain initialization failure as managed state so the existing UI can load and
/// report a safe error instead of terminating the native window.
pub struct DatabaseState(pub Result<AppDb, AppError>);
impl DatabaseState {
    pub(crate) fn db(&self) -> Result<&AppDb, AppError> {
        self.0.as_ref().map_err(Clone::clone)
    }
}
#[tauri::command]
pub fn ensure_default_branch(state: State<'_, DatabaseState>) -> Result<Branch, AppError> {
    branches::ensure_default_branch(state.db()?)
}
#[tauri::command]
pub fn list_branches(state: State<'_, DatabaseState>) -> Result<Vec<Branch>, AppError> {
    branches::list_branches(state.db()?)
}
#[tauri::command]
pub fn get_branch(state: State<'_, DatabaseState>, id: String) -> Result<Branch, AppError> {
    branches::get_branch(state.db()?, &id)
}
#[tauri::command]
pub fn create_branch(
    state: State<'_, DatabaseState>,
    input: CreateBranch,
) -> Result<Branch, AppError> {
    branches::create_branch(state.db()?, &input)
}
