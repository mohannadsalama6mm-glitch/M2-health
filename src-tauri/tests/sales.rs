use m2_health_lib::db::{
    connection::AppDb,
    models::{
        branch::{Branch, CreateBranch},
        catalog::{AddBarcode, CreateProductFull, ProductPackageInput, SetPackagePrice},
        inventory::*,
        sales::*,
    },
    repositories::{barcodes, branches, price_history, product_packages, products, sales},
    services::{inventory as stock_service, sales as sale_service},
};

fn db() -> AppDb {
    AppDb::in_memory().unwrap()
}
fn branch(db: &AppDb, code: &str, name: &str) -> Branch {
    branches::create_branch(
        db,
        &CreateBranch {
            code: code.into(),
            name: name.into(),
            ..Default::default()
        },
    )
    .unwrap()
}
/// Creates a product + one package, sets its selling price (and optional cost),
/// and returns the package id.
fn sellable(db: &AppDb, name: &str, label: &str, selling: i64, cost: Option<i64>) -> String {
    let product = products::create_full(
        db,
        &CreateProductFull {
            commercial_name_en: Some(name.into()),
            packages: vec![ProductPackageInput {
                package_label: label.to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let pkg = product_packages::list(db, &product.id, false)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .id;
    price_history::set(
        db,
        &SetPackagePrice {
            product_package_id: pkg.clone(),
            selling_price_minor: selling,
            cost_price_minor: cost,
            reason: Some("Initial price".into()),
        },
    )
    .unwrap();
    pkg
}
fn open(db: &AppDb, branch_id: &str, package_id: &str, number: &str, expiry: &str, qty: i64, cost: Option<i64>) -> String {
    stock_service::opening_stock(
        db,
        &OpeningStockInput {
            branch_id: branch_id.into(),
            product_package_id: package_id.into(),
            batch_number: number.into(),
            expiry_date: expiry.into(),
            cost_price_minor: cost,
            quantity: qty,
            reason: "Initial stock".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .batch_id
}
fn balance(db: &AppDb, batch_id: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(
            "SELECT COALESCE(quantity,0) FROM stock_balances WHERE batch_id=?1",
            [batch_id],
            |r| r.get(0),
        )
        .unwrap()
}
fn complete(
    db: &AppDb,
    branch_id: &str,
    lines: Vec<(String, i64)>,
    paid: i64,
    discount: i64,
) -> CompleteSaleResult {
    sale_service::complete_sale(
        db,
        &CompleteSaleInput {
            branch_id: branch_id.into(),
            payment_method: "cash".into(),
            paid_minor: paid,
            discount_minor: discount,
            lines: lines
                .into_iter()
                .map(|(package_id, quantity)| SaleLineInput {
                    product_package_id: package_id,
                    quantity,
                })
                .collect(),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn complete_sale_posts_ledger_and_snapshots() {
    let db = db();
    let b = branch(&db, "MAIN", "Main Pharmacy");
    let pk1 = sellable(&db, "Panadol", "20 tab", 1200, Some(800));
    let pk2 = sellable(&db, "Augmentin", "10 tab", 2500, Some(1600));
    let b1 = open(&db, &b.id, &pk1, "BATCH1", "2027-01-01", 10, Some(800));
    let _b2 = open(&db, &b.id, &pk2, "BATCH2", "2027-02-01", 10, Some(1600));
    let result = complete(&db, &b.id, vec![(pk1.clone(), 3), (pk2.clone(), 2)], 8600, 0);
    let sale = &result.sale;
    assert_eq!(sale.sequence, 1);
    assert_eq!(sale.receipt_number, "00001");
    assert_eq!(sale.total_minor, 8600);
    assert_eq!(sale.subtotal_minor, 8600);
    assert_eq!(sale.change_minor, 0);
    assert_eq!(sale.payment_method, "cash");
    assert_eq!(sale.status, "completed");
    assert_eq!(result.movements.len(), 2);
    assert!(result.movements.iter().all(|m| m.movement_type == "sale"));
    assert_eq!(balance(&db, &b1), 7);
    let detail = sales::detail(&db, &sale.id).unwrap();
    assert_eq!(detail.branch_name, "Main Pharmacy");
    assert_eq!(detail.items.len(), 2);
    let pan = detail.items.iter().find(|i| i.product_package_id == pk1).unwrap();
    assert_eq!(pan.quantity, 3);
    assert_eq!(pan.selling_price_minor, 1200);
    assert_eq!(pan.cost_price_minor, Some(800));
    assert_eq!(pan.line_total_minor, 3600);
    assert_eq!(pan.line_cost_minor, Some(2400));
    assert_eq!(pan.batches.len(), 1);
    assert_eq!(pan.batches[0].batch_number, "BATCH1");
    assert_eq!(pan.returnable_quantity, 3);
}

#[test]
fn receipt_number_increments_per_branch() {
    let db = db();
    let b1 = branch(&db, "A", "Branch A");
    let b2 = branch(&db, "B", "Branch B");
    let pk = sellable(&db, "Drug", "1 pc", 1000, Some(600));
    open(&db, &b1.id, &pk, "A1", "2027-01-01", 100, Some(600));
    open(&db, &b2.id, &pk, "B1", "2027-01-01", 100, Some(600));
    let r1 = complete(&db, &b1.id, vec![(pk.clone(), 1)], 1000, 0);
    let r2 = complete(&db, &b2.id, vec![(pk.clone(), 1)], 1000, 0);
    let r3 = complete(&db, &b1.id, vec![(pk.clone(), 1)], 1000, 0);
    assert_eq!(r1.sale.receipt_number, "00001");
    assert_eq!(r1.sale.branch_id, b1.id);
    assert_eq!(r2.sale.receipt_number, "00001");
    assert_eq!(r2.sale.branch_id, b2.id);
    assert_eq!(r3.sale.receipt_number, "00002");
    assert_eq!(r3.sale.branch_id, b1.id);
}

#[test]
fn fefo_allocates_earliest_expiry_first() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    let early = open(&db, &b.id, &pk, "EARLY", "2026-06-01", 3, Some(500));
    let late = open(&db, &b.id, &pk, "LATE", "2027-06-01", 10, Some(700));
    let result = complete(&db, &b.id, vec![(pk.clone(), 5)], 5000, 0);
    let detail = sales::detail(&db, &result.sale.id).unwrap();
    let item = &detail.items[0];
    let mut batches: Vec<(&str, i64)> = item
        .batches
        .iter()
        .map(|b| (b.batch_number.as_str(), b.quantity))
        .collect();
    batches.sort_by_key(|(_, q)| -q);
    assert_eq!(batches, vec![("EARLY", 3), ("LATE", 2)]);
    assert_eq!(balance(&db, &early), 0);
    assert_eq!(balance(&db, &late), 8);
    assert_eq!(item.cost_price_minor, Some(500));
    assert_eq!(item.line_cost_minor, Some(2500));
}

#[test]
fn complete_sale_rejects_insufficient_stock_and_rolls_back() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    let bid = open(&db, &b.id, &pk, "B1", "2027-01-01", 5, Some(500));
    let err = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            payment_method: "cash".into(),
            lines: vec![SaleLineInput {
                product_package_id: pk.clone(),
                quantity: 6,
            }],
            paid_minor: 6000,
            ..Default::default()
        },
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("Insufficient stock"));
    assert_eq!(balance(&db, &bid), 5);
    assert_eq!(
        db.lock()
            .unwrap()
            .query_row("SELECT count(*) FROM sales", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn complete_sale_rejects_empty_cart_and_missing_price() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk_no_price = {
        let product = products::create_full(
            &db,
            &CreateProductFull {
                commercial_name_en: Some("NoPrice".into()),
                packages: vec![ProductPackageInput {
                    package_label: "none".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();
        product_packages::list(&db, &product.id, false)
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .id
    };
    let err_empty = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            payment_method: "cash".into(),
            paid_minor: 0,
            lines: vec![],
            ..Default::default()
        },
    );
    assert!(err_empty.is_err());
    let err_price = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            payment_method: "cash".into(),
            lines: vec![SaleLineInput {
                product_package_id: pk_no_price,
                quantity: 1,
            }],
            paid_minor: 1000,
            ..Default::default()
        },
    );
    assert!(err_price.is_err());
    assert!(err_price
        .unwrap_err()
        .to_string()
        .contains("no selling price"));
}

#[test]
fn return_sale_restocks_original_lot_and_caps_quantity() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    let batch = open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(500));
    let result = complete(&db, &b.id, vec![(pk.clone(), 4)], 4000, 0);
    assert_eq!(balance(&db, &batch), 6);
    let item = sales::detail(&db, &result.sale.id).unwrap().items[0].clone();
    assert_eq!(item.returnable_quantity, 4);
    let sale_item_id = item.id.clone();
    let ret = sale_service::return_sale(
        &db,
        &ReturnSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Changed mind".into(),
            items: vec![ReturnItemInput {
                sale_item_id: sale_item_id.clone(),
                quantity: 2,
                refund_minor: 1000,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(ret.total_refund_minor, 2000);
    assert_eq!(ret.items.len(), 1);
    assert_eq!(ret.items[0].quantity, 2);
    assert_eq!(ret.items[0].product_name, "Drug");
    assert_eq!(balance(&db, &batch), 8);
    let detail2 = sales::detail(&db, &result.sale.id).unwrap();
    let pan = detail2.items.iter().find(|i| i.product_package_id == pk).unwrap();
    assert_eq!(pan.returned_quantity, 2);
    assert_eq!(pan.returnable_quantity, 2);
    assert_eq!(detail2.returns.len(), 1);
    assert_eq!(detail2.returns[0].total_refund_minor, 2000);
    let err_over = sale_service::return_sale(
        &db,
        &ReturnSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "More than sold".into(),
            items: vec![ReturnItemInput {
                sale_item_id: sale_item_id.clone(),
                quantity: 3,
                refund_minor: 1000,
            }],
            ..Default::default()
        },
    );
    assert!(err_over.is_err());
    assert!(err_over.unwrap_err().to_string().contains("exceeds"));
    let err_price = sale_service::return_sale(
        &db,
        &ReturnSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Refund too high".into(),
            items: vec![ReturnItemInput {
                sale_item_id: sale_item_id.clone(),
                quantity: 1,
                refund_minor: 2000,
            }],
            ..Default::default()
        },
    );
    assert!(err_price.is_err());
    assert!(err_price.unwrap_err().to_string().contains("refund"));
}

#[test]
fn return_sale_uses_general_lot_when_original_inactive() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    let lot1 = open(&db, &b.id, &pk, "LOT1", "2027-01-01", 5, Some(500));
    let result = complete(&db, &b.id, vec![(pk.clone(), 3)], 3000, 0);
    assert_eq!(balance(&db, &lot1), 2);
    db.lock()
        .unwrap()
        .execute("UPDATE inventory_batches SET is_active=0 WHERE id=?1", [&lot1])
        .unwrap();
    let sale_item_id = sales::detail(&db, &result.sale.id)
        .unwrap()
        .items[0]
        .id
        .clone();
    sale_service::return_sale(
        &db,
        &ReturnSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Original lot inactive".into(),
            items: vec![ReturnItemInput {
                sale_item_id,
                quantity: 2,
                refund_minor: 1000,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let general_batch: String = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT id FROM inventory_batches WHERE branch_id=?1 AND product_package_id=?2 AND batch_number='GENERAL'",
            [&b.id, &pk],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(balance(&db, &general_batch), 2);
    assert_eq!(balance(&db, &lot1), 2);
}

#[test]
fn void_sale_reverses_exact_batches() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    let lot1 = open(&db, &b.id, &pk, "A", "2026-12-01", 2, Some(400));
    let lot2 = open(&db, &b.id, &pk, "B", "2027-12-01", 8, Some(600));
let result = complete(&db, &b.id, vec![(pk.clone(), 3)], 3000, 0);
    assert_eq!(balance(&db, &lot1), 0);
    assert_eq!(balance(&db, &lot2), 7);
    let sale_movements: i64 = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM stock_movements WHERE reference_id=?1 AND movement_type='sale'",
            [&result.sale.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(sale_movements, 2);
    let v = sale_service::void_sale(
        &db,
        &VoidSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Customer cancelled".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(v.status, "void");
    assert_eq!(balance(&db, &lot1), 2);
    assert_eq!(balance(&db, &lot2), 8);
    let adjustments: i64 = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM stock_movements WHERE reference_id=?1 AND movement_type='adjustment'",
            [&result.sale.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(adjustments, 2);
    let detail = sales::detail(&db, &result.sale.id).unwrap();
    assert_eq!(detail.sale.status, "void");
    assert_eq!(detail.sale.void_reason, "Customer cancelled");
}

#[test]
fn void_sale_blocked_when_returns_exist() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(500));
    let result = complete(&db, &b.id, vec![(pk.clone(), 2)], 2000, 0);
    let sale_item_id = sales::detail(&db, &result.sale.id)
        .unwrap()
        .items[0]
        .id
        .clone();
    sale_service::return_sale(
        &db,
        &ReturnSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Return first".into(),
            items: vec![ReturnItemInput {
                sale_item_id,
                quantity: 1,
                refund_minor: 1000,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let err = sale_service::void_sale(
        &db,
        &VoidSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Now void".into(),
            ..Default::default()
        },
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("cannot be voided"));
}

#[test]
fn discount_and_tax_are_enforced() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(500));
    let too_high = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            payment_method: "cash".into(),
            lines: vec![SaleLineInput {
                product_package_id: pk.clone(),
                quantity: 1,
            }],
            paid_minor: 0,
            discount_minor: 5000,
            ..Default::default()
        },
    );
    assert!(too_high.is_err());
    assert!(too_high.unwrap_err().to_string().contains("discount"));
    let too_little = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            payment_method: "cash".into(),
            lines: vec![SaleLineInput {
                product_package_id: pk.clone(),
                quantity: 1,
            }],
            paid_minor: 500,
            ..Default::default()
        },
    );
    assert!(too_little.is_err());
    assert!(too_little.unwrap_err().to_string().contains("cover"));
}

#[test]
fn pos_search_matches_name_barcode_batch_and_flags_status() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Panadol Forte", "24 tab", 1000, Some(0));
    sellable(&db, "Omeprazole", "14 cap", 2000, Some(0));
    barcodes::add(
        &db,
        &AddBarcode {
            product_package_id: pk.clone(),
            barcode: "6281100123456".into(),
            is_primary: true,
        },
    )
    .unwrap();
    open(&db, &b.id, &pk, "B1", "2027-01-01", 5, Some(0));
    sellable(&db, "Empty Drug", "1 tab", 500, Some(0));
    let all = sales::pos_search(
        &db,
        &PosQuery {
            branch_id: b.id.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(all.items.iter().any(|p| p.product_name.contains("Panadol")));
    assert!(all.items.iter().any(|p| p.product_name.contains("Omeprazole")));
    let by_name = sales::pos_search(
        &db,
        &PosQuery {
            branch_id: b.id.clone(),
            search: Some("Panadol".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(by_name.items.len(), 1);
    assert_eq!(by_name.items[0].quantity, 5);
    assert_eq!(by_name.items[0].status, "in_stock");
    let by_barcode = sales::pos_search(
        &db,
        &PosQuery {
            branch_id: b.id.clone(),
            search: Some("6281100123456".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(by_barcode.items.len(), 1);
    assert_eq!(by_barcode.items[0].package_id, pk);
    let by_batch = sales::pos_search(
        &db,
        &PosQuery {
            branch_id: b.id.clone(),
            search: Some("B1".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(by_batch.items.len(), 1);
    assert_eq!(by_batch.items[0].quantity, 5);
    let by_name = sales::pos_search(
        &db,
        &PosQuery {
            branch_id: b.id.clone(),
            search: Some("Empty Drug".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(by_name.items.len(), 1);
    assert_eq!(by_name.items[0].status, "out_of_stock");
}

#[test]
fn detail_retains_catalog_snapshots_after_price_change() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Panadol", "20 tab", 1000, Some(400));
    open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(400));
    let result = complete(&db, &b.id, vec![(pk.clone(), 1)], 1000, 0);
    price_history::set(
        &db,
        &SetPackagePrice {
            product_package_id: pk.clone(),
            selling_price_minor: 1500,
            cost_price_minor: Some(600),
            reason: Some("Price hike".into()),
        },
    )
    .unwrap();
    let detail = sales::detail(&db, &result.sale.id).unwrap();
    assert_eq!(detail.items[0].selling_price_minor, 1000);
    assert_eq!(detail.items[0].cost_price_minor, Some(400));
    assert_eq!(detail.sale.subtotal_minor, 1000);
    assert_eq!(detail.sale.total_minor, 1000);
}

#[test]
fn return_sale_rejects_non_completed_sale() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(500));
    let result = complete(&db, &b.id, vec![(pk.clone(), 1)], 1000, 0);
    sale_service::void_sale(
        &db,
        &VoidSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Void first".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let sale_item_id = sales::detail(&db, &result.sale.id)
        .unwrap()
        .items[0]
        .id
        .clone();
    let err = sale_service::return_sale(
        &db,
        &ReturnSaleInput {
            sale_id: result.sale.id.clone(),
            reason: "Return voided".into(),
            items: vec![ReturnItemInput {
                sale_item_id,
                quantity: 1,
                refund_minor: 1000,
            }],
            ..Default::default()
        },
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("completed"));
}

#[test]
fn sale_can_be_linked_to_a_customer() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let cust = m2_health_lib::db::repositories::customers::create(
        &db,
        &m2_health_lib::db::models::customers::CreateCustomerInput {
            name: "Reem Adel".into(),
            phone: "01000000000".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(500));
    let result = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            customer_id: Some(cust.id.clone()),
            payment_method: "cash".into(),
            paid_minor: 1000,
            lines: vec![SaleLineInput {
                product_package_id: pk,
                quantity: 1,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(result.sale.customer_id.as_deref(), Some(cust.id.as_str()));
    assert_eq!(result.sale.customer_name, "Reem Adel");
    assert_eq!(result.sale.customer_phone, "01000000000");
}

#[test]
fn sale_rejects_unknown_customer_id() {
    let db = db();
    let b = branch(&db, "MAIN", "Main");
    let pk = sellable(&db, "Drug", "box", 1000, Some(0));
    open(&db, &b.id, &pk, "B1", "2027-01-01", 10, Some(500));
    let err = sale_service::complete_sale(
        &db,
        &CompleteSaleInput {
            branch_id: b.id.clone(),
            customer_id: Some("00000000-0000-0000-0000-000000000000".into()),
            payment_method: "cash".into(),
            paid_minor: 1000,
            lines: vec![SaleLineInput {
                product_package_id: pk,
                quantity: 1,
            }],
            ..Default::default()
        },
    );
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("not found"));
}

#[test]
fn list_sales_filters_by_branch() {
    let db = db();
    let b1 = branch(&db, "A", "Branch A");
    let b2 = branch(&db, "B", "Branch B");
    let pk = sellable(&db, "Drug", "1 pc", 1000, Some(600));
    open(&db, &b1.id, &pk, "A1", "2027-01-01", 100, Some(600));
    open(&db, &b2.id, &pk, "B1", "2027-01-01", 100, Some(600));
    complete(&db, &b1.id, vec![(pk.clone(), 1)], 1000, 0);
    complete(&db, &b2.id, vec![(pk.clone(), 1)], 1000, 0);
    let page_a = sales::list(
        &db,
        &SaleQuery {
            branch_id: Some(b1.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page_a.total, 1);
    assert_eq!(page_a.items[0].branch_id, b1.id);
    let page_b = sales::list(
        &db,
        &SaleQuery {
            branch_id: Some(b2.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page_b.total, 1);
    assert_eq!(page_b.items[0].branch_id, b2.id);
    let all = sales::list(&db, &SaleQuery::default()).unwrap();
    assert_eq!(all.total, 2);
}

