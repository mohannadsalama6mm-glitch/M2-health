use crate::{
    error::AppError,
    inventory::{
        batches,
        models::{InventoryBatch, InventoryMovement},
    },
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

pub(crate) fn canonical_json(value: &serde_json::Value) -> Result<String, AppError> {
    fn canonical(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.iter()
                    .map(|(key, value)| (key.clone(), canonical(value)))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.iter().map(canonical).collect())
            }
            value => value.clone(),
        }
    }
    serde_json::to_string(&canonical(value)).map_err(|_| {
        crate::error::AppError::new(
            crate::error::ErrorCode::Validation,
            "The synchronization payload is invalid.",
        )
    })
}

pub fn enqueue<T: Serialize>(
    connection: &Connection,
    branch_id: Option<&str>,
    actor_user_id: Option<&str>,
    entity_type: &str,
    entity_id: &str,
    operation: &str,
    payload: &T,
    causation_id: Option<&str>,
) -> Result<Option<String>, AppError> {
    let Some(actor) = actor_user_id else {
        return Ok(None);
    };
    let applying: bool = connection.query_row(
        "SELECT applying FROM sync_apply_context WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    if applying {
        return Ok(None);
    }
    let context: Option<(String, String)> = connection
        .query_row(
            "SELECT m.organization_id,d.device_id FROM organization_memberships m JOIN cloud_identity_links l ON l.organization_id=m.organization_id AND l.local_user_id=m.user_id AND l.status='ACTIVE' JOIN cloud_device_enrollments d ON d.organization_id=m.organization_id AND d.status='ENROLLED' LEFT JOIN branch_tenant_ownership o ON o.organization_id=m.organization_id AND o.branch_id=?2 LEFT JOIN organization_branch_memberships b ON b.organization_id=m.organization_id AND b.branch_id=?2 AND b.user_id=m.user_id WHERE m.user_id=?1 AND m.status='ACTIVE' AND (?2 IS NULL OR (o.branch_id IS NOT NULL AND b.user_id IS NOT NULL)) ORDER BY m.organization_id LIMIT 1",
            params![actor, branch_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((organization_id, device_id)) = context else {
        return Ok(None);
    };
    let payload_value = serde_json::to_value(payload).map_err(|_| {
        crate::error::AppError::new(
            crate::error::ErrorCode::Validation,
            "The synchronization payload is invalid.",
        )
    })?;
    let payload_json = canonical_json(&payload_value)?;
    let local_sequence: i64 = connection.query_row(
        "UPDATE sync_local_sequence SET next_value=next_value+1 WHERE singleton=1 RETURNING next_value-1",
        [],
        |row| row.get(0),
    )?;
    let event_id = uuid::Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO sync_outbox(event_id,organization_id,branch_id,device_id,actor_user_id,entity_type,entity_id,operation,payload_json,schema_version,causation_id,occurred_at,local_sequence,payload_hash) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,1,?10,strftime('%Y-%m-%dT%H:%M:%fZ','now'),?11,?12)",
        params![event_id,organization_id,branch_id,device_id,actor,entity_type,entity_id,operation,payload_json,causation_id,local_sequence,hash(&payload_json)],
    )?;
    Ok(Some(event_id))
}

pub fn inventory_movement(
    connection: &Connection,
    movement: &InventoryMovement,
) -> Result<Option<String>, AppError> {
    // A transfer is one cross-branch business transaction. Its two ledger legs
    // are emitted only by `inventory_transfer`, never as independently
    // applicable movement events.
    if movement.transfer_id.is_some() {
        return Ok(None);
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload<'a> {
        movement: &'a InventoryMovement,
        batch: InventoryBatch,
    }
    let batch = batches::get(connection, &movement.branch_id, &movement.batch_id)?;
    enqueue(
        connection,
        Some(&movement.branch_id),
        movement.created_by.as_deref(),
        "inventory_movement",
        &movement.id,
        "APPENDED",
        &Payload { movement, batch },
        movement.reference_id.as_deref(),
    )
}

pub fn inventory_transfer(
    connection: &Connection,
    result: &crate::inventory::models::TransferResult,
    actor_user_id: Option<&str>,
) -> Result<Option<String>, AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
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
    let header = connection.query_row(
        "SELECT source_branch_id,destination_branch_id,product_package_id,quantity,reason,created_by,created_at FROM inventory_transfers WHERE id=?1",
        [&result.id],
        |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?,row.get::<_,String>(4)?,row.get::<_,Option<String>>(5)?,row.get::<_,String>(6)?)),
    )?;
    let payload = Payload {
        id: result.id.clone(),
        source_branch_id: header.0.clone(),
        destination_branch_id: header.1,
        product_package_id: header.2,
        source_batch: batches::get(connection, &header.0, &result.source_batch_id)?,
        destination_batch: batches::get(
            connection,
            &result.in_movement.branch_id,
            &result.destination_batch_id,
        )?,
        quantity: header.3,
        reason: header.4,
        created_by: header.5,
        created_at: header.6,
        out_movement: result.out_movement.clone(),
        in_movement: result.in_movement.clone(),
    };
    enqueue(
        connection,
        Some(&payload.source_branch_id),
        actor_user_id,
        "inventory_transfer",
        &result.id,
        "APPENDED",
        &payload,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_payload_is_stable_across_object_key_order() {
        let first = serde_json::from_str(r#"{"z":{"b":2,"a":1},"a":0}"#).unwrap();
        let second = json!({"a":0,"z":{"a":1,"b":2}});
        assert_eq!(
            canonical_json(&first).unwrap(),
            canonical_json(&second).unwrap()
        );
        assert_eq!(
            hash(&canonical_json(&first).unwrap()),
            hash(&canonical_json(&second).unwrap())
        );
    }
}
