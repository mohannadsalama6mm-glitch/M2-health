//! Phase 7B authorization boundary for every registered application IPC command.
//!
//! The Tauri dispatch gate calls this module before command deserialization or
//! domain code. Unknown commands are denied. Domain transactions retain their
//! existing atomicity because an authorization failure never enters them.

use crate::{
    auth::{self, AuthState, Principal},
    commands::branches::DatabaseState,
    db::connection::AppDb,
    error::{AppError, ErrorCode},
};
use rusqlite::{Connection, OptionalExtension};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::ipc::InvokeBody;
use tauri::Runtime;

/// Serializes authorization with protected command execution. This prevents an
/// administrator membership/role mutation from racing a command that has just
/// checked the former grants. Domain commands still own their SQLite IMMEDIATE
/// transactions and all business atomicity.
#[derive(Clone, Default)]
pub struct AuthorizationState {
    boundary: Arc<Mutex<()>>,
    current_principal: Arc<Mutex<Option<Principal>>>,
}
impl AuthorizationState {
    pub fn boundary(&self) -> Arc<Mutex<()>> {
        Arc::clone(&self.boundary)
    }

    fn enter(&self, principal: Principal) -> Result<PrincipalScope, AppError> {
        let mut current = self.current_principal.lock().map_err(|_| {
            AppError::new(
                ErrorCode::Internal,
                "Authorization state is unavailable. Restart the application.",
            )
        })?;
        if current.is_some() {
            return Err(AppError::new(
                ErrorCode::Internal,
                "Authorization context is already in use.",
            ));
        }
        *current = Some(principal);
        Ok(PrincipalScope {
            current: Arc::clone(&self.current_principal),
        })
    }

    /// Returns the principal established by the centralized dispatch gate for
    /// the currently executing synchronous command.
    pub fn principal(&self) -> Result<Principal, AppError> {
        self.current_principal
            .lock()
            .map_err(|_| {
                AppError::new(
                    ErrorCode::Internal,
                    "Authorization state is unavailable. Restart the application.",
                )
            })?
            .clone()
            .ok_or_else(unauthorized)
    }

    pub fn actor(&self) -> Result<String, AppError> {
        Ok(self.principal()?.user_id)
    }
}

struct PrincipalScope {
    current: Arc<Mutex<Option<Principal>>>,
}
impl Drop for PrincipalScope {
    fn drop(&mut self) {
        if let Ok(mut current) = self.current.lock() {
            *current = None;
        }
    }
}

/// Wraps the generated Tauri handler. This is the executable IPC boundary used
/// by the application.
pub fn guard<R, F>(handler: F) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: Runtime,
    F: Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
{
    move |invoke| {
        let command = invoke.message.command().to_owned();
        if policy(&command).is_some_and(|policy| policy.scope == Scope::Public) {
            return handler(invoke);
        }
        let database = invoke.message.state_ref().get::<DatabaseState>();
        let auth_state = invoke.message.state_ref().get::<AuthState>();
        let authorization_state = invoke.message.state_ref().get::<AuthorizationState>();
        let authorization_boundary = authorization_state.boundary();
        let boundary = match authorization_boundary.lock() {
            Ok(boundary) => boundary,
            Err(_) => {
                invoke.resolver.reject(AppError::new(
                    ErrorCode::Internal,
                    "Authorization state is unavailable. Restart the application.",
                ));
                return true;
            }
        };
        match database
            .db()
            .and_then(|db| authorize_invoke(&command, invoke.message.payload(), db, &auth_state))
        {
            Ok(Some(principal)) => {
                let _principal_scope = match authorization_state.enter(principal) {
                    Ok(scope) => scope,
                    Err(error) => {
                        invoke.resolver.reject(error);
                        return true;
                    }
                };
                let handled = handler(invoke);
                drop(boundary);
                handled
            }
            Ok(None) => {
                invoke.resolver.reject(forbidden());
                true
            }
            Err(error) => {
                invoke.resolver.reject(error);
                true
            }
        }
    }
}

