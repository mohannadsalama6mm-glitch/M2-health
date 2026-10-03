use m2_health_lib::{
    db::{
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        models::branch::CreateBranch,
        repositories::branches,
    },
    error::ErrorCode,
    partners::{self, models::*},
};

fn supplier(name: &str) -> SupplierFields {
    SupplierFields {
        display_name: name.into(),
        contact_person: Some("  Dina Ali  ".into()),
        primary_phone: Some("01000000001".into()),
        secondary_phone: Some("  ".into()),
        email: Some("sales@example.com".into()),
        address: Some("Cairo".into()),
        tax_registration_id: Some("TAX-1".into()),
        notes: Some("Preferred".into()),
    }
}

fn customer(name: &str) -> CustomerFields {
    CustomerFields {
        display_name: name.into(),
        primary_phone: Some("01100000001".into()),
        secondary_phone: Some("01200000002".into()),
        email: Some("buyer@example.com".into()),
        address: Some("Giza".into()),
        notes: Some("Retail".into()),
    }
}

fn all() -> PartnerQuery {
    PartnerQuery {
        limit: Some(100),
        ..Default::default()
    }
}

fn scalar(db: &AppDb, sql: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

#[test]
fn migration_010_creates_only_partner_foundation() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..10]).unwrap();
    assert_eq!(scalar(&db, "SELECT max(version) FROM _schema_version"), 10);
    assert_eq!(scalar(&db, "SELECT count(*) FROM suppliers"), 0);
    assert_eq!(scalar(&db, "SELECT count(*) FROM customers"), 0);
    assert_eq!(scalar(&db, "SELECT count(*) FROM sqlite_master WHERE name IN ('purchases','purchase_items','supplier_payments')"), 0);
}

#[test]
fn upgrade_009_to_010_applies_once_and_preserves_phase_four_data() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..9]).unwrap();
    let branch = branches::create_branch(
        &db,
        &CreateBranch {
            code: "KEEP".into(),
            name: "Keep Pharmacy".into(),
            phone: "".into(),
            address: "".into(),
        },
    )
    .unwrap();
    let mut connection = db.lock().unwrap();
    migrations::run(&mut connection).unwrap();
    migrations::run(&mut connection).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM _schema_version WHERE version=10",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT name FROM branches WHERE id=?1", [&branch.id], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "Keep Pharmacy"
    );
}

#[test]
fn migration_010_failure_rolls_back_every_partial_change() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..9]).unwrap();
    let mut connection = db.lock().unwrap();
    connection
        .execute_batch("CREATE TABLE customers(conflict TEXT)")
        .unwrap();
    assert!(migrations::run(&mut connection).is_err());
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='partner_sequences'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='suppliers'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row("SELECT max(version) FROM _schema_version", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        9
    );
}

#[test]
fn supplier_create_reads_all_fields_and_normalizes_optionals() {
    let db = AppDb::in_memory().unwrap();
    let created = partners::create_supplier(&db, &supplier("  Nile Medical  ")).unwrap();
    assert_eq!(created.code, "SUP-000001");
    assert_eq!(created.display_name, "Nile Medical");
    assert_eq!(created.contact_person.as_deref(), Some("Dina Ali"));
    assert_eq!(created.secondary_phone, None);
    assert_eq!(created.revision, 1);
    assert!(created.is_active);
    assert_eq!(
        partners::get_supplier(&db, &created.id)
            .unwrap()
            .tax_registration_id
            .as_deref(),
        Some("TAX-1")
    );
}

#[test]
fn customer_create_reads_all_fields_and_uses_independent_sequence() {
    let db = AppDb::in_memory().unwrap();
    partners::create_supplier(&db, &supplier("Supplier")).unwrap();
    let created = partners::create_customer(&db, &customer("  Ahmed Hassan ")).unwrap();
    assert_eq!(created.code, "CUS-000001");
    assert_eq!(created.display_name, "Ahmed Hassan");
    assert_eq!(
        partners::get_customer(&db, &created.id)
            .unwrap()
            .primary_phone
            .as_deref(),
        Some("01100000001")
    );
}

