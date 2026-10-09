use m2_health_lib::{
    db::{
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        models::catalog::{CreateProduct, CreateProductPackage},
        repositories::{branches, product_packages, products},
    },
    error::ErrorCode,
    inventory::balances,
    partners::{self, models::SupplierFields},
    purchasing::{self, models::*, receiving},
};
use std::{path::Path, thread};

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    branch: String,
    other_branch: String,
    supplier: String,
    package: String,
    other_package: String,
}
fn with_db(db: AppDb) -> F {
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let other_branch = branches::create_branch(
        &db,
        &m2_health_lib::db::models::branch::CreateBranch {
            code: "DEV".into(),
            name: "Development".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap()
    .id;
    let supplier = partners::create_supplier(
        &db,
        &SupplierFields {
            display_name: "Nile Medical".into(),
            tax_registration_id: Some("REC-TAX".into()),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Medicine One".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let package = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: product.id,
            package_label: "Box 20".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Medicine Two".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let other_package = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: product.id,
            package_label: "Bottle".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    F {
        db,
        branch,
        other_branch,
        supplier,
        package,
        other_package,
    }
}
fn f() -> F {
    with_db(AppDb::in_memory().unwrap())
}
fn ordered(f: &F, quantities: (i64, i64)) -> PurchaseOrder {
    let draft = purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: PurchaseDraftFields {
                supplier_id: f.supplier.clone(),
                supplier_reference: Some("SUP-REF".into()),
                expected_date: None,
                notes: None,
                discount_minor: 0,
                tax_minor: 0,
                lines: vec![
                    PurchaseLineInput {
                        product_package_id: f.package.clone(),
                        ordered_quantity: quantities.0,
                        unit_cost_minor: 100,
                    },
                    PurchaseLineInput {
                        product_package_id: f.other_package.clone(),
                        ordered_quantity: quantities.1,
                        unit_cost_minor: 250,
                    },
                ],
            },
        },
    )
    .unwrap();
    purchasing::order(
        &f.db,
        &PurchaseAction {
            id: draft.id,
            expected_revision: draft.revision,
            request_id: id(),
        },
    )
    .unwrap()
}
fn receive_input(
    f: &F,
    p: &PurchaseOrder,
    lines: Vec<PurchaseReceiptLineInput>,
) -> ReceivePurchase {
    ReceivePurchase {
        id: id(),
        purchase_order_id: p.id.clone(),
        expected_revision: p.revision,
        request_id: id(),
        branch_id: f.branch.clone(),
        created_by: Some("Tester".into()),
        lines,
    }
}
fn line(line: &PurchaseLine, quantity: i64) -> PurchaseReceiptLineInput {
    PurchaseReceiptLineInput {
        purchase_order_line_id: line.id.clone(),
        quantity,
        lot_number: Some("LOT-A".into()),
        expiry_date: Some("2030-01-01".into()),
        acquisition_cost_minor: None,
    }
}
fn stock(f: &F, package: &str) -> i64 {
    balances::package(&f.db, &f.branch, package)
        .unwrap()
        .quantity
}
fn count(db: &AppDb, table: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn migration_012_applies_once_preserves_purchases_and_evolves_guard() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..11]).unwrap();
    let f = with_db(db);
    let purchase_id = id();
    let purchase_number = "PO-20300101-0001";
    let line_id = id();
    {
        let c = f.db.lock().unwrap();
        c.execute("INSERT INTO purchase_orders(id,purchase_number,supplier_id,status,revision,subtotal_minor,discount_minor,tax_minor,total_minor,create_request_id) VALUES(?1,?2,?3,'DRAFT',1,200,0,0,200,?4)",rusqlite::params![purchase_id,purchase_number,f.supplier,id()]).unwrap();
        let product_id: String = c
            .query_row(
                "SELECT product_id FROM product_packages WHERE id=?1",
                [&f.package],
                |r| r.get(0),
            )
            .unwrap();
        c.execute("INSERT INTO purchase_order_lines(id,purchase_order_id,position,product_id,product_package_id,product_name_snapshot,package_label_snapshot,ordered_quantity,unit_cost_minor,line_total_minor) VALUES(?1,?2,0,?3,?4,'Medicine One','Box 20',2,100,200)",rusqlite::params![line_id,purchase_id,product_id,f.package]).unwrap();
        c.execute("UPDATE purchase_orders SET status='ORDERED',revision=2,supplier_code_snapshot='SUP-000001',supplier_name_snapshot='Nile Medical',ordered_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[&purchase_id]).unwrap();
    }
    let mut c = f.db.lock().unwrap();
    migrations::run(&mut c).unwrap();
    migrations::run(&mut c).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM _schema_version WHERE version=12 AND name='purchase_receiving'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT purchase_number FROM purchase_orders WHERE id=?1",
            [&purchase_id],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        purchase_number
    );
    for table in [
        "purchase_receipts",
        "purchase_receipt_lines",
        "purchase_receipt_sequences",
    ] {
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
    }
}

