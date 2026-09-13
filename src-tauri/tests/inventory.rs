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
    inventory::{balances, batches, expiry, levels, models::*, movements, transfers},
};
use rusqlite::{params, Connection};

#[test]
fn migration_four_failure_rolls_back_pending_inventory_schema() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::apply(&mut c, &MIGRATIONS[..3]).unwrap();
    c.execute_batch("CREATE TABLE inventory_levels(marker TEXT); INSERT INTO inventory_levels VALUES('retain');").unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='inventory_batches'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT marker FROM inventory_levels", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "retain"
    );
}

#[test]
fn transfer_header_cannot_commit_without_its_pair() {
    let f = setup();
    let a = opening(&f, 10);
    let mut b = input(&f);
    b.branch_id = f.other.clone();
    let destination = batches::create(&f.db, &b).unwrap();
    let mut c = f.db.lock().unwrap();
    let tx = c.transaction().unwrap();
    tx.execute("INSERT INTO inventory_transfers(id,source_branch_id,destination_branch_id,product_package_id,source_batch_id,destination_batch_id,quantity,out_movement_id,in_movement_id,reason) VALUES (?1,?2,?3,?4,?5,?6,1,?7,?8,'Incomplete')",params![uuid(),f.branch,f.other,f.package,a.batch_id,destination.id,a.id,uuid()]).unwrap();
    assert!(tx.commit().is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM inventory_transfers", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn fefo_uses_receipt_time_for_equal_expiry() {
    let f = setup();
    let mut ids = vec![];
    for date in ["2030-01-02T00:00:00.000Z", "2030-01-01T00:00:00.000Z"] {
        let mut b = input(&f);
        b.expiry_date = Some("2030-06-01".into());
        b.received_at = Some(date.into());
        let m = movements::opening(
            &f.db,
            &OpeningStock {
                batch: b,
                request_id: uuid(),
                quantity: 1,
                reason: "Tie ordering".into(),
                created_by: None,
            },
        )
        .unwrap();
        ids.push(m.batch_id);
    }
    let rows = expiry::fefo(&f.db, &f.branch, &f.package, "2030-01-03", None, None).unwrap();
    assert_eq!(rows[0].batch.id, ids[1]);
    assert_eq!(rows[1].batch.id, ids[0]);
}

#[test]
fn sqlite_composite_fk_rejects_batch_in_another_branch() {
    let f = setup();
    let a = opening(&f, 10);
    assert!(f.db.lock().unwrap().execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reason) VALUES (?1,?2,?3,?4,'ADJUSTMENT_IN',1,'Wrong branch')",params![uuid(),f.other,f.package,a.batch_id]).is_err());
}

#[test]
fn safe_integer_balance_cap_is_enforced_across_batches() {
    let f = setup();
    opening(&f, 9_007_199_254_740_991);
    assert!(movements::opening(
        &f.db,
        &OpeningStock {
            batch: input(&f),
            request_id: uuid(),
            quantity: 1,
            reason: "Overflow".into(),
            created_by: None
        }
    )
    .is_err());
    assert_eq!(count(&f.db, "inventory_batches"), 1);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        9_007_199_254_740_991
    );
}

#[test]
fn unknown_expiry_write_off_requires_an_explicit_reason() {
    let f = setup();
    let a = opening(&f, 10);
    let mut p = op(&f, &a.batch_id, 1);
    p.reason = "".into();
    assert!(movements::write_off(&f.db, &p).is_err());
    p.reason = "Expiry confirmed manually; date unavailable".into();
    let m = movements::write_off(&f.db, &p).unwrap();
    assert_eq!(m.movement_type, MovementType::Expiry);
    assert_eq!(m.quantity_delta, -1);
}

#[test]
fn future_movement_types_are_typed_but_not_generic_ipc_operations() {
    for kind in [
        "OPENING",
        "PURCHASE_RECEIPT",
        "SALE",
        "SALE_RETURN",
        "PURCHASE_RETURN",
        "ADJUSTMENT_IN",
        "ADJUSTMENT_OUT",
        "DAMAGE",
        "EXPIRY",
        "TRANSFER_OUT",
        "TRANSFER_IN",
        "STOCK_COUNT_CORRECTION",
    ] {
        let m: MovementType = serde_json::from_value(serde_json::json!(kind)).unwrap();
        assert_eq!(m.as_str(), kind);
    }
    assert!(serde_json::from_value::<MovementType>(serde_json::json!("ANYTHING")).is_err());
}

