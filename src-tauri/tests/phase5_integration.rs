use m2_health_lib::{
    db::{
        connection::AppDb,
        models::catalog::{CreateProduct, CreateProductPackage, SetPackagePrice},
        repositories::{branches, price_history, product_packages, products},
    },
    desktop::{create_backup, restore_backup, DesktopState},
    finance::{
        self,
        models::{CreateExpense, FinanceQuery},
    },
    inventory::{
        balances,
        counts::{self, models as count_models},
    },
    partners::{
        self,
        models::{CustomerFields, SupplierFields, UpdateCustomer},
    },
    phase6::{self, models::Phase6Filter},
    purchasing::{self, models::*, receiving, returns},
    reporting::{self, models::ReportFilter},
    sales::{self, models as sale_models},
};
use tempfile::tempdir;

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

struct Fixture {
    db: AppDb,
    branch: String,
    supplier: String,
    customer: String,
    package: String,
}

fn fixture() -> Fixture {
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
            primary_phone: Some("01000000000".into()),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Integrated medicine".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let package = product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: product.id,
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
            reason: Some("Phase 5 integration".into()),
        },
    )
    .unwrap();
    Fixture {
        db,
        branch,
        supplier,
        customer,
        package,
    }
}

fn ordered(f: &Fixture, quantity: i64) -> PurchaseOrder {
    let draft = purchasing::create_draft(
        &f.db,
        &CreatePurchaseDraft {
            id: id(),
            request_id: id(),
            fields: PurchaseDraftFields {
                supplier_id: f.supplier.clone(),
                supplier_reference: Some("INT-1".into()),
                expected_date: None,
                notes: None,
                discount_minor: 0,
                tax_minor: 0,
                lines: vec![PurchaseLineInput {
                    product_package_id: f.package.clone(),
                    ordered_quantity: quantity,
                    unit_cost_minor: 100,
                }],
            },
        },
    )
    .unwrap();
    purchasing::order(
        &f.db,
        &PurchaseAction {
            id: draft.id,
            expected_revision: draft.revision,
            request_id: id(),
        },
    )
    .unwrap()
}

fn receive(f: &Fixture, order: &PurchaseOrder, quantity: i64, lot: &str) -> PurchaseReceipt {
    receiving::receive(
        &f.db,
        &ReceivePurchase {
            id: id(),
            purchase_order_id: order.id.clone(),
            expected_revision: order.revision,
            request_id: id(),
            branch_id: f.branch.clone(),
            created_by: Some("integration".into()),
            lines: vec![PurchaseReceiptLineInput {
                purchase_order_line_id: order.lines[0].id.clone(),
                quantity,
                lot_number: Some(lot.into()),
                expiry_date: Some("2030-01-01".into()),
                acquisition_cost_minor: None,
            }],
        },
    )
    .unwrap()
}

