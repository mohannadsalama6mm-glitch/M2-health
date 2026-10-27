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
    inventory::{balances, models::StockOperation, movements},
    partners::{self, models::SupplierFields},
    purchasing::{self, models::*, receiving, returns},
};
use std::{path::Path, thread};

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    branch: String,
    supplier: String,
    package: String,
    other_package: String,
}
fn setup(db: AppDb) -> F {
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    branches::create_branch(
        &db,
        &CreateBranch {
            code: "DEV".into(),
            name: "Development".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap();
    let supplier = partners::create_supplier(
        &db,
        &SupplierFields {
            display_name: "Nile Medical".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let make = |name: &str| {
        let p = products::create(
            &db,
            &CreateProduct {
                commercial_name_en: Some(name.into()),
                ..Default::default()
            },
        )
        .unwrap();
        product_packages::create(
            &db,
            &CreateProductPackage {
                product_id: p.id,
                package_label: "Box".into(),
                ..Default::default()
            },
        )
        .unwrap()
        .id
    };
    let package = make("Medicine One");
    let other_package = make("Medicine Two");
    F {
        db,
        branch,
        supplier,
        package,
        other_package,
    }
}
fn f() -> F {
    setup(AppDb::in_memory().unwrap())
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
fn receive(f: &F, p: &PurchaseOrder, lines: Vec<(usize, i64, &str)>) -> PurchaseReceipt {
    receiving::receive(
        &f.db,
        &ReceivePurchase {
            id: id(),
            purchase_order_id: p.id.clone(),
            expected_revision: p.revision,
            request_id: id(),
            branch_id: f.branch.clone(),
            created_by: Some("Tester".into()),
            lines: lines
                .into_iter()
                .map(|(i, q, lot)| PurchaseReceiptLineInput {
                    purchase_order_line_id: p.lines[i].id.clone(),
                    quantity: q,
                    lot_number: Some(lot.into()),
                    expiry_date: Some("2030-01-01".into()),
                    acquisition_cost_minor: None,
                })
                .collect(),
        },
    )
    .unwrap()
}
fn input(p: &PurchaseOrder, r: &PurchaseReceipt, lines: Vec<(usize, i64)>) -> CreatePurchaseReturn {
    CreatePurchaseReturn {
        id: id(),
        purchase_order_id: p.id.clone(),
        receipt_id: r.id.clone(),
        expected_revision: p.revision,
        request_id: id(),
        reason: "Damaged supplier delivery".into(),
        created_by: Some("Tester".into()),
        lines: lines
            .into_iter()
            .map(|(i, q)| PurchaseReturnLineInput {
                receipt_line_id: r.lines[i].id.clone(),
                quantity: q,
            })
            .collect(),
    }
}
fn count(db: &AppDb, table: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn migration_013_applies_once_after_012_with_expected_schema() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..12]).unwrap();
    let mut c = db.lock().unwrap();
    migrations::run(&mut c).unwrap();
    migrations::run(&mut c).unwrap();
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        MIGRATIONS.len() as i64
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM _schema_version WHERE version=13 AND name='purchase_returns'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    for table in [
        "purchase_returns",
        "purchase_return_lines",
        "purchase_return_sequences",
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
    assert!(
        c.query_row(
            "SELECT count(*) FROM pragma_foreign_key_list('purchase_return_lines')",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap()
            >= 10
    );
}

#[test]
fn migration_013_failure_rolls_back_partial_schema_and_keeps_v12() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..12]).unwrap();
    let mut c = db.lock().unwrap();
    c.execute_batch(
        "CREATE TABLE purchase_returns(marker TEXT); INSERT INTO purchase_returns VALUES('keep');",
    )
    .unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        12
    );
    assert!(c
        .prepare("SELECT * FROM purchase_return_sequences")
        .is_err());
    assert_eq!(
        c.query_row("SELECT marker FROM purchase_returns", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
    assert_eq!(c.query_row("SELECT count(*) FROM sqlite_master WHERE type='trigger' AND name='purchase_order_guard'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
}

#[test]
fn full_return_posts_exact_negative_movement_and_preserves_received_status() {
    let f = f();
    let mut p = ordered(&f, (5, 1));
    let receipt = receive(&f, &p, vec![(0, 5, "LOT-A"), (1, 1, "LOT-B")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let ret = returns::create(&f.db, &input(&p, &receipt, vec![(0, 5), (1, 1)])).unwrap();
    let after = purchasing::detail(&f.db, &p.id).unwrap();
    assert!(ret.return_number.starts_with("PRT-"));
    assert_eq!(ret.total_quantity, 6);
    assert_eq!(after.status, "RECEIVED");
    assert_eq!(after.revision, p.revision + 1);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        0
    );
    let movement:(String,i64,String,String,String)=f.db.lock().unwrap().query_row("SELECT movement_type,quantity_delta,batch_id,reference_type,reference_id FROM inventory_movements WHERE id=?1",[&ret.lines[0].movement_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(
        movement,
        (
            "PURCHASE_RETURN".into(),
            -5,
            receipt.lines[0].batch_id.clone(),
            "purchase_return".into(),
            ret.id
        )
    );
}

#[test]
fn partial_and_multiple_returns_derive_quantities() {
    let f = f();
    let mut p = ordered(&f, (10, 1));
    let receipt = receive(&f, &p, vec![(0, 10, "LOT-A")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let first = returns::create(&f.db, &input(&p, &receipt, vec![(0, 3)])).unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let second = returns::create(&f.db, &input(&p, &receipt, vec![(0, 4)])).unwrap();
    let detail = receiving::get(&f.db, &receipt.id).unwrap();
    assert_eq!(
        (
            detail.lines[0].quantity,
            detail.lines[0].returned_quantity,
            detail.lines[0].remaining_returnable_quantity
        ),
        (10, 7, 3)
    );
    assert_eq!(detail.returns.len(), 2);
    assert_eq!(second.lines[0].previously_returned_quantity, 3);
    assert_eq!(second.lines[0].remaining_returnable_quantity, 3);
    let purchase = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(
        (
            purchase.lines[0].returned_quantity,
            purchase.lines[0].remaining_returnable_quantity
        ),
        (7, 3)
    );
    assert_eq!(purchase.returns.len(), 2);
    let stable = returns::get(&f.db, &first.id).unwrap();
    assert_eq!(stable.lines[0].previously_returned_quantity, 0);
    assert_eq!(stable.lines[0].remaining_returnable_quantity, 7);
}

#[test]
fn zero_over_return_foreign_line_and_stale_revision_are_rejected() {
    let f = f();
    let mut p = ordered(&f, (2, 1));
    let r = receive(&f, &p, vec![(0, 2, "A")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let other = ordered(&f, (1, 1));
    for q in [0, -1, 3] {
        assert!(returns::create(&f.db, &input(&p, &r, vec![(0, q)])).is_err());
    }
    let other_receipt = receive(&f, &other, vec![(0, 1, "B")]);
    let mut foreign = input(&p, &r, vec![(0, 1)]);
    foreign.lines[0].receipt_line_id = other_receipt.lines[0].id.clone();
    assert!(returns::create(&f.db, &foreign).is_err());
    let mut stale = input(&p, &r, vec![(0, 1)]);
    stale.expected_revision -= 1;
    assert!(returns::create(&f.db, &stale).is_err());
    assert_eq!(count(&f.db, "purchase_returns"), 0);
}

#[test]
fn current_stock_and_exact_batch_are_enforced_without_fefo_substitution() {
    let f = f();
    let mut p = ordered(&f, (4, 1));
    let receipt = receive(&f, &p, vec![(0, 2, "LOT-A"), (0, 2, "LOT-B")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    movements::damage(
        &f.db,
        &StockOperation {
            request_id: id(),
            branch_id: f.branch.clone(),
            product_package_id: f.package.clone(),
            batch_id: receipt.lines[0].batch_id.clone(),
            quantity: 2,
            reason: "Sold/damaged".into(),
            created_by: None,
        },
    )
    .unwrap();
    assert_eq!(
        balances::batch(&f.db, &f.branch, &receipt.lines[0].batch_id)
            .unwrap()
            .quantity,
        0
    );
    assert_eq!(
        balances::batch(&f.db, &f.branch, &receipt.lines[1].batch_id)
            .unwrap()
            .quantity,
        2
    );
    assert!(returns::create(&f.db, &input(&p, &receipt, vec![(0, 1)])).is_err());
    let ok = returns::create(&f.db, &input(&p, &receipt, vec![(1, 1)])).unwrap();
    assert_eq!(ok.lines[0].batch_id, receipt.lines[1].batch_id);
}

#[test]
fn multi_line_failure_rolls_back_everything() {
    let f = f();
    let mut p = ordered(&f, (2, 2));
    let receipt = receive(&f, &p, vec![(0, 2, "A"), (1, 2, "B")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let mut bad = input(&p, &receipt, vec![(0, 1), (1, 1)]);
    bad.lines[1].quantity = 3;
    assert!(returns::create(&f.db, &bad).is_err());
    assert_eq!(count(&f.db, "purchase_returns"), 0);
    assert_eq!(count(&f.db, "purchase_return_lines"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 2);
}

#[test]
fn movement_failure_rolls_back_return_sequence_revision_and_stock() {
    let f = f();
    let mut p = ordered(&f, (2, 1));
    let receipt = receive(&f, &p, vec![(0, 2, "A")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_purchase_return BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='PURCHASE_RETURN' BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(returns::create(&f.db, &input(&p, &receipt, vec![(0, 1)])).is_err());
    assert_eq!(count(&f.db, "purchase_returns"), 0);
    assert_eq!(count(&f.db, "purchase_return_sequences"), 0);
    assert_eq!(
        purchasing::detail(&f.db, &p.id).unwrap().revision,
        p.revision
    );
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        2
    );
}

#[test]
fn exact_retry_is_idempotent_before_revision_check_and_payload_is_bound() {
    let f = f();
    let mut p = ordered(&f, (3, 1));
    let receipt = receive(&f, &p, vec![(0, 3, "A")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let request = input(&p, &receipt, vec![(0, 1)]);
    let a = returns::create(&f.db, &request).unwrap();
    let b = returns::create(&f.db, &request).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(a.lines[0].movement_id, b.lines[0].movement_id);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        2
    );
    let mut changed = request;
    changed.lines[0].quantity = 2;
    assert!(returns::create(&f.db, &changed).is_err());
    assert_eq!(count(&f.db, "purchase_returns"), 1);
}

#[test]
fn acquisition_cost_and_original_receipt_are_immutable() {
    let f = f();
    let mut p = ordered(&f, (1, 1));
    let receipt = receiving::receive(
        &f.db,
        &ReceivePurchase {
            id: id(),
            purchase_order_id: p.id.clone(),
            expected_revision: p.revision,
            request_id: id(),
            branch_id: f.branch.clone(),
            created_by: None,
            lines: vec![PurchaseReceiptLineInput {
                purchase_order_line_id: p.lines[0].id.clone(),
                quantity: 1,
                lot_number: None,
                expiry_date: None,
                acquisition_cost_minor: Some(777),
            }],
        },
    )
    .unwrap();
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let before = receiving::get(&f.db, &receipt.id).unwrap();
    let ret = returns::create(&f.db, &input(&p, &receipt, vec![(0, 1)])).unwrap();
    let after = receiving::get(&f.db, &receipt.id).unwrap();
    assert_eq!(ret.lines[0].acquisition_cost_minor, 777);
    assert_eq!(before.lines[0].quantity, after.lines[0].quantity);
    assert_eq!(before.lines[0].movement_id, after.lines[0].movement_id);
    let c = f.db.lock().unwrap();
    assert!(c
        .execute(
            "UPDATE purchase_receipt_lines SET quantity=2 WHERE id=?1",
            [&receipt.lines[0].id]
        )
        .is_err());
    assert!(c
        .execute("DELETE FROM purchase_returns WHERE id=?1", [&ret.id])
        .is_err());
}

#[test]
fn history_detail_filters_and_deterministic_pagination_work() {
    let f = f();
    let mut p = ordered(&f, (3, 1));
    let receipt = receive(&f, &p, vec![(0, 3, "A")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let ret = returns::create(&f.db, &input(&p, &receipt, vec![(0, 1)])).unwrap();
    let detail = returns::get(&f.db, &ret.id).unwrap();
    assert_eq!(detail.receipt_id, receipt.id);
    assert_eq!(detail.supplier_id, f.supplier);
    assert_eq!(detail.lines[0].receipt_line_id, receipt.lines[0].id);
    let page = returns::list(
        &f.db,
        &PurchaseReturnQuery {
            purchase_order_id: Some(p.id),
            supplier_id: Some(f.supplier),
            receipt_id: Some(receipt.id),
            branch_id: Some(f.branch),
            search: Some(ret.return_number),
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((page.total, page.items.len()), (1, 1));
}

#[test]
fn concurrent_requests_cannot_over_return_or_make_stock_negative() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("returns.db");
    let f = setup(AppDb::open(&path).unwrap());
    let mut p = ordered(&f, (5, 1));
    let receipt = receive(&f, &p, vec![(0, 5, "A")]);
    p = purchasing::detail(&f.db, &p.id).unwrap();
    let a = input(&p, &receipt, vec![(0, 5)]);
    let mut b = a.clone();
    b.id = id();
    b.request_id = id();
    drop(f.db);
    let pa = path.clone();
    let pb = path.clone();
    let x = thread::spawn(move || returns::create(&AppDb::open(Path::new(&pa)).unwrap(), &a));
    let y = thread::spawn(move || returns::create(&AppDb::open(Path::new(&pb)).unwrap(), &b));
    let results = [x.join().unwrap(), y.join().unwrap()];
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let db = AppDb::open(&path).unwrap();
    assert_eq!(count(&db, "purchase_returns"), 1);
    assert_eq!(db.lock().unwrap().query_row("SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE movement_type='PURCHASE_RETURN'",[],|r|r.get::<_,i64>(0)).unwrap(),-5);
}

#[test]
fn return_numbers_are_unique_across_concurrent_receipts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("return-numbers.db");
    let f = setup(AppDb::open(&path).unwrap());
    let mut p1 = ordered(&f, (1, 1));
    let r1 = receive(&f, &p1, vec![(0, 1, "A")]);
    p1 = purchasing::detail(&f.db, &p1.id).unwrap();
    let mut p2 = ordered(&f, (1, 1));
    let r2 = receive(&f, &p2, vec![(0, 1, "B")]);
    p2 = purchasing::detail(&f.db, &p2.id).unwrap();
    let a = input(&p1, &r1, vec![(0, 1)]);
    let b = input(&p2, &r2, vec![(0, 1)]);
    drop(f.db);
    let pa = path.clone();
    let pb = path.clone();
    let x = thread::spawn(move || {
        returns::create(&AppDb::open(&pa).unwrap(), &a)
            .unwrap()
            .return_number
    });
    let y = thread::spawn(move || {
        returns::create(&AppDb::open(&pb).unwrap(), &b)
            .unwrap()
            .return_number
    });
    let a = x.join().unwrap();
    let b = y.join().unwrap();
    assert_ne!(a, b);
    assert!(a.starts_with("PRT-") && b.starts_with("PRT-"));
}