#[test]
fn query_pages_are_bounded_and_balances_use_indexes() {
    let f = setup();
    let a = opening(&f, 1);
    let mut query = q(&f);
    query.limit = Some(201);
    assert!(movements::list(&f.db, &query).is_err());
    query.limit = Some(1);
    query.offset = Some(1);
    assert!(movements::list(&f.db, &query).unwrap().is_empty());
    let c = f.db.lock().unwrap();
    let plan:String=c.query_row("EXPLAIN QUERY PLAN SELECT sum(quantity_delta) FROM inventory_movements WHERE batch_id=?1",[a.batch_id],|r|r.get(3)).unwrap();
    assert!(plan.contains("inventory_movements_batch"));
}

#[test]
fn inactive_scope_cannot_post_but_existing_balances_remain_visible() {
    let f = setup();
    let a = opening(&f, 10);
    f.db.lock()
        .unwrap()
        .execute(
            "UPDATE product_packages SET is_active=0 WHERE id=?1",
            [&f.package],
        )
        .unwrap();
    assert!(movements::damage(&f.db, &op(&f, &a.batch_id, 1)).is_err());
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        10
    );
}

#[test]
fn competing_connections_cannot_overspend_the_same_batch() {
    let f = setup();
    let a = opening(&f, 10);
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("concurrent.db");
    f.db.lock()
        .unwrap()
        .backup(rusqlite::MAIN_DB, &path, None)
        .unwrap();
    let one = AppDb::open(&path).unwrap();
    let two = AppDb::open(&path).unwrap();
    let x = op(&f, &a.batch_id, 7);
    let y = op(&f, &a.batch_id, 7);
    let first = std::thread::spawn(move || movements::damage(&one, &x));
    let second = std::thread::spawn(move || movements::damage(&two, &y));
    let a = first.join().unwrap();
    let b = second.join().unwrap();
    assert_ne!(a.is_ok(), b.is_ok());
    let reopened = AppDb::open(&path).unwrap();
    assert_eq!(
        balances::package(&reopened, &f.branch, &f.package)
            .unwrap()
            .quantity,
        3
    );
}
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct Fixture {
    db: AppDb,
    branch: String,
    other: String,
    package: String,
}
fn setup() -> Fixture {
    let db = AppDb::in_memory().unwrap();
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let other = branches::create_branch(
        &db,
        &CreateBranch {
            code: "SECOND".into(),
            name: "Second".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap()
    .id;
    let p = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Test".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let package = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: p.id,
            package_label: "Package".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    Fixture {
        db,
        branch,
        other,
        package,
    }
}
fn input(f: &Fixture) -> CreateBatch {
    CreateBatch {
        branch_id: f.branch.clone(),
        product_package_id: f.package.clone(),
        lot_number: Some("Test lot".into()),
        cost_price_minor: Some(2250),
        ..Default::default()
    }
}
fn opening(f: &Fixture, quantity: i64) -> InventoryMovement {
    movements::opening(
        &f.db,
        &OpeningStock {
            batch: input(f),
            request_id: uuid(),
            quantity,
            reason: "Explicit test opening".into(),
            created_by: None,
        },
    )
    .unwrap()
}
fn op(f: &Fixture, batch: &str, quantity: i64) -> StockOperation {
    StockOperation {
        request_id: uuid(),
        branch_id: f.branch.clone(),
        product_package_id: f.package.clone(),
        batch_id: batch.into(),
        quantity,
        reason: "Test adjustment".into(),
        created_by: None,
    }
}
fn q(f: &Fixture) -> InventoryQuery {
    InventoryQuery {
        branch_id: f.branch.clone(),
        product_package_id: None,
        batch_id: None,
        limit: Some(200),
        offset: None,
    }
}
fn count(db: &AppDb, table: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[test]
fn zero_inventory_is_not_seeded_from_catalog() {
    let f = setup();
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        0
    );
    assert!(balances::list(&f.db, &q(&f)).unwrap().is_empty());
    assert_eq!(count(&f.db, "inventory_movements"), 0);
}
#[test]
fn batch_preserves_nullable_fields_and_cost_without_unsafe_lot_uniqueness() {
    let f = setup();
    let a = batches::create(&f.db, &input(&f)).unwrap();
    let b = batches::create(&f.db, &input(&f)).unwrap();
    assert_ne!(a.id, b.id);
    assert_eq!(a.cost_price_minor, Some(2250));
    assert_eq!(a.expiry_date, None);
    assert_eq!(
        balances::batch(&f.db, &f.branch, &a.id).unwrap().quantity,
        0
    );
    assert!(f
        .db
        .lock()
        .unwrap()
        .execute(
            "UPDATE inventory_batches SET cost_price_minor=99 WHERE id=?1",
            [a.id]
        )
        .is_err());
}
#[test]
fn opening_stock_and_multiple_batches_derive_balances() {
    let f = setup();
    let a = opening(&f, 10);
    let b = opening(&f, 5);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        15
    );
    assert_eq!(
        balances::batch(&f.db, &f.branch, &a.batch_id)
            .unwrap()
            .quantity,
        10
    );
    assert_eq!(
        balances::batch(&f.db, &f.branch, &b.batch_id)
            .unwrap()
            .quantity,
        5
    );
    assert_eq!(balances::list(&f.db, &q(&f)).unwrap()[0].quantity, 15);
}
#[test]
fn adjustments_damage_and_compensation_are_append_only() {
    let f = setup();
    let a = opening(&f, 10);
    let mut adjust = Adjustment {
        operation: op(&f, &a.batch_id, 3),
        direction: AdjustmentDirection::In,
    };
    movements::adjust(&f.db, &adjust).unwrap();
    adjust.operation.request_id = uuid();
    adjust.direction = AdjustmentDirection::Out;
    movements::adjust(&f.db, &adjust).unwrap();
    movements::damage(&f.db, &op(&f, &a.batch_id, 2)).unwrap();
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        8
    );
    let c = f.db.lock().unwrap();
    assert!(c
        .execute("UPDATE inventory_movements SET quantity_delta=20", [])
        .is_err());
    assert!(c.execute("DELETE FROM inventory_movements", []).is_err());
    drop(c);
    assert_eq!(movements::list(&f.db, &q(&f)).unwrap().len(), 4);
}
#[test]
fn no_negative_stock_is_enforced_per_batch_not_just_package() {
    let f = setup();
    let a = opening(&f, 1);
    opening(&f, 100);
    assert!(movements::damage(&f.db, &op(&f, &a.batch_id, 2)).is_err());
    assert_eq!(
        balances::batch(&f.db, &f.branch, &a.batch_id)
            .unwrap()
            .quantity,
        1
    );
    let c = f.db.lock().unwrap();
    assert!(c.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reason) VALUES (?1,?2,?3,?4,'DAMAGE',-2,'Direct invalid')",params![uuid(),f.branch,f.package,a.batch_id]).is_err());
}
#[test]
fn branches_and_batches_cannot_be_crossed() {
    let f = setup();
    let a = opening(&f, 10);
    assert_eq!(
        balances::package(&f.db, &f.other, &f.package)
            .unwrap()
            .quantity,
        0
    );
    assert!(balances::batch(&f.db, &f.other, &a.batch_id).is_err());
    let mut wrong = op(&f, &a.batch_id, 1);
    wrong.branch_id = f.other.clone();
    assert!(movements::damage(&f.db, &wrong).is_err());
    wrong.branch_id = f.branch.clone();
    wrong.product_package_id = uuid();
    assert!(movements::damage(&f.db, &wrong).is_err());
}
#[test]
fn repeated_request_and_opening_are_rejected() {
    let f = setup();
    let a = opening(&f, 10);
    let damage = op(&f, &a.batch_id, 1);
    movements::damage(&f.db, &damage).unwrap();
    assert!(movements::damage(&f.db, &damage).is_err());
    assert!(movements::opening_existing(&f.db, &op(&f, &a.batch_id, 2)).is_err());
    assert_eq!(
        balances::batch(&f.db, &f.branch, &a.batch_id)
            .unwrap()
            .quantity,
        9
    );
}
#[test]
fn required_reason_integer_quantity_and_cost_validation() {
    let f = setup();
    let a = opening(&f, 10);
    let mut bad = op(&f, &a.batch_id, 0);
    assert!(movements::damage(&f.db, &bad).is_err());
    bad.quantity = 1;
    bad.reason = " ".into();
    assert!(movements::damage(&f.db, &bad).is_err());
    let mut batch = input(&f);
    batch.cost_price_minor = Some(-1);
    assert!(batches::create(&f.db, &batch).is_err());
    batch.cost_price_minor = None;
    batch.expiry_date = Some("2026-02-30".into());
    assert!(batches::create(&f.db, &batch).is_err());
    assert!(serde_json::from_str::<StockOperation>(r#"{"requestId":"x","branchId":"x","productPackageId":"x","batchId":"x","quantity":1.5,"reason":"x"}"#).is_err());
}
#[test]
fn opening_batch_rolls_back_if_movement_fails() {
    let f = setup();
    let result = movements::opening(
        &f.db,
        &OpeningStock {
            batch: input(&f),
            request_id: uuid(),
            quantity: 0,
            reason: "Bad".into(),
            created_by: None,
        },
    );
    assert!(result.is_err());
    assert_eq!(count(&f.db, "inventory_batches"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
}
#[test]
fn reorder_upsert_and_low_stock_include_zero_at_threshold() {
    let f = setup();
    let mut i = SetLevel {
        branch_id: f.branch.clone(),
        product_package_id: f.package.clone(),
        reorder_level: 10,
        minimum_stock: Some(2),
        maximum_stock: Some(20),
    };
    let a = levels::set(&f.db, &i).unwrap();
    assert_eq!(levels::low(&f.db, &q(&f)).unwrap()[0].quantity, 0);
    opening(&f, 10);
    assert_eq!(levels::low(&f.db, &q(&f)).unwrap()[0].quantity, 10);
    i.reorder_level = 9;
    assert_eq!(levels::set(&f.db, &i).unwrap().id, a.id);
    assert!(levels::low(&f.db, &q(&f)).unwrap().is_empty());
    assert_eq!(count(&f.db, "inventory_levels"), 1);
    i.minimum_stock = Some(21);
    assert!(levels::set(&f.db, &i).is_err());
}
#[test]
fn expiry_and_fefo_sort_known_dates_before_unknown_and_exclude_expired() {
    let f = setup();
    let mut ids = vec![];
    for expiry in [
        None,
        Some("2030-03-01"),
        Some("2030-02-01"),
        Some("2000-01-01"),
    ] {
        let mut b = input(&f);
        b.expiry_date = expiry.map(str::to_owned);
        let m = movements::opening(
            &f.db,
            &OpeningStock {
                batch: b,
                request_id: uuid(),
                quantity: 2,
                reason: "Expiry test".into(),
                created_by: None,
            },
        )
        .unwrap();
        ids.push(m.batch_id);
    }
    let fefo = expiry::fefo(&f.db, &f.branch, &f.package, "2030-01-15", None, None).unwrap();
    assert_eq!(
        fefo.iter().map(|r| r.batch.id.clone()).collect::<Vec<_>>(),
        vec![ids[2].clone(), ids[1].clone(), ids[0].clone()]
    );
    let mut query = ExpiryQuery {
        branch_id: f.branch.clone(),
        as_of: "2030-01-15".into(),
        within_days: 30,
        expired_only: false,
        limit: None,
        offset: None,
    };
    assert_eq!(expiry::expiring(&f.db, &query).unwrap()[0].batch.id, ids[2]);
    query.expired_only = true;
    assert_eq!(expiry::expiring(&f.db, &query).unwrap()[0].batch.id, ids[3]);
    movements::write_off(&f.db, &op(&f, &ids[3], 2)).unwrap();
    assert!(expiry::expiring(&f.db, &query).unwrap().is_empty());
    assert!(movements::write_off(&f.db, &op(&f, &ids[1], 1)).is_err());
}
#[test]
fn transfer_pair_is_atomic_preserves_cost_and_audit_reference() {
    let f = setup();
    let a = opening(&f, 10);
    let input = TransferStock {
        request_id: uuid(),
        source_branch_id: f.branch.clone(),
        destination_branch_id: f.other.clone(),
        product_package_id: f.package.clone(),
        source_batch_id: a.batch_id.clone(),
        quantity: 4,
        reason: "Transfer test".into(),
        created_by: None,
    };
    let t = transfers::transfer(&f.db, &input).unwrap();
    assert_eq!(t.out_movement.quantity_delta, -4);
    assert_eq!(t.in_movement.quantity_delta, 4);
    assert_eq!(t.in_movement.transfer_id, t.out_movement.transfer_id);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        6
    );
    assert_eq!(
        balances::package(&f.db, &f.other, &f.package)
            .unwrap()
            .quantity,
        4
    );
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row(
                "SELECT cost_price_minor FROM inventory_batches WHERE id=?1",
                [t.destination_batch_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2250
    );
    assert!(transfers::transfer(&f.db, &input).is_err());
}
#[test]
fn transfer_second_leg_failure_rolls_back_all_writes() {
    let f = setup();
    let a = opening(&f, 10);
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_transfer BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='TRANSFER_IN' BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let i = TransferStock {
        request_id: uuid(),
        source_branch_id: f.branch.clone(),
        destination_branch_id: f.other.clone(),
        product_package_id: f.package.clone(),
        source_batch_id: a.batch_id,
        quantity: 4,
        reason: "Transfer".into(),
        created_by: None,
    };
    assert!(transfers::transfer(&f.db, &i).is_err());
    assert_eq!(count(&f.db, "inventory_batches"), 1);
    assert_eq!(count(&f.db, "inventory_transfers"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 1);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        10
    );
}
#[test]
fn migration_from_v3_preserves_catalog_branch_and_import_metadata() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("v3.db");
    {
        let mut c = Connection::open(&path).unwrap();
        migrations::apply(&mut c, &MIGRATIONS[..3]).unwrap();
        c.execute_batch("INSERT INTO branches(id,code,name) VALUES('b','MAIN','Main'); INSERT INTO products(id,commercial_name_en) VALUES('p','Existing'); INSERT INTO product_packages(id,product_id,package_label) VALUES('pk','p','Package'); INSERT INTO catalog_import_runs(id,source_hash,source_path,started_at,total_rows,status,report_path,backup_path) VALUES('run','hash','source','date',1,'complete','report','backup'); INSERT INTO catalog_import_rows(source_id,fingerprint,identity_key,run_id,source_row,product_id,package_id) VALUES('source','fingerprint','identity','run',1,'p','pk');").unwrap();
    }
    let db = AppDb::open(&path).unwrap();
    for t in [
        "branches",
        "products",
        "product_packages",
        "catalog_import_runs",
        "catalog_import_rows",
    ] {
        assert_eq!(count(&db, t), 1);
    }
    assert_eq!(count(&db, "inventory_movements"), 0);
    assert_eq!(count(&db, "_schema_version"), 4);
}
#[test]
fn sqlite_rejects_invalid_type_sign_foreign_keys_and_unpaired_transfer() {
    let f = setup();
    let a = opening(&f, 10);
    let c = f.db.lock().unwrap();
    for (kind, delta) in [
        ("NONSENSE", "1"),
        ("DAMAGE", "1"),
        ("OPENING", "-1"),
        ("ADJUSTMENT_IN", "0"),
        ("ADJUSTMENT_IN", "1.5"),
        ("TRANSFER_OUT", "-1"),
    ] {
        assert!(c.execute(&format!("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reason) VALUES (?1,?2,?3,?4,?5,{delta},'Bad')"),params![uuid(),f.branch,f.package,a.batch_id,kind]).is_err());
    }
    assert!(c
        .execute(
            "INSERT INTO inventory_batches(id,branch_id,product_package_id) VALUES (?1,?2,?3)",
            params![uuid(), uuid(), f.package]
        )
        .is_err());
}
