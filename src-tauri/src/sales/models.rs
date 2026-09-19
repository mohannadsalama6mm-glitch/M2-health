use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cart {
    pub branch_id: String,
    #[serde(default)]
    pub customer_name: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub lines: Vec<Line>,
    #[serde(default)]
    pub discount_minor: i64,
    #[serde(default)]
    pub tax_minor: i64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Line {
    pub product_package_id: String,
    pub quantity: i64,
    #[serde(default)]
    pub unit_price_minor: Option<i64>,
    #[serde(default)]
    pub discount_minor: i64,
    #[serde(default)]
    pub display_name: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Save {
    pub id: String,
    pub expected_revision: Option<i64>,
    pub cart: Cart,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transition {
    pub id: String,
    pub expected_revision: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    Cash,
    Card,
    Other,
}
impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cash => "CASH",
            Self::Card => "CARD",
            Self::Other => "OTHER",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Payment {
    pub payment_method: Method,
    pub amount_minor: i64,
    #[serde(default)]
    pub reference: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Post {
    pub id: String,
    pub expected_revision: i64,
    pub request_id: String,
    pub payments: Vec<Payment>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReturnLine {
    pub sale_item_id: String,
    pub quantity: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReturnDraft {
    pub id: String,
    pub sale_id: String,
    pub reason: String,
    pub lines: Vec<ReturnLine>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReturnPost {
    pub id: String,
    pub request_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Void {
    pub id: String,
    pub request_id: String,
    pub reason: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub branch_id: String,
    pub search: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub payment_method: Option<Method>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Allocation {
    pub batch_id: String,
    pub quantity: i64,
    pub unit_cost_minor: Option<i64>,
    pub expiry_date: Option<String>,
    pub lot_number: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PricedLine {
    pub product_id: String,
    pub product_package_id: String,
    pub product_name_snapshot: String,
    pub package_label_snapshot: String,
    pub quantity: i64,
    pub unit_price_minor: i64,
    pub discount_minor: i64,
    pub sale_discount_minor: i64,
    pub tax_minor: i64,
    pub line_total_minor: i64,
    pub allocations: Vec<Allocation>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub subtotal_minor: i64,
    pub discount_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub lines: Vec<PricedLine>,
}