#[test]
fn codes_are_monotonic_and_failed_unique_insert_does_not_consume_one() {
    let db = AppDb::in_memory().unwrap();
    let one = partners::create_supplier(&db, &supplier("One")).unwrap();
    let mut duplicate = supplier("Duplicate");
    duplicate.tax_registration_id = Some("tax-1".into());
    assert_eq!(
        partners::create_supplier(&db, &duplicate).unwrap_err().code,
        ErrorCode::Conflict
    );
    let mut next = supplier("Two");
    next.tax_registration_id = Some("TAX-2".into());
    let two = partners::create_supplier(&db, &next).unwrap();
    assert_eq!(
        (one.code.as_str(), two.code.as_str()),
        ("SUP-000001", "SUP-000002")
    );
}

#[test]
fn concurrent_creates_generate_unique_sequential_codes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("partners.db");
    let one = AppDb::open(&path).unwrap();
    let two = AppDb::open(&path).unwrap();
    let a = std::thread::spawn(move || {
        partners::create_customer(&one, &customer("Alpha"))
            .unwrap()
            .code
    });
    let b = std::thread::spawn(move || {
        partners::create_customer(&two, &customer("Beta"))
            .unwrap()
            .code
    });
    let mut codes = vec![a.join().unwrap(), b.join().unwrap()];
    codes.sort();
    assert_eq!(codes, ["CUS-000001", "CUS-000002"]);
}

