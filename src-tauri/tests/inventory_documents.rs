use m2_health_lib::{
    db::{
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        models::{
            branch::CreateBranch,
            catalog::{CreateProduct, CreateProductPackage},
        },
        repositories::{branches, product_packages, products},
    },
    inventory::{
        balances,
        documents::{self as docs, models::*},
        models::{CreateBatch, OpeningStock},
        movements,
    },
};
use rusqlite::Connection;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    branch: String,
    other: String,
    package: String,
    batch: String,
}
fn with_db(db: AppDb) -> F {
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let other = branches::create_branch(
        &db,
        &CreateBranch {
            code: "OTHER".into(),
            name: "Other".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap()
    .id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Document medicine".into()),
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
    let batch = movements::opening(
        &db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: branch.clone(),
                product_package_id: package.clone(),
                ..Default::default()
            },
            request_id: id(),
            quantity: 10,
            reason: "Test fixture".into(),
            created_by: None,
        },
    )
    .unwrap()
    .batch_id;
    F {
        db,
        branch,
        other,
        package,
        batch,
    }
}
fn f() -> F {
    with_db(AppDb::in_memory().unwrap())
}
fn input(f: &F, kind: Kind, action: Action, qty: i64) -> SaveDocument {
    SaveDocument {
        document_id: None,
        expected_revision: None,
        kind,
        branch_id: f.branch.clone(),
        destination_branch_id: (kind == Kind::Transfer).then(|| f.other.clone()),
        reference_text: Some("DEV TEST".into()),
        supplier_reference: None,
        notes: None,
        created_by: None,
        lines: vec![LineInput {
            product_package_id: f.package.clone(),
            batch_id: (kind != Kind::Receipt).then(|| f.batch.clone()),
            action,
            quantity: qty,
            reason: "Document test".into(),
            lot_number: None,
            expiry_date: None,
            cost_price_minor: None,
            supplier_reference: None,
        }],
    }
}
fn ready(f: &F, i: &SaveDocument) -> Document {
    let d = docs::save(&f.db, i).unwrap();
    docs::ready(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision,
        },
    )
    .unwrap()
}
fn request(d: &Document) -> PostDocument {
    PostDocument {
        document_id: d.header.id.clone(),
        expected_revision: d.header.revision,
        request_id: id(),
    }
}
fn posted(f: &F, i: &SaveDocument) -> Document {
    let d = ready(f, i);
    docs::post(&f.db, &request(&d)).unwrap()
}
fn stock(f: &F) -> i64 {
    balances::package(&f.db, &f.branch, &f.package)
        .unwrap()
        .quantity
}
fn count(db: &AppDb, t: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn migration_005_creates_only_document_structures() {
    let f = with_db(AppDb::in_memory_with_migrations(&MIGRATIONS[..5]).unwrap());
    assert_eq!(count(&f.db, "_schema_version"), 5);
    for t in [
        "inventory_documents",
        "inventory_document_lines",
        "inventory_post_requests",
    ] {
        assert_eq!(count(&f.db, t), 0);
    }
    assert!(f
        .db
        .lock()
        .unwrap()
        .prepare("SELECT * FROM stock_movements")
        .is_err());
}
#[test]
fn upgrade_v4_preserves_catalog_and_inventory() {
    let f = with_db(AppDb::in_memory_with_migrations(&MIGRATIONS[..4]).unwrap());
    let n = count(&f.db, "products");
    migrations::run(&mut f.db.lock().unwrap()).unwrap();
    assert_eq!(count(&f.db, "products"), n);
    assert_eq!(stock(&f), 10);
    assert_eq!(count(&f.db, "inventory_movements"), 1);
    assert_eq!(count(&f.db, "inventory_batches"), 1);
    assert_eq!(branches::get_branch(&f.db, &f.branch).unwrap().id, f.branch);
}
#[test]
fn migration_005_rolls_back_partial_schema() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::apply(&mut c, &MIGRATIONS[..4]).unwrap();
    c.execute_batch("CREATE TABLE inventory_documents(marker TEXT); INSERT INTO inventory_documents VALUES('keep');").unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert!(c
        .prepare("SELECT * FROM inventory_document_sequences")
        .is_err());
    assert_eq!(
        c.query_row("SELECT marker FROM inventory_documents", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
}
#[test]
fn draft_receipt_is_persistent_and_has_no_stock_effect() {
    let f = f();
    let d = docs::save(&f.db, &input(&f, Kind::Receipt, Action::Receive, 2)).unwrap();
    assert_eq!(d.header.status, Status::Draft);
    assert_eq!(docs::get(&f.db, &d.header.id).unwrap().lines.len(), 1);
    assert_eq!(stock(&f), 10);
    assert_eq!(count(&f.db, "inventory_batches"), 1);
}
#[test]
fn receipt_ready_changes_no_stock() {
    let f = f();
    assert_eq!(
        ready(&f, &input(&f, Kind::Receipt, Action::Receive, 2))
            .header
            .status,
        Status::Ready
    );
    assert_eq!(stock(&f), 10);
}
#[test]
fn receipt_posts_canonical_movement_and_nullable_batch() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    assert_eq!(stock(&f), 12);
    assert_eq!(d.movements[0].movement_type, "PURCHASE_RECEIPT");
    let c = f.db.lock().unwrap();
    let nullable:i64=c.query_row("SELECT count(*) FROM inventory_batches WHERE id=?1 AND lot_number IS NULL AND expiry_date IS NULL AND cost_price_minor IS NULL",[&d.movements[0].batch_id],|r|r.get(0)).unwrap();
    assert_eq!(nullable, 1);
}
#[test]
fn post_retry_returns_original_without_duplicates() {
    let f = f();
    let d = ready(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let p = request(&d);
    let a = docs::post(&f.db, &p).unwrap();
    let b = docs::post(&f.db, &p).unwrap();
    assert_eq!(a.movements[0].movement_id, b.movements[0].movement_id);
    assert_eq!(stock(&f), 12);
    assert_eq!(count(&f.db, "inventory_post_requests"), 1);
}
#[test]
fn different_post_key_cannot_post_twice() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    assert!(docs::post(&f.db, &request(&d)).is_err());
    assert_eq!(stock(&f), 12);
}
#[test]
fn reused_key_with_other_document_is_rejected() {
    let f = f();
    let a = ready(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let b = ready(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let mut p = request(&a);
    docs::post(&f.db, &p).unwrap();
    p.document_id = b.header.id;
    assert!(docs::post(&f.db, &p).is_err());
    assert_eq!(stock(&f), 12);
}
#[test]
fn cancelled_draft_cannot_post() {
    let f = f();
    let d = docs::save(&f.db, &input(&f, Kind::Receipt, Action::Receive, 2)).unwrap();
    let d = docs::cancel(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision,
        },
    )
    .unwrap();
    assert_eq!(d.header.status, Status::Cancelled);
    assert!(docs::post(&f.db, &request(&d)).is_err());
    assert_eq!(stock(&f), 10);
}
#[test]
fn cancelled_ready_cannot_post() {
    let f = f();
    let d = ready(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let d = docs::cancel(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision,
        },
    )
    .unwrap();
    assert!(docs::post(&f.db, &request(&d)).is_err());
    assert_eq!(stock(&f), 10);
}
#[test]
fn posted_header_lines_links_and_requests_are_immutable() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let c = f.db.lock().unwrap();
    for sql in [
        "UPDATE inventory_documents SET notes='changed' WHERE status='POSTED'",
        "DELETE FROM inventory_documents WHERE status='POSTED'",
        "UPDATE inventory_document_lines SET quantity=1",
        "DELETE FROM inventory_document_lines",
        "UPDATE inventory_document_movements SET line_id='other'",
        "DELETE FROM inventory_document_movements",
        "DELETE FROM inventory_post_requests",
    ] {
        assert!(c.execute(sql, []).is_err(), "{sql}");
    }
    drop(c);
    assert!(docs::cancel(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision
        }
    )
    .is_err());
}
#[test]
fn adjustment_in_uses_canonical_batch() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Adjustment, Action::In, 3));
    assert_eq!(stock(&f), 13);
    assert_eq!(d.movements[0].movement_type, "ADJUSTMENT_IN");
    assert_eq!(d.movements[0].batch_id, f.batch);
}
#[test]
fn adjustment_out_previews_and_posts() {
    let f = f();
    let d = ready(&f, &input(&f, Kind::Adjustment, Action::Out, 3));
    assert_eq!(d.preview[0].current_stock, 10);
    assert_eq!(d.preview[0].projected_stock, 7);
    let d = docs::post(&f.db, &request(&d)).unwrap();
    assert_eq!(d.movements[0].quantity_delta, -3);
    assert_eq!(stock(&f), 7);
}
#[test]
fn insufficient_stock_blocks_ready() {
    let f = f();
    let d = docs::save(&f.db, &input(&f, Kind::Adjustment, Action::Out, 11)).unwrap();
    assert!(docs::ready(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision
        }
    )
    .is_err());
    assert_eq!(stock(&f), 10);
}
#[test]
fn duplicate_batch_lines_cannot_overspend() {
    let f = f();
    let mut i = input(&f, Kind::Adjustment, Action::Out, 6);
    i.lines.push(i.lines[0].clone());
    let d = docs::save(&f.db, &i).unwrap();
    assert_eq!(d.preview[1].projected_stock, -2);
    assert!(docs::ready(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision
        }
    )
    .is_err());
}
#[test]
fn damage_posts_negative_damage() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::WriteOff, Action::Damage, 2));
    assert_eq!(d.movements[0].movement_type, "DAMAGE");
    assert_eq!(stock(&f), 8);
}
#[test]
fn expiry_requires_known_expired_date() {
    let f = f();
    let d = docs::save(&f.db, &input(&f, Kind::WriteOff, Action::Expiry, 2)).unwrap();
    assert!(docs::ready(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision
        }
    )
    .is_err());
}
#[test]
fn future_expiry_is_rejected() {
    let f = f();
    let mut r = input(&f, Kind::Receipt, Action::Receive, 2);
    r.lines[0].expiry_date = Some("2099-01-01".into());
    let receipt = posted(&f, &r);
    let mut w = input(&f, Kind::WriteOff, Action::Expiry, 1);
    w.lines[0].batch_id = Some(receipt.movements[0].batch_id.clone());
    let d = docs::save(&f.db, &w).unwrap();
    assert!(docs::ready(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision
        }
    )
    .is_err());
}
#[test]
fn expired_batch_write_off_posts_expiry() {
    let f = f();
    let mut r = input(&f, Kind::Receipt, Action::Receive, 2);
    r.lines[0].expiry_date = Some("2000-01-01".into());
    let receipt = posted(&f, &r);
    let mut w = input(&f, Kind::WriteOff, Action::Expiry, 1);
    w.lines[0].batch_id = Some(receipt.movements[0].batch_id.clone());
    assert_eq!(posted(&f, &w).movements[0].movement_type, "EXPIRY");
    assert_eq!(stock(&f), 11);
}
#[test]
fn transfer_draft_has_no_effect() {
    let f = f();
    docs::save(&f.db, &input(&f, Kind::Transfer, Action::Transfer, 2)).unwrap();
    assert_eq!(stock(&f), 10);
    assert_eq!(count(&f.db, "inventory_transfers"), 0);
}
#[test]
fn ready_transfer_and_damage_recheck_stock_at_posting() {
    for (kind, action) in [
        (Kind::Transfer, Action::Transfer),
        (Kind::WriteOff, Action::Damage),
    ] {
        let f = f();
        let d = ready(&f, &input(&f, kind, action, 8));
        assert_eq!(stock(&f), 10);
        assert_eq!(count(&f.db, "inventory_movements"), 1);
        posted(&f, &input(&f, Kind::Adjustment, Action::Out, 5));
        assert!(docs::post(&f.db, &request(&d)).is_err());
        assert_eq!(stock(&f), 5);
        assert_eq!(count(&f.db, "inventory_movements"), 2);
        assert_eq!(count(&f.db, "inventory_transfers"), 0);
        assert_eq!(
            docs::get(&f.db, &d.header.id).unwrap().header.status,
            Status::Ready
        );
    }
}
#[test]
fn transfer_reversal_restores_exact_original_batches() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Transfer, Action::Transfer, 2));
    let r = docs::reverse(
        &f.db,
        &ReverseDocument {
            document_id: d.header.id.clone(),
            request_id: id(),
            reason: "Undo transfer".into(),
        },
    )
    .unwrap();
    assert_eq!(r.movements.len(), 2);
    for original in &d.movements {
        let compensation = r
            .movements
            .iter()
            .find(|m| m.reverses_movement_id.as_ref() == Some(&original.movement_id))
            .unwrap();
        assert_eq!(compensation.batch_id, original.batch_id);
        assert_eq!(compensation.branch_id, original.branch_id);
        assert_eq!(compensation.quantity_delta, -original.quantity_delta);
    }
    assert_eq!(stock(&f), 10);
    assert_eq!(
        balances::package(&f.db, &f.other, &f.package)
            .unwrap()
            .quantity,
        0
    );
    assert_eq!(
        docs::get(&f.db, &d.header.id).unwrap().header.status,
        Status::Posted
    );
    assert_eq!(count(&f.db, "inventory_transfers"), 1);
}
#[test]
fn transfer_posts_both_sides_with_line_traceability() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Transfer, Action::Transfer, 2));
    assert_eq!(d.movements.len(), 2);
    assert_eq!(d.movements[0].line_id, d.movements[1].line_id);
    assert_eq!(stock(&f), 8);
    assert_eq!(
        balances::package(&f.db, &f.other, &f.package)
            .unwrap()
            .quantity,
        2
    );
    assert_eq!(count(&f.db, "inventory_transfers"), 1);
}
#[test]
fn transfer_second_leg_failure_rolls_back_document_transaction() {
    let f = f();
    let d = ready(&f, &input(&f, Kind::Transfer, Action::Transfer, 2));
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_transfer BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='TRANSFER_IN' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(docs::post(&f.db, &request(&d)).is_err());
    assert_eq!(stock(&f), 10);
    assert_eq!(count(&f.db, "inventory_transfers"), 0);
    assert_eq!(count(&f.db, "inventory_batches"), 1);
    assert_eq!(
        docs::get(&f.db, &d.header.id).unwrap().header.status,
        Status::Ready
    );
}
#[test]
fn wrong_branch_batch_rejected() {
    let f = f();
    let mut i = input(&f, Kind::Adjustment, Action::Out, 1);
    i.branch_id = f.other.clone();
    assert!(docs::save(&f.db, &i).is_err());
}
#[test]
fn reversal_compensates_and_links_original_movements() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let r = docs::reverse(
        &f.db,
        &ReverseDocument {
            document_id: d.header.id.clone(),
            request_id: id(),
            reason: "Correct receipt".into(),
        },
    )
    .unwrap();
    assert_eq!(stock(&f), 10);
    assert_eq!(r.header.reversal_of, Some(d.header.id));
    assert_eq!(
        r.movements[0].reverses_movement_id,
        Some(d.movements[0].movement_id.clone())
    );
    assert_eq!(r.movements[0].quantity_delta, -2);
}
#[test]
fn reversal_negative_stock_rejected_without_partial_records() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Adjustment, Action::In, 2));
    posted(&f, &input(&f, Kind::Adjustment, Action::Out, 11));
    let n = count(&f.db, "inventory_documents");
    assert!(docs::reverse(
        &f.db,
        &ReverseDocument {
            document_id: d.header.id,
            request_id: id(),
            reason: "Undo".into()
        }
    )
    .is_err());
    assert_eq!(stock(&f), 1);
    assert_eq!(count(&f.db, "inventory_documents"), n);
}
#[test]
fn reversal_retry_is_idempotent_and_second_reversal_rejected() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Adjustment, Action::Out, 2));
    let mut r = ReverseDocument {
        document_id: d.header.id,
        request_id: id(),
        reason: "Undo".into(),
    };
    let a = docs::reverse(&f.db, &r).unwrap();
    assert_eq!(docs::reverse(&f.db, &r).unwrap().header.id, a.header.id);
    r.request_id = id();
    assert!(docs::reverse(&f.db, &r).is_err());
    assert_eq!(stock(&f), 10);
}
#[test]
fn numbers_are_unique_and_readable() {
    let f = f();
    let a = docs::save(&f.db, &input(&f, Kind::Receipt, Action::Receive, 1)).unwrap();
    let b = docs::save(&f.db, &input(&f, Kind::Receipt, Action::Receive, 1)).unwrap();
    assert!(a.header.document_number.starts_with("RCV-"));
    assert_ne!(a.header.document_number, b.header.document_number);
}
#[test]
fn all_posted_movements_reference_document() {
    let f = f();
    let d = posted(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row(
                "SELECT reference_id FROM inventory_movements WHERE id=?1",
                [&d.movements[0].movement_id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        d.header.id
    );
}
#[test]
fn competing_posts_cannot_overspend() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("concurrent.db");
    let f = with_db(AppDb::open(&path).unwrap());
    let a = ready(&f, &input(&f, Kind::Adjustment, Action::Out, 8));
    let b = ready(&f, &input(&f, Kind::Adjustment, Action::Out, 8));
    let db2 = AppDb::open(&path).unwrap();
    let barrier = std::sync::Barrier::new(2);
    let (x, y) = std::thread::scope(|s| {
        let bar = &barrier;
        let one = s.spawn(|| {
            bar.wait();
            docs::post(&f.db, &request(&a)).is_ok()
        });
        let two = s.spawn(|| {
            bar.wait();
            docs::post(&db2, &request(&b)).is_ok()
        });
        (one.join().unwrap(), two.join().unwrap())
    });
    assert_ne!(x, y);
    assert_eq!(stock(&f), 2);
}
#[test]
fn restart_preserves_documents_and_post_keys() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("restart.db");
    let f = with_db(AppDb::open(&path).unwrap());
    let d = ready(&f, &input(&f, Kind::Receipt, Action::Receive, 2));
    let p = request(&d);
    docs::post(&f.db, &p).unwrap();
    drop(f.db);
    let db = AppDb::open(&path).unwrap();
    assert_eq!(docs::post(&db, &p).unwrap().header.status, Status::Posted);
    assert_eq!(
        balances::package(&db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        12
    );
    assert_eq!(count(&db, "_schema_version"), 17);
}
#[test]
fn stale_save_and_ready_rejected() {
    let f = f();
    let mut i = input(&f, Kind::Receipt, Action::Receive, 1);
    let d = docs::save(&f.db, &i).unwrap();
    i.document_id = Some(d.header.id.clone());
    i.expected_revision = Some(d.header.revision);
    docs::save(&f.db, &i).unwrap();
    assert!(docs::save(&f.db, &i).is_err());
    assert!(docs::ready(
        &f.db,
        &Transition {
            document_id: d.header.id,
            expected_revision: 1
        }
    )
    .is_err());
}
#[test]
fn list_uses_branch_kind_status_date_and_db_pagination() {
    let f = f();
    for _ in 0..3 {
        docs::save(&f.db, &input(&f, Kind::Receipt, Action::Receive, 1)).unwrap();
    }
    let q = DocumentQuery {
        branch_id: f.branch.clone(),
        kind: Some(Kind::Receipt),
        status: Some(Status::Draft),
        date_from: Some("2000-01-01".into()),
        date_to: Some("2099-01-01".into()),
        limit: Some(2),
        offset: Some(2),
    };
    let p = docs::list(&f.db, &q).unwrap();
    assert_eq!(p.total, 3);
    assert_eq!(p.items.len(), 1);
    assert_eq!(
        docs::list(
            &f.db,
            &DocumentQuery {
                branch_id: f.other,
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        0
    );
}
#[test]
fn receiving_reuses_only_explicit_compatible_batch() {
    let f = f();
    let mut i = input(&f, Kind::Receipt, Action::Receive, 2);
    i.lines[0].batch_id = Some(f.batch.clone());
    posted(&f, &i);
    assert_eq!(count(&f.db, "inventory_batches"), 1);
    i.lines[0].cost_price_minor = Some(200);
    assert!(docs::save(&f.db, &i).is_err());
}
#[test]
fn late_line_failure_rolls_back_all_receipt_batches_and_movements() {
    let f = f();
    let mut i = input(&f, Kind::Receipt, Action::Receive, 2);
    let mut l = i.lines[0].clone();
    l.quantity = 7;
    i.lines.push(l);
    let d = ready(&f, &i);
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_line BEFORE INSERT ON inventory_movements WHEN NEW.quantity_delta=7 BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(docs::post(&f.db, &request(&d)).is_err());
    assert_eq!(stock(&f), 10);
    assert_eq!(count(&f.db, "inventory_batches"), 1);
    assert_eq!(count(&f.db, "inventory_document_movements"), 0);
}
#[test]
fn reasons_and_action_types_are_required() {
    let f = f();
    let mut i = input(&f, Kind::Adjustment, Action::Out, 1);
    i.lines[0].reason = " ".into();
    assert!(docs::save(&f.db, &i).is_err());
    i.lines[0].reason = "Reason".into();
    i.lines[0].action = Action::Receive;
    assert!(docs::save(&f.db, &i).is_err());
}
#[test]
fn ready_document_is_frozen() {
    let f = f();
    let mut i = input(&f, Kind::Receipt, Action::Receive, 1);
    let d = ready(&f, &i);
    i.document_id = Some(d.header.id);
    i.expected_revision = Some(d.header.revision);
    assert!(docs::save(&f.db, &i).is_err());
}
