use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncEvent {
    pub event_id: String,
    pub organization_id: String,
    pub branch_id: Option<String>,
    pub device_id: String,
    pub actor_user_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
    pub payload: serde_json::Value,
    pub payload_hash: String,
    pub schema_version: i64,
    pub causation_id: Option<String>,
    pub occurred_at: String,
    pub local_sequence: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PushReceipt {
    pub event_id: String,
    pub server_sequence: i64,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteEvent {
    pub organization_id: String,
    pub event_id: String,
    pub branch_id: Option<String>,
    pub device_id: String,
    pub actor_user_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
    pub payload: serde_json::Value,
    pub payload_hash: String,
    pub schema_version: i64,
    pub causation_id: Option<String>,
    pub occurred_at: String,
    pub server_sequence: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncCycleResult {
    pub pushed: usize,
    pub pulled: usize,
    pub applied: usize,
    pub conflicts: usize,
    pub checkpoint: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CloudLoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CloudSessionStatus {
    pub authenticated: bool,
    pub subject_id: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub id: String,
    pub event_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub reason_code: String,
    pub detail: String,
    pub status: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub resolution_action: Option<String>,
    pub resolved_by_user_id: Option<String>,
    pub resolution_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveSyncConflictInput {
    pub conflict_id: String,
    pub action: String,
    pub note: String,
}