/// Test seam matching the protected dispatch order used by the Tauri gate. The
/// closure is entered only after session, permission, branch, record ownership,
/// and actor checks pass.
pub fn enforce_protected<T, F>(
    command: &str,
    payload: &InvokeBody,
    db: &AppDb,
    auth_state: &AuthState,
    authorization_state: &AuthorizationState,
    operation: F,
) -> Result<T, AppError>
where
    F: FnOnce() -> T,
{
    let boundary = authorization_state.boundary();
    let _boundary = boundary.lock().map_err(|_| {
        AppError::new(
            ErrorCode::Internal,
            "Authorization state is unavailable. Restart the application.",
        )
    })?;
    let principal = authorize_invoke(command, payload, db, auth_state)?.ok_or_else(forbidden)?;
    let _principal_scope = authorization_state.enter(principal)?;
    Ok(operation())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Public,
    Global,
    SelectedBranch,
    AuthorizedSet,
    CanonicalRecord,
    BothBranches,
    MembershipDirectory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub permission: &'static str,
    pub scope: Scope,
    pub write: bool,
}

fn unauthorized() -> AppError {
    AppError::new(
        ErrorCode::Unauthorized,
        "Authentication is required. Please sign in again.",
    )
}

fn forbidden() -> AppError {
    AppError::new(
        ErrorCode::Forbidden,
        "You do not have permission to perform this operation.",
    )
}

