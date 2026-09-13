pub mod commands;
pub mod db;
pub mod error;
use commands::branches::DatabaseState;
use db::{connection::AppDb, repositories::branches};
use error::{AppError, ErrorCode};
use tauri::Manager;

fn initialize(app: &tauri::App) -> Result<AppDb, AppError> {
    let directory = app.path().app_local_data_dir().map_err(|e| {
        eprintln!("[app data] {e}");
        AppError::new(
            ErrorCode::Io,
            "The local application data directory could not be resolved.",
        )
    })?;
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("m2-health.db");
    #[cfg(debug_assertions)]
    eprintln!("[M² Health] Local SQLite database: {}", path.display());
    let db = AppDb::open(&path)?;
    let branch = branches::ensure_default_branch(&db)?;
    #[cfg(debug_assertions)]
    eprintln!(
        "[M² Health] Schema ready; local branch {} ({})",
        branch.code, branch.id
    );
    Ok(db)
}
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            app.manage(DatabaseState(initialize(app)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::branches::ensure_default_branch,
            commands::branches::list_branches,
            commands::branches::get_branch,
            commands::branches::create_branch,
            commands::catalog::create_manufacturer,
            commands::catalog::list_manufacturers,
            commands::catalog::set_manufacturer_active,
            commands::catalog::create_category,
            commands::catalog::list_categories,
            commands::catalog::set_category_active,
            commands::catalog::create_route,
            commands::catalog::list_routes,
            commands::catalog::set_route_active,
            commands::catalog::create_active_ingredient,
            commands::catalog::list_active_ingredients,
            commands::catalog::set_active_ingredient_active,
            commands::catalog::create_product,
            commands::catalog::create_product_full,
            commands::catalog::update_product_full,
            commands::catalog::get_product,
            commands::catalog::get_product_detail,
            commands::catalog::list_products,
            commands::catalog::set_product_active,
            commands::catalog::link_product_ingredient,
            commands::catalog::create_product_package,
            commands::catalog::list_product_packages,
            commands::catalog::set_product_package_active,
            commands::catalog::add_barcode,
            commands::catalog::list_package_barcodes,
            commands::catalog::set_package_price,
            commands::catalog::get_current_package_price,
            commands::catalog::get_package_price_history,
            commands::customers::create_customer,
            commands::customers::list_customers,
            commands::customers::get_customer,
            commands::customers::update_customer,
            commands::customers::set_customer_active,
            commands::suppliers::create_supplier,
            commands::suppliers::list_suppliers,
            commands::suppliers::get_supplier,
            commands::suppliers::update_supplier,
            commands::suppliers::set_supplier_active,
            commands::inventory::get_stock_overview,
            commands::inventory::get_stock_summary,
            commands::inventory::list_stock_movements,
            commands::inventory::post_opening_stock,
            commands::inventory::adjust_stock,
            commands::inventory::write_off_stock,
            commands::inventory::transfer_stock,
            commands::inventory::list_stock_counts,
            commands::inventory::get_stock_count,
            commands::inventory::create_stock_count,
            commands::inventory::save_count_item,
            commands::inventory::complete_stock_count,
            commands::inventory::list_expiry,
            commands::inventory::list_low_stock,
            commands::inventory::set_reorder_level,
            commands::inventory::fefo_allocation,
            commands::sales::complete_sale,
            commands::sales::return_sale,
            commands::sales::void_sale,
            commands::sales::list_sales,
            commands::sales::get_sale,
            commands::sales::pos_search,
            commands::purchases::complete_purchase,
            commands::purchases::void_purchase,
            commands::purchases::list_purchases,
            commands::purchases::get_purchase,
            commands::purchases::purchase_pos_search,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("M² Health could not start: {error}");
    }
}
