use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Draft,
    InProgress,
    Review,
    Completed,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub session_number: String,
    pub branch_id: String,
    pub status: Status,
    pub scope_type: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub revision: i64,
    pub snapshot_watermark: Option<i64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub reviewed_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub id: String,
    pub session_id: String,
    pub product_package_id: String,
    pub batch_id: String,
    pub expected_quantity: i64,
    pub counted_quantity: Option<i64>,
    pub notes: Option<String>,
    pub excluded_reason: Option<String>,
    pub counted_at: Option<String>,
    pub revision: i64,
    pub product_label: String,
    pub package_label: String,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub current_live: i64,
    pub snapshot_variance: Option<i64>,
    pub correction_delta: Option<i64>,
    pub intervening_movements: i64,
    pub movement_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total: i64,
    pub counted: i64,
    pub uncounted: i64,
    pub excluded: i64,
    pub matches: i64,
    pub positive: i64,
    pub negative: i64,
    pub corrections: i64,
    pub intervening: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub session: Session,
    pub lines: Vec<Line>,
    pub summary: Summary,
    pub review_token: String,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    #[serde(flatten)]
    pub session: Session,
    pub summary: Summary,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub items: Vec<SessionRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Create {
    pub branch_id: String,
    pub notes: Option<String>,
    pub created_by: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transition {
    pub session_id: String,
    pub expected_revision: i64,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub session_id: String,
    pub expected_revision: i64,
    pub line_id: String,
    pub line_revision: i64,
    pub counted_quantity: Option<i64>,
    pub notes: Option<String>,
    pub excluded_reason: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Discover {
    pub session_id: String,
    pub expected_revision: i64,
    pub product_package_id: String,
    pub batch_id: Option<String>,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub notes: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Complete {
    pub session_id: String,
    pub expected_revision: i64,
    pub request_id: String,
    pub review_token: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub branch_id: String,
    pub status: Option<Status>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
