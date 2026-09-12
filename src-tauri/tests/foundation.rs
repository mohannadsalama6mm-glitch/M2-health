use m2_health_lib::{
    db::{
        connection::AppDb,
        migrations::{self, Migration},
        models::branch::CreateBranch,
        repositories::branches::*,
    },
    error::ErrorCode,
};
use rusqlite::Connection;

fn input(code: &str) -> CreateBranch {
    CreateBranch {
        code: code.into(),
        name: "Test Pharmacy".into(),
        phone: "".into(),
        address: "".into(),
    }
}

#[test]
fn fresh_database_initializes_schema_and_pragmas() {
    let db = AppDb::in_memory().unwrap();
    let c = db.lock().unwrap();
    let version: i64 = c
        .query_row("SELECT max(version) FROM _schema_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);
    let foreign_keys: i64 = c
        .pragma_query_value(None, "foreign_keys", |r| r.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 1);
    let timeout: i64 = c
        .pragma_query_value(None, "busy_timeout", |r| r.get(0))
        .unwrap();
    assert_eq!(timeout, 5000);
    let count: i64 = c
        .query_row("SELECT count(*) FROM branches", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
#[test]
fn migrations_are_repeatable_and_leave_two_versions() {
    let db = AppDb::in_memory().unwrap();
    let mut c = db.lock().unwrap();
    migrations::run(&mut c).unwrap();
    migrations::run(&mut c).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
}
#[test]
fn migration_failure_rolls_back_schema_and_metadata_without_losing_data() {
    let db = AppDb::in_memory().unwrap();
    let branch = ensure_default_branch(&db).unwrap();
    {
        let mut c = db.lock().unwrap();
        let failing = [
            Migration {
                version: 1,
                name: "branches",
                sql: "",
            },
            Migration {
                version: 2,
                name: "catalog_core",
                sql: "",
            },
            Migration {
                version: 3,
                name: "broken",
                sql: "CREATE TABLE should_rollback(id TEXT); INSERT INTO missing_table VALUES (1);",
            },
        ];
        assert!(migrations::apply(&mut c, &failing).is_err());
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='should_rollback'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }
    assert_eq!(get_branch(&db, &branch.id).unwrap().id, branch.id);
}
#[test]
fn newer_schema_is_refused_without_reset() {
    let db = AppDb::in_memory().unwrap();
    let mut c = db.lock().unwrap();
    c.execute(
        "INSERT INTO _schema_version(version,name) VALUES(3,'future')",
        [],
    )
    .unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
}
#[test]
fn create_list_get_and_safe_errors() {
    let db = AppDb::in_memory().unwrap();
    let branch = create_branch(&db, &input(" north ")).unwrap();
    assert!(uuid::Uuid::parse_str(&branch.id).is_ok());
    assert_eq!(branch.code, "NORTH");
    assert!(branch.is_active);
    assert!(branch.created_at.ends_with('Z'));
    assert_eq!(branch.created_at, branch.updated_at);
    assert_eq!(list_branches(&db).unwrap().len(), 1);
    assert_eq!(get_branch(&db, &branch.id).unwrap().name, "Test Pharmacy");
    assert_eq!(
        create_branch(&db, &input("NORTH")).unwrap_err().code,
        ErrorCode::Conflict
    );
    assert_eq!(
        create_branch(&db, &input("")).unwrap_err().code,
        ErrorCode::Validation
    );
    assert_eq!(
        get_branch(&db, "bad-id").unwrap_err().code,
        ErrorCode::Validation
    );
    assert_eq!(
        get_branch(&db, &uuid::Uuid::new_v4().to_string())
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let dto = serde_json::to_value(branch).unwrap();
    assert!(dto.get("isActive").is_some());
    assert!(dto.get("createdAt").is_some());
    assert!(dto.get("is_active").is_none());
}
#[test]
fn default_is_idempotent_and_respects_existing_branches() {
    let db = AppDb::in_memory().unwrap();
    let first = ensure_default_branch(&db).unwrap();
    assert_eq!(first.code, "MAIN");
    assert_eq!(first.name, "Main Pharmacy");
    for _ in 0..10 {
        assert_eq!(ensure_default_branch(&db).unwrap().id, first.id);
    }
    assert_eq!(list_branches(&db).unwrap().len(), 1);
    let other = AppDb::in_memory().unwrap();
    let existing = create_branch(&other, &input("CUSTOM")).unwrap();
    assert_eq!(ensure_default_branch(&other).unwrap().id, existing.id);
    assert_eq!(list_branches(&other).unwrap().len(), 1);
}
#[test]
fn active_branch_is_preferred_without_reactivating_inactive_records() {
    let db = AppDb::in_memory().unwrap();
    let first = create_branch(&db, &input("OLD")).unwrap();
    db.lock()
        .unwrap()
        .execute("UPDATE branches SET is_active=0 WHERE id=?1", [&first.id])
        .unwrap();
    assert!(!ensure_default_branch(&db).unwrap().is_active);
    let second = create_branch(&db, &input("NEW")).unwrap();
    assert_eq!(ensure_default_branch(&db).unwrap().id, second.id);
    assert_eq!(list_branches(&db).unwrap().len(), 2);
}
#[test]
fn file_database_reopens_with_wal_and_stable_branch() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("isolated.db");
    let id;
    {
        let db = AppDb::open(&path).unwrap();
        id = ensure_default_branch(&db).unwrap().id;
        assert_eq!(
            db.lock()
                .unwrap()
                .pragma_query_value(None, "journal_mode", |r| r.get::<_, String>(0))
                .unwrap(),
            "wal"
        );
    }
    let reopened = AppDb::open(&path).unwrap();
    assert_eq!(ensure_default_branch(&reopened).unwrap().id, id);
    assert_eq!(list_branches(&reopened).unwrap().len(), 1);
}
#[test]
fn simultaneous_default_calls_create_only_one_record() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("race.db");
    let one = AppDb::open(&path).unwrap();
    let two = AppDb::open(&path).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let a = barrier.clone();
    let b = barrier.clone();
    let first = std::thread::spawn(move || {
        a.wait();
        ensure_default_branch(&one).unwrap().id
    });
    let second = std::thread::spawn(move || {
        b.wait();
        ensure_default_branch(&two).unwrap().id
    });
    assert_eq!(first.join().unwrap(), second.join().unwrap());
    assert_eq!(
        list_branches(&AppDb::open(&path).unwrap()).unwrap().len(),
        1
    );
}
#[test]
fn foreign_keys_are_enforced() {
    let db = AppDb::in_memory().unwrap();
    let c = db.lock().unwrap();
    c.execute_batch("CREATE TABLE test_children(branch_id TEXT REFERENCES branches(id));")
        .unwrap();
    assert!(c
        .execute("INSERT INTO test_children VALUES ('missing')", [])
        .is_err());
}
#[test]
fn invalid_database_file_is_not_recreated() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("not-sqlite.db");
    std::fs::write(&path, b"retain this content").unwrap();
    assert!(AppDb::open(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"retain this content");
}
#[test]
fn fresh_failed_migration_rolls_back_metadata_creation() {
    let mut c = Connection::open_in_memory().unwrap();
    assert!(migrations::apply(
        &mut c,
        &[Migration {
            version: 1,
            name: "bad",
            sql: "INVALID SQL"
        }]
    )
    .is_err());
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='_schema_version'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}