#[test]
fn validation_rejects_bad_names_emails_ids_and_pagination() {
    let db = AppDb::in_memory().unwrap();
    assert_eq!(
        partners::create_supplier(&db, &supplier(" "))
            .unwrap_err()
            .code,
        ErrorCode::Validation
    );
    let mut bad = customer("Valid");
    bad.email = Some("broken@address".into());
    assert_eq!(
        partners::create_customer(&db, &bad).unwrap_err().code,
        ErrorCode::Validation
    );
    assert_eq!(
        partners::get_customer(&db, "bad-id").unwrap_err().code,
        ErrorCode::Validation
    );
    assert_eq!(
        partners::list_customers(
            &db,
            &PartnerQuery {
                limit: Some(101),
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
}

#[test]
fn missing_partner_is_a_structured_not_found() {
    let db = AppDb::in_memory().unwrap();
    assert_eq!(
        partners::get_supplier(&db, &uuid::Uuid::new_v4().to_string())
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

#[test]
fn supplier_update_uses_revision_and_rejects_stale_writes() {
    let db = AppDb::in_memory().unwrap();
    let created = partners::create_supplier(&db, &supplier("Before")).unwrap();
    let mut fields = supplier("After");
    fields.tax_registration_id = Some("TAX-2".into());
    let updated = partners::update_supplier(
        &db,
        &UpdateSupplier {
            id: created.id.clone(),
            expected_revision: 1,
            fields: fields.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        (updated.display_name.as_str(), updated.revision),
        ("After", 2)
    );
    assert_eq!(
        partners::update_supplier(
            &db,
            &UpdateSupplier {
                id: created.id,
                expected_revision: 1,
                fields
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
}

#[test]
fn customer_update_uses_revision_and_rejects_stale_writes() {
    let db = AppDb::in_memory().unwrap();
    let created = partners::create_customer(&db, &customer("Before")).unwrap();
    let updated = partners::update_customer(
        &db,
        &UpdateCustomer {
            id: created.id.clone(),
            expected_revision: 1,
            fields: customer("After"),
        },
    )
    .unwrap();
    assert_eq!(
        (updated.display_name.as_str(), updated.revision),
        ("After", 2)
    );
    assert_eq!(
        partners::update_customer(
            &db,
            &UpdateCustomer {
                id: created.id,
                expected_revision: 1,
                fields: customer("Stale")
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
}

#[test]
fn status_changes_increment_revision_and_reject_repeats() {
    let db = AppDb::in_memory().unwrap();
    let supplier = partners::create_supplier(&db, &supplier("Status supplier")).unwrap();
    let inactive = partners::set_supplier_active(
        &db,
        &SetPartnerActive {
            id: supplier.id.clone(),
            expected_revision: 1,
            active: false,
        },
    )
    .unwrap();
    assert_eq!((inactive.is_active, inactive.revision), (false, 2));
    let active = partners::set_supplier_active(
        &db,
        &SetPartnerActive {
            id: supplier.id.clone(),
            expected_revision: 2,
            active: true,
        },
    )
    .unwrap();
    assert_eq!((active.is_active, active.revision), (true, 3));
    assert_eq!(
        partners::set_supplier_active(
            &db,
            &SetPartnerActive {
                id: supplier.id,
                expected_revision: 3,
                active: true
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
    let customer = partners::create_customer(&db, &customer("Status customer")).unwrap();
    let inactive_customer = partners::set_customer_active(
        &db,
        &SetPartnerActive {
            id: customer.id.clone(),
            expected_revision: 1,
            active: false,
        },
    )
    .unwrap();
    assert!(!inactive_customer.is_active);
    assert!(
        partners::set_customer_active(
            &db,
            &SetPartnerActive {
                id: customer.id,
                expected_revision: 2,
                active: true
            }
        )
        .unwrap()
        .is_active
    );
}

#[test]
fn supplier_search_covers_code_name_contact_phones_email_and_tax() {
    let db = AppDb::in_memory().unwrap();
    let value = partners::create_supplier(&db, &supplier("Nile Medical")).unwrap();
    for term in [
        value.code.as_str(),
        "nile",
        "dina",
        "010000",
        "sales@",
        "tax-1",
    ] {
        assert_eq!(
            partners::list_suppliers(
                &db,
                &PartnerQuery {
                    search: Some(term.into()),
                    ..all()
                }
            )
            .unwrap()
            .total,
            1,
            "{term}"
        );
    }
}

#[test]
fn customer_search_covers_code_name_both_phones_and_email() {
    let db = AppDb::in_memory().unwrap();
    let value = partners::create_customer(&db, &customer("Ahmed Hassan")).unwrap();
    for term in [value.code.as_str(), "ahmed", "011000", "012000", "buyer@"] {
        assert_eq!(
            partners::list_customers(
                &db,
                &PartnerQuery {
                    search: Some(term.into()),
                    ..all()
                }
            )
            .unwrap()
            .total,
            1,
            "{term}"
        );
    }
}

#[test]
fn list_filter_pagination_and_order_are_backend_driven() {
    let db = AppDb::in_memory().unwrap();
    let z = partners::create_customer(&db, &customer("Zulu")).unwrap();
    let a = partners::create_customer(&db, &customer("alpha")).unwrap();
    partners::set_customer_active(
        &db,
        &SetPartnerActive {
            id: z.id,
            expected_revision: 1,
            active: false,
        },
    )
    .unwrap();
    let active = partners::list_customers(
        &db,
        &PartnerQuery {
            is_active: Some(true),
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (active.total, active.items[0].id.as_str()),
        (1, a.id.as_str())
    );
    let second = partners::list_customers(
        &db,
        &PartnerQuery {
            limit: Some(1),
            offset: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (second.total, second.items[0].display_name.as_str()),
        (2, "Zulu")
    );
}

#[test]
fn partner_identity_rows_are_retained() {
    let db = AppDb::in_memory().unwrap();
    let supplier = partners::create_supplier(&db, &supplier("Retained")).unwrap();
    let customer = partners::create_customer(&db, &customer("Retained")).unwrap();
    let connection = db.lock().unwrap();
    assert!(connection
        .execute("DELETE FROM suppliers WHERE id=?1", [&supplier.id])
        .is_err());
    assert!(connection
        .execute("DELETE FROM customers WHERE id=?1", [&customer.id])
        .is_err());
}

#[test]
fn phase_four_sales_schema_and_snapshots_are_unchanged() {
    let db = AppDb::in_memory().unwrap();
    let connection = db.lock().unwrap();
    let sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='sales'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(sql.contains("customer_name TEXT"));
    assert!(sql.contains("customer_id TEXT CHECK(customer_id IS NULL)"));
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM pragma_table_info('sales') WHERE name='customer_revision'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}
