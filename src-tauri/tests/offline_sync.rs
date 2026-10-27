use m2_health_lib::{
    cloud::{
        engine,
        models::ResolveSyncConflictInput,
        models::{PushReceipt, RemoteEvent, SyncEvent},
        outbox, replication,
        transport::SyncTransport,
    },
    db::{
        connection::AppDb,
        models::catalog::{CreateProduct, CreateProductPackage, SetPackagePrice},
        repositories::{price_history, product_packages, products},
    },
    error::{AppError, ErrorCode},
    inventory::{
        counts::{
            self,
            models::{Complete as CompleteCount, Create as CreateCount, Entry as CountEntry},
            models::{Status as CountStatus, Transition as CountTransition},
        },
        documents::{
            self,
            models::{
                Action as DocumentAction, Kind as DocumentKind, LineInput as DocumentLineInput,
                PostDocument, SaveDocument, Transition as DocumentTransition,
            },
        },
        models::{CreateBatch, OpeningStock, TransferStock},
        models::{InventoryBatch, InventoryMovement, MovementType},
        movements, transfers,
    },
    partners::{self, models::SupplierFields},
    purchasing::{
        self,
        models::{
            CreatePurchaseDraft, CreatePurchaseReturn, PurchaseAction, PurchaseDraftFields,
            PurchaseLineInput, PurchaseReceiptLineInput, PurchaseReturnLineInput, ReceivePurchase,
        },
        receiving, returns as purchase_returns,
    },
    sales::{self, models as sale_models},
};
use rusqlite::params;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use uuid::Uuid;

fn id() -> String {
    Uuid::new_v4().to_string()
}

fn add_transfer_branch(fixture: &Fixture) -> String {
    let branch = id();
    let connection = fixture.db.lock().unwrap();
    connection
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'SYNC2','Sync Branch 2')",
            [&branch],
        )
        .unwrap();
    connection.execute("INSERT INTO branch_tenant_ownership(organization_id,branch_id,adopted_by_user_id) VALUES(?1,?2,?3)",params![fixture.org,branch,fixture.actor]).unwrap();
    connection.execute("INSERT INTO organization_branch_memberships(organization_id,user_id,branch_id,granted_by_user_id) VALUES(?1,?2,?3,?2)",params![fixture.org,fixture.actor,branch]).unwrap();
    branch
}

struct Fixture {
    db: AppDb,
    org: String,
    actor: String,
    branch: String,
    package: String,
    cloud_subject: String,
}

fn fixture() -> Fixture {
    let db = AppDb::in_memory().unwrap();
    let org = id();
    let actor = id();
    let branch = id();
    let cloud_subject = id();
    let connection = db.lock().unwrap();
    let device: String = connection
        .query_row("SELECT device_id FROM local_device_identity", [], |row| {
            row.get(0)
        })
        .unwrap();
    connection.execute("INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,'sync-admin','sync-admin','Sync Admin')",[&actor]).unwrap();
    connection
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'SYNC','Sync Branch')",
            [&branch],
        )
        .unwrap();
    drop(connection);
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Synchronized product".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let package = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: product.id,
            package_label: "Box".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let connection = db.lock().unwrap();
    connection.execute("INSERT INTO organizations(id,name,slug,cloud_tenant_id) VALUES(?1,'Sync Org','sync-org',?1)",[&org]).unwrap();
    connection.execute("INSERT INTO organization_memberships(organization_id,user_id,role_code) VALUES(?1,?2,'OWNER')",params![org,actor]).unwrap();
    connection.execute("INSERT INTO branch_tenant_ownership(organization_id,branch_id,adopted_by_user_id) VALUES(?1,?2,?3)",params![org,branch,actor]).unwrap();
    connection.execute("INSERT INTO organization_branch_memberships(organization_id,user_id,branch_id,granted_by_user_id) VALUES(?1,?2,?3,?2)",params![org,actor,branch]).unwrap();
    connection.execute("INSERT INTO cloud_identity_links(organization_id,local_user_id,cloud_subject_id,status,linked_by_user_id) VALUES(?1,?2,?3,'ACTIVE',?2)",params![org,actor,cloud_subject]).unwrap();
    connection.execute("INSERT INTO cloud_device_enrollments(device_id,organization_id,cloud_device_id,status,enrolled_by_user_id,enrolled_at) VALUES(?1,?2,?1,'ENROLLED',?3,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![device,org,actor]).unwrap();
    drop(connection);
    Fixture {
        db,
        org,
        actor,
        branch,
        package,
        cloud_subject,
    }
}

fn enable_test_gate(fixture: &Fixture) {
    fixture.db.lock().unwrap().execute(
        "UPDATE sync_runtime_gate SET phase8c_verified=1,verified_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE singleton=1",
        [],
    ).unwrap();
}

fn enqueue(fixture: &Fixture) -> String {
    outbox::enqueue(
        &fixture.db.lock().unwrap(),
        Some(&fixture.branch),
        Some(&fixture.actor),
        "inventory_movement",
        "movement-1",
        "APPENDED",
        &json!({"quantityDelta":2}),
        None,
    )
    .unwrap()
    .unwrap()
}

fn remote(event: &SyncEvent, sequence: i64) -> RemoteEvent {
    RemoteEvent {
        organization_id: event.organization_id.clone(),
        event_id: event.event_id.clone(),
        branch_id: event.branch_id.clone(),
        device_id: event.device_id.clone(),
        actor_user_id: event.actor_user_id.clone(),
        entity_type: event.entity_type.clone(),
        entity_id: event.entity_id.clone(),
        operation: event.operation.clone(),
        payload: event.payload.clone(),
        payload_hash: event.payload_hash.clone(),
        schema_version: event.schema_version,
        causation_id: event.causation_id.clone(),
        occurred_at: event.occurred_at.clone(),
        server_sequence: sequence,
    }
}

#[derive(Default)]
struct EchoTransport {
    pushed: Mutex<Vec<SyncEvent>>,
    acknowledgements: Mutex<usize>,
}
impl SyncTransport for EchoTransport {
    fn push(&self, events: &[SyncEvent]) -> Result<Vec<PushReceipt>, AppError> {
        *self.pushed.lock().unwrap() = events.to_vec();
        Ok(events
            .iter()
            .enumerate()
            .map(|(index, event)| PushReceipt {
                event_id: event.event_id.clone(),
                server_sequence: index as i64 + 1,
                duplicate: false,
            })
            .collect())
    }
    fn pull(&self, _after: i64, _limit: usize) -> Result<Vec<RemoteEvent>, AppError> {
        Ok(self
            .pushed
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(index, event)| remote(event, index as i64 + 1))
            .collect())
    }
    fn acknowledge(&self, receipts: &[PushReceipt]) -> Result<(), AppError> {
        *self.acknowledgements.lock().unwrap() += receipts.len();
        Ok(())
    }
}

