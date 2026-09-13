use serde::{Deserialize, Serialize};

/// One branch-scoped lot of a ProductPackage. Quantity is never stored here; it is
/// derived from the stock_movements ledger (stock_balances view).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryBatch {
    pub id: String,
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_number: String,
    pub expiry_date: String,
    pub cost_price_minor: Option<i64>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpeningStockInput {
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_number: String,
    #[serde(default)]
    pub expiry_date: String,
    pub cost_price_minor: Option<i64>,
    pub quantity: i64,
    #[serde(default)]
    pub reason: String,
    pub user: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdjustStockInput {
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_id: Option<String>,
    pub batch_number: Option<String>,
    pub expiry_date: Option<String>,
    pub new_quantity: i64,
    pub reason: String,
    pub user: Option<String>,
}
/// Outbound inventory event. `kind` is one of damage/expired/supplier_return.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriteOffInput {
    pub batch_id: String,
    pub kind: String,
    pub quantity: i64,
    pub reason: String,
    pub user: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransferInput {
    pub from_branch_id: String,
    pub to_branch_id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub quantity: i64,
    #[serde(default)]
    pub reason: String,
    pub user: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetReorderInput {
    pub branch_id: String,
    pub product_package_id: String,
    pub reorder_level: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCountInput {
    pub branch_id: String,
    #[serde(default)]
    pub scope: String,
    pub category_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveCountItemInput {
    pub stock_count_id: String,
    pub batch_id: String,
    pub counted_quantity: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteCountInput {
    pub stock_count_id: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovement {
    pub id: String,
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub movement_type: String,
    pub quantity_delta: i64,
    pub cost_price_minor: Option<i64>,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub reason: String,
    pub user: String,
    pub occurred_at: String,
    pub created_at: String,
}
/// Enriched ledger row for the movements grid.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovementItem {
    pub id: String,
    pub occurred_at: String,
    pub movement_type: String,
    pub product: String,
    pub pack: Option<String>,
    pub batch: Option<String>,
    pub branch: String,
    pub incoming: Option<i64>,
    pub outgoing: Option<i64>,
    pub user: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovementPage {
    pub items: Vec<StockMovementItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MovementQuery {
    pub branch_id: Option<String>,
    pub package_id: Option<String>,
    pub batch_id: Option<String>,
    pub movement_type: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchMini {
    pub batch_id: String,
    pub batch_number: String,
    pub expiry_date: String,
    pub quantity: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockRow {
    pub package_id: String,
    pub product_id: String,
    pub name_en: Option<String>,
    pub name_ar: Option<String>,
    pub scientific_name: Option<String>,
    pub manufacturer_name: Option<String>,
    pub package_label: String,
    pub pack_size: Option<String>,
    pub quantity: i64,
    pub reorder_level: i64,
    pub batch_count: i64,
    pub batches: Vec<BatchMini>,
    pub min_expiry_days: Option<i64>,
    pub selling_price_minor: Option<i64>,
    pub cost_price_minor: Option<i64>,
    pub status: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StockOverviewQuery {
    pub branch_id: String,
    pub search: Option<String>,
    pub manufacturer_id: Option<String>,
    pub category_id: Option<String>,
    pub status: Option<String>,
    pub sort: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockOverviewPage {
    pub items: Vec<StockRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockSummary {
    pub in_stock: i64,
    pub low: i64,
    pub out_of_stock: i64,
    pub value_minor: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiryRow {
    pub batch_id: String,
    pub package_id: String,
    pub product_id: String,
    pub name_en: Option<String>,
    pub name_ar: Option<String>,
    pub scientific_name: Option<String>,
    pub package_label: String,
    pub batch_number: String,
    pub expiry_date: String,
    pub days: i64,
    pub quantity: i64,
    pub value_minor: i64,
    pub cost_price_minor: Option<i64>,
    pub branch_name: String,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpiryQuery {
    pub branch_id: String,
    pub window_days: Option<i64>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiryPage {
    pub items: Vec<ExpiryRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LowStockRow {
    pub package_id: String,
    pub product_id: String,
    pub name_en: Option<String>,
    pub name_ar: Option<String>,
    pub package_label: String,
    pub quantity: i64,
    pub reorder_level: i64,
    pub status: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LowStockPage {
    pub items: Vec<LowStockRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LowStockQuery {
    pub branch_id: String,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FefoAllocation {
    pub batch_id: String,
    pub batch_number: String,
    pub expiry_date: String,
    pub available: i64,
    pub take: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCount {
    pub id: String,
    pub branch_id: String,
    pub scope: String,
    pub category_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCountItem {
    pub id: String,
    pub stock_count_id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub system_quantity: i64,
    pub counted_quantity: i64,
    pub variance: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCountItemView {
    pub id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub product: String,
    pub pack: Option<String>,
    pub batch_number: String,
    pub system_quantity: i64,
    pub counted_quantity: i64,
    pub variance: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCountDetail {
    pub count: StockCount,
    pub branch_name: String,
    pub items: Vec<StockCountItemView>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCountRow {
    pub id: String,
    pub branch_name: String,
    pub scope: String,
    pub category_name: Option<String>,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub item_count: i64,
    pub discrepancy_count: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCountPage {
    pub items: Vec<StockCountRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CountQuery {
    pub branch_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySetting {
    pub branch_id: String,
    pub product_package_id: String,
    pub reorder_level: i64,
    pub created_at: String,
    pub updated_at: String,
}
pub const MOVEMENT_TYPES: &[&str] = &[
    "opening",
    "purchase",
    "sale",
    "customer_return",
    "supplier_return",
    "adjustment",
    "damage",
    "expired",
    "transfer_out",
    "transfer_in",
    "count_correction",
];
/// Result of a controlled adjustment. A no-op adjustment posts no movements.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustResult {
    pub new_quantity: i64,
    pub movements: Vec<StockMovement>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub out: StockMovement,
    pub incoming: StockMovement,
    pub from_batch_id: String,
    pub to_batch_id: String,
}
