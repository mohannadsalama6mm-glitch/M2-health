use m2_health_lib::{
    db::{
        connection::AppDb,
        models::catalog::{CreateProduct, CreateProductPackage, SetPackagePrice},
        repositories::{branches, price_history, product_packages, products},
    },
    finance::{self, models::*},
    inventory::{
        balances, batches,
        models::{CreateBatch, StockOperation},
        movements,
    },
    partners::{self, models::SupplierFields},
    phase6::{
        self,
        models::{CsvExportInput, Phase6Filter},
    },
    purchasing::{self, models::*, receiving, returns as purchase_returns},
    sales::{self, models as sale},
};
use rusqlite::params;

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

struct Fixture {
    db: AppDb,
    branch: String,
    supplier: String,
    package: String,
}

fn fixture() -> Fixture {
    let db = AppDb::in_memory().unwrap();
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let supplier = partners::create_supplier(
        &db,
        &SupplierFields {
            display_name: "Finance Supplier".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id;
    let product = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Finance Medicine".into()),
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
            reason: Some("Finance test".into()),
        },
    )
    .unwrap();
    Fixture {
        db,
        branch,
        supplier,
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
                supplier_reference: None,
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

fn receive(
    f: &Fixture,
    order: &PurchaseOrder,
    quantity: i64,
    lot: &str,
    cost: i64,
) -> PurchaseReceipt {
    receiving::receive(
        &f.db,
        &ReceivePurchase {
            id: id(),
            purchase_order_id: order.id.clone(),
            expected_revision: order.revision,
            request_id: id(),
            branch_id: f.branch.clone(),
            created_by: Some("Finance test".into()),
            lines: vec![PurchaseReceiptLineInput {
                purchase_order_line_id: order.lines[0].id.clone(),
                quantity,
                lot_number: Some(lot.into()),
                expiry_date: Some("2035-01-01".into()),
                acquisition_cost_minor: Some(cost),
            }],
        },
    )
    .unwrap()
}

fn post_sale(
    f: &Fixture,
    quantity: i64,
    line_discount: i64,
    discount: i64,
    tax: i64,
) -> serde_json::Value {
    let draft = sales::save(
        &f.db,
        &sale::Save {
            id: id(),
            expected_revision: None,
            cart: sale::Cart {
                branch_id: f.branch.clone(),
                lines: vec![sale::Line {
                    product_package_id: f.package.clone(),
                    quantity,
                    discount_minor: line_discount,
                    ..Default::default()
                }],
                discount_minor: discount,
                tax_minor: tax,
                ..Default::default()
            },
        },
    )
    .unwrap();
    sales::post(
        &f.db,
        &sale::Post {
            id: draft["id"].as_str().unwrap().into(),
            expected_revision: draft["revision"].as_i64().unwrap(),
            request_id: id(),
            payments: vec![sale::Payment {
                payment_method: sale::Method::Cash,
                amount_minor: draft["totalMinor"].as_i64().unwrap(),
                reference: None,
            }],
        },
    )
    .unwrap()
}

fn account(f: &Fixture, code: &str) -> String {
    f.db.lock()
        .unwrap()
        .query_row(
            "SELECT id FROM finance_accounts WHERE code=?1",
            [code],
            |row| row.get(0),
        )
        .unwrap()
}

fn count(f: &Fixture, table: &str) -> i64 {
    f.db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

#[test]
fn migration_015_creates_protected_double_entry_schema() {
    let f = fixture();
    let connection = f.db.lock().unwrap();
    assert_eq!(
        connection
            .query_row("SELECT max(version) FROM _schema_version", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        21
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM finance_accounts WHERE is_system=1",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        14
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
    for table in [
        "finance_accounts",
        "finance_journals",
        "finance_journal_lines",
        "finance_expenses",
    ] {
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
    }
}

#[test]
fn sale_return_void_and_expense_reconcile_exactly() {
    let f = fixture();
    let order = ordered(&f, 5);
    let first = receive(&f, &order, 2, "LOT-A", 100);
    let partial = purchasing::detail(&f.db, &order.id).unwrap();
    let second = receive(&f, &partial, 3, "LOT-B", 130);
    assert_eq!(
        count(&f, "finance_journals"),
        0,
        "purchase receipts are outside Finance 6B"
    );

    let posted = post_sale(&f, 5, 50, 50, 100);
    assert_eq!(posted["totalMinor"], 1000);
    let sale_journal = finance::journals(
        &f.db,
        &FinanceQuery {
            source_type: Some("SALE".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(sale_journal["total"], 1);
    assert_eq!(sale_journal["items"][0]["debitTotalMinor"], 1590);
    assert_eq!(sale_journal["items"][0]["cogsComplete"], true);

    let return_id = id();
    sales::create_return(
        &f.db,
        &sale::ReturnDraft {
            id: return_id.clone(),
            sale_id: posted["id"].as_str().unwrap().into(),
            reason: "Two units".into(),
            lines: vec![sale::ReturnLine {
                sale_item_id: posted["items"][0]["id"].as_str().unwrap().into(),
                quantity: 2,
            }],
        },
    )
    .unwrap();
    sales::post_return(
        &f.db,
        &sale::ReturnPost {
            id: return_id,
            request_id: id(),
        },
    )
    .unwrap();
    let return_journal = finance::journals(
        &f.db,
        &FinanceQuery {
            source_type: Some("SALE_RETURN".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(return_journal["items"][0]["debitTotalMinor"], 600);

    let second_sale = post_sale(&f, 1, 0, 0, 0);
    sales::void(
        &f.db,
        &sale::Void {
            id: second_sale["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: "Void exact reversal".into(),
        },
    )
    .unwrap();

    let expense = finance::create_expense(
        &f.db,
        &CreateExpense {
            id: id(),
            request_id: id(),
            branch_id: Some(f.branch.clone()),
            expense_account_id: account(&f, "6100"),
            payment_account_id: account(&f, "1000"),
            amount_minor: 150,
            description: "October rent".into(),
            created_by: Some("Tester".into()),
        },
    )
    .unwrap();
    assert_eq!(expense["status"], "POSTED");

    let summary = finance::sales_summary(&f.db, &FinanceQuery::default()).unwrap();
    assert_eq!(summary["salesRevenueMinor"], 900);
    assert_eq!(summary["salesReturnsMinor"], 360);
    assert_eq!(summary["netSalesRevenueMinor"], 540);
    assert_eq!(summary["netCogsMinor"], 390);
    assert_eq!(summary["grossMarginMinor"], 150);
    assert_eq!(summary["operatingExpensesMinor"], 150);
    assert_eq!(summary["operatingResultMinor"], 0);
    assert_eq!(summary["refundOutflowMinor"], 400);
    let trial = finance::trial_balance(&f.db, &FinanceQuery::default()).unwrap();
    assert_eq!(trial["balanced"], true);
    assert_eq!(trial["debitTotalMinor"], trial["creditTotalMinor"]);
    assert_eq!(
        balances::package(&f.db, &f.branch, &f.package)
            .unwrap()
            .quantity,
        2
    );
    assert_eq!(first.lines[0].acquisition_cost_minor, 100);
    assert_eq!(second.lines[0].acquisition_cost_minor, 130);
}

#[test]
fn expense_retry_is_idempotent_and_changed_payload_conflicts() {
    let f = fixture();
    let mut input = CreateExpense {
        id: id(),
        request_id: id(),
        branch_id: Some(f.branch.clone()),
        expense_account_id: account(&f, "6110"),
        payment_account_id: account(&f, "1010"),
        amount_minor: 875,
        description: "Utilities".into(),
        created_by: None,
    };
    let first = finance::create_expense(&f.db, &input).unwrap();
    assert_eq!(finance::create_expense(&f.db, &input).unwrap(), first);
    assert_eq!(count(&f, "finance_expenses"), 1);
    assert_eq!(count(&f, "finance_journals"), 1);
    input.amount_minor += 1;
    assert!(finance::create_expense(&f.db, &input).is_err());
    assert_eq!(count(&f, "finance_expenses"), 1);
}

#[test]
fn invalid_expense_rolls_back_number_document_and_journal() {
    let f = fixture();
    let bad = CreateExpense {
        id: id(),
        request_id: id(),
        branch_id: Some(f.branch.clone()),
        expense_account_id: account(&f, "4000"),
        payment_account_id: account(&f, "1000"),
        amount_minor: 100,
        description: "Invalid account".into(),
        created_by: None,
    };
    assert!(finance::create_expense(&f.db, &bad).is_err());
    assert_eq!(count(&f, "finance_expense_sequences"), 0);
    assert_eq!(count(&f, "finance_expenses"), 0);
    assert_eq!(count(&f, "finance_journals"), 0);
}

#[test]
fn posted_finance_history_is_immutable_and_unbalanced_post_is_rejected() {
    let f = fixture();
    let expense = finance::create_expense(
        &f.db,
        &CreateExpense {
            id: id(),
            request_id: id(),
            branch_id: None,
            expense_account_id: account(&f, "6190"),
            payment_account_id: account(&f, "1020"),
            amount_minor: 250,
            description: "Supplies".into(),
            created_by: None,
        },
    )
    .unwrap();
    let journal = expense["journalId"].as_str().unwrap();
    let rent_account = account(&f, "6100");
    let connection = f.db.lock().unwrap();
    assert!(connection
        .execute(
            "UPDATE finance_journals SET description='changed' WHERE id=?1",
            [journal]
        )
        .is_err());
    assert!(connection
        .execute(
            "DELETE FROM finance_journal_lines WHERE journal_id=?1",
            [journal]
        )
        .is_err());
    assert!(connection
        .execute(
            "DELETE FROM finance_expenses WHERE id=?1",
            [expense["id"].as_str().unwrap()]
        )
        .is_err());
    let draft = id();
    connection.execute("INSERT INTO finance_journals(id,journal_number,source_type,source_id,description,debit_total_minor,credit_total_minor) VALUES(?1,'JRN-TEST-1','EXPENSE',?2,'Crafted',100,100)", params![draft, id()]).unwrap();
    connection.execute("INSERT INTO finance_journal_lines(id,journal_id,position,account_id,side,amount_minor) VALUES(?1,?2,0,?3,'DEBIT',100)", params![id(), draft, rent_account]).unwrap();
    assert!(connection.execute("UPDATE finance_journals SET status='POSTED',posted_at='2030-01-01T00:00:00.000Z' WHERE id=?1", [&draft]).is_err());
}

#[test]
fn purchase_receipt_and_return_do_not_invent_payables_or_cash() {
    let f = fixture();
    let order = ordered(&f, 2);
    let receipt = receive(&f, &order, 2, "NO-AP", 100);
    let current = purchasing::detail(&f.db, &order.id).unwrap();
    purchase_returns::create(
        &f.db,
        &CreatePurchaseReturn {
            id: id(),
            purchase_order_id: current.id,
            receipt_id: receipt.id.clone(),
            expected_revision: current.revision,
            request_id: id(),
            reason: "Supplier return without settlement".into(),
            created_by: None,
            lines: vec![PurchaseReturnLineInput {
                receipt_line_id: receipt.lines[0].id.clone(),
                quantity: 1,
            }],
        },
    )
    .unwrap();
    assert_eq!(count(&f, "finance_journals"), 0);
}

#[test]
fn filters_are_branch_and_date_scoped_and_reports_are_read_only() {
    let f = fixture();
    let expense = finance::create_expense(
        &f.db,
        &CreateExpense {
            id: id(),
            request_id: id(),
            branch_id: Some(f.branch.clone()),
            expense_account_id: account(&f, "6120"),
            payment_account_id: account(&f, "1000"),
            amount_minor: 300,
            description: "Delivery".into(),
            created_by: None,
        },
    )
    .unwrap();
    let before = (
        count(&f, "finance_journals"),
        count(&f, "finance_journal_lines"),
    );
    let today: String =
        f.db.lock()
            .unwrap()
            .query_row("SELECT date('now')", [], |r| r.get(0))
            .unwrap();
    let yes = FinanceQuery {
        date_from: Some(today.clone()),
        date_to: Some(today),
        branch_id: Some(f.branch.clone()),
        ..Default::default()
    };
    assert_eq!(finance::expenses(&f.db, &yes).unwrap()["total"], 1);
    let no = FinanceQuery {
        date_from: Some("2000-01-01".into()),
        date_to: Some("2000-01-02".into()),
        ..Default::default()
    };
    assert_eq!(
        finance::trial_balance(&f.db, &no).unwrap()["debitTotalMinor"],
        0
    );
    assert_eq!(
        finance::account_activity(
            &f.db,
            &FinanceQuery {
                account_id: Some(account(&f, "6120")),
                ..yes
            }
        )
        .unwrap()["total"],
        1
    );
    assert_eq!(
        finance::journal(&f.db, expense["journalId"].as_str().unwrap()).unwrap()["sourceType"],
        "EXPENSE"
    );
    assert_eq!(
        before,
        (
            count(&f, "finance_journals"),
            count(&f, "finance_journal_lines")
        )
    );
}

#[test]
fn missing_allocation_cost_posts_revenue_but_suppresses_margin() {
    let f = fixture();
    let batch = batches::create(
        &f.db,
        &CreateBatch {
            branch_id: f.branch.clone(),
            product_package_id: f.package.clone(),
            lot_number: Some("UNKNOWN-COST".into()),
            expiry_date: Some("2035-01-01".into()),
            received_at: None,
            cost_price_minor: None,
            supplier_reference: None,
        },
    )
    .unwrap();
    movements::opening_existing(
        &f.db,
        &StockOperation {
            request_id: id(),
            branch_id: f.branch.clone(),
            product_package_id: f.package.clone(),
            batch_id: batch.id,
            quantity: 1,
            reason: "Explicit unvalued test stock".into(),
            created_by: None,
        },
    )
    .unwrap();
    post_sale(&f, 1, 0, 0, 0);
    let journal = finance::journals(
        &f.db,
        &FinanceQuery {
            source_type: Some("SALE".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(journal["items"][0]["cogsComplete"], false);
    let summary = finance::sales_summary(&f.db, &FinanceQuery::default()).unwrap();
    assert_eq!(summary["salesRevenueMinor"], 200);
    assert_eq!(summary["missingCogsJournalCount"], 1);
    assert!(summary["grossMarginMinor"].is_null());
    assert!(summary["operatingResultMinor"].is_null());
}

#[test]
fn injected_line_or_source_link_failure_rolls_back_every_finance_record() {
    for trigger in [
        "CREATE TRIGGER fail_finance_line BEFORE INSERT ON finance_journal_lines BEGIN SELECT RAISE(ABORT,'injected line failure'); END;",
        "CREATE TRIGGER fail_finance_link BEFORE UPDATE ON finance_expenses BEGIN SELECT RAISE(ABORT,'injected link failure'); END;",
    ] {
        let f = fixture();
        f.db.lock().unwrap().execute_batch(trigger).unwrap();
        let result = finance::create_expense(&f.db, &CreateExpense {
            id: id(), request_id: id(), branch_id: Some(f.branch.clone()), expense_account_id: account(&f, "6100"),
            payment_account_id: account(&f, "1000"), amount_minor: 500, description: "Injected rollback".into(), created_by: None,
        });
        assert!(result.is_err());
        assert_eq!(count(&f, "finance_expenses"), 0);
        assert_eq!(count(&f, "finance_journals"), 0);
        assert_eq!(count(&f, "finance_journal_lines"), 0);
        assert_eq!(count(&f, "finance_expense_sequences"), 0);
        assert_eq!(count(&f, "finance_journal_sequences"), 0);
    }
}

#[test]
fn concurrent_expenses_receive_unique_numbers_and_journals() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("finance-race.db");
    let seed = AppDb::open(&path).unwrap();
    let branch = branches::ensure_default_branch(&seed).unwrap().id;
    let expense_account = seed
        .lock()
        .unwrap()
        .query_row(
            "SELECT id FROM finance_accounts WHERE code='6100'",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap();
    let payment_account = seed
        .lock()
        .unwrap()
        .query_row(
            "SELECT id FROM finance_accounts WHERE code='1000'",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap();
    drop(seed);
    let one = AppDb::open(&path).unwrap();
    let two = AppDb::open(&path).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let make = |db: AppDb, gate: std::sync::Arc<std::sync::Barrier>, suffix: &str| {
        let branch = branch.clone();
        let expense_account = expense_account.clone();
        let payment_account = payment_account.clone();
        let suffix = suffix.to_string();
        std::thread::spawn(move || {
            gate.wait();
            finance::create_expense(
                &db,
                &CreateExpense {
                    id: id(),
                    request_id: id(),
                    branch_id: Some(branch),
                    expense_account_id: expense_account,
                    payment_account_id: payment_account,
                    amount_minor: 100,
                    description: format!("Concurrent {suffix}"),
                    created_by: None,
                },
            )
            .unwrap()
        })
    };
    let first = make(one, barrier.clone(), "one");
    let second = make(two, barrier, "two");
    let a = first.join().unwrap();
    let b = second.join().unwrap();
    assert_ne!(a["expenseNumber"], b["expenseNumber"]);
    assert_ne!(a["journalId"], b["journalId"]);
    let check = AppDb::open(&path).unwrap();
    assert_eq!(
        check
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(DISTINCT expense_number) FROM finance_expenses",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
    assert_eq!(
        check
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(DISTINCT journal_number) FROM finance_journals",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
}

#[test]
fn phase6_dashboard_reconciles_operational_and_finance_values() {
    let f = fixture();
    let order = ordered(&f, 5);
    receive(&f, &order, 2, "P6-DASH-A", 100);
    let partial = purchasing::detail(&f.db, &order.id).unwrap();
    receive(&f, &partial, 3, "P6-DASH-B", 130);
    let posted = post_sale(&f, 5, 50, 50, 100);
    let return_id = id();
    sales::create_return(
        &f.db,
        &sale::ReturnDraft {
            id: return_id.clone(),
            sale_id: posted["id"].as_str().unwrap().into(),
            reason: "Phase 6 exact return".into(),
            lines: vec![sale::ReturnLine {
                sale_item_id: posted["items"][0]["id"].as_str().unwrap().into(),
                quantity: 2,
            }],
        },
    )
    .unwrap();
    sales::post_return(
        &f.db,
        &sale::ReturnPost {
            id: return_id,
            request_id: id(),
        },
    )
    .unwrap();
    let voided = post_sale(&f, 1, 0, 0, 0);
    sales::void(
        &f.db,
        &sale::Void {
            id: voided["id"].as_str().unwrap().into(),
            request_id: id(),
            reason: "Phase 6 exact void".into(),
        },
    )
    .unwrap();
    finance::create_expense(
        &f.db,
        &CreateExpense {
            id: id(),
            request_id: id(),
            branch_id: Some(f.branch.clone()),
            expense_account_id: account(&f, "6100"),
            payment_account_id: account(&f, "1000"),
            amount_minor: 150,
            description: "Phase 6 operating expense".into(),
            created_by: None,
        },
    )
    .unwrap();
    let value = phase6::dashboard(&f.db, &Phase6Filter::default()).unwrap();
    assert_eq!(value["operational"]["postedSalesCount"], 1);
    assert_eq!(value["operational"]["netSalesMinor"], 600);
    assert_eq!(value["operational"]["netUnitsSold"], 3);
    assert_eq!(value["operational"]["currentStockUnits"], 2);
    assert_eq!(value["operational"]["receivedQuantity"], 5);
    assert_eq!(value["financial"]["netSalesRevenueMinor"], 540);
    assert_eq!(value["financial"]["netCogsMinor"], 390);
    assert_eq!(value["financial"]["grossMarginMinor"], 150);
    assert_eq!(value["financial"]["operatingExpensesMinor"], 150);
    assert_eq!(value["financial"]["operatingResultMinor"], 0);
    assert_eq!(value["coverage"]["coverageComplete"], true);
    assert_eq!(value["coverage"]["eligibleSaleCount"], 2);
    assert_eq!(value["coverage"]["postedReturnCount"], 1);
    assert_eq!(value["coverage"]["voidCount"], 1);
    let trace = phase6::sale_finance_trace(&f.db, voided["id"].as_str().unwrap()).unwrap();
    assert_eq!(trace["financePosted"], true);
    assert_eq!(trace["historicalUnposted"], false);
    assert_eq!(trace["journals"].as_array().unwrap().len(), 2);
    assert_eq!(trace["journals"][0]["sourceType"], "SALE");
    assert_eq!(trace["journals"][1]["sourceType"], "SALE_VOID");
}

#[test]
fn phase6_coverage_reports_historical_unposted_sales_honestly() {
    let f = fixture();
    let order = ordered(&f, 1);
    receive(&f, &order, 1, "P6-HISTORY", 100);
    let posted = post_sale(&f, 1, 0, 0, 0);
    {
        // Fixture-only simulation of a pre-Finance operational record. Production
        // immutability is independently tested and is never bypassed by Phase 6.
        let c = f.db.lock().unwrap();
        c.execute_batch(
            "DROP TRIGGER finance_journal_line_no_delete; DROP TRIGGER finance_journal_no_delete;",
        )
        .unwrap();
        let journal: String = c
            .query_row(
                "SELECT id FROM finance_journals WHERE source_type='SALE' AND source_id=?1",
                [posted["id"].as_str().unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        c.execute(
            "DELETE FROM finance_journal_lines WHERE journal_id=?1",
            [&journal],
        )
        .unwrap();
        c.execute("DELETE FROM finance_journals WHERE id=?1", [&journal])
            .unwrap();
    }
    let coverage = phase6::reconciliation(&f.db, &Phase6Filter::default()).unwrap();
    assert_eq!(coverage["eligibleSaleCount"], 1);
    assert_eq!(coverage["representedSaleCount"], 0);
    assert_eq!(coverage["unrepresentedEventCount"], 1);
    assert_eq!(coverage["coverageComplete"], false);
    let trace = phase6::sale_finance_trace(&f.db, posted["id"].as_str().unwrap()).unwrap();
    assert_eq!(trace["historicalUnposted"], true);
    assert_eq!(trace["journals"].as_array().unwrap().len(), 0);
}

#[test]
fn phase6_csv_is_filtered_exact_utf8_safe_and_read_only() {
    let f = fixture();
    let order = ordered(&f, 2);
    receive(&f, &order, 2, "P6-CSV", 100);
    post_sale(&f, 1, 0, 0, 0);
    finance::create_expense(
        &f.db,
        &CreateExpense {
            id: id(),
            request_id: id(),
            branch_id: Some(f.branch.clone()),
            expense_account_id: account(&f, "6190"),
            payment_account_id: account(&f, "1000"),
            amount_minor: 125,
            description: "=cmd, صيانة".into(),
            created_by: None,
        },
    )
    .unwrap();
    let before = (
        count(&f, "finance_journals"),
        count(&f, "inventory_movements"),
    );
    let value = phase6::export_csv(
        &f.db,
        &CsvExportInput {
            kind: "EXPENSES".into(),
            report_filter: Default::default(),
            finance_query: FinanceQuery {
                branch_id: Some(f.branch.clone()),
                ..Default::default()
            },
        },
    )
    .unwrap();
    let content = value["content"].as_str().unwrap();
    assert_eq!(value["rowCount"], 1);
    assert!(content.starts_with("Expense,Category,Description,Amount,Branch,Posted\r\n"));
    assert!(content.contains("\"'=cmd, صيانة\""));
    assert!(content.contains("1.25 EGP"));
    assert_eq!(
        before,
        (
            count(&f, "finance_journals"),
            count(&f, "inventory_movements")
        )
    );
}