#[test]
fn migration_012_failure_rolls_back_partial_schema_and_keeps_v11_guard() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..11]).unwrap();
    let mut c = db.lock().unwrap();
    c.execute_batch("CREATE TABLE purchase_receipts(marker TEXT); INSERT INTO purchase_receipts VALUES('keep');").unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        11
    );
    assert!(c
        .prepare("SELECT * FROM purchase_receipt_sequences")
        .is_err());
    assert_eq!(
        c.query_row("SELECT marker FROM purchase_receipts", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
    assert_eq!(c.query_row("SELECT count(*) FROM sqlite_master WHERE type='trigger' AND name='purchase_order_guard'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
}

#[test]
fn clean_database_reaches_twelve_with_foreign_keys_and_indexes() {
    let f = with_db(AppDb::in_memory_with_migrations(&MIGRATIONS[..12]).unwrap());
    let c = f.db.lock().unwrap();
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        12
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM pragma_foreign_key_list('purchase_receipt_lines')",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        10
    );
    assert!(c.query_row("SELECT count(*) FROM sqlite_master WHERE type='index' AND name='purchase_receipt_lines_batch'",[],|r|r.get::<_,i64>(0)).unwrap()>0);
}

#[test]
fn ordered_one_line_full_receipt_posts_stock_and_becomes_received() {
    let f = f();
    let mut p = ordered(&f, (3, 1));
    let input = receive_input(&f, &p, vec![line(&p.lines[0], 3), line(&p.lines[1], 1)]);
    let receipt = receiving::receive(&f.db, &input).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(receipt.total_quantity, 4);
    assert_eq!(p.status, "RECEIVED");
    assert_eq!(p.revision, 3);
    assert_eq!(stock(&f, &f.package), 3);
    assert_eq!(stock(&f, &f.other_package), 1);
    assert!(p.lines.iter().all(|l| l.remaining_quantity == 0));
}

#[test]
fn selected_branch_owns_batch_movement_and_balance() {
    let f = f();
    let p = ordered(&f, (1, 1));
    let mut input = receive_input(&f, &p, vec![line(&p.lines[0], 1)]);
    input.branch_id = f.other_branch.clone();
    let receipt = receiving::receive(&f.db, &input).unwrap();
    assert_eq!(receipt.branch_id, f.other_branch);
    let scope:(String,String)=f.db.lock().unwrap().query_row("SELECT b.branch_id,m.branch_id FROM inventory_batches b JOIN inventory_movements m ON m.batch_id=b.id WHERE m.id=?1",[&receipt.lines[0].movement_id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(scope, (f.other_branch.clone(), f.other_branch.clone()));
    assert_eq!(
        balances::package(&f.db, &f.other_branch, &f.package)
            .unwrap()
            .quantity,
        1
    );
    assert_eq!(stock(&f, &f.package), 0);
}

#[test]
fn inactive_branch_is_rejected_without_any_receipt_effect() {
    let f = f();
    let p = ordered(&f, (1, 1));
    f.db.lock()
        .unwrap()
        .execute(
            "UPDATE branches SET is_active=0 WHERE id=?1",
            [&f.other_branch],
        )
        .unwrap();
    let mut input = receive_input(&f, &p, vec![line(&p.lines[0], 1)]);
    input.branch_id = f.other_branch.clone();
    assert!(receiving::receive(&f.db, &input).is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
}

#[test]
fn subset_partial_then_multiple_receipts_derive_remaining_and_final_status() {
    let f = f();
    let mut p = ordered(&f, (10, 5));
    let first = receive_input(&f, &p, vec![line(&p.lines[0], 4), line(&p.lines[1], 5)]);
    receiving::receive(&f.db, &first).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(p.status, "PARTIALLY_RECEIVED");
    assert_eq!(
        (p.lines[0].received_quantity, p.lines[0].remaining_quantity),
        (4, 6)
    );
    assert_eq!(p.receipts.len(), 1);
    let second = receive_input(&f, &p, vec![line(&p.lines[0], 6)]);
    receiving::receive(&f.db, &second).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(p.status, "RECEIVED");
    assert_eq!(p.receipts.len(), 2);
    assert_eq!(stock(&f, &f.package), 10);
    assert_eq!(stock(&f, &f.other_package), 5);
}

#[test]
fn draft_cancelled_and_received_are_ineligible() {
    let f = f();
    let draft = purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: PurchaseDraftFields {
                supplier_id: f.supplier.clone(),
                supplier_reference: None,
                expected_date: None,
                notes: None,
                discount_minor: 0,
                tax_minor: 0,
                lines: vec![PurchaseLineInput {
                    product_package_id: f.package.clone(),
                    ordered_quantity: 1,
                    unit_cost_minor: 1,
                }],
            },
        },
    )
    .unwrap();
    assert_eq!(
        receiving::receive(
            &f.db,
            &receive_input(&f, &draft, vec![line(&draft.lines[0], 1)])
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
    let p = ordered(&f, (1, 1));
    let cancelled = purchasing::cancel(
        &f.db,
        &CancelPurchase {
            id: p.id.clone(),
            expected_revision: p.revision,
            request_id: id(),
            reason: "No need".into(),
        },
    )
    .unwrap();
    assert!(receiving::receive(
        &f.db,
        &receive_input(&f, &cancelled, vec![line(&cancelled.lines[0], 1)])
    )
    .is_err());
    let p = ordered(&f, (1, 1));
    receiving::receive(
        &f.db,
        &receive_input(&f, &p, vec![line(&p.lines[0], 1), line(&p.lines[1], 1)]),
    )
    .unwrap();
    let received = purchasing::detail(&f.db, &p.id).unwrap();
    assert!(receiving::receive(
        &f.db,
        &receive_input(&f, &received, vec![line(&received.lines[0], 1)])
    )
    .is_err());
}

#[test]
fn zero_negative_foreign_line_and_over_receipt_roll_back() {
    let f = f();
    let p = ordered(&f, (2, 1));
    for qty in [0, -1, 3] {
        let result =
            receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], qty)]));
        assert!(result.is_err());
    }
    let other = ordered(&f, (1, 1));
    let result = receiving::receive(
        &f.db,
        &receive_input(&f, &p, vec![line(&other.lines[0], 1)]),
    );
    assert!(result.is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
    assert_eq!(count(&f.db, "inventory_batches"), 0);
}

#[test]
fn compatible_batch_is_reused_but_identity_differences_create_batches() {
    let f = f();
    let mut p = ordered(&f, (3, 1));
    let a = receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let b = receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let mut different = line(&p.lines[0], 1);
    different.lot_number = Some("LOT-B".into());
    let c = receiving::receive(&f.db, &receive_input(&f, &p, vec![different])).unwrap();
    assert_eq!(a.lines[0].batch_id, b.lines[0].batch_id);
    assert_ne!(a.lines[0].batch_id, c.lines[0].batch_id);
    assert_eq!(count(&f.db, "inventory_batches"), 2);
}

#[test]
fn no_lot_no_expiry_and_expired_receipt_follow_inventory_semantics() {
    let f = f();
    let mut p = ordered(&f, (2, 1));
    let mut unknown = line(&p.lines[0], 1);
    unknown.lot_number = None;
    unknown.expiry_date = None;
    let a = receiving::receive(&f.db, &receive_input(&f, &p, vec![unknown])).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let mut expired = line(&p.lines[0], 1);
    expired.expiry_date = Some("2020-01-01".into());
    let b = receiving::receive(&f.db, &receive_input(&f, &p, vec![expired])).unwrap();
    assert_eq!(
        (
            a.lines[0].lot_number.clone(),
            a.lines[0].expiry_date.clone()
        ),
        (None, None)
    );
    assert_eq!(b.lines[0].expiry_date.as_deref(), Some("2020-01-01"));
}

#[test]
fn actual_acquisition_cost_is_snapshotted_without_catalog_price_change() {
    let f = f();
    let p = ordered(&f, (1, 1));
    let before: i64 =
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM product_price_history", [], |r| {
                r.get(0)
            })
            .unwrap();
    let mut received = line(&p.lines[0], 1);
    received.acquisition_cost_minor = Some(135);
    let r = receiving::receive(&f.db, &receive_input(&f, &p, vec![received])).unwrap();
    assert_eq!(r.lines[0].acquisition_cost_minor, 135);
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row(
                "SELECT cost_price_minor FROM inventory_batches WHERE id=?1",
                [&r.lines[0].batch_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        135
    );
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM product_price_history", [], |row| row
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        before
    );
}

