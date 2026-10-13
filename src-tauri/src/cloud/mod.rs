pub mod commands;
pub mod config;
pub mod engine;
pub mod models;
pub mod outbox;
pub mod transport;

use crate::{db::connection::AppDb, error::AppError};
use rusqlite::OptionalExtension;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CloudFoundationStatus {
    pub mode: String,
    pub configured: bool,
    pub endpoint_configured: bool,
    pub device_id: String,
    pub device_status: String,
    pub organization_id: Option<String>,
    pub organization_name: Option<String>,
    pub cloud_identity_linked: bool,
    pub cloud_subject_id: Option<String>,
    pub transport_enabled: bool,
    pub sync_available: bool,
    pub setup_required: bool,
    pub outbox_pending: i64,
    pub inbox_applied: i64,
    pub inbox_rejected: i64,
    pub open_conflicts: i64,
    pub last_sync_at: Option<String>,
    pub last_successful_sync_at: Option<String>,
    pub last_sync_outcome: Option<String>,
    pub last_acknowledged_at: Option<String>,
    pub retry_scheduled: i64,
    pub next_retry_at: Option<String>,
}

pub fn status(db: &AppDb) -> Result<CloudFoundationStatus, AppError> {
    let connection = db.lock()?;
    let device_id = connection.query_row(
        "SELECT device_id FROM local_device_identity WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    let enrollment: Option<(String, String, String)> = connection
        .query_row(
            "SELECT e.organization_id,o.name,e.status FROM cloud_device_enrollments e JOIN organizations o ON o.id=e.organization_id WHERE e.device_id=?1",
            [&device_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let transport_enabled: bool = connection.query_row(
        "SELECT transport_enabled FROM sync_schema_metadata WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    let (organization_id, organization_name, device_status) = match enrollment {
        Some((id, name, state)) => (Some(id), Some(name), state),
        None => (None, None, "LOCAL_ONLY".into()),
    };
    let configured = organization_id.is_some() && device_status == "ENROLLED";
    let outbox_pending = connection.query_row(
        "SELECT count(*) FROM sync_outbox WHERE acknowledged_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    let inbox_applied = connection.query_row(
        "SELECT count(*) FROM sync_inbox WHERE status='APPLIED'",
        [],
        |row| row.get(0),
    )?;
    let inbox_rejected = connection.query_row(
        "SELECT count(*) FROM sync_inbox WHERE status='REJECTED'",
        [],
        |row| row.get(0),
    )?;
    let open_conflicts = connection.query_row(
        "SELECT count(*) FROM sync_conflicts WHERE status='OPEN'",
        [],
        |row| row.get(0),
    )?;
    let last_sync: Option<(String, String)> = connection
        .query_row(
            "SELECT coalesce(finished_at,started_at),outcome FROM sync_attempts ORDER BY started_at DESC,id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let last_acknowledged_at =
        connection.query_row("SELECT max(acknowledged_at) FROM sync_outbox", [], |row| {
            row.get(0)
        })?;
    let last_successful_sync_at = connection.query_row(
        "SELECT max(finished_at) FROM sync_attempts WHERE outcome='SUCCEEDED'",
        [],
        |row| row.get(0),
    )?;
    let (retry_scheduled, next_retry_at) = connection.query_row(
        "SELECT count(*),min(next_attempt_at) FROM sync_outbox WHERE acknowledged_at IS NULL AND next_attempt_at IS NOT NULL",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(CloudFoundationStatus {
        mode: "LOCAL_ONLY".into(),
        configured,
        endpoint_configured: false,
        device_id,
        device_status,
        organization_id,
        organization_name,
        cloud_identity_linked: false,
        cloud_subject_id: None,
        transport_enabled,
        sync_available: configured && transport_enabled,
        setup_required: !configured,
        outbox_pending,
        inbox_applied,
        inbox_rejected,
        open_conflicts,
        last_sync_at: last_sync.as_ref().map(|value| value.0.clone()),
        last_successful_sync_at,
        last_sync_outcome: last_sync.map(|value| value.1),
        last_acknowledged_at,
        retry_scheduled,
        next_retry_at,
    })
}

pub fn finalize_status(
    mut status: CloudFoundationStatus,
    endpoint_configured: bool,
    configured_transport_enabled: bool,
    phase8c_verified: bool,
    cloud_identity_linked: bool,
    cloud_authenticated: bool,
    cloud_subject_id: Option<String>,
) -> CloudFoundationStatus {
    status.endpoint_configured = endpoint_configured;
    status.cloud_identity_linked = cloud_identity_linked;
    status.cloud_subject_id = cloud_subject_id;
    status.transport_enabled =
        status.transport_enabled && configured_transport_enabled && phase8c_verified;
    status.sync_available = status.configured
        && endpoint_configured
        && cloud_identity_linked
        && cloud_authenticated
        && status.transport_enabled;
    status.setup_required = !(status.configured && endpoint_configured && cloud_identity_linked);
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enrolled_status() -> CloudFoundationStatus {
        CloudFoundationStatus {
            mode: "LOCAL_ONLY".into(),
            configured: true,
            endpoint_configured: false,
            device_id: "device".into(),
            device_status: "ENROLLED".into(),
            organization_id: Some("organization".into()),
            organization_name: Some("M2".into()),
            cloud_identity_linked: false,
            cloud_subject_id: None,
            transport_enabled: true,
            sync_available: true,
            setup_required: false,
            outbox_pending: 0,
            inbox_applied: 0,
            inbox_rejected: 0,
            open_conflicts: 0,
            last_sync_at: None,
            last_successful_sync_at: None,
            last_sync_outcome: None,
            last_acknowledged_at: None,
            retry_scheduled: 0,
            next_retry_at: None,
        }
    }

    #[test]
    fn synchronization_status_requires_every_security_gate() {
        for gates in [
            (false, true, true, true, true),
            (true, false, true, true, true),
            (true, true, false, true, true),
            (true, true, true, false, true),
            (true, true, true, true, false),
        ] {
            let status = finalize_status(
                enrolled_status(),
                gates.0,
                gates.1,
                gates.2,
                gates.3,
                gates.4,
                Some("cloud-user".into()),
            );
            assert!(!status.sync_available);
            assert!(!status.transport_enabled || gates.2);
        }
        assert!(
            finalize_status(
                enrolled_status(),
                true,
                true,
                true,
                true,
                true,
                Some("cloud-user".into())
            )
            .sync_available
        );
    }
}
