use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertType {
    BatchExpired,
    BatchExpiring,
    LowStock,
    OutOfStock,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Critical,
    High,
    Medium,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Active,
    Resolved,
    Dismissed,
}
pub fn value<T: Serialize>(v: Option<T>) -> Option<String> {
    v.map(|v| {
        serde_json::to_value(v)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    })
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub branch_id: String,
    pub expiry_warning_days: i64,
    pub critical_expiry_days: i64,
    pub expiry_enabled: bool,
    pub low_stock_enabled: bool,
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub branch_id: String,
    pub alert_type: Option<AlertType>,
    pub severity: Option<Severity>,
    pub status: Option<Status>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonitorQuery {
    pub branch_id: String,
    pub business_date: Option<String>,
    pub minimum_days: Option<i64>,
    pub maximum_days: Option<i64>,
    #[serde(default)]
    pub expired: bool,
    #[serde(default)]
    pub out_of_stock: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub business_date: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: String,
    pub branch_id: String,
    pub alert_type: String,
    pub product_package_id: String,
    pub batch_id: Option<String>,
    pub severity: String,
    pub status: String,
    pub fingerprint: String,
    pub details: serde_json::Value,
    pub first_detected_at: String,
    pub last_detected_at: String,
    pub resolved_at: Option<String>,
    pub updated_at: String,
    pub product_name: Option<String>,
    pub product_id: String,
    pub package_label: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub branch_id: String,
    pub product_package_id: String,
    pub batch_id: Option<String>,
    pub product_name: Option<String>,
    pub product_id: String,
    pub package_label: String,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub days_remaining: Option<i64>,
    pub quantity: i64,
    pub cost_price_minor: Option<i64>,
    pub reorder_level: Option<i64>,
    pub minimum_stock: Option<i64>,
    pub maximum_stock: Option<i64>,
    pub status: Option<String>,
    pub severity: Option<String>,
}
