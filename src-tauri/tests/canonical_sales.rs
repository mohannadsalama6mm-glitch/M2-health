use m2_health_lib::{
    db::{
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        models::catalog::{AddBarcode, CreateProduct, CreateProductPackage, SetPackagePrice},
        repositories::{barcodes, branches, price_history, product_packages, products},
    },
    inventory::{
        models::{CreateBatch, OpeningStock},
        movements,
    },
    sales::{self, models::*},
};
use serde_json::Value;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    b: String,
    p: String,
    product: String,
}
fn fixture(db: AppDb) -> F {
    let b = branches::ensure_default_branch(&db).unwrap().id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Test medicine".into()),
            commercial_name_ar: Some("دواء اختبار".into()),
            scientific_name: Some("Test ingredient".into()),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let p = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: product.clone(),
            package_label: "Box".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    price_history::set(
        &db,
        &SetPackagePrice {
            product_package_id: p.clone(),
            selling_price_minor: 101,
            cost_price_minor: None,
            reason: None,
        },
    )
    .unwrap();
    F { db, b, p, product }
}
fn f() -> F {
    fixture(AppDb::in_memory().unwrap())
}
fn seed(f: &F, q: i64, expiry: Option<&str>, cost: Option<i64>) -> String {
    movements::opening(
        &f.db,
        &OpeningStock {
            request_id: id(),
            batch: CreateBatch {
                branch_id: f.b.clone(),
                product_package_id: f.p.clone(),
                expiry_date: expiry.map(str::to_string),
                cost_price_minor: cost,
                ..Default::default()
            },
            quantity: q,
            reason: "Isolated Phase4 fixture".into(),
            created_by: None,
        },
    )
    .unwrap()
    .batch_id
}
fn cart(f: &F, q: i64) -> Cart {
    Cart {
        branch_id: f.b.clone(),
        lines: vec![Line {
            product_package_id: f.p.clone(),
            quantity: q,
            ..Default::default()
        }],
        ..Default::default()
    }
}
fn save(f: &F, c: Cart) -> Value {
    sales::save(
        &f.db,
        &Save {
            id: id(),
            expected_revision: None,
            cart: c,
        },
    )
    .unwrap()
}
fn post_input(s: &Value) -> Post {
    Post {
        id: s["id"].as_str().unwrap().into(),
        expected_revision: s["revision"].as_i64().unwrap(),
        request_id: id(),
        payments: vec![Payment {
            payment_method: Method::Cash,
            amount_minor: s["totalMinor"].as_i64().unwrap(),
            reference: None,
        }],
    }
}
fn posted(f: &F, q: i64) -> Value {
    let s = save(f, cart(f, q));
    sales::post(&f.db, &post_input(&s)).unwrap()
}
fn balance(f: &F) -> i64 {
    f.db.lock()
        .unwrap()
        .query_row(
            "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE branch_id=?1",
            [&f.b],
            |r| r.get(0),
        )
        .unwrap()
}
fn count(f: &F, table: &str) -> i64 {
    f.db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn ret(s: &Value, q: i64) -> ReturnDraft {
    ReturnDraft {
        id: id(),
        sale_id: s["id"].as_str().unwrap().into(),
        reason: "Controlled return".into(),
        lines: vec![ReturnLine {
            sale_item_id: s["items"][0]["id"].as_str().unwrap().into(),
            quantity: q,
        }],
    }
}
fn returned(f: &F, s: &Value, q: i64) -> Value {
    let r = ret(s, q);
    sales::create_return(&f.db, &r).unwrap();
    sales::post_return(
        &f.db,
        &ReturnPost {
            id: r.id,
            request_id: id(),
        },
    )
    .unwrap()
}
fn query(f: &F, search: &str) -> Query {
    Query {
        branch_id: f.b.clone(),
        search: Some(search.into()),
        ..Default::default()
    }
}
#[test]
fn migration_009_creates_domain_without_second_ledger() {
    let f = f();
    assert_eq!(count(&f, "sales"), 0);
    assert_eq!(f.db.lock().unwrap().query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('stock_balances','stock_movements')",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}
#[test]
fn upgrade_008_to_009_applies_once() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..8]).unwrap();
    let mut c = db.lock().unwrap();
    migrations::run(&mut c).unwrap();
    migrations::run(&mut c).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM _schema_version WHERE version=9",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}
#[test]
fn migration_009_rolls_back_every_partial_table() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..8]).unwrap();
    let mut c = db.lock().unwrap();
    c.execute_batch("CREATE TABLE sale_payments(conflict TEXT)")
        .unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='sales'",
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
        8
    );
}
#[test]
fn upgrade_preserves_catalog() {
    let f = fixture(AppDb::in_memory_with_migrations(&MIGRATIONS[..8]).unwrap());
    migrations::run(&mut f.db.lock().unwrap()).unwrap();
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row(
                "SELECT commercial_name_en FROM products WHERE id=?1",
                [&f.product],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Test medicine"
    );
}
#[test]
fn upgrade_preserves_inventory() {
    let f = fixture(AppDb::in_memory_with_migrations(&MIGRATIONS[..8]).unwrap());
    seed(&f, 7, None, None);
    migrations::run(&mut f.db.lock().unwrap()).unwrap();
    assert_eq!(balance(&f), 7);
    assert_eq!(count(&f, "inventory_movements"), 1);
}
#[test]
fn create_draft_snapshots_price() {
    let f = f();
    let s = save(&f, cart(&f, 2));
    assert_eq!(s["status"], "DRAFT");
    assert_eq!(s["cart"]["lines"][0]["unitPriceMinor"], 101);
}
#[test]
fn draft_has_zero_stock_effect() {
    let f = f();
    seed(&f, 2, None, None);
    save(&f, cart(&f, 20));
    assert_eq!(balance(&f), 2);
    assert_eq!(count(&f, "sale_payments"), 0);
}
#[test]
fn hold_persists_without_payment_or_stock() {
    let f = f();
    let s = save(&f, cart(&f, 2));
    let h = sales::transition(
        &f.db,
        &Transition {
            id: s["id"].as_str().unwrap().into(),
            expected_revision: 1,
        },
        true,
    )
    .unwrap();
    assert_eq!(h["status"], "HELD");
    assert_eq!(count(&f, "inventory_movements"), 0);
    assert_eq!(count(&f, "sale_payments"), 0);
}
#[test]
fn resume_restores_exact_cart() {
    let f = f();
    let s = save(&f, cart(&f, 2));
    let key = s["id"].as_str().unwrap();
    sales::transition(
        &f.db,
        &Transition {
            id: key.into(),
            expected_revision: 1,
        },
        true,
    )
    .unwrap();
    let r = sales::transition(
        &f.db,
        &Transition {
            id: key.into(),
            expected_revision: 2,
        },
        false,
    )
    .unwrap();
    assert_eq!(r["cart"], s["cart"]);
    assert_eq!(r["status"], "DRAFT");
}
#[test]
fn barcode_resolves_package() {
    let f = f();
    barcodes::add(
        &f.db,
        &AddBarcode {
            product_package_id: f.p.clone(),
            barcode: "123456789".into(),
            is_primary: true,
        },
    )
    .unwrap();
    let r = sales::search(&f.db, &query(&f, "123456789")).unwrap();
    assert_eq!(r["items"][0]["productPackageId"], f.p);
}
#[test]
fn english_search() {
    let f = f();
    assert_eq!(
        sales::search(&f.db, &query(&f, "MEDICINE")).unwrap()["total"],
        1
    );
}
#[test]
fn arabic_search() {
    let f = f();
    assert_eq!(
        sales::search(&f.db, &query(&f, "دواء")).unwrap()["total"],
        1
    );
}
#[test]
fn scientific_search() {
    let f = f();
    assert_eq!(
        sales::search(&f.db, &query(&f, "ingredient")).unwrap()["total"],
        1
    );
}
#[test]
fn availability_excludes_expired() {
    let f = f();
    seed(&f, 7, Some("2000-01-01"), None);
    seed(&f, 2, None, None);
    assert_eq!(
        sales::search(&f.db, &query(&f, "")).unwrap()["items"][0]["availableQuantity"],
        2
    );
}
#[test]
fn basic_atomic_sale() {
    let f = f();
    seed(&f, 3, None, None);
    let s = posted(&f, 2);
    assert_eq!(s["status"], "POSTED");
    assert_eq!(balance(&f), 1);
    assert_eq!(count(&f, "sale_payments"), 1);
}
#[test]
fn sale_creates_negative_canonical_movement() {
    let f = f();
    seed(&f, 3, None, None);
    posted(&f, 2);
    let c = f.db.lock().unwrap();
    let delta: i64 = c
        .query_row(
            "SELECT quantity_delta FROM inventory_movements WHERE movement_type='SALE'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(delta, -2);
}
#[test]
fn canonical_fefo_order() {
    let f = f();
    let later = seed(&f, 3, Some("2099-05-01"), None);
    let early = seed(&f, 3, Some("2099-01-01"), None);
    let s = posted(&f, 4);
    assert_eq!(s["items"][0]["allocations"][0]["batchId"], early);
    assert_eq!(s["items"][0]["allocations"][1]["batchId"], later);
}
#[test]
fn expired_stock_cannot_cover_sale() {
    let f = f();
    seed(&f, 8, Some("2000-01-01"), None);
    let s = save(&f, cart(&f, 1));
    assert!(sales::post(&f.db, &post_input(&s)).is_err());
    assert_eq!(balance(&f), 8);
}
#[test]
fn unknown_expiry_last() {
    let f = f();
    let unknown = seed(&f, 2, None, None);
    let known = seed(&f, 1, Some("2099-01-01"), None);
    let s = posted(&f, 2);
    assert_eq!(s["items"][0]["allocations"][0]["batchId"], known);
    assert_eq!(s["items"][0]["allocations"][1]["batchId"], unknown);
}
#[test]
fn multiple_batch_allocations_sum_exactly() {
    let f = f();
    seed(&f, 2, None, None);
    seed(&f, 3, None, None);
    let s = posted(&f, 4);
    let a = s["items"][0]["allocations"].as_array().unwrap();
    assert_eq!(a.len(), 2);
    assert_eq!(
        a.iter()
            .map(|a| a["quantity"].as_i64().unwrap())
            .sum::<i64>(),
        4
    );
}
#[test]
fn mixed_costs_never_use_first_batch_as_line_cost() {
    let f = f();
    seed(&f, 2, Some("2099-01-01"), Some(30));
    seed(&f, 3, Some("2099-02-01"), Some(70));
    let s = posted(&f, 4);
    assert!(s["items"][0]["unitCostMinor"].is_null());
    assert_eq!(s["items"][0]["allocations"][0]["unitCostMinor"], 30);
    assert_eq!(s["items"][0]["allocations"][1]["unitCostMinor"], 70);
}
#[test]
fn duplicate_package_rejected_before_any_stock() {
    let f = f();
    seed(&f, 3, None, None);
    let mut c = cart(&f, 2);
    c.lines.push(c.lines[0].clone());
    assert!(sales::save(
        &f.db,
        &Save {
            id: id(),
            expected_revision: None,
            cart: c
        }
    )
    .is_err());
    assert_eq!(balance(&f), 3);
}
#[test]
fn insufficient_stock_leaves_draft() {
    let f = f();
    seed(&f, 1, None, None);
    let s = save(&f, cart(&f, 2));
    assert!(sales::post(&f.db, &post_input(&s)).is_err());
    assert_eq!(
        sales::detail(&f.db, s["id"].as_str().unwrap()).unwrap()["status"],
        "DRAFT"
    );
    assert_eq!(count(&f, "sale_items"), 0);
}
#[test]
fn concurrent_sales_cannot_overspend() {
    let path = std::env::temp_dir().join(format!("phase4-{}.sqlite", id()));
    let f = fixture(AppDb::open(&path).unwrap());
    seed(&f, 1, None, None);
    let a = post_input(&save(&f, cart(&f, 1)));
    let b = post_input(&save(&f, cart(&f, 1)));
    let db2 = AppDb::open(&path).unwrap();
    let x = std::thread::spawn(move || sales::post(&db2, &b).is_ok());
    let ok = sales::post(&f.db, &a).is_ok();
    assert_ne!(ok, x.join().unwrap());
    assert_eq!(balance(&f), 0);
    drop(f);
    let _ = std::fs::remove_file(path);
}
#[test]
fn payment_insert_failure_rolls_back_allocations_and_number() {
    let f = f();
    seed(&f, 2, None, None);
    let s = save(&f, cart(&f, 1));
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_payment BEFORE INSERT ON sale_payments BEGIN SELECT RAISE(ABORT,'forced payment failure'); END;").unwrap();
    assert!(sales::post(&f.db, &post_input(&s)).is_err());
    assert_eq!(balance(&f), 2);
    assert_eq!(count(&f, "sale_items"), 0);
    assert_eq!(count(&f, "sale_sequences"), 0);
}
#[test]
fn posting_idempotency_exact_and_payload_bound() {
    let f = f();
    seed(&f, 3, None, None);
    let s = save(&f, cart(&f, 1));
    let mut p = post_input(&s);
    let a = sales::post(&f.db, &p).unwrap();
    let b = sales::post(&f.db, &p).unwrap();
    assert_eq!(a, b);
    assert_eq!(balance(&f), 2);
    p.payments[0].amount_minor += 1;
    assert!(sales::post(&f.db, &p).is_err());
    assert_eq!(count(&f, "sale_payments"), 1);
}
#[test]
fn stale_revision_rejected() {
    let f = f();
    let s = save(&f, cart(&f, 1));
    sales::transition(
        &f.db,
        &Transition {
            id: s["id"].as_str().unwrap().into(),
            expected_revision: 1,
        },
        true,
    )
    .unwrap();
    assert!(sales::save(
        &f.db,
        &Save {
            id: s["id"].as_str().unwrap().into(),
            expected_revision: Some(1),
            cart: cart(&f, 2)
        }
    )
    .is_err());
}
#[test]
fn price_snapshot_does_not_follow_catalog() {
    let f = f();
    seed(&f, 2, None, None);
    let s = posted(&f, 1);
    price_history::set(
        &f.db,
        &SetPackagePrice {
            product_package_id: f.p.clone(),
            selling_price_minor: 999,
            cost_price_minor: None,
            reason: None,
        },
    )
    .unwrap();
    let after = sales::detail(&f.db, s["id"].as_str().unwrap()).unwrap();
    assert_eq!(after["items"][0]["unitPriceMinor"], 101);
}
#[test]
fn nullable_cost_is_not_fabricated() {
    let f = f();
    seed(&f, 1, None, None);
    let s = posted(&f, 1);
    assert!(s["items"][0]["unitCostMinor"].is_null());
    assert!(s["items"][0]["allocations"][0]["unitCostMinor"].is_null());
}
#[test]
fn payments_require_exact_total_and_support_split() {
    let f = f();
    seed(&f, 2, None, None);
    let s = save(&f, cart(&f, 1));
    let mut p = post_input(&s);
    p.payments[0].amount_minor = 100;
    assert!(sales::post(&f.db, &p).is_err());
    p.payments.push(Payment {
        payment_method: Method::Card,
        amount_minor: 1,
        reference: Some("terminal receipt".into()),
    });
    assert_eq!(
        sales::post(&f.db, &p).unwrap()["payments"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn line_discount_is_authoritative() {
    let f = f();
    let mut c = cart(&f, 3);
    c.lines[0].discount_minor = 3;
    let s = save(&f, c);
    assert_eq!(s["totalMinor"], 300);
}
#[test]
fn sale_discount_apportions_all_minor_units() {
    let f = f();
    seed(&f, 2, None, None);
    let p2 = product_packages::create(
        &f.db,
        &CreateProductPackage {
            product_id: f.product.clone(),
            package_label: "Strip".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let mut c = cart(&f, 1);
    c.lines.push(Line {
        product_package_id: p2,
        quantity: 1,
        unit_price_minor: Some(100),
        discount_minor: 0,
        display_name: None,
    });
    c.discount_minor = 7;
    let preview = sales::calculate(&f.db.lock().unwrap(), &c, false).unwrap();
    assert_eq!(
        preview
            .lines
            .iter()
            .map(|l| l.sale_discount_minor)
            .sum::<i64>(),
        7
    );
    assert_eq!(preview.total_minor, 194);
}
#[test]
fn simple_tax_uses_integer_apportionment() {
    let f = f();
    let mut c = cart(&f, 3);
    c.discount_minor = 3;
    c.tax_minor = 7;
    let p = sales::calculate(&f.db.lock().unwrap(), &c, false).unwrap();
    assert_eq!(p.total_minor, 307);
    assert_eq!(p.lines[0].tax_minor, 7);
}
#[test]
fn sale_numbers_are_unique() {
    let f = f();
    seed(&f, 3, None, None);
    let a = posted(&f, 1);
    let b = posted(&f, 1);
    assert_ne!(a["saleNumber"], b["saleNumber"]);
    assert!(a["saleNumber"].as_str().unwrap().starts_with("SAL-"));
}
#[test]
fn sales_history_filters_and_pages() {
    let f = f();
    save(&f, cart(&f, 1));
    save(&f, cart(&f, 2));
    let mut q = query(&f, "");
    q.limit = Some(1);
    let a = sales::list(&f.db, &q).unwrap();
    q.offset = Some(1);
    let b = sales::list(&f.db, &q).unwrap();
    assert_eq!(a["total"], 2);
    assert_ne!(a["items"][0]["id"], b["items"][0]["id"]);
    q.status = Some("POSTED".into());
    assert_eq!(sales::list(&f.db, &q).unwrap()["total"], 0);
}
#[test]
fn partial_return_restores_exact_quantity() {
    let f = f();
    seed(&f, 5, None, None);
    let s = posted(&f, 3);
    let r = returned(&f, &s, 1);
    assert_eq!(balance(&f), 3);
    assert_eq!(r["refundedMinor"], 101);
}
#[test]
fn full_return_restores_sale() {
    let f = f();
    seed(&f, 5, None, None);
    let s = posted(&f, 3);
    let r = returned(&f, &s, 3);
    assert_eq!(balance(&f), 5);
    assert_eq!(r["refundedMinor"], 303);
}
#[test]
fn over_return_rejected_after_prior_return() {
    let f = f();
    seed(&f, 3, None, None);
    let s = posted(&f, 3);
    returned(&f, &s, 2);
    assert!(sales::create_return(&f.db, &ret(&s, 2)).is_err());
    assert_eq!(balance(&f), 2);
}
#[test]
fn multibatch_return_follows_original_allocation_order() {
    let f = f();
    let a = seed(&f, 2, Some("2099-01-01"), Some(20));
    let b = seed(&f, 3, Some("2099-02-01"), Some(40));
    let s = posted(&f, 5);
    returned(&f, &s, 4);
    let c = f.db.lock().unwrap();
    for (batch, q) in [(a, 2), (b, 2)] {
        assert_eq!(
            c.query_row(
                "SELECT sum(quantity_delta) FROM inventory_movements WHERE batch_id=?1",
                [batch],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            q
        );
    }
}
#[test]
fn refunds_never_exceed_discounted_paid_total() {
    let f = f();
    seed(&f, 3, None, None);
    let mut c = cart(&f, 3);
    c.discount_minor = 103;
    let s = save(&f, c);
    let s = sales::post(&f.db, &post_input(&s)).unwrap();
    let first = returned(&f, &s, 1);
    assert_eq!(first["refundedMinor"], 66);
    returned(&f, &s, 1);
    let last = returned(&f, &s, 1);
    assert_eq!(last["refundedMinor"], 200);
}
#[test]
fn return_retry_is_durable() {
    let f = f();
    seed(&f, 2, None, None);
    let s = posted(&f, 1);
    let r = ret(&s, 1);
    sales::create_return(&f.db, &r).unwrap();
    let p = ReturnPost {
        id: r.id,
        request_id: id(),
    };
    let a = sales::post_return(&f.db, &p).unwrap();
    assert_eq!(sales::post_return(&f.db, &p).unwrap(), a);
    assert_eq!(balance(&f), 2);
    assert_eq!(count(&f, "sale_refunds"), 1);
}
#[test]
fn void_preserves_original_sale_and_payment() {
    let f = f();
    seed(&f, 3, None, None);
    let s = posted(&f, 2);
    let v = sales::void(
        &f.db,
        &Void {
            id: s["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: "Administrative correction".into(),
        },
    )
    .unwrap();
    assert_eq!(v["status"], "VOIDED");
    assert_eq!(v["payments"], s["payments"]);
    assert_eq!(v["refundedMinor"], 202);
}
#[test]
fn void_same_request_retries_new_request_rejected() {
    let f = f();
    seed(&f, 1, None, None);
    let s = posted(&f, 1);
    let mut v = Void {
        id: s["id"].as_str().unwrap().into(),
        request_id: id(),
        reason: "Admin correction".into(),
    };
    sales::void(&f.db, &v).unwrap();
    sales::void(&f.db, &v).unwrap();
    v.request_id = id();
    assert!(sales::void(&f.db, &v).is_err());
    assert_eq!(count(&f, "sale_voids"), 1);
}
#[test]
fn void_restores_each_original_batch() {
    let f = f();
    seed(&f, 2, None, Some(10));
    seed(&f, 2, None, Some(20));
    let s = posted(&f, 3);
    sales::void(
        &f.db,
        &Void {
            id: s["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: "Void test".into(),
        },
    )
    .unwrap();
    assert_eq!(balance(&f), 4);
    assert_eq!(count(&f, "sale_void_allocations"), 2);
}
#[test]
fn return_and_void_movements_have_durable_references() {
    let f = f();
    seed(&f, 2, None, None);
    let s = posted(&f, 1);
    returned(&f, &s, 1);
    let c = f.db.lock().unwrap();
    assert_eq!(c.query_row("SELECT count(*) FROM inventory_movements WHERE movement_type='SALE_RETURN' AND reference_type='sale_return_allocation' AND reference_id IS NOT NULL",[],|r|r.get::<_,i64>(0)).unwrap(),1);
}
#[test]
fn file_restart_preserves_hold_post_return_and_retry() {
    let path = std::env::temp_dir().join(format!("sales-restart-{}.sqlite", id()));
    let f = fixture(AppDb::open(&path).unwrap());
    seed(&f, 3, None, None);
    let s = posted(&f, 1);
    let hold = save(&f, cart(&f, 1));
    sales::transition(
        &f.db,
        &Transition {
            id: hold["id"].as_str().unwrap().into(),
            expected_revision: 1,
        },
        true,
    )
    .unwrap();
    let r = returned(&f, &s, 1);
    let key = s["id"].as_str().unwrap().to_string();
    drop(f);
    let db = AppDb::open(&path).unwrap();
    assert_eq!(sales::detail(&db, &key).unwrap(), r);
    assert_eq!(
        sales::detail(&db, hold["id"].as_str().unwrap()).unwrap()["status"],
        "HELD"
    );
    drop(db);
    let _ = std::fs::remove_file(path);
}
#[test]
fn inactive_batch_return_fails_atomically() {
    let f = f();
    let b = seed(&f, 2, None, None);
    let s = posted(&f, 1);
    f.db.lock()
        .unwrap()
        .execute("UPDATE inventory_batches SET is_active=0 WHERE id=?1", [b])
        .unwrap();
    assert!(sales::create_return(&f.db, &ret(&s, 1)).is_err());
    assert_eq!(balance(&f), 1);
}
#[test]
fn posted_sale_and_allocations_cannot_be_modified() {
    let f = f();
    seed(&f, 1, None, None);
    posted(&f, 1);
    let c = f.db.lock().unwrap();
    assert!(c.execute("UPDATE sales SET total_minor=0", []).is_err());
    assert!(c
        .execute("UPDATE sale_item_allocations SET quantity=9", [])
        .is_err());
    assert!(c.execute("DELETE FROM sale_payments", []).is_err());
}
#[test]
fn void_after_return_is_rejected() {
    let f = f();
    seed(&f, 2, None, None);
    let s = posted(&f, 2);
    returned(&f, &s, 1);
    assert!(sales::void(
        &f.db,
        &Void {
            id: s["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: "No double restoration".into()
        }
    )
    .is_err());
    assert_eq!(balance(&f), 1);
}
#[test]
fn unsafe_money_is_rejected() {
    let f = f();
    let mut c = cart(&f, 2);
    c.lines[0].unit_price_minor = Some(9007199254740991);
    assert!(sales::save(
        &f.db,
        &Save {
            id: id(),
            expected_revision: None,
            cart: c
        }
    )
    .is_err());
}
#[test]
fn persisted_sale_discount_refunds_multiple_items_exactly() {
    let f = f();
    seed(&f, 1, None, Some(30));
    let p2 = product_packages::create(
        &f.db,
        &CreateProductPackage {
            product_id: f.product.clone(),
            package_label: "Second package".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    movements::opening(
        &f.db,
        &OpeningStock {
            request_id: id(),
            batch: CreateBatch {
                branch_id: f.b.clone(),
                product_package_id: p2.clone(),
                ..Default::default()
            },
            quantity: 1,
            reason: "Second test allocation".into(),
            created_by: None,
        },
    )
    .unwrap();
    let mut c = cart(&f, 1);
    c.lines.push(Line {
        product_package_id: p2,
        quantity: 1,
        unit_price_minor: Some(100),
        ..Default::default()
    });
    c.discount_minor = 7;
    let s = save(&f, c);
    let s = sales::post(&f.db, &post_input(&s)).unwrap();
    assert_eq!(s["items"][0]["saleDiscountMinor"], 3);
    assert_eq!(s["items"][1]["saleDiscountMinor"], 4);
    let first = returned(&f, &s, 1);
    assert_eq!(first["refundedMinor"], 98);
    let mut r = ret(&s, 1);
    r.lines[0].sale_item_id = s["items"][1]["id"].as_str().unwrap().into();
    sales::create_return(&f.db, &r).unwrap();
    let final_sale = sales::post_return(
        &f.db,
        &ReturnPost {
            id: r.id,
            request_id: id(),
        },
    )
    .unwrap();
    assert_eq!(final_sale["refundedMinor"], 194);
    assert_eq!(final_sale["totalMinor"], 194);
}
#[test]
fn return_second_allocation_failure_rolls_back_whole_refund() {
    let f = f();
    seed(&f, 1, Some("2099-01-01"), None);
    let b = seed(&f, 1, Some("2099-02-01"), None);
    let s = posted(&f, 2);
    let r = ret(&s, 2);
    sales::create_return(&f.db, &r).unwrap();
    f.db.lock().unwrap().execute_batch(&format!("CREATE TRIGGER fail_return BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='SALE_RETURN' AND NEW.batch_id='{b}' BEGIN SELECT RAISE(ABORT,'forced second allocation failure'); END;")).unwrap();
    assert!(sales::post_return(
        &f.db,
        &ReturnPost {
            id: r.id,
            request_id: id()
        }
    )
    .is_err());
    assert_eq!(balance(&f), 0);
    assert_eq!(count(&f, "sale_return_items"), 0);
    assert_eq!(count(&f, "sale_refunds"), 0);
}
#[test]
fn void_second_allocation_failure_rolls_back_compensation() {
    let f = f();
    seed(&f, 1, Some("2099-01-01"), None);
    let b = seed(&f, 1, Some("2099-02-01"), None);
    let s = posted(&f, 2);
    f.db.lock().unwrap().execute_batch(&format!("CREATE TRIGGER fail_void BEFORE INSERT ON inventory_movements WHEN NEW.movement_type='SALE_RETURN' AND NEW.batch_id='{b}' BEGIN SELECT RAISE(ABORT,'forced second allocation failure'); END;")).unwrap();
    assert!(sales::void(
        &f.db,
        &Void {
            id: s["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: "Failure test".into()
        }
    )
    .is_err());
    assert_eq!(balance(&f), 0);
    assert_eq!(count(&f, "sale_voids"), 0);
    assert_eq!(count(&f, "sale_refunds"), 0);
    assert_eq!(
        sales::detail(&f.db, s["id"].as_str().unwrap()).unwrap()["status"],
        "POSTED"
    );
}
#[test]
fn void_requires_nonblank_reason() {
    let f = f();
    seed(&f, 1, None, None);
    let s = posted(&f, 1);
    assert!(sales::void(
        &f.db,
        &Void {
            id: s["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: " ".into()
        }
    )
    .is_err());
    assert_eq!(balance(&f), 0);
}
