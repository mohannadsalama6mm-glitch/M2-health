use m2_health_lib::db::{
    connection::AppDb,
    models::{
        branch::CreateBranch,
        catalog::{CreateProductFull, ProductPackageInput},
        inventory::*,
    },
    repositories::{branches, movements, product_packages, products, stocks},
    services::{counts as count_service, inventory as stock_service, settings as setting_service},
};

fn db() -> AppDb {
    AppDb::in_memory().unwrap()
}
fn branch(db: &AppDb, code: &str, name: &str) -> m2_health_lib::db::models::branch::Branch {
    branches::create_branch(
        db,
        &CreateBranch {
            code: code.into(),
            name: name.into(),
            ..Default::default()
        },
    )
    .unwrap()
}
/// Creates a product with one active package and returns the package id.
fn package(db: &AppDb, name: &str, label: &str) -> String {
    let product = products::create_full(
        db,
        &CreateProductFull {
            commercial_name_en: Some(name.into()),
            packages: vec![ProductPackageInput {
                package_label: label.to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();
    product_packages::list(db, &product.id, false)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .id
}
fn open(
    db: &AppDb,
    branch_id: &str,
    package_id: &str,
    number: &str,
    expiry: &str,
    qty: i64,
) -> String {
    stock_service::opening_stock(
        db,
        &OpeningStockInput {
            branch_id: branch_id.into(),
            product_package_id: package_id.into(),
            batch_number: number.into(),
            expiry_date: expiry.into(),
            quantity: qty,
            reason: "Initial stock".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .batch_id
}
fn balance(db: &AppDb, batch_id: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(
            "SELECT COALESCE(quantity,0) FROM stock_balances WHERE batch_id=?1",
            [batch_id],
            |r| r.get(0),
        )
        .unwrap()
}
fn movement_count(db: &AppDb, batch_id: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM stock_movements WHERE batch_id=?1",
            [batch_id],
            |r| r.get(0),
        )
        .unwrap()
}
fn date_offset(db: &AppDb, modifier: &str) -> String {
    db.lock()
        .unwrap()
        .query_row("SELECT date('now', ?1)", [modifier], |r| r.get(0))
        .unwrap()
}

#[test]
fn empty_inventory_reports_empty_everywhere() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let page = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(page.items.is_empty());
    let summary = stocks::summary(&db, &b.id).unwrap();
    assert_eq!(
        (summary.in_stock, summary.low, summary.out_of_stock),
        (0, 0, 0)
    );
    assert_eq!(summary.value_minor, 0);
    assert!(movements::list(&db, &MovementQuery::default())
        .unwrap()
        .items
        .is_empty());
    assert_eq!(
        stocks::expiry(
            &db,
            &ExpiryQuery {
                branch_id: b.id.clone(),
                ..Default::default()
            }
        )
        .unwrap()
        .items
        .len(),
        0
    );
    assert_eq!(
        stocks::low_stock(
            &db,
            &LowStockQuery {
                branch_id: b.id.clone(),
                ..Default::default()
            }
        )
        .unwrap()
        .items
        .len(),
        0
    );
    let _ = pk;
}

#[test]
fn opening_stock_creates_batch_and_movement_and_derives_balance() {
    let db = db();
    let b = branch(&db, "CAIRO", "Cairo Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let movement = stock_service::opening_stock(
        &db,
        &OpeningStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            batch_number: "PN-2608".into(),
            expiry_date: "2027-08-31".into(),
            cost_price_minor: Some(2400),
            quantity: 120,
            reason: "Initial stock".into(),
            user: None,
        },
    )
    .unwrap();
    assert_eq!(movement.movement_type, "opening");
    assert_eq!(movement.quantity_delta, 120);
    assert_eq!(movement.cost_price_minor, Some(2400));
    let allocated = stocks::fefo_allocation(&db, &b.id, &pk, None).unwrap();
    assert_eq!(allocated.len(), 1);
    assert_eq!(allocated[0].batch_id, movement.batch_id);
    assert_eq!(allocated[0].available, 120);
    assert_eq!(allocated[0].batch_number, "PN-2608");
    assert_eq!(allocated[0].expiry_date, "2027-08-31");
    assert_eq!(balance(&db, &movement.batch_id), 120);
    // Stock is not on the product or package records.
    let c = db.lock().unwrap();
    let cols: Vec<String> = c
        .prepare("SELECT name FROM pragma_table_info('products')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    for forbidden in ["quantity", "stock", "batch", "expiry"] {
        assert!(
            !cols.iter().any(|c| c.as_str() == forbidden),
            "{forbidden} leaked onto products"
        );
    }
    let package_rows: i64 = c
        .query_row(
            "SELECT count(*) FROM inventory_batches WHERE product_package_id=?1",
            [&pk],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(package_rows, 1);
}

#[test]
fn duplicate_lot_is_reused_not_duplicated() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Paracetamol", "10 tablets");
    let first = open(&db, &b.id, &pk, "B-1", "2027-01-01", 10);
    let second = open(&db, &b.id, &pk, "B-1", "2027-01-01", 5);
    assert_eq!(first, second);
    assert_eq!(balance(&db, &first), 15);
    assert_eq!(movement_count(&db, &first), 2);
    let count: i64 = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM inventory_batches WHERE branch_id=?1 AND product_package_id=?2",
            [&b.id, &pk],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    // Same lot number with a different expiry is a distinct batch.
    let other = open(&db, &b.id, &pk, "B-1", "2028-01-01", 3);
    assert_ne!(first, other);
    assert_eq!(balance(&db, &other), 3);
}

#[test]
fn branch_inventory_is_isolated_and_persists_independently() {
    let db = db();
    let a = branch(&db, "MAIN", "Main Pharmacy");
    let b2 = branch(&db, "MADI", "Mandi Pharmacy");
    let pk = package(&db, "Brufen", "20 tablets");
    let batch_a = open(&db, &a.id, &pk, "B1", "2027-06-01", 30);
    let page_a = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: a.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page_a.items[0].quantity, 30);
    let page_b = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b2.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page_b.items.len(), 1);
    assert_eq!(page_b.items[0].quantity, 0);
    assert_eq!(page_b.items[0].status, "out_of_stock");
    // Adjusting branch A cannot touch branch B.
    stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: a.id.clone(),
            product_package_id: pk.clone(),
            batch_id: Some(batch_a.clone()),
            new_quantity: 25,
            reason: "Count review".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(balance(&db, &batch_a), 25);
    assert_eq!(page_b.items[0].quantity, 0);
    assert_eq!(balance_check(&db, &pk, &b2.id), 0);
}

fn balance_check(db: &AppDb, package_id: &str, branch_id: &str) -> i64 {
    let page = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: branch_id.into(),
            ..Default::default()
        },
    )
    .unwrap();
    page.items
        .iter()
        .find(|r| r.package_id == package_id)
        .map(|r| r.quantity)
        .unwrap_or(0)
}