pub fn policy(command: &str) -> Option<Policy> {
    use Scope::*;
    let p = match command {
        "get_auth_status" | "bootstrap_admin" | "login" | "recovery_status" => Policy {
            permission: "public",
            scope: Public,
            write: matches!(command, "bootstrap_admin" | "login"),
        },
        "current_session" | "logout" => Policy {
            permission: "session.self",
            scope: Global,
            write: command == "logout",
        },
        "list_users" | "get_user" => Policy {
            permission: "users.read",
            scope: Global,
            write: false,
        },
        "create_user"
        | "update_user"
        | "set_user_active"
        | "reset_user_password"
        | "set_user_branches" => Policy {
            permission: "users.manage",
            scope: Global,
            write: true,
        },
        "list_roles" => Policy {
            permission: "roles.read",
            scope: Global,
            write: false,
        },
        "assign_user_role" | "remove_user_role" => Policy {
            permission: "roles.manage",
            scope: Global,
            write: true,
        },

        "get_phase6_dashboard" => Policy {
            permission: "dashboard.read",
            scope: AuthorizedSet,
            write: false,
        },
        "get_phase6_reconciliation" | "get_sale_finance_trace" => Policy {
            permission: "reports.finance.read",
            scope: if command == "get_sale_finance_trace" {
                CanonicalRecord
            } else {
                AuthorizedSet
            },
            write: false,
        },
        "export_phase6_csv" => Policy {
            permission: "reports.export",
            scope: AuthorizedSet,
            write: false,
        },

        "list_finance_accounts" => Policy {
            permission: "finance.read",
            scope: Global,
            write: false,
        },
        "create_finance_expense" => Policy {
            permission: "finance.expense.manage",
            scope: AuthorizedSet,
            write: true,
        },
        "get_finance_expense" | "get_finance_journal" => Policy {
            permission: "finance.read",
            scope: CanonicalRecord,
            write: false,
        },
        "list_finance_expenses"
        | "list_finance_journals"
        | "get_trial_balance"
        | "get_account_activity"
        | "get_sales_financial_summary"
        | "get_expense_report" => Policy {
            permission: "finance.read",
            scope: AuthorizedSet,
            write: false,
        },

        "get_reporting_overview"
        | "get_sales_report"
        | "get_sales_by_product_report"
        | "get_sales_by_customer_report"
        | "get_inventory_stock_report"
        | "get_inventory_movement_report"
        | "get_expiry_report"
        | "get_low_stock_report"
        | "get_purchasing_report"
        | "get_purchase_receipt_report"
        | "get_purchase_return_report"
        | "get_supplier_analytics" => Policy {
            permission: "reports.operational.read",
            scope: AuthorizedSet,
            write: false,
        },

        "create_purchase_order_draft"
        | "update_purchase_order_draft"
        | "order_purchase_order"
        | "cancel_purchase_order" => Policy {
            permission: "purchasing.manage",
            scope: Global,
            write: true,
        },
        "get_purchase_order" | "list_purchase_orders" => Policy {
            permission: "purchasing.read",
            scope: Global,
            write: false,
        },
        "search_purchase_packages" => Policy {
            permission: "catalog.read",
            scope: Global,
            write: false,
        },
        "receive_purchase_order" => Policy {
            permission: "purchasing.receive",
            scope: SelectedBranch,
            write: true,
        },
        "get_purchase_receipt" | "get_purchase_return" => Policy {
            permission: "purchasing.read",
            scope: CanonicalRecord,
            write: false,
        },
        "list_purchase_receipts" | "list_purchase_returns" => Policy {
            permission: "purchasing.read",
            scope: AuthorizedSet,
            write: false,
        },
        "create_purchase_return" => Policy {
            permission: "purchasing.return",
            scope: CanonicalRecord,
            write: true,
        },

        "create_canonical_supplier"
        | "update_canonical_supplier"
        | "set_canonical_supplier_active" => Policy {
            permission: "suppliers.manage",
            scope: Global,
            write: true,
        },
        "get_canonical_supplier" | "list_canonical_suppliers" => Policy {
            permission: "suppliers.read",
            scope: Global,
            write: false,
        },
        "create_canonical_customer"
        | "update_canonical_customer"
        | "set_canonical_customer_active" => Policy {
            permission: "customers.manage",
            scope: Global,
            write: true,
        },
        "get_canonical_customer" | "list_canonical_customers" => Policy {
            permission: "customers.read",
            scope: Global,
            write: false,
        },

        "search_pos_products" | "get_pos_package_availability" => Policy {
            permission: "sales.read",
            scope: SelectedBranch,
            write: false,
        },
        "save_sale_draft" | "preview_sale_totals" => Policy {
            permission: "sales.create",
            scope: SelectedBranch,
            write: command == "save_sale_draft",
        },
        "hold_sale" | "resume_sale" | "post_sale" => Policy {
            permission: "sales.create",
            scope: CanonicalRecord,
            write: true,
        },
        "list_canonical_sales" | "list_customer_sales" => Policy {
            permission: "sales.read",
            scope: AuthorizedSet,
            write: false,
        },
        "get_canonical_sale" => Policy {
            permission: "sales.read",
            scope: CanonicalRecord,
            write: false,
        },
        "preview_sale_return" | "create_sale_return" | "post_sale_return" => Policy {
            permission: "sales.return",
            scope: CanonicalRecord,
            write: command != "preview_sale_return",
        },
        "void_canonical_sale" => Policy {
            permission: "sales.void",
            scope: CanonicalRecord,
            write: true,
        },

        "inspect_inventory_overview"
        | "inspect_inventory_batches"
        | "inspect_inventory_movements"
        | "list_inventory_alerts"
        | "list_expiry_inventory"
        | "list_low_stock_inventory"
        | "list_count_sessions"
        | "list_inventory_documents" => Policy {
            permission: "inventory.read",
            scope: AuthorizedSet,
            write: false,
        },
        "get_inventory_alert_settings"
        | "get_fefo_inventory_batches"
        | "list_workflow_batches"
        | "fefo_allocation"
        | "get_package_stock"
        | "get_batch_stock"
        | "list_branch_inventory"
        | "list_stock_movements"
        | "list_low_stock"
        | "list_expiring_batches"
        | "get_fefo_batches" => Policy {
            permission: "inventory.read",
            scope: SelectedBranch,
            write: false,
        },
        "get_inventory_alert" | "get_count_session" | "get_inventory_document" => Policy {
            permission: "inventory.read",
            scope: CanonicalRecord,
            write: false,
        },
        "update_inventory_alert_settings" | "evaluate_inventory_alerts" => Policy {
            permission: "inventory.alert.manage",
            scope: SelectedBranch,
            write: true,
        },
        "dismiss_inventory_alert" => Policy {
            permission: "inventory.alert.manage",
            scope: CanonicalRecord,
            write: true,
        },
        "create_count_session" => Policy {
            permission: "inventory.count",
            scope: SelectedBranch,
            write: true,
        },
        "start_count_session"
        | "record_count_entry"
        | "discover_count_stock"
        | "review_count_session"
        | "resume_count_session"
        | "cancel_count_session"
        | "complete_count_session" => Policy {
            permission: "inventory.count",
            scope: CanonicalRecord,
            write: true,
        },
        "save_inventory_document" => Policy {
            permission: "inventory.adjust",
            scope: BothBranches,
            write: true,
        },
        "mark_inventory_document_ready"
        | "cancel_inventory_document"
        | "post_inventory_document"
        | "reverse_inventory_document" => Policy {
            permission: "inventory.adjust",
            scope: CanonicalRecord,
            write: true,
        },
        "create_inventory_batch" => Policy {
            permission: "inventory.receive",
            scope: SelectedBranch,
            write: true,
        },
        "record_opening_stock"
        | "record_batch_opening_stock"
        | "record_adjustment"
        | "record_damage"
        | "record_expiry_write_off"
        | "set_inventory_level" => Policy {
            permission: "inventory.adjust",
            scope: SelectedBranch,
            write: true,
        },
        "transfer_inventory_stock" => Policy {
            permission: "inventory.adjust",
            scope: BothBranches,
            write: true,
        },

        "profile_catalog_source"
        | "dry_run_catalog_import"
        | "get_catalog_import_status"
        | "get_catalog_import_report" => Policy {
            permission: "catalog.import",
            scope: Global,
            write: false,
        },
        "apply_catalog_import" => Policy {
            permission: "catalog.import",
            scope: Global,
            write: true,
        },

        "ensure_default_branch" | "create_branch" => Policy {
            permission: "branches.manage",
            scope: Global,
            write: true,
        },
        "list_branches" => Policy {
            permission: "branches.read",
            scope: MembershipDirectory,
            write: false,
        },
        "get_branch" => Policy {
            permission: "branches.read",
            scope: CanonicalRecord,
            write: false,
        },

        "list_manufacturers"
        | "list_categories"
        | "list_routes"
        | "list_active_ingredients"
        | "get_product"
        | "get_product_detail"
        | "list_products"
        | "list_product_packages"
        | "list_package_barcodes"
        | "get_current_package_price"
        | "get_package_price_history" => Policy {
            permission: "catalog.read",
            scope: Global,
            write: false,
        },
        "create_manufacturer"
        | "set_manufacturer_active"
        | "create_category"
        | "set_category_active"
        | "create_route"
        | "set_route_active"
        | "create_active_ingredient"
        | "set_active_ingredient_active"
        | "create_product"
        | "create_product_full"
        | "update_product_full"
        | "set_product_active"
        | "link_product_ingredient"
        | "create_product_package"
        | "set_product_package_active"
        | "add_barcode"
        | "set_package_price" => Policy {
            permission: "catalog.manage",
            scope: Global,
            write: true,
        },
        _ => return None,
    };
    Some(p)
}

