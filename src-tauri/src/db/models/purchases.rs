use serde::{Deserialize, Serialize};
use super::inventory::StockMovement;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub id: String,
    pub branch_id: String,
    pub supplier_id: Option<String>,
    pub sequence: i64,
    pub purchase_number: String,
    pub invoice_number: String,
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
pub struct PurchaseLineInput {
    pub product_package_id: String,
    pub quantity: i64,
    pub unit_cost_minor: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletePurchaseInput {
    pub branch_id: String,
    pub supplier_id: Option<String>,
    #[serde(default)]
    pub invoice_number: String,
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
    pub lines: Vec<PurchaseLineInput>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletePurchaseResult {
    pub purchase: Purchase,
    pub movements: Vec<StockMovement>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseItem {
    pub id: String,
    pub purchase_id: String,
    pub product_package_id: String,
    pub product_name: String,
    pub package_label: String,
    pub quantity: i64,
    pub unit_cost_minor: i64,
    pub line_total_minor: i64,
    pub position: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseBatchAllocation {
    pub id: String,
    pub purchase_id: String,
    pub purchase_item_id: String,
    pub batch_id: String,
    pub quantity: i64,
    pub unit_cost_minor: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseItemView {
    pub id: String,
    pub product_package_id: String,
    pub product_name: String,
    pub package_label: String,
    pub quantity: i64,
    pub unit_cost_minor: i64,
    pub line_total_minor: i64,
    pub position: i64,
    pub batches: Vec<PurchaseBatchAllocationView>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseBatchAllocationView {
    pub id: String,
    pub batch_id: String,
    pub quantity: i64,
    pub unit_cost_minor: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchasePayment {
    pub id: String,
    pub purchase_id: String,
    pub method: String,
    pub amount_minor: i64,
    pub created_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoidPurchaseInput {
    pub purchase_id: String,
    pub reason: String,
    pub user: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseDetail {
    pub purchase: Purchase,
    pub branch_name: String,
    pub supplier_name: Option<String>,
    pub items: Vec<PurchaseItemView>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRow {
    pub id: String,
    pub branch_id: String,
    pub branch_name: String,
    pub supplier_name: Option<String>,
    pub sequence: i64,
    pub purchase_number: String,
    pub invoice_number: String,
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
pub struct PurchasePage {
    pub items: Vec<PurchaseRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseQuery {
    pub branch_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseProduct {
    pub package_id: String,
    pub product_id: String,
    pub product_name: String,
    pub package_label: String,
    pub pack_size: Option<String>,
    pub cost_price_minor: Option<i64>,
    pub status: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseProductPage {
    pub items: Vec<PurchaseProduct>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseNumberQuery {
    pub branch_id: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PosPurchaseQuery {
    pub branch_id: String,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
