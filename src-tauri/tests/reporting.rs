use m2_health_lib::{
    db::{
        connection::AppDb,
        models::catalog::{CreateProduct, CreateProductPackage, SetPackagePrice},
        repositories::{branches, price_history, product_packages, products},
    },
    partners::{
        self,
        models::{CustomerFields, SupplierFields},
    },
    purchasing::{self, models::*, receiving, returns},
    reporting::{self, models::ReportFilter},
    sales::{self, models as sm},
};
fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}
struct F {
    db: AppDb,
    branch: String,
    supplier: String,
    customer: String,
    product: String,
    package: String,
}
fn fixture() -> F {
    let db = AppDb::in_memory().unwrap();
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let supplier = partners::create_supplier(
        &db,
        &SupplierFields {
            display_name: "Nile Medical".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let customer = partners::create_customer(
        &db,
        &CustomerFields {
            display_name: "Ahmed Hassan".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Report medicine".into()),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let package = product_packages::create(
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
            product_package_id: package.clone(),
            selling_price_minor: 200,
            cost_price_minor: Some(100),
            reason: Some("Reporting fixture".into()),
        },
    )
    .unwrap();
    F {
        db,
        branch,
        supplier,
        customer,
        product,
        package,
    }
}
fn seed(f: &F) {
    let draft = purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: uid(),
            request_id: uid(),
            fields: PurchaseDraftFields {
                supplier_id: f.supplier.clone(),
                supplier_reference: None,
                expected_date: None,
                notes: None,
                discount_minor: 0,
                tax_minor: 0,
                lines: vec![PurchaseLineInput {
                    product_package_id: f.package.clone(),
                    ordered_quantity: 10,
                    unit_cost_minor: 100,
                }],
            },
        },
    )
    .unwrap();
    let order = purchasing::order(
        &f.db,
        &PurchaseAction {
            id: draft.id,
            expected_revision: draft.revision,
            request_id: uid(),
        },
    )
    .unwrap();
    let receipt = receiving::receive(
        &f.db,
        &ReceivePurchase {
            id: uid(),
            purchase_order_id: order.id.clone(),
            expected_revision: order.revision,
            request_id: uid(),
            branch_id: f.branch.clone(),
            created_by: Some("report-test".into()),
            lines: vec![PurchaseReceiptLineInput {
                purchase_order_line_id: order.lines[0].id.clone(),
                quantity: 10,
                lot_number: Some("LOT-R".into()),
                expiry_date: Some("2030-01-01".into()),
                acquisition_cost_minor: None,
            }],
        },
    )
    .unwrap();
    let complete = purchasing::detail(&f.db, &order.id).unwrap();
    returns::create(
        &f.db,
        &CreatePurchaseReturn {
            id: uid(),
            purchase_order_id: order.id,
            receipt_id: receipt.id.clone(),
            expected_revision: complete.revision,
            request_id: uid(),
            reason: "One damaged".into(),
            created_by: Some("report-test".into()),
            lines: vec![PurchaseReturnLineInput {
                receipt_line_id: receipt.lines[0].id.clone(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    let draft = sales::save(
        &f.db,
        &sm::Save {
            id: uid(),
            expected_revision: None,
            cart: sm::Cart {
                branch_id: f.branch.clone(),
                customer_id: Some(f.customer.clone()),
                lines: vec![sm::Line {
                    product_package_id: f.package.clone(),
                    quantity: 2,
                    ..Default::default()
                }],
                ..Default::default()
            },
        },
    )
    .unwrap();
    let posted = sales::post(
        &f.db,
        &sm::Post {
            id: draft["id"].as_str().unwrap().into(),
            expected_revision: draft["revision"].as_i64().unwrap(),
            request_id: uid(),
            payments: vec![sm::Payment {
                payment_method: sm::Method::Cash,
                amount_minor: draft["totalMinor"].as_i64().unwrap(),
                reference: None,
            }],
        },
    )
    .unwrap();
    let rid = uid();
    sales::create_return(
        &f.db,
        &sm::ReturnDraft {
            id: rid.clone(),
            sale_id: posted["id"].as_str().unwrap().into(),
            reason: "Return one".into(),
            lines: vec![sm::ReturnLine {
                sale_item_id: posted["items"][0]["id"].as_str().unwrap().into(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    sales::post_return(
        &f.db,
        &sm::ReturnPost {
            id: rid,
            request_id: uid(),
        },
    )
    .unwrap();
    f.db.lock().unwrap().execute("INSERT INTO inventory_levels(id,branch_id,product_package_id,reorder_level) VALUES(?1,?2,?3,20)",rusqlite::params![uid(),f.branch,f.package]).unwrap();
}
fn post_sale(f: &F, customer_id: Option<String>, quantity: i64) -> serde_json::Value {
    let draft = sales::save(
        &f.db,
        &sm::Save {
            id: uid(),
            expected_revision: None,
            cart: sm::Cart {
                branch_id: f.branch.clone(),
                customer_id,
                lines: vec![sm::Line {
                    product_package_id: f.package.clone(),
                    quantity,
                    ..Default::default()
                }],
                ..Default::default()
            },
        },
    )
    .unwrap();
    sales::post(
        &f.db,
        &sm::Post {
            id: draft["id"].as_str().unwrap().into(),
            expected_revision: draft["revision"].as_i64().unwrap(),
            request_id: uid(),
            payments: vec![sm::Payment {
                payment_method: sm::Method::Cash,
                amount_minor: draft["totalMinor"].as_i64().unwrap(),
                reference: None,
            }],
        },
    )
    .unwrap()
}
fn filter(f: &F) -> ReportFilter {
    ReportFilter {
        branch_id: Some(f.branch.clone()),
        date_from: Some("2020-01-01".into()),
        date_to: Some("2040-01-01".into()),
        limit: Some(20),
        offset: Some(0),
        ..Default::default()
    }
}
fn total(v: &serde_json::Value) -> i64 {
    v["total"].as_i64().unwrap()
}

#[test]
fn operational_reports_reconcile_without_join_multiplication() {
    let f = fixture();
    seed(&f);
    let q = filter(&f);
    let o = reporting::overview(&f.db, &q).unwrap();
    assert_eq!(o["grossSalesMinor"], 400);
    assert_eq!(o["refundedMinor"], 200);
    assert_eq!(o["netSalesMinor"], 200);
    assert_eq!(o["netUnitsSold"], 1);
    assert_eq!(o["currentStockUnits"], 8);
    assert_eq!(o["receivedQuantity"], 10);
    assert_eq!(o["purchaseReturnQuantity"], 1);
    assert_eq!(total(&reporting::sales_report(&f.db, &q).unwrap()), 1);
    let p = reporting::sales_by_product(&f.db, &q).unwrap();
    assert_eq!(p["items"][0]["netQuantity"], 1);
    assert_eq!(p["items"][0]["netMinor"], 200);
    assert_eq!(total(&reporting::sales_by_customer(&f.db, &q).unwrap()), 1);
    let stock = reporting::inventory_stock(&f.db, &q).unwrap();
    assert_eq!(stock["items"][0]["quantity"], 8);
    assert_eq!(stock["items"][0]["acquisitionCostValuationMinor"], 800);
    assert_eq!(stock["items"][0]["stockState"], "LOW_STOCK");
    assert_eq!(
        total(&reporting::inventory_movements(&f.db, &q).unwrap()),
        4
    );
    assert_eq!(total(&reporting::expiry_report(&f.db, &q).unwrap()), 1);
    assert_eq!(total(&reporting::low_stock_report(&f.db, &q).unwrap()), 1);
    assert_eq!(
        reporting::purchasing_report(&f.db, &q).unwrap()["items"][0]["returnedQuantity"],
        1
    );
    assert_eq!(
        reporting::purchase_receipt_report(&f.db, &q).unwrap()["items"][0]["acquisitionValueMinor"],
        1000
    );
    assert_eq!(
        reporting::purchase_return_report(&f.db, &q).unwrap()["items"][0]["acquisitionValueMinor"],
        100
    );
    assert_eq!(
        reporting::supplier_analytics(&f.db, &q).unwrap()["items"][0]["returnedValueMinor"],
        100
    );
}

#[test]
fn reports_are_read_only_and_validate_boundaries() {
    let f = fixture();
    seed(&f);
    let tables = [
        "sales",
        "sale_returns",
        "inventory_movements",
        "purchase_orders",
        "purchase_receipts",
        "purchase_returns",
        "products",
        "suppliers",
        "customers",
    ];
    let snapshot = || {
        let c = f.db.lock().unwrap();
        tables.map(|t| {
            c.query_row(&format!("SELECT count(*) FROM {t}"), [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap()
        })
    };
    let before = snapshot();
    let q = filter(&f);
    reporting::overview(&f.db, &q).unwrap();
    reporting::inventory_stock(&f.db, &q).unwrap();
    reporting::supplier_analytics(&f.db, &q).unwrap();
    assert_eq!(before, snapshot());
    let bad = ReportFilter {
        date_from: Some("2030-01-02".into()),
        date_to: Some("2030-01-01".into()),
        ..Default::default()
    };
    assert!(reporting::overview(&f.db, &bad).is_err());
    let bad = ReportFilter {
        branch_id: Some(uid()),
        ..Default::default()
    };
    assert!(reporting::overview(&f.db, &bad).is_err());
    let bad = ReportFilter {
        expiry_days: Some(36501),
        ..Default::default()
    };
    assert!(reporting::expiry_report(&f.db, &bad).is_err());
}

#[test]
fn empty_reports_return_stable_zero_and_empty_pages() {
    let f = fixture();
    let q = filter(&f);
    let o = reporting::overview(&f.db, &q).unwrap();
    assert_eq!(o["netSalesMinor"], 0);
    assert_eq!(o["currentStockUnits"], 0);
    for result in [
        reporting::sales_report(&f.db, &q),
        reporting::sales_by_product(&f.db, &q),
        reporting::sales_by_customer(&f.db, &q),
        reporting::inventory_stock(&f.db, &q),
        reporting::inventory_movements(&f.db, &q),
        reporting::expiry_report(&f.db, &q),
        reporting::low_stock_report(&f.db, &q),
        reporting::purchasing_report(&f.db, &q),
        reporting::purchase_receipt_report(&f.db, &q),
        reporting::purchase_return_report(&f.db, &q),
    ] {
        assert_eq!(total(&result.unwrap()), 0)
    }
}

#[test]
fn package_and_partner_filters_are_scoped() {
    let f = fixture();
    seed(&f);
    let mut q = filter(&f);
    q.product_id = Some(f.product.clone());
    q.product_package_id = Some(f.package.clone());
    assert_eq!(total(&reporting::sales_by_product(&f.db, &q).unwrap()), 1);
    q.product_package_id = Some(uid());
    assert_eq!(total(&reporting::sales_by_product(&f.db, &q).unwrap()), 0);
    let mut q = filter(&f);
    q.supplier_id = Some(f.supplier.clone());
    assert_eq!(total(&reporting::supplier_analytics(&f.db, &q).unwrap()), 1);
    q.supplier_id = Some(uid());
    assert_eq!(total(&reporting::supplier_analytics(&f.db, &q).unwrap()), 0);
}

#[test]
fn sales_summary_separates_gross_refunds_and_returned_units() {
    let f = fixture();
    seed(&f);
    let report = reporting::sales_report(&f.db, &filter(&f)).unwrap();
    assert_eq!(report["summary"]["postedSalesCount"], 1);
    assert_eq!(report["summary"]["grossSalesMinor"], 400);
    assert_eq!(report["summary"]["refundedMinor"], 200);
    assert_eq!(report["summary"]["netSalesMinor"], 200);
    assert_eq!(report["summary"]["unitsSold"], 2);
    assert_eq!(report["summary"]["returnedQuantity"], 1);
    assert_eq!(report["summary"]["netUnitsSold"], 1);
}

#[test]
fn voided_sales_are_auditable_but_excluded_from_totals() {
    let f = fixture();
    seed(&f);
    let sale = post_sale(&f, None, 1);
    sales::void(
        &f.db,
        &sm::Void {
            id: sale["id"].as_str().unwrap().into(),
            request_id: uid(),
            reason: "Operator correction".into(),
        },
    )
    .unwrap();
    let report = reporting::sales_report(&f.db, &filter(&f)).unwrap();
    assert_eq!(report["total"], 2);
    assert_eq!(report["summary"]["postedSalesCount"], 1);
    assert_eq!(report["summary"]["grossSalesMinor"], 400);
    let voided = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["status"] == "VOIDED")
        .unwrap();
    assert_eq!(voided["netMinor"], 0);
}

#[test]
fn customer_analytics_exclude_walk_in_sales() {
    let f = fixture();
    seed(&f);
    post_sale(&f, None, 1);
    let report = reporting::sales_by_customer(&f.db, &filter(&f)).unwrap();
    assert_eq!(report["total"], 1);
    assert_eq!(report["items"][0]["customerId"], f.customer);
    assert_eq!(
        reporting::overview(&f.db, &filter(&f)).unwrap()["postedSalesCount"],
        2
    );
}

#[test]
fn single_utc_day_boundaries_are_inclusive() {
    let f = fixture();
    seed(&f);
    let day: String =
        f.db.lock()
            .unwrap()
            .query_row("SELECT date('now')", [], |r| r.get(0))
            .unwrap();
    let mut q = filter(&f);
    q.date_from = Some(day.clone());
    q.date_to = Some(day);
    assert_eq!(total(&reporting::sales_report(&f.db, &q).unwrap()), 1);
    assert_eq!(
        total(&reporting::purchase_receipt_report(&f.db, &q).unwrap()),
        1
    );
}

#[test]
fn movement_filter_and_backend_page_are_honored() {
    let f = fixture();
    seed(&f);
    let mut q = filter(&f);
    q.movement_type = Some("SALE_RETURN".into());
    let result = reporting::inventory_movements(&f.db, &q).unwrap();
    assert_eq!(result["total"], 1);
    assert_eq!(result["items"][0]["movementType"], "SALE_RETURN");
    q.movement_type = None;
    q.limit = Some(1);
    q.offset = Some(1);
    let result = reporting::inventory_movements(&f.db, &q).unwrap();
    assert_eq!(result["items"].as_array().unwrap().len(), 1);
    assert_eq!(result["limit"], 1);
    assert_eq!(result["offset"], 1);
}

#[test]
fn invalid_status_search_and_pagination_are_rejected() {
    let f = fixture();
    assert!(reporting::sales_report(
        &f.db,
        &ReportFilter {
            status: Some("PAID".into()),
            ..Default::default()
        }
    )
    .is_err());
    assert!(reporting::purchasing_report(
        &f.db,
        &ReportFilter {
            search: Some("x".repeat(201)),
            ..Default::default()
        }
    )
    .is_err());
    assert!(reporting::sales_report(
        &f.db,
        &ReportFilter {
            limit: Some(0),
            ..Default::default()
        }
    )
    .is_err());
}
