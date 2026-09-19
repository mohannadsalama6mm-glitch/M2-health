use super::{models::*, *};
use crate::commands::branches::DatabaseState;
use tauri::State;
#[tauri::command]
pub fn get_pos_package_availability(
    state: State<'_, DatabaseState>,
    branch_id: String,
    product_package_id: String,
) -> Result<Value, AppError> {
    let c = state.db()?.lock()?;
    let quantity = money(
        eligible(&c, &branch_id, &product_package_id)?
            .iter()
            .map(|r| r.quantity as i128)
            .sum(),
    )?;
    let price = crate::db::repositories::price_history::current_on(&c, &product_package_id)?;
    Ok(
        json!({"branchId":branch_id,"productPackageId":product_package_id,"availableQuantity":quantity,"currentPriceMinor":price.map(|p|p.selling_price_minor),"stockStatus":if quantity==0{"OUT_OF_STOCK"}else{"IN_STOCK"}}),
    )
}
#[tauri::command]
pub fn search_pos_products(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Value, AppError> {
    search(state.db()?, &query)
}
#[tauri::command]
pub fn save_sale_draft(state: State<'_, DatabaseState>, input: Save) -> Result<Value, AppError> {
    save(state.db()?, &input)
}
#[tauri::command]
pub fn hold_sale(state: State<'_, DatabaseState>, input: Transition) -> Result<Value, AppError> {
    transition(state.db()?, &input, true)
}
#[tauri::command]
pub fn resume_sale(state: State<'_, DatabaseState>, input: Transition) -> Result<Value, AppError> {
    transition(state.db()?, &input, false)
}
#[tauri::command]
pub fn preview_sale_totals(
    state: State<'_, DatabaseState>,
    cart: Cart,
) -> Result<Preview, AppError> {
    preview(state.db()?, &cart)
}
#[tauri::command]
pub fn post_sale(state: State<'_, DatabaseState>, input: Post) -> Result<Value, AppError> {
    post(state.db()?, &input)
}
#[tauri::command]
pub fn list_canonical_sales(
    state: State<'_, DatabaseState>,
    query: Query,
) -> Result<Value, AppError> {
    list(state.db()?, &query)
}
#[tauri::command]
pub fn get_canonical_sale(state: State<'_, DatabaseState>, id: String) -> Result<Value, AppError> {
    detail(state.db()?, &id)
}
#[tauri::command]
pub fn preview_sale_return(
    state: State<'_, DatabaseState>,
    input: ReturnDraft,
) -> Result<Value, AppError> {
    preview_return(state.db()?, &input)
}
#[tauri::command]
pub fn create_sale_return(
    state: State<'_, DatabaseState>,
    input: ReturnDraft,
) -> Result<Value, AppError> {
    create_return(state.db()?, &input)
}
#[tauri::command]
pub fn post_sale_return(
    state: State<'_, DatabaseState>,
    input: ReturnPost,
) -> Result<Value, AppError> {
    post_return(state.db()?, &input)
}
#[tauri::command]
pub fn void_canonical_sale(
    state: State<'_, DatabaseState>,
    input: Void,
) -> Result<Value, AppError> {
    void(state.db()?, &input)
}