#[test]
fn adjustment_increase_and_decrease_preserve_history() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Cetirizine", "10 tablets");
    let batch = open(&db, &b.id, &pk, "C-1", "2026-12-01", 10);
    let result = stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            batch_id: Some(batch.clone()),
            new_quantity: 15,
            reason: "Opening correction".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(result.new_quantity, 15);
    assert_eq!(result.movements.len(), 1);
    assert_eq!(result.movements[0].movement_type, "adjustment");
    assert_eq!(result.movements[0].quantity_delta, 5);
    assert_eq!(balance(&db, &batch), 15);
    // Decrease
    stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            batch_id: Some(batch.clone()),
            new_quantity: 12,
            reason: "Dispensing review".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(balance(&db, &batch), 12);
    // No-op posts nothing
    let noop = stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            batch_id: Some(batch.clone()),
            new_quantity: 12,
            reason: "Same value".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(noop.movements.is_empty());
    assert_eq!(movement_count(&db, &batch), 3);
}

#[test]
fn adjust_increase_without_batch_targets_general_lot() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Vitamin C", "30 tablets");
    let result = stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            new_quantity: 20,
            reason: "Damaged packaging found".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(result.movements.len(), 1);
    let allocated = stocks::fefo_allocation(&db, &b.id, &pk, None).unwrap();
    assert_eq!(allocated[0].batch_number, "GENERAL");
    assert_eq!(allocated[0].available, 20);
}

