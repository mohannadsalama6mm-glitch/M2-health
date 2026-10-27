use m2_health_lib::{
    auth::{self, models::*, AuthState},
    db::{
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        repositories::branches,
    },
};
use std::{
    sync::{Arc, Barrier},
    thread,
};

fn bootstrap(db: &AppDb, state: &AuthState) -> LoginResult {
    auth::bootstrap(
        db,
        state,
        &BootstrapInput {
            username: "admin.local".into(),
            display_name: "Local Administrator".into(),
            password: "VeryStrong!Pass2026".into(),
        },
    )
    .unwrap()
}
fn user_input(username: &str, roles: &[&str], branches_: Vec<String>) -> CreateUser {
    CreateUser {
        username: username.into(),
        display_name: format!("User {username}"),
        password: "Another!Strong2026".into(),
        role_codes: roles.iter().map(|v| v.to_string()).collect(),
        primary_branch_id: branches_.first().cloned(),
        branch_ids: branches_,
    }
}

#[test]
fn migration_016_creates_identity_foundation_without_users_or_credentials() {
    let db = AppDb::in_memory().unwrap();
    let c = db.lock().unwrap();
    assert_eq!(
        c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        MIGRATIONS.len() as i64
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM roles", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        5
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM users", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM user_credentials", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        0
    );
}