#[test]
fn phase8c_gate_prevents_network_and_preserves_pending_events() {
    let fixture = fixture();
    enqueue(&fixture);
    let transport = EchoTransport::default();
    assert_eq!(
        engine::run_cycle(&fixture.db, &transport).unwrap_err().code,
        ErrorCode::Forbidden
    );
    assert!(transport.pushed.lock().unwrap().is_empty());
    assert_eq!(
        fixture
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM sync_outbox WHERE acknowledged_at IS NULL",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
}

#[test]
fn successful_cycle_pushes_once_applies_echo_and_advances_after_durable_inbox() {
    let fixture = fixture();
    let event_id = enqueue(&fixture);
    enable_test_gate(&fixture);
    let transport = EchoTransport::default();
    let result = engine::run_cycle(&fixture.db, &transport).unwrap();
    assert_eq!(
        (
            result.pushed,
            result.pulled,
            result.applied,
            result.conflicts,
            result.checkpoint
        ),
        (1, 1, 1, 0, 1)
    );
    assert_eq!(*transport.acknowledgements.lock().unwrap(), 1);
    let connection = fixture.db.lock().unwrap();
    assert_eq!(connection.query_row("SELECT count(*) FROM sync_outbox WHERE event_id=?1 AND acknowledged_at IS NOT NULL",[&event_id],|row|row.get::<_,i64>(0)).unwrap(),1);
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sync_inbox WHERE event_id=?1 AND status='APPLIED'",
                [&event_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sync_attempts WHERE outcome='SUCCEEDED'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
}

struct PushFailure;
impl SyncTransport for PushFailure {
    fn push(&self, _: &[SyncEvent]) -> Result<Vec<PushReceipt>, AppError> {
        Err(AppError::new(ErrorCode::Internal, "isolated failure"))
    }
    fn pull(&self, _: i64, _: usize) -> Result<Vec<RemoteEvent>, AppError> {
        unreachable!()
    }
    fn acknowledge(&self, _: &[PushReceipt]) -> Result<(), AppError> {
        unreachable!()
    }
}

#[test]
fn failed_push_keeps_event_schedules_retry_and_records_only_safe_metadata() {
    let fixture = fixture();
    enqueue(&fixture);
    enable_test_gate(&fixture);
    assert_eq!(
        engine::run_cycle(&fixture.db, &PushFailure)
            .unwrap_err()
            .code,
        ErrorCode::Internal
    );
    let connection = fixture.db.lock().unwrap();
    let row: (i64, Option<String>, Option<String>, Option<String>) = connection
        .query_row(
            "SELECT attempt_count,next_attempt_at,acknowledged_at,last_error FROM sync_outbox",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(row.0, 1);
    assert!(row.1.is_some());
    assert!(row.2.is_none());
    assert_eq!(
        row.3.as_deref(),
        Some("Cloud request failed; retry scheduled.")
    );
    assert_eq!(connection.query_row("SELECT count(*) FROM sync_attempts WHERE outcome='FAILED' AND safe_message NOT LIKE '%isolated failure%'",[],|row|row.get::<_,i64>(0)).unwrap(),1);
}

struct BadReceipt;
impl SyncTransport for BadReceipt {
    fn push(&self, _: &[SyncEvent]) -> Result<Vec<PushReceipt>, AppError> {
        Ok(vec![])
    }
    fn pull(&self, _: i64, _: usize) -> Result<Vec<RemoteEvent>, AppError> {
        unreachable!()
    }
    fn acknowledge(&self, _: &[PushReceipt]) -> Result<(), AppError> {
        unreachable!()
    }
}

#[test]
fn mismatched_receipts_never_acknowledge_local_events() {
    let fixture = fixture();
    enqueue(&fixture);
    enable_test_gate(&fixture);
    assert_eq!(
        engine::run_cycle(&fixture.db, &BadReceipt)
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    assert_eq!(fixture.db.lock().unwrap().query_row("SELECT count(*) FROM sync_outbox WHERE acknowledged_at IS NULL AND lease_owner IS NULL",[],|row|row.get::<_,i64>(0)).unwrap(),1);
}

struct FixedPull {
    events: Vec<RemoteEvent>,
}
impl SyncTransport for FixedPull {
    fn push(&self, events: &[SyncEvent]) -> Result<Vec<PushReceipt>, AppError> {
        Ok(events
            .iter()
            .enumerate()
            .map(|(i, e)| PushReceipt {
                event_id: e.event_id.clone(),
                server_sequence: i as i64 + 100,
                duplicate: false,
            })
            .collect())
    }
    fn pull(&self, _: i64, _: usize) -> Result<Vec<RemoteEvent>, AppError> {
        Ok(self.events.clone())
    }
    fn acknowledge(&self, _: &[PushReceipt]) -> Result<(), AppError> {
        Ok(())
    }
}

fn fixed_remote(fixture: &Fixture, payload: serde_json::Value) -> RemoteEvent {
    let encoded = outbox::canonical_json(&payload).unwrap();
    RemoteEvent {
        organization_id: fixture.org.clone(),
        event_id: id(),
        branch_id: Some(fixture.branch.clone()),
        device_id: id(),
        actor_user_id: id(),
        entity_type: "inventory_movement".into(),
        entity_id: "remote-movement".into(),
        operation: "APPENDED".into(),
        payload,
        payload_hash: format!("{:x}", Sha256::digest(encoded.as_bytes())),
        schema_version: 1,
        causation_id: None,
        occurred_at: "2026-10-10T00:00:00Z".into(),
        server_sequence: 7,
    }
}

fn domain_remote(
    fixture: &Fixture,
    entity_type: &str,
    entity_id: &str,
    payload: serde_json::Value,
    sequence: i64,
) -> RemoteEvent {
    let encoded = outbox::canonical_json(&payload).unwrap();
    RemoteEvent {
        organization_id: fixture.org.clone(),
        event_id: id(),
        branch_id: Some(fixture.branch.clone()),
        device_id: id(),
        actor_user_id: id(),
        entity_type: entity_type.into(),
        entity_id: entity_id.into(),
        operation: "APPENDED".into(),
        payload,
        payload_hash: format!("{:x}", Sha256::digest(encoded.as_bytes())),
        schema_version: 1,
        causation_id: None,
        occurred_at: "2026-10-10T00:00:00Z".into(),
        server_sequence: sequence,
    }
}

#[test]
fn malformed_foreign_transaction_is_quarantined_without_touching_the_ledger() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let remote = fixed_remote(&fixture, json!({"quantityDelta":-99}));
    let result = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![remote],
        },
    )
    .unwrap();
    assert_eq!(
        (
            result.pulled,
            result.applied,
            result.conflicts,
            result.checkpoint
        ),
        (1, 0, 1, 7)
    );
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM inventory_movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(connection.query_row("SELECT count(*) FROM sync_conflicts WHERE reason_code='INVALID_PAYLOAD' AND status='OPEN'",[],|row|row.get::<_,i64>(0)).unwrap(),1);
}

#[test]
fn invalid_remote_envelope_rolls_back_inbox_and_checkpoint() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let mut remote = fixed_remote(&fixture, json!({"quantityDelta":1}));
    remote.payload_hash = "0".repeat(64);
    assert_eq!(
        engine::run_cycle(
            &fixture.db,
            &FixedPull {
                events: vec![remote]
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM sync_inbox", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM sync_checkpoints", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn repeated_remote_identity_is_idempotent() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let event = fixed_remote(&fixture, json!({"notice":"unsupported but deterministic"}));
    let transport = FixedPull {
        events: vec![event],
    };
    let first = engine::run_cycle(&fixture.db, &transport).unwrap();
    let second = engine::run_cycle(&fixture.db, &transport).unwrap();
    assert_eq!((first.pulled, first.conflicts), (1, 1));
    assert_eq!((second.pulled, second.applied, second.conflicts), (0, 0, 0));
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM sync_inbox", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM sync_conflicts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn conflict_resolution_is_authorized_audited_and_never_writes_domain_data() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let event = fixed_remote(&fixture, json!({"unsupported":true}));
    engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![event],
        },
    )
    .unwrap();
    let conflict = engine::list_conflicts(&fixture.db).unwrap().remove(0);
    assert_eq!(
        engine::resolve_conflict(
            &fixture.db,
            &id(),
            &ResolveSyncConflictInput {
                conflict_id: conflict.id.clone(),
                action: "DISMISS".into(),
                note: "Unauthorized actor".into()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::NotFound
    );
    engine::resolve_conflict(
        &fixture.db,
        &fixture.actor,
        &ResolveSyncConflictInput {
            conflict_id: conflict.id.clone(),
            action: "DISMISS".into(),
            note: "Unsupported entity reviewed; no domain write authorized.".into(),
        },
    )
    .unwrap();
    let resolved = engine::list_conflicts(&fixture.db).unwrap().remove(0);
    assert_eq!(resolved.status, "DISMISSED");
    assert_eq!(resolved.resolution_action.as_deref(), Some("DISMISSED"));
    assert_eq!(
        resolved.resolved_by_user_id.as_deref(),
        Some(fixture.actor.as_str())
    );
    assert_eq!(
        fixture
            .db
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM inventory_movements", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn revoked_membership_stops_new_outbox_events() {
    let fixture = fixture();
    fixture.db.lock().unwrap().execute("UPDATE organization_memberships SET status='REVOKED',revoked_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE organization_id=?1 AND user_id=?2",params![fixture.org,fixture.actor]).unwrap();
    assert!(outbox::enqueue(
        &fixture.db.lock().unwrap(),
        Some(&fixture.branch),
        Some(&fixture.actor),
        "inventory_movement",
        "movement-2",
        "APPENDED",
        &json!({}),
        None
    )
    .unwrap()
    .is_none());
    assert_eq!(
        fixture
            .db
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM sync_outbox", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn sync_identity_must_match_active_local_link_and_enrolled_device() {
    let fixture = fixture();
    engine::verify_local_cloud_identity(&fixture.db, &fixture.actor, &fixture.cloud_subject)
        .unwrap();
    assert_eq!(
        engine::verify_local_cloud_identity(&fixture.db, &fixture.actor, &id())
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    fixture.db.lock().unwrap().execute("UPDATE cloud_identity_links SET status='REVOKED',revoked_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE organization_id=?1 AND local_user_id=?2",params![fixture.org,fixture.actor]).unwrap();
    assert_eq!(
        engine::verify_local_cloud_identity(&fixture.db, &fixture.actor, &fixture.cloud_subject)
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
}

#[test]
fn verified_remote_inventory_event_applies_once_through_canonical_constraints() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let batch_id = id();
    let movement_id = id();
    let stamp = "2026-10-10T00:00:00.000Z".to_string();
    let batch = InventoryBatch {
        id: batch_id.clone(),
        branch_id: fixture.branch.clone(),
        product_package_id: fixture.package.clone(),
        lot_number: Some("REMOTE-LOT".into()),
        expiry_date: Some("2030-01-01".into()),
        received_at: Some(stamp.clone()),
        cost_price_minor: Some(1250),
        supplier_reference: None,
        is_active: true,
        created_at: stamp.clone(),
        updated_at: stamp.clone(),
    };
    let movement = InventoryMovement {
        id: movement_id.clone(),
        branch_id: fixture.branch.clone(),
        product_package_id: fixture.package.clone(),
        batch_id,
        movement_type: MovementType::Opening,
        quantity_delta: 5,
        reference_type: None,
        reference_id: None,
        reason: "Verified remote opening".into(),
        created_by: Some(id()),
        transfer_id: None,
        created_at: stamp,
    };
    let payload = json!({"movement":movement,"batch":batch});
    let event = domain_remote(&fixture, "inventory_movement", &movement_id, payload, 11);
    let result = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![event],
        },
    )
    .unwrap();
    assert_eq!((result.applied, result.conflicts), (1, 0));
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=?1",
                [&movement.batch_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        5
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM sync_outbox", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn remote_negative_stock_is_quarantined_and_created_batch_is_rolled_back() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let batch_id = id();
    let movement_id = id();
    let stamp = "2026-10-10T00:00:00.000Z".to_string();
    let payload = json!({
        "batch": {
            "id":batch_id,"branchId":fixture.branch,"productPackageId":fixture.package,
            "lotNumber":"NEGATIVE","expiryDate":null,"receivedAt":null,"costPriceMinor":null,
            "supplierReference":null,"isActive":true,"createdAt":stamp,"updatedAt":stamp
        },
        "movement": {
            "id":movement_id,"branchId":fixture.branch,"productPackageId":fixture.package,
            "batchId":batch_id,"movementType":"SALE","quantityDelta":-1,
            "referenceType":null,"referenceId":null,"reason":"Impossible remote sale",
            "createdBy":null,"transferId":null,"createdAt":stamp
        }
    });
    let event = domain_remote(&fixture, "inventory_movement", &movement_id, payload, 12);
    let result = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![event],
        },
    )
    .unwrap();
    assert_eq!((result.applied, result.conflicts), (0, 1));
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_batches WHERE id=?1",
                [&batch_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_movements WHERE id=?1",
                [&movement_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sync_conflicts WHERE reason_code='REVISION_CONFLICT'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    connection.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,is_active,created_at,updated_at) VALUES(?1,?2,?3,'NEGATIVE',1,?4,?4)",params![batch_id,fixture.branch,fixture.package,stamp]).unwrap();
    connection.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reason,created_at) VALUES(?1,?2,?3,?4,'OPENING',5,'Dependency arrived',?5)",params![id(),fixture.branch,fixture.package,batch_id,stamp]).unwrap();
    drop(connection);
    let retried = engine::run_cycle(&fixture.db, &FixedPull { events: vec![] }).unwrap();
    assert_eq!((retried.applied, retried.conflicts), (1, 0));
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT sum(quantity_delta) FROM inventory_movements WHERE batch_id=?1",
                [&batch_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        4
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sync_conflicts WHERE status='RESOLVED'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
}

#[test]
fn verified_remote_finance_journal_posts_balanced_lines_once() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let journal = id();
    let payload = json!({
        "id":journal,"journalNumber":"JRN-20261010-99999","status":"POSTED",
        "sourceType":"SALE","sourceId":id(),"sourceParentId":null,"branchId":fixture.branch,
        "branchName":"Ignored presentation field","reversalOfJournalId":null,
        "description":"Verified remote journal","currency":"EGP","debitTotalMinor":500,
        "creditTotalMinor":500,"cogsComplete":true,"postedAt":"2026-10-10T00:00:00.000Z",
        "lines":[
            {"id":id(),"position":0,"accountId":"00000000-0000-4000-8000-00000000f001","accountCode":"1000","accountName":"Ignored","accountType":"ASSET","side":"DEBIT","amountMinor":500,"memo":"Remote debit"},
            {"id":id(),"position":1,"accountId":"00000000-0000-4000-8000-00000000f007","accountCode":"4000","accountName":"Ignored","accountType":"REVENUE","side":"CREDIT","amountMinor":500,"memo":"Remote credit"}
        ]
    });
    let event = domain_remote(&fixture, "finance_journal", &journal, payload, 13);
    let result = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![event],
        },
    )
    .unwrap();
    assert_eq!((result.applied, result.conflicts), (1, 0));
    let connection = fixture.db.lock().unwrap();
    assert_eq!(connection.query_row("SELECT count(*) FROM finance_journals WHERE id=?1 AND status='POSTED' AND debit_total_minor=credit_total_minor",[&journal],|row|row.get::<_,i64>(0)).unwrap(),1);
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM finance_journal_lines WHERE journal_id=?1",
                [&journal],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
}

#[test]
fn canonical_inventory_post_and_outbox_event_commit_atomically() {
    let fixture = fixture();
    let movement = movements::opening(
        &fixture.db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: fixture.branch.clone(),
                product_package_id: fixture.package.clone(),
                lot_number: Some("SYNC-LOT".into()),
                ..Default::default()
            },
            request_id: id(),
            quantity: 4,
            reason: "Isolated sync atomicity".into(),
            created_by: Some(fixture.actor.clone()),
        },
    )
    .unwrap();
    let connection = fixture.db.lock().unwrap();
    let row: (String, i64) = connection
        .query_row(
            "SELECT entity_id,local_sequence FROM sync_outbox WHERE entity_type='inventory_movement'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(row, (movement.id, 1));
}

