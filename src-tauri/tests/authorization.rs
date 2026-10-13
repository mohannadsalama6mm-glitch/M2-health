use m2_health_lib::{
    auth::{self, models::*, AuthState},
    authorization::{
        authorize_json, enforce_protected, policy, role_grants, AuthorizationState, Scope,
    },
    db::{connection::AppDb, models::branch::CreateBranch, repositories::branches},
    error::ErrorCode,
};
use serde_json::json;
use std::collections::HashSet;
use std::sync::{mpsc, Arc};
use tauri::ipc::InvokeBody;

fn setup() -> (AppDb, AuthState, LoginResult, String, String) {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = auth::bootstrap(
        &db,
        &state,
        &BootstrapInput {
            username: "admin.local".into(),
            display_name: "Admin".into(),
            password: "VeryStrong!Pass2026".into(),
        },
    )
    .unwrap();
    let first = branches::ensure_default_branch(&db).unwrap();
    let second = branches::create_branch(
        &db,
        &CreateBranch {
            code: "SECOND".into(),
            name: "Second Branch".into(),
            phone: String::new(),
            address: String::new(),
        },
    )
    .unwrap();
    let viewer = auth::create(
        &db,
        &CreateUser {
            username: "viewer.local".into(),
            display_name: "Viewer".into(),
            password: "Viewer!Strong2026".into(),
            role_codes: vec!["VIEWER".into()],
            branch_ids: vec![first.id.clone()],
            primary_branch_id: Some(first.id.clone()),
        },
        &admin.user.id,
    )
    .unwrap();
    let viewer_login = auth::authenticate_password(
        &db,
        &state,
        &LoginInput {
            username: viewer.username,
            password: "Viewer!Strong2026".into(),
        },
    )
    .unwrap();
    (db, state, admin, viewer_login.token, second.id)
}

fn registered_commands() -> Vec<&'static str> {
    let source = include_str!("../src/lib.rs");
    source
        .split("tauri::generate_handler![")
        .nth(1)
        .unwrap()
        .split("]))")
        .next()
        .unwrap()
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.contains("::")
                .then(|| line.rsplit("::").next().unwrap())
        })
        .collect()
}

#[test]
fn every_registered_command_has_exactly_one_explicit_policy() {
    let source = include_str!("../src/lib.rs");
    let commands = registered_commands();
    assert_eq!(commands.len(), 169);
    assert_eq!(commands.iter().copied().collect::<HashSet<_>>().len(), 169);
    for command in commands {
        assert!(
            policy(command).is_some(),
            "unclassified IPC command: {command}"
        );
    }
    assert!(policy("legacy_unregistered_bypass").is_none());
    assert_eq!(source.matches(".invoke_handler(").count(), 1);
    assert_eq!(source.matches("tauri::generate_handler![").count(), 1);
    assert!(source.contains(".invoke_handler(authorization::guard(tauri::generate_handler!["));
}

#[test]
fn public_surface_is_limited_and_unknown_commands_fail_closed() {
    let expected = [
        "get_auth_status",
        "bootstrap_admin",
        "login",
        "recovery_status",
    ];
    for command in expected {
        assert_eq!(policy(command).unwrap().scope, Scope::Public);
    }
    let actual: HashSet<_> = registered_commands()
        .into_iter()
        .filter(|command| policy(command).is_some_and(|value| value.scope == Scope::Public))
        .collect();
    assert_eq!(actual, expected.into_iter().collect());
    for command in [
        "list_products",
        "list_users",
        "list_branches",
        "get_phase6_dashboard",
    ] {
        assert_ne!(policy(command).unwrap().scope, Scope::Public);
    }
}