#[test]
fn migration_017_adds_actor_attribution_and_immutable_structured_security_events() {
    let db = AppDb::in_memory_with_migrations(&MIGRATIONS[..16]).unwrap();
    {
        let c = db.lock().unwrap();
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='security_events'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
    }
    {
        let mut c = db.lock().unwrap();
        migrations::apply(&mut c, &MIGRATIONS[..17]).unwrap();
        assert_eq!(
            c.query_row("SELECT max(version) FROM _schema_version", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            17
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM security_events", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM pragma_table_info('inventory_batches') WHERE name='created_by'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM pragma_table_info('sale_returns') WHERE name='created_by'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM users", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
            0
        );
    }
}

#[test]
fn identity_security_events_are_atomic_structured_and_secret_free() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    let branch = branches::ensure_default_branch(&db).unwrap();
    let created = auth::create(
        &db,
        &user_input("audited.user", &[], vec![]),
        &admin.user.id,
    )
    .unwrap();
    let updated = auth::update(
        &db,
        &UpdateUser {
            id: created.id.clone(),
            expected_revision: created.revision,
            username: "audited.user".into(),
            display_name: "Audited User".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    auth::reset_password(
        &db,
        &ResetPassword {
            id: created.id.clone(),
            password: "Replacement!Secret2026".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    auth::assign_role(
        &db,
        &UserRoleInput {
            user_id: created.id.clone(),
            role_code: "VIEWER".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    auth::set_branches(
        &db,
        &SetUserBranches {
            user_id: created.id.clone(),
            branch_ids: vec![branch.id.clone()],
            primary_branch_id: Some(branch.id),
        },
        &admin.user.id,
    )
    .unwrap();
    auth::remove_role(
        &db,
        &UserRoleInput {
            user_id: created.id.clone(),
            role_code: "VIEWER".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    auth::set_active(
        &db,
        &SetUserActive {
            id: created.id.clone(),
            expected_revision: updated.revision + 1,
            active: false,
        },
        &admin.user.id,
    )
    .unwrap();

    let c = db.lock().unwrap();
    let rows = c
        .prepare("SELECT actor_user_id,subject_user_id,event_type,detail FROM security_events WHERE subject_user_id=?1 ORDER BY created_at,id")
        .unwrap()
        .query_map([&created.id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows.len(), 7);
    assert!(rows
        .iter()
        .all(|row| row.0 == admin.user.id && row.1 == created.id));
    let kinds = rows
        .iter()
        .map(|row| row.2.as_str())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        kinds,
        [
            "USER_CREATED",
            "USER_UPDATED",
            "PASSWORD_RESET",
            "ROLE_ASSIGNED",
            "BRANCH_MEMBERSHIP_CHANGED",
            "ROLE_REMOVED",
            "USER_DEACTIVATED",
        ]
        .into_iter()
        .collect()
    );
    let serialized = rows
        .iter()
        .map(|row| row.3.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!serialized.contains("Replacement!Secret2026"));
    assert!(!serialized.contains(&admin.token));
    assert!(c
        .execute("UPDATE security_events SET detail='changed'", [])
        .is_err());
    assert!(c.execute("DELETE FROM security_events", []).is_err());
}

#[test]
fn failed_security_event_rolls_back_the_privileged_identity_change() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    let worker = auth::create(
        &db,
        &user_input("rollback.user", &["VIEWER"], vec![]),
        &admin.user.id,
    )
    .unwrap();
    {
        let c = db.lock().unwrap();
        c.execute_batch("CREATE TRIGGER fail_security_event BEFORE INSERT ON security_events BEGIN SELECT RAISE(ABORT,'injected_security_event_failure'); END;")
            .unwrap();
    }
    assert!(auth::set_active(
        &db,
        &SetUserActive {
            id: worker.id.clone(),
            expected_revision: worker.revision,
            active: false,
        },
        &admin.user.id,
    )
    .is_err());
    let reloaded = auth::user(&db, &worker.id).unwrap();
    assert!(reloaded.is_active);
    assert_eq!(reloaded.revision, worker.revision);
}

#[test]
fn bootstrap_is_one_time_atomic_and_stores_only_argon2id() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    assert!(!auth::status(&db).unwrap().initialized);
    let first = bootstrap(&db, &state);
    assert!(auth::status(&db).unwrap().initialized);
    assert!(first.user.roles.iter().any(|r| r.code == "ADMIN"));
    assert!(auth::bootstrap(
        &db,
        &state,
        &BootstrapInput {
            username: "other.admin".into(),
            display_name: "Other".into(),
            password: "OtherStrong!Pass2026".into()
        }
    )
    .is_err());
    let c = db.lock().unwrap();
    let hash: String = c
        .query_row("SELECT password_hash FROM user_credentials", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
    assert!(!hash.contains("VeryStrong!Pass2026"));
    assert_eq!(
        c.query_row("SELECT count(*) FROM users", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn concurrent_bootstrap_creates_exactly_one_initial_admin() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.db");
    drop(AppDb::open(&path).unwrap());
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = vec![];
    for n in 0..2 {
        let p = path.clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            let db = AppDb::open(&p).unwrap();
            let state = AuthState::default();
            b.wait();
            auth::bootstrap(
                &db,
                &state,
                &BootstrapInput {
                    username: format!("admin{n}"),
                    display_name: format!("Admin {n}"),
                    password: "Concurrent!Pass2026".into(),
                },
            )
            .is_ok()
        }));
    }
    let successes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(|succeeded| *succeeded)
        .count();
    assert_eq!(successes, 1);
    let db = AppDb::open(&path).unwrap();
    let c = db.lock().unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM users", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM auth_bootstrap", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn login_is_generic_throttled_and_restart_clears_sessions() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let first = bootstrap(&db, &state);
    state.logout(&first.token);
    assert!(auth::principal(&db, &state, &first.token).is_err());
    for name in ["admin.local", "missing.user"] {
        let error = auth::authenticate_password(
            &db,
            &state,
            &LoginInput {
                username: name.into(),
                password: "wrong-password".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.message, "Invalid username or password.");
    }
    for _ in 0..4 {
        let _ = auth::authenticate_password(
            &db,
            &state,
            &LoginInput {
                username: "admin.local".into(),
                password: "wrong-password".into(),
            },
        );
    }
    assert_eq!(
        auth::authenticate_password(
            &db,
            &state,
            &LoginInput {
                username: "admin.local".into(),
                password: "VeryStrong!Pass2026".into()
            }
        )
        .unwrap_err()
        .message,
        "Invalid username or password."
    );
    let restarted = AuthState::default();
    assert!(auth::principal(&db, &restarted, &first.token).is_err());
    let login = auth::authenticate_password(
        &db,
        &restarted,
        &LoginInput {
            username: "ADMIN.LOCAL".into(),
            password: "VeryStrong!Pass2026".into(),
        },
    )
    .unwrap();
    assert!(login.expires_at > 0);
    assert!(auth::principal(&db, &restarted, &login.token).is_ok());
}

#[test]
fn users_support_search_filters_pagination_stale_updates_and_unique_salts() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    let branch = branches::ensure_default_branch(&db).unwrap();
    let one = auth::create(
        &db,
        &user_input("pharmacist.one", &["PHARMACIST"], vec![branch.id.clone()]),
        &admin.user.id,
    )
    .unwrap();
    let two = auth::create(
        &db,
        &user_input("viewer.two", &["VIEWER"], vec![]),
        &admin.user.id,
    )
    .unwrap();
    let page = auth::users(
        &db,
        &UserQuery {
            search: Some("pharmacist".into()),
            role_code: Some("PHARMACIST".into()),
            limit: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].id, one.id);
    assert_eq!(page.items[0].branches[0].branch_id, branch.id);
    let updated = auth::update(
        &db,
        &UpdateUser {
            id: two.id.clone(),
            expected_revision: two.revision,
            username: "viewer.renamed".into(),
            display_name: "Renamed Viewer".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    assert_eq!(updated.revision, two.revision + 1);
    assert!(auth::update(
        &db,
        &UpdateUser {
            id: two.id.clone(),
            expected_revision: two.revision,
            username: "viewer.stale".into(),
            display_name: "Stale".into()
        },
        &admin.user.id,
    )
    .is_err());
    let c = db.lock().unwrap();
    let hashes = c
        .prepare("SELECT password_hash FROM user_credentials ORDER BY user_id")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(hashes.len(), 3);
    assert_ne!(hashes[0], hashes[1]);
}

#[test]
fn password_reset_and_deactivation_revoke_sessions() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    let created = auth::create(
        &db,
        &user_input("worker.user", &["PHARMACIST"], vec![]),
        &admin.user.id,
    )
    .unwrap();
    let login = auth::authenticate_password(
        &db,
        &state,
        &LoginInput {
            username: "worker.user".into(),
            password: "Another!Strong2026".into(),
        },
    )
    .unwrap();
    auth::reset_password(
        &db,
        &ResetPassword {
            id: created.id.clone(),
            password: "Replacement!Pass2026".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    state.revoke_user(&created.id);
    assert!(auth::principal(&db, &state, &login.token).is_err());
    assert!(auth::authenticate_password(
        &db,
        &state,
        &LoginInput {
            username: "worker.user".into(),
            password: "Another!Strong2026".into()
        }
    )
    .is_err());
    let login = auth::authenticate_password(
        &db,
        &state,
        &LoginInput {
            username: "worker.user".into(),
            password: "Replacement!Pass2026".into(),
        },
    )
    .unwrap();
    let current = auth::user(&db, &created.id).unwrap();
    auth::set_active(
        &db,
        &SetUserActive {
            id: created.id.clone(),
            expected_revision: current.revision,
            active: false,
        },
        &admin.user.id,
    )
    .unwrap();
    state.revoke_user(&created.id);
    assert!(auth::principal(&db, &state, &login.token).is_err());
    assert_eq!(
        auth::authenticate_password(
            &db,
            &AuthState::default(),
            &LoginInput {
                username: "worker.user".into(),
                password: "Replacement!Pass2026".into()
            }
        )
        .unwrap_err()
        .message,
        "Invalid username or password."
    );
}

#[test]
fn final_active_admin_and_system_roles_are_protected() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    assert!(auth::set_active(
        &db,
        &SetUserActive {
            id: admin.user.id.clone(),
            expected_revision: admin.user.revision,
            active: false
        },
        &admin.user.id
    )
    .is_err());
    assert!(auth::remove_role(
        &db,
        &UserRoleInput {
            user_id: admin.user.id.clone(),
            role_code: "ADMIN".into()
        },
        &admin.user.id
    )
    .is_err());
    let c = db.lock().unwrap();
    assert!(c
        .execute("UPDATE roles SET name='Owner' WHERE code='ADMIN'", [])
        .is_err());
    assert!(c
        .execute("DELETE FROM users WHERE id=?1", [admin.user.id])
        .is_err());
}

#[test]
fn role_and_branch_assignments_validate_duplicates_and_final_admin_transfer() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    let branch = branches::ensure_default_branch(&db).unwrap();
    let second = auth::create(
        &db,
        &user_input("second.admin", &[], vec![]),
        &admin.user.id,
    )
    .unwrap();
    let second = auth::assign_role(
        &db,
        &UserRoleInput {
            user_id: second.id.clone(),
            role_code: "ADMIN".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    assert!(auth::assign_role(
        &db,
        &UserRoleInput {
            user_id: second.id.clone(),
            role_code: "ADMIN".into()
        },
        &admin.user.id
    )
    .is_err());
    auth::remove_role(
        &db,
        &UserRoleInput {
            user_id: admin.user.id.clone(),
            role_code: "ADMIN".into(),
        },
        &admin.user.id,
    )
    .unwrap();
    let second = auth::set_branches(
        &db,
        &SetUserBranches {
            user_id: second.id.clone(),
            branch_ids: vec![branch.id.clone()],
            primary_branch_id: Some(branch.id.clone()),
        },
        &second.id,
    )
    .unwrap();
    assert!(second.branches[0].is_primary);
    assert!(auth::set_branches(
        &db,
        &SetUserBranches {
            user_id: second.id,
            branch_ids: vec![],
            primary_branch_id: Some(branch.id)
        },
        &admin.user.id
    )
    .is_err());
}

#[test]
fn administration_requires_a_valid_admin_session() {
    let db = AppDb::in_memory().unwrap();
    let state = AuthState::default();
    let admin = bootstrap(&db, &state);
    let worker = auth::create(
        &db,
        &user_input("plain.viewer", &["VIEWER"], vec![]),
        &admin.user.id,
    )
    .unwrap();
    let login = auth::authenticate_password(
        &db,
        &state,
        &LoginInput {
            username: worker.username,
            password: "Another!Strong2026".into(),
        },
    )
    .unwrap();
    assert!(auth::require_admin(&db, &state, &login.token).is_err());
    assert!(auth::require_admin(&db, &state, &admin.token).is_ok());
    assert!(auth::principal(&db, &state, "not-a-session").is_err());
}