pub fn role_grants(role: &str, permission: &str) -> bool {
    if role == "ADMIN" {
        return true;
    }
    match role {
        "PHARMACIST" => matches!(
            permission,
            "branches.read"
                | "catalog.read"
                | "inventory.read"
                | "inventory.adjust"
                | "inventory.alert.manage"
                | "sales.read"
                | "sales.create"
                | "sales.return"
                | "customers.read"
                | "customers.manage"
                | "reports.operational.read"
                | "dashboard.read"
        ),
        "CASHIER" => matches!(
            permission,
            "branches.read"
                | "catalog.read"
                | "inventory.read"
                | "sales.read"
                | "sales.create"
                | "sales.return"
                | "customers.read"
                | "customers.manage"
                | "dashboard.read"
        ),
        "INVENTORY_MANAGER" => matches!(
            permission,
            "branches.read"
                | "catalog.read"
                | "catalog.manage"
                | "catalog.import"
                | "inventory.read"
                | "inventory.adjust"
                | "inventory.receive"
                | "inventory.count"
                | "inventory.alert.manage"
                | "purchasing.read"
                | "purchasing.manage"
                | "purchasing.receive"
                | "purchasing.return"
                | "suppliers.read"
                | "suppliers.manage"
                | "reports.operational.read"
                | "dashboard.read"
        ),
        "VIEWER" => matches!(
            permission,
            "branches.read"
                | "catalog.read"
                | "inventory.read"
                | "sales.read"
                | "purchasing.read"
                | "suppliers.read"
                | "customers.read"
                | "reports.operational.read"
                | "dashboard.read"
        ),
        _ => false,
    }
}

