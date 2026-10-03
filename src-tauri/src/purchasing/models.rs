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
