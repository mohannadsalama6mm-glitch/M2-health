use serde::{Deserialize, Serialize};
use super::inventory::StockMovement;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sale {
    pub id: String,
    pub branch_id: String,
    pub sequence: i64,
    pub receipt_number: String,
    pub customer_id: Option<String>,
    pub customer_name: String,
    pub customer_phone: String,
    pub status: String,
    pub subtotal_minor: i64,
    pub discount_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub paid_minor: i64,
    pub change_minor: i64,
    pub payment_method: String,
    pub user: String,
    pub note: String,
    pub void_reason: String,
    pub voided_at: Option<String>,
    pub voided_by: Option<String>,
    pub completed_at: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaleLineInput {
    pub product_package_id: String,
    pub quantity: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteSaleInput {
    pub branch_id: String,
    #[serde(default)]
    pub customer_id: Option<String>,
    #[serde(default)]
    pub customer_name: String,
    #[serde(default)]
    pub customer_phone: String,
    #[serde(default)]
    pub discount_minor: i64,
    #[serde(default)]
    pub tax_minor: i64,
    pub paid_minor: i64,
    #[serde(default)]
    pub payment_method: String,
    pub user: Option<String>,
    #[serde(default)]
    pub note: String,
    pub lines: Vec<SaleLineInput>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteSaleResult {
    pub sale: Sale,
    pub movements: Vec<StockMovement>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItem {
    pub id: String,
    pub sale_id: String,
    pub product_package_id: String,
    pub product_name: String,
    pub package_label: String,
    pub quantity: i64,
    pub selling_price_minor: i64,
    pub cost_price_minor: Option<i64>,
    pub line_total_minor: i64,
    pub line_cost_minor: Option<i64>,
    pub position: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleBatchAllocation {
    pub batch_id: String,
    pub batch_number: String,
    pub expiry_date: String,
    pub quantity: i64,
    pub cost_price_minor: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItemView {
    pub id: String,
    pub product_package_id: String,
    pub product_name: String,
    pub package_label: String,
    pub quantity: i64,
    pub selling_price_minor: i64,
    pub cost_price_minor: Option<i64>,
    pub line_total_minor: i64,
    pub line_cost_minor: Option<i64>,
    pub position: i64,
    pub batches: Vec<SaleBatchAllocation>,
    pub returned_quantity: i64,
    pub returnable_quantity: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalePayment {
    pub id: String,
    pub sale_id: String,
    pub method: String,
    pub amount_minor: i64,
    pub created_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReturnItemInput {
    pub sale_item_id: String,
    pub quantity: i64,
    pub refund_minor: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReturnSaleInput {
    pub sale_id: String,
    pub reason: String,
    pub user: Option<String>,
    pub items: Vec<ReturnItemInput>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleReturnItemView {
    pub id: String,
    pub sale_item_id: String,
    pub product_name: String,
    pub package_label: String,
    pub quantity: i64,
    pub refund_minor: i64,
    pub line_refund_minor: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleReturn {
    pub id: String,
    pub sale_id: String,
    pub branch_id: String,
    pub reason: String,
    pub user: String,
    pub total_refund_minor: i64,
    pub returned_at: String,
    pub items: Vec<SaleReturnItemView>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoidSaleInput {
    pub sale_id: String,
    pub reason: String,
    pub user: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleDetail {
    pub sale: Sale,
    pub branch_name: String,
    pub items: Vec<SaleItemView>,
    pub returns: Vec<SaleReturn>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleRow {
    pub id: String,
    pub branch_id: String,
    pub branch_name: String,
    pub sequence: i64,
    pub receipt_number: String,
    pub customer_name: String,
    pub status: String,
    pub total_minor: i64,
    pub paid_minor: i64,
    pub payment_method: String,
    pub item_count: i64,
    pub user: String,
    pub completed_at: String,
    pub void_reason: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalePage {
    pub items: Vec<SaleRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaleQuery {
    pub branch_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
/// A sellable package matched by the POS lookup: name, package, barcode or batch.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PosProduct {
    pub package_id: String,
    pub product_id: String,
    pub product_name: String,
    pub package_label: String,
    pub pack_size: Option<String>,
    pub selling_price_minor: Option<i64>,
    pub quantity: i64,
    pub status: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PosProductPage {
    pub items: Vec<PosProduct>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PosQuery {
    pub branch_id: String,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}