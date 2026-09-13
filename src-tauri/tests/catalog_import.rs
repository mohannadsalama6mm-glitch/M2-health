use m2_health_lib::{
    catalog_import::{importer, planner, source},
    db::{
        catalog_validation::{egp_to_minor, normalize},
        connection::AppDb,
        migrations::{self, MIGRATIONS},
    },
};
use std::{fs, path::Path};
fn fixture(rows: &[Vec<String>]) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir(d.path().join("config")).unwrap();
    fs::write(d.path().join("config/catalog-source.json"),r#"{"id":"egyptian-drugs","path":"source.csv","format":"csv","encoding":"utf-8-sig","delimiter":","}"#).unwrap();
    let mut w = csv::Writer::from_path(d.path().join("source.csv")).unwrap();
    w.write_record(source::HEADERS).unwrap();
    for row in rows {
        w.write_record(row).unwrap();
    }
    w.flush().unwrap();
    d
}
fn row(name: &str, price: &str) -> Vec<String> {
    [
        name,
        "اسم عربي",
        "Ingredient A + Ingredient B",
        " Maker ",
        " Class ",
        "UNKNOWN",
        price,
    ]
    .map(str::to_owned)
    .to_vec()
}
fn count(db: &AppDb, table: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[test]
fn canonical_resolution_and_utf8_multiline_profile() {
    let mut r = row("Test", "22.50");
    r[2] = "A, B\nC".into();
    let d = fixture(&[r.clone()]);
    let (p, rows) = source::read(d.path()).unwrap();
    assert_eq!(rows[0], r);
    assert_eq!(p.total_rows, 1);
    assert_eq!(
        p.hash,
        source::hash(&fs::read(d.path().join("source.csv")).unwrap())
    );
    assert_eq!(p.columns, source::HEADERS);
}
#[test]
fn structural_errors_and_escape_are_fatal() {
    let d = fixture(&[row("One", "1")]);
    fs::write(d.path().join("source.csv"), "wrong,header\na,b").unwrap();
    assert!(source::read(d.path()).is_err());
    fs::write(d.path().join("config/catalog-source.json"),r#"{"id":"egyptian-drugs","path":"../missing.csv","format":"csv","encoding":"utf-8-sig","delimiter":","}"#).unwrap();
    assert!(source::read(d.path()).is_err());
}
#[test]
fn money_is_exact_and_conservative() {
    for (s, n) in [("22.50", 2250), ("84", 8400), ("0", 0), ("0.3", 30)] {
        assert_eq!(egp_to_minor(s).unwrap(), n);
    }
    for s in ["4.415", "", "bad", "-1", "1e3", "90071992547410"] {
        assert!(egp_to_minor(s).is_err());
    }
    assert_eq!(normalize("  SOME\n Name  "), "some name");
    assert_eq!(normalize("دواء عربي"), "دواء عربي");
}
#[test]
fn dry_run_dispositions_and_no_database_writes() {
    let mut rows = vec![
        row("Good", "22.50"),
        row("Good", "22.50"),
        row("Uncertain", "1"),
        row(" uncertain ", "2"),
        row("Precision", "4.415"),
        row("Bad", "-1"),
        row("Optional", "84"),
    ];
    rows[6][2..5].fill(String::new());
    let d = fixture(&rows);
    let db = AppDb::in_memory().unwrap();
    let before = planner::snapshot(&db.lock().unwrap()).unwrap();
    let changes: i64 = db
        .lock()
        .unwrap()
        .query_row("SELECT total_changes()", [], |r| r.get(0))
        .unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(p.report.counts["ready"], 2);
    assert_eq!(p.report.counts["exact_duplicate_source"], 1);
    assert_eq!(p.report.counts["possible_duplicate"], 2);
    assert_eq!(p.report.counts["ambiguous_price"], 1);
    assert_eq!(p.report.counts["invalid_price"], 1);
    assert_eq!(p.report.counts["missing_manufacturer"], 1);
    assert_eq!(p.report.counts["review_required"], 4);
    assert_eq!(count(&db, "catalog_import_runs"), 0);
    assert_eq!(before, planner::snapshot(&db.lock().unwrap()).unwrap());
    assert_eq!(
        changes,
        db.lock()
            .unwrap()
            .query_row("SELECT total_changes()", [], |r| r.get::<_, i64>(0))
            .unwrap()
    );
    assert!(Path::new(&p.report.report_path).exists());
}
#[test]
fn apply_mapping_backup_reports_and_repeat_idempotency() {
    let mut b = row("Second", "84");
    b[3] = "maker".into();
    b[4] = "class".into();
    let mut missing = row("Missing", "1");
    missing[2..5].fill(String::new());
    let d = fixture(&[row("First", "22.50"), b, missing]);
    let db = AppDb::open(&d.path().join("test.db")).unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    let r = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(r.status, "complete");
    assert_eq!(r.imported, 3);
    assert_eq!(count(&db, "products"), 3);
    assert_eq!(count(&db, "product_packages"), 3);
    assert_eq!(count(&db, "product_price_history"), 3);
    for t in ["manufacturers", "categories", "routes"] {
        assert_eq!(count(&db, t), 1);
    }
    for t in [
        "barcodes",
        "active_ingredients",
        "product_active_ingredients",
    ] {
        assert_eq!(count(&db, t), 0);
    }
    let c = db.lock().unwrap();
    assert_eq!(c.query_row("SELECT count(*) FROM product_packages WHERE package_label='Default Package' AND pack_size IS NULL AND unit_name IS NULL AND units_per_package IS NULL AND is_default=1",[],|r|r.get::<_,i64>(0)).unwrap(),3);
    assert_eq!(
        c.query_row(
            "SELECT count(DISTINCT effective_from) FROM product_price_history",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT scientific_name FROM products WHERE commercial_name_en='First'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "Ingredient A + Ingredient B"
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM product_price_history WHERE selling_price_minor=2250",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    drop(c);
    let backup = rusqlite::Connection::open(r.backup_path.unwrap()).unwrap();
    assert_eq!(
        backup
            .query_row("SELECT count(*) FROM products", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(p.report.counts["already_imported"], 3);
    let second = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(second.imported, 0);
    assert_eq!(count(&db, "products"), 3);
    assert_eq!(count(&db, "catalog_import_runs"), 2);
}
#[test]
fn changed_source_and_catalog_refuse_stale_plan() {
    let d = fixture(&[row("One", "1")]);
    let db = AppDb::in_memory().unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    db.lock()
        .unwrap()
        .execute(
            "INSERT INTO routes(id,name,normalized_name) VALUES('x','X','x')",
            [],
        )
        .unwrap();
    assert!(importer::apply(&db, d.path(), d.path(), p, |_| {}).is_err());
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    fs::write(d.path().join("source.csv"), "wrong").unwrap();
    assert!(importer::apply(&db, d.path(), d.path(), p, |_| {}).is_err());
    assert_eq!(count(&db, "products"), 0);
}
#[test]
fn failed_batch_rolls_back_and_retry_preserves_committed_batches() {
    let rows = (0..501)
        .map(|i| row(&format!("Item {i}"), "1"))
        .collect::<Vec<_>>();
    let d = fixture(&rows);
    let db = AppDb::in_memory().unwrap();
    db.lock().unwrap().execute_batch("CREATE TRIGGER injected_failure BEFORE INSERT ON products WHEN NEW.commercial_name_en='Item 260' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    let r = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(r.status, "failed");
    assert_eq!(r.imported, 250);
    for t in [
        "products",
        "product_packages",
        "product_price_history",
        "catalog_import_rows",
    ] {
        assert_eq!(count(&db, t), 250);
    }
    db.lock()
        .unwrap()
        .execute_batch("DROP TRIGGER injected_failure")
        .unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(p.report.counts["already_imported"], 250);
    assert_eq!(p.report.counts["ready"], 251);
    let r = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(r.imported, 251);
    assert_eq!(count(&db, "products"), 501);
}
#[test]
fn twenty_five_thousand_row_scale_smoke() {
    let rows = (0..25000)
        .map(|i| row(&format!("Scale {i}"), "22.50"))
        .collect::<Vec<_>>();
    let d = fixture(&rows);
    let db = AppDb::open(&d.path().join("scale.db")).unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(p.report.counts["ready"], 25000);
    let r = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(r.imported, 25000);
    assert_eq!(count(&db, "products"), 25000);
    eprintln!(
        "25k import: {} ms, {} rows/s",
        r.duration_ms, r.rows_per_second
    );
}
#[test]
fn migration_three_preserves_v2_catalog() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("v2.db");
    {
        let mut c = rusqlite::Connection::open(&path).unwrap();
        migrations::apply(&mut c, &MIGRATIONS[..2]).unwrap();
        c.execute(
            "INSERT INTO products(id,commercial_name_en) VALUES ('preserve','Existing')",
            [],
        )
        .unwrap();
    }
    let db = AppDb::open(&path).unwrap();
    assert_eq!(count(&db, "products"), 1);
    assert_eq!(count(&db, "catalog_import_runs"), 0);
}

#[test]
fn existing_identity_and_name_conflicts_are_reviewed_and_lookup_ids_reused() {
    let d = fixture(&[row("Existing", "22.50"), row("Different", "2")]);
    let db = AppDb::in_memory().unwrap();
    {
        let c = db.lock().unwrap();
        for table in ["manufacturers", "categories", "routes"] {
            let name = match table {
                "manufacturers" => "maker",
                "categories" => "class",
                _ => "unknown",
            };
            c.execute(
                &format!("INSERT INTO {table}(id,name,normalized_name) VALUES (?1,?2,?2)"),
                [table, name],
            )
            .unwrap();
        }
        c.execute("INSERT INTO products(id,commercial_name_en,commercial_name_ar,scientific_name,manufacturer_id,category_id,route_id) VALUES ('existing','Existing','اسم عربي','Ingredient A + Ingredient B','manufacturers','categories','routes')",[]).unwrap();
    }
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(p.report.counts["duplicate_existing_catalog"], 1);
    assert_eq!(p.report.counts["ready"], 1);
    assert_eq!(p.report.lookup_reused["manufacturers"], 1);
    let r = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(r.imported, 1);
    for table in ["manufacturers", "categories", "routes"] {
        assert_eq!(count(&db, table), 1);
    }
    let backup = rusqlite::Connection::open(r.backup_path.unwrap()).unwrap();
    assert_eq!(
        backup
            .query_row("SELECT count(*) FROM products", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn invalid_required_data_is_skipped_and_all_rows_have_dispositions() {
    let mut bad = row("", "1");
    bad[1] = String::new();
    let mut long = row("Long", "1");
    long[2] = "a".repeat(4001);
    let d = fixture(&[bad, long, row("Valid", "0")]);
    let db = AppDb::in_memory().unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    assert_eq!(p.report.counts["invalid_required_data"], 2);
    let r = importer::apply(&db, d.path(), d.path(), p, |_| {}).unwrap();
    assert_eq!(r.rows.len(), 3);
    assert_eq!(r.imported, 1);
    assert!(r.rows.iter().all(|r| r.disposition != "ready"));
}

#[test]
fn another_connection_write_stops_between_batches_without_corruption() {
    let d = fixture(
        &(0..251)
            .map(|i| row(&format!("Concurrent {i}"), "1"))
            .collect::<Vec<_>>(),
    );
    let path = d.path().join("concurrent.db");
    let db = AppDb::open(&path).unwrap();
    let other = rusqlite::Connection::open(&path).unwrap();
    let p = planner::build(&db, d.path(), d.path()).unwrap();
    let r=importer::apply(&db,d.path(),d.path(),p,|r|{
        if r.status=="applying" && r.imported==250 {other.execute("INSERT INTO routes(id,name,normalized_name) VALUES('external','External','external')",[]).unwrap();}
    }).unwrap();
    assert_eq!(r.status, "failed");
    assert_eq!(r.imported, 250);
    assert_eq!(count(&db, "products"), 250);
}