fn is_admin(principal: &Principal) -> bool {
    principal.roles.iter().any(|role| role == "ADMIN")
}

fn has_permission(principal: &Principal, permission: &str) -> bool {
    permission == "session.self"
        || principal
            .roles
            .iter()
            .any(|role| role_grants(role, permission))
}

fn collect_named(value: &Value, names: &[&str], output: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if names.iter().any(|name| key == name) {
                    if let Some(text) = value.as_str().filter(|text| !text.is_empty()) {
                        output.push(text.to_owned());
                    }
                }
                collect_named(value, names, output);
            }
        }
        Value::Array(values) => values
            .iter()
            .for_each(|value| collect_named(value, names, output)),
        _ => {}
    }
}

fn strings(value: &Value, names: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    collect_named(value, names, &mut values);
    values.sort();
    values.dedup();
    values
}

fn first(value: &Value, names: &[&str]) -> Option<String> {
    strings(value, names).into_iter().next()
}

fn paths(value: &Value, pointers: &[&str]) -> Vec<String> {
    let mut values: Vec<String> = pointers
        .iter()
        .filter_map(|pointer| value.pointer(pointer))
        .filter_map(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect();
    values.sort();
    values.dedup();
    values
}

fn request_branches(command: &str, payload: &Value) -> Vec<String> {
    match command {
        "get_phase6_dashboard" | "get_phase6_reconciliation" => {
            paths(payload, &["/filter/branchId"])
        }
        "export_phase6_csv" => {
            let kind = payload.pointer("/input/kind").and_then(Value::as_str);
            match kind {
                Some("SALES" | "INVENTORY_STOCK") => {
                    paths(payload, &["/input/reportFilter/branchId"])
                }
                Some("EXPENSES" | "TRIAL_BALANCE" | "ACCOUNT_ACTIVITY") => {
                    paths(payload, &["/input/financeQuery/branchId"])
                }
                _ => Vec::new(),
            }
        }
        "get_reporting_overview"
        | "get_sales_report"
        | "get_sales_by_product_report"
        | "get_sales_by_customer_report"
        | "get_inventory_stock_report"
        | "get_inventory_movement_report"
        | "get_expiry_report"
        | "get_low_stock_report"
        | "get_purchasing_report"
        | "get_purchase_receipt_report"
        | "get_purchase_return_report"
        | "get_supplier_analytics" => paths(payload, &["/filter/branchId"]),
        "list_finance_expenses"
        | "list_finance_journals"
        | "get_trial_balance"
        | "get_account_activity"
        | "get_sales_financial_summary"
        | "get_expense_report"
        | "list_canonical_sales"
        | "list_customer_sales"
        | "list_purchase_receipts"
        | "list_purchase_returns"
        | "inspect_inventory_overview"
        | "inspect_inventory_batches"
        | "inspect_inventory_movements"
        | "list_inventory_alerts"
        | "list_expiry_inventory"
        | "list_low_stock_inventory"
        | "list_count_sessions"
        | "list_inventory_documents" => paths(payload, &["/query/branchId"]),
        "search_pos_products" => paths(payload, &["/query/branchId"]),
        "preview_sale_totals" => paths(payload, &["/cart/branchId"]),
        "get_pos_package_availability"
        | "get_inventory_alert_settings"
        | "evaluate_inventory_alerts"
        | "get_fefo_inventory_batches"
        | "list_workflow_batches"
        | "fefo_allocation"
        | "get_package_stock"
        | "get_batch_stock" => paths(payload, &["/branchId"]),
        "list_branch_inventory"
        | "list_stock_movements"
        | "list_low_stock"
        | "list_expiring_batches"
        | "get_fefo_batches" => paths(payload, &["/query/branchId", "/branchId"]),
        "save_sale_draft" => paths(payload, &["/input/cart/branchId"]),
        "create_inventory_batch"
        | "receive_purchase_order"
        | "create_finance_expense"
        | "update_inventory_alert_settings"
        | "create_count_session"
        | "record_batch_opening_stock"
        | "record_damage"
        | "record_expiry_write_off"
        | "set_inventory_level" => paths(payload, &["/input/branchId"]),
        "record_opening_stock" => paths(payload, &["/input/batch/branchId"]),
        "record_adjustment" => paths(payload, &["/input/operation/branchId"]),
        "transfer_inventory_stock" => paths(
            payload,
            &["/input/sourceBranchId", "/input/destinationBranchId"],
        ),
        "save_inventory_document" => {
            paths(payload, &["/input/branchId", "/input/destinationBranchId"])
        }
        _ => Vec::new(),
    }
}

fn verify_actor(value: &Value, principal: &Principal) -> Result<(), AppError> {
    let actors = strings(
        value,
        &[
            "createdBy",
            "created_by",
            "postedBy",
            "posted_by",
            "voidedBy",
            "voided_by",
            "performedBy",
            "performed_by",
            "assignedBy",
            "assigned_by",
            "user",
        ],
    );
    if actors.iter().any(|actor| actor != &principal.user_id) {
        return Err(forbidden());
    }
    Ok(())
}

fn member(principal: &Principal, branch: &str) -> bool {
    is_admin(principal) || principal.branch_ids.iter().any(|id| id == branch)
}

fn require_branches(principal: &Principal, branches: &[String]) -> Result<(), AppError> {
    if branches.is_empty() || branches.iter().any(|branch| !member(principal, branch)) {
        return Err(forbidden());
    }
    Ok(())
}

fn query_branch(c: &Connection, table: &str, id: &str) -> Result<Vec<String>, AppError> {
    let sql = format!("SELECT branch_id FROM {table} WHERE id=?1");
    let branch = c
        .query_row(&sql, [id], |row| row.get::<_, Option<String>>(0))
        .optional()?;
    match branch {
        Some(Some(value)) => Ok(vec![value]),
        Some(None) => Ok(Vec::new()),
        None => Err(forbidden()),
    }
}

fn record_branches(command: &str, payload: &Value, db: &AppDb) -> Result<Vec<String>, AppError> {
    if command == "get_branch" {
        return first(payload, &["id"])
            .map(|id| vec![id])
            .ok_or_else(forbidden);
    }
    let c = db.lock()?;
    match command {
        "get_sale_finance_trace"
        | "get_canonical_sale"
        | "hold_sale"
        | "resume_sale"
        | "post_sale"
        | "void_canonical_sale" => first(payload, &["saleId", "id"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "sales", &id)),
        "preview_sale_return" | "create_sale_return" => first(payload, &["saleId"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "sales", &id)),
        "post_sale_return" => first(payload, &["id", "returnId"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "sale_returns", &id)),
        "get_purchase_receipt" => first(payload, &["id", "receiptId"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "purchase_receipts", &id)),
        "create_purchase_return" => first(payload, &["receiptId"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "purchase_receipts", &id)),
        "get_purchase_return" => first(payload, &["id", "returnId"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "purchase_returns", &id)),
        "get_finance_expense" => first(payload, &["id"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "finance_expenses", &id)),
        "get_finance_journal" => first(payload, &["id"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "finance_journals", &id)),
        "get_inventory_alert" | "dismiss_inventory_alert" => first(payload, &["id", "alertId"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "inventory_alerts", &id)),
        "get_count_session"
        | "start_count_session"
        | "record_count_entry"
        | "discover_count_stock"
        | "review_count_session"
        | "resume_count_session"
        | "cancel_count_session"
        | "complete_count_session" => first(payload, &["sessionId", "id"])
            .ok_or_else(forbidden)
            .and_then(|id| query_branch(&c, "stock_count_sessions", &id)),
        "get_inventory_document"
        | "mark_inventory_document_ready"
        | "cancel_inventory_document"
        | "post_inventory_document"
        | "reverse_inventory_document" => {
            let id = first(payload, &["documentId", "id"]).ok_or_else(forbidden)?;
            let mut branches = query_branch(&c, "inventory_documents", &id)?;
            let destination = c
                .query_row(
                    "SELECT destination_branch_id FROM inventory_documents WHERE id=?1",
                    [&id],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()?
                .flatten();
            if let Some(destination) = destination {
                branches.push(destination);
            }
            Ok(branches)
        }
        _ => Err(forbidden()),
    }
}

pub fn authorize_json(
    command: &str,
    payload: &Value,
    db: &AppDb,
    state: &AuthState,
) -> Result<Option<Principal>, AppError> {
    let policy = policy(command).ok_or_else(forbidden)?;
    if policy.scope == Scope::Public {
        return Ok(None);
    }
    let token = payload
        .get("sessionToken")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(unauthorized)?;
    let principal = auth::principal(db, state, token).map_err(|_| unauthorized())?;
    if !has_permission(&principal, policy.permission) {
        return Err(forbidden());
    }
    if policy.write {
        verify_actor(payload, &principal)?;
    }
    let selected = request_branches(command, payload);
    match policy.scope {
        Scope::Public | Scope::Global => {}
        Scope::SelectedBranch => require_branches(&principal, &selected)?,
        Scope::BothBranches => {
            let expected = if command == "save_inventory_document" {
                if payload.pointer("/input/kind").and_then(Value::as_str) == Some("TRANSFER") {
                    2
                } else {
                    1
                }
            } else {
                2
            };
            if selected.len() < expected {
                return Err(forbidden());
            }
            require_branches(&principal, &selected)?;
        }
        Scope::AuthorizedSet => {
            if selected.is_empty() {
                if !is_admin(&principal) {
                    return Err(forbidden());
                }
            } else {
                require_branches(&principal, &selected)?;
            }
        }
        Scope::CanonicalRecord => {
            let branches = record_branches(command, payload, db)?;
            if branches.is_empty() {
                if !is_admin(&principal) {
                    return Err(forbidden());
                }
            } else {
                require_branches(&principal, &branches)?;
            }
        }
        Scope::MembershipDirectory => {}
    }
    Ok(Some(principal))
}

pub fn authorize_invoke(
    command: &str,
    body: &InvokeBody,
    db: &AppDb,
    state: &AuthState,
) -> Result<Option<Principal>, AppError> {
    match body {
        InvokeBody::Json(value) => authorize_json(command, value, db, state),
        InvokeBody::Raw(_) => Err(forbidden()),
    }
}