#[test]
fn write_off_damage_and_expired_post_ledger_entries() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Cough syrup", "100 ml");
    let batch = open(
        &db,
        &b.id,
        &pk,
        "S-9",
        date_offset(&db, "+30 days").as_str(),
        25,
    );
    stock_service::write_off(
        &db,
        &WriteOffInput {
            batch_id: batch.clone(),
            kind: "damage".into(),
            quantity: 2,
            reason: "Broken bottle".into(),
            user: None,
        },
    )
    .unwrap();
    stock_service::write_off(
        &db,
        &WriteOffInput {
            batch_id: batch.clone(),
            kind: "expired".into(),
            quantity: 3,
            reason: "Past shelf life".into(),
            user: None,
        },
    )
    .unwrap();
    assert_eq!(balance(&db, &batch), 20);
    let page = movements::list(
        &db,
        &MovementQuery {
            branch_id: Some(b.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.total, 3);
    let kinds: Vec<&str> = page
        .items
        .iter()
        .map(|m| m.movement_type.as_str())
        .collect();
    assert!(kinds.contains(&"opening"));
    assert!(kinds.contains(&"damage"));
    assert!(kinds.contains(&"expired"));
}

#[test]
fn write_off_insufficient_fails_without_partial_records() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let batch = open(&db, &b.id, &pk, "PN-1", "2027-08-31", 5);
    let before = movement_count(&db, &batch);
    let err = stock_service::write_off(
        &db,
        &WriteOffInput {
            batch_id: batch.clone(),
            kind: "expired".into(),
            quantity: 50,
            reason: "Clearance".into(),
            user: None,
        },
    )
    .err()
    .unwrap();
    assert_eq!(err.code, m2_health_lib::error::ErrorCode::Conflict);
    assert_eq!(movement_count(&db, &batch), before);
    assert_eq!(balance(&db, &batch), 5);
}

#[test]
fn cross_branch_adjustment_is_rejected() {
    let db = db();
    let a = branch(&db, "MAIN", "Main Pharmacy");
    let b2 = branch(&db, "MADI", "Mandi Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let batch = open(&db, &a.id, &pk, "PN-2", "2027-08-31", 10);
    let err = stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b2.id.clone(),
            product_package_id: pk.clone(),
            batch_id: Some(batch.clone()),
            new_quantity: 100,
            reason: "Wrong branch".into(),
            ..Default::default()
        },
    )
    .err()
    .unwrap();
    assert_eq!(err.code, m2_health_lib::error::ErrorCode::Validation);
    assert_eq!(balance(&db, &batch), 10);
}

