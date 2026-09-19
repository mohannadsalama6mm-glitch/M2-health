use m2_health_lib::{
    db::{
        connection::AppDb,
        models::{
            branch::CreateBranch,
            catalog::{CreateProduct, CreateProductPackage},
        },
        repositories::{branches, product_packages, products},
    },
    inventory::{
        batches, counts, documents,
        inspection::{self, Query, StockState},
        levels,
        models::*,
        movements,
    },
};
use serde_json::{json, Value};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    b: String,
    p: String,
    product: String,
}
fn f() -> F {
    let db = AppDb::in_memory().unwrap();
    let b = branches::ensure_default_branch(&db).unwrap().id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Inspection medicine".into()),
            commercial_name_ar: Some("دواء".into()),
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
    F { db, b, p, product }
}
fn q(f: &F) -> Query {
    Query {
        branch_id: f.b.clone(),
        ..Default::default()
    }
}
fn batch(f: &F) -> CreateBatch {
    CreateBatch {
        branch_id: f.b.clone(),
        product_package_id: f.p.clone(),
        ..Default::default()
    }
}
fn stock(f: &F, n: i64) -> String {
    movements::opening(
        &f.db,
        &OpeningStock {
            batch: batch(f),
            request_id: id(),
            quantity: n,
            reason: "Inspection fixture".into(),
            created_by: Some("tester".into()),
        },
    )
    .unwrap()
    .batch_id
}
fn level(f: &F, n: i64) {
    levels::set(
        &f.db,
        &SetLevel {
            branch_id: f.b.clone(),
            product_package_id: f.p.clone(),
            reorder_level: n,
            minimum_stock: None,
            maximum_stock: None,
        },
    )
    .unwrap();
}
fn rows(f: &F) -> Vec<Value> {
    inspection::overview(&f.db, &q(f)).unwrap().items
}
#[test]
fn unrelated_catalog_not_loaded() {
    let f = f();
    assert!(rows(&f).is_empty());
}
#[test]
fn active_empty_batch_visible_at_zero() {
    let f = f();
    batches::create(&f.db, &batch(&f)).unwrap();
    assert_eq!(rows(&f)[0]["quantity"], 0);
    assert_eq!(rows(&f)[0]["stockState"], "OUT_OF_STOCK");
}
#[test]
fn configured_zero_without_batch_visible() {
    let f = f();
    level(&f, 2);
    let r = rows(&f);
    assert_eq!(r[0]["batchCount"], 0);
    assert_eq!(r[0]["stockState"], "OUT_OF_STOCK");
}
#[test]
fn history_visible_with_inactive_batch_and_package() {
    let f = f();
    let b = stock(&f, 2);
    let c = f.db.lock().unwrap();
    c.execute("UPDATE inventory_batches SET is_active=0 WHERE id=?1", [b])
        .unwrap();
    c.execute(
        "UPDATE product_packages SET is_active=0 WHERE id=?1",
        [&f.p],
    )
    .unwrap();
    drop(c);
    assert_eq!(rows(&f)[0]["quantity"], 2);
    assert_eq!(rows(&f)[0]["packageActive"], 0);
}
#[test]
fn draft_document_only_package_visible() {
    let f = f();
    let input=serde_json::from_value(json!({"kind":"RECEIPT","branchId":f.b,"lines":[{"productPackageId":f.p,"action":"RECEIVE","quantity":2,"reason":"Draft"}]})).unwrap();
    documents::save(&f.db, &input).unwrap();
    assert_eq!(rows(&f)[0]["quantity"], 0);
}
#[test]
fn no_fake_low_threshold_and_equality_uses_saved_level() {
    let f = f();
    stock(&f, 1);
    assert_eq!(rows(&f)[0]["stockState"], "IN_STOCK");
    assert!(rows(&f)[0]["reorderLevel"].is_null());
    level(&f, 1);
    assert_eq!(rows(&f)[0]["stockState"], "LOW_STOCK");
}
#[test]
fn english_arabic_scientific_search_and_missing() {
    let f = f();
    stock(&f, 1);
    for search in ["INSPECTION", "دواء", "ingredient"] {
        let mut q = q(&f);
        q.search = Some(search.into());
        assert_eq!(inspection::overview(&f.db, &q).unwrap().total, 1);
    }
    let mut q = q(&f);
    q.search = Some("missing".into());
    assert_eq!(inspection::overview(&f.db, &q).unwrap().total, 0);
}
#[test]
fn barcode_manufacturer_search_and_combined_filters() {
    let f = f();
    stock(&f, 1);
    let c = f.db.lock().unwrap();
    let manufacturer = id();
    let category = id();
    let route = id();
    c.execute(
        "INSERT INTO manufacturers(id,name,normalized_name) VALUES(?1,'Acme','acme')",
        [&manufacturer],
    )
    .unwrap();
    c.execute(
        "INSERT INTO categories(id,name,normalized_name) VALUES(?1,'Test','test')",
        [&category],
    )
    .unwrap();
    c.execute(
        "INSERT INTO routes(id,name,normalized_name) VALUES(?1,'Oral','oral')",
        [&route],
    )
    .unwrap();
    c.execute(
        "UPDATE products SET manufacturer_id=?2,category_id=?3,route_id=?4 WHERE id=?1",
        rusqlite::params![f.product, manufacturer, category, route],
    )
    .unwrap();
    c.execute(
        "INSERT INTO barcodes(id,product_package_id,barcode) VALUES(?1,?2,'123456')",
        rusqlite::params![id(), f.p],
    )
    .unwrap();
    drop(c);
    let mut q = q(&f);
    q.manufacturer_id = Some(manufacturer);
    q.category_id = Some(category);
    q.route_id = Some(route);
    for search in ["123456", "Acme"] {
        q.search = Some(search.into());
        assert_eq!(inspection::overview(&f.db, &q).unwrap().total, 1);
    }
    q.stock_state = Some(StockState::OutOfStock);
    assert_eq!(inspection::overview(&f.db, &q).unwrap().total, 0);
}
#[test]
fn branch_isolation() {
    let f = f();
    stock(&f, 5);
    let other = branches::create_branch(
        &f.db,
        &CreateBranch {
            code: "OTHER".into(),
            name: "Other".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap();
    let mut q = q(&f);
    q.branch_id = other.id;
    assert_eq!(inspection::overview(&f.db, &q).unwrap().total, 0);
    assert_eq!(inspection::batch_list(&f.db, &q).unwrap().total, 0);
    assert_eq!(inspection::movement_list(&f.db, &q).unwrap().total, 0);
}
#[test]
fn batch_and_overview_quantities_agree_without_stock_mutation() {
    let f = f();
    stock(&f, 2);
    stock(&f, 3);
    let r = inspection::batch_list(&f.db, &q(&f)).unwrap();
    assert_eq!(r.total, 2);
    assert_eq!(
        r.items
            .iter()
            .map(|r| r["quantity"].as_i64().unwrap())
            .sum::<i64>(),
        5
    );
    assert_eq!(rows(&f)[0]["quantity"], 5);
    assert_eq!(inspection::movement_list(&f.db, &q(&f)).unwrap().total, 2);
}
#[test]
fn sql_pagination_retains_total_and_no_overlap() {
    let f = f();
    for _ in 0..3 {
        stock(&f, 1);
    }
    let mut q = q(&f);
    q.limit = Some(1);
    let a = inspection::movement_list(&f.db, &q).unwrap();
    q.offset = Some(1);
    let b = inspection::movement_list(&f.db, &q).unwrap();
    assert_eq!(a.total, 3);
    assert_eq!(a.items.len(), 1);
    assert_ne!(a.items[0]["id"], b.items[0]["id"]);
    assert_eq!(inspection::batch_list(&f.db, &q).unwrap().items.len(), 1);
}
#[test]
fn movement_filters_and_actor() {
    let f = f();
    let b = stock(&f, 1);
    let mut q = q(&f);
    q.batch_id = Some(b);
    q.product_package_id = Some(f.p.clone());
    q.movement_type = Some(MovementType::Opening);
    let r = inspection::movement_list(&f.db, &q).unwrap();
    assert_eq!(r.items[0]["createdBy"], "tester");
    q.movement_type = Some(MovementType::Damage);
    assert_eq!(inspection::movement_list(&f.db, &q).unwrap().total, 0);
    q.movement_type = None;
    q.date_to = Some("2000-01-01".into());
    assert_eq!(inspection::movement_list(&f.db, &q).unwrap().total, 0);
}
#[test]
fn traceability_links_receipt_and_stock_count() {
    let f = f();
    let d=documents::save(&f.db,&serde_json::from_value(json!({"kind":"RECEIPT","branchId":f.b,"lines":[{"productPackageId":f.p,"action":"RECEIVE","quantity":1,"reason":"Receipt test"}]})).unwrap()).unwrap();
    let d = documents::ready(
        &f.db,
        &documents::models::Transition {
            document_id: d.header.id,
            expected_revision: d.header.revision,
        },
    )
    .unwrap();
    let d = documents::post(
        &f.db,
        &documents::models::PostDocument {
            document_id: d.header.id,
            expected_revision: d.header.revision,
            request_id: id(),
        },
    )
    .unwrap();
    let r = inspection::movement_list(&f.db, &q(&f)).unwrap();
    assert_eq!(r.items[0]["documentId"], d.header.id);
    let mut query = q(&f);
    query.document_reference = Some(d.header.document_number.clone());
    assert_eq!(inspection::movement_list(&f.db, &query).unwrap().total, 1);
    let d = counts::create(
        &f.db,
        &counts::models::Create {
            branch_id: f.b.clone(),
            notes: None,
            created_by: None,
        },
    )
    .unwrap();
    let tr = |d: &counts::models::Detail| counts::models::Transition {
        session_id: d.session.id.clone(),
        expected_revision: d.session.revision,
    };
    let d = counts::start(&f.db, &tr(&d)).unwrap();
    let d = counts::record(
        &f.db,
        &counts::models::Entry {
            session_id: d.session.id.clone(),
            expected_revision: d.session.revision,
            line_id: d.lines[0].id.clone(),
            line_revision: d.lines[0].revision,
            counted_quantity: Some(2),
            notes: None,
            excluded_reason: None,
        },
    )
    .unwrap();
    let d = counts::transition(&f.db, &tr(&d), counts::models::Status::Review).unwrap();
    let d = counts::complete(
        &f.db,
        &counts::models::Complete {
            session_id: d.session.id,
            expected_revision: d.session.revision,
            request_id: id(),
            review_token: d.review_token,
        },
    )
    .unwrap();
    let r = inspection::movement_list(&f.db, &q(&f)).unwrap();
    assert!(r.items.iter().any(|r| r["countSessionId"] == d.session.id));
}
#[test]
fn invalid_queries_rejected() {
    let f = f();
    let mut q = q(&f);
    q.limit = Some(201);
    assert!(inspection::overview(&f.db, &q).is_err());
    q.limit = None;
    q.date_from = Some("2026-02-30".into());
    assert!(inspection::movement_list(&f.db, &q).is_err());
    q.date_from = Some("2026-04-01".into());
    q.date_to = Some("2026-03-01".into());
    assert!(inspection::movement_list(&f.db, &q).is_err());
}
