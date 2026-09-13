use m2_health_lib::db::{
    connection::AppDb,
    models::customers::{CreateCustomerInput, CustomerQuery, CustomerPage, UpdateCustomerInput},
    models::suppliers::{CreateSupplierInput, SupplierQuery, UpdateSupplierInput},
    repositories::{customers, suppliers},
};

fn db() -> AppDb {
    AppDb::in_memory().unwrap()
}

#[test]
fn customer_create_generates_code_and_lookups() {
    let db = db();
    let c = customers::create(
        &db,
        &CreateCustomerInput {
            name: "Ahmed Hassan".into(),
            phone: "01012345678".into(),
            email: "ahmed@example.com".into(),
            address: "Cairo".into(),
            notes: "VIP".into(),
        },
    )
    .unwrap();
    assert_eq!(c.name, "Ahmed Hassan");
    assert!(c.code.starts_with("AHM-"));
    assert_eq!(c.code.chars().count(), 8);
    assert!(c.is_active);
    let fetched = customers::get_customer(&db, &c.id).unwrap();
    assert_eq!(fetched.code, c.code);
    assert_eq!(fetched.phone, "01012345678");
}

#[test]
fn customer_validation_rejects_empty_name_and_long_fields() {
    let db = db();
    let err_name = customers::create(&db, &CreateCustomerInput { name: "   ".into(), ..Default::default() });
    assert!(err_name.is_err());
    assert!(err_name.unwrap_err().to_string().contains("name"));
    let err_long = customers::create(
        &db,
        &CreateCustomerInput {
            name: "Ok Name".into(),
            phone: "x".repeat(41),
            ..Default::default()
        },
    );
    assert!(err_long.is_err());
}

#[test]
fn customer_codes_are_unique_for_same_name() {
    let db = db();
    let a = customers::create(&db, &CreateCustomerInput { name: "Sara Ali".into(), ..Default::default() }).unwrap();
    let b = customers::create(&db, &CreateCustomerInput { name: "Sara Ali".into(), ..Default::default() }).unwrap();
    assert_ne!(a.code, b.code);
    assert!(a.code.starts_with("SAR-"));
    assert!(b.code.starts_with("SAR-"));
}

#[test]
fn customer_non_ascii_names_fall_back_to_prefix() {
    let db = db();
    let c = customers::create(&db, &CreateCustomerInput { name: "محمد أحمد".into(), ..Default::default() }).unwrap();
    assert!(c.code.starts_with("CUS-"));
}

#[test]
fn customer_list_search_and_active_filter() {
    let db = db();
    let c1 = customers::create(&db, &CreateCustomerInput { name: "Omar Farouk".into(), phone: "0111".into(), ..Default::default() }).unwrap();
    let c2 = customers::create(&db, &CreateCustomerInput { name: "Nouran Saad".into(), email: "n@e.com".into(), ..Default::default() }).unwrap();
    customers::set_active(&db, &c2.id, false).unwrap();
    let all = customers::list(&db, &CustomerQuery::default()).unwrap();
    assert_eq!(all.total, 2);
    let by_search = customers::list(&db, &CustomerQuery { search: Some("Omar".into()), ..Default::default() }).unwrap();
    assert_eq!(by_search.items.len(), 1);
    assert_eq!(by_search.items[0].id, c1.id);
    let by_email_search = customers::list(&db, &CustomerQuery { search: Some("n@e".into()), ..Default::default() }).unwrap();
    assert_eq!(by_email_search.items.len(), 1);
    assert_eq!(by_email_search.items[0].id, c2.id);
    let active_only = customers::list(&db, &CustomerQuery { is_active: Some(true), ..Default::default() }).unwrap();
    assert_eq!(active_only.total, 1);
    assert_eq!(active_only.items[0].id, c1.id);
    let inactive_only = customers::list(&db, &CustomerQuery { is_active: Some(false), ..Default::default() }).unwrap();
    assert_eq!(inactive_only.items.len(), 1);
    assert_eq!(inactive_only.items[0].id, c2.id);
}

#[test]
fn customer_update_preserves_code_and_pages() {
    let db = db();
    let c = customers::create(&db, &CreateCustomerInput { name: "Laila".into(), ..Default::default() }).unwrap();
    let updated = customers::update(
        &db,
        &UpdateCustomerInput { id: c.id.clone(), name: "Laila M".into(), phone: "0122".into(), ..Default::default() },
    )
    .unwrap();
    assert_eq!(updated.id, c.id);
    assert_eq!(updated.code, c.code);
    assert_eq!(updated.name, "Laila M");
    assert_eq!(updated.phone, "0122");
    let page: CustomerPage = customers::list(&db, &CustomerQuery { search: Some("Laila M".into()), ..Default::default() }).unwrap();
    assert_eq!(page.total, 1);
}

#[test]
fn customer_deactivation_and_reactivation() {
    let db = db();
    let c = customers::create(&db, &CreateCustomerInput { name: "Tamer".into(), ..Default::default() }).unwrap();
    let off = customers::set_active(&db, &c.id, false).unwrap();
    assert!(!off.is_active);
    let on = customers::set_active(&db, &c.id, true).unwrap();
    assert!(on.is_active);
}

#[test]
fn supplier_create_generates_code_and_lookups() {
    let db = db();
    let s = suppliers::create(
        &db,
        &CreateSupplierInput {
            name: "Novartis Egypt".into(),
            phone: "02 23456789".into(),
            address: "6th October".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(s.code.starts_with("NOV-"));
    assert!(s.is_active);
    let fetched = suppliers::get_supplier(&db, &s.id).unwrap();
    assert_eq!(fetched.name, "Novartis Egypt");
    assert_eq!(fetched.email, "");
}

#[test]
fn supplier_validation_and_code_fallback() {
    let db = db();
    let err = suppliers::create(&db, &CreateSupplierInput { name: "".into(), ..Default::default() });
    assert!(err.is_err());
    let arabic = suppliers::create(&db, &CreateSupplierInput { name: "شركة أدوية".into(), ..Default::default() }).unwrap();
    assert!(arabic.code.starts_with("SUP-"));
}

#[test]
fn supplier_search_update_and_active_filter() {
    let db = db();
    let s1 = suppliers::create(&db, &CreateSupplierInput { name: "Pharco".into(), ..Default::default() }).unwrap();
    let s2 = suppliers::create(&db, &CreateSupplierInput { name: "EIPICO".into(), ..Default::default() }).unwrap();
    suppliers::set_active(&db, &s2.id, false).unwrap();
    let by_name = suppliers::list(&db, &SupplierQuery { search: Some("Pharco".into()), ..Default::default() }).unwrap();
    assert_eq!(by_name.total, 1);
    let active = suppliers::list(&db, &SupplierQuery { is_active: Some(true), ..Default::default() }).unwrap();
    assert_eq!(active.total, 1);
    let updated = suppliers::update(
        &db,
        &UpdateSupplierInput { id: s1.id.clone(), name: "Pharco Group".into(), ..Default::default() },
    )
    .unwrap();
    assert_eq!(updated.code, s1.code);
    assert_eq!(updated.name, "Pharco Group");
}

#[test]
fn partners_namespaces_are_independent() {
    let db = db();
    let c = customers::create(&db, &CreateCustomerInput { name: "Alpha Co".into(), ..Default::default() }).unwrap();
    let s = suppliers::create(&db, &CreateSupplierInput { name: "Alpha Co".into(), ..Default::default() }).unwrap();
    assert!(c.code.starts_with("ALP-"));
    assert!(s.code.starts_with("ALP-"));
    assert_ne!(c.code, s.code);
}