#[test]
fn local_transfer_emits_one_paired_event_in_the_same_transaction() {
    let fixture = fixture();
    let destination = add_transfer_branch(&fixture);
    let opening = movements::opening(
        &fixture.db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: fixture.branch.clone(),
                product_package_id: fixture.package.clone(),
                lot_number: Some("PAIR-LOT".into()),
                ..Default::default()
            },
            request_id: id(),
            quantity: 5,
            reason: "Transfer source".into(),
            created_by: Some(fixture.actor.clone()),
        },
    )
    .unwrap();
    fixture
        .db
        .lock()
        .unwrap()
        .execute("DELETE FROM sync_outbox", [])
        .unwrap();
    let result = transfers::transfer(
        &fixture.db,
        &TransferStock {
            request_id: id(),
            source_branch_id: fixture.branch.clone(),
            destination_branch_id: destination,
            product_package_id: fixture.package.clone(),
            source_batch_id: opening.batch_id,
            quantity: 2,
            reason: "Paired cloud transfer".into(),
            created_by: Some(fixture.actor.clone()),
        },
    )
    .unwrap();
    let connection = fixture.db.lock().unwrap();
    assert_eq!(connection.query_row("SELECT count(*) FROM sync_outbox WHERE entity_type='inventory_transfer' AND entity_id=?1",[&result.id],|r|r.get::<_,i64>(0)).unwrap(),1);
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sync_outbox WHERE entity_type='inventory_movement'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn remote_paired_transfer_applies_both_legs_atomically_and_idempotently() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let destination = add_transfer_branch(&fixture);
    let opening = movements::opening(
        &fixture.db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: fixture.branch.clone(),
                product_package_id: fixture.package.clone(),
                lot_number: Some("REMOTE-PAIR".into()),
                ..Default::default()
            },
            request_id: id(),
            quantity: 5,
            reason: "Remote transfer source".into(),
            created_by: Some(fixture.actor.clone()),
        },
    )
    .unwrap();
    let source_batch=fixture.db.lock().unwrap().query_row("SELECT id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at FROM inventory_batches WHERE id=?1",[&opening.batch_id],|r|Ok(InventoryBatch{id:r.get(0)?,branch_id:r.get(1)?,product_package_id:r.get(2)?,lot_number:r.get(3)?,expiry_date:r.get(4)?,received_at:r.get(5)?,cost_price_minor:r.get(6)?,supplier_reference:r.get(7)?,is_active:r.get(8)?,created_at:r.get(9)?,updated_at:r.get(10)?})).unwrap();
    let transfer = id();
    let destination_batch = id();
    let out = id();
    let incoming = id();
    let stamp = "2026-10-10T12:00:00.000Z";
    let destination_value = InventoryBatch {
        id: destination_batch.clone(),
        branch_id: destination.clone(),
        product_package_id: fixture.package.clone(),
        lot_number: source_batch.lot_number.clone(),
        expiry_date: source_batch.expiry_date.clone(),
        received_at: Some(stamp.into()),
        cost_price_minor: source_batch.cost_price_minor,
        supplier_reference: source_batch.supplier_reference.clone(),
        is_active: true,
        created_at: stamp.into(),
        updated_at: stamp.into(),
    };
    let movement =
        |movement_id: String, branch: String, batch: String, kind: MovementType, delta: i64| {
            InventoryMovement {
                id: movement_id,
                branch_id: branch,
                product_package_id: fixture.package.clone(),
                batch_id: batch,
                movement_type: kind,
                quantity_delta: delta,
                reference_type: Some("inventory_transfer".into()),
                reference_id: Some(transfer.clone()),
                reason: "Paired remote transfer".into(),
                created_by: Some(fixture.actor.clone()),
                transfer_id: Some(transfer.clone()),
                created_at: stamp.into(),
            }
        };
    let payload = json!({"id":transfer,"sourceBranchId":fixture.branch,"destinationBranchId":destination,"productPackageId":fixture.package,"sourceBatch":source_batch,"destinationBatch":destination_value,"quantity":2,"reason":"Paired remote transfer","createdBy":fixture.actor,"createdAt":stamp,"outMovement":movement(out,fixture.branch.clone(),opening.batch_id,MovementType::TransferOut,-2),"inMovement":movement(incoming,destination.clone(),destination_batch,MovementType::TransferIn,2)});
    let event = domain_remote(&fixture, "inventory_transfer", &transfer, payload, 20);
    let first = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![event.clone()],
        },
    )
    .unwrap();
    let second = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![event],
        },
    )
    .unwrap();
    assert_eq!((first.applied, first.conflicts), (1, 0));
    assert_eq!((second.applied, second.conflicts), (0, 0));
    let connection = fixture.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_transfers WHERE id=?1",
                [&transfer],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_movements WHERE transfer_id=?1",
                [&transfer],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
}

