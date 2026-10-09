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
        self,
        counts::{self as counts, models::*},
        models::{Adjustment, AdjustmentDirection, CreateBatch, OpeningStock, StockOperation},
    },
};
use rusqlite::Connection;
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    branch: String,
    other: String,
    package: String,
    batch: String,
}
fn setup(db: AppDb) -> F {
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let other = branches::create_branch(
        &db,
        &CreateBranch {
            code: "DEV".into(),
            name: "DEV".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap()
    .id;
    let p = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Count medicine".into()),
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
    let batch = inventory::movements::opening(
        &db,
        &OpeningStock {
            batch: CreateBatch {
                branch_id: branch.clone(),
                product_package_id: package.clone(),
                ..Default::default()
            },
            request_id: uuid(),
            quantity: 10,
            reason: "DEV fixture".into(),
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
    setup(AppDb::in_memory().unwrap())
}
fn create(f: &F) -> Detail {
    counts::create(
        &f.db,
        &Create {
            branch_id: f.branch.clone(),
            notes: Some("DEV count".into()),
            created_by: None,
        },
    )
    .unwrap()
}
fn tr(d: &Detail) -> Transition {
    Transition {
        session_id: d.session.id.clone(),
        expected_revision: d.session.revision,
    }
}
fn start(f: &F) -> Detail {
    counts::start(&f.db, &tr(&create(f))).unwrap()
}
fn entry(d: &Detail, n: usize, q: Option<i64>) -> Entry {
    Entry {
        session_id: d.session.id.clone(),
        expected_revision: d.session.revision,
        line_id: d.lines[n].id.clone(),
        line_revision: d.lines[n].revision,
        counted_quantity: q,
        notes: None,
        excluded_reason: None,
    }
}
fn enter(f: &F, d: &Detail, q: i64) -> Detail {
    counts::record(&f.db, &entry(d, 0, Some(q))).unwrap()
}
fn review(f: &F, d: &Detail) -> Detail {
    counts::transition(&f.db, &tr(d), Status::Review).unwrap()
}
fn req(d: &Detail) -> Complete {
    Complete {
        session_id: d.session.id.clone(),
        expected_revision: d.session.revision,
        request_id: uuid(),
        review_token: d.review_token.clone(),
    }
}
fn finish(f: &F, q: i64) -> Detail {
    let d = enter(f, &start(f), q);
    counts::complete(&f.db, &req(&review(f, &d))).unwrap()
}
fn stock(f: &F) -> i64 {
    inventory::balances::package(&f.db, &f.branch, &f.package)
        .unwrap()
        .quantity
}
fn total(db: &AppDb, t: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
        .unwrap()
}
fn adjust(f: &F, delta: i64) {
    inventory::movements::adjust(
        &f.db,
        &Adjustment {
            direction: if delta > 0 {
                AdjustmentDirection::In
            } else {
                AdjustmentDirection::Out
            },
            operation: StockOperation {
                request_id: uuid(),
                branch_id: f.branch.clone(),
                product_package_id: f.package.clone(),
                batch_id: f.batch.clone(),
                quantity: delta.abs(),
                reason: "Intervening movement".into(),
                created_by: None,
            },
        },
    )
    .unwrap();
}
#[test]
fn migration_006_has_single_canonical_ledger() {
    let f = f();
    assert_eq!(total(&f.db, "_schema_version"), 17);
    assert_eq!(total(&f.db, "stock_count_sessions"), 0);
    assert!(f
        .db
        .lock()
        .unwrap()
        .prepare("SELECT * FROM stock_movements")
        .is_err());
}
#[test]
fn upgrade_005_preserves_catalog_inventory_and_documents() {
    let f = setup(AppDb::in_memory_with_migrations(&MIGRATIONS[..5]).unwrap());
    let d = inventory::documents::save(
        &f.db,
        &inventory::documents::models::SaveDocument {
            document_id: None,
            expected_revision: None,
            kind: inventory::documents::models::Kind::Receipt,
            branch_id: f.branch.clone(),
            destination_branch_id: None,
            reference_text: None,
            supplier_reference: None,
            notes: Some("Preserve".into()),
            created_by: None,
            lines: vec![],
        },
    )
    .unwrap();
    let before = serde_json::to_value(&d).unwrap();
    migrations::run(&mut f.db.lock().unwrap()).unwrap();
    assert_eq!(total(&f.db, "products"), 1);
    assert_eq!(stock(&f), 10);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
    assert_eq!(
        serde_json::to_value(inventory::documents::get(&f.db, &d.header.id).unwrap()).unwrap(),
        before
    );
}
#[test]
fn migration_006_failure_rolls_back_every_partial_table() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::apply(&mut c, &MIGRATIONS[..5]).unwrap();
    c.execute_batch("CREATE TABLE stock_count_lines(marker TEXT); INSERT INTO stock_count_lines VALUES('preserved');").unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        5
    );
    assert!(c.prepare("SELECT * FROM stock_count_sessions").is_err());
    assert!(c.prepare("SELECT * FROM stock_count_sequences").is_err());
    assert_eq!(
        c.query_row("SELECT marker FROM stock_count_lines", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "preserved"
    );
}
#[test]
fn create_is_draft_and_has_no_stock_effect() {
    let f = f();
    let d = create(&f);
    assert_eq!(d.session.status, Status::Draft);
    assert_eq!(d.summary.total, 0);
    assert_eq!(stock(&f), 10);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
}
#[test]
fn readable_numbers_are_unique() {
    let f = f();
    let a = create(&f);
    let b = create(&f);
    assert!(a.session.session_number.starts_with("CNT-"));
    assert_ne!(a.session.session_number, b.session.session_number);
}
#[test]
fn start_captures_expected_ten_and_null_count() {
    let f = f();
    let d = start(&f);
    assert_eq!(d.session.status, Status::InProgress);
    assert_eq!(d.lines[0].expected_quantity, 10);
    assert_eq!(d.lines[0].counted_quantity, None);
    assert_eq!(d.lines[0].snapshot_variance, None);
    assert_eq!(d.summary.uncounted, 1);
    assert_eq!(stock(&f), 10);
}
#[test]
fn zero_stock_batches_are_in_snapshot() {
    let f = f();
    adjust(&f, -10);
    let d = start(&f);
    assert_eq!(d.summary.total, 1);
    assert_eq!(d.lines[0].expected_quantity, 0);
    assert_eq!(d.lines[0].counted_quantity, None);
}
#[test]
fn explicit_zero_is_counted() {
    let f = f();
    let d = enter(&f, &start(&f), 0);
    assert_eq!(d.summary.counted, 1);
    assert_eq!(d.summary.uncounted, 0);
    assert_eq!(d.lines[0].counted_quantity, Some(0));
    assert_eq!(stock(&f), 10);
}
#[test]
fn positive_entry_does_not_post() {
    let f = f();
    let d = enter(&f, &start(&f), 12);
    assert_eq!(d.lines[0].counted_quantity, Some(12));
    assert_eq!(d.lines[0].snapshot_variance, Some(2));
    assert_eq!(stock(&f), 10);
}
#[test]
fn negative_and_unsafe_quantities_rejected() {
    let f = f();
    let d = start(&f);
    for q in [-1, 9007199254740992] {
        assert!(counts::record(&f.db, &entry(&d, 0, Some(q))).is_err());
    }
    assert_eq!(
        counts::get(&f.db, &d.session.id, None, None).unwrap().lines[0].counted_quantity,
        None
    );
}
#[test]
fn stale_line_revision_is_rejected() {
    let f = f();
    let d = start(&f);
    let updated = enter(&f, &d, 12);
    let mut i = entry(&updated, 0, Some(8));
    i.line_revision = d.lines[0].revision;
    assert!(counts::record(&f.db, &i).is_err());
    assert_eq!(
        counts::get(&f.db, &d.session.id, None, None).unwrap().lines[0].counted_quantity,
        Some(12)
    );
}
#[test]
fn stale_session_revision_is_rejected() {
    let f = f();
    let d = start(&f);
    enter(&f, &d, 12);
    assert!(counts::record(&f.db, &entry(&d, 0, Some(8))).is_err());
}
#[test]
fn review_and_resume_preserve_snapshot() {
    let f = f();
    let d = review(&f, &enter(&f, &start(&f), 12));
    assert_eq!(d.session.status, Status::Review);
    assert!(counts::record(&f.db, &entry(&d, 0, Some(8))).is_err());
    let resumed = counts::transition(&f.db, &tr(&d), Status::InProgress).unwrap();
    assert_eq!(resumed.lines[0].expected_quantity, 10);
    assert_eq!(resumed.lines[0].counted_quantity, Some(12));
}
#[test]
fn review_can_expose_uncounted_but_completion_rejects() {
    let f = f();
    let d = review(&f, &start(&f));
    assert_eq!(d.summary.uncounted, 1);
    assert!(counts::complete(&f.db, &req(&d))
        .unwrap_err()
        .message
        .contains("Uncounted"));
    assert_eq!(stock(&f), 10);
}
#[test]
fn excluded_line_requires_recorded_reason() {
    let f = f();
    let d = start(&f);
    let mut i = entry(&d, 0, None);
    i.excluded_reason = Some(" ".into());
    assert!(counts::record(&f.db, &i).is_err());
    i.excluded_reason = Some("Sealed stock held for separate count".into());
    let d = counts::record(&f.db, &i).unwrap();
    let done = counts::complete(&f.db, &req(&review(&f, &d))).unwrap();
    assert_eq!(done.summary.excluded, 1);
    assert_eq!(done.lines[0].correction_delta, Some(0));
    assert_eq!(stock(&f), 10);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
}
#[test]
fn positive_variance_posts_plus_two() {
    let f = f();
    let d = finish(&f, 12);
    assert_eq!(stock(&f), 12);
    assert_eq!(d.lines[0].correction_delta, Some(2));
    assert_eq!(d.summary.positive, 1);
}
#[test]
fn negative_variance_posts_minus_three() {
    let f = f();
    let d = finish(&f, 7);
    assert_eq!(stock(&f), 7);
    assert_eq!(d.lines[0].correction_delta, Some(-3));
    assert_eq!(d.summary.negative, 1);
}
#[test]
fn zero_variance_creates_no_movement() {
    let f = f();
    let d = finish(&f, 10);
    assert_eq!(d.summary.matches, 1);
    assert_eq!(d.summary.corrections, 0);
    assert_eq!(d.lines[0].movement_id, None);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
}
#[test]
fn target_zero_never_goes_negative() {
    let f = f();
    let d = finish(&f, 0);
    assert_eq!(stock(&f), 0);
    assert_eq!(d.lines[0].correction_delta, Some(-10));
}
#[test]
fn completed_session_lines_and_results_are_immutable() {
    let f = f();
    let d = finish(&f, 12);
    assert!(counts::record(&f.db, &entry(&d, 0, Some(1))).is_err());
    assert!(counts::transition(&f.db, &tr(&d), Status::Cancelled).is_err());
    let c = f.db.lock().unwrap();
    for sql in [
        "UPDATE stock_count_sessions SET status='IN_PROGRESS',revision=revision+1",
        "UPDATE stock_count_lines SET expected_quantity=0",
        "UPDATE stock_count_lines SET counted_quantity=0,revision=revision+1",
        "UPDATE stock_count_results SET live_baseline=0",
        "DELETE FROM stock_count_results",
        "DELETE FROM stock_count_lines",
        "DELETE FROM stock_count_sessions",
        "DELETE FROM stock_count_completion_requests",
    ] {
        assert!(c.execute(sql, []).is_err(), "{sql}");
    }
}
fn cancellation(status: Status) {
    let f = f();
    let d = match status {
        Status::Draft => create(&f),
        Status::InProgress => start(&f),
        _ => review(&f, &start(&f)),
    };
    let cancelled = counts::transition(&f.db, &tr(&d), Status::Cancelled).unwrap();
    assert_eq!(cancelled.session.status, Status::Cancelled);
    assert_eq!(stock(&f), 10);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
    assert!(counts::complete(&f.db, &req(&cancelled)).is_err());
    assert!(counts::start(&f.db, &tr(&cancelled)).is_err());
}
#[test]
fn cancel_draft_has_no_movements() {
    cancellation(Status::Draft);
}
#[test]
fn cancel_in_progress_has_no_movements() {
    cancellation(Status::InProgress);
}
#[test]
fn cancel_review_has_no_movements() {
    cancellation(Status::Review);
}
#[test]
fn durable_completion_retry_does_not_repost() {
    let f = f();
    let d = review(&f, &enter(&f, &start(&f), 12));
    let r = req(&d);
    let done = counts::complete(&f.db, &r).unwrap();
    let again = counts::complete(&f.db, &r).unwrap();
    assert_eq!(done.lines[0].movement_id, again.lines[0].movement_id);
    assert_eq!(total(&f.db, "inventory_movements"), 2);
    assert_eq!(total(&f.db, "stock_count_completion_requests"), 1);
}
#[test]
fn changed_completion_payload_and_second_key_rejected() {
    let f = f();
    let d = review(&f, &enter(&f, &start(&f), 12));
    let r = req(&d);
    counts::complete(&f.db, &r).unwrap();
    let mut different = r.clone();
    different.review_token = "other".into();
    assert!(counts::complete(&f.db, &different).is_err());
    different = req(&d);
    assert!(counts::complete(&f.db, &different).is_err());
    assert_eq!(stock(&f), 12);
}
#[test]
fn correction_traces_to_session_and_line() {
    let f = f();
    let d = finish(&f, 12);
    let c = f.db.lock().unwrap();
    let row:(String,String,String)=c.query_row("SELECT m.movement_type,m.reference_type,l.session_id FROM inventory_movements m JOIN stock_count_lines l ON l.id=m.reference_id WHERE m.id=?1",[d.lines[0].movement_id.as_ref().unwrap()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(
        row,
        (
            "STOCK_COUNT_CORRECTION".into(),
            "stock_count_line".into(),
            d.session.id
        )
    );
}
#[test]
fn fixed_snapshot_survives_incoming_and_reload() {
    let f = f();
    let d = start(&f);
    adjust(&f, 3);
    let reloaded = counts::get(&f.db, &d.session.id, None, None).unwrap();
    assert_eq!(reloaded.lines[0].expected_quantity, 10);
    assert_eq!(reloaded.lines[0].current_live, 13);
    assert_eq!(reloaded.lines[0].intervening_movements, 1);
    assert!(f
        .db
        .lock()
        .unwrap()
        .execute(
            "UPDATE stock_count_lines SET expected_quantity=13,revision=revision+1",
            []
        )
        .is_err());
}
#[test]
fn outgoing_intervention_reconciles_live_seven_to_physical_eight() {
    let f = f();
    let d = start(&f);
    adjust(&f, -3);
    let d = review(&f, &enter(&f, &d, 8));
    assert_eq!(d.lines[0].snapshot_variance, Some(-2));
    assert_eq!(d.lines[0].current_live, 7);
    assert_eq!(d.lines[0].correction_delta, Some(1));
    let done = counts::complete(&f.db, &req(&d)).unwrap();
    assert_eq!(stock(&f), 8);
    assert_eq!(done.lines[0].expected_quantity, 10);
    assert_eq!(done.lines[0].current_live, 7);
}
#[test]
fn incoming_intervention_reconciles_live_thirteen_to_physical_eleven() {
    let f = f();
    let d = start(&f);
    adjust(&f, 3);
    let d = review(&f, &enter(&f, &d, 11));
    assert_eq!(d.lines[0].snapshot_variance, Some(1));
    assert_eq!(d.lines[0].correction_delta, Some(-2));
    counts::complete(&f.db, &req(&d)).unwrap();
    assert_eq!(stock(&f), 11);
}
#[test]
fn stock_change_after_confirmation_preview_requires_refresh() {
    let f = f();
    let d = review(&f, &enter(&f, &start(&f), 8));
    adjust(&f, -3);
    assert!(counts::complete(&f.db, &req(&d))
        .unwrap_err()
        .message
        .contains("Stock changed"));
    assert_eq!(stock(&f), 7);
    let fresh = counts::get(&f.db, &d.session.id, None, None).unwrap();
    counts::complete(&f.db, &req(&fresh)).unwrap();
    assert_eq!(stock(&f), 8);
}
#[test]
fn net_zero_intervening_movements_are_detected() {
    let f = f();
    let d = start(&f);
    adjust(&f, 1);
    adjust(&f, -1);
    let d = counts::get(&f.db, &d.session.id, None, None).unwrap();
    assert_eq!(d.lines[0].current_live, 10);
    assert_eq!(d.lines[0].intervening_movements, 2);
}
#[test]
fn snapshot_branch_isolation() {
    let f = f();
    let d = counts::create(
        &f.db,
        &Create {
            branch_id: f.other.clone(),
            notes: None,
            created_by: None,
        },
    )
    .unwrap();
    let d = counts::start(&f.db, &tr(&d)).unwrap();
    assert_eq!(d.summary.total, 0);
    assert_eq!(stock(&f), 10);
}
#[test]
fn wrong_branch_discovery_rejected() {
    let f = f();
    let d = counts::create(
        &f.db,
        &Create {
            branch_id: f.other.clone(),
            notes: None,
            created_by: None,
        },
    )
    .unwrap();
    let d = counts::start(&f.db, &tr(&d)).unwrap();
    assert!(counts::discover(
        &f.db,
        &Discover {
            session_id: d.session.id,
            expected_revision: d.session.revision,
            product_package_id: f.package.clone(),
            batch_id: Some(f.batch.clone()),
            lot_number: None,
            expiry_date: None,
            notes: "Found".into()
        }
    )
    .is_err());
}
fn discover(f: &F, d: &Detail) -> Detail {
    counts::discover(
        &f.db,
        &Discover {
            session_id: d.session.id.clone(),
            expected_revision: d.session.revision,
            product_package_id: f.package.clone(),
            batch_id: None,
            lot_number: Some("Discovered physical lot".into()),
            expiry_date: None,
            notes: "Physically found during count".into(),
        },
    )
    .unwrap()
}
#[test]
fn discovered_batch_has_zero_expected_and_no_fake_quantity() {
    let f = f();
    let d = discover(&f, &start(&f));
    let new = d.lines.iter().find(|l| l.batch_id != f.batch).unwrap();
    assert_eq!(new.expected_quantity, 0);
    assert_eq!(new.counted_quantity, None);
    assert_eq!(stock(&f), 10);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
}
#[test]
fn discovered_stock_is_reconciled_on_completion() {
    let f = f();
    let mut d = discover(&f, &start(&f));
    for n in 0..d.lines.len() {
        let q = if d.lines[n].batch_id == f.batch {
            10
        } else {
            2
        };
        d = counts::record(&f.db, &entry(&d, n, Some(q))).unwrap();
    }
    let d = counts::complete(&f.db, &req(&review(&f, &d))).unwrap();
    assert_eq!(stock(&f), 12);
    assert_eq!(d.summary.corrections, 1);
}
#[test]
fn whole_completion_rolls_back_when_second_correction_fails() {
    let f = f();
    let mut d = discover(&f, &start(&f));
    for n in 0..d.lines.len() {
        let q = if d.lines[n].batch_id == f.batch { 9 } else { 2 };
        d = counts::record(&f.db, &entry(&d, n, Some(q))).unwrap();
    }
    let d = review(&f, &d);
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_second_count BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='STOCK_COUNT_CORRECTION' AND NEW.quantity_delta>0 BEGIN SELECT RAISE(ABORT,'isolated failure'); END;").unwrap();
    assert!(counts::complete(&f.db, &req(&d)).is_err());
    assert_eq!(stock(&f), 10);
    assert_eq!(total(&f.db, "inventory_movements"), 1);
    assert_eq!(total(&f.db, "stock_count_results"), 0);
    assert_eq!(total(&f.db, "stock_count_completion_requests"), 0);
    assert_eq!(
        counts::get(&f.db, &d.session.id, None, None)
            .unwrap()
            .session
            .status,
        Status::Review
    );
}
#[test]
fn sessions_and_lines_use_bounded_pagination() {
    let f = f();
    for _ in 0..3 {
        create(&f);
    }
    let p = counts::list(
        &f.db,
        &Query {
            branch_id: f.branch.clone(),
            status: Some(Status::Draft),
            date_from: None,
            date_to: None,
            limit: Some(1),
            offset: Some(1),
        },
    )
    .unwrap();
    assert_eq!(p.total, 3);
    assert_eq!(p.items.len(), 1);
    let d = discover(&f, &start(&f));
    let page = counts::get(&f.db, &d.session.id, Some(1), Some(1)).unwrap();
    assert_eq!(page.lines.len(), 1);
    assert_eq!(page.summary.total, 2);
    assert!(counts::get(&f.db, &d.session.id, Some(201), None).is_err());
}
#[test]
fn restart_preserves_results_and_completion_keys() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("count.db");
    let f = setup(AppDb::open(&path).unwrap());
    let d = review(&f, &enter(&f, &start(&f), 12));
    let r = req(&d);
    let done = counts::complete(&f.db, &r).unwrap();
    drop(f);
    let db = AppDb::open(&path).unwrap();
    let again = counts::complete(&db, &r).unwrap();
    assert_eq!(again.session.status, Status::Completed);
    assert_eq!(again.lines[0].movement_id, done.lines[0].movement_id);
    assert_eq!(again.lines[0].current_live, 10);
    assert_eq!(total(&db, "inventory_movements"), 2);
}
#[test]
fn concurrent_workstations_cannot_silently_overwrite_entry() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("race.db");
    let f = setup(AppDb::open(&path).unwrap());
    let d = start(&f);
    let a = entry(&d, 0, Some(8));
    let b = entry(&d, 0, Some(12));
    let one = AppDb::open(&path).unwrap();
    let two = AppDb::open(&path).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let first = barrier.clone();
    let t1 = std::thread::spawn(move || {
        first.wait();
        counts::record(&one, &a).is_ok()
    });
    let t2 = std::thread::spawn(move || {
        barrier.wait();
        counts::record(&two, &b).is_ok()
    });
    assert_ne!(t1.join().unwrap(), t2.join().unwrap());
    assert_eq!(stock(&f), 10);
}
#[test]
fn completed_audit_baseline_does_not_follow_later_stock() {
    let f = f();
    let d = finish(&f, 12);
    adjust(&f, 2);
    let after = counts::get(&f.db, &d.session.id, None, None).unwrap();
    assert_eq!(after.lines[0].current_live, 10);
    assert_eq!(after.lines[0].correction_delta, Some(2));
    assert_eq!(stock(&f), 14);
}
#[test]
fn incoming_thirteen_reconciles_to_nine_with_minus_four() {
    let f = f();
    let d = start(&f);
    adjust(&f, 3);
    let d = review(&f, &enter(&f, &d, 9));
    assert_eq!(d.lines[0].expected_quantity, 10);
    assert_eq!(d.lines[0].current_live, 13);
    assert_eq!(d.lines[0].snapshot_variance, Some(-1));
    assert_eq!(d.lines[0].correction_delta, Some(-4));
    counts::complete(&f.db, &req(&d)).unwrap();
    assert_eq!(stock(&f), 9);
}
#[test]
fn cancelled_lines_and_session_are_immutable() {
    let f = f();
    let d = start(&f);
    let d = counts::transition(&f.db, &tr(&d), Status::Cancelled).unwrap();
    assert!(counts::record(&f.db, &entry(&d, 0, Some(5))).is_err());
    assert!(counts::transition(&f.db, &tr(&d), Status::InProgress).is_err());
    let c = f.db.lock().unwrap();
    assert!(c.execute("UPDATE stock_count_sessions SET status='IN_PROGRESS',revision=revision+1,cancelled_at=NULL",[]).is_err());
    assert!(c
        .execute(
            "UPDATE stock_count_lines SET counted_quantity=0,revision=revision+1",
            []
        )
        .is_err());
}
#[test]
fn completed_cannot_return_to_entry_or_review() {
    let f = f();
    let d = finish(&f, 12);
    assert!(counts::transition(&f.db, &tr(&d), Status::InProgress).is_err());
    assert!(counts::transition(&f.db, &tr(&d), Status::Review).is_err());
    assert_eq!(stock(&f), 12);
}
