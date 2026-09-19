use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Kind {
    Receipt,
    Adjustment,
    WriteOff,
    Transfer,
    Reversal,
}
impl Kind {
    pub fn text(self) -> &'static str {
        match self {
            Self::Receipt => "RECEIPT",
            Self::Adjustment => "ADJUSTMENT",
            Self::WriteOff => "WRITE_OFF",
            Self::Transfer => "TRANSFER",
            Self::Reversal => "REVERSAL",
        }
    }
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Receipt => "RCV",
            Self::Adjustment => "ADJ",
            Self::WriteOff => "WOF",
            Self::Transfer => "TRF",
            Self::Reversal => "REV",
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Draft,
    Ready,
    Posted,
    Cancelled,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    Receive,
    In,
    Out,
    Damage,
    Expiry,
    Transfer,
}
impl Action {
    pub fn text(self) -> &'static str {
        match self {
            Self::Receive => "RECEIVE",
            Self::In => "IN",
            Self::Out => "OUT",
            Self::Damage => "DAMAGE",
            Self::Expiry => "EXPIRY",
            Self::Transfer => "TRANSFER",
        }
    }
    pub fn outgoing(self) -> bool {
        matches!(
            self,
            Self::Out | Self::Damage | Self::Expiry | Self::Transfer
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LineInput {
    pub product_package_id: String,
    pub batch_id: Option<String>,
    pub action: Action,
    pub quantity: i64,
    pub reason: String,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub cost_price_minor: Option<i64>,
    pub supplier_reference: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveDocument {
    pub document_id: Option<String>,
    pub expected_revision: Option<i64>,
    pub kind: Kind,
    pub branch_id: String,
    pub destination_branch_id: Option<String>,
    pub reference_text: Option<String>,
    pub supplier_reference: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub lines: Vec<LineInput>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transition {
    pub document_id: String,
    pub expected_revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostDocument {
    pub document_id: String,
    pub expected_revision: i64,
    pub request_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReverseDocument {
    pub document_id: String,
    pub request_id: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub id: String,
    pub document_number: String,
    pub kind: Kind,
    pub status: Status,
    pub revision: i64,
    pub branch_id: String,
    pub destination_branch_id: Option<String>,
    pub reference_text: Option<String>,
    pub supplier_reference: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<String>,
    pub reversal_of: Option<String>,
    pub reversal_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub posted_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub id: String,
    #[serde(flatten)]
    pub input: LineInput,
    pub product_label: String,
    pub package_label: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub movement_id: String,
    pub line_id: String,
    pub reverses_movement_id: Option<String>,
    pub branch_id: String,
    pub batch_id: String,
    pub product_package_id: String,
    pub movement_type: String,
    pub quantity_delta: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub line_id: String,
    pub current_stock: i64,
    pub projected_stock: i64,
    pub expiry_date: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[serde(flatten)]
    pub header: Header,
    pub lines: Vec<Line>,
    pub movements: Vec<Link>,
    pub preview: Vec<Preview>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentQuery {
    pub branch_id: String,
    pub kind: Option<Kind>,
    pub status: Option<Status>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPage {
    pub items: Vec<Header>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchChoice {
    pub id: String,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub cost_price_minor: Option<i64>,
    pub quantity: i64,
}