#[test]
fn outbox_insert_failure_rolls_back_canonical_inventory_post() {
    let fixture = fixture();
    fixture.db.lock().unwrap().execute_batch(
        "CREATE TRIGGER isolated_outbox_failure BEFORE INSERT ON sync_outbox BEGIN SELECT RAISE(ABORT,'isolated outbox failure'); END;",
    ).unwrap();
    let result = movements::opening(
        &fixture.db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: fixture.branch.clone(),
                product_package_id: fixture.package.clone(),
                lot_number: Some("ROLLBACK-LOT".into()),
                ..Default::default()
            },
            request_id: id(),
            quantity: 4,
            reason: "Rollback sync atomicity".into(),
            created_by: Some(fixture.actor.clone()),
        },
    );
    assert!(result.is_err());
    let connection = fixture.db.lock().unwrap();
    for table in ["inventory_batches", "inventory_movements", "sync_outbox"] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0,
            "{table}"
        );
    }
}

#[test]
fn catalog_and_sale_documents_replicate_and_diverged_catalog_is_quarantined() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let product = id();
    let package = id();
    let stamp = "2026-10-10T12:00:00.000Z";
    let catalog = json!({
        "product":{"id":product,"commercialNameEn":"Remote catalog","commercialNameAr":null,"normalizedNameEn":"remote catalog","normalizedNameAr":null,"scientificName":null,"normalizedScientificName":null,"manufacturerId":null,"categoryId":null,"routeId":null,"description":null,"notes":null,"isActive":1,"createdAt":stamp,"updatedAt":stamp},
        "ingredients":[],
        "packages":[{"id":package,"productId":product,"packageLabel":"Box","packSize":null,"unitName":null,"unitsPerPackage":null,"strengthText":null,"isDefault":1,"isActive":1,"createdAt":stamp,"updatedAt":stamp}],
        "barcodes":[],
        "prices":[]
    });
    let catalog_event = domain_remote(&fixture, "catalog_product", &product, catalog.clone(), 30);
    let first = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![catalog_event.clone()],
        },
    )
    .unwrap();
    let second = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![catalog_event],
        },
    )
    .unwrap();
    assert_eq!((first.applied, first.conflicts), (1, 0));
    assert_eq!((second.applied, second.conflicts), (0, 0));
    let diverged = json!({
        "product":{"id":product,"commercialNameEn":"Other name","commercialNameAr":null,"normalizedNameEn":"other name","normalizedNameAr":null,"scientificName":null,"normalizedScientificName":null,"manufacturerId":null,"categoryId":null,"routeId":null,"description":null,"notes":null,"isActive":1,"createdAt":stamp,"updatedAt":stamp},
        "ingredients":[],
        "packages":[],
        "barcodes":[],
        "prices":[]
    });
    let conflict = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![domain_remote(
                &fixture,
                "catalog_product",
                &product,
                diverged,
                31,
            )],
        },
    )
    .unwrap();
    assert_eq!(conflict.conflicts, 1);
    let sale = id();
    let item = id();
    let allocation = id();
    let movement = id();
    let payment = id();
    let sale_payload = json!({
        "sale":{"id":sale,"saleNumber":"SAL-20261010-0001","branchId":fixture.branch,"customerId":null,"customerName":null,"customerCodeSnapshot":null,"customerPhoneSnapshot":null,"status":"POSTED","revision":2,"draftJson":"{\"branchId\":\"x\"}","subtotalMinor":10,"discountMinor":0,"taxMinor":0,"totalMinor":10,"paymentStatus":"PAID","createdBy":fixture.actor,"notes":null,"postRequestId":id(),"createdAt":stamp,"postedAt":stamp,"voidedAt":null},
        "items":[{"id":item,"saleId":sale,"position":0,"productId":product,"productPackageId":package,"productNameSnapshot":"Remote catalog","packageLabelSnapshot":"Box","quantity":1,"unitPriceMinor":10,"discountMinor":0,"saleDiscountMinor":0,"taxMinor":0,"lineTotalMinor":10,"unitCostMinor":null,"createdAt":stamp}],
        "allocations":[{"id":allocation,"saleItemId":item,"position":0,"batchId":id(),"quantity":1,"unitCostMinor":null,"movementId":movement}],
        "payments":[{"id":payment,"saleId":sale,"paymentMethod":"CASH","amountMinor":10,"reference":null,"createdAt":stamp}],
        "requests":[],
        "movements":[],
        "batches":[]
    });
    let sale_result = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![domain_remote(
                &fixture,
                "sale_document",
                &sale,
                sale_payload,
                32,
            )],
        },
    )
    .unwrap();
    assert!(sale_result.conflicts >= 1);
}

