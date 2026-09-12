use super::branches::DatabaseState;
use crate::{
    db::{models::catalog::*, repositories::*},
    error::AppError,
};
use tauri::State;
#[tauri::command]
pub fn create_manufacturer(
    state: State<'_, DatabaseState>,
    input: CreateManufacturer,
) -> Result<Manufacturer, AppError> {
    manufacturers::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_manufacturers(
    state: State<'_, DatabaseState>,
    include_inactive: Option<bool>,
) -> Result<Vec<Manufacturer>, AppError> {
    manufacturers::list(state.db()?, include_inactive.unwrap_or(false))
}
#[tauri::command]
pub fn set_manufacturer_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<Manufacturer, AppError> {
    manufacturers::set_active(state.db()?, &id, active)
}
#[tauri::command]
pub fn create_category(
    state: State<'_, DatabaseState>,
    input: CreateCategory,
) -> Result<Category, AppError> {
    categories::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_categories(
    state: State<'_, DatabaseState>,
    include_inactive: Option<bool>,
) -> Result<Vec<Category>, AppError> {
    categories::list(state.db()?, include_inactive.unwrap_or(false))
}
#[tauri::command]
pub fn set_category_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<Category, AppError> {
    categories::set_active(state.db()?, &id, active)
}
#[tauri::command]
pub fn create_route(
    state: State<'_, DatabaseState>,
    input: CreateRoute,
) -> Result<Route, AppError> {
    routes::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_routes(
    state: State<'_, DatabaseState>,
    include_inactive: Option<bool>,
) -> Result<Vec<Route>, AppError> {
    routes::list(state.db()?, include_inactive.unwrap_or(false))
}
#[tauri::command]
pub fn set_route_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<Route, AppError> {
    routes::set_active(state.db()?, &id, active)
}
#[tauri::command]
pub fn create_active_ingredient(
    state: State<'_, DatabaseState>,
    input: CreateActiveIngredient,
) -> Result<ActiveIngredient, AppError> {
    active_ingredients::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_active_ingredients(
    state: State<'_, DatabaseState>,
    include_inactive: Option<bool>,
) -> Result<Vec<ActiveIngredient>, AppError> {
    active_ingredients::list(state.db()?, include_inactive.unwrap_or(false))
}
#[tauri::command]
pub fn set_active_ingredient_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<ActiveIngredient, AppError> {
    active_ingredients::set_active(state.db()?, &id, active)
}
#[tauri::command]
pub fn create_product(
    state: State<'_, DatabaseState>,
    input: CreateProduct,
) -> Result<Product, AppError> {
    products::create(state.db()?, &input)
}
#[tauri::command]
pub fn get_product(state: State<'_, DatabaseState>, id: String) -> Result<Product, AppError> {
    products::get_product(state.db()?, &id)
}
#[tauri::command]
pub fn get_product_detail(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<ProductDetail, AppError> {
    products::detail(state.db()?, &id)
}
#[tauri::command]
pub fn list_products(
    state: State<'_, DatabaseState>,
    query: ProductQuery,
) -> Result<ProductPage, AppError> {
    products::list(state.db()?, &query)
}
#[tauri::command]
pub fn set_product_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<Product, AppError> {
    products::set_active(state.db()?, &id, active)
}
#[tauri::command]
pub fn link_product_ingredient(
    state: State<'_, DatabaseState>,
    input: LinkProductIngredient,
) -> Result<Vec<ProductIngredient>, AppError> {
    product_ingredients::link(state.db()?, &input)
}
#[tauri::command]
pub fn create_product_package(
    state: State<'_, DatabaseState>,
    input: CreateProductPackage,
) -> Result<ProductPackage, AppError> {
    product_packages::create(state.db()?, &input)
}
#[tauri::command]
pub fn list_product_packages(
    state: State<'_, DatabaseState>,
    product_id: String,
    include_inactive: Option<bool>,
) -> Result<Vec<ProductPackage>, AppError> {
    product_packages::list(state.db()?, &product_id, include_inactive.unwrap_or(false))
}
#[tauri::command]
pub fn set_product_package_active(
    state: State<'_, DatabaseState>,
    id: String,
    active: bool,
) -> Result<ProductPackage, AppError> {
    product_packages::set_active(state.db()?, &id, active)
}
#[tauri::command]
pub fn add_barcode(
    state: State<'_, DatabaseState>,
    input: AddBarcode,
) -> Result<Barcode, AppError> {
    barcodes::add(state.db()?, &input)
}
#[tauri::command]
pub fn list_package_barcodes(
    state: State<'_, DatabaseState>,
    package_id: String,
) -> Result<Vec<Barcode>, AppError> {
    barcodes::list(state.db()?, &package_id)
}
#[tauri::command]
pub fn set_package_price(
    state: State<'_, DatabaseState>,
    input: SetPackagePrice,
) -> Result<PackagePrice, AppError> {
    price_history::set(state.db()?, &input)
}
#[tauri::command]
pub fn get_current_package_price(
    state: State<'_, DatabaseState>,
    package_id: String,
) -> Result<Option<PackagePrice>, AppError> {
    price_history::current(state.db()?, &package_id)
}
#[tauri::command]
pub fn get_package_price_history(
    state: State<'_, DatabaseState>,
    package_id: String,
) -> Result<Vec<PackagePrice>, AppError> {
    price_history::history(state.db()?, &package_id)
}
