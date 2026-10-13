use super::{
    models::{
        PushReceipt, RemoteEvent, ResolveSyncConflictInput, SyncConflict, SyncCycleResult,
        SyncEvent,
    },
    replication,
    transport::SyncTransport,
};
use crate::{
    db::connection::AppDb,
    error::{AppError, ErrorCode},
    inventory::models::{InventoryBatch, InventoryMovement, MovementType},
};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn device_context(db: &AppDb) -> Result<(String, String), AppError> {
    db.lock()?
        .query_row(
            "SELECT organization_id,device_id FROM cloud_device_enrollments WHERE status='ENROLLED' ORDER BY organization_id LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::new(ErrorCode::Validation, "This device is not enrolled for cloud synchronization."))
}

pub fn list_conflicts(db: &AppDb) -> Result<Vec<SyncConflict>, AppError> {
    let (organization, _) = device_context(db)?;
    let connection = db.lock()?;
    let mut statement = connection.prepare("SELECT id,event_id,entity_type,entity_id,reason_code,detail,status,created_at,resolved_at,resolution_action,resolved_by_user_id,resolution_note FROM sync_conflicts WHERE organization_id=?1 ORDER BY status='OPEN' DESC,created_at DESC,id DESC LIMIT 200")?;
    let rows = statement
        .query_map([organization], |row| {
            Ok(SyncConflict {
                id: row.get(0)?,
                event_id: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                reason_code: row.get(4)?,
                detail: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
                resolved_at: row.get(8)?,
                resolution_action: row.get(9)?,
                resolved_by_user_id: row.get(10)?,
                resolution_note: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn resolve_conflict(
    db: &AppDb,
    actor: &str,
    input: &ResolveSyncConflictInput,
) -> Result<(), AppError> {
    uuid::Uuid::parse_str(&input.conflict_id)
        .map_err(|_| AppError::new(ErrorCode::Validation, "Invalid conflict identifier."))?;
    let note = input.note.trim();
    if note.is_empty() || note.len() > 500 {
        return Err(AppError::new(
            ErrorCode::Validation,
            "A resolution note between 1 and 500 characters is required.",
        ));
    }
    if !matches!(input.action.as_str(), "RETRY" | "DISMISS") {
        return Err(AppError::new(
            ErrorCode::Validation,
            "Conflict action must be RETRY or DISMISS.",
        ));
    }
    let (organization, _) = device_context(db)?;
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let event:Option<String>=tx.query_row("SELECT c.event_id FROM sync_conflicts c JOIN organization_memberships m ON m.organization_id=c.organization_id AND m.user_id=?3 AND m.status='ACTIVE' WHERE c.id=?1 AND c.organization_id=?2 AND c.status='OPEN'",params![input.conflict_id,organization,actor],|r|r.get(0)).optional()?;
    let event = event.ok_or_else(|| {
        AppError::new(
            ErrorCode::NotFound,
            "The open synchronization conflict was not found.",
        )
    })?;
    if input.action == "RETRY" {
        tx.execute("UPDATE sync_conflicts SET resolution_action='RETRIED',resolved_by_user_id=?2,resolution_note=?3 WHERE id=?1",params![input.conflict_id,actor,note])?;
        tx.execute("UPDATE sync_inbox SET status='RECEIVED',applied_at=NULL,error=NULL WHERE organization_id=?1 AND event_id=?2",params![organization,event])?;
    } else {
        tx.execute("UPDATE sync_conflicts SET status='DISMISSED',resolved_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),resolution_action='DISMISSED',resolved_by_user_id=?2,resolution_note=?3 WHERE id=?1",params![input.conflict_id,actor,note])?;
    }
    tx.commit()?;
    Ok(())
}

pub fn phase8c_gate(db: &AppDb) -> Result<bool, AppError> {
    Ok(db.lock()?.query_row(
        "SELECT phase8c_verified FROM sync_runtime_gate WHERE singleton=1",
        [],
        |row| row.get(0),
    )?)
}

pub fn verify_local_cloud_identity(
    db: &AppDb,
    local_user_id: &str,
    cloud_subject_id: &str,
) -> Result<(), AppError> {
    let valid: bool = db.lock()?.query_row(
        "SELECT EXISTS(SELECT 1 FROM cloud_device_enrollments e JOIN organization_memberships m ON m.organization_id=e.organization_id AND m.user_id=?1 AND m.status='ACTIVE' JOIN cloud_identity_links l ON l.organization_id=e.organization_id AND l.local_user_id=?1 AND l.cloud_subject_id=?2 AND l.status='ACTIVE' WHERE e.device_id=(SELECT device_id FROM local_device_identity WHERE singleton=1) AND e.status='ENROLLED')",
        params![local_user_id,cloud_subject_id],
        |row| row.get(0),
    )?;
    if !valid {
        return Err(AppError::new(
            ErrorCode::Forbidden,
            "The cloud identity is not linked to this local user and enrolled device.",
        ));
    }
    Ok(())
}

fn claim(
    db: &AppDb,
    organization: &str,
    device: &str,
) -> Result<(String, Vec<SyncEvent>), AppError> {
    let lease = uuid::Uuid::new_v4().to_string();
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let rows = {
        let mut statement = tx.prepare("SELECT event_id,organization_id,branch_id,device_id,actor_user_id,entity_type,entity_id,operation,payload_json,payload_hash,schema_version,causation_id,occurred_at,local_sequence FROM sync_outbox WHERE organization_id=?1 AND device_id=?2 AND acknowledged_at IS NULL AND (next_attempt_at IS NULL OR next_attempt_at<=strftime('%Y-%m-%dT%H:%M:%fZ','now')) AND (lease_expires_at IS NULL OR lease_expires_at<=strftime('%Y-%m-%dT%H:%M:%fZ','now')) ORDER BY local_sequence LIMIT 100")?;
        let mapped = statement
            .query_map(params![organization, device], |row| {
                let payload: String = row.get(8)?;
                let value = serde_json::from_str(&payload).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(SyncEvent {
                    event_id: row.get(0)?,
                    organization_id: row.get(1)?,
                    branch_id: row.get(2)?,
                    device_id: row.get(3)?,
                    actor_user_id: row.get(4)?,
                    entity_type: row.get(5)?,
                    entity_id: row.get(6)?,
                    operation: row.get(7)?,
                    payload: value,
                    payload_hash: row.get(9)?,
                    schema_version: row.get(10)?,
                    causation_id: row.get(11)?,
                    occurred_at: row.get(12)?,
                    local_sequence: row.get(13)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        mapped
    };
    for event in &rows {
        tx.execute("UPDATE sync_outbox SET lease_owner=?2,lease_expires_at=strftime('%Y-%m-%dT%H:%M:%fZ','now','+5 minutes'),last_attempt_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),attempt_count=attempt_count+1 WHERE event_id=?1",params![event.event_id,lease])?;
    }
    tx.commit()?;
    Ok((lease, rows))
}

fn release_failed(db: &AppDb, lease: &str) -> Result<(), AppError> {
    db.lock()?.execute("UPDATE sync_outbox SET lease_owner=NULL,lease_expires_at=NULL,last_error='Cloud request failed; retry scheduled.',next_attempt_at=strftime('%Y-%m-%dT%H:%M:%fZ','now','+'||min(3600,30*(1 << min(attempt_count,7)))||' seconds') WHERE lease_owner=?1 AND acknowledged_at IS NULL",[lease])?;
    Ok(())
}

fn validate_receipts(events: &[SyncEvent], receipts: &[PushReceipt]) -> Result<(), AppError> {
    let map: HashMap<_, _> = receipts
        .iter()
        .map(|receipt| (receipt.event_id.as_str(), receipt.server_sequence))
        .collect();
    if map.len() != events.len()
        || events
            .iter()
            .any(|event| !map.contains_key(event.event_id.as_str()))
    {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "The cloud receipt set did not match the pushed events.",
        ));
    }
    Ok(())
}

fn accept_receipts(
    db: &AppDb,
    lease: &str,
    events: &[SyncEvent],
    receipts: &[PushReceipt],
) -> Result<(), AppError> {
    validate_receipts(events, receipts)?;
    let map: HashMap<_, _> = receipts
        .iter()
        .map(|receipt| (receipt.event_id.as_str(), receipt.server_sequence))
        .collect();
    if map.len() != events.len()
        || events
            .iter()
            .any(|event| !map.contains_key(event.event_id.as_str()))
    {
        release_failed(db, lease)?;
        return Err(AppError::new(
            ErrorCode::Conflict,
            "The cloud receipt set did not match the pushed events.",
        ));
    }
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for event in events {
        let sequence = map[event.event_id.as_str()];
        tx.execute("UPDATE sync_outbox SET acknowledged_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),lease_owner=NULL,lease_expires_at=NULL,last_error=NULL,next_attempt_at=NULL WHERE event_id=?1 AND lease_owner=?2",params![event.event_id,lease])?;
        tx.execute("INSERT OR IGNORE INTO sync_receipts(organization_id,event_id,device_id,server_sequence) VALUES(?1,?2,?3,?4)",params![event.organization_id,event.event_id,event.device_id,sequence])?;
    }
    tx.commit()?;
    Ok(())
}

fn attempt_start(
    db: &AppDb,
    organization: &str,
    device: &str,
    direction: &str,
) -> Result<String, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    db.lock()?.execute("INSERT INTO sync_attempts(id,organization_id,device_id,direction,outcome) VALUES(?1,?2,?3,?4,'STARTED')",params![id,organization,device,direction])?;
    Ok(id)
}

fn attempt_finish(
    db: &AppDb,
    id: &str,
    outcome: &str,
    count: usize,
    message: Option<&str>,
) -> Result<(), AppError> {
    db.lock()?.execute("UPDATE sync_attempts SET outcome=?2,event_count=?3,safe_message=?4,finished_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,outcome,count as i64,message])?;
    Ok(())
}

fn checkpoint(db: &AppDb, organization: &str, device: &str) -> Result<i64, AppError> {
    Ok(db.lock()?.query_row("SELECT coalesce(CAST(cursor AS INTEGER),0) FROM sync_checkpoints WHERE organization_id=?1 AND device_id=?2 AND stream='primary'",params![organization,device],|row|row.get(0)).optional()?.unwrap_or(0))
}

fn store_remote(
    db: &AppDb,
    organization: &str,
    device: &str,
    events: &[RemoteEvent],
) -> Result<(usize, usize, usize, i64), AppError> {
    let mut connection = db.lock()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut inserted = 0usize;
    let mut max_sequence = checkpoint_on(&tx, organization, device)?;
    for event in events {
        if event.organization_id != organization
            || event.server_sequence <= 0
            || event.payload_hash != hash(&super::outbox::canonical_json(&event.payload)?)
        {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "The cloud returned an invalid event envelope.",
            ));
        }
        let payload = super::outbox::canonical_json(&event.payload)?;
        let changed=tx.execute("INSERT OR IGNORE INTO sync_inbox(organization_id,event_id,branch_id,payload_hash,status,server_sequence,device_id,actor_user_id,entity_type,entity_id,operation,payload_json,schema_version,causation_id,occurred_at) VALUES(?1,?2,?3,?4,'RECEIVED',?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",params![event.organization_id,event.event_id,event.branch_id,event.payload_hash,event.server_sequence,event.device_id,event.actor_user_id,event.entity_type,event.entity_id,event.operation,payload,event.schema_version,event.causation_id,event.occurred_at])?;
        if changed == 0 {
            let old: String = tx.query_row(
                "SELECT payload_hash FROM sync_inbox WHERE organization_id=?1 AND event_id=?2",
                params![organization, event.event_id],
                |row| row.get(0),
            )?;
            if old != event.payload_hash {
                return Err(AppError::new(
                    ErrorCode::Conflict,
                    "A remote event identity was reused with different content.",
                ));
            }
        } else {
            inserted += 1;
        }
        max_sequence = max_sequence.max(event.server_sequence);
    }
    tx.execute("INSERT INTO sync_checkpoints(organization_id,device_id,stream,cursor) VALUES(?1,?2,'primary',?3) ON CONFLICT(organization_id,device_id,stream) DO UPDATE SET cursor=excluded.cursor,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![organization,device,max_sequence.to_string()])?;
    let (applied, conflicts) = apply_received(&tx, organization, device)?;
    tx.commit()?;
    Ok((inserted, applied, conflicts, max_sequence))
}

fn checkpoint_on(
    connection: &rusqlite::Connection,
    organization: &str,
    device: &str,
) -> Result<i64, AppError> {
    Ok(connection.query_row("SELECT coalesce(CAST(cursor AS INTEGER),0) FROM sync_checkpoints WHERE organization_id=?1 AND device_id=?2 AND stream='primary'",params![organization,device],|row|row.get(0)).optional()?.unwrap_or(0))
}

struct ApplyFailure {
    reason: &'static str,
    detail: &'static str,
}

fn invalid_payload() -> ApplyFailure {
    ApplyFailure {
        reason: "INVALID_PAYLOAD",
        detail: "The remote event payload did not match the versioned domain contract.",
    }
}

fn immutable_conflict() -> ApplyFailure {
    ApplyFailure {
        reason: "IMMUTABLE_CONFLICT",
        detail: "The immutable transaction identity already exists with different content.",
    }
}

fn database_conflict(error: &rusqlite::Error) -> ApplyFailure {
    let text = error.to_string();
    if text.contains("balance_range") || text.contains("negative") {
        ApplyFailure {
            reason: "REVISION_CONFLICT",
            detail: "Applying the remote transaction would violate the canonical stock balance.",
        }
    } else {
        ApplyFailure {
            reason: "MISSING_DEPENDENCY",
            detail: "A required local branch, catalog, batch, account, or prior transaction is unavailable.",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InventoryPayload {
    movement: InventoryMovement,
    batch: InventoryBatch,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransferPayload {
    id: String,
    source_branch_id: String,
    destination_branch_id: String,
    product_package_id: String,
    source_batch: InventoryBatch,
    destination_batch: InventoryBatch,
    quantity: i64,
    reason: String,
    created_by: Option<String>,
    created_at: String,
    out_movement: InventoryMovement,
    in_movement: InventoryMovement,
}

fn movement_on(
    connection: &rusqlite::Connection,
    id: &str,
) -> Result<Option<InventoryMovement>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at FROM inventory_movements WHERE id=?1",
            [id],
            |row| {
                let kind: String = row.get(4)?;
                let movement_type = serde_json::from_value(serde_json::Value::String(kind))
                    .map_err(|error| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    ))?;
                Ok(InventoryMovement {
                    id: row.get(0)?, branch_id: row.get(1)?, product_package_id: row.get(2)?,
                    batch_id: row.get(3)?, movement_type, quantity_delta: row.get(5)?,
                    reference_type: row.get(6)?, reference_id: row.get(7)?, reason: row.get(8)?,
                    created_by: row.get(9)?, transfer_id: row.get(10)?, created_at: row.get(11)?,
                })
            },
        )
        .optional()
}

fn batch_on(
    connection: &rusqlite::Connection,
    id: &str,
) -> Result<Option<InventoryBatch>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at FROM inventory_batches WHERE id=?1",
            [id],
            |row| Ok(InventoryBatch {
                id: row.get(0)?, branch_id: row.get(1)?, product_package_id: row.get(2)?,
                lot_number: row.get(3)?, expiry_date: row.get(4)?, received_at: row.get(5)?,
                cost_price_minor: row.get(6)?, supplier_reference: row.get(7)?, is_active: row.get(8)?,
                created_at: row.get(9)?, updated_at: row.get(10)?,
            }),
        )
        .optional()
}

fn apply_inventory(
    connection: &rusqlite::Connection,
    branch: Option<&str>,
    entity: &str,
    payload: &serde_json::Value,
) -> Result<(), ApplyFailure> {
    let value: InventoryPayload =
        serde_json::from_value(payload.clone()).map_err(|_| invalid_payload())?;
    let movement = &value.movement;
    if uuid::Uuid::parse_str(&movement.id).is_err()
        || movement.id != entity
        || branch != Some(movement.branch_id.as_str())
        || movement.batch_id != value.batch.id
        || movement.branch_id != value.batch.branch_id
        || movement.product_package_id != value.batch.product_package_id
    {
        return Err(invalid_payload());
    }
    if movement.transfer_id.is_some()
        || matches!(
            movement.movement_type,
            MovementType::TransferIn | MovementType::TransferOut
        )
    {
        return Err(ApplyFailure {
            reason: "MISSING_DEPENDENCY",
            detail: "Atomic transfers require their paired transfer transaction before either ledger side can be applied.",
        });
    }
    if let Some(existing) =
        movement_on(connection, &movement.id).map_err(|error| database_conflict(&error))?
    {
        return if existing == *movement {
            Ok(())
        } else {
            Err(immutable_conflict())
        };
    }
    match batch_on(connection, &value.batch.id).map_err(|error| database_conflict(&error))? {
        Some(existing) if existing != value.batch => return Err(immutable_conflict()),
        None => {
            connection.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![value.batch.id,value.batch.branch_id,value.batch.product_package_id,value.batch.lot_number,value.batch.expiry_date,value.batch.received_at,value.batch.cost_price_minor,value.batch.supplier_reference,value.batch.is_active,value.batch.created_at,value.batch.updated_at]).map_err(|error|database_conflict(&error))?;
        }
        Some(_) => {}
    }
    connection.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,NULL,?11)",params![movement.id,movement.branch_id,movement.product_package_id,movement.batch_id,movement.movement_type.as_str(),movement.quantity_delta,movement.reference_type,movement.reference_id,movement.reason,movement.created_by,movement.created_at]).map_err(|error|database_conflict(&error))?;
    Ok(())
}

fn apply_transfer(
    connection: &rusqlite::Connection,
    branch: Option<&str>,
    entity: &str,
    payload: &serde_json::Value,
) -> Result<(), ApplyFailure> {
    let value: TransferPayload =
        serde_json::from_value(payload.clone()).map_err(|_| invalid_payload())?;
    let valid = value.id == entity
        && branch == Some(value.source_branch_id.as_str())
        && value.source_branch_id != value.destination_branch_id
        && value.quantity > 0
        && value.source_batch.id == value.out_movement.batch_id
        && value.destination_batch.id == value.in_movement.batch_id
        && value.source_batch.branch_id == value.source_branch_id
        && value.destination_batch.branch_id == value.destination_branch_id
        && value.source_batch.product_package_id == value.product_package_id
        && value.destination_batch.product_package_id == value.product_package_id
        && value.out_movement.transfer_id.as_deref() == Some(value.id.as_str())
        && value.in_movement.transfer_id.as_deref() == Some(value.id.as_str())
        && value.out_movement.movement_type == MovementType::TransferOut
        && value.in_movement.movement_type == MovementType::TransferIn
        && value.out_movement.quantity_delta == -value.quantity
        && value.in_movement.quantity_delta == value.quantity
        && value.out_movement.branch_id == value.source_branch_id
        && value.in_movement.branch_id == value.destination_branch_id
        && value.out_movement.product_package_id == value.product_package_id
        && value.in_movement.product_package_id == value.product_package_id;
    if !valid {
        return Err(invalid_payload());
    }
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM inventory_transfers WHERE id=?1)",
            [&value.id],
            |row| row.get(0),
        )
        .map_err(|error| database_conflict(&error))?;
    if exists {
        let header_matches: bool = connection.query_row(
            "SELECT source_branch_id=?2 AND destination_branch_id=?3 AND product_package_id=?4 AND source_batch_id=?5 AND destination_batch_id=?6 AND quantity=?7 AND out_movement_id=?8 AND in_movement_id=?9 AND reason=?10 AND created_by IS ?11 AND created_at=?12 FROM inventory_transfers WHERE id=?1",
            params![value.id,value.source_branch_id,value.destination_branch_id,value.product_package_id,value.source_batch.id,value.destination_batch.id,value.quantity,value.out_movement.id,value.in_movement.id,value.reason,value.created_by,value.created_at],
            |row| row.get(0),
        ).map_err(|error| database_conflict(&error))?;
        let out_matches = movement_on(connection, &value.out_movement.id)
            .map_err(|error| database_conflict(&error))?
            == Some(value.out_movement);
        let in_matches = movement_on(connection, &value.in_movement.id)
            .map_err(|error| database_conflict(&error))?
            == Some(value.in_movement);
        return if header_matches && out_matches && in_matches {
            Ok(())
        } else {
            Err(immutable_conflict())
        };
    }
    match batch_on(connection, &value.source_batch.id).map_err(|error| database_conflict(&error))? {
        Some(existing) if existing == value.source_batch => {}
        Some(_) => return Err(immutable_conflict()),
        None => {
            return Err(ApplyFailure {
                reason: "MISSING_DEPENDENCY",
                detail: "The transfer source batch has not arrived.",
            })
        }
    }
    match batch_on(connection, &value.destination_batch.id)
        .map_err(|error| database_conflict(&error))?
    {
        Some(existing) if existing == value.destination_batch => {}
        Some(_) => return Err(immutable_conflict()),
        None => {
            connection.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![value.destination_batch.id,value.destination_batch.branch_id,value.destination_batch.product_package_id,value.destination_batch.lot_number,value.destination_batch.expiry_date,value.destination_batch.received_at,value.destination_batch.cost_price_minor,value.destination_batch.supplier_reference,value.destination_batch.is_active,value.destination_batch.created_at,value.destination_batch.updated_at]).map_err(|error|database_conflict(&error))?;
        }
    }
    connection.execute("INSERT INTO inventory_transfers(id,source_branch_id,destination_branch_id,product_package_id,source_batch_id,destination_batch_id,quantity,out_movement_id,in_movement_id,reason,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![value.id,value.source_branch_id,value.destination_branch_id,value.product_package_id,value.source_batch.id,value.destination_batch.id,value.quantity,value.out_movement.id,value.in_movement.id,value.reason,value.created_by,value.created_at]).map_err(|error|database_conflict(&error))?;
    for movement in [&value.out_movement, &value.in_movement] {
        connection.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![movement.id,movement.branch_id,movement.product_package_id,movement.batch_id,movement.movement_type.as_str(),movement.quantity_delta,movement.reference_type,movement.reference_id,movement.reason,movement.created_by,movement.transfer_id,movement.created_at]).map_err(|error|database_conflict(&error))?;
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JournalPayload {
    id: String,
    journal_number: String,
    status: String,
    source_type: String,
    source_id: String,
    source_parent_id: Option<String>,
    branch_id: Option<String>,
    reversal_of_journal_id: Option<String>,
    description: String,
    currency: String,
    debit_total_minor: i64,
    credit_total_minor: i64,
    cogs_complete: bool,
    posted_at: String,
    lines: Vec<JournalLinePayload>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JournalLinePayload {
    id: String,
    position: i64,
    account_id: String,
    side: String,
    amount_minor: i64,
    memo: Option<String>,
}

fn existing_journal_matches(
    connection: &rusqlite::Connection,
    value: &JournalPayload,
) -> Result<bool, rusqlite::Error> {
    let header: Option<bool> = connection.query_row(
        "SELECT journal_number=?2 AND status=?3 AND source_type=?4 AND source_id=?5 AND source_parent_id IS ?6 AND branch_id IS ?7 AND reversal_of_journal_id IS ?8 AND description=?9 AND currency=?10 AND debit_total_minor=?11 AND credit_total_minor=?12 AND cogs_complete=?13 AND posted_at=?14 FROM finance_journals WHERE id=?1",
        params![value.id,value.journal_number,value.status,value.source_type,value.source_id,value.source_parent_id,value.branch_id,value.reversal_of_journal_id,value.description,value.currency,value.debit_total_minor,value.credit_total_minor,value.cogs_complete,value.posted_at],
        |row| row.get(0),
    ).optional()?;
    if header != Some(true) {
        return Ok(false);
    }
    let count: i64 = connection.query_row(
        "SELECT count(*) FROM finance_journal_lines WHERE journal_id=?1",
        [&value.id],
        |row| row.get(0),
    )?;
    if count != value.lines.len() as i64 {
        return Ok(false);
    }
    for line in &value.lines {
        let matches: bool = connection.query_row(
            "SELECT position=?3 AND account_id=?4 AND side=?5 AND amount_minor=?6 AND memo IS ?7 FROM finance_journal_lines WHERE id=?1 AND journal_id=?2",
            params![line.id,value.id,line.position,line.account_id,line.side,line.amount_minor,line.memo],
            |row| row.get(0),
        ).optional()?.unwrap_or(false);
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}

fn apply_finance(
    connection: &rusqlite::Connection,
    branch: Option<&str>,
    entity: &str,
    payload: &serde_json::Value,
) -> Result<(), ApplyFailure> {
    let value: JournalPayload =
        serde_json::from_value(payload.clone()).map_err(|_| invalid_payload())?;
    if uuid::Uuid::parse_str(&value.id).is_err()
        || value.id != entity
        || branch != value.branch_id.as_deref()
        || value.status != "POSTED"
        || value.currency != "EGP"
        || value.lines.len() < 2
    {
        return Err(invalid_payload());
    }
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM finance_journals WHERE id=?1)",
            [&value.id],
            |row| row.get(0),
        )
        .map_err(|error| database_conflict(&error))?;
    if exists {
        return if existing_journal_matches(connection, &value)
            .map_err(|error| database_conflict(&error))?
        {
            Ok(())
        } else {
            Err(immutable_conflict())
        };
    }
    connection.execute("INSERT INTO finance_journals(id,journal_number,status,source_type,source_id,source_parent_id,branch_id,reversal_of_journal_id,description,currency,debit_total_minor,credit_total_minor,cogs_complete,created_at) VALUES(?1,?2,'POSTING',?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",params![value.id,value.journal_number,value.source_type,value.source_id,value.source_parent_id,value.branch_id,value.reversal_of_journal_id,value.description,value.currency,value.debit_total_minor,value.credit_total_minor,value.cogs_complete,value.posted_at]).map_err(|error|database_conflict(&error))?;
    for line in &value.lines {
        connection.execute("INSERT INTO finance_journal_lines(id,journal_id,position,account_id,side,amount_minor,memo) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![line.id,value.id,line.position,line.account_id,line.side,line.amount_minor,line.memo]).map_err(|error|database_conflict(&error))?;
    }
    connection
        .execute(
            "UPDATE finance_journals SET status='POSTED',posted_at=?2 WHERE id=?1",
            params![value.id, value.posted_at],
        )
        .map_err(|error| database_conflict(&error))?;
    Ok(())
}

fn apply_received(
    connection: &rusqlite::Connection,
    organization: &str,
    local_device: &str,
) -> Result<(usize, usize), AppError> {
    let rows = {
        let mut statement=connection.prepare("SELECT i.event_id,i.device_id,i.entity_type,i.entity_id,i.branch_id,i.operation,i.payload_json,i.schema_version FROM sync_inbox i LEFT JOIN sync_conflicts c ON c.organization_id=i.organization_id AND c.event_id=i.event_id WHERE i.organization_id=?1 AND (i.status='RECEIVED' OR (i.status='REJECTED' AND c.status='OPEN' AND c.reason_code IN ('MISSING_DEPENDENCY','REVISION_CONFLICT','APPLY_FAILED'))) ORDER BY i.server_sequence,i.event_id")?;
        let mapped = statement
            .query_map([organization], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        mapped
    };
    let mut conflicts = 0;
    let mut applied = 0;
    for (event, origin, kind, entity, branch, operation, payload, schema_version) in rows {
        if origin.as_deref() == Some(local_device) {
            connection.execute("UPDATE sync_inbox SET status='APPLIED',applied_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE organization_id=?1 AND event_id=?2",params![organization,event])?;
            applied += 1;
            continue;
        }
        let kind = kind.unwrap_or_else(|| "unknown".into());
        let entity = entity.unwrap_or_else(|| "unknown".into());
        let parsed = payload
            .as_deref()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok());
        connection.execute_batch("SAVEPOINT sync_remote_apply")?;
        connection.execute(
            "UPDATE sync_apply_context SET applying=1 WHERE singleton=1",
            [],
        )?;
        let outcome = if operation.as_deref() != Some("APPENDED") || schema_version != Some(1) {
            Err(invalid_payload())
        } else if let Some(payload) = parsed.as_ref() {
            match kind.as_str() {
                "inventory_movement" => {
                    apply_inventory(connection, branch.as_deref(), &entity, payload)
                }
                "inventory_transfer" => {
                    apply_transfer(connection, branch.as_deref(), &entity, payload)
                }
                "finance_journal" => apply_finance(connection, branch.as_deref(), &entity, payload),
                other => {
                    replication::apply(connection, other, &entity, payload).map_err(|failure| {
                        ApplyFailure {
                            reason: failure.reason,
                            detail: failure.detail,
                        }
                    })
                }
            }
        } else {
            Err(invalid_payload())
        };
        match outcome {
            Ok(()) => {
                connection.execute(
                    "UPDATE sync_apply_context SET applying=0 WHERE singleton=1",
                    [],
                )?;
                connection.execute_batch("RELEASE sync_remote_apply")?;
                connection.execute("UPDATE sync_inbox SET status='APPLIED',applied_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),error=NULL WHERE organization_id=?1 AND event_id=?2",params![organization,event])?;
                connection.execute("UPDATE sync_conflicts SET status='RESOLVED',resolved_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),detail='Resolved automatically after required dependencies became available.' WHERE organization_id=?1 AND event_id=?2 AND status='OPEN'",params![organization,event])?;
                applied += 1;
                continue;
            }
            Err(failure) => {
                connection
                    .execute_batch("ROLLBACK TO sync_remote_apply; RELEASE sync_remote_apply")?;
                connection.execute("INSERT OR IGNORE INTO sync_conflicts(id,organization_id,event_id,entity_type,entity_id,reason_code,detail) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![uuid::Uuid::new_v4().to_string(),organization,event,kind,entity,failure.reason,failure.detail])?;
            }
        }
        connection.execute("UPDATE sync_inbox SET status='REJECTED',error='Quarantined for explicit conflict handling.',attempt_count=attempt_count+1 WHERE organization_id=?1 AND event_id=?2",params![organization,event])?;
        conflicts += 1;
    }
    Ok((applied, conflicts))
}

pub fn run_cycle<T: SyncTransport>(db: &AppDb, transport: &T) -> Result<SyncCycleResult, AppError> {
    if !phase8c_gate(db)? {
        return Err(AppError::new(
            ErrorCode::Forbidden,
            "Synchronization remains disabled until Phase 8C verification.",
        ));
    }
    let (organization, device) = device_context(db)?;
    let push_attempt = attempt_start(db, &organization, &device, "PUSH")?;
    let (lease, events) = claim(db, &organization, &device)?;
    let receipts = match transport.push(&events) {
        Ok(value) => value,
        Err(error) => {
            release_failed(db, &lease)?;
            attempt_finish(
                db,
                &push_attempt,
                "FAILED",
                events.len(),
                Some("Cloud push failed; a retry was scheduled."),
            )?;
            return Err(error);
        }
    };
    if let Err(error) = validate_receipts(&events, &receipts) {
        release_failed(db, &lease)?;
        attempt_finish(
            db,
            &push_attempt,
            "FAILED",
            events.len(),
            Some("Cloud receipts did not match the pending batch."),
        )?;
        return Err(error);
    }
    if let Err(error) = transport.acknowledge(&receipts) {
        release_failed(db, &lease)?;
        attempt_finish(
            db,
            &push_attempt,
            "FAILED",
            events.len(),
            Some("Cloud receipt acknowledgement failed; a retry was scheduled."),
        )?;
        return Err(error);
    }
    accept_receipts(db, &lease, &events, &receipts)?;
    attempt_finish(db, &push_attempt, "SUCCEEDED", events.len(), None)?;
    let pull_attempt = attempt_start(db, &organization, &device, "PULL")?;
    let after = checkpoint(db, &organization, &device)?;
    let remote = match transport.pull(after, 200) {
        Ok(value) => value,
        Err(error) => {
            attempt_finish(
                db,
                &pull_attempt,
                "FAILED",
                0,
                Some("Cloud pull failed; the durable checkpoint was retained."),
            )?;
            return Err(error);
        }
    };
    let (pulled, applied, conflicts, checkpoint) =
        match store_remote(db, &organization, &device, &remote) {
            Ok(value) => value,
            Err(error) => {
                attempt_finish(
                    db,
                    &pull_attempt,
                    "FAILED",
                    remote.len(),
                    Some("Cloud events were rejected before the checkpoint advanced."),
                )?;
                return Err(error);
            }
        };
    attempt_finish(db, &pull_attempt, "SUCCEEDED", remote.len(), None)?;
    Ok(SyncCycleResult {
        pushed: events.len(),
        pulled,
        applied,
        conflicts,
        checkpoint,
    })
}
