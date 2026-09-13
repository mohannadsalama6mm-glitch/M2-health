use m2_health_lib::db::{
    connection::AppDb,
    models::{
        branch::{Branch, CreateBranch},
        catalog::{CreateProductFull, ProductPackageInput},
        purchases::*,
    },
    repositories::{branches, product_packages, products, purchases},
    services::purchases as purchase_service,
};

fn db() -> AppDb {
    AppDb::in_memory().unwrap()
}

fn branch(db: &AppDb, code: &str, name: &str) -> Branch {
    branches::create_branch(db, &CreateBranch { code: code.into(), name: name.into(), ..Default::default() }).unwrap()
}

/// Creates a product + one package and returns the package id.
/// No price_history needed for purchase tests — cost is set per line.
fn purchasable(db: &AppDb, name: &str, label: &str) -> String {
    let product = products::create_full(
        db,
        &CreateProductFull {
            commercial_name_en: Some(name.into()),
            packages: vec![ProductPackageInput { package_label: label.to_string(), ..Default::default() }],
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

fn balance(db: &AppDb, batch_id: &str) -> i64 {
    db.lock().unwrap()
        .query_row("SELECT COALESCE(quantity,0) FROM stock_balances WHERE batch_id=?1", [batch_id], |r| r.get(0))
        .unwrap()
}

fn complete_purchase(
    db: &AppDb,
    branch_id: &str,
    lines: Vec<(String, i64, i64)>,
    paid: i64,
    discount: i64,
    supplier_id: Option<String>,
    payment_method: &str,
) -> CompletePurchaseResult {
    purchase_service::complete_purchase(
        db,
        &CompletePurchaseInput {
            branch_id: branch_id.into(),
            supplier_id,
            discount_minor: discount,
            paid_minor: paid,
            payment_method: payment_method.into(),
            lines: lines.into_iter().map(|(pkg, qty, cost)| PurchaseLineInput { product_package_id: pkg, quantity: qty, unit_cost_minor: cost }).collect(),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn complete_purchase_posts_ledger_and_snapshots() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk1 = purchasable(&db, "Panadol", "20 tab");
    let pk2 = purchasable(&db, "Augmentin", "10 tab");
    let result = complete_purchase(&db, &b.id, vec![(pk1.clone(), 20, 800), (pk2.clone(), 50, 1600)], 96000, 0, None, "cash");
    let purchase = &result.purchase;
    assert_eq!(purchase.sequence, 1);
    assert_eq!(purchase.purchase_number, "00001");
    assert_eq!(purchase.subtotal_minor, 96000);
    assert_eq!(purchase.total_minor, 96000);
    assert_eq!(purchase.paid_minor, 96000);
    assert_eq!(purchase.change_minor, 0);
    assert_eq!(purchase.payment_method, "cash");
    assert_eq!(purchase.status, "completed");
    assert_eq!(result.movements.len(), 2);
    assert!(result.movements.iter().all(|m| m.movement_type == "purchase"));
    let detail = purchases::detail(&db, &purchase.id).unwrap();
    assert_eq!(detail.branch_name, "Main");
    assert_eq!(detail.supplier_name, None);
    assert_eq!(detail.items.len(), 2);
    let pan = detail.items.iter().find(|i| i.product_package_id == pk1).unwrap();
    assert_eq!(pan.quantity, 20);
    assert_eq!(pan.unit_cost_minor, 800);
    assert_eq!(pan.line_total_minor, 16000);
    assert_eq!(pan.product_name, "Panadol");
    assert_eq!(pan.batches.len(), 1);
    assert_eq!(pan.batches[0].unit_cost_minor, Some(800));
}

#[test]
fn complete_purchase_increases_stock_balance() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let batch_id = {
        let result = complete_purchase(&db, &b.id, vec![(pk.clone(), 10, 500)], 5000, 0, None, "cash");
        let detail = purchases::detail(&db, &result.purchase.id).unwrap();
        detail.items[0].batches[0].batch_id.clone()
    };
    assert_eq!(balance(&db, &batch_id), 10);
}

#[test]
fn purchase_number_increments_per_branch() {
    let db = db();
    let b1 = branch(&db, "A", "Branch A");
    let b2 = branch(&db, "B", "Branch B");
    let pk = purchasable(&db, "Drug", "box");
    let r1 = complete_purchase(&db, &b1.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    let r2 = complete_purchase(&db, &b2.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    let r3 = complete_purchase(&db, &b1.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    assert_eq!(r1.purchase.purchase_number, "00001");
    assert_eq!(r2.purchase.purchase_number, "00001");
    assert_eq!(r3.purchase.purchase_number, "00002");
}

#[test]
fn complete_purchase_rejects_empty_lines_and_low_payment() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let err_empty = purchase_service::complete_purchase(&db, &CompletePurchaseInput {
        branch_id: b.id.clone(), payment_method: "cash".into(), paid_minor: 1000, lines: vec![], ..Default::default()
    });
    assert!(err_empty.is_err());
    assert!(err_empty.unwrap_err().to_string().contains("at least one line"));
    let pk = purchasable(&db, "Drug", "box");
    let err_low = purchase_service::complete_purchase(&db, &CompletePurchaseInput {
        branch_id: b.id.clone(), payment_method: "cash".into(), paid_minor: 100,
        lines: vec![PurchaseLineInput { product_package_id: pk, quantity: 10, unit_cost_minor: 100 }],
        ..Default::default()
    });
    assert!(err_low.is_err());
    assert!(err_low.unwrap_err().to_string().contains("cover"));
}

#[test]
fn complete_purchase_rejects_inactive_product() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let product_id: String = db
        .lock()
        .unwrap()
        .query_row("SELECT product_id FROM product_packages WHERE id=?1", [&pk], |r| r.get(0))
        .unwrap();
    products::set_active(&db, &product_id, false).unwrap();
    let err = purchase_service::complete_purchase(&db, &CompletePurchaseInput {
        branch_id: b.id.clone(), payment_method: "cash".into(), paid_minor: 1000,
        lines: vec![PurchaseLineInput { product_package_id: pk, quantity: 10, unit_cost_minor: 100 }],
        ..Default::default()
    });
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("Inactive"));
}

#[test]
fn discount_and_tax_are_enforced_purchase() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let err_discount = purchase_service::complete_purchase(&db, &CompletePurchaseInput {
        branch_id: b.id.clone(), payment_method: "cash".into(), paid_minor: 0, discount_minor: 5000,
        lines: vec![PurchaseLineInput { product_package_id: pk.clone(), quantity: 1, unit_cost_minor: 1000 }],
        ..Default::default()
    });
    assert!(err_discount.is_err());
    assert!(err_discount.unwrap_err().to_string().contains("discount"));
    let err_method = purchase_service::complete_purchase(&db, &CompletePurchaseInput {
        branch_id: b.id.clone(), payment_method: "bitcoin".into(), paid_minor: 1000,
        lines: vec![PurchaseLineInput { product_package_id: pk, quantity: 1, unit_cost_minor: 1000 }],
        ..Default::default()
    });
    assert!(err_method.is_err());
    assert!(err_method.unwrap_err().to_string().contains("Payment method"));
}

#[test]
fn void_purchase_reverses_stock_and_movements() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let result = complete_purchase(&db, &b.id, vec![(pk.clone(), 10, 500)], 5000, 0, None, "cash");
    let detail = purchases::detail(&db, &result.purchase.id).unwrap();
    let batch_id = detail.items[0].batches[0].batch_id.clone();
    assert_eq!(balance(&db, &batch_id), 10);
    let adj_count: i64 = db.lock().unwrap().query_row(
        "SELECT count(*) FROM stock_movements WHERE reference_id=?1 AND movement_type='adjustment'", [&result.purchase.id], |r| r.get(0),
    ).unwrap();
    assert_eq!(adj_count, 0);
    let v = purchase_service::void_purchase(
        &db,
        &VoidPurchaseInput { purchase_id: result.purchase.id.clone(), reason: "Wrong order".into(), user: Some("admin".into()) },
    )
    .unwrap();
    assert_eq!(v.status, "void");
    assert_eq!(balance(&db, &batch_id), 0);
    let adj_count: i64 = db.lock().unwrap().query_row(
        "SELECT count(*) FROM stock_movements WHERE reference_id=?1 AND movement_type='adjustment'", [&result.purchase.id], |r| r.get(0),
    ).unwrap();
    assert_eq!(adj_count, 1);
}

#[test]
fn void_purchase_rejects_non_completed() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let result = complete_purchase(&db, &b.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    purchase_service::void_purchase(&db, &VoidPurchaseInput {
        purchase_id: result.purchase.id.clone(), reason: "Void".into(), ..Default::default()
    }).unwrap();
    let err = purchase_service::void_purchase(&db, &VoidPurchaseInput {
        purchase_id: result.purchase.id, reason: "Again".into(), ..Default::default()
    });
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("completed"));
}

#[test]
fn void_purchase_rejects_empty_reason() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let result = complete_purchase(&db, &b.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    let err = purchase_service::void_purchase(&db, &VoidPurchaseInput {
        purchase_id: result.purchase.id, reason: "".into(), ..Default::default()
    });
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("reason"));
}