#[test]
fn raw_ipc_bodies_and_disabled_users_fail_before_domain_access() {
    let (db, state, _admin, viewer, _) = setup();
    let raw = m2_health_lib::authorization::authorize_invoke(
        "list_products",
        &InvokeBody::Raw(vec![1, 2, 3]),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(raw.code, ErrorCode::Forbidden);

    let principal = auth::principal(&db, &state, &viewer).unwrap();
    db.lock()
        .unwrap()
        .execute(
            "UPDATE users SET is_active=0 WHERE id=?1",
            [&principal.user_id],
        )
        .unwrap();
    let disabled = authorize_json(
        "list_products",
        &json!({"sessionToken": viewer}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(disabled.code, ErrorCode::Unauthorized);
}

#[test]
fn durable_role_and_membership_changes_take_effect_without_trusting_session_snapshots() {
    let (db, state, _admin, viewer, _) = setup();
    let principal = auth::principal(&db, &state, &viewer).unwrap();
    let branch = principal.branch_ids[0].clone();
    authorize_json(
        "list_branch_inventory",
        &json!({"sessionToken": viewer, "branchId": branch}),
        &db,
        &state,
    )
    .unwrap();
    {
        let connection = db.lock().unwrap();
        connection
            .execute(
                "DELETE FROM user_branch_memberships WHERE user_id=?1",
                [&principal.user_id],
            )
            .unwrap();
    }
    let branch_denied = authorize_json(
        "list_branch_inventory",
        &json!({"sessionToken": viewer, "branchId": branch}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(branch_denied.code, ErrorCode::Forbidden);
    {
        let connection = db.lock().unwrap();
        connection
            .execute(
                "DELETE FROM user_roles WHERE user_id=?1",
                [&principal.user_id],
            )
            .unwrap();
    }
    let role_denied = authorize_json(
        "list_products",
        &json!({"sessionToken": viewer}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(role_denied.code, ErrorCode::Forbidden);
}

#[test]
fn non_admin_identity_mutation_is_denied_without_writes() {
    let (db, state, _admin, viewer, _) = setup();
    let before: i64 = db
        .lock()
        .unwrap()
        .query_row("SELECT count(*) FROM users", [], |row| row.get(0))
        .unwrap();
    let denied = authorize_json(
        "create_user",
        &json!({"sessionToken": viewer, "input": {"username": "bypass", "displayName": "Bypass", "password": "Bypass!Password2026", "roleCodes": ["ADMIN"], "branchIds": []}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(denied.code, ErrorCode::Forbidden);
    let after: i64 = db
        .lock()
        .unwrap()
        .query_row("SELECT count(*) FROM users", [], |row| row.get(0))
        .unwrap();
    assert_eq!(after, before);
}

#[test]
fn branch_owned_batch_opening_and_adjustment_paths_cannot_be_bypassed() {
    let (db, state, admin, viewer, other_branch) = setup();
    let own_branch = auth::principal(&db, &state, &viewer).unwrap().branch_ids[0].clone();
    let inventory_user = auth::create(
        &db,
        &CreateUser {
            username: "inventory.security".into(),
            display_name: "Inventory Security".into(),
            password: "Inventory!Secure2026".into(),
            role_codes: vec!["INVENTORY_MANAGER".into()],
            branch_ids: vec![own_branch.clone()],
            primary_branch_id: Some(own_branch.clone()),
        },
        &admin.user.id,
    )
    .unwrap();
    let login = auth::authenticate_password(
        &db,
        &state,
        &LoginInput {
            username: inventory_user.username,
            password: "Inventory!Secure2026".into(),
        },
    )
    .unwrap();
    let token = login.token;
    let actor = login.user.id;

    authorize_json(
        "create_inventory_batch",
        &json!({"sessionToken": token, "input": {"branchId": own_branch}}),
        &db,
        &state,
    )
    .unwrap();
    let batch_denied = authorize_json(
        "create_inventory_batch",
        &json!({"sessionToken": token, "input": {"branchId": other_branch}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(batch_denied.code, ErrorCode::Forbidden);

    authorize_json(
        "record_opening_stock",
        &json!({"sessionToken": token, "input": {"batch": {"branchId": own_branch}, "createdBy": actor}}),
        &db,
        &state,
    )
    .unwrap();
    let opening_denied = authorize_json(
        "record_opening_stock",
        &json!({"sessionToken": token, "input": {"branchId": own_branch, "batch": {"branchId": other_branch}, "createdBy": actor}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(opening_denied.code, ErrorCode::Forbidden);

    authorize_json(
        "record_adjustment",
        &json!({"sessionToken": token, "input": {"operation": {"branchId": own_branch, "createdBy": actor}}}),
        &db,
        &state,
    )
    .unwrap();
    let adjustment_denied = authorize_json(
        "record_adjustment",
        &json!({"sessionToken": token, "input": {"branchId": own_branch, "operation": {"branchId": other_branch, "createdBy": actor}}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(adjustment_denied.code, ErrorCode::Forbidden);
}

#[test]
fn admin_can_create_organization_expense_but_non_admin_cannot_bypass_finance() {
    let (db, state, admin, viewer, _) = setup();
    authorize_json(
        "create_finance_expense",
        &json!({"sessionToken": admin.token, "input": {"branchId": null, "createdBy": admin.user.id}}),
        &db,
        &state,
    )
    .unwrap();
    let denied = authorize_json(
        "create_finance_expense",
        &json!({"sessionToken": viewer, "input": {"branchId": null}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(denied.code, ErrorCode::Forbidden);
}

#[test]
fn missing_session_role_and_branch_fail_before_domain_access() {
    let (db, state, _admin, viewer, other_branch) = setup();
    let missing = authorize_json("list_products", &json!({}), &db, &state).unwrap_err();
    assert_eq!(missing.code, ErrorCode::Unauthorized);

    let finance = authorize_json(
        "list_finance_accounts",
        &json!({"sessionToken": viewer}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(finance.code, ErrorCode::Forbidden);

    let denied = authorize_json(
        "list_branch_inventory",
        &json!({"sessionToken": viewer, "branchId": other_branch}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(denied.code, ErrorCode::Forbidden);
}

#[test]
fn authorized_branch_and_admin_organization_scope_are_explicit() {
    let (db, state, admin, viewer, _) = setup();
    let own_branch = auth::principal(&db, &state, &viewer).unwrap().branch_ids[0].clone();
    authorize_json(
        "list_branch_inventory",
        &json!({"sessionToken": viewer, "branchId": own_branch}),
        &db,
        &state,
    )
    .unwrap();
    let null_scope = authorize_json(
        "get_reporting_overview",
        &json!({"sessionToken": viewer, "filter": {"branchId": null}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(null_scope.code, ErrorCode::Forbidden);
    authorize_json(
        "get_reporting_overview",
        &json!({"sessionToken": admin.token, "filter": {"branchId": null}}),
        &db,
        &state,
    )
    .unwrap();
}

#[test]
fn forged_actor_is_rejected_and_authenticated_actor_is_accepted() {
    let (db, state, admin, _, _) = setup();
    let branch = branches::ensure_default_branch(&db).unwrap().id;
    let forged = authorize_json(
        "create_finance_expense",
        &json!({"sessionToken": admin.token, "input": {"branchId": branch, "createdBy": "forged-user"}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(forged.code, ErrorCode::Forbidden);
    authorize_json(
        "create_finance_expense",
        &json!({"sessionToken": admin.token, "input": {"branchId": branch, "createdBy": admin.user.id}}),
        &db,
        &state,
    )
    .unwrap();
}

#[test]
fn viewer_is_read_only_and_finance_remains_admin_only() {
    assert!(role_grants("VIEWER", "catalog.read"));
    assert!(role_grants("VIEWER", "inventory.read"));
    assert!(!role_grants("VIEWER", "inventory.adjust"));
    assert!(!role_grants("VIEWER", "finance.read"));
    assert!(!role_grants("CASHIER", "reports.export"));
    assert!(!role_grants("PHARMACIST", "sales.void"));
    assert!(role_grants("INVENTORY_MANAGER", "purchasing.receive"));
    assert!(role_grants("ADMIN", "anything.explicit"));
}

#[test]
fn protected_dispatch_boundary_denies_before_operation_and_allows_authorized_operation() {
    let (db, state, _admin, viewer, other_branch) = setup();
    let boundary = AuthorizationState::default();
    let mut mutations = 0;
    let denied = enforce_protected(
        "create_branch",
        &InvokeBody::Json(
            json!({"sessionToken": viewer, "input": {"code": "FORGED", "name": "Forged"}}),
        ),
        &db,
        &state,
        &boundary,
        || mutations += 1,
    )
    .unwrap_err();
    assert_eq!(denied.code, ErrorCode::Forbidden);
    assert_eq!(mutations, 0, "denial must not enter the command operation");

    let own_branch = auth::principal(&db, &state, &viewer).unwrap().branch_ids[0].clone();
    enforce_protected(
        "list_branch_inventory",
        &InvokeBody::Json(json!({"sessionToken": viewer, "branchId": own_branch})),
        &db,
        &state,
        &boundary,
        || mutations += 1,
    )
    .unwrap();
    assert_eq!(mutations, 1);

    let denied = enforce_protected(
        "list_branch_inventory",
        &InvokeBody::Json(json!({"sessionToken": viewer, "branchId": other_branch})),
        &db,
        &state,
        &boundary,
        || mutations += 1,
    )
    .unwrap_err();
    assert_eq!(denied.code, ErrorCode::Forbidden);
    assert_eq!(mutations, 1);
}

#[test]
fn invalid_and_revoked_sessions_are_unauthorized_at_dispatch() {
    let (db, state, _admin, viewer, _) = setup();
    let invalid = authorize_json(
        "list_products",
        &json!({"sessionToken": "0".repeat(64)}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(invalid.code, ErrorCode::Unauthorized);

    state.logout(&viewer);
    let revoked = authorize_json(
        "list_products",
        &json!({"sessionToken": viewer}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(revoked.code, ErrorCode::Unauthorized);
}

#[test]
fn irrelevant_branch_fields_cannot_bypass_all_branch_or_export_scope() {
    let (db, state, _admin, viewer, _) = setup();
    let own_branch = auth::principal(&db, &state, &viewer).unwrap().branch_ids[0].clone();
    let all_branch = authorize_json(
        "get_reporting_overview",
        &json!({"sessionToken": viewer, "branchId": own_branch, "filter": {"branchId": null}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(all_branch.code, ErrorCode::Forbidden);

    let export = authorize_json(
        "export_phase6_csv",
        &json!({"sessionToken": viewer, "input": {"kind": "SALES", "reportFilter": {"branchId": own_branch}}}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(export.code, ErrorCode::Forbidden);
}

#[test]
fn canonical_sale_ids_resolve_persisted_branch_and_finance_requires_permission() {
    let (db, state, admin, viewer, other_branch) = setup();
    let sale_id = uuid::Uuid::new_v4().to_string();
    let journal_id = uuid::Uuid::new_v4().to_string();
    {
        let connection = db.lock().unwrap();
        connection
            .execute(
                "INSERT INTO sales(id,branch_id,status,draft_json) VALUES(?1,?2,'DRAFT','{}')",
                rusqlite::params![sale_id, other_branch],
            )
            .unwrap();
        connection.execute(
            "INSERT INTO finance_journals(id,journal_number,source_type,source_id,branch_id,description,debit_total_minor,credit_total_minor) VALUES(?1,'JRN-AUTH-1','SALE',?2,?3,'Authorization fixture',1,1)",
            rusqlite::params![journal_id, sale_id, other_branch],
        ).unwrap();
    }
    for (command, payload) in [
        (
            "get_canonical_sale",
            json!({"sessionToken": viewer, "id": sale_id.clone()}),
        ),
        (
            "get_sale_finance_trace",
            json!({"sessionToken": viewer, "saleId": sale_id}),
        ),
        (
            "get_finance_journal",
            json!({"sessionToken": viewer, "id": journal_id}),
        ),
    ] {
        let error = authorize_json(command, &payload, &db, &state).unwrap_err();
        assert_eq!(error.code, ErrorCode::Forbidden);
    }
    let unknown = authorize_json(
        "get_finance_journal",
        &json!({"sessionToken": admin.token, "id": uuid::Uuid::new_v4().to_string()}),
        &db,
        &state,
    )
    .unwrap_err();
    assert_eq!(unknown.code, ErrorCode::Forbidden);
}

#[test]
fn protected_dispatch_serializes_role_change_with_in_flight_sync_operation() {
    let (db, state, admin, viewer, _) = setup();
    let own_branch = auth::principal(&db, &state, &viewer).unwrap().branch_ids[0].clone();
    let db = Arc::new(db);
    let state = Arc::new(state);
    let boundary = AuthorizationState::default();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (changed_tx, changed_rx) = mpsc::channel();

    let read = {
        let db = Arc::clone(&db);
        let state = Arc::clone(&state);
        let boundary = boundary.clone();
        std::thread::spawn(move || {
            enforce_protected(
                "list_branch_inventory",
                &InvokeBody::Json(json!({"sessionToken": viewer, "branchId": own_branch})),
                &db,
                &state,
                &boundary,
                || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                },
            )
            .unwrap()
        })
    };
    entered_rx.recv().unwrap();
    let change = {
        let db = Arc::clone(&db);
        let state = Arc::clone(&state);
        let boundary = boundary.clone();
        std::thread::spawn(move || {
            enforce_protected(
            "assign_user_role",
            &InvokeBody::Json(json!({"sessionToken": admin.token, "input": {"userId": admin.user.id, "roleCode": "ADMIN"}})),
            &db,
            &state,
            &boundary,
            || changed_tx.send(()).unwrap(),
        ).unwrap()
        })
    };
    assert!(matches!(
        changed_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    release_tx.send(()).unwrap();
    read.join().unwrap();
    change.join().unwrap();
    changed_rx.recv().unwrap();
}

#[test]
fn protected_dispatch_exposes_only_the_validated_principal_during_execution() {
    let (db, state, admin, _, _) = setup();
    let boundary = AuthorizationState::default();
    let expected = admin.user.id.clone();
    let observed = enforce_protected(
        "create_finance_expense",
        &InvokeBody::Json(json!({
            "sessionToken": admin.token,
            "input": {"branchId": null}
        })),
        &db,
        &state,
        &boundary,
        || boundary.actor().unwrap(),
    )
    .unwrap();
    assert_eq!(observed, expected);
    assert_eq!(boundary.actor().unwrap_err().code, ErrorCode::Unauthorized);
}

#[test]
fn logout_waits_for_an_in_flight_protected_request_then_revokes_the_session() {
    let (db, state, _admin, viewer, _) = setup();
    let branch = auth::principal(&db, &state, &viewer).unwrap().branch_ids[0].clone();
    let db = Arc::new(db);
    let state = Arc::new(state);
    let boundary = AuthorizationState::default();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let active = {
        let db = Arc::clone(&db);
        let state = Arc::clone(&state);
        let boundary = boundary.clone();
        let token = viewer.clone();
        std::thread::spawn(move || {
            enforce_protected(
                "list_branch_inventory",
                &InvokeBody::Json(json!({"sessionToken": token, "branchId": branch})),
                &db,
                &state,
                &boundary,
                || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                },
            )
            .unwrap();
        })
    };
    entered_rx.recv().unwrap();
    let logout = {
        let db = Arc::clone(&db);
        let state = Arc::clone(&state);
        let boundary = boundary.clone();
        let token = viewer.clone();
        let revoke_token = viewer.clone();
        std::thread::spawn(move || {
            enforce_protected(
                "logout",
                &InvokeBody::Json(json!({"sessionToken": token})),
                &db,
                &state,
                &boundary,
                || state.logout(&revoke_token),
            )
            .unwrap();
        })
    };
    assert!(logout.is_finished() == false);
    release_tx.send(()).unwrap();
    active.join().unwrap();
    logout.join().unwrap();
    assert_eq!(
        authorize_json(
            "list_products",
            &json!({"sessionToken": viewer}),
            &db,
            &state,
        )
        .unwrap_err()
        .code,
        ErrorCode::Unauthorized
    );
}

#[test]
fn every_async_registered_command_is_read_only_and_dispatch_time_authorized() {
    let source = include_str!("../src/catalog_import/commands.rs");
    let async_commands = [
        "profile_catalog_source",
        "dry_run_catalog_import",
        "get_catalog_import_report",
    ];
    assert_eq!(
        source.matches("pub async fn ").count(),
        async_commands.len()
    );
    for command in async_commands {
        assert!(source.contains(&format!("pub async fn {command}")));
        let policy = policy(command).unwrap();
        assert!(!policy.write, "async mutation is not permitted: {command}");
        assert_ne!(policy.scope, Scope::Public);
    }
    assert!(source.contains("pub fn apply_catalog_import"));
    assert!(policy("apply_catalog_import").unwrap().write);
}

#[test]
fn actor_bearing_command_adapters_source_identity_from_authorization_state() {
    for source in [
        include_str!("../src/finance/commands.rs"),
        include_str!("../src/purchasing/commands.rs"),
        include_str!("../src/inventory/commands.rs"),
        include_str!("../src/inventory/documents/commands.rs"),
        include_str!("../src/inventory/counts/commands.rs"),
        include_str!("../src/sales/commands.rs"),
    ] {
        assert!(source.contains("authorization.actor()?"));
    }
}