#[test]
fn malformed_remote_stock_count_is_quarantined_without_replaying_corrections() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let session = id();
    let before: i64 = fixture
        .db
        .lock()
        .unwrap()
        .query_row("SELECT count(*) FROM inventory_movements", [], |r| r.get(0))
        .unwrap();
    let payload = json!({
        "session":{"id":session,"sessionNumber":"CNT-20261010-0001","branchId":fixture.branch,"status":"COMPLETED","revision":4,"createdBy":fixture.actor,"completedAt":"2026-10-10T12:00:00.000Z"},
        "lines":[],
        "results":[],
        "movements":[]
    });
    let result = engine::run_cycle(
        &fixture.db,
        &FixedPull {
            events: vec![domain_remote(
                &fixture,
                "stock_count",
                &session,
                payload,
                40,
            )],
        },
    )
    .unwrap();
    assert_eq!((result.applied, result.conflicts), (0, 1));
    let connection = fixture.db.lock().unwrap();
    let reason: String = connection
        .query_row(
            "SELECT reason_code FROM sync_conflicts WHERE entity_id=?1",
            [&session],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(reason, "INVALID_PAYLOAD");
    let after: i64 = connection
        .query_row("SELECT count(*) FROM inventory_movements", [], |r| r.get(0))
        .unwrap();
    assert_eq!(before, after);
}