#[test]
fn list_purchases_filters_by_branch() {
    let db = db();
    let b1 = branch(&db, "A", "Branch A");
    let b2 = branch(&db, "B", "Branch B");
    let pk = purchasable(&db, "Drug", "box");
    complete_purchase(&db, &b1.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    complete_purchase(&db, &b2.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    let page_a = purchases::list(&db, &PurchaseQuery { branch_id: Some(b1.id.clone()), ..Default::default() }).unwrap();
    assert_eq!(page_a.total, 1);
    assert_eq!(page_a.items[0].branch_id, b1.id);
    let all = purchases::list(&db, &PurchaseQuery::default()).unwrap();
    assert_eq!(all.total, 2);
}

#[test]
fn list_purchases_void_filter_and_search() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let r1 = complete_purchase(&db, &b.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    let r2 = complete_purchase(&db, &b.id, vec![(pk.clone(), 1, 100)], 100, 0, None, "cash");
    purchase_service::void_purchase(&db, &VoidPurchaseInput {
        purchase_id: r1.purchase.id.clone(), reason: "Cancel".into(), ..Default::default()
    }).unwrap();
    let completed = purchases::list(&db, &PurchaseQuery { status: Some("completed".into()), ..Default::default() }).unwrap();
    assert_eq!(completed.total, 1);
    assert_eq!(completed.items[0].id, r2.purchase.id);
    let voided = purchases::list(&db, &PurchaseQuery { status: Some("void".into()), ..Default::default() }).unwrap();
    assert_eq!(voided.total, 1);
    let by_search = purchases::list(&db, &PurchaseQuery { search: Some(r2.purchase.purchase_number.clone()), ..Default::default() }).unwrap();
    assert_eq!(by_search.total, 1);
}

#[test]
fn purchase_pos_search_returns_cost_price_and_name() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Panadol Forte", "24 tab");
    let all = purchases::pos_search(&db, &PosPurchaseQuery { branch_id: b.id.clone(), ..Default::default() }).unwrap();
    assert!(all.items.iter().any(|p| p.product_name.contains("Panadol")));
    let by_name = purchases::pos_search(&db, &PosPurchaseQuery { branch_id: b.id.clone(), search: Some("Panadol".into()), ..Default::default() }).unwrap();
    assert_eq!(by_name.items.len(), 1);
    assert_eq!(by_name.items[0].package_id, pk);
}

#[test]
fn complete_purchase_with_supplier() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let sup = m2_health_lib::db::repositories::suppliers::create(
        &db,
        &m2_health_lib::db::models::suppliers::CreateSupplierInput { name: "Supplier Co".into(), ..Default::default() },
    ).unwrap();
    let result = complete_purchase(&db, &b.id, vec![(pk.clone(), 10, 500)], 5000, 0, Some(sup.id.clone()), "card");
    assert_eq!(result.purchase.supplier_id.as_deref(), Some(sup.id.as_str()));
    assert_eq!(result.purchase.payment_method, "card");
    let detail = purchases::detail(&db, &result.purchase.id).unwrap();
    assert_eq!(detail.supplier_name.as_deref(), Some("Supplier Co"));
}

#[test]
fn complete_purchase_rejects_invalid_supplier_id() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = purchasable(&db, "Drug", "box");
    let err = purchase_service::complete_purchase(&db, &CompletePurchaseInput {
        branch_id: b.id.clone(),
        supplier_id: Some("00000000-0000-0000-0000-000000000000".into()),
        payment_method: "cash".into(), paid_minor: 1000,
        lines: vec![PurchaseLineInput { product_package_id: pk, quantity: 1, unit_cost_minor: 1000 }],
        ..Default::default()
    });
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("not found"));
}