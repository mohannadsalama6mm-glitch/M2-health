use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseLineInput {
    pub product_package_id: String,
    pub ordered_quantity: i64,
    pub unit_cost_minor: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseDraftFields {
    pub supplier_id: String,
    pub supplier_reference: Option<String>,
    pub expected_date: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub discount_minor: i64,
    #[serde(default)]
    pub tax_minor: i64,
    pub lines: Vec<PurchaseLineInput>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePurchaseDraft {
    pub id: String,
    pub request_id: String,
    #[serde(flatten)]
    pub fields: PurchaseDraftFields,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdatePurchaseDraft {
    pub id: String,
    pub expected_revision: i64,
    #[serde(flatten)]
    pub fields: PurchaseDraftFields,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseAction {
    pub id: String,
    pub expected_revision: i64,
    pub request_id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelPurchase {
    pub id: String,
    pub expected_revision: i64,
    pub request_id: String,
    pub reason: String,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseQuery {
    pub search: Option<String>,
    pub supplier_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseLine {
    pub id: String,
    pub product_id: String,
    pub product_package_id: String,
    pub product_name_snapshot: String,
    pub package_label_snapshot: String,
    pub ordered_quantity: i64,
    pub unit_cost_minor: i64,
    pub line_total_minor: i64,
    pub position: i64,
    pub received_quantity: i64,
    pub remaining_quantity: i64,
    pub returned_quantity: i64,
    pub remaining_returnable_quantity: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrder {
    pub id: String,
    pub purchase_number: String,
    pub supplier_id: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub status: String,
    pub revision: i64,
    pub supplier_reference: Option<String>,
    pub expected_date: Option<String>,
    pub notes: Option<String>,
    pub subtotal_minor: i64,
    pub discount_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub created_at: String,
    pub updated_at: String,
    pub ordered_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub cancellation_reason: Option<String>,
    pub lines: Vec<PurchaseLine>,
    pub receipts: Vec<PurchaseReceiptSummary>,
    pub returns: Vec<PurchaseReturnSummary>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRow {
    pub id: String,
    pub purchase_number: String,
    pub supplier_id: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub status: String,
    pub revision: i64,
    pub total_minor: i64,
    pub line_count: i64,
    pub created_at: String,
    pub ordered_at: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchasePage {
    pub items: Vec<PurchaseRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchasePackage {
    pub product_id: String,
    pub product_package_id: String,
    pub product_name: String,
    pub package_label: String,
    pub manufacturer_name: Option<String>,
    pub barcode: Option<String>,
    pub current_cost_minor: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackagePage {
    pub items: Vec<PurchasePackage>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseReceiptLineInput {
    pub purchase_order_line_id: String,
    pub quantity: i64,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub acquisition_cost_minor: Option<i64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceivePurchase {
    pub id: String,
    pub purchase_order_id: String,
    pub expected_revision: i64,
    pub request_id: String,
    pub branch_id: String,
    pub created_by: Option<String>,
    pub lines: Vec<PurchaseReceiptLineInput>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReceiptLine {
    pub id: String,
    pub purchase_order_line_id: String,
    pub product_package_id: String,
    pub product_name_snapshot: String,
    pub package_label_snapshot: String,
    pub quantity: i64,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub acquisition_cost_minor: i64,
    pub batch_id: String,
    pub movement_id: String,
    pub position: i64,
    pub returned_quantity: i64,
    pub remaining_returnable_quantity: i64,
    pub current_batch_quantity: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReceipt {
    pub id: String,
    pub receipt_number: String,
    pub purchase_order_id: String,
    pub purchase_number: String,
    pub branch_id: String,
    pub branch_code: String,
    pub branch_name: String,
    pub supplier_id: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub total_quantity: i64,
    pub created_by: Option<String>,
    pub posted_at: String,
    pub created_at: String,
    pub lines: Vec<PurchaseReceiptLine>,
    pub returns: Vec<PurchaseReturnSummary>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReceiptSummary {
    pub id: String,
    pub receipt_number: String,
    pub branch_id: String,
    pub branch_code: String,
    pub branch_name: String,
    pub total_quantity: i64,
    pub posted_at: String,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseReceiptQuery {
    pub purchase_order_id: Option<String>,
    pub supplier_id: Option<String>,
    pub branch_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReceiptPage {
    pub items: Vec<PurchaseReceiptSummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseReturnLineInput {
    pub receipt_line_id: String,
    pub quantity: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePurchaseReturn {
    pub id: String,
    pub purchase_order_id: String,
    pub receipt_id: String,
    pub expected_revision: i64,
    pub request_id: String,
    pub reason: String,
    pub created_by: Option<String>,
    pub lines: Vec<PurchaseReturnLineInput>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnLine {
    pub id: String,
    pub receipt_line_id: String,
    pub purchase_order_line_id: String,
    pub product_package_id: String,
    pub product_name_snapshot: String,
    pub package_label_snapshot: String,
    pub received_quantity: i64,
    pub previously_returned_quantity: i64,
    pub remaining_returnable_quantity: i64,
    pub quantity: i64,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub acquisition_cost_minor: i64,
    pub batch_id: String,
    pub movement_id: String,
    pub position: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturn {
    pub id: String,
    pub return_number: String,
    pub purchase_order_id: String,
    pub purchase_number: String,
    pub receipt_id: String,
    pub receipt_number: String,
    pub branch_id: String,
    pub branch_code: String,
    pub branch_name: String,
    pub supplier_id: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub source_purchase_revision: i64,
    pub reason: String,
    pub total_quantity: i64,
    pub created_by: Option<String>,
    pub posted_at: String,
    pub created_at: String,
    pub lines: Vec<PurchaseReturnLine>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnSummary {
    pub id: String,
    pub return_number: String,
    pub purchase_order_id: String,
    pub purchase_number: String,
    pub receipt_id: String,
    pub receipt_number: String,
    pub branch_id: String,
    pub branch_code: String,
    pub branch_name: String,
    pub supplier_id: String,
    pub supplier_code: String,
    pub supplier_name: String,
    pub total_quantity: i64,
    pub reason: String,
    pub posted_at: String,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurchaseReturnQuery {
    pub purchase_order_id: Option<String>,
    pub supplier_id: Option<String>,
    pub receipt_id: Option<String>,
    pub branch_id: Option<String>,
    pub search: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseReturnPage {
    pub items: Vec<PurchaseReturnSummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
