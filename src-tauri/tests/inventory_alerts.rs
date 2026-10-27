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
        alerts::{self, models::*},
        balances, batches,
        counts::{self, models as cm},
        documents::{self as docs, models as dm},
        expiry, levels,
        models::*,
        movements,
    },
};
use rusqlite::{params, Connection};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    branch: String,
    other: String,
    package: String,
}
fn f() -> F {
    with_db(AppDb::in_memory().unwrap())
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
    let p = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Alert medicine".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let package = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: p.id,
            package_label: "Box".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    F {
        db,
        branch,
        other,
        package,
    }
}
#[test]
fn upgrade_preserves_seeded_catalog_ledger_documents_and_count() {
    let f = with_db(AppDb::in_memory_with_migrations(&MIGRATIONS[..7]).unwrap());
    stock(&f, None, 2);
    receipt(&f, 1);
    counts::create(
        &f.db,
        &cm::Create {
            branch_id: f.branch.clone(),
            notes: Some("Preserve draft".into()),
            created_by: None,
        },
    )
    .unwrap();
    let mut c = f.db.lock().unwrap();
    let snapshot = |c: &Connection| {
        let mut result = Vec::new();
        for table in [
            "products",
            "product_packages",
            "inventory_batches",
            "inventory_movements",
            "inventory_documents",
            "inventory_document_lines",
            "inventory_document_movements",
            "stock_count_sessions",
            "_schema_version",
        ] {
            let mut q = c
                .prepare(&format!("SELECT * FROM {table} ORDER BY 1"))
                .unwrap();
            let n = q.column_count();
            let rows = q
                .query_map([], |r| {
                    Ok((0..n)
                        .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                        .collect::<Vec<_>>())
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.push(rows);
        }
        result
    };
    let before = snapshot(&c);
    migrations::apply(&mut c, &MIGRATIONS[..8]).unwrap();
    let mut after = snapshot(&c);
    after.last_mut().unwrap().pop();
    assert_eq!(before, after);
    assert_eq!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(!c
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}
#[test]
fn invalid_ranges_and_pagination_fail_closed() {
    let f = f();
    for (from, to) in [("2026-02-30", "2026-03-01"), ("2026-03-01", "2026-02-01")] {
        assert!(alerts::list(
            &f.db,
            &Query {
                branch_id: f.branch.clone(),
                from_date: Some(from.into()),
                to_date: Some(to.into()),
                ..Default::default()
            }
        )
        .is_err());
    }
    assert!(alerts::list(
        &f.db,
        &Query {
            branch_id: f.branch.clone(),
            limit: Some(201),
            ..Default::default()
        }
    )
    .is_err());
    assert!(alerts::monitor(&f.db, &monitor(&f, 31, 30), false).is_err());
}
fn batch(f: &F, date: Option<&str>) -> CreateBatch {
    CreateBatch {
        branch_id: f.branch.clone(),
        product_package_id: f.package.clone(),
        expiry_date: date.map(str::to_owned),
        lot_number: Some(id()),
        ..Default::default()
    }
}
fn stock(f: &F, date: Option<&str>, quantity: i64) -> String {
    movements::opening(
        &f.db,
        &OpeningStock {
            batch: batch(f, date),
            request_id: id(),
            quantity,
            reason: "DEV alert test".into(),
            created_by: None,
        },
    )
    .unwrap()
    .batch_id
}
fn op(f: &F, b: &str, q: i64) -> StockOperation {
    StockOperation {
        request_id: id(),
        branch_id: f.branch.clone(),
        product_package_id: f.package.clone(),
        batch_id: b.into(),
        quantity: q,
        reason: "DEV alert test".into(),
        created_by: None,
    }
}
fn level(f: &F, q: i64) {
    levels::set(
        &f.db,
        &SetLevel {
            branch_id: f.branch.clone(),
            product_package_id: f.package.clone(),
            reorder_level: q,
            minimum_stock: Some(0),
            maximum_stock: Some(100),
        },
    )
    .unwrap();
}
fn eval(f: &F) {
    alerts::evaluate(&f.db, &f.branch, Some("2026-01-01")).unwrap();
}
fn rows(f: &F) -> Vec<Alert> {
    alerts::list(
        &f.db,
        &Query {
            branch_id: f.branch.clone(),
            ..Default::default()
        },
    )
    .unwrap()
    .items
}
fn active(f: &F) -> Vec<Alert> {
    rows(f)
        .into_iter()
        .filter(|a| a.status == "ACTIVE")
        .collect()
}
fn count(db: &AppDb, t: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
        .unwrap()
}
fn monitor(f: &F, min: i64, max: i64) -> MonitorQuery {
    MonitorQuery {
        branch_id: f.branch.clone(),
        business_date: Some("2026-01-01".into()),
        minimum_days: Some(min),
        maximum_days: Some(max),
        expired: false,
        out_of_stock: false,
        limit: None,
        offset: None,
    }
}
#[test]
fn migration_eight_creates_only_advisory_schema() {
    let f = f();
    assert_eq!(count(&f.db, "_schema_version"), 22);
    assert_eq!(count(&f.db, "inventory_alerts"), 0);
    assert_eq!(count(&f.db, "inventory_movements"), 0);
    assert_eq!(MIGRATIONS[7].name, "inventory_alerts");
}
#[test]
fn seven_to_eight_preserves_prior_metadata() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::apply(&mut c, &MIGRATIONS[..7]).unwrap();
    let before: String = c
        .query_row(
            "SELECT group_concat(version||name||applied_at) FROM _schema_version",
            [],
            |r| r.get(0),
        )
        .unwrap();
    migrations::run(&mut c).unwrap();
    migrations::run(&mut c).unwrap();
    let after: String = c
        .query_row(
            "SELECT group_concat(version||name||applied_at) FROM _schema_version WHERE version<8",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM _schema_version WHERE version=8",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}
#[test]
fn migration_eight_rolls_back_partial_schema() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::apply(&mut c, &MIGRATIONS[..7]).unwrap();
    c.execute_batch(
        "CREATE TABLE inventory_alerts(marker TEXT); INSERT INTO inventory_alerts VALUES('keep');",
    )
    .unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='inventory_alert_settings'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        7
    );
    assert_eq!(
        c.query_row("SELECT marker FROM inventory_alerts", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
}
#[test]
fn catalog_unchanged_by_evaluation() {
    let f = f();
    stock(&f, Some("2025-01-01"), 2);
    let c = f.db.lock().unwrap();
    let before: String = c
        .query_row(
            "SELECT group_concat(id||updated_at) FROM products",
            [],
            |r| r.get(0),
        )
        .unwrap();
    drop(c);
    eval(&f);
    assert_eq!(count(&f.db, "products"), 1);
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row(
                "SELECT group_concat(id||updated_at) FROM products",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        before
    );
}
#[test]
fn ledger_and_workflow_tables_unchanged_by_evaluation() {
    let f = f();
    stock(&f, Some("2025-01-01"), 2);
    let tables = [
        "inventory_movements",
        "inventory_batches",
        "inventory_transfers",
        "inventory_documents",
        "stock_count_sessions",
    ];
    let before = tables.map(|t| count(&f.db, t));
    eval(&f);
    assert_eq!(before, tables.map(|t| count(&f.db, t)));
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        2
    );
}
#[test]
fn settings_defaults_and_update_persist() {
    let f = f();
    let mut s = alerts::settings(&f.db, &f.branch).unwrap();
    assert_eq!((s.expiry_warning_days, s.critical_expiry_days), (90, 30));
    s.expiry_warning_days = 60;
    s.critical_expiry_days = 15;
    s.low_stock_enabled = false;
    alerts::update_settings(&f.db, &s).unwrap();
    let got = alerts::settings(&f.db, &f.branch).unwrap();
    assert_eq!(got.expiry_warning_days, 60);
    assert!(!got.low_stock_enabled);
    assert_eq!(count(&f.db, "inventory_levels"), 0);
}
#[test]
fn invalid_settings_rejected_atomically() {
    let f = f();
    let mut s = alerts::settings(&f.db, &f.branch).unwrap();
    s.critical_expiry_days = 91;
    assert!(alerts::update_settings(&f.db, &s).is_err());
    assert_eq!(count(&f.db, "inventory_alert_settings"), 0);
}
#[test]
fn expired_positive_stock_detected() {
    let f = f();
    stock(&f, Some("2025-12-31"), 1);
    eval(&f);
    assert_eq!(active(&f)[0].alert_type, "BATCH_EXPIRED");
}
#[test]
fn zero_stock_expired_excluded() {
    let f = f();
    batches::create(&f.db, &batch(&f, Some("2025-12-31"))).unwrap();
    eval(&f);
    assert!(rows(&f).is_empty());
}
#[test]
fn unknown_expiry_excluded() {
    let f = f();
    stock(&f, None, 1);
    eval(&f);
    assert!(rows(&f).is_empty());
}
#[test]
fn inactive_batch_excluded() {
    let f = f();
    let b = stock(&f, Some("2025-12-31"), 1);
    f.db.lock()
        .unwrap()
        .execute("UPDATE inventory_batches SET is_active=0 WHERE id=?1", [b])
        .unwrap();
    eval(&f);
    assert!(rows(&f).is_empty());
}
#[test]
fn expiry_today_is_expiring_not_expired() {
    let f = f();
    stock(&f, Some("2026-01-01"), 1);
    eval(&f);
    assert_eq!(active(&f)[0].alert_type, "BATCH_EXPIRING");
    assert_eq!(active(&f)[0].details["daysRemaining"], 0);
}
#[test]
fn thirty_day_boundary_inclusive() {
    let f = f();
    stock(&f, Some("2026-01-31"), 1);
    stock(&f, Some("2026-02-01"), 1);
    let p = alerts::monitor(&f.db, &monitor(&f, 0, 30), false).unwrap();
    assert_eq!(p.total, 1);
    assert_eq!(p.items[0].days_remaining, Some(30));
}
#[test]
fn sixty_day_bucket_boundaries() {
    let f = f();
    for d in ["2026-01-31", "2026-02-01", "2026-03-02", "2026-03-03"] {
        stock(&f, Some(d), 1);
    }
    let p = alerts::monitor(&f.db, &monitor(&f, 31, 60), false).unwrap();
    assert_eq!(p.total, 2);
    assert_eq!(p.items[0].days_remaining, Some(31));
    assert_eq!(p.items[1].days_remaining, Some(60));
}
#[test]
fn ninety_day_boundary_and_beyond() {
    let f = f();
    stock(&f, Some("2026-04-01"), 1);
    stock(&f, Some("2026-04-02"), 1);
    eval(&f);
    assert_eq!(active(&f).len(), 1);
    assert_eq!(
        alerts::monitor(&f.db, &monitor(&f, 61, 90), false)
            .unwrap()
            .total,
        1
    );
}
#[test]
fn impossible_calendar_dates_rejected() {
    let f = f();
    assert!(alerts::evaluate(&f.db, &f.branch, Some("2026-02-30")).is_err());
    let mut q = monitor(&f, 0, 30);
    q.business_date = Some("2026-02-30".into());
    assert!(alerts::monitor(&f.db, &q, false).is_err());
    assert!(batches::create(&f.db, &batch(&f, Some("2026-02-30"))).is_err());
}
#[test]
fn fefo_excludes_expired_empty_and_other_branch() {
    let f = f();
    stock(&f, Some("2025-12-31"), 2);
    let good = stock(&f, Some("2026-02-01"), 1);
    batches::create(&f.db, &batch(&f, Some("2026-01-01"))).unwrap();
    let r = expiry::fefo(&f.db, &f.branch, &f.package, "2026-01-01", None, None).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].batch.id, good);
    assert!(
        expiry::fefo(&f.db, &f.other, &f.package, "2026-01-01", None, None)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn fefo_unknown_expiry_last() {
    let f = f();
    let unknown = stock(&f, None, 1);
    let known = stock(&f, Some("2026-02-01"), 1);
    let r = expiry::fefo(&f.db, &f.branch, &f.package, "2026-01-01", None, None).unwrap();
    assert_eq!(r[0].batch.id, known);
    assert_eq!(r[1].batch.id, unknown);
}
#[test]
fn configured_low_stock() {
    let f = f();
    stock(&f, None, 2);
    level(&f, 3);
    eval(&f);
    assert_eq!(active(&f)[0].alert_type, "LOW_STOCK");
}
#[test]
fn equality_at_reorder_is_low() {
    let f = f();
    stock(&f, None, 3);
    level(&f, 3);
    eval(&f);
    assert_eq!(active(&f)[0].alert_type, "LOW_STOCK");
}
#[test]
fn no_level_invents_no_threshold() {
    let f = f();
    stock(&f, None, 1);
    eval(&f);
    assert!(rows(&f).is_empty());
    assert_eq!(
        alerts::monitor(&f.db, &monitor(&f, 0, 90), true)
            .unwrap()
            .total,
        0
    );
}
#[test]
fn zero_stock_has_only_out_of_stock() {
    let f = f();
    level(&f, 3);
    eval(&f);
    let r = active(&f);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].alert_type, "OUT_OF_STOCK");
    assert_eq!(r[0].severity, "CRITICAL");
}
#[test]
fn creation_has_durable_fingerprint_and_event() {
    let f = f();
    level(&f, 0);
    eval(&f);
    assert!(active(&f)[0].fingerprint.starts_with(&f.branch));
    assert_eq!(count(&f.db, "inventory_alert_history"), 1);
}
#[test]
fn ten_evaluations_deduplicate() {
    let f = f();
    stock(&f, Some("2025-12-31"), 1);
    for _ in 0..10 {
        eval(&f);
    }
    assert_eq!(rows(&f).len(), 1);
    assert_eq!(count(&f.db, "inventory_alert_history"), 1);
}
#[test]
fn last_detected_advances_first_detected_preserved() {
    let f = f();
    level(&f, 0);
    eval(&f);
    let before = rows(&f).remove(0);
    eval(&f);
    let after = rows(&f).remove(0);
    assert!(after.last_detected_at > before.last_detected_at);
    assert_eq!(before.first_detected_at, after.first_detected_at);
}
#[test]
fn stock_increase_resolves_out_and_low() {
    let f = f();
    level(&f, 2);
    eval(&f);
    stock(&f, None, 3);
    eval(&f);
    assert!(active(&f).is_empty());
    assert_eq!(rows(&f)[0].status, "RESOLVED");
    assert!(rows(&f)[0].resolved_at.is_some());
}
#[test]
fn stock_zero_resolves_expiry() {
    let f = f();
    let b = stock(&f, Some("2025-12-31"), 1);
    eval(&f);
    movements::damage(&f.db, &op(&f, &b, 1)).unwrap();
    eval(&f);
    assert!(active(&f).is_empty());
    assert_eq!(rows(&f)[0].status, "RESOLVED");
}
#[test]
fn branches_isolate_alerts_and_settings() {
    let f = f();
    level(&f, 0);
    eval(&f);
    assert!(alerts::list(
        &f.db,
        &Query {
            branch_id: f.other.clone(),
            ..Default::default()
        }
    )
    .unwrap()
    .items
    .is_empty());
    assert!(alerts::get(&f.db, &f.other, &rows(&f)[0].id).is_err());
    assert!(alerts::dismiss(&f.db, &f.other, &rows(&f)[0].id).is_err());
}
#[test]
fn fingerprint_unique_constraint() {
    let f = f();
    level(&f, 0);
    eval(&f);
    assert!(f.db.lock().unwrap().execute("INSERT INTO inventory_alerts SELECT ?1,branch_id,alert_type,product_package_id,batch_id,severity,status,fingerprint,details_json,first_detected_at,last_detected_at,resolved_at,created_at,updated_at FROM inventory_alerts",[id()]).is_err());
}
#[test]
fn deterministic_severity() {
    let f = f();
    stock(&f, Some("2025-12-31"), 1);
    stock(&f, Some("2026-01-31"), 1);
    stock(&f, Some("2026-02-01"), 1);
    eval(&f);
    let r = rows(&f);
    for (days, severity) in [(-1, "CRITICAL"), (30, "HIGH"), (31, "MEDIUM")] {
        assert_eq!(
            r.iter()
                .find(|r| r.details["daysRemaining"] == days)
                .unwrap()
                .severity,
            severity
        );
    }
}
#[test]
fn db_pagination_and_typed_filters() {
    let f = f();
    for _ in 0..3 {
        stock(&f, Some("2025-12-31"), 1);
    }
    eval(&f);
    let p = alerts::list(
        &f.db,
        &Query {
            branch_id: f.branch.clone(),
            status: Some(Status::Active),
            alert_type: Some(AlertType::BatchExpired),
            limit: Some(1),
            offset: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.total, 3);
    assert_eq!(p.items.len(), 1);
    assert!(serde_json::from_str::<Query>(r#"{"branchId":"a","alertType":"BOGUS"}"#).is_err());
}
#[test]
fn history_cannot_be_deleted_or_updated() {
    let f = f();
    level(&f, 0);
    eval(&f);
    let c = f.db.lock().unwrap();
    assert!(c.execute("DELETE FROM inventory_alerts", []).is_err());
    assert!(c
        .execute("DELETE FROM inventory_alert_history", [])
        .is_err());
    assert!(c
        .execute("UPDATE inventory_alert_history SET status='DISMISSED'", [])
        .is_err());
}
#[test]
fn dismissal_survives_refresh_and_reactivates_after_clear() {
    let f = f();
    level(&f, 2);
    eval(&f);
    let a = rows(&f)[0].id.clone();
    alerts::dismiss(&f.db, &f.branch, &a).unwrap();
    eval(&f);
    assert_eq!(rows(&f)[0].status, "DISMISSED");
    let b = stock(&f, None, 3);
    eval(&f);
    movements::damage(&f.db, &op(&f, &b, 3)).unwrap();
    eval(&f);
    assert_eq!(rows(&f)[0].status, "ACTIVE");
    assert_eq!(rows(&f)[0].id, a);
    assert_eq!(count(&f.db, "inventory_alert_history"), 4);
}
#[test]
fn disabling_expiry_resolves_without_stock_change() {
    let f = f();
    stock(&f, Some("2025-12-31"), 1);
    eval(&f);
    let mut s = alerts::settings(&f.db, &f.branch).unwrap();
    s.expiry_enabled = false;
    alerts::update_settings(&f.db, &s).unwrap();
    eval(&f);
    assert!(active(&f).is_empty());
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        1
    );
}
fn receipt(f: &F, q: i64) {
    let i = dm::SaveDocument {
        document_id: None,
        expected_revision: None,
        kind: dm::Kind::Receipt,
        branch_id: f.branch.clone(),
        destination_branch_id: None,
        reference_text: Some("DEV alert receipt".into()),
        supplier_reference: None,
        notes: None,
        created_by: None,
        lines: vec![dm::LineInput {
            product_package_id: f.package.clone(),
            batch_id: None,
            action: dm::Action::Receive,
            quantity: q,
            reason: "DEV receipt".into(),
            lot_number: Some(id()),
            expiry_date: None,
            cost_price_minor: None,
            supplier_reference: None,
        }],
    };
    let d = docs::save(&f.db, &i).unwrap();
    let d = docs::ready(
        &f.db,
        &dm::Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision,
        },
    )
    .unwrap();
    docs::post(
        &f.db,
        &dm::PostDocument {
            document_id: d.header.id,
            expected_revision: d.header.revision,
            request_id: id(),
        },
    )
    .unwrap();
    alerts::refresh_all(&f.db);
}
#[test]
fn phase_b_receipt_refresh_resolves_low_stock() {
    let f = f();
    stock(&f, None, 1);
    level(&f, 2);
    eval(&f);
    receipt(&f, 2);
    assert!(active(&f).is_empty());
    assert_eq!(count(&f.db, "inventory_documents"), 1);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        3
    );
}
#[test]
fn phase_c_correction_refresh_resolves_low_stock() {
    let f = f();
    stock(&f, None, 1);
    level(&f, 2);
    eval(&f);
    let d = counts::create(
        &f.db,
        &cm::Create {
            branch_id: f.branch.clone(),
            notes: Some("DEV alerts".into()),
            created_by: None,
        },
    )
    .unwrap();
    let tr = |d: &cm::Detail| cm::Transition {
        session_id: d.session.id.clone(),
        expected_revision: d.session.revision,
    };
    let d = counts::start(&f.db, &tr(&d)).unwrap();
    let d = counts::record(
        &f.db,
        &cm::Entry {
            session_id: d.session.id.clone(),
            expected_revision: d.session.revision,
            line_id: d.lines[0].id.clone(),
            line_revision: d.lines[0].revision,
            counted_quantity: Some(3),
            notes: None,
            excluded_reason: None,
        },
    )
    .unwrap();
    let d = counts::transition(&f.db, &tr(&d), cm::Status::Review).unwrap();
    counts::complete(
        &f.db,
        &cm::Complete {
            session_id: d.session.id,
            expected_revision: d.session.revision,
            request_id: id(),
            review_token: d.review_token,
        },
    )
    .unwrap();
    alerts::refresh_all(&f.db);
    assert!(active(&f).is_empty());
    assert_eq!(count(&f.db, "stock_count_sessions"), 1);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        3
    );
}
#[test]
fn concurrent_evaluation_is_serialized() {
    let f = f();
    level(&f, 0);
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("concurrent.db");
    f.db.lock()
        .unwrap()
        .backup(rusqlite::MAIN_DB, &p, None)
        .unwrap();
    let a = AppDb::open(&p).unwrap();
    let b = AppDb::open(&p).unwrap();
    let x = f.branch.clone();
    let y = x.clone();
    let one = std::thread::spawn(move || alerts::evaluate(&a, &x, Some("2026-01-01")));
    let two = std::thread::spawn(move || alerts::evaluate(&b, &y, Some("2026-01-01")));
    one.join().unwrap().unwrap();
    two.join().unwrap().unwrap();
    let db = AppDb::open(&p).unwrap();
    assert_eq!(count(&db, "inventory_alerts"), 1);
    assert_eq!(count(&db, "inventory_alert_history"), 1);
}
#[test]
fn restart_keeps_settings_lifecycle_and_dedup() {
    let f = f();
    level(&f, 0);
    eval(&f);
    let a = rows(&f)[0].id.clone();
    alerts::dismiss(&f.db, &f.branch, &a).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("restart.db");
    f.db.lock()
        .unwrap()
        .backup(rusqlite::MAIN_DB, &p, None)
        .unwrap();
    let db = AppDb::open(&p).unwrap();
    alerts::evaluate(&db, &f.branch, Some("2026-01-01")).unwrap();
    assert_eq!(count(&db, "inventory_alerts"), 1);
    assert_eq!(
        alerts::get(&db, &f.branch, &a).unwrap()["alert"]["status"],
        "DISMISSED"
    );
    assert_eq!(
        alerts::settings(&db, &f.branch)
            .unwrap()
            .expiry_warning_days,
        90
    );
}
#[test]
fn failed_evaluation_rolls_back_all_alert_changes() {
    let f = f();
    level(&f, 0);
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_history BEFORE INSERT ON inventory_alert_history BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(alerts::evaluate(&f.db, &f.branch, None).is_err());
    assert_eq!(count(&f.db, "inventory_alerts"), 0);
    assert_eq!(count(&f.db, "inventory_alert_settings"), 0);
}
#[test]
fn advisory_refresh_failure_does_not_undo_receipt() {
    let f = f();
    level(&f, 10);
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_history BEFORE INSERT ON inventory_alert_history BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    receipt(&f, 2);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        2
    );
    assert_eq!(count(&f.db, "inventory_documents"), 1);
}
#[test]
fn no_negative_stock_still_enforced_with_alerts() {
    let f = f();
    let b = stock(&f, None, 1);
    level(&f, 1);
    eval(&f);
    assert!(movements::damage(&f.db, &op(&f, &b, 2)).is_err());
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        1
    );
    assert!(f
        .db
        .lock()
        .unwrap()
        .execute(
            "UPDATE inventory_movements SET quantity_delta=99 WHERE batch_id=?1",
            params![b]
        )
        .is_err());
}