#[test]
fn authorized_retry_reopens_a_stock_conflict_without_last_write_wins() {
    let fixture = fixture();
    enable_test_gate(&fixture);
    let event = id();
    fixture.db.lock().unwrap().execute(
        "INSERT INTO sync_inbox(organization_id,event_id,payload_hash,status,server_sequence) VALUES(?1,?2,?3,'REJECTED',9)",
        params![fixture.org, event, "0".repeat(64)],
    ).unwrap();
    fixture.db.lock().unwrap().execute(
        "INSERT INTO sync_conflicts(id,organization_id,event_id,entity_type,entity_id,reason_code,detail) VALUES(?1,?2,?3,'inventory_movement',?3,'REVISION_CONFLICT','Applying the remote transaction would violate the canonical stock balance.')",
        params![id(), fixture.org, event],
    ).unwrap();
    let conflict_id: String = fixture
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT id FROM sync_conflicts WHERE event_id=?1",
            [&event],
            |row| row.get(0),
        )
        .unwrap();
    engine::resolve_conflict(
        &fixture.db,
        &fixture.actor,
        &ResolveSyncConflictInput {
            conflict_id,
            action: "RETRY".into(),
            note: "Authorized retry after local stock correction.".into(),
        },
    )
    .unwrap();
    let status: String = fixture
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT status FROM sync_inbox WHERE event_id=?1",
            [&event],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "RECEIVED");
}

#[test]
fn purchase_receipt_and_return_reconstruct_canonical_documents_idempotently() {
    let source = fixture();
    let supplier = partners::create_supplier(
        &source.db,
        &SupplierFields {
            display_name: "Replicated supplier".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let product_id: String = source
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT product_id FROM product_packages WHERE id=?1",
            [&source.package],
            |row| row.get(0),
        )
        .unwrap();
    replication::catalog_product(&source.db.lock().unwrap(), &product_id).unwrap();

    let draft = purchasing::create_draft(
        &source.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: PurchaseDraftFields {
                supplier_id: supplier.id.clone(),
                supplier_reference: Some("SYNC-PO".into()),
                expected_date: None,
                notes: Some("Replicated purchase".into()),
                discount_minor: 0,
                tax_minor: 0,
                lines: vec![PurchaseLineInput {
                    product_package_id: source.package.clone(),
                    ordered_quantity: 2,
                    unit_cost_minor: 125,
                }],
            },
        },
    )
    .unwrap();
    let ordered = purchasing::order(
        &source.db,
        &PurchaseAction {
            id: draft.id,
            expected_revision: draft.revision,
            request_id: id(),
        },
    )
    .unwrap();
    let receipt = receiving::receive(
        &source.db,
        &ReceivePurchase {
            id: id(),
            purchase_order_id: ordered.id.clone(),
            expected_revision: ordered.revision,
            request_id: id(),
            branch_id: source.branch.clone(),
            created_by: Some(source.actor.clone()),
            lines: vec![PurchaseReceiptLineInput {
                purchase_order_line_id: ordered.lines[0].id.clone(),
                quantity: 2,
                lot_number: Some("SYNC-RECEIPT".into()),
                expiry_date: Some("2030-01-01".into()),
                acquisition_cost_minor: None,
            }],
        },
    )
    .unwrap();
    let current = purchasing::detail(&source.db, &ordered.id).unwrap();
    let returned = purchase_returns::create(
        &source.db,
        &CreatePurchaseReturn {
            id: id(),
            purchase_order_id: ordered.id.clone(),
            receipt_id: receipt.id.clone(),
            expected_revision: current.revision,
            request_id: id(),
            reason: "Damaged on arrival".into(),
            created_by: Some(source.actor.clone()),
            lines: vec![PurchaseReturnLineInput {
                receipt_line_id: receipt.lines[0].id.clone(),
                quantity: 1,
            }],
        },
    )
    .unwrap();

    let payload = |kind: &str| -> serde_json::Value {
        let encoded: String = source
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT payload_json FROM sync_outbox WHERE entity_type=?1 ORDER BY local_sequence DESC LIMIT 1",
                [kind],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&encoded).unwrap()
    };
    let catalog = payload("catalog_product");
    let partner = payload("partner_record");
    let received = payload("purchase_receipt");
    let returned_payload = payload("purchase_return");

    let replica = AppDb::in_memory().unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,'replica-admin','replica-admin','Replica Admin')",
            [&source.actor],
        )
        .unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'SYNC','Sync Branch')",
            [&source.branch],
        )
        .unwrap();
    {
        let connection = replica.lock().unwrap();
        replication::apply(&connection, "catalog_product", &product_id, &catalog).unwrap();
        replication::apply(&connection, "partner_record", &supplier.id, &partner).unwrap();
        replication::apply(&connection, "purchase_receipt", &receipt.id, &received).unwrap();
        replication::apply(
            &connection,
            "purchase_return",
            &returned.id,
            &returned_payload,
        )
        .unwrap();
        replication::apply(&connection, "purchase_receipt", &receipt.id, &received).unwrap();
        replication::apply(
            &connection,
            "purchase_return",
            &returned.id,
            &returned_payload,
        )
        .unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM purchase_receipts WHERE id=?1 AND status='POSTED'",
                    [&receipt.id],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM purchase_returns WHERE id=?1 AND status='POSTED'",
                    [&returned.id],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=?1",
                    [&receipt.lines[0].batch_id],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            1
        );
    }
}