#[test]
fn exact_retry_returns_original_and_changed_payload_is_rejected() {
    let f = f();
    let p = ordered(&f, (2, 1));
    let input = receive_input(&f, &p, vec![line(&p.lines[0], 1)]);
    let a = receiving::receive(&f.db, &input).unwrap();
    let b = receiving::receive(&f.db, &input).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(a.lines[0].movement_id, b.lines[0].movement_id);
    assert_eq!(stock(&f, &f.package), 1);
    let mut changed = input;
    changed.lines[0].quantity = 2;
    assert!(receiving::receive(&f.db, &changed).is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 1);
    assert_eq!(count(&f.db, "inventory_movements"), 1);
}

#[test]
fn stale_revision_rejected_but_successful_replay_wins_before_revision_check() {
    let f = f();
    let p = ordered(&f, (2, 1));
    let input = receive_input(&f, &p, vec![line(&p.lines[0], 1)]);
    receiving::receive(&f.db, &input).unwrap();
    assert!(receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).is_err());
    assert!(receiving::receive(&f.db, &input).is_ok());
}

#[test]
fn invalid_second_line_rolls_back_receipt_batch_movement_and_status() {
    let f = f();
    let p = ordered(&f, (2, 1));
    let mut bad = line(&p.lines[1], 2);
    bad.expiry_date = Some("bad-date".into());
    assert!(receiving::receive(
        &f.db,
        &receive_input(&f, &p, vec![line(&p.lines[0], 1), bad])
    )
    .is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 0);
    assert_eq!(count(&f.db, "inventory_batches"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
    assert_eq!(purchasing::detail(&f.db, &p.id).unwrap().status, "ORDERED");
}

#[test]
fn injected_movement_failure_rolls_back_everything() {
    let f = f();
    let p = ordered(&f, (1, 1));
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_receipt_movement BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='PURCHASE_RECEIPT' BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 0);
    assert_eq!(count(&f.db, "inventory_batches"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
}

#[test]
fn injected_batch_failure_rolls_back_receipt_number_and_status() {
    let f = f();
    let p = ordered(&f, (1, 1));
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_receipt_batch BEFORE INSERT ON inventory_batches BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 0);
    assert_eq!(count(&f.db, "purchase_receipt_sequences"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
    assert_eq!(purchasing::detail(&f.db, &p.id).unwrap().status, "ORDERED");
}

#[test]
fn injected_status_failure_rolls_back_stock_and_business_receipt() {
    let f = f();
    let p = ordered(&f, (1, 1));
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_receipt_status BEFORE UPDATE ON purchase_orders WHEN NEW.status='PARTIALLY_RECEIVED' OR NEW.status='RECEIVED' BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).is_err());
    assert_eq!(count(&f.db, "purchase_receipts"), 0);
    assert_eq!(count(&f.db, "inventory_batches"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
}

#[test]
fn traceability_and_receipt_list_detail_are_complete_and_immutable() {
    let f = f();
    let p = ordered(&f, (2, 1));
    let r = receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).unwrap();
    let movement: (String, String, String) = f
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT movement_type,reference_type,reference_id FROM inventory_movements WHERE id=?1",
            [&r.lines[0].movement_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        movement,
        (
            "PURCHASE_RECEIPT".into(),
            "purchase_receipt".into(),
            r.id.clone()
        )
    );
    assert_eq!(
        receiving::get(&f.db, &r.id).unwrap().lines[0].batch_id,
        r.lines[0].batch_id
    );
    {
        let c = f.db.lock().unwrap();
        c.execute("UPDATE products SET commercial_name_en='Changed' WHERE id=(SELECT product_id FROM product_packages WHERE id=?1)",[&f.package]).unwrap();
        c.execute(
            "UPDATE suppliers SET display_name='Changed Supplier',revision=revision+1 WHERE id=?1",
            [&f.supplier],
        )
        .unwrap();
    }
    let stable = receiving::get(&f.db, &r.id).unwrap();
    assert_eq!(stable.lines[0].product_name_snapshot, "Medicine One");
    assert_eq!(stable.supplier_name, "Nile Medical");
    let page = receiving::list(
        &f.db,
        &PurchaseReceiptQuery {
            purchase_order_id: Some(p.id),
            supplier_id: Some(f.supplier.clone()),
            branch_id: Some(f.branch.clone()),
            limit: Some(10),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((page.total, page.items.len()), (1, 1));
    assert!(f
        .db
        .lock()
        .unwrap()
        .execute("DELETE FROM purchase_receipts WHERE id=?1", [r.id])
        .is_err());
}

#[test]
fn partially_and_fully_received_purchases_cannot_normal_cancel() {
    let f = f();
    let mut p = ordered(&f, (2, 1));
    receiving::receive(&f.db, &receive_input(&f, &p, vec![line(&p.lines[0], 1)])).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(p.status, "PARTIALLY_RECEIVED");
    assert!(purchasing::cancel(
        &f.db,
        &CancelPurchase {
            id: p.id.clone(),
            expected_revision: p.revision,
            request_id: id(),
            reason: "No".into()
        }
    )
    .is_err());
    let final_input = receive_input(&f, &p, vec![line(&p.lines[0], 1), line(&p.lines[1], 1)]);
    receiving::receive(&f.db, &final_input).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(p.status, "RECEIVED");
    assert!(purchasing::cancel(
        &f.db,
        &CancelPurchase {
            id: p.id,
            expected_revision: p.revision,
            request_id: id(),
            reason: "No".into()
        }
    )
    .is_err());
}

#[test]
fn ordered_with_zero_receipts_can_still_cancel() {
    let f = f();
    let p = ordered(&f, (1, 1));
    assert_eq!(
        purchasing::cancel(
            &f.db,
            &CancelPurchase {
                id: p.id,
                expected_revision: p.revision,
                request_id: id(),
                reason: "Supplier failed".into()
            }
        )
        .unwrap()
        .status,
        "CANCELLED"
    );
}

#[test]
fn concurrent_receipts_cannot_over_receive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("receive.db");
    let f = with_db(AppDb::open(&path).unwrap());
    let p = ordered(&f, (5, 1));
    let a = receive_input(&f, &p, vec![line(&p.lines[0], 5)]);
    let mut b = a.clone();
    b.id = id();
    b.request_id = id();
    drop(f.db);
    let path_a = path.clone();
    let path_b = path.clone();
    let x =
        thread::spawn(move || receiving::receive(&AppDb::open(Path::new(&path_a)).unwrap(), &a));
    let y =
        thread::spawn(move || receiving::receive(&AppDb::open(Path::new(&path_b)).unwrap(), &b));
    let results = [x.join().unwrap(), y.join().unwrap()];
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let db = AppDb::open(&path).unwrap();
    assert_eq!(
        db.lock()
            .unwrap()
            .query_row(
                "SELECT coalesce(sum(quantity),0) FROM purchase_receipt_lines",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        5
    );
    assert_eq!(count(&db, "purchase_receipts"), 1);
}

#[test]
fn concurrent_receipt_numbers_are_unique() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("numbers.db");
    let f = with_db(AppDb::open(&path).unwrap());
    let p1 = ordered(&f, (1, 1));
    let p2 = ordered(&f, (1, 1));
    let a = receive_input(&f, &p1, vec![line(&p1.lines[0], 1)]);
    let b = receive_input(&f, &p2, vec![line(&p2.lines[0], 1)]);
    drop(f.db);
    let pa = path.clone();
    let pb = path.clone();
    let x = thread::spawn(move || {
        receiving::receive(&AppDb::open(&pa).unwrap(), &a)
            .unwrap()
            .receipt_number
    });
    let y = thread::spawn(move || {
        receiving::receive(&AppDb::open(&pb).unwrap(), &b)
            .unwrap()
            .receipt_number
    });
    let x = x.join().unwrap();
    let y = y.join().unwrap();
    assert_ne!(x, y);
    assert!(x.starts_with("PR-") && y.starts_with("PR-"));
}