#[test]
fn transfer_out_in_preserves_lot_and_cost() {
    let db = db();
    let a = branch(&db, "MAIN", "Main Pharmacy");
    let b2 = branch(&db, "MADI", "Mandi Pharmacy");
    let pk = package(&db, "Augmentin", "14 tablets");
    let batch = stock_service::opening_stock(
        &db,
        &OpeningStockInput {
            branch_id: a.id.clone(),
            product_package_id: pk.clone(),
            batch_number: "AM-7".into(),
            expiry_date: "2027-03-01".into(),
            cost_price_minor: Some(12500),
            quantity: 40,
            reason: "Initial".into(),
            user: None,
        },
    )
    .unwrap()
    .batch_id;
    let result = stock_service::transfer_stock(
        &db,
        &TransferInput {
            from_branch_id: a.id.clone(),
            to_branch_id: b2.id.clone(),
            product_package_id: pk.clone(),
            batch_id: batch.clone(),
            quantity: 15,
            reason: "Replenish Maadi".into(),
            user: None,
        },
    )
    .unwrap();
    assert_eq!(result.out.movement_type, "transfer_out");
    assert_eq!(result.incoming.movement_type, "transfer_in");
    assert_eq!(result.out.quantity_delta, -15);
    assert_eq!(result.incoming.quantity_delta, 15);
    assert_ne!(result.from_batch_id, result.to_batch_id);
    assert_eq!(balance(&db, &batch), 25);
    assert_eq!(balance(&db, &result.to_batch_id), 15);
    let to = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT batch_number, expiry_date, cost_price_minor FROM inventory_batches WHERE id=?1",
            [&result.to_batch_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(to.0, "AM-7");
    assert_eq!(to.1, "2027-03-01");
    assert_eq!(to.2, Some(12500));
    assert_eq!(balance_check(&db, &pk, &a.id), 25);
    assert_eq!(balance_check(&db, &pk, &b2.id), 15);
}

#[test]
fn transfer_validation_blocks_same_branch_and_oversell() {
    let db = db();
    let a = branch(&db, "MAIN", "Main Pharmacy");
    let b2 = branch(&db, "MADI", "Mandi Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let batch = open(&db, &a.id, &pk, "PN-3", "2027-08-31", 8);
    let same = transfer_error(&db, &a.id, &a.id, &pk, &batch, 1);
    assert_eq!(same, m2_health_lib::error::ErrorCode::Validation);
    let oversell = transfer_error(&db, &a.id, &b2.id, &pk, &batch, 99);
    assert_eq!(oversell, m2_health_lib::error::ErrorCode::Conflict);
    assert_eq!(balance(&db, &batch), 8);
}
fn transfer_error(
    db: &AppDb,
    from: &str,
    to: &str,
    pk: &str,
    batch: &str,
    qty: i64,
) -> m2_health_lib::error::ErrorCode {
    stock_service::transfer_stock(
        db,
        &TransferInput {
            from_branch_id: from.into(),
            to_branch_id: to.into(),
            product_package_id: pk.into(),
            batch_id: batch.into(),
            quantity: qty,
            reason: "Test".into(),
            user: None,
        },
    )
    .err()
    .unwrap()
    .code
}

#[test]
fn fefo_orders_earliest_expiry_first_and_no_expiry_last() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Salbutamol", "200 doses");
    let late = open(&db, &b.id, &pk, "SB-L", "2028-06-01", 10);
    let early = open(&db, &b.id, &pk, "SB-E", "2026-10-01", 10);
    let none = open(&db, &b.id, &pk, "SB-X", "", 10);
    let allocated = stocks::fefo_allocation(&db, &b.id, &pk, None).unwrap();
    assert_eq!(allocated.len(), 3);
    assert_eq!(allocated[0].batch_id, early);
    assert_eq!(allocated[1].batch_id, late);
    assert_eq!(allocated[2].batch_id, none);
    let bounded = stocks::fefo_allocation(&db, &b.id, &pk, Some(12)).unwrap();
    assert_eq!(bounded[0].take, 10);
    assert_eq!(bounded[1].take, 2);
    assert_eq!(bounded[2].take, 0);
    // Deterministic across calls
    let again = stocks::fefo_allocation(&db, &b.id, &pk, None).unwrap();
    assert_eq!(
        allocated
            .iter()
            .map(|a| a.batch_id.clone())
            .collect::<Vec<_>>(),
        again.iter().map(|a| a.batch_id.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn reorder_default_and_override_drive_low_stock() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Flucort", "10 tablets");
    open(&db, &b.id, &pk, "FC-1", "2027-01-01", 5);
    let rows = stocks::low_stock(
        &db,
        &LowStockQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(rows.total, 1);
    assert_eq!(rows.items[0].reorder_level, 10);
    assert_eq!(rows.items[0].status, "low");
    // Raising reorder above stock keeps it low; a higher stock clears it.
    setting_service::set_reorder(
        &db,
        &SetReorderInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            reorder_level: 3,
        },
    )
    .unwrap();
    assert!(stocks::low_stock(
        &db,
        &LowStockQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        }
    )
    .unwrap()
    .items
    .is_empty());
    let page = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.items[0].reorder_level, 3);
    // Zero reorder disables the threshold.
    setting_service::set_reorder(
        &db,
        &SetReorderInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            reorder_level: 0,
        },
    )
    .unwrap();
    assert!(stocks::low_stock(
        &db,
        &LowStockQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        }
    )
    .unwrap()
    .items
    .is_empty());
    // Zero stock is reported as out of stock.
    stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            new_quantity: 0,
            reason: "Sold out".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let rows = stocks::low_stock(
        &db,
        &LowStockQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(rows.items[0].status, "out_of_stock");
}