#[test]
fn inventory_document_reconstructs_canonical_receipt_idempotently() {
    let source = fixture();
    let product_id: String = source
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT product_id FROM product_packages WHERE id=?1",
            [&source.package],
            |row| row.get(0),
        )
        .unwrap();
    replication::catalog_product(&source.db.lock().unwrap(), &product_id).unwrap();
    let draft = documents::save(
        &source.db,
        &SaveDocument {
            document_id: None,
            expected_revision: None,
            kind: DocumentKind::Receipt,
            branch_id: source.branch.clone(),
            destination_branch_id: None,
            reference_text: Some("SYNC-INVENTORY-RECEIPT".into()),
            supplier_reference: None,
            notes: Some("Replicated inventory receipt".into()),
            created_by: Some(source.actor.clone()),
            lines: vec![DocumentLineInput {
                product_package_id: source.package.clone(),
                batch_id: None,
                action: DocumentAction::Receive,
                quantity: 4,
                reason: "Initial synchronized receipt".into(),
                lot_number: Some("SYNC-DOC".into()),
                expiry_date: Some("2030-01-01".into()),
                cost_price_minor: Some(175),
                supplier_reference: None,
            }],
        },
    )
    .unwrap();
    let ready = documents::ready(
        &source.db,
        &DocumentTransition {
            document_id: draft.header.id.clone(),
            expected_revision: draft.header.revision,
        },
    )
    .unwrap();
    let posted = documents::post(
        &source.db,
        &PostDocument {
            document_id: ready.header.id.clone(),
            expected_revision: ready.header.revision,
            request_id: id(),
        },
    )
    .unwrap();
    let payload = |kind: &str| -> serde_json::Value {
        let encoded: String = source
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT payload_json FROM sync_outbox WHERE entity_type=?1 ORDER BY local_sequence DESC LIMIT 1",
                [kind],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&encoded).unwrap()
    };
    let catalog = payload("catalog_product");
    let document_payload = payload("inventory_document");
    let replica = AppDb::in_memory().unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,'document-admin','document-admin','Document Admin')",
            [&source.actor],
        )
        .unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'SYNC','Sync Branch')",
            [&source.branch],
        )
        .unwrap();
    let connection = replica.lock().unwrap();
    replication::apply(&connection, "catalog_product", &product_id, &catalog).unwrap();
    replication::apply(
        &connection,
        "inventory_document",
        &posted.header.id,
        &document_payload,
    )
    .unwrap();
    replication::apply(
        &connection,
        "inventory_document",
        &posted.header.id,
        &document_payload,
    )
    .unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_documents WHERE id=?1 AND status='POSTED'",
                [&posted.header.id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_document_movements WHERE document_id=?1",
                [&posted.header.id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE product_package_id=?1 AND branch_id=?2",
                params![source.package, source.branch],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        4
    );
}

