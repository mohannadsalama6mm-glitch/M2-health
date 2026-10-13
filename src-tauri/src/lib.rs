pub mod auth;
pub mod authorization;
pub mod catalog_import;
pub mod cloud;
pub mod commands;
pub mod db;
pub mod error;
pub mod finance;
pub mod inventory;
pub mod partners;
pub mod phase6;
pub mod purchasing;
pub mod recovery;
pub mod reporting;
pub mod sales;
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
    inventory::alerts::refresh_all(&db);
    Ok(db)
}
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            app.manage(DatabaseState(initialize(app)));
            app.manage(auth::AuthState::default());
            app.manage(authorization::AuthorizationState::default());
            app.manage(cloud::config::CloudConfigState::default());
            app.manage(cloud::transport::CloudAuthState::default());
            app.manage(catalog_import::commands::ImportState::default());
            Ok(())
        })
        .invoke_handler(authorization::guard(tauri::generate_handler![
            auth::commands::get_auth_status,
            auth::commands::bootstrap_admin,
            auth::commands::login,
            auth::commands::current_session,
            auth::commands::logout,
            cloud::commands::get_cloud_foundation_status,
            cloud::commands::get_cloud_session_status,
            cloud::commands::cloud_sign_in,
            cloud::commands::cloud_sign_out,
            cloud::commands::run_cloud_sync_cycle,
            cloud::commands::list_sync_conflicts,
            cloud::commands::resolve_sync_conflict,
            auth::commands::list_users,
            auth::commands::get_user,
            auth::commands::create_user,
            auth::commands::update_user,
            auth::commands::set_user_active,
            auth::commands::reset_user_password,
            auth::commands::list_roles,
            auth::commands::assign_user_role,
            auth::commands::remove_user_role,
            auth::commands::set_user_branches,
            phase6::commands::get_phase6_dashboard,
            phase6::commands::get_phase6_reconciliation,
            phase6::commands::get_sale_finance_trace,
            phase6::commands::export_phase6_csv,
            finance::commands::list_finance_accounts,
            finance::commands::create_finance_expense,
            finance::commands::get_finance_expense,
            finance::commands::list_finance_expenses,
            finance::commands::get_finance_journal,
            finance::commands::list_finance_journals,
            finance::commands::get_trial_balance,
            finance::commands::get_account_activity,
            finance::commands::get_sales_financial_summary,
            finance::commands::get_expense_report,
            reporting::commands::get_reporting_overview,
            reporting::commands::get_sales_report,
            reporting::commands::get_sales_by_product_report,
            reporting::commands::get_sales_by_customer_report,
            reporting::commands::get_inventory_stock_report,
            reporting::commands::get_inventory_movement_report,
            reporting::commands::get_expiry_report,
            reporting::commands::get_low_stock_report,
            reporting::commands::get_purchasing_report,
            reporting::commands::get_purchase_receipt_report,
            reporting::commands::get_purchase_return_report,
            reporting::commands::get_supplier_analytics,
            purchasing::commands::create_purchase_order_draft,
            purchasing::commands::update_purchase_order_draft,
            purchasing::commands::order_purchase_order,
            purchasing::commands::cancel_purchase_order,
            purchasing::commands::get_purchase_order,
            purchasing::commands::list_purchase_orders,
            purchasing::commands::search_purchase_packages,
            purchasing::commands::receive_purchase_order,
            purchasing::commands::get_purchase_receipt,
            purchasing::commands::list_purchase_receipts,
            purchasing::commands::create_purchase_return,
            purchasing::commands::get_purchase_return,
            purchasing::commands::list_purchase_returns,
            partners::commands::create_canonical_supplier,
            partners::commands::get_canonical_supplier,
            partners::commands::update_canonical_supplier,
            partners::commands::set_canonical_supplier_active,
            partners::commands::list_canonical_suppliers,
            partners::commands::create_canonical_customer,
            partners::commands::get_canonical_customer,
            partners::commands::update_canonical_customer,
            partners::commands::set_canonical_customer_active,
            partners::commands::list_canonical_customers,
            sales::commands::search_pos_products,
            sales::commands::get_pos_package_availability,
            sales::commands::save_sale_draft,
            sales::commands::hold_sale,
            sales::commands::resume_sale,
            sales::commands::preview_sale_totals,
            sales::commands::post_sale,
            sales::commands::list_canonical_sales,
            sales::commands::list_customer_sales,
            sales::commands::get_canonical_sale,
            sales::commands::preview_sale_return,
            sales::commands::create_sale_return,
            sales::commands::post_sale_return,
            sales::commands::void_canonical_sale,
            inventory::inspection::inspect_inventory_overview,
            inventory::inspection::inspect_inventory_batches,
            inventory::inspection::inspect_inventory_movements,
            recovery::recovery_status,
            inventory::alerts::commands::get_inventory_alert_settings,
            inventory::alerts::commands::update_inventory_alert_settings,
            inventory::alerts::commands::evaluate_inventory_alerts,
            inventory::alerts::commands::list_inventory_alerts,
            inventory::alerts::commands::get_inventory_alert,
            inventory::alerts::commands::dismiss_inventory_alert,
            inventory::alerts::commands::list_expiry_inventory,
            inventory::alerts::commands::list_low_stock_inventory,
            inventory::alerts::commands::get_fefo_inventory_batches,
            inventory::counts::commands::create_count_session,
            inventory::counts::commands::get_count_session,
            inventory::counts::commands::list_count_sessions,
            inventory::counts::commands::start_count_session,
            inventory::counts::commands::record_count_entry,
            inventory::counts::commands::discover_count_stock,
            inventory::counts::commands::review_count_session,
            inventory::counts::commands::resume_count_session,
            inventory::counts::commands::cancel_count_session,
            inventory::counts::commands::complete_count_session,
            inventory::documents::commands::save_inventory_document,
            inventory::documents::commands::get_inventory_document,
            inventory::documents::commands::list_inventory_documents,
            inventory::documents::commands::mark_inventory_document_ready,
            inventory::documents::commands::cancel_inventory_document,
            inventory::documents::commands::post_inventory_document,
            inventory::documents::commands::reverse_inventory_document,
            inventory::documents::commands::list_workflow_batches,
            recovery::fefo_allocation,
            inventory::commands::create_inventory_batch,
            inventory::commands::record_opening_stock,
            inventory::commands::record_batch_opening_stock,
            inventory::commands::record_adjustment,
            inventory::commands::record_damage,
            inventory::commands::record_expiry_write_off,
            inventory::commands::transfer_inventory_stock,
            inventory::commands::get_package_stock,
            inventory::commands::get_batch_stock,
            inventory::commands::list_branch_inventory,
            inventory::commands::list_stock_movements,
            inventory::commands::set_inventory_level,
            inventory::commands::list_low_stock,
            inventory::commands::list_expiring_batches,
            inventory::commands::get_fefo_batches,
            catalog_import::commands::profile_catalog_source,
            catalog_import::commands::dry_run_catalog_import,
            catalog_import::commands::apply_catalog_import,
            catalog_import::commands::get_catalog_import_status,
            catalog_import::commands::get_catalog_import_report,
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
        ]))
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("M² Health could not start: {error}");
    }
}
