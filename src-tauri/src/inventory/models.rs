use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MovementType {
    Opening,
    PurchaseReceipt,
    Sale,
    SaleReturn,
    PurchaseReturn,
    AdjustmentIn,
    AdjustmentOut,
    Damage,
    Expiry,
    TransferOut,
    TransferIn,
    StockCountCorrection,
}
impl MovementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Opening => "OPENING",
            Self::PurchaseReceipt => "PURCHASE_RECEIPT",
            Self::Sale => "SALE",
            Self::SaleReturn => "SALE_RETURN",
            Self::PurchaseReturn => "PURCHASE_RETURN",
            Self::AdjustmentIn => "ADJUSTMENT_IN",
            Self::AdjustmentOut => "ADJUSTMENT_OUT",
            Self::Damage => "DAMAGE",
            Self::Expiry => "EXPIRY",
            Self::TransferOut => "TRANSFER_OUT",
            Self::TransferIn => "TRANSFER_IN",
            Self::StockCountCorrection => "STOCK_COUNT_CORRECTION",
        }
    }
}
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateBatch {
    pub branch_id: String,
    pub product_package_id: String,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub received_at: Option<String>,
    pub cost_price_minor: Option<i64>,
    pub supplier_reference: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryBatch {
    pub id: String,
    pub branch_id: String,
    pub product_package_id: String,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub received_at: Option<String>,
    pub cost_price_minor: Option<i64>,
    pub supplier_reference: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StockOperation {
    pub request_id: String,
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub quantity: i64,
    pub reason: String,
    pub created_by: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpeningStock {
    pub batch: CreateBatch,
    pub request_id: String,
    pub quantity: i64,
    pub reason: String,
    pub created_by: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Adjustment {
    pub operation: StockOperation,
    pub direction: AdjustmentDirection,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentDirection {
    In,
    Out,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryMovement {
    pub id: String,
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub movement_type: MovementType,
    pub quantity_delta: i64,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub reason: String,
    pub created_by: Option<String>,
    pub transfer_id: Option<String>,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryBalance {
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_id: Option<String>,
    pub quantity: i64,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryQuery {
    pub branch_id: String,
    pub product_package_id: Option<String>,
    pub batch_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryLevel {
    pub id: String,
    pub branch_id: String,
    pub product_package_id: String,
    pub reorder_level: i64,
    pub minimum_stock: Option<i64>,
    pub maximum_stock: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetLevel {
    pub branch_id: String,
    pub product_package_id: String,
    pub reorder_level: i64,
    pub minimum_stock: Option<i64>,
    pub maximum_stock: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LowStockItem {
    pub branch_id: String,
    pub product_package_id: String,
    pub quantity: i64,
    pub reorder_level: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiryItem {
    pub batch: InventoryBatch,
    pub quantity: i64,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpiryQuery {
    pub branch_id: String,
    pub as_of: String,
    pub within_days: i64,
    pub expired_only: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransferStock {
    pub request_id: String,
    pub source_branch_id: String,
    pub destination_branch_id: String,
    pub product_package_id: String,
    pub source_batch_id: String,
    pub quantity: i64,
    pub reason: String,
    pub created_by: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub id: String,
    pub source_batch_id: String,
    pub destination_batch_id: String,
    pub out_movement: InventoryMovement,
    pub in_movement: InventoryMovement,
}
