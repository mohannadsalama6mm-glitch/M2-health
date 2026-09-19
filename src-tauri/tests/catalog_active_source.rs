use m2_health_lib::{
    catalog_import::{active, importer, planner, source},
    db::connection::AppDb,
};
use std::{fs, path::Path};
fn row(name: &str, price: &str) -> Vec<String> {
    [
        name,
        "دواء عربي",
        "Ingredient A",
        "Maker",
        "Class",
        "ORAL.SOLID",
        price,
    ]
    .map(str::to_owned)
    .to_vec()
}
fn configure(path: &Path, rows: usize) {
    let bytes = fs::read(path.join("source.csv")).unwrap();
    fs::write(path.join("config/catalog-source.json"),serde_json::to_vec(&serde_json::json!({
        "id":"egyptian-drugs","path":"source.csv","format":"csv","encoding":"utf-8-sig","delimiter":",",
        "current_version":"v2","import_policy":active::POLICY,
        "versions":[{"version":"v2","sha256":source::hash(&bytes),"size_bytes":bytes.len(),"data_rows":rows,"apply_allowed":true,"trim_headers":true}]
    })).unwrap()).unwrap();
}
fn write(path: &Path, rows: &[Vec<String>]) {
    let mut writer = csv::Writer::from_path(path.join("source.csv")).unwrap();
    writer.write_record(source::HEADERS).unwrap();
    for r in rows {
        writer.write_record(r).unwrap();
    }
    writer.flush().unwrap();
    configure(path, rows.len());
}
fn fixture(rows: &[Vec<String>]) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir(d.path().join("config")).unwrap();
    write(d.path(), rows);
    d
}
fn count(db: &AppDb, table: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn apply(db: &AppDb, d: &Path) -> m2_health_lib::catalog_import::Report {
    let plan = planner::build(db, d, d).unwrap();
    importer::apply(db, d, d, plan, |_| {}).unwrap()
}
#[test]
fn real_catalog_insert_reuse_and_readonly_idempotency_record_versioned_actions() {
    let d = fixture(&[row("First 10 MG 20 TABS", "22.50")]);
    let db = AppDb::in_memory().unwrap();
    let first = apply(&db, d.path());
    assert_eq!(first.status, "complete");
    assert_eq!(first.imported, 1);
    assert_eq!(count(&db, "products"), 1);
    assert_eq!(count(&db, "product_packages"), 1);
    assert_eq!(count(&db, "product_price_history"), 1);
    assert_eq!(count(&db, "catalog_source_rows"), 1);
    assert_eq!(count(&db, "catalog_import_rows"), 0);
    let before = planner::snapshot(&db.lock().unwrap()).unwrap();
    let next = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(next.report.counts["already_imported"], 1);
    assert_eq!(next.report.counts.get("ready_insert"), None);
    assert_eq!(before, planner::snapshot(&db.lock().unwrap()).unwrap());
    assert_eq!(count(&db, "catalog_import_runs"), 1);
    let second = importer::apply(&db, d.path(), d.path(), next, |_| {}).unwrap();
    assert_eq!(second.reused, 1);
    assert_eq!(second.imported, 0);
    assert_eq!(count(&db, "products"), 1);
    assert_eq!(count(&db, "product_price_history"), 1);
    let c = db.lock().unwrap();
    assert_eq!(
        c.query_row(
            "SELECT source_version FROM catalog_source_runs LIMIT 1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "v2"
    );
    assert!(c.execute("DELETE FROM catalog_source_rows", []).is_err());
    let backup = rusqlite::Connection::open(first.backup_path.unwrap()).unwrap();
    assert_eq!(
        backup
            .query_row("SELECT count(*) FROM products", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn changed_price_appends_history_preserves_original_amount_and_cost() {
    let d = fixture(&[row("First", "10")]);
    let db = AppDb::in_memory().unwrap();
    apply(&db, d.path());
    let original: String = db
        .lock()
        .unwrap()
        .query_row("SELECT id FROM product_price_history", [], |r| r.get(0))
        .unwrap();
    write(d.path(), &[row("First", "11.25")]);
    let r = apply(&db, d.path());
    assert_eq!(r.updated, 1);
    assert_eq!(r.imported, 0);
    assert_eq!(count(&db, "product_price_history"), 2);
    assert_eq!(
        db.lock()
            .unwrap()
            .query_row(
                "SELECT selling_price_minor FROM product_price_history WHERE id=?1",
                [original],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1000
    );
    assert_eq!(
        db.lock()
            .unwrap()
            .query_row(
                "SELECT selling_price_minor FROM product_price_history WHERE effective_to IS NULL",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1125
    );
    assert_eq!(apply(&db, d.path()).updated, 0);
    assert_eq!(count(&db, "product_price_history"), 2);
}
#[test]
fn name_only_collision_never_merges_or_inserts_a_second_identity() {
    let d = fixture(&[row("First", "10")]);
    let db = AppDb::in_memory().unwrap();
    apply(&db, d.path());
    let mut changed = row("First", "99");
    changed[2] = "Different ingredient".into();
    changed[3] = "Other manufacturer".into();
    write(d.path(), &[changed]);
    let r = apply(&db, d.path());
    assert_eq!(r.counts["skipped_ambiguous"], 1);
    assert_eq!(r.updated, 0);
    assert_eq!(r.imported, 0);
    assert_eq!(count(&db, "products"), 1);
    assert_eq!(count(&db, "product_price_history"), 1);
}
#[test]
fn source_duplicate_conflicts_and_invalid_prices_are_skipped() {
    let a = row("Duplicate", "10");
    let mut b = row("Conflict", "10");
    b[1] = "other arabic".into();
    let mut other = b.clone();
    other[3] = "Different maker".into();
    let d = fixture(&[a.clone(), a, b, other, row("Precision", "4.415")]);
    let db = AppDb::in_memory().unwrap();
    let r = apply(&db, d.path());
    assert_eq!(r.imported, 1);
    assert_eq!(r.counts["skipped_duplicate"], 1);
    assert_eq!(r.counts["skipped_ambiguous"], 2);
    assert_eq!(r.counts["skipped_price"], 1);
    assert_eq!(count(&db, "catalog_source_rows"), 5);
}
#[test]
fn malformed_and_padded_quotes_do_not_fail_safe_rows() {
    let d = fixture(&[row("Good", "10")]);
    let path = d.path().join("source.csv");
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("fragment,only\n\"Padded\" ,Arabic,Science,Maker,Class,ORAL.SOLID,10\n");
    fs::write(path, text).unwrap();
    configure(d.path(), 3);
    let db = AppDb::in_memory().unwrap();
    let r = apply(&db, d.path());
    assert_eq!(r.status, "complete");
    assert_eq!(r.imported, 1);
    assert_eq!(r.counts["skipped_malformed"], 1);
    assert_eq!(r.counts["skipped_quoting"], 1);
    assert_eq!(count(&db, "catalog_source_rows"), 3);
}
#[test]
fn scientific_and_arabic_values_are_preserved_and_missing_lookups_stay_null() {
    let mut f = row(" New medicine ", "84");
    f[2] = "Science, with \"quoted\" text".into();
    f[3] = " ".into();
    f[4] = "".into();
    let d = fixture(&[f.clone()]);
    let db = AppDb::in_memory().unwrap();
    let r = apply(&db, d.path());
    assert_eq!(r.imported, 1);
    let c = db.lock().unwrap();
    let stored:(String,String,String,Option<String>,Option<String>)=c.query_row("SELECT commercial_name_en,commercial_name_ar,scientific_name,manufacturer_id,category_id FROM products",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(
        stored,
        (f[0].clone(), f[1].clone(), f[2].clone(), None, None)
    );
    drop(c);
    for table in [
        "barcodes",
        "active_ingredients",
        "product_active_ingredients",
        "inventory_movements",
        "inventory_documents",
        "stock_count_sessions",
    ] {
        assert_eq!(count(&db, table), 0);
    }
}
#[test]
fn failure_rolls_back_products_prices_actions_and_preserves_failed_run() {
    let mut second = row("Second", "20");
    second[1] = "Second Arabic".into();
    let d = fixture(&[row("First", "10"), second]);
    let db = AppDb::in_memory().unwrap();
    db.lock().unwrap().execute_batch("CREATE TRIGGER reject_second BEFORE INSERT ON products WHEN NEW.commercial_name_en='Second' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    let r = apply(&db, d.path());
    assert_eq!(r.status, "failed");
    assert_eq!(r.imported, 0);
    assert_eq!(r.failed, 2);
    for table in [
        "products",
        "product_packages",
        "product_price_history",
        "catalog_source_rows",
        "manufacturers",
        "categories",
        "routes",
    ] {
        assert_eq!(count(&db, table), 0);
    }
    assert_eq!(count(&db, "catalog_import_runs"), 1);
    assert_eq!(count(&db, "catalog_source_runs"), 1);
}
#[test]
fn stale_catalog_plan_is_refused_before_mutation() {
    let d = fixture(&[row("First", "10")]);
    let db = AppDb::in_memory().unwrap();
    let plan = planner::build(&db, d.path(), d.path()).unwrap();
    db.lock()
        .unwrap()
        .execute(
            "INSERT INTO manufacturers(id,name,normalized_name) VALUES ('other','Other','other')",
            [],
        )
        .unwrap();
    assert!(importer::apply(&db, d.path(), d.path(), plan, |_| {}).is_err());
    assert_eq!(count(&db, "catalog_import_runs"), 0);
    assert!(!d.path().join("backups").exists());
}