#[test]
fn sale_and_return_reconstruct_canonical_documents_and_stock_idempotently() {
    let source = fixture();
    price_history::set(
        &source.db,
        &SetPackagePrice {
            product_package_id: source.package.clone(),
            selling_price_minor: 250,
            cost_price_minor: Some(125),
            reason: Some("Cross-device sale price".into()),
        },
    )
    .unwrap();
    let product_id: String = source
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT product_id FROM product_packages WHERE id=?1",
            [&source.package],
            |row| row.get(0),
        )
        .unwrap();
    replication::catalog_product(&source.db.lock().unwrap(), &product_id).unwrap();
    let receipt = documents::save(
        &source.db,
        &SaveDocument {
            document_id: None,
            expected_revision: None,
            kind: DocumentKind::Receipt,
            branch_id: source.branch.clone(),
            destination_branch_id: None,
            reference_text: Some("SALE-BASELINE".into()),
            supplier_reference: None,
            notes: None,
            created_by: Some(source.actor.clone()),
            lines: vec![DocumentLineInput {
                product_package_id: source.package.clone(),
                batch_id: None,
                action: DocumentAction::Receive,
                quantity: 5,
                reason: "Sale replication baseline".into(),
                lot_number: Some("SALE-SYNC".into()),
                expiry_date: Some("2030-01-01".into()),
                cost_price_minor: Some(125),
                supplier_reference: None,
            }],
        },
    )
    .unwrap();
    let receipt = documents::ready(
        &source.db,
        &DocumentTransition {
            document_id: receipt.header.id,
            expected_revision: receipt.header.revision,
        },
    )
    .unwrap();
    let receipt = documents::post(
        &source.db,
        &PostDocument {
            document_id: receipt.header.id,
            expected_revision: receipt.header.revision,
            request_id: id(),
        },
    )
    .unwrap();
    let draft = sales::save(
        &source.db,
        &sale_models::Save {
            id: id(),
            expected_revision: None,
            cart: sale_models::Cart {
                branch_id: source.branch.clone(),
                lines: vec![sale_models::Line {
                    product_package_id: source.package.clone(),
                    quantity: 2,
                    ..Default::default()
                }],
                ..Default::default()
            },
        },
    )
    .unwrap();
    let sale = sales::post(
        &source.db,
        &sale_models::Post {
            id: draft["id"].as_str().unwrap().into(),
            expected_revision: draft["revision"].as_i64().unwrap(),
            request_id: id(),
            payments: vec![sale_models::Payment {
                payment_method: sale_models::Method::Cash,
                amount_minor: draft["totalMinor"].as_i64().unwrap(),
                reference: None,
            }],
        },
    )
    .unwrap();
    let return_id = id();
    sales::create_return(
        &source.db,
        &sale_models::ReturnDraft {
            id: return_id.clone(),
            sale_id: sale["id"].as_str().unwrap().into(),
            reason: "Cross-device customer return".into(),
            lines: vec![sale_models::ReturnLine {
                sale_item_id: sale["items"][0]["id"].as_str().unwrap().into(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    sales::post_return(
        &source.db,
        &sale_models::ReturnPost {
            id: return_id.clone(),
            request_id: id(),
        },
    )
    .unwrap();

    let payload = |kind: &str| -> serde_json::Value {
        let encoded: String = source
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT payload_json FROM sync_outbox WHERE entity_type=?1 ORDER BY local_sequence DESC LIMIT 1",
                [kind],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&encoded).unwrap()
    };
    let catalog = payload("catalog_product");
    let inventory = payload("inventory_document");
    let sale_payload = payload("sale_document");
    let return_payload = payload("sale_return");
    let replica = AppDb::in_memory().unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,'sale-admin','sale-admin','Sale Admin')",
            [&source.actor],
        )
        .unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'SYNC','Sync Branch')",
            [&source.branch],
        )
        .unwrap();
    let connection = replica.lock().unwrap();
    replication::apply(&connection, "catalog_product", &product_id, &catalog).unwrap();
    replication::apply(
        &connection,
        "inventory_document",
        &receipt.header.id,
        &inventory,
    )
    .unwrap();
    replication::apply(
        &connection,
        "sale_document",
        sale["id"].as_str().unwrap(),
        &sale_payload,
    )
    .unwrap();
    replication::apply(&connection, "sale_return", &return_id, &return_payload).unwrap();
    replication::apply(
        &connection,
        "sale_document",
        sale["id"].as_str().unwrap(),
        &sale_payload,
    )
    .unwrap();
    replication::apply(&connection, "sale_return", &return_id, &return_payload).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sales WHERE id=?1 AND status='POSTED'",
                [sale["id"].as_str().unwrap()],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sale_returns WHERE id=?1 AND status='POSTED'",
                [&return_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sale_return_items WHERE return_id=?1",
                [&return_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sale_refunds WHERE return_id=?1",
                [&return_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE product_package_id=?1 AND branch_id=?2",
                params![source.package, source.branch],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        4
    );
}

#[test]
fn stock_count_reconstructs_document_and_correction_once_on_matching_snapshot() {
    let source = fixture();
    let product_id: String = source
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT product_id FROM product_packages WHERE id=?1",
            [&source.package],
            |row| row.get(0),
        )
        .unwrap();
    replication::catalog_product(&source.db.lock().unwrap(), &product_id).unwrap();
    let opening = movements::opening(
        &source.db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: source.branch.clone(),
                product_package_id: source.package.clone(),
                lot_number: Some("COUNT-SYNC".into()),
                ..Default::default()
            },
            request_id: id(),
            quantity: 5,
            reason: "Count replication baseline".into(),
            created_by: Some(source.actor.clone()),
        },
    )
    .unwrap();
    let created = counts::create(
        &source.db,
        &CreateCount {
            branch_id: source.branch.clone(),
            notes: Some("Cross-device count".into()),
            created_by: Some(source.actor.clone()),
        },
    )
    .unwrap();
    let started = counts::start(
        &source.db,
        &CountTransition {
            session_id: created.session.id.clone(),
            expected_revision: created.session.revision,
        },
    )
    .unwrap();
    let entered = counts::record(
        &source.db,
        &CountEntry {
            session_id: started.session.id.clone(),
            expected_revision: started.session.revision,
            line_id: started.lines[0].id.clone(),
            line_revision: started.lines[0].revision,
            counted_quantity: Some(3),
            notes: Some("Verified by pharmacist".into()),
            excluded_reason: None,
        },
    )
    .unwrap();
    let reviewed = counts::transition(
        &source.db,
        &CountTransition {
            session_id: entered.session.id.clone(),
            expected_revision: entered.session.revision,
        },
        CountStatus::Review,
    )
    .unwrap();
    let completed = counts::complete(
        &source.db,
        &CompleteCount {
            session_id: reviewed.session.id.clone(),
            expected_revision: reviewed.session.revision,
            request_id: id(),
            review_token: reviewed.review_token.clone(),
        },
    )
    .unwrap();

    let payload = |kind: &str| -> serde_json::Value {
        let encoded: String = source
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT payload_json FROM sync_outbox WHERE entity_type=?1 ORDER BY local_sequence DESC LIMIT 1",
                [kind],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&encoded).unwrap()
    };
    let catalog = payload("catalog_product");
    let baseline = payload("inventory_movement");
    let count_payload = payload("stock_count");
    let replica = AppDb::in_memory().unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,'count-admin','count-admin','Count Admin')",
            [&source.actor],
        )
        .unwrap();
    replica
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'SYNC','Sync Branch')",
            [&source.branch],
        )
        .unwrap();
    let connection = replica.lock().unwrap();
    replication::apply(&connection, "catalog_product", &product_id, &catalog).unwrap();
    let batch = baseline.get("batch").unwrap();
    let movement = baseline.get("movement").unwrap();
    connection.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![batch["id"].as_str(),batch["branchId"].as_str(),batch["productPackageId"].as_str(),batch["lotNumber"].as_str(),batch["expiryDate"].as_str(),batch["receivedAt"].as_str(),batch["costPriceMinor"].as_i64(),batch["supplierReference"].as_str(),batch["isActive"].as_bool(),batch["createdAt"].as_str(),batch["updatedAt"].as_str()]).unwrap();
    connection.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![movement["id"].as_str(),movement["branchId"].as_str(),movement["productPackageId"].as_str(),movement["batchId"].as_str(),movement["movementType"].as_str(),movement["quantityDelta"].as_i64(),movement["referenceType"].as_str(),movement["referenceId"].as_str(),movement["reason"].as_str(),movement["createdBy"].as_str(),movement["transferId"].as_str(),movement["createdAt"].as_str()]).unwrap();
    replication::apply(
        &connection,
        "stock_count",
        &completed.session.id,
        &count_payload,
    )
    .unwrap();
    replication::apply(
        &connection,
        "stock_count",
        &completed.session.id,
        &count_payload,
    )
    .unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM stock_count_sessions WHERE id=?1 AND status='COMPLETED'",
                [&completed.session.id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM inventory_movements WHERE reference_type='stock_count_line'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=?1",
                [&opening.batch_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        3
    );
    drop(connection);

    let divergent = AppDb::in_memory().unwrap();
    divergent
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,'divergent-admin','divergent-admin','Divergent Admin')",
            [&source.actor],
        )
        .unwrap();
    divergent
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'DIVERGED','Diverged Branch')",
            [&source.branch],
        )
        .unwrap();
    let mut divergent_connection = divergent.lock().unwrap();
    replication::apply(
        &divergent_connection,
        "catalog_product",
        &product_id,
        &catalog,
    )
    .unwrap();
    divergent_connection.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![batch["id"].as_str(),batch["branchId"].as_str(),batch["productPackageId"].as_str(),batch["lotNumber"].as_str(),batch["expiryDate"].as_str(),batch["receivedAt"].as_str(),batch["costPriceMinor"].as_i64(),batch["supplierReference"].as_str(),batch["isActive"].as_bool(),batch["createdAt"].as_str(),batch["updatedAt"].as_str()]).unwrap();
    divergent_connection.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![movement["id"].as_str(),movement["branchId"].as_str(),movement["productPackageId"].as_str(),movement["batchId"].as_str(),movement["movementType"].as_str(),6,movement["referenceType"].as_str(),movement["referenceId"].as_str(),movement["reason"].as_str(),movement["createdBy"].as_str(),movement["transferId"].as_str(),movement["createdAt"].as_str()]).unwrap();
    let transaction = divergent_connection.transaction().unwrap();
    let conflict = replication::apply(
        &transaction,
        "stock_count",
        &completed.session.id,
        &count_payload,
    )
    .unwrap_err();
    assert!(format!("{conflict:?}").contains("REVISION_CONFLICT"));
    transaction.rollback().unwrap();
    assert_eq!(
        divergent_connection
            .query_row(
                "SELECT count(*) FROM stock_count_sessions WHERE id=?1",
                [&completed.session.id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        0
    );
}

#[test]
fn frontend_never_persists_cloud_credentials_and_local_config_is_ignored() {
    let frontend = include_str!("../../src/lib/tauri/cloud.ts").to_ascii_lowercase();
    let ignore = include_str!("../../.gitignore");
    let rust_sources = [
        include_str!("../src/cloud/config.rs"),
        include_str!("../src/cloud/transport.rs"),
        include_str!("../src/cloud/commands.rs"),
    ]
    .join("\n")
    .to_ascii_lowercase();
    assert!(!frontend.contains("localstorage"));
    assert!(!frontend.contains("sessionstorage"));
    assert!(!frontend.contains("indexeddb"));
    assert!(!rust_sources.contains("service_role"));
    assert!(ignore
        .lines()
        .any(|line| line.trim() == "config/cloud.local.json"));
}
