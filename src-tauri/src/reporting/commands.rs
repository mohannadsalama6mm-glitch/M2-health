use super::{models::ReportFilter, *};
use crate::{commands::branches::DatabaseState, error::AppError};
use serde_json::Value;
use tauri::State;
macro_rules! command {
    ($name:ident,$service:ident) => {
        #[tauri::command]
        pub fn $name(
            state: State<'_, DatabaseState>,
            filter: ReportFilter,
        ) -> Result<Value, AppError> {
            $service(state.db()?, &filter)
        }
    };
}
command!(get_reporting_overview, overview);
command!(get_sales_report, sales_report);
command!(get_sales_by_product_report, sales_by_product);
command!(get_sales_by_customer_report, sales_by_customer);
command!(get_inventory_stock_report, inventory_stock);
command!(get_inventory_movement_report, inventory_movements);
command!(get_expiry_report, expiry_report);
command!(get_low_stock_report, low_stock_report);
command!(get_purchasing_report, purchasing_report);
command!(get_purchase_receipt_report, purchase_receipt_report);
command!(get_purchase_return_report, purchase_return_report);
command!(get_supplier_analytics, supplier_analytics);
