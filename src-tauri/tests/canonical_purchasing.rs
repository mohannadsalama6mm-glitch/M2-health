use m2_health_lib::{
    db::{
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        models::catalog::{CreateProduct, CreateProductPackage},
        repositories::{product_packages, products},
    },
    error::ErrorCode,
    partners::{self, models::SupplierFields},
    purchasing::{self, models::*},
};

struct Fixture {
    db: AppDb,
    supplier: String,
    other_supplier: String,
    package: String,
    other_package: String,
}
fn supplier_fields(name: &str, tax: &str) -> SupplierFields {
    SupplierFields {
        display_name: name.into(),
        tax_registration_id: Some(tax.into()),
        ..Default::default()
    }
}
fn fixture(db: AppDb) -> Fixture {
    let supplier = partners::create_supplier(&db, &supplier_fields("Nile Medical", "TAX-1"))
        .unwrap()
        .id;
    let other_supplier = partners::create_supplier(&db, &supplier_fields("Delta Pharma", "TAX-2"))
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
    let other = products::create(
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
            product_id: other.id,
            package_label: "Bottle".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    Fixture {
        db,
        supplier,
        other_supplier,
        package,
        other_package,
    }
}
fn f() -> Fixture {
    fixture(AppDb::in_memory().unwrap())
}
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn fields(f: &Fixture) -> PurchaseDraftFields {
    PurchaseDraftFields {
        supplier_id: f.supplier.clone(),
        supplier_reference: Some(" REF-10 ".into()),
        expected_date: Some("2030-05-01".into()),
        notes: Some(" Restock ".into()),
        discount_minor: 50,
        tax_minor: 20,
        lines: vec![PurchaseLineInput {
            product_package_id: f.package.clone(),
            ordered_quantity: 3,
            unit_cost_minor: 100,
        }],
    }
}
fn create(f: &Fixture) -> PurchaseOrder {
    purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: fields(f),
        },
    )
    .unwrap()
}
fn movement_count(db: &AppDb) -> i64 {
    db.lock()
        .unwrap()
        .query_row("SELECT count(*) FROM inventory_movements", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn migration_011_creates_only_pre_receiving_schema() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..11]).unwrap();
    let c = db.lock().unwrap();
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        11
    );
    for t in [
        "purchase_orders",
        "purchase_order_lines",
        "purchase_order_requests",
    ] {
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [t],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
    }
    assert_eq!(c.query_row("SELECT count(*) FROM sqlite_master WHERE name LIKE 'purchase_receipt%' OR name LIKE 'supplier_payment%'",[],|r|r.get::<_,i64>(0)).unwrap(),0)
}
#[test]
fn migration_010_to_011_applies_once_and_preserves_partners() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..10]).unwrap();
    let s = partners::create_supplier(&db, &supplier_fields("Keep", "KEEP")).unwrap();
    let mut c = db.lock().unwrap();
    migrations::run(&mut c).unwrap();
    migrations::run(&mut c).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM _schema_version WHERE version=11",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT display_name FROM suppliers WHERE id=?1",
            [s.id],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "Keep"
    )
}
#[test]
fn valid_draft_has_number_supplier_lines_and_backend_totals() {
    let f = f();
    let p = create(&f);
    assert!(p.purchase_number.starts_with("PO-"));
    assert_eq!(p.status, "DRAFT");
    assert_eq!(p.supplier_name, "Nile Medical");
    assert_eq!(p.lines.len(), 1);
    assert_eq!(
        (
            p.subtotal_minor,
            p.discount_minor,
            p.tax_minor,
            p.total_minor
        ),
        (300, 50, 20, 270)
    );
    assert_eq!(p.supplier_reference.as_deref(), Some("REF-10"));
    assert_eq!(movement_count(&f.db), 0)
}
#[test]
fn create_is_idempotent_and_payload_bound() {
    let f = f();
    let input = CreatePurchaseDraft {
        id: id(),
        request_id: id(),
        fields: fields(&f),
    };
    let a = purchasing::create_draft(&f.db, &input).unwrap();
    let b = purchasing::create_draft(&f.db, &input).unwrap();
    assert_eq!(a.id, b.id);
    let mut changed = input;
    changed.fields.notes = Some("changed".into());
    assert_eq!(
        purchasing::create_draft(&f.db, &changed).unwrap_err().code,
        ErrorCode::Validation
    );
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM purchase_orders", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    )
}
#[test]
fn invalid_missing_and_inactive_supplier_are_rejected() {
    let f = f();
    let mut v = fields(&f);
    v.supplier_id = id();
    assert_eq!(
        purchasing::create_draft(
            &f.db,
            &CreatePurchaseDraft {
                id: id(),
                request_id: id(),
                fields: v
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::NotFound
    );
    let s = partners::get_supplier(&f.db, &f.supplier).unwrap();
    partners::set_supplier_active(
        &f.db,
        &m2_health_lib::partners::models::SetPartnerActive {
            id: s.id.clone(),
            expected_revision: s.revision,
            active: false,
        },
    )
    .unwrap();
    let mut v = fields(&f);
    v.supplier_id = s.id;
    assert_eq!(
        purchasing::create_draft(
            &f.db,
            &CreatePurchaseDraft {
                id: id(),
                request_id: id(),
                fields: v
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    )
}
#[test]
fn invalid_lines_quantities_costs_duplicates_and_totals_roll_back() {
    let f = f();
    for mutate in 0..5 {
        let mut v = fields(&f);
        match mutate {
            0 => v.lines[0].ordered_quantity = 0,
            1 => v.lines[0].ordered_quantity = -1,
            2 => v.lines[0].unit_cost_minor = -1,
            3 => v.lines.push(v.lines[0].clone()),
            _ => v.discount_minor = 9999,
        };
        assert!(purchasing::create_draft(
            &f.db,
            &CreatePurchaseDraft {
                id: id(),
                request_id: id(),
                fields: v
            }
        )
        .is_err());
    }
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM purchase_orders", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    let p = create(&f);
    assert!(p.purchase_number.ends_with("0001"))
}
#[test]
fn concurrent_numbering_is_unique_and_sequential() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("purchase.db");
    let base = fixture(AppDb::open(&path).unwrap());
    let supplier = base.supplier.clone();
    let package = base.package.clone();
    drop(base);
    let run = move |db: AppDb, name: &str| {
        purchasing::create_draft(
            &db,
            &CreatePurchaseDraft {
                id: id(),
                request_id: id(),
                fields: PurchaseDraftFields {
                    supplier_id: supplier.clone(),
                    supplier_reference: Some(name.into()),
                    expected_date: None,
                    notes: None,
                    discount_minor: 0,
                    tax_minor: 0,
                    lines: vec![PurchaseLineInput {
                        product_package_id: package.clone(),
                        ordered_quantity: 1,
                        unit_cost_minor: 10,
                    }],
                },
            },
        )
        .unwrap()
        .purchase_number
    };
    let db1 = AppDb::open(&path).unwrap();
    let db2 = AppDb::open(&path).unwrap();
    let supplier2 = partners::list_suppliers(&db2, &Default::default())
        .unwrap()
        .items[0]
        .id
        .clone();
    let package2 = db2
        .lock()
        .unwrap()
        .query_row(
            "SELECT id FROM product_packages ORDER BY id LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap();
    let a = std::thread::spawn(move || run(db1, "A"));
    let b = std::thread::spawn(move || {
        purchasing::create_draft(
            &db2,
            &CreatePurchaseDraft {
                id: id(),
                request_id: id(),
                fields: PurchaseDraftFields {
                    supplier_id: supplier2,
                    supplier_reference: Some("B".into()),
                    expected_date: None,
                    notes: None,
                    discount_minor: 0,
                    tax_minor: 0,
                    lines: vec![PurchaseLineInput {
                        product_package_id: package2,
                        ordered_quantity: 1,
                        unit_cost_minor: 10,
                    }],
                },
            },
        )
        .unwrap()
        .purchase_number
    });
    let mut values = vec![a.join().unwrap(), b.join().unwrap()];
    values.sort();
    assert!(values[0].ends_with("0001"));
    assert!(values[1].ends_with("0002"))
}
#[test]
fn draft_update_changes_supplier_lines_metadata_and_revision_atomically() {
    let f = f();
    let p = create(&f);
    let mut v = fields(&f);
    v.supplier_id = f.other_supplier.clone();
    v.supplier_reference = Some("NEW".into());
    v.lines = vec![PurchaseLineInput {
        product_package_id: f.other_package.clone(),
        ordered_quantity: 2,
        unit_cost_minor: 70,
    }];
    let u = purchasing::update_draft(
        &f.db,
        &UpdatePurchaseDraft {
            id: p.id,
            expected_revision: 1,
            fields: v,
        },
    )
    .unwrap();
    assert_eq!(u.revision, 2);
    assert_eq!(u.supplier_name, "Delta Pharma");
    assert_eq!(u.lines[0].product_name_snapshot, "Medicine Two");
    assert_eq!(u.total_minor, 110);
    assert_eq!(movement_count(&f.db), 0)
}
#[test]
fn stale_update_rejected_and_original_remains() {
    let f = f();
    let p = create(&f);
    let mut first = fields(&f);
    first.notes = Some("first".into());
    purchasing::update_draft(
        &f.db,
        &UpdatePurchaseDraft {
            id: p.id.clone(),
            expected_revision: 1,
            fields: first,
        },
    )
    .unwrap();
    let mut stale = fields(&f);
    stale.notes = Some("stale".into());
    assert_eq!(
        purchasing::update_draft(
            &f.db,
            &UpdatePurchaseDraft {
                id: p.id.clone(),
                expected_revision: 1,
                fields: stale
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    assert_eq!(
        purchasing::detail(&f.db, &p.id).unwrap().notes.as_deref(),
        Some("first")
    )
}
#[test]
fn failed_line_replacement_rolls_back_header_and_lines() {
    let f = f();
    let p = create(&f);
    f.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_po_line BEFORE INSERT ON purchase_order_lines BEGIN SELECT RAISE(ABORT,'fail'); END;").unwrap();
    let mut v = fields(&f);
    v.notes = Some("must rollback".into());
    assert!(purchasing::update_draft(
        &f.db,
        &UpdatePurchaseDraft {
            id: p.id.clone(),
            expected_revision: 1,
            fields: v
        }
    )
    .is_err());
    let d = purchasing::detail(&f.db, &p.id).unwrap();
    assert_eq!(d.revision, 1);
    assert_eq!(d.notes.as_deref(), Some("Restock"));
    assert_eq!(d.lines.len(), 1)
}
#[test]
fn ordering_requires_lines_freezes_snapshots_and_is_idempotent() {
    let f = f();
    let mut empty_fields = fields(&f);
    empty_fields.lines.clear();
    empty_fields.discount_minor = 0;
    empty_fields.tax_minor = 0;
    let empty = purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: empty_fields,
        },
    )
    .unwrap();
    assert!(purchasing::order(
        &f.db,
        &PurchaseAction {
            id: empty.id,
            expected_revision: 1,
            request_id: id()
        }
    )
    .is_err());
    let p = create(&f);
    let action = PurchaseAction {
        id: p.id.clone(),
        expected_revision: 1,
        request_id: id(),
    };
    let a = purchasing::order(&f.db, &action).unwrap();
    let b = purchasing::order(&f.db, &action).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(a.status, "ORDERED");
    assert_eq!(a.revision, 2);
    f.db.lock()
        .unwrap()
        .execute(
            "UPDATE suppliers SET display_name='Changed later' WHERE id=?1",
            [f.supplier],
        )
        .unwrap();
    assert_eq!(
        purchasing::detail(&f.db, &p.id).unwrap().supplier_name,
        "Nile Medical"
    );
    assert_eq!(movement_count(&f.db), 0)
}
#[test]
fn stale_order_and_ordered_edit_are_rejected() {
    let f = f();
    let p = create(&f);
    assert_eq!(
        purchasing::order(
            &f.db,
            &PurchaseAction {
                id: p.id.clone(),
                expected_revision: 2,
                request_id: id()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    let ordered = purchasing::order(
        &f.db,
        &PurchaseAction {
            id: p.id.clone(),
            expected_revision: 1,
            request_id: id(),
        },
    )
    .unwrap();
    assert_eq!(
        purchasing::update_draft(
            &f.db,
            &UpdatePurchaseDraft {
                id: p.id,
                expected_revision: ordered.revision,
                fields: fields(&f)
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    )
}
#[test]
fn draft_and_ordered_cancellation_require_reason_and_retry_safely() {
    for ordered in [false, true] {
        let f = f();
        let mut p = create(&f);
        if ordered {
            p = purchasing::order(
                &f.db,
                &PurchaseAction {
                    id: p.id,
                    expected_revision: p.revision,
                    request_id: id(),
                },
            )
            .unwrap();
        }
        let action = CancelPurchase {
            id: p.id.clone(),
            expected_revision: p.revision,
            request_id: id(),
            reason: "No longer needed".into(),
        };
        let a = purchasing::cancel(&f.db, &action).unwrap();
        let b = purchasing::cancel(&f.db, &action).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(a.status, "CANCELLED");
        assert!(a.cancelled_at.is_some());
        assert_eq!(movement_count(&f.db), 0);
    }
}
#[test]
fn cancelled_order_is_terminal() {
    let f = f();
    let p = create(&f);
    let cancelled = purchasing::cancel(
        &f.db,
        &CancelPurchase {
            id: p.id.clone(),
            expected_revision: 1,
            request_id: id(),
            reason: "Stop".into(),
        },
    )
    .unwrap();
    assert!(purchasing::order(
        &f.db,
        &PurchaseAction {
            id: p.id.clone(),
            expected_revision: cancelled.revision,
            request_id: id()
        }
    )
    .is_err());
    assert!(purchasing::update_draft(
        &f.db,
        &UpdatePurchaseDraft {
            id: p.id,
            expected_revision: cancelled.revision,
            fields: fields(&f)
        }
    )
    .is_err())
}
#[test]
fn list_search_filter_and_pagination_are_database_backed() {
    let f = f();
    let a = create(&f);
    let mut v = fields(&f);
    v.supplier_id = f.other_supplier.clone();
    v.supplier_reference = Some("SPECIAL".into());
    let b = purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: v,
        },
    )
    .unwrap();
    purchasing::order(
        &f.db,
        &PurchaseAction {
            id: b.id.clone(),
            expected_revision: 1,
            request_id: id(),
        },
    )
    .unwrap();
    assert_eq!(
        purchasing::list(
            &f.db,
            &PurchaseQuery {
                search: Some("special".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .items[0]
            .id,
        b.id
    );
    assert_eq!(
        purchasing::list(
            &f.db,
            &PurchaseQuery {
                supplier_id: Some(f.supplier.clone()),
                ..Default::default()
            }
        )
        .unwrap()
        .items[0]
            .id,
        a.id
    );
    assert_eq!(
        purchasing::list(
            &f.db,
            &PurchaseQuery {
                status: Some("ORDERED".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        1
    );
    let page = purchasing::list(
        &f.db,
        &PurchaseQuery {
            limit: Some(1),
            offset: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((page.total, page.items.len()), (2, 1))
}
#[test]
fn detail_is_complete_and_readable_after_supplier_deactivation() {
    let f = f();
    let p = create(&f);
    let ordered = purchasing::order(
        &f.db,
        &PurchaseAction {
            id: p.id.clone(),
            expected_revision: 1,
            request_id: id(),
        },
    )
    .unwrap();
    let s = partners::get_supplier(&f.db, &f.supplier).unwrap();
    partners::set_supplier_active(
        &f.db,
        &m2_health_lib::partners::models::SetPartnerActive {
            id: s.id,
            expected_revision: s.revision,
            active: false,
        },
    )
    .unwrap();
    let d = purchasing::detail(&f.db, &ordered.id).unwrap();
    assert_eq!(d.supplier_code, "SUP-000001");
    assert_eq!(d.lines[0].ordered_quantity, 3);
    assert_eq!(d.total_minor, 270)
}
#[test]
fn catalog_search_uses_packages_and_never_changes_prices() {
    let f = f();
    let before =
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM product_price_history", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap();
    let result = purchasing::search_packages(
        &f.db,
        &PackageQuery {
            search: Some("Medicine One".into()),
            limit: Some(10),
            offset: Some(0),
        },
    )
    .unwrap();
    assert_eq!(result.items[0].product_package_id, f.package);
    create(&f);
    let after =
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM product_price_history", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap();
    assert_eq!(before, after)
}
#[test]
fn every_phase_5b_operation_leaves_inventory_untouched() {
    let f = f();
    let before = movement_count(&f.db);
    let p = create(&f);
    let mut v = fields(&f);
    v.notes = Some("edited".into());
    let p = purchasing::update_draft(
        &f.db,
        &UpdatePurchaseDraft {
            id: p.id,
            expected_revision: 1,
            fields: v,
        },
    )
    .unwrap();
    let p = purchasing::order(
        &f.db,
        &PurchaseAction {
            id: p.id,
            expected_revision: p.revision,
            request_id: id(),
        },
    )
    .unwrap();
    purchasing::cancel(
        &f.db,
        &CancelPurchase {
            id: p.id,
            expected_revision: p.revision,
            request_id: id(),
            reason: "cancel".into(),
        },
    )
    .unwrap();
    assert_eq!(movement_count(&f.db), before);
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM inventory_batches", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    )
}