#[test]
fn supplier_purchase_receipt_return_traceability_is_integrated() {
    let f = fixture();
    let ordered = ordered(&f, 5);
    let first = receive(&f, &ordered, 2, "LOT-A");
    let partial = purchasing::detail(&f.db, &ordered.id).unwrap();
    assert_eq!(partial.status, "PARTIALLY_RECEIVED");
    let second = receive(&f, &partial, 3, "LOT-B");
    let complete = purchasing::detail(&f.db, &ordered.id).unwrap();
    assert_eq!(complete.status, "RECEIVED");
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        5
    );

    let returned = returns::create(
        &f.db,
        &CreatePurchaseReturn {
            id: id(),
            purchase_order_id: complete.id.clone(),
            receipt_id: second.id.clone(),
            expected_revision: complete.revision,
            request_id: id(),
            reason: "Damaged delivery".into(),
            created_by: Some("integration".into()),
            lines: vec![PurchaseReturnLineInput {
                receipt_line_id: second.lines[0].id.clone(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    assert_eq!(returned.lines[0].batch_id, second.lines[0].batch_id);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        4
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
        .total,
        1
    );
    assert_eq!(
        receiving::list(
            &f.db,
            &PurchaseReceiptQuery {
                supplier_id: Some(f.supplier.clone()),
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        2
    );
    assert_eq!(
        returns::list(
            &f.db,
            &PurchaseReturnQuery {
                supplier_id: Some(f.supplier),
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        1
    );
    assert_eq!(first.lines[0].movement_id.is_empty(), false);
}

#[test]
fn customer_sale_return_uses_received_stock_and_stable_snapshot() {
    let f = fixture();
    let ordered = ordered(&f, 3);
    let receipt = receive(&f, &ordered, 3, "CUSTOMER-FLOW");
    let cart = sale_models::Cart {
        branch_id: f.branch.clone(),
        customer_id: Some(f.customer.clone()),
        lines: vec![sale_models::Line {
            product_package_id: f.package.clone(),
            quantity: 1,
            ..Default::default()
        }],
        ..Default::default()
    };
    let draft = sales::save(
        &f.db,
        &sale_models::Save {
            id: id(),
            expected_revision: None,
            cart,
        },
    )
    .unwrap();
    let posted = sales::post(
        &f.db,
        &sale_models::Post {
            id: draft["id"].as_str().unwrap().into(),
            expected_revision: draft["revision"].as_i64().unwrap(),
            request_id: id(),
            payments: vec![sale_models::Payment {
                payment_method: sale_models::Method::Cash,
                amount_minor: draft["totalMinor"].as_i64().unwrap(),
                reference: None,
            }],
        },
    )
    .unwrap();
    assert_eq!(
        posted["items"][0]["allocations"][0]["batchId"],
        receipt.lines[0].batch_id
    );
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        2
    );

    partners::update_customer(
        &f.db,
        &UpdateCustomer {
            id: f.customer.clone(),
            expected_revision: 1,
            fields: CustomerFields {
                display_name: "Ahmed H.".into(),
                primary_phone: Some("01111111111".into()),
                ..Default::default()
            },
        },
    )
    .unwrap();
    let history = sales::customer_history(
        &f.db,
        &sale_models::CustomerSalesQuery {
            customer_id: f.customer.clone(),
            branch_id: f.branch.clone(),
            limit: Some(10),
            offset: Some(0),
        },
    )
    .unwrap();
    assert_eq!(history["total"], 1);
    assert_eq!(history["items"][0]["customerName"], "Ahmed Hassan");

    let return_id = id();
    sales::create_return(
        &f.db,
        &sale_models::ReturnDraft {
            id: return_id.clone(),
            sale_id: posted["id"].as_str().unwrap().into(),
            reason: "Customer return".into(),
            lines: vec![sale_models::ReturnLine {
                sale_item_id: posted["items"][0]["id"].as_str().unwrap().into(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    let returned = sales::post_return(
        &f.db,
        &sale_models::ReturnPost {
            id: return_id,
            request_id: id(),
        },
    )
    .unwrap();
    assert_eq!(returned["customerId"], f.customer);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        3
    );
}

#[test]
fn phase7c_cross_domain_flow_reconciles_stock_finance_history_and_integrity() {
    let f = fixture();
    let ordered = ordered(&f, 5);
    let receipt = receive(&f, &ordered, 5, "PHASE-7C");
    let complete = purchasing::detail(&f.db, &ordered.id).unwrap();
    let purchase_return = returns::create(
        &f.db,
        &CreatePurchaseReturn {
            id: id(),
            purchase_order_id: complete.id.clone(),
            receipt_id: receipt.id.clone(),
            expected_revision: complete.revision,
            request_id: id(),
            reason: "Phase 7C supplier return".into(),
            created_by: Some("phase-7c-actor".into()),
            lines: vec![PurchaseReturnLineInput {
                receipt_line_id: receipt.lines[0].id.clone(),
                quantity: 1,
            }],
        },
    )
    .unwrap();

    let draft = sales::save(
        &f.db,
        &sale_models::Save {
            id: id(),
            expected_revision: None,
            cart: sale_models::Cart {
                branch_id: f.branch.clone(),
                customer_id: Some(f.customer.clone()),
                lines: vec![sale_models::Line {
                    product_package_id: f.package.clone(),
                    quantity: 2,
                    ..Default::default()
                }],
                ..Default::default()
            },
        },
    )
    .unwrap();
    let post = sale_models::Post {
        id: draft["id"].as_str().unwrap().into(),
        expected_revision: draft["revision"].as_i64().unwrap(),
        request_id: id(),
        payments: vec![sale_models::Payment {
            payment_method: sale_models::Method::Cash,
            amount_minor: draft["totalMinor"].as_i64().unwrap(),
            reference: None,
        }],
    };
    let sale = sales::post(&f.db, &post).unwrap();
    assert_eq!(sales::post(&f.db, &post).unwrap()["id"], sale["id"]);

    let return_id = id();
    sales::create_return(
        &f.db,
        &sale_models::ReturnDraft {
            id: return_id.clone(),
            sale_id: sale["id"].as_str().unwrap().into(),
            reason: "Phase 7C customer return".into(),
            lines: vec![sale_models::ReturnLine {
                sale_item_id: sale["items"][0]["id"].as_str().unwrap().into(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    let return_post = sale_models::ReturnPost {
        id: return_id,
        request_id: id(),
    };
    sales::post_return(&f.db, &return_post).unwrap();
    sales::post_return(&f.db, &return_post).unwrap();

    let count = counts::create(
        &f.db,
        &count_models::Create {
            branch_id: f.branch.clone(),
            notes: Some("Desktop closure journey count".into()),
            created_by: None,
        },
    )
    .unwrap();
    let count = counts::start(
        &f.db,
        &count_models::Transition {
            session_id: count.session.id,
            expected_revision: count.session.revision,
        },
    )
    .unwrap();
    assert_eq!(count.lines.len(), 1);
    let count = counts::record(
        &f.db,
        &count_models::Entry {
            session_id: count.session.id.clone(),
            expected_revision: count.session.revision,
            line_id: count.lines[0].id.clone(),
            line_revision: count.lines[0].revision,
            counted_quantity: Some(4),
            notes: Some("Verified physical count".into()),
            excluded_reason: None,
        },
    )
    .unwrap();
    let count = counts::transition(
        &f.db,
        &count_models::Transition {
            session_id: count.session.id,
            expected_revision: count.session.revision,
        },
        count_models::Status::Review,
    )
    .unwrap();
    counts::complete(
        &f.db,
        &count_models::Complete {
            session_id: count.session.id,
            expected_revision: count.session.revision,
            request_id: id(),
            review_token: count.review_token,
        },
    )
    .unwrap();

    let (expense_account, cash_account): (String, String) = {
        let c = f.db.lock().unwrap();
        (
            c.query_row(
                "SELECT id FROM finance_accounts WHERE code='6100'",
                [],
                |r| r.get(0),
            )
            .unwrap(),
            c.query_row(
                "SELECT id FROM finance_accounts WHERE code='1000'",
                [],
                |r| r.get(0),
            )
            .unwrap(),
        )
    };
    let expense = CreateExpense {
        id: id(),
        request_id: id(),
        branch_id: Some(f.branch.clone()),
        expense_account_id: expense_account,
        payment_account_id: cash_account,
        amount_minor: 100,
        description: "Phase 7C operating expense".into(),
        created_by: Some("phase-7c-actor".into()),
    };
    let first_expense = finance::create_expense(&f.db, &expense).unwrap();
    assert_eq!(
        finance::create_expense(&f.db, &expense).unwrap()["id"],
        first_expense["id"]
    );

    let report = ReportFilter {
        branch_id: Some(f.branch.clone()),
        ..Default::default()
    };
    let overview = reporting::overview(&f.db, &report).unwrap();
    assert_eq!(overview["receivedQuantity"], 5);
    assert_eq!(overview["purchaseReturnQuantity"], 1);
    assert_eq!(overview["netUnitsSold"], 1);
    assert_eq!(overview["currentStockUnits"], 4);
    assert_eq!(
        reporting::supplier_analytics(
            &f.db,
            &ReportFilter {
                supplier_id: Some(f.supplier.clone()),
                ..report.clone()
            }
        )
        .unwrap()["total"],
        1
    );
    assert_eq!(
        sales::customer_history(
            &f.db,
            &sale_models::CustomerSalesQuery {
                customer_id: f.customer.clone(),
                branch_id: f.branch.clone(),
                limit: Some(10),
                offset: Some(0)
            }
        )
        .unwrap()["total"],
        1
    );

    let finance_query = FinanceQuery {
        branch_id: Some(f.branch.clone()),
        ..Default::default()
    };
    let trial = finance::trial_balance(&f.db, &finance_query).unwrap();
    assert_eq!(trial["balanced"], true);
    assert_eq!(trial["debitTotalMinor"], trial["creditTotalMinor"]);
    let dashboard = phase6::dashboard(
        &f.db,
        &Phase6Filter {
            branch_id: Some(f.branch.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(dashboard["operational"]["currentStockUnits"], 4);
    assert_eq!(dashboard["coverage"]["coverageComplete"], true);

    let c = f.db.lock().unwrap();
    assert_eq!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        0
    );
    assert_eq!(c.query_row("SELECT count(*) FROM inventory_movements WHERE reference_type='purchase_return' AND reference_id=?1", [&purchase_return.id], |r| r.get::<_, i64>(0)).unwrap(), 1);
    drop(c);

    let root = tempdir().unwrap();
    let backup_state = DesktopState::new(root.path().join("backups"));
    f.db.lock()
        .unwrap()
        .execute(
            "UPDATE local_settings SET pharmacy_name='Journey Snapshot' WHERE singleton=1",
            [],
        )
        .unwrap();
    let backup = create_backup(&f.db, &backup_state, "backup").unwrap();
    f.db.lock()
        .unwrap()
        .execute(
            "UPDATE local_settings SET pharmacy_name='Changed After Backup' WHERE singleton=1",
            [],
        )
        .unwrap();
    let restored = restore_backup(
        &f.db,
        &backup_state,
        &backup.id,
        "RESTORE",
        "Desktop closure acceptance journey",
    )
    .unwrap();
    assert!(restored.restart_required);
    assert!(root
        .path()
        .join("backups")
        .join(&restored.safety_backup_id)
        .is_file());
    assert_eq!(
        f.db.lock()
            .unwrap()
            .query_row(
                "SELECT pharmacy_name FROM local_settings WHERE singleton=1",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "Journey Snapshot"
    );
}
