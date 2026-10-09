use serde::Deserialize;
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReportFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub branch_id: Option<String>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub customer_id: Option<String>,
    pub supplier_id: Option<String>,
    pub product_id: Option<String>,
    pub product_package_id: Option<String>,
    pub movement_type: Option<String>,
    pub expiry_days: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