#[test]
fn expiry_detection_and_windows() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk1 = package(&db, "One", "10 tablets");
    let pk2 = package(&db, "Two", "10 tablets");
    let pk3 = package(&db, "Three", "10 tablets");
    open(
        &db,
        &b.id,
        &pk1,
        "E-1",
        date_offset(&db, "-1 day").as_str(),
        6,
    );
    open(
        &db,
        &b.id,
        &pk2,
        "N-1",
        date_offset(&db, "+5 days").as_str(),
        6,
    );
    open(
        &db,
        &b.id,
        &pk3,
        "F-1",
        date_offset(&db, "+60 days").as_str(),
        6,
    );
    let expired = stocks::expiry(
        &db,
        &ExpiryQuery {
            branch_id: b.id.clone(),
            window_days: Some(-1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(expired.total, 1);
    assert_eq!(expired.items[0].batch_number, "E-1");
    assert!(expired.items[0].days < 0);
    let thirty = stocks::expiry(
        &db,
        &ExpiryQuery {
            branch_id: b.id.clone(),
            window_days: Some(30),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(thirty.total, 1);
    assert_eq!(thirty.items[0].batch_number, "N-1");
    assert!(thirty.items[0].days >= 0);
    let ninety = stocks::expiry(
        &db,
        &ExpiryQuery {
            branch_id: b.id.clone(),
            window_days: Some(90),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(ninety.total, 2);
    let value = stocks::expiry(
        &db,
        &ExpiryQuery {
            branch_id: b.id.clone(),
            window_days: Some(30),
            ..Default::default()
        },
    )
    .unwrap();
    let _ = value.items[0].value_minor;
}

#[test]
fn stock_count_snapshot_counting_and_completion_posts_corrections() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let batch_a = open(&db, &b.id, &pk, "A-1", "2027-01-01", 10);
    let batch_b = open(&db, &b.id, &pk, "B-1", "2027-06-01", 5);
    let count = count_service::create_count(
        &db,
        &CreateCountInput {
            branch_id: b.id.clone(),
            scope: "Pain relief".into(),
            category_id: None,
        },
    )
    .unwrap();
    assert_eq!(count.status, "draft");
    let detail = m2_health_lib::db::repositories::counts::detail(&db, &count.id).unwrap();
    assert_eq!(detail.items.len(), 2);
    assert_eq!(
        detail.items[0].system_quantity + detail.items[1].system_quantity,
        15
    );
    let saved = count_service::save_item(
        &db,
        &SaveCountItemInput {
            stock_count_id: count.id.clone(),
            batch_id: batch_a.clone(),
            counted_quantity: 8,
        },
    )
    .unwrap();
    assert_eq!(saved.variance, -2);
    let detail = m2_health_lib::db::repositories::counts::detail(&db, &count.id).unwrap();
    assert_eq!(detail.count.status, "in_progress");
    assert!(detail
        .items
        .iter()
        .any(|i| i.batch_id == batch_a && i.variance == -2));
    let completed = count_service::complete_count(
        &db,
        &CompleteCountInput {
            stock_count_id: count.id.clone(),
        },
    )
    .unwrap();
    assert_eq!(completed.count.status, "completed");
    assert_eq!(balance(&db, &batch_a), 8);
    assert_eq!(balance(&db, &batch_b), 5);
    // Corrections are ledger movements with a reference, not rewrites.
    let page = movements::list(
        &db,
        &MovementQuery {
            branch_id: Some(b.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let corrections: Vec<_> = page
        .items
        .iter()
        .filter(|m| m.movement_type == "count_correction")
        .collect();
    assert_eq!(corrections.len(), 1);
    assert_eq!(corrections[0].outgoing, Some(2));
    assert_eq!(movement_count(&db, &batch_a), 2);
}

#[test]
fn count_snapshot_only_covers_branch_scope() {
    let db = db();
    let a = branch(&db, "MAIN", "Main Pharmacy");
    let b2 = branch(&db, "MADI", "Mandi Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    open(&db, &a.id, &pk, "A-1", "2027-01-01", 10);
    open(&db, &b2.id, &pk, "M-1", "2027-02-01", 4);
    let count = count_service::create_count(
        &db,
        &CreateCountInput {
            branch_id: a.id.clone(),
            scope: "All".into(),
            category_id: None,
        },
    )
    .unwrap();
    let detail = m2_health_lib::db::repositories::counts::detail(&db, &count.id).unwrap();
    assert_eq!(detail.items.len(), 1);
    assert_eq!(detail.items[0].system_quantity, 10);
}

#[test]
fn empty_count_completes_without_corrections() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let count = count_service::create_count(
        &db,
        &CreateCountInput {
            branch_id: b.id.clone(),
            scope: "Nothing".into(),
            category_id: None,
        },
    )
    .unwrap();
    let completed = count_service::complete_count(
        &db,
        &CompleteCountInput {
            stock_count_id: count.id.clone(),
        },
    )
    .unwrap();
    assert_eq!(completed.count.status, "completed");
    assert!(completed.items.is_empty());
    let page = movements::list(&db, &MovementQuery::default()).unwrap();
    assert!(page.items.is_empty());
}

#[test]
fn movements_are_never_overwritten_and_replay_balances() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    let batch = open(&db, &b.id, &pk, "PN-9", "2027-08-31", 30);
    stock_service::write_off(
        &db,
        &WriteOffInput {
            batch_id: batch.clone(),
            kind: "damage".into(),
            quantity: 5,
            reason: "Drop".into(),
            user: None,
        },
    )
    .unwrap();
    stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            new_quantity: 50,
            reason: "Restock".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let page = movements::list(
        &db,
        &MovementQuery {
            batch_id: Some(batch.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let deltas: Vec<i64> = page
        .items
        .iter()
        .map(|m| {
            m.incoming
                .map(|v| v as i64)
                .or(m.outgoing.map(|v| -(v as i64)))
                .unwrap()
        })
        .collect();
    assert_eq!(deltas.iter().sum::<i64>(), 25);
    assert_eq!(balance(&db, &batch), 25);
    // The increase without a batch is routed to the GENERAL lot, so the package
    // total reflects it while the original batch only holds its own history.
    let page = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.items[0].quantity, 50);
    // The ledger rows themselves are immutable history: re-running reads never mutate.
    assert_eq!(movement_count(&db, &batch), 2);
}

#[test]
fn restart_persists_inventory_state() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("inv.db");
    let package_id;
    let branch_id;
    {
        let db = AppDb::open(&path).unwrap();
        let b = branch(&db, "MAIN", "Main Pharmacy");
        branch_id = b.id.clone();
        let pk = package(&db, "Panadol", "20 tablets");
        package_id = pk.clone();
        open(&db, &branch_id, &pk, "PN-10", "2027-08-31", 42);
    }
    let db = AppDb::open(&path).unwrap();
    let allocated = stocks::fefo_allocation(&db, &branch_id, &package_id, None).unwrap();
    assert_eq!(allocated[0].available, 42);
    let page = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.items[0].quantity, 42);
}

#[test]
fn overview_statuses_and_filters() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk = package(&db, "Panadol", "20 tablets");
    open(&db, &b.id, &pk, "PN-1", "2027-01-01", 5);
    let low = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            status: Some("low".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(low.total, 1);
    let in_stock = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            status: Some("in_stock".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(in_stock.total, 0);
    stock_service::adjust_stock(
        &db,
        &AdjustStockInput {
            branch_id: b.id.clone(),
            product_package_id: pk.clone(),
            new_quantity: 40,
            reason: "Top up".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let in_stock = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            status: Some("in_stock".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(in_stock.total, 1);
    let low = stocks::overview(
        &db,
        &StockOverviewQuery {
            branch_id: b.id.clone(),
            status: Some("low".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(low.total, 0);
}
